use std::collections::HashMap;

use super::parser::{parse_bool, parse_u32};

/// Configuration parameters for the `wgpu` rendering layer and window context.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    /// Force OpenGL GLES fallback even if Vulkan/DirectX is available.
    pub force_opengl: bool,
    pub shadow_map_resolution: u32,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            vsync: true,
            force_opengl: false,
            shadow_map_resolution: 2048,
        }
    }
}

impl GraphicsConfig {
    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        let defaults = Self::default();
        let section = ini_data.get("Graphics").cloned().unwrap_or_default();

        Self {
            width: section
                .get("width")
                .and_then(|value| parse_u32(value))
                .unwrap_or(defaults.width),
            height: section
                .get("height")
                .and_then(|value| parse_u32(value))
                .unwrap_or(defaults.height),
            vsync: section
                .get("vsync")
                .and_then(|value| parse_bool(value))
                .unwrap_or(defaults.vsync),
            force_opengl: section
                .get("force_opengl")
                .and_then(|value| parse_bool(value))
                .unwrap_or(defaults.force_opengl),
            shadow_map_resolution: section
                .get("shadow_map_resolution")
                .and_then(|value| parse_u32(value))
                .unwrap_or(defaults.shadow_map_resolution),
        }
    }
}