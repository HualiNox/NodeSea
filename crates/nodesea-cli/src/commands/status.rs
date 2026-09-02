//! Implementation of the `nodesea status` command.

use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    execute,
    style::{Color, ResetColor, SetForegroundColor},
};
use nodesea_proto::v1::EngineStatus;
use rust_i18n::t;

use crate::{DaemonClient, cli::OutputFormat, commands::color_enabled};

/// Fetches and prints the daemon engine status in the requested format.
///
/// Text output may use Crossterm colors when stdout is an interactive
/// terminal; JSON output intentionally never contains terminal styling.
pub async fn run(client: &mut DaemonClient, output: OutputFormat) -> Result<()> {
    let status = client.get_status().await?;

    match output {
        OutputFormat::Text => {
            let label = t!("status.label");
            let value = localized_status(status);

            if color_enabled() {
                // Keep one locked writer for the whole colored line. Passing
                // `&mut stdout` lets Crossterm and the `write!` macros reuse
                // the same writer without moving it between calls.
                let mut stdout = io::stdout().lock();

                execute!(&mut stdout, SetForegroundColor(Color::Cyan))?;
                write!(&mut stdout, "{}: ", label)?;
                execute!(&mut stdout, SetForegroundColor(status_color(status)))?;
                writeln!(&mut stdout, "{value}")?;
                execute!(&mut stdout, ResetColor)?;
            } else {
                println!("{}: {}", label, value);
            }
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

/// Chooses a color that reflects the daemon status without changing its value.
fn status_color(status: EngineStatus) -> Color {
    match status {
        EngineStatus::Running => Color::Green,
        EngineStatus::Starting | EngineStatus::Stopping => Color::Yellow,
        EngineStatus::Failed => Color::Red,
        EngineStatus::Idle | EngineStatus::Stopped => Color::DarkGrey,
        EngineStatus::Unspecified => Color::White,
    }
}
