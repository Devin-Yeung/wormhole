//! Redis HA (High Availability) configuration with typed builder.
//!
//! This module provides a configurable way to set up Redis HA deployments
//! with customizable numbers of masters, replicas, and sentinels.

use typed_builder::TypedBuilder;

/// Configuration for Redis HA setup with typed builder pattern.
///
/// # Default Configuration
///
/// The defaults provide a production-ready setup:
/// - 1 master
/// - 2 replicas
/// - 3 sentinels (quorum: 2)
/// - Service name: "wormhole"
///
/// # Example
///
/// ```rust
/// use wormhole_test_infra::redis::RedisHAConfig;
///
/// // Use defaults (2 replicas, 3 sentinels)
/// let config = RedisHAConfig::builder().build();
///
/// // Custom configuration
/// let config = RedisHAConfig::builder()
///     .num_replicas(3)
///     .num_sentinels(5)
///     .quorum(3)
///     .service_name("wormhole".to_string())
///     .build();
/// ```
#[derive(Debug, Clone, TypedBuilder)]
pub struct RedisHAConfig {
    /// Number of Redis replicas in the deployment.
    #[builder(default = 2)]
    pub num_replicas: usize,

    /// Number of Redis Sentinel instances.
    #[builder(default = 3)]
    pub num_sentinels: usize,

    /// Quorum required for sentinel failover (must be <= num_sentinels).
    #[builder(default = 2)]
    pub quorum: usize,

    /// Sentinel service name (e.g., "wormhole-master").
    #[builder(default = "wormhole-master".to_string())]
    pub service_name: String,
}

impl Default for RedisHAConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl RedisHAConfig {
    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Quorum is greater than the number of sentinels
    /// - Number of sentinels is zero
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.num_sentinels == 0 {
            return Err(ConfigError::InvalidSentinels(
                "Number of sentinels must be at least 1".to_string(),
            ));
        }

        if self.quorum > self.num_sentinels {
            return Err(ConfigError::InvalidQuorum(format!(
                "Quorum ({}) cannot exceed number of sentinels ({})",
                self.quorum, self.num_sentinels
            )));
        }

        Ok(())
    }
}

/// Configuration validation errors.
#[derive(Debug)]
pub enum ConfigError {
    InvalidSentinels(String),
    InvalidQuorum(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSentinels(msg) => write!(f, "Invalid sentinels configuration: {}", msg),
            Self::InvalidQuorum(msg) => write!(f, "Invalid quorum: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RedisHAConfig::builder().build();
        assert_eq!(config.num_replicas, 2);
        assert_eq!(config.num_sentinels, 3);
        assert_eq!(config.quorum, 2);
        assert_eq!(config.service_name, "wormhole-master");
    }

    #[test]
    fn test_default_trait() {
        let config = RedisHAConfig::default();
        assert_eq!(config.num_replicas, 2);
        assert_eq!(config.num_sentinels, 3);
    }

    #[test]
    fn test_custom_config() {
        let config = RedisHAConfig::builder()
            .num_replicas(3)
            .num_sentinels(5)
            .quorum(3)
            .service_name("custom_master".to_string())
            .build();

        assert_eq!(config.num_replicas, 3);
        assert_eq!(config.num_sentinels, 5);
        assert_eq!(config.quorum, 3);
        assert_eq!(config.service_name, "custom_master");
    }

    #[test]
    fn test_validate_success() {
        let config = RedisHAConfig::builder().build();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_quorum_exceeds_sentinels() {
        let config = RedisHAConfig::builder().num_sentinels(2).quorum(3).build();
        assert!(config.validate().is_err());
    }
}
