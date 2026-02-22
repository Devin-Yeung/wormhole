//! Cache trait and implementations shared across Wormhole services.

pub mod bloom_filter;
pub mod cache;
pub mod error;
pub mod layered;
pub mod moka;
pub mod redis;
pub mod redis_ha;

pub use bloom_filter::{BloomFilter, BloomFilterConfig};
pub use cache::UrlCache;
pub use error::{CacheError, Result};
pub use layered::LayeredCache;
pub use moka::MokaUrlCache;
pub use redis::RedisUrlCache;
pub use redis_ha::RedisHAUrlCache;
