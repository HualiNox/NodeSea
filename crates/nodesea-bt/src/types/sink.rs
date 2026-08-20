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
