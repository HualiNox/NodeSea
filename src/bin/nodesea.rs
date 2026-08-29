//! NodeSea command-line client entry point.

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Keep the top-level binary thin; CLI definitions and execution remain in
    // the reusable nodesea-cli library.
    nodesea_cli::run(nodesea_cli::Cli::parse()).await
}
