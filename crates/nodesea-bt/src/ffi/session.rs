//! Private session/error CXX wire models and their domain conversions.

use crate::{
    AlertsDropped, BtEvent, BtEventKind, DhtError, ExternalIp, ListenFailed, SessionError,
    SessionLog, SessionStats, UdpError,
};

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

    /// Payload for session statistics.
    pub(super) struct SessionStatsPayload {
        /// Counter values in libtorrent metric order.
        counters: Vec<i64>,
        /// Human-readable statistics message.
        message: String,
    }

    /// Payload for the external IP alert.
    pub(super) struct ExternalIpPayload {
        /// External IP address reported by libtorrent.
        address: String,
        /// Human-readable alert message.
        message: String,
    }

    /// Payload for a session log alert.
    pub(super) struct SessionLogPayload {
        /// Log message reported by libtorrent.
        message: String,
    }
}

pub(super) use bridge::{
    AlertsDroppedPayload, DhtErrorPayload, ExternalIpPayload, ListenFailedPayload,
    SessionErrorPayload, SessionLogPayload, SessionStatsPayload, UdpErrorPayload,
};

impl From<bridge::SessionErrorPayload> for BtEvent {
    fn from(value: bridge::SessionErrorPayload) -> Self {
        Self::new(BtEventKind::SessionError(SessionError::from_ffi(
            value.message,
        )))
    }
}

impl From<bridge::ListenFailedPayload> for BtEvent {
    fn from(value: bridge::ListenFailedPayload) -> Self {
        Self::new(BtEventKind::ListenFailed(ListenFailed::from_ffi(
            value.message,
        )))
    }
}

impl From<bridge::UdpErrorPayload> for BtEvent {
    fn from(value: bridge::UdpErrorPayload) -> Self {
        Self::new(BtEventKind::UdpError(UdpError::from_ffi(value.message)))
    }
}

impl From<bridge::DhtErrorPayload> for BtEvent {
    fn from(value: bridge::DhtErrorPayload) -> Self {
        Self::new(BtEventKind::DhtError(DhtError::from_ffi(value.message)))
    }
}

impl From<bridge::AlertsDroppedPayload> for BtEvent {
    fn from(value: bridge::AlertsDroppedPayload) -> Self {
        Self::new(BtEventKind::AlertsDropped(AlertsDropped::from_ffi(
            value.message,
        )))
    }
}

impl From<bridge::SessionStatsPayload> for BtEvent {
    fn from(value: bridge::SessionStatsPayload) -> Self {
        Self::new(BtEventKind::SessionStats(SessionStats::from_ffi(
            value.counters,
            value.message,
        )))
    }
}

impl From<bridge::ExternalIpPayload> for BtEvent {
    fn from(value: bridge::ExternalIpPayload) -> Self {
        Self::new(BtEventKind::ExternalIp(ExternalIp::from_ffi(
            value.address,
            value.message,
        )))
    }
}

impl From<bridge::SessionLogPayload> for BtEvent {
    fn from(value: bridge::SessionLogPayload) -> Self {
        Self::new(BtEventKind::SessionLog(SessionLog::from_ffi(value.message)))
    }
}
