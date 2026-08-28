use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(unix)]
mod unix;

#[cfg(unix)]
mod helper;

mod errors;

pub use errors::TransportError;

#[cfg(unix)]
pub use unix::UnixEndpoint as Endpoint;

#[cfg(unix)]
pub(crate) use unix::UnixTransport as PlatformTransport;

/// A platform-specific transport implementation.
pub(crate) trait Transport {
    type Endpoint;
    type Listener: Listener;

    async fn bind(endpoint: &Self::Endpoint) -> Result<Self::Listener, TransportError>;
}

#[cfg(unix)]
/// Operations required from a local transport listener.
pub(crate) trait Listener {
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;

    async fn accept(&self) -> Result<Self::Stream, TransportError>;

    fn cleanup(&self) -> Result<(), TransportError>;
}
