use crate::Result;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub struct RedisReplica {
    container: ContainerAsync<GenericImage>,
}

impl RedisReplica {
    pub async fn new(master_host: &str, master_port: u16) -> Result<Self> {
        let replica = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(6379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .with_cmd(vec![
                "redis-server".to_string(),
                "--replicaof".to_string(),
                master_host.to_string(),
                master_port.to_string(),
            ])
            .start()
            .await?;
        Ok(Self { container: replica })
    }

    pub async fn host(&self) -> Result<String> {
        let host = self.container.get_host().await?.to_string();
        Ok(match host.as_str() {
            "localhost" => String::from("127.0.0.1"),
            _ => host,
        })
    }

    pub async fn port(&self) -> Result<u16> {
        Ok(self.container.get_host_port_ipv4(6379).await?)
    }

    /// Returns the underlying container reference.
    pub fn container(&self) -> &ContainerAsync<GenericImage> {
        &self.container
    }
}
