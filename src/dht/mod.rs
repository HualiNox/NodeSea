//! Data structures used by the DHT routing layer.
//!
//! The module provides node identifiers, node metadata, routing-table
//! buckets, and the errors produced while updating the routing table.

mod bucket;
mod errors;
mod node;
mod routingtable;

pub(crate) use bucket::BUCKET_COUNT;
pub(crate) use node::NODE_ID_BYTES;

pub use errors::{BucketError, DhtError, RoutingTableError};
pub use node::{Node, NodeID};
pub use routingtable::RoutingTable;
