//! Simple observer example that polls and prints BitTorrent engine events.
use nodesea_bt::{BtEvent, DhtTarget, Engine, EventSink, InfoHash};
use std::{collections::HashSet, net::SocketAddr};

/// Example event sink that observes DHT activity and schedules metadata
/// requests for discovered infohashes.
struct Observer {
    /// Infohashes waiting for metadata fetches.
    pending: Vec<InfoHash>,
    /// Infohashes already seen through DHT announce/get-peers events.
    seen: HashSet<InfoHash>,
    /// DHT endpoints discovered from live-node snapshots.
    nodes: HashSet<SocketAddr>,
    /// Endpoints sampled once by this example.
    sampled: HashSet<SocketAddr>,

    /// Whether the initial DHT bootstrap event has been observed.
    dht_bootstrapped: bool,
}

impl EventSink for Observer {
    fn on_event(&mut self, event: BtEvent) {
        match event {
            BtEvent::DhtAnnounce { info_hash, .. } => {
                println!("dht announce: {info_hash}");
                if self.seen.insert(info_hash) {
                    self.pending.push(info_hash);
                }
            }

            BtEvent::DhtGetPeers { info_hash } => {
                println!("dht getpeers: {info_hash}");
                if self.seen.insert(info_hash) {
                    self.pending.push(info_hash);
                }
            }

            BtEvent::MetadataReceived { info_hash, data } => {
                println!("metadata: {info_hash}, {} bytes", data.len());
            }

            BtEvent::MetadataFailed { info_hash, message } => {
                println!("metadata failed: {info_hash}: {message}");
            }

            BtEvent::DhtStats {
                node_count,
                local_ip,
                local_port,
            } => {
                println!("DHT stats: {node_count} nodes, listening on {local_ip}:{local_port}");
            }

            BtEvent::DhtBootstrap => {
                println!("DHT bootstrap completed");
                self.dht_bootstrapped = true;
            }

            BtEvent::DhtSampleInfohashes {
                node,
                interval,
                num_infohashes,
                samples,
                nodes,
            } => {
                println!(
                    "dht sample: node={}, interval={:?}, num_infohashes={}, samples={}, nodes={}",
                    node.endpoint,
                    interval,
                    num_infohashes,
                    samples.len(),
                    nodes.len()
                );
            }
            BtEvent::DhtPkt {
                endpoint,
                direction,
                packet,
            } => {
                self.nodes.insert(endpoint);
                println!(
                    "dht pkt: endpoint={}, direction={:?}, packet_len={}",
                    endpoint,
                    direction,
                    packet.len()
                );
            }
            BtEvent::DhtLiveNodes {
                local_node_id,
                nodes,
            } => {
                println!(
                    "dht live nodes: local_node_id={}, nodes={}",
                    local_node_id,
                    nodes.len()
                );
                for node in nodes {
                    self.nodes.insert(node.endpoint);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    // Create the BitTorrent engine and start polling for DHT events
    let mut engine = Engine::new().expect("failed to initialize engine");
    let mut observer = Observer {
        pending: Vec::new(),
        seen: HashSet::new(),
        dht_bootstrapped: false,
        sampled: HashSet::new(),
        nodes: HashSet::new(),
    };

    // Start polling loop
    loop {
        engine.post_dht_stats();

        if observer.dht_bootstrapped {
            engine.post_dht_live_nodes();
            if let Some(node) = observer
                .nodes
                .iter()
                .find(|node| !observer.sampled.contains(node))
                .copied()
            {
                println!("sampling from {node}");

                if engine.post_dht_sample_infohashes(&node, &DhtTarget::from_bytes([0; 20])) {
                    observer.sampled.insert(node);
                }
            }
        }

        engine.poll_events(&mut observer);

        for hash in observer.pending.drain(..) {
            if engine.fetch_metadata(&hash) {
                println!("fetching {hash}");
            }
        }

        // Wait before next poll
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
