use nodesea_daemon::{Endpoint, NodeSeaDaemon};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let endpoint = Endpoint::default_endpoint()?;

    let daemon = NodeSeaDaemon::new(endpoint).await?;
    daemon.run().await?;
    Ok(())
}
