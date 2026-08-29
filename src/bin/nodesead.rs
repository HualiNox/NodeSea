//! NodeSea daemon process entry point.
//!
//! Runtime composition lives in [`nodesea_daemon`]; this binary only
//! initializes logging and starts the daemon.

use nodesea_daemon::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    run().await?;
    Ok(())
}
