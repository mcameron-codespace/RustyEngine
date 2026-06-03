use std::collections::HashMap;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use super::source::{InputSource, GamepadAxis};

/// Action definitions that are decoupled from raw hardware inputs.
/// This acts as your gameplay logical abstraction layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    // Shared
    MoveForward,
    MoveBackward,
    TurnLeft,
    TurnRight,
    Interact,
    
    // Survival specific
    Jump,
    Inventory,
    UseItem,
    
    // Sim Racing specific
    Accelerate,
    Brake,
    Steer,
    Handbrake,
    ShiftUp,
    ShiftDown,
}

/// Evaluated dynamic evaluation state of a mapped Action.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActionState {
    /// Digital representation (useful for binary inputs like jump, item use, shifts)
    pub is_pressed: bool,
    /// Analog intensity representation clamped to $[-1.0, 1.0]$ (useful for triggers, steering, movement)
    pub value: f32,
}

/// Configurable response shape for a single analog-to-action binding.
/// All presets preserve output range $[-1.0, 1.0]$.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CurvePreset {
    /// Linear mapping after deadzone.
    #[default]
    Linear,
    /// Smooth S-curve for stronger centering feel.
    SCurve,
    /// Exponential curve useful for pedal/trigger inputs.
    Exponential,
}

#[inline]
fn smooth_step(x: f32, deadzone: f32, sharpness: f32) -> f32 {
    let sign = x.signum();
    let abs = x.abs();
    let dz = deadzone.clamp(0.0, 0.4999);
    let t = ((abs - dz) / (1.0 - dz)).clamp(0.0, 1.0);
    let shaped = if sharpness <= 1.0 {
        t
    } else if sharpness <= 2.0 {
        // S-curve: f(t) = 3t² - 2t³, midpoint-preserving
        let t2 = t * t;
        (3.0 * t2) - (2.0 * t2 * t)
    } else {
        // Exponential curve: t^sharpness, more aggressive at high inputs
        t.powf(sharpness)
    };
    sign * shaped
}

pub(crate) struct AnalogBinding {
    pub(crate) source: InputSource,
    pub(crate) scale: f32,
    pub(crate) curve: CurvePreset,
    pub(crate) deadzone: f32,
}

/// A highly-efficient input management engine.
/// Stores physical state flags and translates them to Action States at runtime.
pub struct InputMapper {
    keyboard_states: HashMap<KeyCode, bool>,
    mouse_button_states: HashMap<MouseButton, bool>,
    gamepad_button_states: HashMap<u32, bool>,
    gamepad_axis_states: HashMap<GamepadAxis, f32>,

    digital_bindings: HashMap<Action, Vec<InputSource>>,
    analog_bindings: HashMap<Action, Vec<AnalogBinding>>,
}

impl InputMapper {
    /// Initialize empty mapping pools with pre-allocated tracking layouts.
    pub fn new() -> Self {
        Self {
            keyboard_states: HashMap::with_capacity(64),
            mouse_button_states: HashMap::with_capacity(8),
            gamepad_button_states: HashMap::with_capacity(16),
            gamepad_axis_states: HashMap::with_capacity(8),
            digital_bindings: HashMap::with_capacity(16),
            analog_bindings: HashMap::with_capacity(16),
        }
    }

    /// Map a physical digital input (key/button) to a gameplay action.
    pub fn bind_digital(&mut self, action: Action, source: InputSource) {
        self.digital_bindings
            .entry(action)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(source);
    }

    /// Map a physical input source to an analog axis with a scaling multiplier.
    pub fn bind_analog(&mut self, action: Action, source: InputSource, scale: f32) {
        self.analog_bindings
            .entry(action)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(AnalogBinding {
                source,
                scale,
                curve: CurvePreset::Linear,
                deadzone: 0.0,
            });
    }

    /// Map a physical input source to an analog axis with a scaling multiplier and shaping preset.
    pub fn bind_analog_with_curve(
        &mut self,
        action: Action,
        source: InputSource,
        scale: f32,
        curve: CurvePreset,
        deadzone: f32,
    ) {
        self.analog_bindings
            .entry(action)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(AnalogBinding {
                source,
                scale,
                curve,
                deadzone: deadzone.clamp(0.0, 0.4999),
            });
    }

    /// Clear all mapping tables to prepare for a custom layout rewrite.
    pub fn clear_bindings(&mut self) {
        self.digital_bindings.clear();
        self.analog_bindings.clear();
    }

    // =========================================================================
    // PHYSICAL HARDWARE STATUS SETTERS
    // =========================================================================

    pub fn update_keyboard_state(&mut self, key: KeyCode, is_pressed: bool) {
        self.keyboard_states.insert(key, is_pressed);
    }

    pub fn update_mouse_button(&mut self, button: MouseButton, is_pressed: bool) {
        self.mouse_button_states.insert(button, is_pressed);
    }

    pub fn update_gamepad_button(&mut self, button_id: u32, is_pressed: bool) {
        self.gamepad_button_states.insert(button_id, is_pressed);
    }

    pub fn update_gamepad_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.gamepad_axis_states.insert(axis, value.clamp(-1.0, 1.0));
    }

    // =========================================================================
    // LOGICAL STATE QUERIES
    // =========================================================================

    fn get_source_value(&self, source: InputSource) -> f32 {
        match source {
            InputSource::Keyboard(key) => {
                if *self.keyboard_states.get(&key).unwrap_or(&false) {
                    1.0
                } else {
                    0.0
                }
            }
            InputSource::MouseButton(button) => {
                if *self.mouse_button_states.get(&button).unwrap_or(&false) {
                    1.0
                } else {
                    0.0
                }
            }
            InputSource::GamepadButton(btn) => {
                if *self.gamepad_button_states.get(&btn).unwrap_or(&false) {
                    1.0
                } else {
                    0.0
                }
            }
            InputSource::GamepadAxis(axis) => {
                *self.gamepad_axis_states.get(&axis).unwrap_or(&0.0)
            }
        }
    }

    /// Resolve and aggregate the active physical keys and axes mapped to a gameplay action.
    pub fn get_action_state(&self, action: Action) -> ActionState {
        let mut state = ActionState::default();

        // 1. Evaluate Digital Bindings (Any match activates the action)
        if let Some(bindings) = self.digital_bindings.get(&action) {
            for &source in bindings {
                if self.get_source_value(source) > 0.5 {
                    state.is_pressed = true;
                    state.value = 1.0;
                    break;
                }
            }
        }

        // 2. Evaluate Analog Bindings (Aggregate scaled inputs, clamped to safe boundaries)
        if let Some(bindings) = self.analog_bindings.get(&action) {
            let mut combined_value = 0.0f32;
            let mut active = false;

            for binding in bindings {
                let mut physical_val = self.get_source_value(binding.source);
                if physical_val.abs() > 0.001 {
                    // Response shaping (applied only in the active region).
                    let sharpness = match binding.curve {
                        CurvePreset::Linear => 1.0,
                        CurvePreset::SCurve => 2.0,
                        CurvePreset::Exponential => 3.0,
                    };
                    physical_val = smooth_step(physical_val, binding.deadzone, sharpness);
                    combined_value += physical_val * binding.scale;
                    active = true;
                }
            }

            if active {
                state.value = combined_value.clamp(-1.0, 1.0);
                state.is_pressed = state.value.abs() > 0.5;
            }
        }

        state
    }
}