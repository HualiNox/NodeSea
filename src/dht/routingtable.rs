use crate::dht::{BUCKET_COUNT, DhtError, NODE_ID_BYTES, RoutingTableError, node::Node};

use super::{bucket::Bucket, node::NodeID};

/// The routing table for a node in the DHT.
///
/// The table contains 160 buckets. Each bucket represents one range of XOR
/// distances from the local node identifier.
///
/// Each bucket contains the nodes whose identifiers fall within that range.
#[derive(Debug)]
pub struct RoutingTable {
    local_id: NodeID,
    buckets: Vec<Bucket>,
}

impl RoutingTable {
    /// Creates an empty routing table for the given local node identifier.
    ///
    /// # Arguments
    ///
    /// * `local_id` - The identifier of the local node.
    ///
    /// # Returns
    ///
    /// A routing table containing 160 empty buckets.
    pub fn new(local_id: NodeID) -> Self {
        Self {
            local_id,
            buckets: Bucket::new_bucket_collection(),
        }
    }

    /// Inserts a node into the routing table.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to insert.
    ///
    /// # Errors
    ///
    /// Returns [`DhtError::RoutingTable`] containing
    /// [`RoutingTableError::NodeIsSelf`] if `node` has the same identifier as
    /// the local node.
    ///
    /// Returns [`DhtError::Bucket`] containing
    /// [`crate::dht::BucketError::Full`] if the target bucket has reached its
    /// capacity.
    ///
    /// Returns [`DhtError::Bucket`] containing
    /// [`crate::dht::BucketError::NodeAlreadyInBucket`] if the exact node is
    /// already stored in the target bucket.
    pub fn insert(&mut self, node: Node) -> Result<(), DhtError> {
        let distance = self.local_id.distance(node.id());
        let index = Self::bucket_index(distance)?;

        self.buckets[index].add(node)?;

        Ok(())
    }

    /// Maps a non-zero XOR distance to its bucket index.
    ///
    /// # Arguments
    ///
    /// * `distance` - A 20-byte XOR distance from the local node.
    ///
    /// # Returns
    ///
    /// The index of the bucket corresponding to the distance.
    ///
    /// # Errors
    ///
    /// Returns [`DhtError::RoutingTable`] containing
    /// [`RoutingTableError::NodeIsSelf`] if the distance is zero.
    fn bucket_index(distance: [u8; NODE_ID_BYTES]) -> Result<usize, DhtError> {
        let Some((i, byte)) = distance.iter().enumerate().find(|(_, b)| **b != 0) else {
            return Err(DhtError::RoutingTable(RoutingTableError::NodeIsSelf));
        };

        Ok(BUCKET_COUNT - 1 - (i * 8 + byte.leading_zeros() as usize))
    }

    /// Returns the closest nodes to a target identifier.
    ///
    /// # Arguments
    ///
    /// * `target` - The identifier to measure distance from.
    /// * `count` - The maximum number of nodes to return.
    ///
    /// # Returns
    ///
    /// A vector containing at most `count` nodes, ordered by ascending XOR
    /// distance from `target`.
    pub fn find_closest(&self, target: &NodeID, count: usize) -> Vec<&Node> {
        let mut nodes = Vec::new();

        for bucket in &self.buckets {
            nodes.extend(bucket.nodes());
        }
        nodes.sort_by_key(|node| target.distance(node.id()));

        nodes.truncate(count);

        nodes
    }
}

#[cfg(test)]
mod tests {
    use crate::dht::{BucketError, bucket::BUCKET_SIZE};

    use super::*;

    #[test]
    fn test_new() {
        let rt = RoutingTable::new(NodeID::random());
        assert_eq!(rt.buckets.len(), BUCKET_COUNT);
        for bucket in &rt.buckets {
            assert!(bucket.nodes().next().is_none());
        }
    }

    #[test]
    fn test_bucket_index() {
        let distance = [0u8; NODE_ID_BYTES];
        assert_eq!(
            RoutingTable::bucket_index(distance),
            Err(DhtError::RoutingTable(RoutingTableError::NodeIsSelf))
        );

        for bit_position in 0..BUCKET_COUNT {
            let mut distance = [0u8; NODE_ID_BYTES];
            let byte_position = bit_position / 8;
            let bit_in_byte = bit_position % 8;
            distance[byte_position] = 1 << (7 - bit_in_byte);

            assert_eq!(
                RoutingTable::bucket_index(distance),
                Ok(BUCKET_COUNT - 1 - bit_position),
                "unexpected bucket for bit position {bit_position}"
            );
        }
    }

    #[test]
    fn test_insert_self_returns_error() {
        let local = NodeID::from_id([0u8; NODE_ID_BYTES]);
        let mut table = RoutingTable::new(local);

        let result = table.insert(Node::from_id(local, "127.0.0.1".into(), 6881));

        assert_eq!(
            result,
            Err(DhtError::RoutingTable(RoutingTableError::NodeIsSelf))
        );
    }

    #[test]
    fn test_insert_full_bucket_returns_error() {
        let local = NodeID::from_id([0u8; NODE_ID_BYTES]);
        let mut table = RoutingTable::new(local);

        for value in 0..BUCKET_SIZE {
            let mut id = [0u8; NODE_ID_BYTES];
            id[0] = 0x80;
            id[NODE_ID_BYTES - 1] = value as u8;
            assert!(
                table
                    .insert(Node::from_id(NodeID::from_id(id), "a".into(), 6881))
                    .is_ok()
            );
        }

        let mut id = [0u8; NODE_ID_BYTES];
        id[0] = 0x80;
        id[NODE_ID_BYTES - 1] = BUCKET_SIZE as u8;
        assert_eq!(
            table.insert(Node::from_id(NodeID::from_id(id), "a".into(), 6881)),
            Err(DhtError::Bucket(BucketError::Full))
        );
    }

    #[test]
    fn test_routing_table_insert() {
        let local = NodeID::from_id([0u8; NODE_ID_BYTES]);

        let mut table = RoutingTable::new(local);

        let mut id = [0u8; NODE_ID_BYTES];
        id[NODE_ID_BYTES - 1] = 1;
        let result = table.insert(Node::from_id(NodeID::from_id(id), "127.0.0.1".into(), 6881));
        assert!(result.is_ok());
        assert_eq!(table.buckets[0].nodes().count(), 1);
    }

    #[test]
    fn test_find_closest() {
        let local = NodeID::from_id([0; NODE_ID_BYTES]);

        let mut table = RoutingTable::new(local);

        let mut id1 = [0; NODE_ID_BYTES];
        id1[NODE_ID_BYTES - 1] = 1;

        let mut id2 = [0; NODE_ID_BYTES];
        id2[NODE_ID_BYTES - 1] = 2;

        let result = table.insert(Node::from_id(NodeID::from_id(id1), "a".into(), 1));
        assert!(result.is_ok());

        let result = table.insert(Node::from_id(NodeID::from_id(id2), "b".into(), 2));
        assert!(result.is_ok());

        let result = table.find_closest(&NodeID::from_id([0; NODE_ID_BYTES]), 1);

        let node = result[0].id().node_id();
        assert_eq!(node[NODE_ID_BYTES - 1], 1);
    }
}
