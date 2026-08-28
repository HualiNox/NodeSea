//! Local daemon runtime for NodeSea.
//!
//! The daemon owns the local transport and, later, the BitTorrent engine. The
//! transport implementation is kept private; callers only need an endpoint
//! and [`NodeSeaDaemon`].
#![warn(missing_docs)]

mod transport;

pub use transport::{Endpoint, TransportError};

use crate::transport::{Listener, PlatformTransport, Transport};

/// A NodeSea background service bound to one local endpoint.
pub struct NodeSeaDaemon {
    listener: <PlatformTransport as Transport>::Listener,
}

impl NodeSeaDaemon {
    /// Binds a daemon to `endpoint`.
    ///
    /// Binding happens during construction so that a successfully returned
    /// daemon already owns the listener and its endpoint.
    pub async fn new(endpoint: Endpoint) -> Result<Self, TransportError> {
        let listener = PlatformTransport::bind(&endpoint).await?;
        Ok(Self { listener })
    }

    /// Accepts local client connections until the listener returns an error.
    ///
    /// The method currently logs accepted connections and continues waiting
    /// for the next one. Request handling will be added above this transport
    /// layer.
    pub async fn run(&self) -> Result<(), TransportError> {
        loop {
            let _stream = self.listener.accept().await?;
        }
    }
}

impl Drop for NodeSeaDaemon {
    fn drop(&mut self) {
        if let Err(error) = self.listener.cleanup() {
            tracing::error!(%error, "Failed to clean up the daemon transport");
        }
    }
}
