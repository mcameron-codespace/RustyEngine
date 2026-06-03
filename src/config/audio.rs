use std::collections::HashMap;

use super::parser::parse_f32;

/// Configuration parameters for the Kira spatial audio mixer.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub spatial_min_distance: f32,
    pub spatial_max_distance: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
            spatial_min_distance: 1.0,
            spatial_max_distance: 50.0,
        }
    }
}

impl AudioConfig {
    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        let defaults = Self::default();
        let section = ini_data.get("Audio").cloned().unwrap_or_default();

        Self {
            master_volume: section
                .get("master_volume")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.master_volume),
            music_volume: section
                .get("music_volume")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.music_volume),
            sfx_volume: section
                .get("sfx_volume")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.sfx_volume),
            spatial_min_distance: section
                .get("spatial_min_distance")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.spatial_min_distance),
            spatial_max_distance: section
                .get("spatial_max_distance")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.spatial_max_distance),
        }
    }
}