use crate::engine::renderer::{Color, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

#[derive(Debug, Clone)]
pub struct UiStyle {
    pub background: Color,
    pub border: Color,
    pub border_width: f32,
    pub text_color: Color,
    pub font_size: f32,
    pub padding: f32,
    pub corner_radius: f32,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            background: Color { r: 0.15, g: 0.12, b: 0.08, a: 0.95 },
            border: Color { r: 0.55, g: 0.42, b: 0.2, a: 1.0 },
            border_width: 3.0,
            text_color: Color { r: 0.96, g: 0.84, b: 0.25, a: 1.0 },
            font_size: 14.0,
            padding: 8.0,
            corner_radius: 4.0,
        }
    }
}

impl UiStyle {
    pub fn stardew_wood() -> Self {
        Self {
            background: Color { r: 0.36, g: 0.22, b: 0.1, a: 0.95 },
            border: Color { r: 0.55, g: 0.42, b: 0.2, a: 1.0 },
            border_width: 4.0,
            text_color: Color { r: 0.96, g: 0.84, b: 0.25, a: 1.0 },
            font_size: 14.0,
            padding: 12.0,
            corner_radius: 6.0,
        }
    }
    
    pub fn stardew_button() -> Self {
        Self {
            background: Color { r: 0.45, g: 0.3, b: 0.15, a: 1.0 },
            border: Color { r: 0.65, g: 0.5, b: 0.25, a: 1.0 },
            border_width: 2.0,
            text_color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            font_size: 12.0,
            padding: 6.0,
            corner_radius: 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UiLayout {
    Fixed { x: f32, y: f32, width: f32, height: f32 },
    Anchor { top: Option<f32>, bottom: Option<f32>, left: Option<f32>, right: Option<f32> },
    Center { width: f32, height: f32 },
}

#[derive(Debug, Clone)]
pub struct Widget {
    pub id: WidgetId,
    pub layout: UiLayout,
    pub style: UiStyle,
    pub visible: bool,
    pub children: Vec<WidgetId>,
}

impl Widget {
    pub fn new(id: u64, layout: UiLayout) -> Self {
        Self {
            id: WidgetId(id), layout, style: UiStyle::default(),
            visible: true, children: Vec::new(),
        }
    }
    
    pub fn with_style(mut self, style: UiStyle) -> Self { self.style = style; self }
    pub fn with_child(mut self, child: WidgetId) -> Self { self.children.push(child); self }
    
    pub fn bounds(&self, screen_width: f32, screen_height: f32) -> Rect {
        match &self.layout {
            UiLayout::Fixed { x, y, width, height } => {
                Rect { x: *x, y: *y, width: *width, height: *height }
            }
            UiLayout::Center { width, height } => {
                Rect {
                    x: (screen_width - width) / 2.0,
                    y: (screen_height - height) / 2.0,
                    width: *width, height: *height,
                }
            }
            UiLayout::Anchor { top, bottom, left, right } => {
                let w = 200.0;
                let h = 100.0;
                let x = left.unwrap_or_else(|| screen_width - right.unwrap_or(0.0) - w);
                let y = top.unwrap_or_else(|| screen_height - bottom.unwrap_or(0.0) - h);
                Rect { x, y, width: w, height: h }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_fixed_bounds() {
        let w = Widget::new(1, UiLayout::Fixed { x: 10.0, y: 20.0, width: 100.0, height: 50.0 });
        let b = w.bounds(800.0, 600.0);
        assert_eq!(b.x, 10.0);
        assert_eq!(b.width, 100.0);
    }

    #[test]
    fn test_widget_center_bounds() {
        let w = Widget::new(1, UiLayout::Center { width: 200.0, height: 100.0 });
        let b = w.bounds(800.0, 600.0);
        assert_eq!(b.x, 300.0);
        assert_eq!(b.y, 250.0);
    }

    #[test]
    fn test_ui_style_defaults() {
        let s = UiStyle::default();
        assert_eq!(s.border_width, 3.0);
        let sw = UiStyle::stardew_wood();
        assert_eq!(sw.border_width, 4.0);
    }
}
