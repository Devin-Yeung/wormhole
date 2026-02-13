//! Cache implementations for the redirector service.

pub mod layered;
pub mod moka;
pub mod redis;
pub mod redis_ha;

pub use self::moka::MokaUrlCache;
pub use layered::LayeredCache;
pub use redis::RedisUrlCache;
pub use redis_ha::RedisHAUrlCache;
