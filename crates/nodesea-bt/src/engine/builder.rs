//! Builder API for configuring and creating a BitTorrent engine.

use tokio::sync::mpsc;

use crate::{
    EngineConfig, SettingsPack,
    engine::{Engine, EngineExtension, extension::EngineExtensionBox},
};

/// Builder for a BitTorrent engine.
pub struct EngineBuilder {
    settings: SettingsPack,
    extensions: Vec<EngineExtensionBox>,
}

impl EngineBuilder {
    /// Creates a builder with an empty settings pack.
    pub fn new() -> Self {
        Self {
            settings: SettingsPack::new(),
            extensions: Vec::new(),
        }
    }

    /// Replaces the settings pack used to create the engine.
    pub fn set_settings_pack(mut self, settings: SettingsPack) -> Self {
        self.settings = settings;
        self
    }

    /// Adds an extension to the engine.
    pub fn add_extension<E>(mut self, extension: E) -> Self
    where
        E: EngineExtension + Send + 'static,
    {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Creates the engine with the configured settings.
    pub fn build(self) -> Engine {
        let (command_tx, command_rx) = mpsc::channel(128);

        Engine::new(
            EngineConfig::new().with_settings_pack(self.settings),
            self.extensions,
            command_tx,
            command_rx,
        )
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
