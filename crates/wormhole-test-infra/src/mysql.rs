use crate::Result;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers::{ContainerAsync, GenericImage};

/// Test fixture for a disposable MySQL server.
pub struct MySqlServer {
    container: ContainerAsync<GenericImage>,
    database: String,
    username: String,
    password: String,
}

impl MySqlServer {
    /// Starts a MySQL container suitable for integration tests.
    pub async fn new() -> Result<Self> {
        let database = "wormhole".to_string();
        let username = "wormhole".to_string();
        let password = "wormhole".to_string();

        let container = GenericImage::new("mysql", "8.4")
            .with_exposed_port(3306_u16.tcp())
            .with_wait_for(WaitFor::message_on_stderr("ready for connections"))
            .with_env_var("MYSQL_DATABASE", database.clone())
            .with_env_var("MYSQL_USER", username.clone())
            .with_env_var("MYSQL_PASSWORD", password.clone())
            .with_env_var("MYSQL_ROOT_PASSWORD", "root")
            .start()
            .await?;

        Ok(Self {
            container,
            database,
            username,
            password,
        })
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
            self.username, self.password, host, port, self.database
        ))
    }

    /// Returns the underlying container reference.
    pub fn container(&self) -> &ContainerAsync<GenericImage> {
        &self.container
    }
}
