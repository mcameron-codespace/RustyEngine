use std::collections::HashMap;

use super::graphics::GraphicsConfig;
use super::audio::AudioConfig;
use super::physics::PhysicsConfig;
use super::network::NetworkConfig;
use super::ui::UiConfig;

/// Unified configuration manager orchestrating the default settings for the entire Aether engine.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EngineConfig {
    pub graphics: GraphicsConfig,
    pub audio: AudioConfig,
    pub physics: PhysicsConfig,
    pub network: NetworkConfig,
    pub ui: UiConfig,
}

impl EngineConfig {
    /// Create a high-fidelity preset (e.g. for Sim Racing, high physics rates, larger resolution, vsync disabled).
    pub fn sim_racing_preset() -> Self {
        let mut config = Self::default();
        config.graphics.width = 1920;
        config.graphics.height = 1080;
        config.graphics.vsync = false;
        config.graphics.shadow_map_resolution = 4096;
        config.physics.tick_rate_hz = 360.0;
        config
    }

    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        Self {
            graphics: GraphicsConfig::from_ini_data(ini_data),
            audio: AudioConfig::from_ini_data(ini_data),
            physics: PhysicsConfig::from_ini_data(ini_data),
            network: NetworkConfig::from_ini_data(ini_data),
            ui: UiConfig::from_ini_data(ini_data),
        }
    }
}