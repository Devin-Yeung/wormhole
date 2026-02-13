use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::CopyDataSource::Data;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub struct RedisSentinel {
    container: ContainerAsync<GenericImage>,
}

impl RedisSentinel {
    async fn setup() -> ContainerAsync<GenericImage> {
        let container = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(26379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Sentinel ID is"))
            .with_cmd(vec!["redis-sentinel", "/etc/redis/sentinel.conf"])
            // an empty sentinel.conf is sufficient since we'll configure it via the Redis client after startup
            .with_copy_to("/etc/redis/sentinel.conf", Data(Vec::new()))
            .start()
            .await
            .expect("Failed to start Redis sentinel container");
        container
    }

    pub async fn new(master_host: &str, master_port: u16, master_name: &str) -> Self {
        let container = Self::setup().await;

        let host = container
            .get_host()
            .await
            .expect("Failed to get sentinel host")
            .to_string();
        let port = container
            .get_host_port_ipv4(26379)
            .await
            .expect("Failed to get sentinel port");

        // Configure sentinel via Redis client
        let client = redis::Client::open(format!("redis://{}:{}", host, port))
            .expect("Failed to create Redis client");
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to sentinel");

        dbg!(master_host, master_port);

        // Send SENTINEL MONITOR command
        let _: () = redis::cmd("SENTINEL")
            .arg("MONITOR")
            .arg(master_name)
            .arg(master_host)
            .arg(master_port)
            .arg(1)
            .query_async(&mut conn)
            .await
            .expect("Failed to configure sentinel monitor");

        Self { container }
    }

    pub async fn host(&self) -> String {
        self.container
            .get_host()
            .await
            .expect("Failed to get sentinel host")
            .to_string()
    }

    pub async fn port(&self) -> u16 {
        self.container
            .get_host_port_ipv4(26379)
            .await
            .expect("Failed to get sentinel port")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::master::RedisMaster;

    #[tokio::test]
    async fn test_sentinel_setup() {
        let master = RedisMaster::new().await;
        let master_host = master.host().await;
        let master_port = master.port().await;

        // Start sentinel
        let _ = RedisSentinel::new(&master_host, master_port, "wormhole-master").await;
    }
}
