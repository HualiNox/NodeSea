use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
/// Errors returned while creating or using a local daemon transport.
pub enum TransportError {
    /// The user runtime directory is not available for the default endpoint.
    #[error("user runtime directory is not available for the default transport endpoint")]
    MissingRuntimeDirectory,

    /// The user's home directory is not available for the default endpoint.
    #[error("home directory is not available for the default transport endpoint")]
    MissingHomeDirectory,

    /// Setting permissions on a transport path failed.
    #[error("failed to set permissions on transport path `{path}`: {source}")]
    SetPermissions {
        /// Path whose permissions could not be set.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },

    /// Inspecting a transport path failed.
    #[error("failed to inspect transport path `{path}`: {source}")]
    InspectPath {
        /// Path whose metadata could not be read.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },

    /// Creating the parent directory for a transport failed.
    #[error("failed to create transport directory `{path}`: {source}")]
    CreateDirectory {
        /// Directory that could not be created.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },

    /// The parent path exists but is not a directory.
    #[error("parent path is not a directory: {}", path.display())]
    ParentNotDirectory {
        /// Parent path that is not a directory.
        path: PathBuf,
    },

    /// The configured path exists but is not a Unix socket.
    #[error("transport path is not a Unix socket: {}", path.display())]
    PathNotSocket {
        /// Occupied transport path.
        path: PathBuf,
    },

    /// Another process is already listening on the transport socket.
    #[error("transport socket is already in use: {}", path.display())]
    SocketAlreadyInUse {
        /// Socket path already in use.
        path: PathBuf,
    },

    /// Connecting to an existing socket while checking its state failed.
    #[error(
        "failed to connect to transport socket `{path}` while checking whether it is active: {source}"
    )]
    SocketConnect {
        /// Existing socket path that could not be checked.
        path: PathBuf,
        /// Underlying connection error.
        #[source]
        source: io::Error,
    },

    /// The socket path changed while it was being checked.
    #[error("transport socket path changed while checking: {}", path.display())]
    SocketPathChanged {
        /// Socket path whose identity changed.
        path: PathBuf,
    },

    /// Binding the transport socket failed.
    #[error("failed to bind transport socket `{path}`: {source}")]
    Bind {
        /// Socket path that could not be bound.
        path: PathBuf,
        /// Underlying bind error.
        #[source]
        source: io::Error,
    },

    /// Accepting a client connection failed.
    #[error("failed to accept a transport connection on `{path}`: {source}")]
    Accept {
        /// Listener path that could not accept a connection.
        path: PathBuf,
        /// Underlying accept error.
        #[source]
        source: io::Error,
    },

    /// Removing the transport socket failed.
    #[error("failed to remove transport socket `{path}`: {source}")]
    RemoveSocket {
        /// Socket path that could not be removed.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: io::Error,
    },
}
