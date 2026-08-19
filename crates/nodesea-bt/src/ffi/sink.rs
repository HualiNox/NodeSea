use crate::{BtEvent, EventSink};

/// Rust-owned adapter passed to C++ for synchronous alert dispatch.
pub(super) struct FfiEventSink {
    data: *mut (),
    emit_fn: unsafe fn(*mut (), BtEvent),
}

impl FfiEventSink {
    /// Creates a synchronous adapter for a concrete Rust event sink.
    ///
    /// The raw pointer is valid only while the caller is inside the native
    /// `poll_events` call; C++ must not store the adapter or invoke it later.
    pub(super) fn new<S: EventSink>(sink: &mut S) -> Self {
        Self {
            data: sink as *mut S as *mut (),
            emit_fn: emit::<S>,
        }
    }

    pub(super) fn emit(&mut self, event: BtEvent) {
        // The adapter is created and borrowed only for the duration of one
        // synchronous poll_events call. C++ never stores this pointer.
        unsafe { (self.emit_fn)(self.data, event) }
    }
}

unsafe fn emit<S: EventSink>(data: *mut (), event: BtEvent) {
    // `data` points to the `S` passed to `FfiEventSink::new`. The adapter is
    // used synchronously, so that value remains valid for every callback.
    unsafe { (&mut *data.cast::<S>()).on_event(event) };
}
