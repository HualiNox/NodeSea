//! Builder API for configuring and creating a BitTorrent engine.

use crate::SettingsPack;

/// Builder for a BitTorrent engine.
pub struct EngineBuilder {
    settings: SettingsPack,
}

impl EngineBuilder {
    /// Creates a builder with an empty settings pack.
    pub fn new() -> Self {
        Self {
            settings: SettingsPack::new(),
        }
    }

    /// Replaces the settings pack used to create the engine.
    pub fn set_settings_pack(mut self, settings: SettingsPack) -> Self {
        self.settings = settings;
        self
    }

    /// Creates the engine with the configured settings.
    pub fn build(self) -> Result<super::Engine, String> {
        super::Engine::new(self.settings)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
