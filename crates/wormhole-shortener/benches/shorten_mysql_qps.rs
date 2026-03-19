//! QPS benchmark for the MySQL-backed shortener service.
//!
//! Measures throughput of `ShortenerService::shorten()` against a real MySQL
//! instance started in a Docker container via `wormhole-test-infra`.

use std::time::Instant;
use tokio::runtime::Builder;
use wormhole_generator::seq::SeqGenerator;
use wormhole_shortener::service::ShortenerService;
use wormhole_shortener::shortener::{ExpirationPolicy, ShortenParams, Shortener};
use wormhole_storage::MySqlRepository;
use wormhole_test_infra::mysql::{MySqlServer, MysqlConfig};

const DEFAULT_BATCH_SIZE: usize = 10_000;

fn main() {
    let batch_size = std::env::var("BENCH_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_BATCH_SIZE);

    // Multi-threaded runtime with enough workers to drive concurrent I/O.
    // The entire benchmark runs inside this runtime so that:
    //  - MySQL container async teardown executes in a live runtime context
    //  - All database operations are non-blocking
    let rt = Builder::new_multi_thread()
        .worker_threads(16)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        bench_mysql(batch_size).await;
    });
}

async fn bench_mysql(batch_size: usize) {
    // Spin up a disposable MySQL container.
    let mysql = MySqlServer::new(MysqlConfig::builder().build())
        .await
        .expect("start mysql container");

    // Give MySQL extra time to finish initializing before connecting.
    // The container reports "ready for connections" before it actually accepts them.
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let db_url = mysql
        .database_url()
        .await
        .expect("failed to get database URL");

    // Connection pool sized to saturate MySQL with concurrent connections.
    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(64)
        .min_connections(16)
        .connect(&db_url)
        .await
        .expect("connect mysql");

    let repo = MySqlRepository::new(pool);
    repo.migrate().await.expect("run migrations");

    let generator = SeqGenerator::with_prefix("bench");
    let service = ShortenerService::new(repo, generator);

    // Pre-generate all parameters so measurement covers only the shorten calls.
    // Clone upfront to give the async block owned values (required for 'static).
    let params: Vec<_> = (0..batch_size)
        .map(|i| ShortenParams {
            original_url: format!("https://example{}.com", i),
            expiration: ExpirationPolicy::Never,
            custom_alias: None,
        })
        .collect();

    let start = Instant::now();
    let handles: Vec<_> = params
        .into_iter()
        .map(|p| {
            let svc = service.clone();
            tokio::spawn(async move { svc.shorten(p).await })
        })
        .collect();

    for h in handles {
        let _ = h.await.unwrap();
    }
    let elapsed = start.elapsed();

    let qps = batch_size as f64 / elapsed.as_secs_f64();
    println!(
        "[mysql] batch_size={} elapsed={:.2}s qps={:.0}",
        batch_size,
        elapsed.as_secs_f64(),
        qps
    );
}
