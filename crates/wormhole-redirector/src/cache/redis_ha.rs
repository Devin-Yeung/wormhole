#[derive(Debug, Clone)]
pub struct RedisHAUrlCache {
    master_pool: deadpool_redis::sentinel::Pool,
    replica_pool: deadpool_redis::sentinel::Pool,
}

impl RedisHAUrlCache {
    pub fn new<T: AsRef<str>>(sentinels: Vec<T>, service_name: &str) -> Self {
        let sentinels = sentinels
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect::<Vec<_>>();

        let config = deadpool_redis::sentinel::Config::from_urls(
            sentinels.clone(),
            service_name.into(),
            deadpool_redis::sentinel::SentinelServerType::Master,
        );

        let master_pool = config.create_pool(None).unwrap();

        let replica_config = deadpool_redis::sentinel::Config::from_urls(
            sentinels,
            service_name.into(),
            deadpool_redis::sentinel::SentinelServerType::Replica,
        );

        let replica_pool = replica_config.create_pool(None).unwrap();

        Self {
            master_pool,
            replica_pool,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::RedisHAUrlCache;
    use wormhole_test_infra::redis::{RedisHA, RedisHAConfig};

    #[tokio::test]
    async fn it_works() {
        let redis = RedisHA::new(RedisHAConfig::default()).await.unwrap();

        let sentinels = redis
            .sentinel_addresses()
            .await
            .into_iter()
            .map(|(host, port)| format!("redis://{}:{}", host, port))
            .collect::<Vec<_>>();

        let _ = RedisHAUrlCache::new(sentinels, redis.name());
    }
}
