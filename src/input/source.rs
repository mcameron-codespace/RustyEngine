use winit::event::MouseButton;
use winit::keyboard::KeyCode;

/// Supported hardware button/axis categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Keyboard(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(u32),
    GamepadAxis(GamepadAxis),
}

/// Standardized analog stick and pedal identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}