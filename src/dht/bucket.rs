use std::collections::HashMap;

use crate::dht::{NODE_ID_BYTES, NodeID, errors::BucketError, node::Node};

/// The number of buckets in the routing table.
pub(crate) const BUCKET_COUNT: usize = NODE_ID_BYTES * 8;
/// The maximum number of nodes that can be stored in a bucket.
pub(crate) const BUCKET_SIZE: usize = 20;

/// Stores nodes belonging to one XOR-distance range in a routing table.
#[derive(Clone, Debug)]
pub(crate) struct Bucket {
    node_map: HashMap<NodeID, Node>,
}

impl Bucket {
    /// Creates an empty bucket.
    ///
    /// # Returns
    ///
    /// A new `Bucket` instance with no nodes.
    pub(crate) fn new() -> Self {
        Self {
            node_map: HashMap::new(),
        }
    }

    /// Creates a new collection of empty buckets.
    ///
    /// # Returns
    ///
    /// A vector of empty `Bucket` instances.
    pub(crate) fn new_bucket_collection() -> Vec<Bucket> {
        vec![Bucket::new(); BUCKET_COUNT]
    }

    /// Adds a node to the bucket.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the node was added successfully.
    ///
    /// # Errors
    ///
    /// Returns [`BucketError::NodeAlreadyInBucket`] if the same node is
    /// already stored in the bucket.
    ///
    /// A node with an existing identifier but updated endpoint replaces the
    /// stored node without consuming another bucket slot.
    ///
    /// Returns [`BucketError::Full`] if the bucket already contains
    /// [`BUCKET_SIZE`] different nodes.
    pub(crate) fn add(&mut self, node: Node) -> Result<(), BucketError> {
        let node_id = *node.id();

        if let Some(existing) = self.node_map.get(&node_id) {
            if existing == &node {
                return Err(BucketError::NodeAlreadyInBucket);
            }

            self.node_map.insert(node_id, node);
            return Ok(());
        }

        if self.node_map.len() < BUCKET_SIZE {
            self.node_map.insert(node_id, node);
            Ok(())
        } else {
            Err(BucketError::Full)
        }
    }

    /// Returns an immutable view of the nodes in the bucket.
    ///
    /// # Returns
    ///
    /// An iterator over the nodes currently stored in the bucket.
    pub(crate) fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.node_map.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let mut bucket = Bucket::new();
        let node = Node::random(String::from("127.0.0.1"), 12400);
        let result = bucket.add(node);
        assert!(result.is_ok());
        assert_eq!(bucket.nodes().count(), 1);
    }

    #[test]
    fn test_add_overflow() {
        let mut bucket = Bucket::new();
        for p in 0..BUCKET_SIZE {
            let mut id = [0u8; NODE_ID_BYTES];
            id[NODE_ID_BYTES - 1] = p as u8;
            let result = bucket.add(Node::from_id(
                NodeID::from_id(id),
                String::from("127.0.0.1"),
                p as u16,
            ));
            assert!(result.is_ok())
        }
        for p in BUCKET_SIZE..2 * BUCKET_SIZE {
            let mut id = [0u8; NODE_ID_BYTES];
            id[NODE_ID_BYTES - 1] = p as u8;
            let result = bucket.add(Node::from_id(
                NodeID::from_id(id),
                String::from("127.0.0.1"),
                p as u16,
            ));
            assert!(result.is_err())
        }
        assert_eq!(bucket.nodes().count(), BUCKET_SIZE);
    }

    #[test]
    fn test_add_same_node_id_updates_endpoint() {
        let mut bucket = Bucket::new();
        let id = NodeID::from_id([0x80; NODE_ID_BYTES]);

        assert!(
            bucket
                .add(Node::from_id(id, "127.0.0.1".into(), 6881))
                .is_ok()
        );
        assert!(
            bucket
                .add(Node::from_id(id, "127.0.0.2".into(), 6882))
                .is_ok()
        );

        assert_eq!(bucket.nodes().count(), 1);
        let node = bucket.nodes().next().expect("updated node should exist");
        assert_eq!(node.id(), &id);
        assert_eq!(node.address(), "127.0.0.2");
        assert_eq!(node.port(), 6882);
    }

    #[test]
    fn test_add_rejects_exact_duplicate() {
        let mut bucket = Bucket::new();
        let id = NodeID::from_id([0x80; NODE_ID_BYTES]);
        let node = Node::from_id(id, "127.0.0.1".into(), 6881);

        assert!(bucket.add(node.clone()).is_ok());
        assert_eq!(bucket.add(node), Err(BucketError::NodeAlreadyInBucket));
        assert_eq!(bucket.nodes().count(), 1);
    }
}
