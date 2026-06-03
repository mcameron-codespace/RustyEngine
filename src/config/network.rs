use std::collections::HashMap;

use super::parser::{parse_f32, parse_u32};

/// Configuration parameters for the Renet real-time UDP connection protocol.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkConfig {
    pub server_address: String,
    pub connection_port: u16,
    pub rate_hz: f32,
    pub timeout_seconds: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1".to_string(),
            connection_port: 5000,
            rate_hz: 60.0,
            timeout_seconds: 5,
        }
    }
}

impl NetworkConfig {
    pub fn from_ini_data(ini_data: &HashMap<String, HashMap<String, String>>) -> Self {
        let defaults = Self::default();
        let section = ini_data.get("Network").cloned().unwrap_or_default();

        Self {
            server_address: section
                .get("server_address")
                .cloned()
                .unwrap_or(defaults.server_address),
            connection_port: section
                .get("connection_port")
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(defaults.connection_port),
            rate_hz: section
                .get("rate_hz")
                .and_then(|value| parse_f32(value))
                .unwrap_or(defaults.rate_hz),
            timeout_seconds: section
                .get("timeout_seconds")
                .and_then(|value| parse_u32(value))
                .map(|value| value as u32)
                .unwrap_or(defaults.timeout_seconds),
        }
    }
}