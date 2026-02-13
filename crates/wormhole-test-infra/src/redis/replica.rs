use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

pub struct RedisReplica {
    container: ContainerAsync<GenericImage>,
}

impl RedisReplica {
    pub async fn new(master_host: &str, master_port: u16) -> Self {
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
            .await
            .expect("Failed to start Redis replica container");
        Self { container: replica }
    }
}
