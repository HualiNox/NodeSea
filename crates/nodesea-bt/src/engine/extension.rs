//! Extension interface for observing engine events.

use crate::BtEvent;

/// Receives events synchronously on the dedicated `nodesea-event` worker.
///
/// Events are delivered serially to each registered implementation. Implementations
/// should return quickly; long-running work should be handed off to another task
/// so that later events are not delayed. Panics raised by an implementation are
/// isolated by the dispatcher, but the implementation remains registered for
/// later events.
pub trait EngineExtension {
    /// Handles one engine event by reference.
    ///
    /// The callback runs synchronously on the `nodesea-event` worker, not on the
    /// native engine runner. Implementations should avoid blocking operations
    /// and hand long-running work to another task.
    fn on_event(&mut self, event: &BtEvent);
}

/// A sendable boxed engine extension.
pub(crate) type EngineExtensionBox = Box<dyn EngineExtension + Send>;
