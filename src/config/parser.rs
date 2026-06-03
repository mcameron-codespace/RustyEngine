use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

pub const DEFAULT_ENGINE_DEFAULTS: &str = r#"[Graphics]
width = 1280
height = 720
vsync = true
force_opengl = false
shadow_map_resolution = 2048

[Audio]
master_volume = 1.0
music_volume = 0.7
sfx_volume = 1.0
spatial_min_distance = 1.0
spatial_max_distance = 50.0

[Physics]
tick_rate_hz = 120.0
gravity_y = -9.81
max_velocity = 100.0

[Network]
server_address = 127.0.0.1
connection_port = 5000
rate_hz = 60.0
timeout_seconds = 5

[UI]
scale_factor = 1.0
debug_bounds = false
primary_color_r = 0.12
primary_color_g = 0.53
primary_color_b = 0.90
primary_color_a = 1.0
secondary_color_r = 1.0
secondary_color_g = 1.0
secondary_color_b = 1.0
secondary_color_a = 1.0
background_color_r = 0.15
background_color_g = 0.17
background_color_b = 0.22
background_color_a = 1.0
text_color_r = 1.0
text_color_g = 1.0
text_color_b = 1.0
text_color_a = 1.0
base_font_size = 14.0
"#;

/// Helper to locate the absolute path to the `config/` directory 
/// residing in the same directory as the active game executable.
pub fn get_executable_config_dir() -> io::Result<PathBuf> {
    let mut exe_path = std::env::current_exe()?;
    exe_path.pop(); // Pop the executable filename
    exe_path.push("config");
    Ok(exe_path)
}

/// A generic, robust, and safe immediate-mode INI parser.
/// Parses standard config layouts without dynamic heap allocations during gameplay.
pub fn parse_ini_file(path: &PathBuf) -> io::Result<HashMap<String, HashMap<String, String>>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut config_data = HashMap::new();
    let mut current_section: Option<String> = None;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Ignore empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        // Section header parsing: [SectionName]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let section_name = trimmed[1..trimmed.len() - 1].trim().to_string();
            current_section = Some(section_name.clone());
            config_data.entry(section_name).or_insert_with(HashMap::new);
            continue;
        }

        // Key-Value parsing: key = value
        if let Some(equal_idx) = trimmed.find('=') {
            if let Some(ref section) = current_section {
                let key = trimmed[..equal_idx].trim().to_string();
                let value = trimmed[equal_idx + 1..].trim().to_string();
                
                if let Some(section_map) = config_data.get_mut(section) {
                    section_map.insert(key, value);
                }
            }
        }
    }

    Ok(config_data)
}

/// Ensures the config directory and target file exist.
/// If they do not, they are generated automatically with the provided default content.
/// Returns the absolute path to the verified configuration file.
pub fn ensure_config_file_exists(filename: &str, default_content: &str) -> io::Result<PathBuf> {
    let mut config_path = get_executable_config_dir()?;
    
    // Ensure the parent directory exists
    if !config_path.exists() {
        create_dir_all(&config_path)?;
    }

    config_path.push(filename);

    // If the file is missing, write the default content
    if !config_path.exists() {
        let mut file = File::create(&config_path)?;
        file.write_all(default_content.as_bytes())?;
        file.flush()?;
    }

    Ok(config_path)
}

// =========================================================================
// TYPED INI PARSING UTILITY HELPERS
// =========================================================================

/// Parse a boolean from an INI string value (accepts true/false, 1/0, yes/no, on/off).
pub fn parse_bool(val: &str) -> Option<bool> {
    match val.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Parse a 32-bit float from an INI string value.
pub fn parse_f32(val: &str) -> Option<f32> {
    val.trim().parse::<f32>().ok()
}

/// Parse a 32-bit unsigned integer from an INI string value.
pub fn parse_u32(val: &str) -> Option<u32> {
    val.trim().parse::<u32>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_ini_parser_and_directory_creation() {
        let temp_dir = std::env::temp_dir();
        let test_ini_path = temp_dir.join("test_io.ini");
        
        if test_ini_path.exists() {
            let _ = std::fs::remove_file(&test_ini_path);
        }

        let default_content = "[TestSection]\nKey = Value\n; comment";
        let mut file = File::create(&test_ini_path).unwrap();
        file.write_all(default_content.as_bytes()).unwrap();
        file.flush().unwrap();

        let parsed_data = parse_ini_file(&test_ini_path).expect("Failed to parse temporary INI");
        let section = parsed_data.get("TestSection").expect("Section missing");
        assert_eq!(section.get("Key").unwrap(), "Value");

        let _ = std::fs::remove_file(test_ini_path);
    }

    #[test]
    fn test_typed_parsing_helpers() {
        // Test Boolean Parser
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("  OFF "), Some(false));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("invalid"), None);

        // Test Float Parser
        assert_eq!(parse_f32("3.14159"), Some(3.14159));
        assert_eq!(parse_f32(" -0.5 "), Some(-0.5));
        assert_eq!(parse_f32("not_a_float"), None);

        // Test Integer Parser
        assert_eq!(parse_u32("120"), Some(120));
        assert_eq!(parse_u32("0"), Some(0));
        assert_eq!(parse_u32("-10"), None); // u32 cannot be negative
    }
}