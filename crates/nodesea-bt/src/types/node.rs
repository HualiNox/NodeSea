//! Domain representation of a DHT node and its endpoint.

use std::net::SocketAddr;

use super::identity::NodeId;

/// A DHT node identity and its network endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DhtNode {
    node_id: NodeId,
    endpoint: SocketAddr,
}

impl DhtNode {
    pub(crate) fn from_ffi(node_id: NodeId, endpoint: SocketAddr) -> Self {
        Self { node_id, endpoint }
    }

    /// Returns the node's DHT identity.
    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    /// Returns the node's reachable network endpoint.
    pub fn endpoint(&self) -> &SocketAddr {
        &self.endpoint
    }
}
