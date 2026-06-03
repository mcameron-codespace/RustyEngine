pub mod widget;
pub mod context;

pub use widget::{Rect, Color, WidgetStyle, WidgetType};
pub use context::{UiContext, UiDrawCommand};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_boundary_intersection() {
        let rect = Rect::new(50.0, 50.0, 150.0, 50.0);
        
        // Inside boundary coordinates
        assert!(rect.contains(100.0, 75.0));
        // Outside boundary coordinates
        assert!(!rect.contains(30.0, 75.0));
    }

    #[test]
    fn test_immediate_mode_button_states() {
        let mut ui = UiContext::new();
        let button_id = 42u64;
        let button_rect = Rect::new(10.0, 10.0, 100.0, 30.0);
        let style = WidgetStyle::default();

        // 1. Simulate mouse hovering but not clicking
        ui.begin_frame([15.0, 15.0], false);
        let clicked = ui.button(button_id, "Test Button", button_rect, style);
        
        assert!(!clicked);
        assert_eq!(ui.draw_commands.len(), 2); // 1 background + 1 text overlay

        // 2. Simulate mouse hovering and clicking
        ui.begin_frame([15.0, 15.0], true);
        let clicked_active = ui.button(button_id, "Test Button", button_rect, style);
        
        assert!(clicked_active);
    }
}