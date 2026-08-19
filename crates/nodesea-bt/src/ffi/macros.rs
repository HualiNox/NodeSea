//! Private macros for repetitive CXX callback dispatch glue.

macro_rules! event_callback {
    ($method:ident, $payload:ty) => {
        fn $method(&mut self, event: $payload) {
            self.emit(event.into());
        }
    };
}
