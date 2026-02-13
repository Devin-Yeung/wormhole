//! Redirector service library with caching support.
//!
//! This crate provides a [`RedirectorService`] that can resolve short codes
//! to their original URLs. It uses the Repository decorator pattern to
//! add transparent caching via either Redis or in-memory (Moka) caches.
//!
//! # Example with Redis
//!
//! ```rust,no_run
//! use wormhole_redirector::{RedirectorService, RedisUrlCache, CachedRepository};
//! use wormhole_core::InMemoryRepository;
//! use wormhole_core::ShortCode;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Redis connection
//! let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
//! let redis_conn = redis_client.get_multiplexed_async_connection().await?;
//!
//! // Create repository with Redis caching
//! let inner_repo = InMemoryRepository::new();
//! let cache = RedisUrlCache::new(redis_conn);
//! let cached_repo = CachedRepository::new(inner_repo, cache);
//!
//! // Create redirector service
//! let service = RedirectorService::new(cached_repo);
//!
//! // Resolve a short code
//! let code = ShortCode::new("abc123")?;
//! if let Some(url) = service.resolve(&code).await? {
//!     println!("Redirect to: {}", url);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Example with Moka (in-memory)
//!
//! ```rust
//! use wormhole_redirector::{RedirectorService, MokaUrlCache, CachedRepository};
//! use wormhole_core::InMemoryRepository;
//! use wormhole_core::ShortCode;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create repository with Moka in-memory caching
//! let inner_repo = InMemoryRepository::new();
//! let cache = MokaUrlCache::new();
//! let cached_repo = CachedRepository::new(inner_repo, cache);
//!
//! // Create redirector service
//! let service = RedirectorService::new(cached_repo);
//!
//! // Resolve a short code
//! let code = ShortCode::new("abc123")?;
//! if let Some(url) = service.resolve(&code).await? {
//!     println!("Redirect to: {}", url);
//! }
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod repository;
pub mod service;

pub use cache::{MokaUrlCache, RedisUrlCache};
pub use repository::CachedRepository;
pub use service::RedirectorService;
