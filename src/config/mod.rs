pub mod graphics;
pub mod audio;
pub mod physics;
pub mod network;
pub mod ui;
pub mod engine;
pub mod parser;

pub use graphics::GraphicsConfig;
pub use audio::AudioConfig;
pub use physics::PhysicsConfig;
pub use network::NetworkConfig;
pub use ui::{UiConfig, ConfigColor};
pub use engine::EngineConfig;
pub use parser::{ensure_config_file_exists, parse_bool, parse_f32, parse_ini_file, parse_u32};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parser::{DEFAULT_ENGINE_DEFAULTS, ensure_config_file_exists, parse_ini_file};

    #[test]
    fn test_global_engine_configuration_initialization() {
        let config = EngineConfig::default();

        assert_eq!(config.graphics.width, GraphicsConfig::default().width);
        assert_eq!(config.physics.tick_rate_hz, PhysicsConfig::default().tick_rate_hz);
        assert_eq!(config.network.connection_port, NetworkConfig::default().connection_port);
        assert_eq!(config.audio.spatial_max_distance, AudioConfig::default().spatial_max_distance);
    }

    #[test]
    fn test_engine_preset_generation() {
        let racing_config = EngineConfig::sim_racing_preset();

        assert_eq!(racing_config.physics.tick_rate_hz, 360.0);
        assert_eq!(racing_config.graphics.vsync, false);
        assert_eq!(racing_config.graphics.width, 1920);
        assert_eq!(racing_config.graphics.shadow_map_resolution, 4096);
    }

    #[test]
    fn test_engine_config_loads_default_ini_seed() {
        let path = ensure_config_file_exists("engine-defaults.ini", DEFAULT_ENGINE_DEFAULTS)
            .expect("default INI should be created");
        let ini_data = parse_ini_file(&path).expect("default INI should parse");
        let config = EngineConfig::from_ini_data(&ini_data);

        assert_eq!(config.graphics.width, 1280);
        assert_eq!(config.audio.master_volume, 1.0);
        assert_eq!(config.physics.tick_rate_hz, 120.0);
        assert_eq!(config.network.server_address, "127.0.0.1");
        assert_eq!(config.ui.primary_color.r, 0.12);
    }
}