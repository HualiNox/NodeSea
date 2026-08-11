//! Data structures used by the DHT routing layer.
//!
//! The module provides node identifiers, node metadata, routing-table
//! buckets, a local DHT node, and the errors produced while updating the
//! routing table.

mod bucket;
mod errors;
mod node;
mod routingtable;

pub(crate) use node::NODE_ID_BYTES;

pub use errors::{BucketError, DhtError, RoutingTableError};
pub use node::{Node, NodeID};
pub use routingtable::RoutingTable;

/// Represents a local DHT node and its routing table.
pub struct DhtNode {
    local: Node,
    routing_table: RoutingTable,
}

impl DhtNode {
    /// Creates a new DhtNode with the given local node.
    ///
    /// # Arguments
    ///
    /// * `local` - The local node.
    ///
    /// # Returns
    ///
    /// A new DhtNode.
    pub fn new(local: Node) -> Self {
        let local_id = *local.id();

        Self {
            local,
            routing_table: RoutingTable::new(local_id),
        }
    }

    /// Returns the local node's ID.
    ///
    /// # Returns
    ///
    /// The local node's ID.
    pub fn id(&self) -> &NodeID {
        self.local.id()
    }

    /// Returns a reference to the local node.
    ///
    /// # Returns
    ///
    /// A reference to the local node.
    pub fn local(&self) -> &Node {
        &self.local
    }

    /// Returns a reference to the routing table.
    ///
    /// # Returns
    ///
    /// A reference to the routing table.
    pub fn routing_table(&self) -> &RoutingTable {
        &self.routing_table
    }

    /// Records a node observation in the routing table.
    ///
    /// Observing an existing node ID refreshes its endpoint and routing
    /// metadata without adding another entry.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to insert into the routing table.
    ///
    /// # Returns
    ///
    /// Returns [`DhtError::RoutingTable`] containing
    /// [`RoutingTableError::NodeIsSelf`] if `node` is the local node.
    ///
    /// Returns [`DhtError::Bucket`] containing
    /// [`crate::dht::BucketError::Full`] if `node` has a new ID and the target
    /// bucket has reached its capacity but cannot be split.
    ///
    pub fn insert(&mut self, node: Node) -> Result<(), DhtError> {
        self.routing_table.insert(node)?;
        Ok(())
    }
}
