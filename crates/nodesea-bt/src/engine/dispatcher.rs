//! Bounded event queue and dedicated extension worker.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::mpsc::{self, SyncSender},
    thread::{self, JoinHandle},
};

use crate::{BtEvent, engine::extension::EngineExtensionBox, types::EventSink};

const EVENT_QUEUE_CAPACITY: usize = 1024;

/// The queue is intentionally bounded so a slow extension cannot grow memory
/// without limit or block the native alert consumer.
/// Queues events for a dedicated extension worker.
pub(crate) struct EventDispatcher {
    sender: Option<SyncSender<BtEvent>>,
    join: Option<JoinHandle<()>>,
}

impl EventDispatcher {
    /// Creates a dispatcher and starts its dedicated extension worker.
    pub(crate) fn new(mut extensions: Vec<EngineExtensionBox>) -> Self {
        let (sender, receiver) = mpsc::sync_channel(EVENT_QUEUE_CAPACITY);
        let join = thread::Builder::new()
            .name("nodesea-event".to_owned())
            .spawn(move || {
                while let Ok(event) = receiver.recv() {
                    for extension in &mut extensions {
                        let _ = catch_unwind(AssertUnwindSafe(|| {
                            extension.on_event(&event);
                        }));
                    }
                }
            })
            .expect("event worker thread should start");

        Self {
            sender: Some(sender),
            join: Some(join),
        }
    }

    /// Stops the worker after all queued events have been processed.
    pub(crate) fn shutdown(&mut self) {
        // Dropping the last sender closes the receiver after all queued events
        // have been drained, then joining provides a deterministic stop point.
        self.sender.take();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl EventSink for EventDispatcher {
    /// Enqueues one event without blocking the native runner.
    ///
    /// Events are dropped when the bounded queue is full. This is deliberate:
    /// event delivery must never delay command execution or alert polling.
    fn on_event(&mut self, event: BtEvent) {
        if let Some(sender) = &self.sender {
            let _ = sender.try_send(event);
        }
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        self.shutdown();
    }
}
