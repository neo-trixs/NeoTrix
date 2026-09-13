use crate::engine::renderer::{Color, Vec2, Rect, DrawCommand};

// ---------------------------------------------------------------------------
// UI Color Theme
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub panel_bg: Color,
    pub panel_border: Color,
    pub button_normal: Color,
    pub button_hover: Color,
    pub button_pressed: Color,
    pub button_text: Color,
    pub bar_bg: Color,
    pub bar_hp: Color,
    pub bar_mp: Color,
    pub bar_exp: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub inventory_bg: Color,
    pub inventory_slot: Color,
    pub inventory_slot_hover: Color,
    pub inventory_border: Color,
    pub minimap_bg: Color,
    pub minimap_border: Color,
}

impl UiTheme {
    pub fn default_dark() -> Self {
        Self {
            panel_bg: Color::rgba(0.12, 0.12, 0.15, 0.92),
            panel_border: Color::rgba(0.4, 0.4, 0.5, 0.8),
            button_normal: Color::rgba(0.25, 0.25, 0.35, 1.0),
            button_hover: Color::rgba(0.35, 0.35, 0.50, 1.0),
            button_pressed: Color::rgba(0.15, 0.15, 0.25, 1.0),
            button_text: Color::rgba(0.9, 0.9, 1.0, 1.0),
            bar_bg: Color::rgba(0.15, 0.15, 0.15, 1.0),
            bar_hp: Color::rgba(0.85, 0.15, 0.15, 1.0),
            bar_mp: Color::rgba(0.2, 0.4, 0.9, 1.0),
            bar_exp: Color::rgba(0.2, 0.8, 0.2, 1.0),
            text_primary: Color::rgba(0.95, 0.95, 1.0, 1.0),
            text_secondary: Color::rgba(0.6, 0.6, 0.7, 1.0),
            inventory_bg: Color::rgba(0.08, 0.08, 0.10, 0.90),
            inventory_slot: Color::rgba(0.20, 0.20, 0.25, 1.0),
            inventory_slot_hover: Color::rgba(0.30, 0.30, 0.40, 1.0),
            inventory_border: Color::rgba(0.35, 0.35, 0.45, 0.8),
            minimap_bg: Color::rgba(0.0, 0.0, 0.0, 0.7),
            minimap_border: Color::rgba(0.5, 0.5, 0.5, 0.8),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self { Self::default_dark() }
}

// ---------------------------------------------------------------------------
// Panel — background + border + optional title
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Panel {
    pub rect: Rect,
    pub title: Option<String>,
    pub title_size: f32,
    pub bg_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub visible: bool,
}

impl Panel {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            rect: Rect::new(x, y, w, h),
            title: None, title_size: 14.0,
            bg_color: Color::rgba(0.12, 0.12, 0.15, 0.92),
            border_color: Color::rgba(0.4, 0.4, 0.5, 0.8),
            border_width: 1.0, visible: true,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self { self.title = Some(title.to_string()); self }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        // Background
        cmds.push(DrawCommand::DrawRect { rect: self.rect, color: self.bg_color });
        // Border
        if self.border_width > 0.0 {
            let bw = self.border_width;
            let r = &self.rect;
            // Top
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, bw), color: self.border_color });
            // Bottom
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - bw, r.width, bw), color: self.border_color });
            // Left
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, bw, r.height), color: self.border_color });
            // Right
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - bw, r.y, bw, r.height), color: self.border_color });
        }
        // Title
        if let Some(ref title) = self.title {
            cmds.push(DrawCommand::DrawText {
                text: title.clone(),
                position: Vec2::new(self.rect.x + 8.0, self.rect.y + 6.0),
                color: theme.text_primary,
                size: self.title_size,
            });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Button — normal / hover / pressed states
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState { Normal, Hover, Pressed }

#[derive(Debug, Clone)]
pub struct Button {
    pub rect: Rect,
    pub label: String,
    pub state: ButtonState,
    pub label_size: f32,
    pub visible: bool,
    pub enabled: bool,
}

impl Button {
    pub fn new(x: f32, y: f32, w: f32, h: f32, label: &str) -> Self {
        Self {
            rect: Rect::new(x, y, w, h),
            label: label.to_string(),
            state: ButtonState::Normal,
            label_size: 14.0, visible: true, enabled: true,
        }
    }

    pub fn with_size(mut self, w: f32, h: f32) -> Self { self.rect.width = w; self.rect.height = h; self }

    /// Check if screen position is inside the button.
    pub fn hit_test(&self, pos: Vec2) -> bool {
        self.visible && self.enabled && self.rect.contains(pos)
    }

    pub fn set_state(&mut self, state: ButtonState) { self.state = state; }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        let bg = if !self.enabled {
            Color::rgba(0.15, 0.15, 0.15, 0.8)
        } else {
            match self.state {
                ButtonState::Normal => theme.button_normal,
                ButtonState::Hover => theme.button_hover,
                ButtonState::Pressed => theme.button_pressed,
            }
        };
        cmds.push(DrawCommand::DrawRect { rect: self.rect, color: bg });
        // Border
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y, self.rect.width, 1.0), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y + self.rect.height - 1.0, self.rect.width, 1.0), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y, 1.0, self.rect.height), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(self.rect.x + self.rect.width - 1.0, self.rect.y, 1.0, self.rect.height), color: theme.panel_border });
        // Label
        let text_color = if self.enabled { theme.button_text } else { theme.text_secondary };
        cmds.push(DrawCommand::DrawText {
            text: self.label.clone(),
            position: Vec2::new(self.rect.x + 8.0, self.rect.y + (self.rect.height - self.label_size) / 2.0),
            color: text_color,
            size: self.label_size,
        });
        cmds
    }
}

// ---------------------------------------------------------------------------
// TextLabel — simple text display
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TextLabel {
    pub position: Vec2,
    pub text: String,
    pub color: Color,
    pub size: f32,
    pub visible: bool,
}

impl TextLabel {
    pub fn new(x: f32, y: f32, text: &str) -> Self {
        Self {
            position: Vec2::new(x, y), text: text.to_string(),
            color: Color::rgba(0.95, 0.95, 1.0, 1.0), size: 14.0, visible: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self { self.color = color; self }
    pub fn with_size(mut self, size: f32) -> Self { self.size = size; self }

    pub fn render(&self) -> Option<DrawCommand> {
        if !self.visible { return None; }
        Some(DrawCommand::DrawText {
            text: self.text.clone(), position: self.position,
            color: self.color, size: self.size,
        })
    }
}

// ---------------------------------------------------------------------------
// Bar — HP / MP / EXP bar rendering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarKind { Hp, Mp, Exp }

#[derive(Debug, Clone)]
pub struct Bar {
    pub rect: Rect,
    pub kind: BarKind,
    pub current: f32,
    pub max: f32,
    pub show_text: bool,
    pub text_size: f32,
    pub visible: bool,
}

impl Bar {
    pub fn new(x: f32, y: f32, w: f32, h: f32, kind: BarKind, current: f32, max: f32) -> Self {
        Self { rect: Rect::new(x, y, w, h), kind, current, max, show_text: true, text_size: 10.0, visible: true }
    }

    pub fn set_values(&mut self, current: f32, max: f32) { self.current = current; self.max = max; }

    pub fn ratio(&self) -> f32 {
        if self.max > 0.0 { (self.current / self.max).clamp(0.0, 1.0) } else { 0.0 }
    }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        // Background
        cmds.push(DrawCommand::DrawRect { rect: self.rect, color: theme.bar_bg });
        // Fill
        let fill_color = match self.kind {
            BarKind::Hp => theme.bar_hp,
            BarKind::Mp => theme.bar_mp,
            BarKind::Exp => theme.bar_exp,
        };
        let fill_w = self.rect.width * self.ratio();
        if fill_w > 0.0 {
            cmds.push(DrawCommand::DrawRect {
                rect: Rect::new(self.rect.x, self.rect.y, fill_w, self.rect.height),
                color: fill_color,
            });
        }
        // Border
        let r = &self.rect;
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, 1.0), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - 1.0, r.width, 1.0), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, 1.0, r.height), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - 1.0, r.y, 1.0, r.height), color: theme.panel_border });
        // Text
        if self.show_text {
            let label = format!("{:.0}/{:.0}", self.current, self.max);
            cmds.push(DrawCommand::DrawText {
                text: label,
                position: Vec2::new(self.rect.x + 4.0, self.rect.y + (self.rect.height - self.text_size) / 2.0),
                color: theme.text_primary,
                size: self.text_size,
            });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// InventoryGrid — NxN slot grid
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct InventorySlot {
    pub item_id: Option<u32>,
    pub count: u32,
    pub highlighted: bool,
}

#[derive(Debug, Clone)]
pub struct InventoryGrid {
    pub rect: Rect,
    pub cols: usize,
    pub rows: usize,
    pub slot_size: f32,
    pub gap: f32,
    pub slots: Vec<InventorySlot>,
    pub title: String,
    pub visible: bool,
}

impl InventoryGrid {
    pub fn new(x: f32, y: f32, cols: usize, rows: usize, slot_size: f32) -> Self {
        let total_w = cols as f32 * slot_size + (cols - 1) as f32 * 2.0;
        let total_h = rows as f32 * slot_size + (rows - 1) as f32 * 2.0;
        let slots = vec![InventorySlot { item_id: None, count: 0, highlighted: false }; cols * rows];
        Self {
            rect: Rect::new(x, y, total_w, total_h),
            cols, rows, slot_size, gap: 2.0,
            slots, title: String::new(), visible: true,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self { self.title = title.to_string(); self }

    pub fn set_item(&mut self, col: usize, row: usize, item_id: u32, count: u32) {
        if let Some(slot) = self.slots.get_mut(row * self.cols + col) {
            slot.item_id = Some(item_id);
            slot.count = count;
        }
    }

    pub fn clear_slot(&mut self, col: usize, row: usize) {
        if let Some(slot) = self.slots.get_mut(row * self.cols + col) {
            slot.item_id = None;
            slot.count = 0;
            slot.highlighted = false;
        }
    }

    pub fn slot_at(&self, pos: Vec2) -> Option<(usize, usize)> {
        if !self.rect.contains(pos) { return None; }
        let local_x = pos.x - self.rect.x;
        let local_y = pos.y - self.rect.y;
        let col = (local_x / (self.slot_size + self.gap)) as usize;
        let row = (local_y / (self.slot_size + self.gap)) as usize;
        if col < self.cols && row < self.rows { Some((col, row)) } else { None }
    }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        // Panel background
        cmds.push(DrawCommand::DrawRect { rect: self.rect, color: theme.inventory_bg });
        // Border
        let r = &self.rect;
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, 1.0), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - 1.0, r.width, 1.0), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, 1.0, r.height), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - 1.0, r.y, 1.0, r.height), color: theme.inventory_border });
        // Title
        if !self.title.is_empty() {
            cmds.push(DrawCommand::DrawText {
                text: self.title.clone(),
                position: Vec2::new(self.rect.x + 6.0, self.rect.y + 4.0),
                color: theme.text_primary,
                size: 12.0,
            });
        }
        // Slots
        for row in 0..self.rows {
            for col in 0..self.cols {
                let sx = self.rect.x + col as f32 * (self.slot_size + self.gap);
                let sy = self.rect.y + row as f32 * (self.slot_size + self.gap) + if self.title.is_empty() { 0.0 } else { 20.0 };
                let slot_rect = Rect::new(sx, sy, self.slot_size, self.slot_size);
                let slot = &self.slots[row * self.cols + col];
                let bg = if slot.highlighted { theme.inventory_slot_hover } else { theme.inventory_slot };
                cmds.push(DrawCommand::DrawRect { rect: slot_rect, color: bg });
                // Item count text
                if slot.count > 1 {
                    cmds.push(DrawCommand::DrawText {
                        text: format!("{}", slot.count),
                        position: Vec2::new(sx + self.slot_size - 12.0, sy + self.slot_size - 10.0),
                        color: theme.text_primary,
                        size: 9.0,
                    });
                }
            }
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// UiMinimap — minimap with camera viewport indicator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct UiMinimap {
    pub rect: Rect,
    pub pixel_size: f32,
    pub border_width: f32,
    pub visible: bool,
}

impl UiMinimap {
    pub fn new(x: f32, y: f32, map_width: usize, map_height: usize, pixel_size: f32) -> Self {
        Self {
            rect: Rect::new(x, y, map_width as f32 * pixel_size, map_height as f32 * pixel_size),
            pixel_size, border_width: 1.0, visible: true,
        }
    }

    /// Render a minimap from tile colors and a camera viewport indicator.
    /// `tile_colors`: flat array [y * map_width + x] = Color
    /// `cam_rect`: the camera's visible world rect (for viewport indicator)
    /// `map_width`: width in tiles
    pub fn render(
        &self,
        tile_colors: &[Color],
        map_width: usize,
        cam_rect: Option<Rect>,
        tile_size: f32,
        theme: &UiTheme,
    ) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        // Background
        cmds.push(DrawCommand::DrawRect { rect: self.rect, color: theme.minimap_bg });
        // Tiles
        let map_height = tile_colors.len() / map_width;
        for ty in 0..map_height {
            for tx in 0..map_width {
                let color = tile_colors[ty * map_width + tx];
                let sx = self.rect.x + tx as f32 * self.pixel_size;
                let sy = self.rect.y + ty as f32 * self.pixel_size;
                cmds.push(DrawCommand::DrawRect {
                    rect: Rect::new(sx, sy, self.pixel_size, self.pixel_size),
                    color,
                });
            }
        }
        // Camera viewport indicator
        if let Some(cam) = cam_rect {
            let vx = self.rect.x + (cam.x / tile_size) * self.pixel_size;
            let vy = self.rect.y + (cam.y / tile_size) * self.pixel_size;
            let vw = (cam.width / tile_size) * self.pixel_size;
            let vh = (cam.height / tile_size) * self.pixel_size;
            let vc = Color::rgba(1.0, 1.0, 1.0, 0.7);
            let cw = 1.0;
            // Top
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy, vw, cw), color: vc });
            // Bottom
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy + vh - cw, vw, cw), color: vc });
            // Left
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy, cw, vh), color: vc });
            // Right
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx + vw - cw, vy, cw, vh), color: vc });
        }
        // Border
        let r = &self.rect;
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, self.border_width), color: theme.minimap_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - self.border_width, r.width, self.border_width), color: theme.minimap_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, self.border_width, r.height), color: theme.minimap_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - self.border_width, r.y, self.border_width, r.height), color: theme.minimap_border });
        cmds
    }
}

// ---------------------------------------------------------------------------
// UIRenderer — orchestrates all UI elements into DrawCommands
// ---------------------------------------------------------------------------

pub struct UIRenderer {
    pub theme: UiTheme,
}

impl UIRenderer {
    pub fn new() -> Self { Self { theme: UiTheme::default_dark() } }
    pub fn with_theme(theme: UiTheme) -> Self { Self { theme } }

    pub fn render_panel(&self, panel: &Panel) -> Vec<DrawCommand> { panel.render(&self.theme) }
    pub fn render_button(&self, button: &Button) -> Vec<DrawCommand> { button.render(&self.theme) }
    pub fn render_text(&self, label: &TextLabel) -> Vec<DrawCommand> {
        label.render().map_or_else(Vec::new, |c| vec![c])
    }
    pub fn render_bar(&self, bar: &Bar) -> Vec<DrawCommand> { bar.render(&self.theme) }
    pub fn render_inventory(&self, grid: &InventoryGrid) -> Vec<DrawCommand> { grid.render(&self.theme) }
    pub fn render_minimap(&self, minimap: &UiMinimap, tile_colors: &[Color], map_width: usize, cam_rect: Option<Rect>, tile_size: f32) -> Vec<DrawCommand> {
        minimap.render(tile_colors, map_width, cam_rect, tile_size, &self.theme)
    }
}

impl Default for UIRenderer {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_render() {
        let panel = Panel::new(10.0, 10.0, 200.0, 100.0).with_title("Inventory");
        let cmds = panel.render(&UiTheme::default_dark());
        assert!(!cmds.is_empty());
        // bg + 4 border + title = 6
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn test_button_hit_test() {
        let btn = Button::new(50.0, 50.0, 100.0, 30.0, "OK");
        assert!(btn.hit_test(Vec2::new(75.0, 65.0)));
        assert!(!btn.hit_test(Vec2::new(10.0, 10.0)));
    }

    #[test]
    fn test_button_states() {
        let mut btn = Button::new(0.0, 0.0, 80.0, 24.0, "Go");
        btn.set_state(ButtonState::Hover);
        let cmds = btn.render(&UiTheme::default_dark());
        assert!(!cmds.is_empty());
    }

    #[test]
    fn test_bar_ratio() {
        let bar = Bar::new(0.0, 0.0, 100.0, 16.0, BarKind::Hp, 75.0, 100.0);
        assert!((bar.ratio() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_bar_zero_max() {
        let bar = Bar::new(0.0, 0.0, 100.0, 16.0, BarKind::Mp, 50.0, 0.0);
        assert_eq!(bar.ratio(), 0.0);
    }

    #[test]
    fn test_inventory_slot_at() {
        let grid = InventoryGrid::new(0.0, 0.0, 4, 4, 32.0);
        let slot = grid.slot_at(Vec2::new(5.0, 5.0));
        assert_eq!(slot, Some((0, 0)));
        let out = grid.slot_at(Vec2::new(500.0, 500.0));
        assert_eq!(out, None);
    }

    #[test]
    fn test_inventory_set_clear() {
        let mut grid = InventoryGrid::new(0.0, 0.0, 3, 3, 32.0);
        grid.set_item(1, 1, 42, 5);
        assert_eq!(grid.slots[4].item_id, Some(42));
        assert_eq!(grid.slots[4].count, 5);
        grid.clear_slot(1, 1);
        assert_eq!(grid.slots[4].item_id, None);
    }

    #[test]
    fn test_text_label() {
        let label = TextLabel::new(10.0, 20.0, "Hello");
        let cmd = label.render().unwrap();
        match cmd {
            DrawCommand::DrawText { text, .. } => assert_eq!(text, "Hello"),
            _ => panic!("expected DrawText"),
        }
    }
}
