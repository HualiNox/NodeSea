//! Generated gRPC bindings shared by the NodeSea daemon and clients.
//!
//! The source schemas live under `proto/nodesea/v1`. Generated types are
//! versioned by module so a future protocol version can coexist without
//! changing existing clients.

/// Version 1 of the capability-oriented local daemon API.
pub mod v1 {
    tonic::include_proto!("nodesea.v1");
}
