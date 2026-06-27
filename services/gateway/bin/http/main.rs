//! Main entry point for the gateway HTTP server.
mod cli;

use wormhole_gateway::adapter::grpc::GrpcUrlAdapter;
use wormhole_gateway::app::App;
use wormhole_gateway::state::AppState;
use wormhole_telemetry::init_tracing;

use crate::cli::CLI;
use clap::Parser;
use tonic::transport::Endpoint;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry = init_tracing("wormhole-gateway")?;

    // Parse CLI arguments
    let config = CLI::parse();

    info!(
        listen_addr = %config.listen_addr,
        shortener_addr = %config.shortener_addr,
        redirector_addr = %config.redirector_addr,
        "starting gateway HTTP server"
    );

    // Create gRPC channels to remote services
    let shortener_channel = Endpoint::from_shared(config.shortener_addr.clone())?
        .connect()
        .await?;
    let redirector_channel = Endpoint::from_shared(config.redirector_addr.clone())?
        .connect()
        .await?;

    // Create gRPC clients
    let shortener_client =
        wormhole_proto_schema::v1::shortener_service_client::ShortenerServiceClient::new(
            shortener_channel,
        );
    let redirector_client =
        wormhole_proto_schema::v1::redirector_service_client::RedirectorServiceClient::new(
            redirector_channel,
        );

    // Create the adapter using the gRPC clients
    let adapter = GrpcUrlAdapter::builder()
        .shortener(shortener_client)
        .redirector(redirector_client)
        .build();

    // Create application state with the adapter
    let state = AppState::builder()
        .url_service(adapter)
        .base_url("https://worm.hole".to_string())
        .build();

    // Build and start the Axum router
    let app = App::router(state);

    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    info!(listen_addr = %listener.local_addr()?, "gateway HTTP server listening");

    axum::serve(listener, app).await?;

    Ok(())
}
