mod cli;
mod server;

use crate::cli::{StorageBackendArg, CLI};
use crate::server::ShortenerGrpcServer;
use clap::Parser;
use jiff::Timestamp;
use tonic::transport::Server;
use tracing::info;
use wormhole_generator::obfuscated::{ObfuscatedTinyFlake, Obfuscator};
use wormhole_generator::Generator;
use wormhole_proto_schema::v1::shortener_service_server::ShortenerServiceServer;
use wormhole_storage::{InMemoryRepository, MySqlRepository, Repository};
use wormhole_tinyflake::TinyflakeSettings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = CLI::try_parse()?;

    info!(
        listen_addr = %config.listen_addr,
        node_id = %config.node_id,
        storage_backend = %config.storage,
        "starting shortener gRPC server"
    );

    let obfuscator = Obfuscator::builder().build();
    // todo: make the start epoch configurable
    let start_epoch: Timestamp = "2026-01-01T00:00:00+08[Asia/Shanghai]".parse().unwrap();

    let tinyflake_settings = TinyflakeSettings::builder()
        .node_id(config.node_id)
        .start_epoch(start_epoch)
        .build();

    let generator = ObfuscatedTinyFlake::new(tinyflake_settings, obfuscator);

    match config.storage {
        StorageBackendArg::InMemory => {
            run_server(config.listen_addr, InMemoryRepository::new(), generator).await?;
        }
        StorageBackendArg::Mysql => {
            let mysql_dsn = config
                .mysql_dsn
                .ok_or("mysql dsn is required when storage backend is mysql")?;
            let repository = MySqlRepository::connect(&mysql_dsn).await?;
            run_server(config.listen_addr, repository, generator).await?;
        }
    }

    Ok(())
}

async fn run_server<R: Repository, G: Generator>(
    listen_addr: std::net::SocketAddr,
    repository: R,
    generator: G,
) -> Result<(), tonic::transport::Error> {
    let service = ShortenerGrpcServer::new(repository, generator);

    Server::builder()
        .add_service(ShortenerServiceServer::new(service))
        .serve(listen_addr)
        .await
}
