use nodesea::dht::{DhtError, DhtNode, Node, NodeID, RoutingTable, RoutingTableError};

#[test]
fn public_dht_api_supports_routing_table_queries() {
    let local = NodeID::from_id([0u8; 20]);
    let mut table = RoutingTable::new(local);

    let mut remote_id = [0u8; 20];
    remote_id[19] = 1;
    assert!(
        table
            .insert(Node::from_id(
                NodeID::from_id(remote_id),
                "127.0.0.1".into(),
                6881,
            ))
            .is_ok()
    );

    let target = NodeID::from_id([0u8; 20]);
    let closest = table.find_closest(&target, 1);

    assert_eq!(closest.len(), 1);
    assert_eq!(closest[0].id().node_id(), &remote_id);
}

#[test]
fn public_dht_node_wraps_local_identity_and_routing_table() {
    let local_id = NodeID::from_id([0u8; 20]);
    let local = Node::from_id(local_id, "127.0.0.1".into(), 6881);
    let mut dht_node = DhtNode::new(local.clone());

    assert_eq!(dht_node.id(), local.id());
    assert_eq!(dht_node.local(), &local);
    assert!(
        dht_node
            .routing_table()
            .find_closest(&local_id, 1)
            .is_empty()
    );

    let mut remote_id = [0u8; 20];
    remote_id[19] = 1;
    let remote = Node::from_id(NodeID::from_id(remote_id), "127.0.0.2".into(), 6882);

    assert!(dht_node.insert(remote).is_ok());
    let closest = dht_node.routing_table().find_closest(&local_id, 1);

    assert_eq!(closest.len(), 1);
    assert_eq!(closest[0].id().node_id(), &remote_id);
}

#[test]
fn public_dht_node_rejects_inserting_itself() {
    let local_id = NodeID::from_id([0u8; 20]);
    let local = Node::from_id(local_id, "127.0.0.1".into(), 6881);
    let mut dht_node = DhtNode::new(local.clone());

    assert_eq!(
        dht_node.insert(local),
        Err(DhtError::RoutingTable(RoutingTableError::NodeIsSelf))
    );
}
