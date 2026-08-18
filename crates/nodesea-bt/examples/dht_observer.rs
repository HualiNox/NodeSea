//! Simple observer example that polls and prints BitTorrent engine events.
use nodesea_bt::{BtEvent, Engine, EventSink};

struct Printer;

// A simple event sink that prints all received events.
impl EventSink for Printer {
    fn on_event(&mut self, event: BtEvent) {
        println!("{event:?}");
    }
}

fn main() {
    // Create the BitTorrent engine and start polling for DHT events
    let mut engine = Engine::new().expect("failed to initialize engine");
    let mut sink = Printer;

    // Start polling loop
    loop {
        engine.post_dht_stats();
        engine.poll_events(&mut sink);

        // Wait before next poll
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
