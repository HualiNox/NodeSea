//! Extension interface for observing engine events.

use crate::BtEvent;

/// Receives events synchronously from the engine dispatcher.
///
/// Implementations should return quickly. Long-running work should be handed
/// off to another task so that alert polling is not delayed. Panics raised by
/// an implementation are isolated by the dispatcher, but the implementation
/// remains registered for later events.
pub trait EngineExtension {
    /// Handles one engine event by reference.
    ///
    /// The callback runs synchronously on the engine runner. Implementations
    /// should avoid blocking operations and hand long-running work to another
    /// task.
    fn on_event(&mut self, event: &BtEvent);
}

/// A sendable boxed engine extension.
pub(crate) type EngineExtensionBox = Box<dyn EngineExtension + Send>;
