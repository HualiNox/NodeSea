use nodesea_daemon::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    run().await?;
    Ok(())
}
