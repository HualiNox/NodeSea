use nodesea::dht::{Node, NodeID, RoutingTable};

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
