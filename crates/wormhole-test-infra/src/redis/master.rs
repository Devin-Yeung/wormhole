use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

pub struct RedisMaster {
    container: ContainerAsync<GenericImage>,
}

impl RedisMaster {
    pub async fn new() -> Self {
        let container = GenericImage::new("redis", "8.6.0")
            .with_exposed_port(6379_u16.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .start()
            .await
            .expect("Failed to start Redis master container");
        Self { container }
    }

    pub async fn host(&self) -> String {
        let host = self
            .container
            .get_host()
            .await
            .expect("Failed to get replica host")
            .to_string();

        match host.as_str() {
            "localhost" => String::from("127.0.0.1"),
            _ => host,
        }
    }

    pub async fn port(&self) -> u16 {
        self.container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to get master port")
    }

    /// Returns the underlying container reference.
    pub fn container(&self) -> &ContainerAsync<GenericImage> {
        &self.container
    }
}
