use crate::redis::{RedisHAConfig, RedisMaster, RedisReplica, RedisSentinel};
use crate::Result;

pub struct RedisHA {
    config: RedisHAConfig,
    master: RedisMaster,
    replicas: Vec<RedisReplica>,
    sentinel: Vec<RedisSentinel>,
}

impl RedisHA {
    pub async fn new(config: RedisHAConfig) -> Result<Self> {
        let master = RedisMaster::new().await?;

        // WARN: take the addr we use here, which is different from the one we get from host/port
        // we use bridge addr to configure replicas and sentinels, since they need to connect to the master from within the Docker network
        // but for clients outside the Docker network, they should use host/port to connect to the master / sentinels / replicas
        let (host, port) = master.bridge_addr().await?;

        let mut replicas = Vec::new();
        for _ in 0..config.num_replicas {
            let replica = RedisReplica::new(&host, port).await?;
            replicas.push(replica);
        }

        let mut sentinels = Vec::new();
        for _ in 0..config.num_sentinels {
            let sentinel = RedisSentinel::new(&host, port, &config.service_name).await?;
            sentinels.push(sentinel);
        }

        Ok(Self {
            config,
            master,
            replicas,
            sentinel: sentinels,
        })
    }

    pub fn name(&self) -> &str {
        &self.config.service_name
    }

    pub async fn replica_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        // collect replica addresses, if error, skip
        for replica in &self.replicas {
            let host = replica.host().await;
            let port = replica.port().await;
            if let (Ok(host), Ok(port)) = (host, port) {
                let address = format!("redis://{}:{}", host, port);
                addresses.push(address);
            }
        }
        addresses
    }

    pub async fn sentinel_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        for sentinel in &self.sentinel {
            let host = sentinel.host().await;
            let port = sentinel.port().await;
            if let (Ok(host), Ok(port)) = (host, port) {
                let address = format!("redis://{}:{}", host, port);
                addresses.push(address);
            }
        }
        addresses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_ha_setup() -> Result<()> {
        let config = RedisHAConfig::builder()
            .num_replicas(2)
            .num_sentinels(3)
            .quorum(2)
            .service_name("wormhole-master".to_string())
            .build();

        let ha = RedisHA::new(config).await?;
        assert_eq!(ha.replicas.len(), 2);
        assert_eq!(ha.sentinel.len(), 3);
        Ok(())
    }
}
