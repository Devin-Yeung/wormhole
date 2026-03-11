//! Main entry point for the gateway HTTP server.
mod cli;

use wormhole_gateway::adapter::grpc::GrpcUrlAdapter;
use wormhole_gateway::app::App;
use wormhole_gateway::state::AppState;

use crate::cli::CLI;
use clap::Parser;
use tonic::transport::Endpoint;
use tracing::dispatcher::set_global_default;
use tracing::info;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    LogTracer::init().expect("Failed to set logger");
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    let fmt_layer = tracing_subscriber::fmt::layer().json();

    // Initialize tracing with JSON formatting and env filter
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);

    set_global_default(subscriber.into())?;

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
