mod cli;

use crate::cli::CLI;
use clap::Parser;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use wormhole_cache::RedisUrlCache;
use wormhole_proto_schema::v1::redirector_service_server::RedirectorServiceServer;
use wormhole_redirector::grpc::RedirectorGrpcServer;
use wormhole_redirector::repository::CachedRepository;
use wormhole_redirector::service::RedirectorService;
use wormhole_storage::MySqlRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = CLI::parse();

    info!(
        listen_addr = %config.listen_addr,
        mysql_dsn = "[REDACTED]",
        redis_url = %config.redis_url,
        "starting redirector gRPC server"
    );

    // Create Redis cache connection
    let client = redis::Client::open(config.redis_url.as_str())?;
    let conn = client.get_multiplexed_async_connection().await?;
    let cache = RedisUrlCache::new(conn);

    // Create MySQL repository
    let inner = MySqlRepository::connect(&config.mysql_dsn).await?;
    // do the migration before starting the server
    inner.migrate().await?;

    // Wrap with caching layer
    let repository = CachedRepository::new(inner, cache);

    let service = RedirectorService::new(repository);
    let grpc_server = RedirectorGrpcServer::new(service);

    let (_, health_service) = tonic_health::server::health_reporter();

    Server::builder()
        .add_service(health_service)
        .add_service(RedirectorServiceServer::new(grpc_server))
        .serve(config.listen_addr)
        .await?;

    Ok(())
}
