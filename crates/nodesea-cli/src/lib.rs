//! Command-line client for the NodeSea daemon.
//!
//! The CLI talks to a running daemon over local IPC. It does not own the
//! daemon or the libtorrent engine lifecycle.

use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use hyper_util::rt::TokioIo;
use nodesea_proto::v1::{
    EngineStatus, GetStatusRequest, engine_status_service_client::EngineStatusServiceClient,
};
use rust_i18n::t;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint};
use tower::service_fn;

rust_i18n::i18n!("locales");
// Translation catalogs are embedded into the binary, so installed commands
// do not depend on the source tree or a runtime locale directory.

/// Command-line argument definitions.
pub mod cli;
mod commands;

pub use cli::Cli;

/// RPC client for capabilities exposed by a running `nodesead`.
pub struct DaemonClient {
    engine_status: EngineStatusServiceClient<Channel>,
}

impl DaemonClient {
    /// Connects to the daemon through a local Unix socket.
    ///
    /// A refused or missing socket is reported as a localized daemon-not-found
    /// error. Other socket errors retain their underlying cause.
    pub async fn connect(socket_path: impl Into<PathBuf>) -> Result<Self> {
        let socket_path = socket_path.into();

        // Probe first so the common "daemon is not running" case is not hidden
        // behind tonic's nested transport error contexts.
        if let Err(error) = UnixStream::connect(&socket_path).await {
            if matches!(
                error.kind(),
                ErrorKind::ConnectionRefused | ErrorKind::NotFound
            ) {
                bail!(
                    "{}",
                    t!("error.daemon_not_found", socket = socket_path.display())
                );
            }

            return Err(error).with_context(|| {
                format!(
                    "failed to connect to NodeSea daemon at {}",
                    socket_path.display()
                )
            });
        }

        // The URI is a tonic placeholder; the connector below uses the local
        // Unix socket instead of opening a TCP connection.
        let channel = Endpoint::from_static("http://[::]:50051")
            .connect_with_connector(service_fn(move |_| {
                let socket_path = socket_path.clone();
                async move {
                    // Tonic may invoke the connector more than once, so the
                    // closure owns a clonable socket path.
                    let stream = UnixStream::connect(&socket_path).await?;
                    Ok::<_, std::io::Error>(TokioIo::new(stream))
                }
            }))
            .await
            .context("failed to establish NodeSea daemon channel")?;

        Ok(Self {
            engine_status: EngineStatusServiceClient::new(channel),
        })
    }

    /// Queries the current engine status from the daemon.
    pub async fn get_status(&mut self) -> Result<EngineStatus> {
        let response = self
            .engine_status
            .get_status(GetStatusRequest {})
            .await
            .context("failed to query engine status")?
            .into_inner();

        EngineStatus::try_from(response.status)
            .map_err(|_| anyhow::anyhow!("daemon returned an unknown engine status"))
    }
}

/// Parses the selected locale, connects to the daemon, and dispatches a command.
pub async fn run(cli: Cli) -> Result<()> {
    // Read the OS locale before command execution can produce user-visible text.
    cli::apply_system_locale();

    // Keep endpoint policy in the helper crate so CLI and daemon resolve the
    // same default socket. An explicit option takes precedence.
    let socket_path = cli
        .global
        .socket
        .unwrap_or(nodesea_helper::default_socket_path()?);
    let mut client = DaemonClient::connect(socket_path).await?;

    commands::run(&mut client, cli.command, cli.global.output).await
}

/// Queries engine status for library callers that do not need command parsing.
pub async fn get_status(socket_path: impl AsRef<Path>) -> Result<EngineStatus> {
    let mut client = DaemonClient::connect(socket_path.as_ref().to_owned()).await?;
    client.get_status().await
}
