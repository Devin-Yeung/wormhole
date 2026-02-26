//! Command-line interface for the gateway HTTP server.

use clap::Parser;
use std::net::SocketAddr;

pub const LISTEN_ADDR_ENV: &str = "WORMHOLE_GATEWAY_LISTEN_ADDR";
pub const SHORTENER_ADDR_ENV: &str = "WORMHOLE_GATEWAY_SHORTENER_ADDR";
pub const REDIRECTOR_ADDR_ENV: &str = "WORMHOLE_GATEWAY_REDIRECTOR_ADDR";
pub const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:8080";
pub const DEFAULT_SHORTENER_ADDR: &str = "http://127.0.0.1:50051";
pub const DEFAULT_REDIRECTOR_ADDR: &str = "http://127.0.0.1:50052";

#[derive(Debug, Parser)]
#[command(name = "wormhole-gateway")]
/// HTTP gateway server for the URL shortener service.
pub struct CLI {
    #[arg(long, env = LISTEN_ADDR_ENV, default_value = DEFAULT_LISTEN_ADDR)]
    /// Socket address to listen on, e.g., "127.0.0.1:8080"
    pub listen_addr: SocketAddr,

    #[arg(long, env = SHORTENER_ADDR_ENV, default_value = DEFAULT_SHORTENER_ADDR)]
    /// gRPC address for the shortener service, e.g., "http://127.0.0.1:50051"
    pub shortener_addr: String,

    #[arg(long, env = REDIRECTOR_ADDR_ENV, default_value = DEFAULT_REDIRECTOR_ADDR)]
    /// gRPC address for the redirector service, e.g., "http://127.0.0.1:50052"
    pub redirector_addr: String,
}
