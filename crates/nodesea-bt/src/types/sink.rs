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
