use crate::redis::{RedisHAConfig, RedisMaster, RedisReplica, RedisSentinel};
use crate::Result;

pub struct RedisHA {
    master: RedisMaster,
    replicas: Vec<RedisReplica>,
    sentinel: Vec<RedisSentinel>,
}

impl RedisHA {
    pub async fn new(config: RedisHAConfig) -> Result<Self> {
        let master = RedisMaster::new().await?;

        let master_host = master.host().await?;
        let master_port = master.port().await?;

        let mut replicas = Vec::new();
        for _ in 0..config.num_replicas {
            let replica = RedisReplica::new(&master_host, master_port).await?;
            replicas.push(replica);
        }

        let mut sentinels = Vec::new();
        for _ in 0..config.num_sentinels {
            let sentinel =
                RedisSentinel::new(&master_host, master_port, &config.service_name).await?;
            sentinels.push(sentinel);
        }

        Ok(Self {
            master,
            replicas,
            sentinel: sentinels,
        })
    }

    pub async fn replica_addresses(&self) -> Vec<(String, u16)> {
        let mut addresses = Vec::new();
        // collect replica addresses, if error, skip
        for replica in &self.replicas {
            if let (Ok(host), Ok(port)) = (replica.host().await, replica.port().await) {
                addresses.push((host, port));
            }
        }
        addresses
    }

    pub async fn sentinel_addresses(&self) -> Vec<(String, u16)> {
        let mut addresses = Vec::new();
        // collect sentinel addresses, if error, skip
        for sentinel in &self.sentinel {
            if let (Ok(host), Ok(port)) = (sentinel.host().await, sentinel.port().await) {
                addresses.push((host, port));
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
