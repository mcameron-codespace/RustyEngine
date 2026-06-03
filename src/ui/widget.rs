/// Basic rectangular bounding box for UI element layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Check if a physical screen point resides within these boundary dimensions.
    pub fn contains(&self, point_x: f32, point_y: f32) -> bool {
        point_x >= self.x && point_x <= self.x + self.width &&
        point_y >= self.y && point_y <= self.y + self.height
    }
}

/// Dynamic color specifications using standard normalized RGBA channels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const SLATE: Self = Self { r: 0.15, g: 0.17, b: 0.22, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.12, g: 0.53, b: 0.9, a: 1.0 };
}

/// Visual presentation tokens applied to UI widgets.
#[derive(Debug, Clone, Copy)]
pub struct WidgetStyle {
    pub background_color: Color,
    pub border_color: Color,
    pub border_thickness: f32,
    pub text_color: Color,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        Self {
            background_color: Color::SLATE,
            border_color: Color::WHITE,
            border_thickness: 1.0,
            text_color: Color::WHITE,
        }
    }
}

/// Structural UI components supported natively by the engine.
#[derive(Debug, Clone)]
pub enum WidgetType {
    Panel,
    Label { text: String },
    Button { text: String, is_hovered: bool, is_pressed: bool },
    Slider { value: f32, min: f32, max: f32 },
}