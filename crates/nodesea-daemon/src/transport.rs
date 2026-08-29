//! Platform transport traits and endpoint implementations.

#[cfg(unix)]
use std::io::Error;

use tokio::io::{AsyncRead, AsyncWrite};

#[cfg(unix)]
mod unix;

mod errors;

pub use errors::TransportError;

#[cfg(unix)]
use tokio_stream::Stream;
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
    /// Accepted connection stream consumed by tonic.
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;
    /// Incoming connections. Owning this value also owns socket cleanup.
    type Incoming: Stream<Item = Result<Self::Stream, Error>> + Send + 'static;

    /// Transfers listener ownership into the incoming stream.
    ///
    /// This is why daemon startup takes `self`: tonic must keep the listener
    /// alive for the entire serving loop.
    fn into_incoming(self) -> Self::Incoming;

    /// Removes the socket if this listener still owns its endpoint.
    fn cleanup(&self) -> Result<(), TransportError>;
}
