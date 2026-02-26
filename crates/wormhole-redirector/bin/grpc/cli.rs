use clap::Parser;
use std::net::SocketAddr;

pub const LISTEN_ADDR_ENV: &str = "WORMHOLE_REDIRECTOR_GRPC_LISTEN_ADDR";
pub const MYSQL_DSN_ENV: &str = "WORMHOLE_REDIRECTOR_MYSQL_DSN";
pub const REDIS_URL_ENV: &str = "WORMHOLE_REDIRECTOR_REDIS_URL";
pub const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:50052";

#[derive(Debug, Parser)]
#[command(name = "wormhole-redirector-grpc-server")]
pub struct CLI {
    #[arg(long, env = LISTEN_ADDR_ENV, default_value = DEFAULT_LISTEN_ADDR)]
    pub listen_addr: SocketAddr,

    #[arg(long, env = MYSQL_DSN_ENV)]
    /// MySQL DSN, e.g. "mysql://user:password@host:port/database"
    pub mysql_dsn: String,

    #[arg(long, env = REDIS_URL_ENV)]
    /// Redis URL, e.g. "redis://localhost:6379"
    pub redis_url: String,
}
