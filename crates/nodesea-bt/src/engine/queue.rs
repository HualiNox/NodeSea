//! Event buffering used by the single-event engine polling API.

use std::collections::VecDeque;

use crate::{BtEvent, EventSink};

/// A queue-based event sink that buffers events in a deque.
pub(super) struct QueueSink<'a> {
    /// Destination buffer for callbacks received during one poll.
    pub(super) buffer: &'a mut VecDeque<BtEvent>,
}

impl EventSink for QueueSink<'_> {
    fn on_event(&mut self, event: BtEvent) {
        self.buffer.push_back(event);
    }
}
