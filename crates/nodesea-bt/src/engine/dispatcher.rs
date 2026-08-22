//! Synchronous event fan-out for engine extensions.

use crate::{EventSink, engine::extension::EngineExtensionBox};

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
    /// Delivers one event to every registered extension.
    fn on_event(&mut self, event: crate::BtEvent) {
        for extension in &mut self.extensions {
            extension.on_event(&event);
        }
    }
}
