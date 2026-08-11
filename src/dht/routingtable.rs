use crate::dht::{DhtError, RoutingTableError, node::Node};

use super::{bucket::Bucket, node::NodeID};

/// The routing table for a node in the DHT.
///
/// The table starts with one bucket covering the entire node ID space. A full
/// bucket that contains the local node ID is split into two child ranges.
///
/// Each bucket contains the nodes whose identifiers fall within its prefix
/// range.
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
    /// A routing table containing one empty root bucket.
    pub fn new(local_id: NodeID) -> Self {
        Self {
            local_id,
            buckets: Bucket::new_bucket_collection(),
        }
    }

    /// Records a node observation in the routing table.
    ///
    /// Observing an existing node ID refreshes its endpoint and routing
    /// metadata without adding another entry.
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
    /// [`crate::dht::BucketError::Full`] if `node` has a new ID and the target
    /// bucket has reached its capacity but cannot be split because it does not
    /// contain the local node ID or has reached the maximum prefix depth.
    ///
    pub fn insert(&mut self, node: Node) -> Result<(), DhtError> {
        if node.id() == &self.local_id {
            return Err(DhtError::RoutingTable(RoutingTableError::NodeIsSelf));
        }

        loop {
            let index = self
                .buckets
                .iter()
                .position(|bucket| bucket.contains(node.id()))
                .expect("every node ID must belong to a bucket");

            match self.buckets[index].observe(node.clone()) {
                Ok(()) => return Ok(()),
                Err(crate::dht::BucketError::Full) => {
                    if !self.buckets[index].contains(&self.local_id)
                        || !self.buckets[index].can_split()
                    {
                        return Err(DhtError::Bucket(crate::dht::BucketError::Full));
                    }

                    let bucket = self.buckets.remove(index);
                    let (zero, one) = bucket.split();
                    self.buckets.insert(index, one);
                    self.buckets.insert(index, zero);
                }
            }
        }
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
    use crate::dht::{BucketError, NODE_ID_BYTES, bucket::BUCKET_SIZE};

    use super::*;

    #[test]
    fn test_new() {
        let local = NodeID::random();
        let rt = RoutingTable::new(local);
        assert_eq!(rt.buckets.len(), 1);
        assert!(rt.buckets[0].contains(&local));
        assert!(rt.buckets[0].nodes().next().is_none());
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
    fn test_full_bucket_containing_local_id_is_split() {
        let local = NodeID::from_id([0u8; 20]);
        let mut table = RoutingTable::new(local);

        for value in 0..BUCKET_SIZE - 1 {
            let mut id = [0u8; 20];
            id[0] = 0x80;
            id[19] = value as u8;
            table
                .insert(Node::from_id(NodeID::from_id(id), "a".into(), 6881))
                .unwrap();
        }

        let mut left_id = [0u8; 20];
        left_id[19] = 1;
        table
            .insert(Node::from_id(NodeID::from_id(left_id), "b".into(), 6882))
            .unwrap();

        let mut new_id = [0u8; 20];
        new_id[0] = 0x80;
        new_id[19] = 100;
        table
            .insert(Node::from_id(NodeID::from_id(new_id), "c".into(), 6883))
            .unwrap();

        assert_eq!(table.buckets.len(), 2);
        assert!(
            table
                .buckets
                .iter()
                .all(|bucket| bucket.nodes().count() <= BUCKET_SIZE)
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
