use crate::engine::renderer::{Color, Vec2, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum WidgetState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

pub trait Widget {
    fn update(&mut self, dt: f32, mouse_pos: Vec2, mouse_clicked: bool);
    fn render(&self, canvas: &mut dyn std::any::Any);
    fn bounds(&self) -> Rect;
    fn state(&self) -> &WidgetState;
}

pub struct Button {
    pub rect: Rect,
    pub text: String,
    pub color: Color,
    pub state: WidgetState,
    pub on_click: Option<Box<dyn FnOnce()>>,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: &str) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            text: text.to_string(),
            color: Color { r: 0.3, g: 0.3, b: 0.4, a: 1.0 },
            state: WidgetState::Normal,
            on_click: None,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = Color { r, g, b, a: 1.0 };
        self
    }
}

impl Widget for Button {
    fn update(&mut self, _dt: f32, mouse_pos: Vec2, mouse_clicked: bool) {
        if self.rect.contains(&mouse_pos) {
            if mouse_clicked {
                self.state = WidgetState::Pressed;
            } else {
                self.state = WidgetState::Hovered;
            }
        } else {
            self.state = WidgetState::Normal;
        }
    }

    fn render(&self, _canvas: &mut dyn std::any::Any) {}

    fn bounds(&self) -> Rect { self.rect }
    fn state(&self) -> &WidgetState { &self.state }
}

pub struct Label {
    pub rect: Rect,
    pub text: String,
    pub color: Color,
    pub font_size: f32,
}

impl Label {
    pub fn new(x: f32, y: f32, text: &str) -> Self {
        Self {
            rect: Rect::new(x, y, 200.0, 30.0),
            text: text.to_string(),
            color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            font_size: 16.0,
        }
    }
}

pub struct ProgressBar {
    pub rect: Rect,
    pub value: f32,
    pub max: f32,
    pub color: Color,
    pub bg_color: Color,
}

impl ProgressBar {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
            value: 0.0, max: 1.0,
            color: Color { r: 0.2, g: 0.8, b: 0.2, a: 1.0 },
            bg_color: Color { r: 0.1, g: 0.1, b: 0.1, a: 0.8 },
        }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, self.max);
    }

    pub fn percentage(&self) -> f32 {
        self.value / self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 100.0, 50.0);
        assert!(rect.contains(&Vec2 { x: 50.0, y: 25.0 }));
        assert!(!rect.contains(&Vec2 { x: 150.0, y: 25.0 }));
    }

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new(0.0, 0.0, 100.0, 10.0);
        bar.set_value(0.5);
        assert!((bar.percentage() - 0.5).abs() < 0.01);
    }
}
