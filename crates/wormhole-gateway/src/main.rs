use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    info!(listen_addr = %listener.local_addr()?, "starting gateway server");

    Ok(())
}
