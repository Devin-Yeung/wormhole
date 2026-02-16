//! Bloom filter cache implementation for fast negative lookups.
//!
//! This module provides a cache decorator that uses a Bloom filter to quickly
//! determine if a short code is definitely not present in the cache, avoiding
//! unnecessary lookups to the underlying storage.
//!
//! # How It Works
//!
//! A Bloom filter is a space-efficient probabilistic data structure that can
//! tell you with certainty if an item is NOT in a set, or that it MIGHT be
//! in the set (with a configurable false positive rate).
//!
//! In this implementation:
//! - `get_url()` first checks the Bloom filter
//!   - If the filter says "definitely not present", returns `None` immediately
//!   - If the filter says "might be present", delegates to the underlying cache
//! - `set_url()` adds the code to the Bloom filter and the underlying cache
//! - `del()` only removes from the underlying cache (Bloom filters don't support deletion)
//!
//! # Use Case
//!
//! This is useful when the underlying cache is expensive to query (e.g., network
//! round-trip to Redis) and most lookups are for non-existent codes.

use async_trait::async_trait;
use parking_lot::RwLock;
use typed_builder::TypedBuilder;
use wormhole_core::{cache::Result, CacheError, ShortCode, UrlCache, UrlRecord};

/// Configuration for the Bloom filter.
///
/// The Bloom filter is a probabilistic data structure that trades a small
/// false positive rate for significant memory savings.
#[derive(Debug, TypedBuilder)]
pub struct BloomFilterConfig {
    /// Expected number of items to be inserted into the filter.
    ///
    /// This should be an estimate of how many unique short codes you expect
    /// to cache. Setting this too low will increase the false positive rate.
    #[builder]
    pub expected_items: usize,

    /// Desired false positive rate as a probability between 0.0 and 1.0.
    ///
    /// For example, a value of 0.01 means approximately 1% false positive rate.
    /// Lower values use more memory but reduce false positives.
    #[builder]
    pub false_positive_rate: f64,
}

/// A cache decorator that uses a Bloom filter for fast negative lookups.
///
/// This wrapper adds a Bloom filter in front of an existing cache implementation.
/// It provides O(1) negative lookups with no false negatives - if the filter
/// says a code is not present, it's definitely not in the cache.
///
/// # Type Parameters
///
/// * `C` - The underlying cache implementation that stores actual URL records
///
/// # Example
///
/// ```rust,ignore
/// use wormhole_redirector::cache::{BloomFilter, BloomFilterConfig};
/// use wormhole_core::UrlCache;
///
/// let config = BloomFilterConfig::builder()
///     .expected_items(1_000_000)
///     .false_positive_rate(0.01) // 1% false positive rate
///     .build();
///
/// let cache = BloomFilter::new(config, underlying_cache)?;
/// ```
pub struct BloomFilter<C: UrlCache> {
    /// The underlying Bloom filter data structure.
    /// Wrapped in an async-aware RwLock for thread-safe concurrent access.
    bloom: RwLock<bloomfilter::Bloom<ShortCode>>,
    /// The underlying cache that stores actual URL records.
    cache: C,
}

impl<C: UrlCache> BloomFilter<C> {
    /// Creates a new Bloom filter cache decorator.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the Bloom filter size and accuracy
    /// * `cache` - The underlying cache to wrap
    ///
    /// # Errors
    ///
    /// Returns `CacheError::Initialization` if Bloom filter setup fails.
    pub fn new(config: BloomFilterConfig, cache: C) -> Result<Self> {
        let bloom =
            bloomfilter::Bloom::new_for_fp_rate(config.expected_items, config.false_positive_rate)
                .map_err(|e| CacheError::Initialization(e.to_string()))?;
        let bloom = RwLock::new(bloom);
        Ok(Self { bloom, cache })
    }
}

#[async_trait]
impl<C: UrlCache> UrlCache for BloomFilter<C> {
    /// Retrieves a URL record from the cache.
    ///
    /// First checks the Bloom filter for a quick negative lookup. If the filter
    /// indicates the code is definitely not present, returns `None` immediately
    /// without querying the underlying cache.
    ///
    /// If the filter indicates the code might be present (including false
    /// positives), delegates to the underlying cache for the actual lookup.
    async fn get_url(&self, code: &ShortCode) -> Result<Option<UrlRecord>> {
        {
            let guard = self.bloom.read();
            if !guard.check(code) {
                // Bloom filter guarantees no false negatives - if it says not present,
                // the code is definitely not in the cache
                return Ok(None);
            }
        }
        // Bloom filter indicates the code might be present (could be a false positive)
        // Delegate to underlying cache to verify
        self.cache.get_url(code).await
    }

    /// Stores a URL record in the cache.
    ///
    /// Adds the short code to the Bloom filter and delegates to the underlying
    /// cache for actual storage. Once a code is added to the filter, subsequent
    /// `get_url` calls will check the underlying cache for that code.
    async fn set_url(&self, code: &ShortCode, record: &UrlRecord) -> Result<()> {
        {
            let mut guard = self.bloom.write();
            guard.set(code);
        }
        self.cache.set_url(code, record).await
    }

    /// Deletes a URL record from the cache.
    ///
    /// # Limitations
    ///
    /// Bloom filters do not support deletion. Removing a code from the underlying
    /// cache does not remove it from the Bloom filter. This means:
    ///
    /// - After deletion, `get_url` may still check the underlying cache (false positive)
    /// - The underlying cache will correctly return `None` for deleted codes
    /// - The false positive rate for deleted codes will gradually increase over time
    ///
    /// For workloads with frequent deletions, consider using a different caching
    /// strategy or periodically rebuilding the Bloom filter.
    async fn del(&self, code: &ShortCode) -> Result<()> {
        // Note: Standard Bloom filters don't support deletion. Counting Bloom filters
        // could be used instead, but that would require additional memory overhead.
        // For now, we accept that deleted items may still trigger cache lookups.
        self.cache.del(code).await
    }
}
