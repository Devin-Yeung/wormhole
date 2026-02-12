//! Redirector service library with Redis caching support.
//!
//! This crate provides a [`RedirectorService`] that can resolve short codes
//! to their original URLs. It uses the Repository decorator pattern to
//! add transparent Redis caching.
//!
//! # Example usage
//!
//! ```rust,no_run
//! use wormhole_redirector::RedirectorService;
//! use wormhole_redirector::repository::CachedRepository;
//! use wormhole_core::InMemoryRepository;
//! use wormhole_core::ShortCode;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create Redis connection
//! let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
//! let redis_conn = redis_client.get_multiplexed_tokio_connection().await?;
//!
//! // Create repository with Redis caching
//! let inner_repo = InMemoryRepository::new();
//! let cached_repo = CachedRepository::new(inner_repo, redis_conn, None);
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

pub mod repository;
pub mod service;

pub use service::RedirectorService;
