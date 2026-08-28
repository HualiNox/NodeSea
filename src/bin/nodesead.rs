#[cfg(not(target_os = "linux"))]
use std::error::Error;
use std::path::PathBuf;

use nodesea_daemon::{Endpoint, NodeSeaDaemon};

#[cfg(target_os = "linux")]
fn ensure_service_permissions() -> Result<(), Box<dyn Error>> {
    // SAFETY: geteuid has no preconditions and only reads the effective UID.
    if unsafe { libc::geteuid() } != 0 {
        return Err("nodesead must run as root on Linux".into());
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn ensure_service_permissions() -> Result<(), Box<dyn Error>> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn default_socket_path() -> Result<PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join("Library/Application Support/NodeSea/nodesead.sock"))
}

#[cfg(target_os = "linux")]
fn default_socket_path() -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from("/run/nodesea/nodesead.sock"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    ensure_service_permissions()?;

    let socket_path = default_socket_path()?;
    let endpoint = Endpoint::new(socket_path.clone());
    tracing::info!(path = %socket_path.display(), "Starting NodeSea daemon");

    let daemon = NodeSeaDaemon::new(endpoint).await?;
    daemon.run().await?;
    Ok(())
}
