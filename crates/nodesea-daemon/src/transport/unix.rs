//! Unix-domain socket transport implementation.

use std::{
    fs,
    io::ErrorKind,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::PathBuf,
};

use tokio::net::{UnixListener as TokioUnixListener, UnixStream as TokioUnixStream};

use crate::transport::{Listener, Transport, TransportError};

const SOCKET_MODE: u32 = 0o600;
const USER_DIRECTORY_MODE: u32 = 0o700;
const SYSTEM_DIRECTORY_MODE: u32 = 0o755;
/// Endpoint backed by a Unix domain socket.
pub struct UnixEndpoint {
    /// Filesystem path of the Unix domain socket.
    path: PathBuf,
}

impl UnixEndpoint {
    /// Creates an endpoint backed by a Unix domain socket at `path`.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Creates the default endpoint for the current process.
    ///
    /// Root uses the platform service path. Other users use the platform-specific
    /// per-user endpoint path.
    #[cfg(target_os = "macos")]
    pub fn default_endpoint() -> Result<Self, TransportError> {
        let euid = unsafe { libc::geteuid() };
        let path = if euid == 0 {
            PathBuf::from("/var/run/nodesea/socket")
        } else {
            super::helper::current_user_endpoint_path()?
        };

        Ok(Self::new(path))
    }

    /// Creates the default endpoint for the current Linux process.
    #[cfg(target_os = "linux")]
    pub fn default_endpoint() -> Result<Self, TransportError> {
        let path = if unsafe { libc::geteuid() } == 0 {
            PathBuf::from("/run/nodesea/socket")
        } else {
            super::helper::current_user_endpoint_path()?
        };

        Ok(Self::new(path))
    }
}

pub(crate) struct UnixTransport;

pub(crate) struct UnixListener {
    socket: TokioUnixListener,
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Listener for UnixListener {
    type Stream = TokioUnixStream;

    async fn accept(&self) -> Result<Self::Stream, TransportError> {
        match self.socket.accept().await {
            Ok((stream, _)) => Ok(stream),
            Err(e) => {
                tracing::error!(
                    "Failed to accept connection on Unix socket {}: {}",
                    self.path.display(),
                    e
                );
                Err(TransportError::Accept {
                    path: self.path.clone(),
                    source: e,
                })
            }
        }
    }

    fn cleanup(&self) -> Result<(), TransportError> {
        // Check if the socket file exists
        let metadata = match fs::symlink_metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    tracing::error!(
                        "Failed to get metadata for path {}: {}",
                        self.path.display(),
                        e
                    );
                    return Err(TransportError::InspectPath {
                        path: self.path.clone(),
                        source: e,
                    });
                } else {
                    // If the file does not exist, we consider it already cleaned up
                    return Ok(());
                }
            }
        };

        // Check if the socket file is the same as the one created by this listener
        if metadata.dev() != self.device || metadata.ino() != self.inode {
            tracing::error!(
                "Socket file {} is not the same as the one created by this listener, device and inode numbers do not match",
                self.path.display()
            );
            return Err(TransportError::SocketPathChanged {
                path: self.path.clone(),
            });
        }

        // Check if the socket file is a Unix socket
        if !metadata.file_type().is_socket() {
            tracing::error!(
                "Path {} is not a Unix socket, expecting a socket for Unix transport",
                self.path.display()
            );
            return Err(TransportError::PathNotSocket {
                path: self.path.clone(),
            });
        }

        // Remove the socket file
        if let Err(e) = fs::remove_file(&self.path)
            && e.kind() != ErrorKind::NotFound
        {
            tracing::error!(
                "Failed to remove socket file {}: {}",
                self.path.display(),
                e
            );
            return Err(TransportError::RemoveSocket {
                path: self.path.clone(),
                source: e,
            });
        }

        Ok(())
    }
}

impl Transport for UnixTransport {
    type Endpoint = UnixEndpoint;
    type Listener = UnixListener;

    async fn bind(endpoint: &UnixEndpoint) -> Result<Self::Listener, TransportError> {
        let UnixEndpoint { path } = endpoint;

        // Ensure the parent directory exists and is a directory
        if let Some(parent) = path.parent() {
            let mut created_parent = false;
            match fs::metadata(parent) {
                Ok(metadata) if metadata.is_dir() => {}
                Ok(_) => {
                    tracing::error!(
                        "Parent path {} is not a directory, expecting a directory for Unix socket",
                        parent.display()
                    );
                    return Err(TransportError::ParentNotDirectory {
                        path: parent.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    if let Err(error) = fs::create_dir_all(parent) {
                        tracing::error!(
                            "Failed to create transport directory {}: {}",
                            parent.display(),
                            error
                        );
                        return Err(TransportError::CreateDirectory {
                            path: parent.to_path_buf(),
                            source: error,
                        });
                    }
                    created_parent = true;
                }
                Err(error) => {
                    tracing::error!(
                        "Failed to inspect transport directory {}: {}",
                        parent.display(),
                        error
                    );
                    return Err(TransportError::InspectPath {
                        path: parent.to_path_buf(),
                        source: error,
                    });
                }
            }

            if created_parent {
                let mode = if unsafe { libc::geteuid() } == 0 {
                    SYSTEM_DIRECTORY_MODE
                } else {
                    USER_DIRECTORY_MODE
                };
                if let Err(source) = fs::set_permissions(parent, fs::Permissions::from_mode(mode)) {
                    tracing::error!(
                        "Failed to set permissions on transport directory {}: {}",
                        parent.display(),
                        source
                    );
                    return Err(TransportError::SetPermissions {
                        path: parent.to_path_buf(),
                        source,
                    });
                }
            }
        }

        // Check if the path is a socket
        let socket_metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == ErrorKind::NotFound => None,
            Err(error) => {
                tracing::error!(
                    "Failed to inspect socket path {}: {}",
                    path.display(),
                    error
                );
                return Err(TransportError::InspectPath {
                    path: path.to_path_buf(),
                    source: error,
                });
            }
        };

        if let Some(metadata) = socket_metadata {
            let metadata_dev = metadata.dev();
            let metadata_ino = metadata.ino();

            if !metadata.file_type().is_socket() {
                tracing::error!(
                    "Path {} is not a Unix socket, expecting a socket for Unix transport",
                    path.display()
                );
                return Err(TransportError::PathNotSocket {
                    path: path.to_path_buf(),
                });
            }

            // Check if the socket is already in use
            match TokioUnixStream::connect(path).await {
                Ok(_) => {
                    tracing::error!(
                        "Socket file {} is already in use, connection succeeded",
                        path.display()
                    );
                    return Err(TransportError::SocketAlreadyInUse {
                        path: path.to_path_buf(),
                    });
                }
                Err(e)
                    if e.kind() != ErrorKind::ConnectionRefused
                        && e.kind() != ErrorKind::NotFound =>
                {
                    tracing::error!(
                        "Failed to connect to existing Unix socket {} while checking whether it is active: {}",
                        path.display(),
                        e
                    );
                    return Err(TransportError::SocketConnect {
                        path: path.to_path_buf(),
                        source: e,
                    });
                }
                Err(_) => {
                    // Connection refused or not found, which means the socket is not in use
                }
            }

            // Check if the socket file is not in use by comparing device and inode numbers
            let current_metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(e) => {
                    tracing::error!("Failed to get metadata for path {}: {}", path.display(), e);
                    return Err(TransportError::InspectPath {
                        path: path.to_path_buf(),
                        source: e,
                    });
                }
            };
            if current_metadata.dev() != metadata_dev || current_metadata.ino() != metadata_ino {
                tracing::error!(
                    "Socket file {} changed while checking, device or inode numbers do not match",
                    path.display()
                );
                return Err(TransportError::SocketPathChanged {
                    path: path.to_path_buf(),
                });
            }

            // Delete the existing socket file if it is a socket and not in use
            if let Err(e) = fs::remove_file(path) {
                tracing::error!(
                    "Failed to remove existing socket file {}: {}",
                    path.display(),
                    e
                );
                return Err(TransportError::RemoveSocket {
                    path: path.to_path_buf(),
                    source: e,
                });
            }
        }

        // Bind the Unix socket
        let listener = match TokioUnixListener::bind(path) {
            Ok(listener) => listener,
            Err(e) => {
                tracing::error!("Failed to bind Unix socket {}: {}", path.display(), e);
                return Err(TransportError::Bind {
                    path: path.to_path_buf(),
                    source: e,
                });
            }
        };

        if let Err(source) = fs::set_permissions(path, fs::Permissions::from_mode(SOCKET_MODE)) {
            tracing::error!(
                "Failed to set permissions on Unix socket {}: {}",
                path.display(),
                source
            );
            let _ = fs::remove_file(path);
            return Err(TransportError::SetPermissions {
                path: path.to_path_buf(),
                source,
            });
        }

        // Get the device and inode numbers of the newly created socket file
        let current_metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                tracing::error!("Failed to get metadata for path {}: {}", path.display(), e);
                return Err(TransportError::InspectPath {
                    path: path.to_path_buf(),
                    source: e,
                });
            }
        };

        Ok(UnixListener {
            socket: listener,
            path: path.to_path_buf(),
            device: current_metadata.dev(),
            inode: current_metadata.ino(),
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use tokio::net::UnixStream;

    use super::*;

    static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let sequence = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nodesea-daemon-transport-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self(path)
        }

        fn socket(&self) -> PathBuf {
            self.0.join("socket")
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn bind_creates_missing_parent_directory() {
        let directory = TestDir::new();
        let socket = directory.0.join("run/socket");

        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("listener should bind");

        assert!(socket.exists());
        assert!(socket.parent().is_some_and(Path::is_dir));
        listener.cleanup().expect("socket should be cleaned up");
    }

    #[tokio::test]
    async fn accept_returns_connected_stream() {
        let directory = TestDir::new();
        let socket = directory.socket();
        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("listener should bind");

        let client = UnixStream::connect(&socket);
        let (client, server) = tokio::join!(client, listener.accept());

        assert!(client.is_ok());
        assert!(server.is_ok());
        listener.cleanup().expect("socket should be cleaned up");
    }

    #[tokio::test]
    async fn cleanup_removes_socket() {
        let directory = TestDir::new();
        let socket = directory.socket();
        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("listener should bind");

        listener.cleanup().expect("socket should be removed");
        assert!(!socket.exists());
    }

    #[tokio::test]
    async fn bind_rejects_parent_that_is_not_a_directory() {
        let directory = TestDir::new();
        let parent = directory.0.join("not-a-directory");
        fs::write(&parent, b"file").expect("test file should be created");

        let result = UnixTransport::bind(&UnixEndpoint::new(parent.join("socket"))).await;

        assert!(matches!(
            result,
            Err(TransportError::ParentNotDirectory { .. })
        ));
    }

    #[tokio::test]
    async fn bind_rejects_existing_non_socket_path() {
        let directory = TestDir::new();
        let socket = directory.socket();
        fs::write(&socket, b"file").expect("test file should be created");

        let result = UnixTransport::bind(&UnixEndpoint::new(socket.clone())).await;

        assert!(matches!(result, Err(TransportError::PathNotSocket { .. })));
    }

    #[tokio::test]
    async fn bind_rejects_active_socket() {
        let directory = TestDir::new();
        let socket = directory.socket();
        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("first listener should bind");

        let result = UnixTransport::bind(&UnixEndpoint::new(socket.clone())).await;

        assert!(matches!(
            result,
            Err(TransportError::SocketAlreadyInUse { .. })
        ));
        listener.cleanup().expect("socket should be cleaned up");
    }

    #[tokio::test]
    async fn bind_replaces_stale_socket() {
        let directory = TestDir::new();
        let socket = directory.socket();
        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("first listener should bind");
        drop(listener);

        let listener = UnixTransport::bind(&UnixEndpoint::new(socket.clone()))
            .await
            .expect("stale socket should be replaced");

        listener.cleanup().expect("socket should be cleaned up");
    }
}
