//! Public configuration types for the BitTorrent engine.

mod macros;
mod settings;

pub use settings::*;

pub(crate) struct EngineConfig {
    settings_pack: SettingsPack,
}

impl EngineConfig {
    pub(crate) fn new() -> Self {
        Self {
            settings_pack: SettingsPack::default(),
        }
    }

    pub(crate) fn with_settings_pack(mut self, settings_pack: SettingsPack) -> Self {
        self.settings_pack = settings_pack;
        self
    }

    pub(crate) fn settings_pack(&self) -> SettingsPack {
        self.settings_pack.clone()
    }
}
