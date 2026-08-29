//! Implementation of the `nodesea status` command.

use anyhow::Result;
use nodesea_proto::v1::EngineStatus;
use rust_i18n::t;

use crate::{DaemonClient, cli::OutputFormat};

/// Fetches and prints the daemon engine status in the requested format.
pub async fn run(client: &mut DaemonClient, output: OutputFormat) -> Result<()> {
    let status = client.get_status().await?;

    match output {
        OutputFormat::Text => {
            println!("{}: {}", t!("status.label"), localized_status(status));
        }
        OutputFormat::Json => {
            // Keep machine-readable output stable and independent of locale.
            println!("{{\"status\":\"{}\"}}", status_key(status));
        }
    }

    Ok(())
}

/// Translates a protocol status enum into the active human-readable locale.
fn localized_status(status: EngineStatus) -> String {
    t!(format!("status.{}", status_key(status))).to_string()
}

/// Returns the stable machine-readable key used by JSON output and translation keys.
fn status_key(status: EngineStatus) -> &'static str {
    match status {
        EngineStatus::Idle => "idle",
        EngineStatus::Starting => "starting",
        EngineStatus::Running => "running",
        EngineStatus::Stopping => "stopping",
        EngineStatus::Stopped => "stopped",
        EngineStatus::Failed => "failed",
        EngineStatus::Unspecified => "unspecified",
    }
}
