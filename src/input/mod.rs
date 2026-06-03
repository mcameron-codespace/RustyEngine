pub mod source;
pub mod mapper;

pub use source::{InputSource, GamepadAxis};
pub use mapper::{Action, ActionState, CurvePreset, InputMapper};

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;

    #[test]
    fn test_core_action_input_mapping() {
        let mut mapper = InputMapper::new();

        // Map WASD and Gamepad Left Stick to Steer & Accelerate actions
        mapper.bind_digital(Action::Accelerate, InputSource::Keyboard(KeyCode::KeyW));
        mapper.bind_analog(Action::Steer, InputSource::GamepadAxis(GamepadAxis::LeftStickX), 1.0);

        // Simulate pressing 'W'
        mapper.update_keyboard_state(KeyCode::KeyW, true);
        
        let accelerate_state = mapper.get_action_state(Action::Accelerate);
        assert!(accelerate_state.is_pressed);
        assert_eq!(accelerate_state.value, 1.0);

        // Simulate moving gamepad stick half-way to the right
        mapper.update_gamepad_axis(GamepadAxis::LeftStickX, 0.5);

        let steer_state = mapper.get_action_state(Action::Steer);
        assert_eq!(steer_state.value, 0.5);
    }
}