//! Builder API for configuring and creating a BitTorrent engine.

use tokio::sync::{mpsc, watch};

use crate::{
    EngineConfig, SettingsPack,
    engine::{Engine, EngineExtension, extension::EngineExtensionBox, runner::EngineStatus},
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
        // Create a channel for sending commands to the engine runner.
        let (command_tx, command_rx) = mpsc::channel(128);
        // Create a channel for broadcasting the engine status to external observers.
        let (status_tx, status_rx) = watch::channel(EngineStatus::Idle);

        Engine::new(
            EngineConfig::new().with_settings_pack(self.settings),
            self.extensions,
            command_tx,
            command_rx,
            status_tx,
            status_rx,
        )
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
