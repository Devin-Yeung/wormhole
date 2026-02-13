use crate::Result;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::CopyDataSource::Data;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub struct RedisSentinel {
    container: ContainerAsync<GenericImage>,
}

impl RedisSentinel {
    async fn setup() -> Result<ContainerAsync<GenericImage>> {
        let container = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(26379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Sentinel ID is"))
            .with_cmd(vec!["redis-sentinel", "/etc/redis/sentinel.conf"])
            // an empty sentinel.conf is sufficient since we'll configure it via the Redis client after startup
            .with_copy_to("/etc/redis/sentinel.conf", Data(Vec::new()))
            .start()
            .await?;
        Ok(container)
    }

    pub async fn new(master_host: &str, master_port: u16, master_name: &str) -> Result<Self> {
        let container = Self::setup().await?;

        let host = container.get_host().await?.to_string();
        let port = container.get_host_port_ipv4(26379).await?;

        // Configure sentinel via Redis client
        let client = redis::Client::open(format!("redis://{}:{}", host, port))?;
        let mut conn = client.get_multiplexed_async_connection().await?;

        // Send SENTINEL MONITOR command
        let _: () = redis::cmd("SENTINEL")
            .arg("MONITOR")
            .arg(master_name)
            .arg(master_host)
            .arg(master_port)
            .arg(1)
            .query_async(&mut conn)
            .await?;

        Ok(Self { container })
    }

    pub async fn host(&self) -> Result<String> {
        Ok(self.container.get_host().await?.to_string())
    }

    pub async fn port(&self) -> Result<u16> {
        Ok(self.container.get_host_port_ipv4(26379).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::master::RedisMaster;

    #[tokio::test]
    async fn test_sentinel_setup() {
        let master = RedisMaster::new().await.unwrap();
        let (host, port) = master.bridge_addr().await.unwrap();

        // Start sentinel
        let _ = RedisSentinel::new(&host, port, "wormhole-master")
            .await
            .unwrap();
    }
}
