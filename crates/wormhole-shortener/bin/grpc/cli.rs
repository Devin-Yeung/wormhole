use clap::{Parser, ValueEnum};
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

pub const LISTEN_ADDR_ENV: &str = "WORMHOLE_SHORTENER_GRPC_LISTEN_ADDR";
pub const STORAGE_BACKEND_ENV: &str = "WORMHOLE_SHORTENER_STORAGE_BACKEND";
pub const MYSQL_DSN_ENV: &str = "WORMHOLE_SHORTENER_MYSQL_DSN";
pub const GENERATOR_NODE_ID: &str = "WORMHOLE_SHORTENER_GENERATOR_NODE_ID";
pub const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:50051";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StorageBackendArg {
    #[value(name = "in-memory")]
    InMemory,
    #[value(name = "mysql")]
    Mysql,
}

impl Display for StorageBackendArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackendArg::InMemory => write!(f, "in-memory"),
            StorageBackendArg::Mysql => write!(f, "mysql"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "wormhole-shortener-grpc-server")]
pub struct CLI {
    #[arg(long, env = LISTEN_ADDR_ENV, default_value = DEFAULT_LISTEN_ADDR)]
    pub listen_addr: SocketAddr,

    #[arg(long, env = GENERATOR_NODE_ID)]
    pub node_id: u8,

    #[arg(
        long,
        env = STORAGE_BACKEND_ENV,
        value_enum,
        default_value_t = StorageBackendArg::InMemory
    )]
    pub storage: StorageBackendArg,

    #[arg(long, env = MYSQL_DSN_ENV, required_if_eq("storage", "mysql"))]
    pub mysql_dsn: Option<String>,
}
