//! Domain representation of a DHT node and its endpoint.

use std::net::SocketAddr;

use super::identity::NodeId;

/// A DHT node identity and its network endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DhtNode {
    /// The node's DHT identity.
    pub node_id: NodeId,
    /// The node's reachable network endpoint.
    pub endpoint: SocketAddr,
}
