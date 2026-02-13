use crate::Result;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

pub struct RedisMaster {
    container: ContainerAsync<GenericImage>,
}

impl RedisMaster {
    pub async fn new() -> Result<Self> {
        let container = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(6379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .start()
            .await?;
        Ok(Self { container })
    }

    pub async fn host(&self) -> Result<String> {
        let host = self.container.get_host().await?.to_string();
        Ok(host)
    }

    pub async fn bridge_addr(&self) -> Result<(String, u16)> {
        let ip = self.container.get_bridge_ip_address().await?.to_string();
        Ok((ip, 6379))
    }

    pub async fn port(&self) -> Result<u16> {
        Ok(self.container.get_host_port_ipv4(6379).await?)
    }

    /// Returns the underlying container reference.
    pub fn container(&self) -> &ContainerAsync<GenericImage> {
        &self.container
    }
}
