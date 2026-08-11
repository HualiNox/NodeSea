use std::collections::HashMap;

use crate::dht::{NODE_ID_BYTES, NodeID, errors::BucketError, node::Node};

/// The maximum number of nodes that can be stored in a bucket.
pub(crate) const BUCKET_SIZE: usize = 8;

#[derive(Clone, Debug)]
pub(crate) struct BucketRange {
    prefix: [u8; NODE_ID_BYTES],
    depth: u8,
}

impl BucketRange {
    /// Creates the root range, which contains every possible node ID.
    fn root() -> Self {
        Self {
            prefix: [0; NODE_ID_BYTES],
            depth: 0,
        }
    }

    /// Returns whether `id` shares this range's prefix.
    ///
    /// Node IDs are compared from the most significant bit to the least
    /// significant bit. For each prefix bit, the byte index and the bit index
    /// within that byte are calculated first. The selected bit is then moved
    /// to the least significant position and masked with `1` to obtain either
    /// `0` or `1`.
    ///
    /// A root range has a depth of zero, so it has no prefix bits to compare
    /// and therefore contains every node ID.
    fn contains(&self, id: &NodeID) -> bool {
        for bit_position in 0..self.depth as usize {
            // Eight bits make up one byte; NodeID prefixes are read in
            // big-endian bit order, from bit 7 down to bit 0.
            let byte_position = bit_position / 8;
            let bit_in_byte = 7 - bit_position % 8;

            // Shift the selected bit to position 0, then keep only that bit.
            let prefix_bit = (self.prefix[byte_position] >> bit_in_byte) & 1;
            let id_bit = (id.node_id()[byte_position] >> bit_in_byte) & 1;

            if prefix_bit != id_bit {
                return false;
            }
        }

        true
    }

    /// Creates the child range whose next prefix bit is `bit`.
    fn child(&self, bit: u8) -> Self {
        let mut prefix = self.prefix;
        let bit_position = self.depth as usize;
        let byte_position = bit_position / 8;
        let bit_in_byte = 7 - bit_position % 8;
        let mask = 1 << bit_in_byte;

        prefix[byte_position] &= !mask;
        prefix[byte_position] |= bit << bit_in_byte;

        Self {
            prefix,
            depth: self.depth + 1,
        }
    }
}

/// Stores nodes whose IDs belong to one prefix range in the routing table.
#[derive(Clone, Debug)]
pub(crate) struct Bucket {
    range: BucketRange,
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
            range: BucketRange::root(),
            node_map: HashMap::new(),
        }
    }

    /// Creates a new collection of empty buckets.
    ///
    /// # Returns
    ///
    /// A vector containing the initial root bucket.
    pub(crate) fn new_bucket_collection() -> Vec<Bucket> {
        vec![Bucket::new()]
    }

    /// Returns whether the node ID belongs to this bucket's range.
    pub(crate) fn contains(&self, id: &NodeID) -> bool {
        self.range.contains(id)
    }

    /// Returns whether this bucket can be split into two child ranges.
    pub(crate) fn can_split(&self) -> bool {
        (self.range.depth as usize) < NODE_ID_BYTES * 8
    }

    /// Splits this bucket into a `0` child and a `1` child.
    ///
    /// Existing nodes are redistributed according to the next bit after the
    /// current range prefix.
    pub(crate) fn split(self) -> (Bucket, Bucket) {
        let zero_range = self.range.child(0);
        let one_range = self.range.child(1);
        let mut zero = Bucket {
            range: zero_range,
            node_map: HashMap::new(),
        };
        let mut one = Bucket {
            range: one_range,
            node_map: HashMap::new(),
        };

        for (id, node) in self.node_map {
            if zero.contains(&id) {
                zero.node_map.insert(id, node);
            } else {
                one.node_map.insert(id, node);
            }
        }

        (zero, one)
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
    fn test_range_split_distributes_nodes_by_next_bit() {
        let mut bucket = Bucket::new();
        let mut zero_id = [0u8; NODE_ID_BYTES];
        zero_id[0] = 0x01;
        let mut one_id = [0u8; NODE_ID_BYTES];
        one_id[0] = 0x80;

        bucket
            .add(Node::from_id(NodeID::from_id(zero_id), "a".into(), 1))
            .unwrap();
        bucket
            .add(Node::from_id(NodeID::from_id(one_id), "b".into(), 2))
            .unwrap();

        let (zero, one) = bucket.split();

        assert!(zero.contains(&NodeID::from_id(zero_id)));
        assert!(!zero.contains(&NodeID::from_id(one_id)));
        assert!(one.contains(&NodeID::from_id(one_id)));
        assert!(!one.contains(&NodeID::from_id(zero_id)));
        assert_eq!(zero.nodes().count(), 1);
        assert_eq!(one.nodes().count(), 1);
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
