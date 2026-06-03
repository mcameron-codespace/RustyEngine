use super::widget::{Rect, Color, WidgetStyle};

/// Primitive graphics commands sent to the rendering pipeline.
#[derive(Debug, Clone)]
pub enum UiDrawCommand {
    DrawRect {
        rect: Rect,
        color: Color,
        border_color: Option<Color>,
        border_thickness: f32,
    },
    DrawText {
        rect: Rect,
        text: String,
        color: Color,
    },
}

/// Immediate-mode UI orchestration context.
/// Manages mouse interaction queries, widget allocations, and draw command lists.
pub struct UiContext {
    // Current input state
    mouse_x: f32,
    mouse_y: f32,
    mouse_clicked: bool,
    
    // Interaction tracking
    hovered_id: Option<u64>,
    active_id: Option<u64>,
    
    // Pre-allocated flat buffers for UI drawing (Zero mid-loop allocation rule)
    pub draw_commands: Vec<UiDrawCommand>,
}

impl UiContext {
    /// Initialize the UI context layout tables.
    pub fn new() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_clicked: false,
            hovered_id: None,
            active_id: None,
            draw_commands: Vec::with_capacity(128),
        }
    }

    /// Update frame context inputs before compiling UI layouts.
    pub fn begin_frame(&mut self, mouse_pos: [f32; 2], is_clicked: bool) {
        self.mouse_x = mouse_pos[0];
        self.mouse_y = mouse_pos[1];
        self.mouse_clicked = is_clicked;
        self.draw_commands.clear();
    }

    /// Render a flat window container/panel overlay.
    pub fn panel(&mut self, _id: u64, rect: Rect, style: WidgetStyle) {
        self.draw_commands.push(UiDrawCommand::DrawRect {
            rect,
            color: style.background_color,
            border_color: Some(style.border_color),
            border_thickness: style.border_thickness,
        });
    }

    /// Draw a static text label.
    pub fn label(&mut self, text: &str, rect: Rect, color: Color) {
        self.draw_commands.push(UiDrawCommand::DrawText {
            rect,
            text: text.to_string(),
            color,
        });
    }

    /// Render an interactive button and return true if clicked on the current frame.
    pub fn button(&mut self, id: u64, text: &str, rect: Rect, mut style: WidgetStyle) -> bool {
        let is_hovered = rect.contains(self.mouse_x, self.mouse_y);
        let mut clicked = false;

        if is_hovered {
            self.hovered_id = Some(id);
            // Highlight button feedback on hover
            style.background_color.r += 0.1;
            style.background_color.g += 0.1;
            style.background_color.b += 0.1;

            if self.mouse_clicked {
                self.active_id = Some(id);
                clicked = true;
            }
        } else if self.hovered_id == Some(id) {
            self.hovered_id = None;
        }

        // Add background rect
        self.draw_commands.push(UiDrawCommand::DrawRect {
            rect,
            color: style.background_color,
            border_color: Some(style.border_color),
            border_thickness: style.border_thickness,
        });

        // Add foreground text centering offset approximation
        let text_rect = Rect::new(rect.x + 10.0, rect.y + (rect.height * 0.25), rect.width, rect.height);
        self.draw_commands.push(UiDrawCommand::DrawText {
            rect: text_rect,
            text: text.to_string(),
            color: style.text_color,
        });

        clicked
    }

    /// End layout generation, flushing state registers.
    pub fn end_frame(&mut self) {
        if !self.mouse_clicked {
            self.active_id = None;
        }
    }
}