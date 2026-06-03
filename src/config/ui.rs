use std::collections::HashMap;

use super::parser::{parse_bool, parse_f32};

/// High-level RGBA float representation to maintain config module autonomy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Theme, scaling, and behavior configurations for the immediate-mode UI system.
#[derive(Debug, Clone, PartialEq)]
pub struct UiConfig {
    pub scale_factor: f32,
    pub debug_bounds: bool,
    pub primary_color: ConfigColor,
    pub secondary_color: ConfigColor,
    pub background_color: ConfigColor,
    pub text_color: ConfigColor,
    pub base_font_size: f32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            debug_bounds: false,
            primary_color: ConfigColor { r: 0.12, g: 0.53, b: 0.9, a: 1.0 }, // Blue
            secondary_color: ConfigColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White
            background_color: ConfigColor { r: 0.15, g: 0.17, b: 0.22, a: 1.0 }, // Slate
            text_color: ConfigColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White
            base_font_size: 14.0,
        }
    }
}

impl UiConfig {
    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        let defaults = Self::default();
        let section = ini_data.get("UI").cloned().unwrap_or_default();

        fn color(section: &HashMap<String, String>, prefix: &str, fallback: ConfigColor) -> ConfigColor {
            ConfigColor {
                r: section
                    .get(&format!("{prefix}_r"))
                    .and_then(|value| parse_f32(value))
                    .unwrap_or(fallback.r),
                g: section
                    .get(&format!("{prefix}_g"))
                    .and_then(|value| parse_f32(value))
                    .unwrap_or(fallback.g),
                b: section
                    .get(&format!("{prefix}_b"))
                    .and_then(|value| parse_f32(value))
                    .unwrap_or(fallback.b),
                a: section
                    .get(&format!("{prefix}_a"))
                    .and_then(|value| parse_f32(value))
                    .unwrap_or(fallback.a),
            }
        }

        Self {
            scale_factor: section
                .get("scale_factor")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.scale_factor),
            debug_bounds: section
                .get("debug_bounds")
                .and_then(|value| parse_bool(value))
                .unwrap_or(defaults.debug_bounds),
            primary_color: color(&section, "primary_color", defaults.primary_color),
            secondary_color: color(&section, "secondary_color", defaults.secondary_color),
            background_color: color(&section, "background_color", defaults.background_color),
            text_color: color(&section, "text_color", defaults.text_color),
            base_font_size: section
                .get("base_font_size")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.base_font_size),
        }
    }
}