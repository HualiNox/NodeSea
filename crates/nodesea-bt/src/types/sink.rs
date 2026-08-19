use super::event::BtEvent;

/// Receives events produced by the BitTorrent engine.
pub trait EventSink {
    /// Handles one event synchronously.
    ///
    /// The event is owned by the sink after this call returns. Implementations
    /// should not retain references into FFI payloads.
    fn on_event(&mut self, event: BtEvent);
}

/// A simple event sink that stores received events in a vector.
#[derive(Debug, Default)]
pub struct EventCollector {
    events: Vec<BtEvent>,
}

impl EventSink for EventCollector {
    fn on_event(&mut self, event: BtEvent) {
        self.events.push(event);
    }
}

impl EventCollector {
    /// Creates a new, empty event collector.
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Creates a new event collector with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Returns a slice of the collected events.
    pub fn events(&self) -> &[BtEvent] {
        &self.events
    }

    /// Takes the collected events out of the collector, leaving it empty.
    pub fn take_events(&mut self) -> Vec<BtEvent> {
        std::mem::take(&mut self.events)
    }

    /// Clears the collector.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
