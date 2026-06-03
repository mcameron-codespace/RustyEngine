 use std::collections::HashMap;
 use std::path::PathBuf;
 use winit::keyboard::KeyCode;
 use crate::input::{Action, CurvePreset, InputSource, GamepadAxis, InputMapper};
 use super::{parse_ini_file, get_executable_config_dir};

/// Structured analog entry consumed by the config layer.
/// Carries physical source, scaling, deadzone, and response-curve shaping.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InputBinding {
    pub source: InputSource,
    pub scale: f32,
    pub curve: CurvePreset,
    pub deadzone: f32,
}

impl InputBinding {
    pub fn new(source: InputSource, scale: f32) -> Self {
        Self {
            source,
            scale,
            curve: CurvePreset::Linear,
            deadzone: 0.0,
        }
    }

    pub fn with_curve(mut self, curve: CurvePreset) -> Self {
        self.curve = curve;
        self
    }

    pub fn with_deadzone(mut self, deadzone: f32) -> Self {
        self.deadzone = deadzone.clamp(0.0, 0.4999);
        self
    }
}

/// Configuration representing user-defined input mappings loaded from an INI file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputConfig {
    /// Maps dynamic actions to their configured keyboard/mouse sources
    pub digital_mappings: HashMap<Action, Vec<InputSource>>,
    /// Maps dynamic actions to their configured analog sources and curve bindings
    pub analog_mappings: HashMap<Action, Vec<InputBinding>>,
}

impl InputConfig {
    /// Attempt to load and parse the default `input.ini` file located in the executable config directory.
    /// Falls back to standard default mappings if the file does not exist.
    pub fn load_from_executable_dir() -> Self {
        if let Ok(mut config_path) = get_executable_config_dir() {
            config_path.push("input.ini");
            if config_path.exists() {
                if let Ok(parsed_data) = parse_ini_file(&config_path) {
                    return Self::from_ini_data(parsed_data);
                }
            }
        }
        Self::default_mappings()
    }

    /// Default mapping preset for Keyboard/Mouse and Gamepad controls.
    pub fn default_mappings() -> Self {
        let mut digital = HashMap::new();
        let mut analog = HashMap::new();

        // Default Digital Mappings
        digital.insert(Action::Jump, vec![InputSource::Keyboard(KeyCode::Space)]);
        digital.insert(Action::Interact, vec![InputSource::Keyboard(KeyCode::KeyE)]);
        digital.insert(Action::ShiftUp, vec![InputSource::Keyboard(KeyCode::KeyE)]);
        digital.insert(
            Action::ShiftDown,
            vec![InputSource::Keyboard(KeyCode::KeyQ)],
        );

        // Default Analog Mappings
        analog.insert(
            Action::MoveForward,
            vec![InputBinding::new(InputSource::Keyboard(KeyCode::KeyW), 1.0)],
        );
        analog.insert(
            Action::MoveBackward,
            vec![InputBinding::new(InputSource::Keyboard(KeyCode::KeyS), 1.0)],
        );
        analog.insert(
            Action::Steer,
            vec![
                InputBinding::new(InputSource::Keyboard(KeyCode::KeyA), -1.0),
                InputBinding::new(InputSource::Keyboard(KeyCode::KeyD), 1.0),
                InputBinding::new(
                    InputSource::GamepadAxis(GamepadAxis::LeftStickX),
                    1.0,
                ),
            ],
        );
        analog.insert(
            Action::Accelerate,
            vec![InputBinding::new(
                InputSource::GamepadAxis(GamepadAxis::RightTrigger),
                1.0,
            )],
        );
        analog.insert(
            Action::Brake,
            vec![InputBinding::new(
                InputSource::GamepadAxis(GamepadAxis::LeftTrigger),
                1.0,
            )],
        );

        Self {
            digital_mappings: digital,
            analog_mappings: analog,
        }
    }

    /// Parse raw key-value string pairs from INI maps into structured Action maps.
    pub fn from_ini_data(ini_data: HashMap<String, HashMap<String, String>>) -> Self {
        let mut config = Self {
            digital_mappings: HashMap::new(),
            analog_mappings: HashMap::new(),
        };

        // Parse Digital Keyboard / Mouse Mappings
        if let Some(keyboard_section) = ini_data.get("KeyboardMappings") {
            for (action_str, key_str) in keyboard_section {
                if let Some(action) = parse_action(action_str) {
                    if let Some(key) = parse_key(key_str) {
                        config
                            .digital_mappings
                            .entry(action)
                            .or_default()
                            .push(InputSource::Keyboard(key));
                    }
                }
            }
        }

        // Parse Analog Gamepad Axis Mappings
        if let Some(gamepad_section) = ini_data.get("GamepadMappings") {
            for (action_str, axis_str) in gamepad_section {
                if let Some(action) = parse_action(action_str) {
                    if let Some(axis) = parse_axis(axis_str) {
                        config
                            .analog_mappings
                            .entry(action)
                            .or_default()
                            .push(InputBinding::new(
                                InputSource::GamepadAxis(axis),
                                1.0,
                            ));
                    }
                }
            }
        }

        config
    }

    /// Apply these parsed configuration mappings to an active game InputMapper.
    pub fn apply_to_mapper(&self, mapper: &mut InputMapper) {
        mapper.clear_bindings();

        for (&action, sources) in &self.digital_mappings {
            for &source in sources {
                mapper.bind_digital(action, source);
            }
        }

        for (&action, bindings) in &self.analog_mappings {
            for binding in bindings {
                mapper.bind_analog_with_curve(
                    action,
                    binding.source,
                    binding.scale,
                    binding.curve,
                    binding.deadzone,
                );
            }
        }
    }
}

// =========================================================================
// PARSING HELPER UTILITIES
// =========================================================================

fn parse_action(name: &str) -> Option<Action> {
    match name.to_lowercase().as_str() {
        "moveforward" => Some(Action::MoveForward),
        "movebackward" => Some(Action::MoveBackward),
        "turnleft" => Some(Action::TurnLeft),
        "turnright" => Some(Action::TurnRight),
        "interact" => Some(Action::Interact),
        "jump" => Some(Action::Jump),
        "inventory" => Some(Action::Inventory),
        "useitem" => Some(Action::UseItem),
        "accelerate" => Some(Action::Accelerate),
        "brake" => Some(Action::Brake),
        "steer" => Some(Action::Steer),
        "handbrake" => Some(Action::Handbrake),
        "shiftup" => Some(Action::ShiftUp),
        "shiftdown" => Some(Action::ShiftDown),
        _ => None,
    }
}

fn parse_axis(name: &str) -> Option<GamepadAxis> {
    match name.to_lowercase().as_str() {
        "leftstickx" => Some(GamepadAxis::LeftStickX),
        "leftsticky" => Some(GamepadAxis::LeftStickY),
        "rightstickx" => Some(GamepadAxis::RightStickX),
        "rightsticky" => Some(GamepadAxis::RightStickY),
        "lefttrigger" => Some(GamepadAxis::LeftTrigger),
        "righttrigger" => Some(GamepadAxis::RightTrigger),
        _ => None,
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    match name.to_uppercase().as_str() {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "SPACE" => Some(KeyCode::Space),
        "LSHIFT" => Some(KeyCode::ShiftLeft),
        "RSHIFT" => Some(KeyCode::ShiftRight),
        "LCONTROL" => Some(KeyCode::ControlLeft),
        "RCONTROL" => Some(KeyCode::ControlRight),
        "ESCAPE" => Some(KeyCode::Escape),
        "ENTER" => Some(KeyCode::Enter),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mappings_generation() {
        let config = InputConfig::default_mappings();

        // Assert dynamic keyboard maps exist
        let jump_bindings = config.digital_mappings.get(&Action::Jump).unwrap();
        assert_eq!(jump_bindings[0], InputSource::Keyboard(KeyCode::Space));

        // Assert steer analogs have scaled values
        let steer_bindings = config.analog_mappings.get(&Action::Steer).unwrap();
        assert_eq!(steer_bindings[0].source, InputSource::Keyboard(KeyCode::KeyA));
        assert_eq!(steer_bindings[0].scale, -1.0);
    }

    #[test]
    fn test_custom_ini_translation() {
        let mut mock_ini = HashMap::new();

        let mut keyboard_section = HashMap::new();
        keyboard_section.insert("MoveForward".to_string(), "W".to_string());
        keyboard_section.insert("Jump".to_string(), "Space".to_string());
        mock_ini.insert("KeyboardMappings".to_string(), keyboard_section);

        let mut gamepad_section = HashMap::new();
        gamepad_section.insert("Steer".to_string(), "LeftStickX".to_string());
        mock_ini.insert("GamepadMappings".to_string(), gamepad_section);

        let config = InputConfig::from_ini_data(mock_ini);

        let forward_analog_bindings = config.digital_mappings.get(&Action::MoveForward);
        assert!(forward_analog_bindings.is_none()); // MoveForward parsed under digital? No, handled by custom logic or defaults

        let jump_bindings = config.digital_mappings.get(&Action::Jump).unwrap();
        assert_eq!(jump_bindings[0], InputSource::Keyboard(KeyCode::Space));

        let steer_bindings = config.analog_mappings.get(&Action::Steer).unwrap();
        assert_eq!(
            steer_bindings[0].source,
            InputSource::GamepadAxis(GamepadAxis::LeftStickX)
        );
        assert_eq!(steer_bindings[0].scale, 1.0);
    }
}
