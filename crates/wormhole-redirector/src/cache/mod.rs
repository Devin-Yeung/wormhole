//! Cache implementations for the redirector service.

pub mod moka;
pub mod redis;

pub use self::moka::MokaUrlCache;
pub use redis::RedisUrlCache;
