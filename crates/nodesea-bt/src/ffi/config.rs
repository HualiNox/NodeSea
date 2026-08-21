//! CXX-compatible representation of the Rust settings pack.

use crate::{Setting, SettingsPack};

#[cxx::bridge(namespace = "nodesea::bt")]
mod bridge {

    /// The value type carried by a setting payload.
    pub(super) enum SettingKind {
        Bool,
        Int,
        String,
    }

    /// One typed setting update passed to the native engine.
    pub(super) struct SettingPayload {
        /// The setting value category.
        kind: SettingKind,
        /// The libtorrent setting key, including its type prefix.
        key: u16,
        /// The integer value; unused for other setting kinds.
        int_value: i64,
        /// The boolean value; unused for other setting kinds.
        bool_value: bool,
        /// The string value; empty for other setting kinds.
        string_value: String,
    }

    /// The collection of setting updates consumed by the native engine.
    pub(super) struct SettingsPackPayload {
        /// Settings are applied in their stored order.
        settings: Vec<SettingPayload>,
    }
}

pub(super) use bridge::SettingsPackPayload;

impl From<Setting> for bridge::SettingPayload {
    fn from(setting: Setting) -> Self {
        match setting {
            Setting::Bool(key, value) => bridge::SettingPayload {
                kind: bridge::SettingKind::Bool,
                key: key as u16,
                int_value: 0,
                bool_value: value,
                string_value: String::new(),
            },
            Setting::Int(key, value) => bridge::SettingPayload {
                kind: bridge::SettingKind::Int,
                key: key as u16,
                int_value: value,
                bool_value: false,
                string_value: String::new(),
            },
            Setting::String(key, value) => bridge::SettingPayload {
                kind: bridge::SettingKind::String,
                key: key as u16,
                int_value: 0,
                bool_value: false,
                string_value: value,
            },
        }
    }
}

impl From<SettingsPack> for bridge::SettingsPackPayload {
    fn from(settings_pack: SettingsPack) -> Self {
        bridge::SettingsPackPayload {
            settings: settings_pack
                .into_values()
                .into_iter()
                .map(bridge::SettingPayload::from)
                .collect(),
        }
    }
}
