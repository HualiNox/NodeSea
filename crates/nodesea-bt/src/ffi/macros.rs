//! Private macros for repetitive CXX callback dispatch glue.

macro_rules! info_message_callback {
    ($method:ident, $variant:ident) => {
        fn $method(&mut self, event: bridge::InfoMessagePayload) {
            let (info_hash, message): (InfoHash, String) = event.into();
            self.emit(BtEvent::$variant { info_hash, message });
        }
    };
}

macro_rules! message_callback {
    ($method:ident, $variant:ident) => {
        fn $method(&mut self, event: bridge::MessagePayload) {
            self.emit(BtEvent::$variant {
                message: event.into(),
            });
        }
    };
}
