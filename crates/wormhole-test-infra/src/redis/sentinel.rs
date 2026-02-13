use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub struct RedisSentinel {
    container: ContainerAsync<GenericImage>,
}

impl RedisSentinel {
    pub async fn new(master_host: &str, master_port: u16, master_name: &str) -> Self {
        let container = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(26379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Sentinel running"))
            .with_cmd(vec!["redis-sentinel", "--sentinel"])
            .start()
            .await
            .expect("Failed to start Redis sentinel container");

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
