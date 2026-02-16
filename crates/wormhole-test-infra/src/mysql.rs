use crate::Result;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers::{ContainerAsync, GenericImage};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct MysqlConfig {
    #[builder(default = "wormhole".to_string())]
    database: String,
    #[builder(default = "wormhole".to_string())]
    username: String,
    #[builder(default = "wormhole".to_string())]
    password: String,
}

/// Test fixture for a disposable MySQL server.
pub struct MySqlServer {
    container: ContainerAsync<GenericImage>,
    config: MysqlConfig,
}

impl MySqlServer {
    /// Starts a MySQL container suitable for integration tests.
    pub async fn new(config: MysqlConfig) -> Result<Self> {
        let container = GenericImage::new("mysql", "8.4")
            .with_exposed_port(3306_u16.tcp())
            .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
            .with_env_var("MYSQL_DATABASE", config.database.as_str())
            .with_env_var("MYSQL_USER", config.username.as_str())
            .with_env_var("MYSQL_PASSWORD", config.password.as_str())
            .with_env_var("MYSQL_ROOT_PASSWORD", "root")
            .start()
            .await?;

        Ok(Self { container, config })
    }

    pub async fn host(&self) -> Result<String> {
        Ok(self.container.get_host().await?.to_string())
    }

    pub async fn port(&self) -> Result<u16> {
        Ok(self.container.get_host_port_ipv4(3306).await?)
    }

    pub async fn database_url(&self) -> Result<String> {
        let host = self.host().await?;
        let port = self.port().await?;
        Ok(format!(
            "mysql://{}:{}@{}:{}/{}",
            self.config.username, self.config.password, host, port, self.config.database
        ))
    }

    /// Returns the underlying container reference.
    pub fn container(&self) -> &ContainerAsync<GenericImage> {
        &self.container
    }
}
