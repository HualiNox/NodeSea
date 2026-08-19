//! Private session/error CXX wire models and their domain conversions.

use crate::BtEvent;

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {
    /// Payload for a session error alert.
    pub(super) struct SessionErrorPayload {
        /// Human-readable error description.
        message: String,
    }

    /// Payload for a listen failure alert.
    pub(super) struct ListenFailedPayload {
        /// Human-readable error description.
        message: String,
    }

    /// Payload for a UDP error alert.
    pub(super) struct UdpErrorPayload {
        /// Human-readable error description.
        message: String,
    }

    /// Payload for a DHT error alert.
    pub(super) struct DhtErrorPayload {
        /// Human-readable error description.
        message: String,
    }

    /// Payload for an alerts-dropped alert.
    pub(super) struct AlertsDroppedPayload {
        /// Human-readable error description.
        message: String,
    }
}

pub(super) use bridge::{
    AlertsDroppedPayload, DhtErrorPayload, ListenFailedPayload, SessionErrorPayload,
    UdpErrorPayload,
};

impl From<bridge::SessionErrorPayload> for BtEvent {
    fn from(value: bridge::SessionErrorPayload) -> Self {
        Self::SessionError {
            message: value.message,
        }
    }
}

impl From<bridge::ListenFailedPayload> for BtEvent {
    fn from(value: bridge::ListenFailedPayload) -> Self {
        Self::ListenFailed {
            message: value.message,
        }
    }
}

impl From<bridge::UdpErrorPayload> for BtEvent {
    fn from(value: bridge::UdpErrorPayload) -> Self {
        Self::UdpError {
            message: value.message,
        }
    }
}

impl From<bridge::DhtErrorPayload> for BtEvent {
    fn from(value: bridge::DhtErrorPayload) -> Self {
        Self::DhtError {
            message: value.message,
        }
    }
}

impl From<bridge::AlertsDroppedPayload> for BtEvent {
    fn from(value: bridge::AlertsDroppedPayload) -> Self {
        Self::AlertsDropped {
            message: value.message,
        }
    }
}
