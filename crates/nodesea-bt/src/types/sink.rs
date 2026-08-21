//! Public event sink traits and in-memory event collection.

use super::event::BtEvent;

/// Receives events produced by the BitTorrent engine.
pub trait EventSink {
    /// Handles one event synchronously.
    ///
    /// The event is owned by the sink after this call returns. Implementations
    /// should not retain references into FFI payloads.
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`&mut Self`) - The event sink receiving the event.
    /// - `event` (`BtEvent`) - The event to handle.
    ///
    /// # Returns
    ///
    /// - `()` - Nothing; the sink handles the event synchronously.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::{BtEvent, EventSink};
    ///
    /// struct Sink;
    /// impl EventSink for Sink {
    ///     fn on_event(&mut self, _event: BtEvent) {}
    /// }
    /// ```
    fn on_event(&mut self, event: BtEvent);
}

/// A simple event sink that stores received events in a vector.
#[derive(Debug, Default)]
pub struct EventCollector {
    /// Buffered domain events in arrival order.
    events: Vec<BtEvent>,
}

impl EventSink for EventCollector {
    fn on_event(&mut self, event: BtEvent) {
        self.events.push(event);
    }
}

impl EventCollector {
    /// Creates a new, empty event collector.
    ///
    /// # Arguments
    ///
    /// This function takes no arguments.
    ///
    /// # Returns
    ///
    /// - `Self` - An empty event collector.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::EventCollector;
    ///
    /// let _collector = EventCollector::new();
    /// ```
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Creates a new event collector with the specified capacity.
    ///
    /// # Arguments
    ///
    /// - `capacity` (`usize`) - The initial capacity of the event buffer.
    ///
    /// # Returns
    ///
    /// - `Self` - An empty event collector with the requested capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::EventCollector;
    ///
    /// let _collector = EventCollector::with_capacity(16);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Returns a slice of the collected events.
    ///
    /// # Arguments
    ///
    /// - `&self` (`&Self`) - The event collector to inspect.
    ///
    /// # Returns
    ///
    /// - `&[BtEvent]` - The collected events in arrival order.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::EventCollector;
    ///
    /// let collector = EventCollector::new();
    /// let _events = collector.events();
    /// ```
    pub fn events(&self) -> &[BtEvent] {
        &self.events
    }

    /// Takes the collected events out of the collector, leaving it empty.
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`&mut Self`) - The event collector to drain.
    ///
    /// # Returns
    ///
    /// - `Vec<BtEvent>` - The collected events in arrival order.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::EventCollector;
    ///
    /// let mut collector = EventCollector::new();
    /// let _events = collector.take_events();
    /// ```
    pub fn take_events(&mut self) -> Vec<BtEvent> {
        std::mem::take(&mut self.events)
    }

    /// Clears the collector.
    ///
    /// # Arguments
    ///
    /// - `&mut self` (`&mut Self`) - The event collector to clear.
    ///
    /// # Returns
    ///
    /// - `()` - Nothing; all buffered events are removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use nodesea_bt::EventCollector;
    ///
    /// let mut collector = EventCollector::new();
    /// collector.clear();
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BtEventKind, DhtInfoHash};

    #[test]
    fn test_event_collector_sink() {
        let mut collector = EventCollector::new();
        assert!(collector.events().is_empty());

        let info_hash = DhtInfoHash::from_bytes([0xab; 20]);
        let get_peers_event = BtEvent::new(BtEventKind::DhtGetPeers(
            super::super::event::DhtGetPeers::from_ffi(info_hash),
        ));
        let announce_event = BtEvent::new(BtEventKind::DhtAnnounce(
            super::super::event::DhtAnnounce::from_ffi(info_hash, "127.0.0.1".to_string(), 6881),
        ));

        collector.on_event(BtEvent::new(BtEventKind::DhtBootstrap(
            super::super::event::DhtBootstrap::from_ffi(),
        )));
        collector.on_event(get_peers_event.clone());
        collector.on_event(announce_event.clone());

        assert_eq!(collector.events().len(), 3);
        assert!(matches!(
            collector.events()[0].kind(),
            BtEventKind::DhtBootstrap(_)
        ));
        assert_eq!(collector.events()[1], get_peers_event);
        assert_eq!(collector.events()[2], announce_event);

        let taken = collector.take_events();
        assert_eq!(taken.len(), 3);
        assert!(collector.events().is_empty());

        collector.on_event(get_peers_event);
        assert_eq!(collector.events().len(), 1);
        collector.clear();
        assert!(collector.events().is_empty());
    }
}
