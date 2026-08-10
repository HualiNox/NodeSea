/// Errors returned by DHT data structures.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DhtError {
    /// An error occurred while updating a bucket.
    #[error(transparent)]
    Bucket(#[from] BucketError),

    /// An error occurred while updating the routing table.
    #[error(transparent)]
    RoutingTable(#[from] RoutingTableError),
}

/// Errors returned when updating a routing-table bucket.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BucketError {
    /// The bucket has reached its maximum capacity.
    #[error("bucket is full")]
    Full,

    /// The exact node is already stored in the bucket.
    #[error("node is already in bucket")]
    NodeAlreadyInBucket,
}

/// Errors returned when updating a routing table.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RoutingTableError {
    /// The inserted node has the same identifier as the local node.
    #[error("node is self")]
    NodeIsSelf,
}
