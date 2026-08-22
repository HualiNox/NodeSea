//! Internal event sink trait used by the FFI adapter and dispatcher.

use super::event::BtEvent;

/// Receives events produced by the BitTorrent engine internally.
pub(crate) trait EventSink {
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
    fn on_event(&mut self, event: BtEvent);
}
