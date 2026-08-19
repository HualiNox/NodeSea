//! Private session/error CXX wire models and their domain conversions.

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// Payload containing a session or protocol error message.
    pub(super) struct MessagePayload {
        message: String,
    }
}

pub(super) use bridge::MessagePayload;

impl From<bridge::MessagePayload> for String {
    fn from(value: bridge::MessagePayload) -> Self {
        value.message
    }
}
