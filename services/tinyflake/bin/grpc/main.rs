use clap::Parser;
use jiff::Timestamp;
use std::net::SocketAddr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic_health::server::HealthReporter;
use tracing::info;
use wormhole_telemetry::init_tracing;
use wormhole_tinyflake::{Tinyflake, TinyflakeSettings};

// Include generated gRPC stubs for the tinyflake service.
pub mod tinyflake {
    pub mod v1 {
        tonic::include_proto!("tinyflake.v1");
    }
}

use tinyflake::v1::tinyflake_service_server::{TinyflakeService, TinyflakeServiceServer};
use tinyflake::v1::{NextBatchRequest, NextBatchResponse, NextIdRequest, NextIdResponse};

// ==============================================================================
// CLI
// ==============================================================================

#[derive(Debug, Parser)]
struct Cli {
    /// gRPC listen address.
    #[arg(long, default_value = "0.0.0.0:50053", env = "TINYFLAKE_ADDR")]
    listen_addr: SocketAddr,

    /// Node ID in range [0, 3]. Must be unique across all tinyflake instances.
    #[arg(long, env = "TINYFLAKE_NODE_ID")]
    node_id: u8,

    /// Custom epoch as a RFC 3339 timestamp (e.g. 2024-01-01T00:00:00Z).
    /// All generated IDs embed seconds elapsed since this epoch.
    #[arg(long, env = "TINYFLAKE_START_EPOCH")]
    start_epoch: String,
}

// ==============================================================================
// gRPC service implementation
// ==============================================================================

struct TinyflakeServiceImpl {
    generator: Tinyflake<wormhole_tinyflake::SystemClock>,
}

#[tonic::async_trait]
impl TinyflakeService for TinyflakeServiceImpl {
    async fn next_id(
        &self,
        _request: Request<NextIdRequest>,
    ) -> Result<Response<NextIdResponse>, Status> {
        let tiny_id = self
            .generator
            .next_id()
            .map_err(|e| Status::internal(e.to_string()))?;

        // Zero-extend the 5-byte (40-bit) TinyId into a u64.
        let bytes = tiny_id.into_bytes();
        let mut buf = [0u8; 8];
        buf[..5].copy_from_slice(&bytes);
        let id = u64::from_le_bytes(buf);

        Ok(Response::new(NextIdResponse { id }))
    }

    async fn next_batch(
        &self,
        request: Request<NextBatchRequest>,
    ) -> Result<Response<NextBatchResponse>, Status> {
        let count = request.into_inner().count.min(256) as usize;

        let mut ids = Vec::with_capacity(count);
        for _ in 0..count {
            let tiny_id = self
                .generator
                .next_id()
                .map_err(|e| Status::internal(e.to_string()))?;

            let bytes = tiny_id.into_bytes();
            let mut buf = [0u8; 8];
            buf[..5].copy_from_slice(&bytes);
            ids.push(u64::from_le_bytes(buf));
        }

        Ok(Response::new(NextBatchResponse { ids }))
    }
}

// ==============================================================================
// Health reporter
// ==============================================================================

// Sets the tinyflake service as serving immediately. A more complete
// implementation would set NOT_SERVING during shutdown.
async fn set_serving(mut reporter: HealthReporter) {
    reporter
        .set_serving::<TinyflakeServiceServer<TinyflakeServiceImpl>>()
        .await;
}

// ==============================================================================
// Entry point
// ==============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _telemetry = init_tracing("wormhole-tinyflake")?;

    let cli = Cli::parse();

    let start_epoch = cli
        .start_epoch
        .parse::<Timestamp>()
        .map_err(|e| format!("invalid start epoch: {e}"))?;

    let settings = TinyflakeSettings::builder()
        .node_id(cli.node_id)
        .start_epoch(start_epoch)
        .build();

    let generator = Tinyflake::new(settings)?;

    info!(
        listen_addr = %cli.listen_addr,
        node_id = cli.node_id,
        "starting tinyflake gRPC service"
    );

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    tokio::spawn(set_serving(health_reporter.clone()));

    // health_reporter is unused after the spawn — suppress the warning.
    drop(health_reporter);

    let svc = TinyflakeServiceImpl { generator };

    Server::builder()
        .add_service(health_service)
        .add_service(TinyflakeServiceServer::new(svc))
        .serve(cli.listen_addr)
        .await?;

    Ok(())
}
