mod cli;
mod server;

use crate::cli::{StorageBackendArg, CLI};
use crate::server::ShortenerGrpcServer;
use clap::Parser;
use tonic::transport::Server;
use tracing::info;
use wormhole_generator::seq::SeqGenerator;
use wormhole_proto_schema::v1::shortener_service_server::ShortenerServiceServer;
use wormhole_storage::{InMemoryRepository, MySqlRepository, Repository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = CLI::try_parse()?;

    info!(
        listen_addr = %config.listen_addr,
        generator_prefix = %config.generator_prefix,
        storage_backend = %config.storage,
        "starting shortener gRPC server"
    );

    match config.storage {
        StorageBackendArg::InMemory => {
            run_server(
                config.listen_addr,
                InMemoryRepository::new(),
                SeqGenerator::with_prefix(config.generator_prefix),
            )
            .await?;
        }
        StorageBackendArg::Mysql => {
            let mysql_dsn = config
                .mysql_dsn
                .ok_or("mysql dsn is required when storage backend is mysql")?;
            let repository = MySqlRepository::connect(&mysql_dsn).await?;
            run_server(
                config.listen_addr,
                repository,
                SeqGenerator::with_prefix(config.generator_prefix),
            )
            .await?;
        }
    }

    Ok(())
}

async fn run_server<R: Repository>(
    listen_addr: std::net::SocketAddr,
    repository: R,
    generator: SeqGenerator,
) -> Result<(), tonic::transport::Error> {
    let service = ShortenerGrpcServer::new(repository, generator);

    Server::builder()
        .add_service(ShortenerServiceServer::new(service))
        .serve(listen_addr)
        .await
}
