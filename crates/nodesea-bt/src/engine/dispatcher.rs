//! Synchronous event fan-out for engine extensions.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{BtEvent, engine::extension::EngineExtensionBox, types::EventSink};

/// Dispatches each event to the configured engine extensions in registration
/// order.
pub(crate) struct EventDispatcher {
    extensions: Vec<EngineExtensionBox>,
}

impl EventDispatcher {
    /// Creates a dispatcher from the configured extensions.
    pub(crate) fn new(extensions: Vec<EngineExtensionBox>) -> Self {
        Self { extensions }
    }
}

impl EventSink for EventDispatcher {
    /// Delivers one event to each registered extension in order.
    ///
    /// An extension panic is isolated from the engine runner. The extension
    /// remains registered, so later events may invoke it again.
    fn on_event(&mut self, event: BtEvent) {
        for extension in &mut self.extensions {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                extension.on_event(&event);
            }));
        }
    }
}
