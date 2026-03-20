//! Criterion benchmark for the MySQL-backed shortener service.
//!
//! Measures the throughput of `ShortenerService::shorten()` against a real
//! MySQL instance started in a Docker container via `wormhole-test-infra`.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sqlx::mysql::MySqlPoolOptions;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};
use wormhole_generator::seq::SeqGenerator;
use wormhole_shortener::service::ShortenerService;
use wormhole_shortener::shortener::{ExpirationPolicy, ShortenParams, Shortener};
use wormhole_storage::MySqlRepository;
use wormhole_test_infra::mysql::{MySqlServer, MysqlConfig};

const DEFAULT_BATCH_SIZE: usize = 8192;
const DEFAULT_WORKER_THREADS: usize = 16;
const DEFAULT_MAX_CONNECTIONS: u32 = 64;
const DEFAULT_MIN_CONNECTIONS: u32 = 16;
const MYSQL_READY_GRACE_PERIOD: Duration = Duration::from_secs(5);

type BenchService = ShortenerService<MySqlRepository, SeqGenerator>;

// ==============================================================================
// Benchmark Bootstrap
// ==============================================================================
//
// Criterion expects a synchronous entrypoint, but the workload itself is async
// and depends on a disposable MySQL container. We build one runtime up front,
// keep the fixture alive for the full benchmark, and then explicitly tear it
// down inside the runtime again so async cleanup can finish cleanly.

fn shorten_mysql_qps_benchmark(c: &mut Criterion) {
    let batch_size = bench_usize_env("BENCH_BATCH_SIZE", DEFAULT_BATCH_SIZE);
    let worker_threads = bench_usize_env("BENCH_WORKER_THREADS", DEFAULT_WORKER_THREADS);
    let runtime = build_runtime(worker_threads);
    let environment = runtime.block_on(MySqlBenchEnvironment::new());

    let mut group = c.benchmark_group("shorten/mysql_qps");
    group.throughput(Throughput::Elements(batch_size as u64));
    group.bench_with_input(
        BenchmarkId::from_parameter(batch_size),
        &batch_size,
        |b, &batch_size| {
            b.iter_custom(|iters| {
                runtime.block_on(async {
                    let mut total = Duration::ZERO;

                    for _ in 0..iters {
                        environment.reset_short_urls().await;
                        let params = build_shorten_params(batch_size);

                        let start = Instant::now();
                        shorten_batch(&environment.service, params).await;
                        total += start.elapsed();
                    }

                    total
                })
            });
        },
    );
    group.finish();

    runtime.block_on(environment.shutdown());
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(15))
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = shorten_mysql_qps_benchmark
}
criterion_main!(benches);

fn build_runtime(worker_threads: usize) -> Runtime {
    Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .expect("build tokio runtime")
}

fn bench_usize_env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn build_shorten_params(batch_size: usize) -> Vec<ShortenParams> {
    (0..batch_size)
        .map(|i| ShortenParams {
            original_url: format!("https://example{}.com", i),
            expiration: ExpirationPolicy::Never,
            custom_alias: None,
        })
        .collect()
}

// ==============================================================================
// Shared Benchmark State
// ==============================================================================
//
// Criterion executes the benchmark closure many times. Reusing one MySQL
// container keeps startup costs out of the measurement, while truncating the
// table between samples keeps later samples from being dominated by index and
// table growth from earlier runs.

struct MySqlBenchEnvironment {
    _mysql: MySqlServer,
    pool: sqlx::MySqlPool,
    service: BenchService,
}

impl MySqlBenchEnvironment {
    async fn new() -> Self {
        let mysql = MySqlServer::new(MysqlConfig::builder().build())
            .await
            .expect("start mysql container");

        tokio::time::sleep(MYSQL_READY_GRACE_PERIOD).await;

        let database_url = mysql.database_url().await.expect("get mysql database url");

        // The pool is intentionally larger than the batch worker count so the
        // benchmark stresses repository throughput instead of queueing on the
        // client side.
        let pool = MySqlPoolOptions::new()
            .max_connections(DEFAULT_MAX_CONNECTIONS)
            .min_connections(DEFAULT_MIN_CONNECTIONS)
            .connect(&database_url)
            .await
            .expect("connect mysql");

        let repository = MySqlRepository::new(pool.clone());
        repository.migrate().await.expect("run mysql migrations");

        let generator = SeqGenerator::with_prefix("bench");
        let service = ShortenerService::new(repository, generator);

        Self {
            _mysql: mysql,
            pool,
            service,
        }
    }

    async fn reset_short_urls(&self) {
        sqlx::query("TRUNCATE TABLE short_urls")
            .execute(&self.pool)
            .await
            .expect("truncate short_urls");
    }

    async fn shutdown(self) {
        self.pool.close().await;
        drop(self);
    }
}

// ==============================================================================
// Timed Workload
// ==============================================================================
//
// The benchmark measures only the shorten calls themselves. Input generation
// and table resets happen outside the timed region so QPS reflects service and
// database behavior rather than fixture maintenance.

async fn shorten_batch(service: &BenchService, params: Vec<ShortenParams>) {
    let mut handles = Vec::with_capacity(params.len());

    for params in params {
        let service = service.clone();
        handles.push(tokio::spawn(async move { service.shorten(params).await }));
    }

    for handle in handles {
        handle
            .await
            .expect("join shorten task")
            .expect("shorten url");
    }
}
