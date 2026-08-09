use crate::dht::{NODE_ID_BYTES, errors::BucketError, node::Node};

/// The number of buckets in the routing table.
pub(crate) const BUCKET_COUNT: usize = NODE_ID_BYTES * 8;
/// The maximum number of nodes that can be stored in a bucket.
pub(crate) const BUCKET_SIZE: usize = 20;

/// Stores nodes belonging to one XOR-distance range in a routing table.
#[derive(Clone, Debug)]
pub(crate) struct Bucket {
    nodes: Vec<Node>,
}

impl Bucket {
    /// Creates an empty bucket.
    ///
    /// # Returns
    ///
    /// A new `Bucket` instance with an empty list of nodes.
    pub(crate) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates a new collection of empty buckets.
    ///
    /// # Returns
    ///
    /// A vector of `Bucket` instances, each initialized with an empty list of nodes.
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
    /// Returns [`BucketError::Full`] if the
    /// bucket already contains [`BUCKET_SIZE`] nodes.
    pub(crate) fn add(&mut self, node: Node) -> Result<(), BucketError> {
        if self.nodes.len() < BUCKET_SIZE {
            self.nodes.push(node);
            Ok(())
        } else {
            Err(BucketError::Full)
        }
    }

    /// Returns an immutable view of the nodes in the bucket.
    ///
    /// # Returns
    ///
    /// A slice containing the nodes currently stored in the bucket.
    pub(crate) fn nodes(&self) -> &[Node] {
        &self.nodes
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
        assert_eq!(bucket.nodes.len(), 1);
    }

    #[test]
    fn test_add_overflow() {
        let mut bucket = Bucket::new();
        for p in 0..BUCKET_SIZE {
            let result = bucket.add(Node::random(String::from("127.0.0.1"), p as u16));
            assert!(result.is_ok())
        }
        for p in BUCKET_SIZE..2 * BUCKET_SIZE {
            let result = bucket.add(Node::random(String::from("127.0.0.1"), p as u16));
            assert!(result.is_err())
        }
        assert_eq!(bucket.nodes.len(), BUCKET_SIZE);
    }
}
