//! Implementations of user-facing CLI commands.

mod status;

use anyhow::Result;

use crate::{
    DaemonClient,
    cli::{Command, OutputFormat},
};

/// Dispatches a parsed command to its command-specific implementation.
pub async fn run(client: &mut DaemonClient, command: Command, output: OutputFormat) -> Result<()> {
    // Command modules own orchestration and presentation; this client only
    // provides access to daemon RPC capabilities.
    match command {
        Command::Status => status::run(client, output).await,
    }
}
