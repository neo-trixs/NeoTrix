use crate::engine::renderer::{Color, Vec2, Rect, DrawCommand};
use crate::engine::inventory::EquipSlot;

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
    pub dialogue_bg: Color,
    pub dialogue_border: Color,
    pub dialogue_text: Color,
    pub dialogue_name: Color,
    pub choice_bg: Color,
    pub choice_hover: Color,
    pub choice_border: Color,
    pub menu_bg: Color,
    pub menu_overlay: Color,
    pub rarity_common: Color,
    pub rarity_uncommon: Color,
    pub rarity_rare: Color,
    pub rarity_epic: Color,
    pub rarity_legendary: Color,
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
            dialogue_bg: Color::rgba(0.08, 0.08, 0.12, 0.95),
            dialogue_border: Color::rgba(0.5, 0.4, 0.2, 0.9),
            dialogue_text: Color::rgba(0.95, 0.95, 0.9, 1.0),
            dialogue_name: Color::rgba(1.0, 0.85, 0.3, 1.0),
            choice_bg: Color::rgba(0.15, 0.15, 0.22, 0.9),
            choice_hover: Color::rgba(0.25, 0.25, 0.38, 0.95),
            choice_border: Color::rgba(0.4, 0.35, 0.2, 0.8),
            menu_bg: Color::rgba(0.05, 0.05, 0.08, 0.96),
            menu_overlay: Color::rgba(0.0, 0.0, 0.0, 0.6),
            rarity_common: Color::rgba(0.7, 0.7, 0.7, 1.0),
            rarity_uncommon: Color::rgba(0.2, 0.8, 0.2, 1.0),
            rarity_rare: Color::rgba(0.2, 0.4, 0.9, 1.0),
            rarity_epic: Color::rgba(0.7, 0.2, 0.9, 1.0),
            rarity_legendary: Color::rgba(1.0, 0.6, 0.1, 1.0),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self { Self::default_dark() }
}

// ---------------------------------------------------------------------------
// Panel
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
            rect: Rect::new(x, y, w, h), title: None, title_size: 14.0,
            bg_color: Color::rgba(0.12, 0.12, 0.15, 0.92),
            border_color: Color::rgba(0.4, 0.4, 0.5, 0.8),
            border_width: 1.0, visible: true,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self { self.title = Some(title.to_string()); self }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = vec![
            DrawCommand::DrawRect { rect: self.rect, color: self.bg_color },
        ];
        if self.border_width > 0.0 {
            let bw = self.border_width;
            let r = &self.rect;
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, bw), color: self.border_color });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - bw, r.width, bw), color: self.border_color });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, bw, r.height), color: self.border_color });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - bw, r.y, bw, r.height), color: self.border_color });
        }
        if let Some(ref title) = self.title {
            cmds.push(DrawCommand::DrawText {
                text: title.clone(),
                position: Vec2::new(self.rect.x + 8.0, self.rect.y + 6.0),
                color: theme.text_primary, size: self.title_size,
            });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Button
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
        Self { rect: Rect::new(x, y, w, h), label: label.to_string(), state: ButtonState::Normal, label_size: 14.0, visible: true, enabled: true }
    }

    pub fn hit_test(&self, pos: Vec2) -> bool { self.visible && self.enabled && self.rect.contains(&pos) }
    pub fn set_state(&mut self, state: ButtonState) { self.state = state; }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let bg = if !self.enabled { Color::rgba(0.15, 0.15, 0.15, 0.8) } else {
            match self.state {
                ButtonState::Normal => theme.button_normal,
                ButtonState::Hover => theme.button_hover,
                ButtonState::Pressed => theme.button_pressed,
            }
        };
        let text_color = if self.enabled { theme.button_text } else { theme.text_secondary };
        vec![
            DrawCommand::DrawRect { rect: self.rect, color: bg },
            DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y, self.rect.width, 1.0), color: theme.panel_border },
            DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y + self.rect.height - 1.0, self.rect.width, 1.0), color: theme.panel_border },
            DrawCommand::DrawRect { rect: Rect::new(self.rect.x, self.rect.y, 1.0, self.rect.height), color: theme.panel_border },
            DrawCommand::DrawRect { rect: Rect::new(self.rect.x + self.rect.width - 1.0, self.rect.y, 1.0, self.rect.height), color: theme.panel_border },
            DrawCommand::DrawText {
                text: self.label.clone(),
                position: Vec2::new(self.rect.x + 8.0, self.rect.y + (self.rect.height - self.label_size) / 2.0),
                color: text_color, size: self.label_size,
            },
        ]
    }
}

// ---------------------------------------------------------------------------
// TextLabel
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
        Self { position: Vec2::new(x, y), text: text.to_string(), color: Color::rgba(0.95, 0.95, 1.0, 1.0), size: 14.0, visible: true }
    }
    pub fn with_color(mut self, color: Color) -> Self { self.color = color; self }
    pub fn with_size(mut self, size: f32) -> Self { self.size = size; self }

    pub fn render(&self) -> Option<DrawCommand> {
        if !self.visible { return None; }
        Some(DrawCommand::DrawText { text: self.text.clone(), position: self.position, color: self.color, size: self.size })
    }
}

// ---------------------------------------------------------------------------
// Bar
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

    pub fn ratio(&self) -> f32 { if self.max > 0.0 { (self.current / self.max).clamp(0.0, 1.0) } else { 0.0 } }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let fill_color = match self.kind {
            BarKind::Hp => theme.bar_hp, BarKind::Mp => theme.bar_mp, BarKind::Exp => theme.bar_exp,
        };
        let fill_w = self.rect.width * self.ratio();
        let r = &self.rect;
        let mut cmds = vec![
            DrawCommand::DrawRect { rect: self.rect, color: theme.bar_bg },
        ];
        if fill_w > 0.0 {
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, fill_w, r.height), color: fill_color });
        }
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, 1.0), color: theme.panel_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - 1.0, r.width, 1.0), color: theme.panel_border });
        if self.show_text {
            cmds.push(DrawCommand::DrawText {
                text: format!("{:.0}/{:.0}", self.current, self.max),
                position: Vec2::new(r.x + 4.0, r.y + (r.height - self.text_size) / 2.0),
                color: theme.text_primary, size: self.text_size,
            });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// InventoryGrid
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
    pub drag_source: Option<(usize, usize)>,
    pub drag_cursor: Option<Vec2>,
}

impl InventoryGrid {
    pub fn new(x: f32, y: f32, cols: usize, rows: usize, slot_size: f32) -> Self {
        let total_w = cols as f32 * slot_size + (cols - 1) as f32 * 2.0;
        let total_h = rows as f32 * slot_size + (rows - 1) as f32 * 2.0;
        let slots = vec![InventorySlot { item_id: None, count: 0, highlighted: false }; cols * rows];
        Self { rect: Rect::new(x, y, total_w, total_h), cols, rows, slot_size, gap: 2.0, slots, title: String::new(), visible: true, drag_source: None, drag_cursor: None }
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
            *slot = InventorySlot { item_id: None, count: 0, highlighted: false };
        }
    }

    pub fn slot_at(&self, pos: Vec2) -> Option<(usize, usize)> {
        if !self.rect.contains(&pos) { return None; }
        let local_x = pos.x - self.rect.x;
        let local_y = pos.y - self.rect.y - if self.title.is_empty() { 0.0 } else { 20.0 };
        let col = (local_x / (self.slot_size + self.gap)) as usize;
        let row = (local_y / (self.slot_size + self.gap)) as usize;
        if col < self.cols && row < self.rows { Some((col, row)) } else { None }
    }

    pub fn begin_drag(&mut self, col: usize, row: usize) { self.drag_source = Some((col, row)); }
    pub fn update_drag_cursor(&mut self, pos: Vec2) { self.drag_cursor = Some(pos); }
    pub fn end_drag(&mut self) -> Option<((usize, usize), (usize, usize))> {
        let src = self.drag_source.take()?;
        let cursor = self.drag_cursor.take()?;
        let dst = self.slot_at(cursor)?;
        if src == dst { return None; }
        Some((src, dst))
    }

    pub fn swap_slots(&mut self, a: (usize, usize), b: (usize, usize)) {
        let idx_a = a.1 * self.cols + a.0;
        let idx_b = b.1 * self.cols + b.0;
        self.slots.swap(idx_a, idx_b);
    }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = vec![
            DrawCommand::DrawRect { rect: self.rect, color: theme.inventory_bg },
        ];
        let r = &self.rect;
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, 1.0), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - 1.0, r.width, 1.0), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, 1.0, r.height), color: theme.inventory_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x + r.width - 1.0, r.y, 1.0, r.height), color: theme.inventory_border });
        if !self.title.is_empty() {
            cmds.push(DrawCommand::DrawText {
                text: self.title.clone(),
                position: Vec2::new(r.x + 6.0, r.y + 4.0),
                color: theme.text_primary, size: 12.0,
            });
        }
        for row in 0..self.rows {
            for col in 0..self.cols {
                let sx = r.x + col as f32 * (self.slot_size + self.gap);
                let sy = r.y + row as f32 * (self.slot_size + self.gap) + if self.title.is_empty() { 0.0 } else { 20.0 };
                let slot_rect = Rect::new(sx, sy, self.slot_size, self.slot_size);
                let slot = &self.slots[row * self.cols + col];
                let bg = if slot.highlighted { theme.inventory_slot_hover } else { theme.inventory_slot };
                cmds.push(DrawCommand::DrawRect { rect: slot_rect, color: bg });
                if slot.count > 1 {
                    cmds.push(DrawCommand::DrawText {
                        text: format!("{}", slot.count),
                        position: Vec2::new(sx + self.slot_size - 12.0, sy + self.slot_size - 10.0),
                        color: theme.text_primary, size: 9.0,
                    });
                }
            }
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// DialogueBox
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DialogueChoice {
    pub text: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct DialogueBox {
    pub rect: Rect,
    pub speaker: String,
    pub portrait: Option<String>,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub selected_choice: usize,
    pub visible: bool,
    pub text_progress: f32,
    pub text_speed: f32,
    pub full_text: String,
}

impl DialogueBox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            rect: Rect::new(x, y, w, h), speaker: String::new(), portrait: None,
            text: String::new(), choices: Vec::new(), selected_choice: 0,
            visible: false, text_progress: 0.0, text_speed: 30.0, full_text: String::new(),
        }
    }

    pub fn show(&mut self, speaker: &str, text: &str) {
        self.speaker = speaker.to_string();
        self.full_text = text.to_string();
        self.text.clear();
        self.text_progress = 0.0;
        self.choices.clear();
        self.visible = true;
    }

    pub fn show_choices(&mut self, speaker: &str, text: &str, choices: Vec<String>) {
        self.show(speaker, text);
        self.choices = choices.into_iter().enumerate()
            .map(|(i, t)| DialogueChoice { text: t, index: i }).collect();
    }

    pub fn update(&mut self, dt: f32) {
        if !self.visible { return; }
        self.text_progress += self.text_speed * dt;
        let char_count = self.text_progress as usize;
        if char_count >= self.full_text.len() {
            self.text = self.full_text.clone();
        } else {
            self.text = self.full_text[..char_count].to_string();
        }
    }

    pub fn is_text_complete(&self) -> bool { self.text.len() >= self.full_text.len() }
    pub fn skip_text(&mut self) { self.text = self.full_text.clone(); }

    pub fn select_next(&mut self) {
        if !self.choices.is_empty() {
            self.selected_choice = (self.selected_choice + 1) % self.choices.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.choices.is_empty() {
            self.selected_choice = if self.selected_choice == 0 { self.choices.len() - 1 } else { self.selected_choice - 1 };
        }
    }

    pub fn confirm_choice(&self) -> Option<usize> {
        if self.choices.is_empty() { None } else { Some(self.selected_choice) }
    }

    pub fn render(&self, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = vec![
            DrawCommand::DrawRect { rect: self.rect, color: theme.dialogue_bg },
        ];
        // Border
        let r = &self.rect;
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y, r.width, 2.0), color: theme.dialogue_border });
        cmds.push(DrawCommand::DrawRect { rect: Rect::new(r.x, r.y + r.height - 2.0, r.width, 2.0), color: theme.dialogue_border });
        // Speaker name
        if !self.speaker.is_empty() {
            cmds.push(DrawCommand::DrawText {
                text: self.speaker.clone(),
                position: Vec2::new(r.x + 12.0, r.y + 8.0),
                color: theme.dialogue_name, size: 14.0,
            });
        }
        // Dialogue text
        let text_y = if self.speaker.is_empty() { r.y + 12.0 } else { r.y + 28.0 };
        cmds.push(DrawCommand::DrawText {
            text: self.text.clone(),
            position: Vec2::new(r.x + 12.0, text_y),
            color: theme.dialogue_text, size: 12.0,
        });
        // Choices
        if self.is_text_complete() && !self.choices.is_empty() {
            let choice_y_start = r.y + r.height - 10.0 - self.choices.len() as f32 * 22.0;
            for (i, choice) in self.choices.iter().enumerate() {
                let cy = choice_y_start + i as f32 * 22.0;
                let selected = i == self.selected_choice;
                let bg = if selected { theme.choice_hover } else { theme.choice_bg };
                let choice_rect = Rect::new(r.x + 12.0, cy, r.width - 24.0, 20.0);
                cmds.push(DrawCommand::DrawRect { rect: choice_rect, color: bg });
                cmds.push(DrawCommand::DrawText {
                    text: format!("{}. {}", i + 1, choice.text),
                    position: Vec2::new(r.x + 18.0, cy + 3.0),
                    color: if selected { theme.dialogue_name } else { theme.dialogue_text }, size: 11.0,
                });
            }
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// MenuSystem
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    Main,
    Pause,
    Settings,
    Inventory,
    Equipment,
    QuestJournal,
    Map,
    Shop,
    Crafting,
}

pub struct MenuSystem {
    pub current_screen: MenuScreen,
    pub stack: Vec<MenuScreen>,
    pub visible: bool,
    pub settings_volume_master: f32,
    pub settings_volume_music: f32,
    pub settings_volume_sfx: f32,
    pub settings_controls: Vec<(String, String)>,
}

impl MenuSystem {
    pub fn new() -> Self {
        Self {
            current_screen: MenuScreen::Main,
            stack: Vec::new(),
            visible: true,
            settings_volume_master: 1.0,
            settings_volume_music: 0.7,
            settings_volume_sfx: 1.0,
            settings_controls: vec![
                ("Move".into(), "WASD".into()),
                ("Attack".into(), "Space".into()),
                ("Interact".into(), "E".into()),
                ("Inventory".into(), "I".into()),
                ("Map".into(), "M".into()),
                ("Pause".into(), "Escape".into()),
            ],
        }
    }

    pub fn open(&mut self, screen: MenuScreen) {
        self.stack.push(self.current_screen);
        self.current_screen = screen;
        self.visible = true;
    }

    pub fn close(&mut self) -> bool {
        if let Some(prev) = self.stack.pop() {
            self.current_screen = prev;
            if self.stack.is_empty() { self.visible = false; }
            true
        } else {
            self.visible = false;
            false
        }
    }

    pub fn open_main(&mut self) { self.stack.clear(); self.current_screen = MenuScreen::Main; self.visible = true; }
    pub fn open_pause(&mut self) { self.open(MenuScreen::Pause); }
    pub fn open_settings(&mut self) { self.open(MenuScreen::Settings); }
    pub fn open_inventory(&mut self) { self.open(MenuScreen::Inventory); }
    pub fn open_equipment(&mut self) { self.open(MenuScreen::Equipment); }
    pub fn open_quest_journal(&mut self) { self.open(MenuScreen::QuestJournal); }
    pub fn open_map(&mut self) { self.open(MenuScreen::Map); }
    pub fn open_shop(&mut self) { self.open(MenuScreen::Shop); }
    pub fn open_crafting(&mut self) { self.open(MenuScreen::Crafting); }

    pub fn is_main_menu(&self) -> bool { self.current_screen == MenuScreen::Main && self.stack.is_empty() }
}

impl Default for MenuSystem {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Equipment Screen
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EquipmentSlot {
    pub slot: EquipSlot,
    pub item_name: Option<String>,
    pub item_level: Option<u32>,
    pub stat_bonuses: Vec<(String, f32)>,
}

pub struct EquipmentScreen {
    pub slots: Vec<EquipmentSlot>,
    pub selected_slot: Option<usize>,
    pub visible: bool,
}

impl EquipmentScreen {
    pub fn new() -> Self {
        Self {
            slots: vec![
                EquipmentSlot { slot: EquipSlot::Weapon, item_name: None, item_level: None, stat_bonuses: Vec::new() },
                EquipmentSlot { slot: EquipSlot::Armor, item_name: None, item_level: None, stat_bonuses: Vec::new() },
                EquipmentSlot { slot: EquipSlot::Helmet, item_name: None, item_level: None, stat_bonuses: Vec::new() },
                EquipmentSlot { slot: EquipSlot::Boots, item_name: None, item_level: None, stat_bonuses: Vec::new() },
                EquipmentSlot { slot: EquipSlot::Accessory, item_name: None, item_level: None, stat_bonuses: Vec::new() },
            ],
            selected_slot: None,
            visible: false,
        }
    }

    pub fn slot_name(slot: &EquipSlot) -> &'static str {
        match slot {
            EquipSlot::Weapon => "Weapon",
            EquipSlot::Armor => "Armor",
            EquipSlot::Helmet => "Helmet",
            EquipSlot::Boots => "Boots",
            EquipSlot::Accessory => "Accessory",
        }
    }
}

impl Default for EquipmentScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Quest Journal Screen
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct QuestJournalEntry {
    pub quest_id: String,
    pub name: String,
    pub state: String,
    pub progress: f32,
    pub objectives: Vec<String>,
    pub is_main: bool,
}

pub struct QuestJournalScreen {
    pub entries: Vec<QuestJournalEntry>,
    pub selected_index: Option<usize>,
    pub filter: QuestFilter,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestFilter { All, Active, Completed, Failed }

impl QuestJournalScreen {
    pub fn new() -> Self {
        Self { entries: Vec::new(), selected_index: None, filter: QuestFilter::All, visible: false }
    }

    pub fn filtered_entries(&self) -> Vec<&QuestJournalEntry> {
        self.entries.iter().filter(|e| match self.filter {
            QuestFilter::All => true,
            QuestFilter::Active => e.state == "Active",
            QuestFilter::Completed => e.state == "Complete",
            QuestFilter::Failed => e.state == "Failed",
        }).collect()
    }
}

impl Default for QuestJournalScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Shop Screen
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ShopItem {
    pub item_id: String,
    pub name: String,
    pub price: u64,
    pub description: String,
    pub in_stock: bool,
    pub quantity: u32,
}

pub struct ShopScreen {
    pub items: Vec<ShopItem>,
    pub player_gold: u64,
    pub selected_index: Option<usize>,
    pub is_buying: bool,
    pub visible: bool,
}

impl ShopScreen {
    pub fn new() -> Self {
        Self { items: Vec::new(), player_gold: 0, selected_index: None, is_buying: true, visible: false }
    }

    pub fn can_afford(&self) -> bool {
        if let Some(i) = self.selected_index {
            if let Some(item) = self.items.get(i) {
                return self.player_gold >= item.price;
            }
        }
        false
    }
}

impl Default for ShopScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Crafting Screen
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Recipe {
    pub result_item: String,
    pub result_name: String,
    pub result_count: u32,
    pub ingredients: Vec<(String, String, u32)>,
    pub craftable: bool,
}

pub struct CraftingScreen {
    pub recipes: Vec<Recipe>,
    pub selected_index: Option<usize>,
    pub visible: bool,
}

impl CraftingScreen {
    pub fn new() -> Self { Self { recipes: Vec::new(), selected_index: None, visible: false } }
}

impl Default for CraftingScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// Settings Screen
// ---------------------------------------------------------------------------

pub struct SettingsScreen {
    pub volume_master: f32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps: bool,
    pub controls: Vec<(String, String)>,
    pub visible: bool,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            volume_master: 1.0, volume_music: 0.7, volume_sfx: 1.0,
            fullscreen: false, vsync: true, show_fps: false,
            controls: vec![
                ("Move".into(), "WASD".into()),
                ("Attack".into(), "Space".into()),
                ("Interact".into(), "E".into()),
                ("Inventory".into(), "I".into()),
                ("Map".into(), "M".into()),
                ("Pause".into(), "Escape".into()),
            ],
            visible: false,
        }
    }
}

impl Default for SettingsScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// UiMinimap
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
        Self { rect: Rect::new(x, y, map_width as f32 * pixel_size, map_height as f32 * pixel_size), pixel_size, border_width: 1.0, visible: true }
    }

    pub fn render(&self, tile_colors: &[Color], map_width: usize, cam_rect: Option<Rect>, tile_size: f32, theme: &UiTheme) -> Vec<DrawCommand> {
        if !self.visible { return Vec::new(); }
        let mut cmds = vec![DrawCommand::DrawRect { rect: self.rect, color: theme.minimap_bg }];
        let map_height = tile_colors.len() / map_width;
        for ty in 0..map_height {
            for tx in 0..map_width {
                let color = tile_colors[ty * map_width + tx];
                cmds.push(DrawCommand::DrawRect {
                    rect: Rect::new(self.rect.x + tx as f32 * self.pixel_size, self.rect.y + ty as f32 * self.pixel_size, self.pixel_size, self.pixel_size),
                    color,
                });
            }
        }
        if let Some(cam) = cam_rect {
            let vc = Color::rgba(1.0, 1.0, 1.0, 0.7);
            let vx = self.rect.x + (cam.x / tile_size) * self.pixel_size;
            let vy = self.rect.y + (cam.y / tile_size) * self.pixel_size;
            let vw = (cam.width / tile_size) * self.pixel_size;
            let vh = (cam.height / tile_size) * self.pixel_size;
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy, vw, 1.0), color: vc });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy + vh - 1.0, vw, 1.0), color: vc });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx, vy, 1.0, vh), color: vc });
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(vx + vw - 1.0, vy, 1.0, vh), color: vc });
        }
        cmds
    }
}

// ---------------------------------------------------------------------------
// Map Screen
// ---------------------------------------------------------------------------

pub struct MapScreen {
    pub zones: Vec<MapZone>,
    pub current_zone: usize,
    pub discovered: Vec<bool>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct MapZone {
    pub id: String,
    pub name: String,
    pub position: Vec2,
    pub connections: Vec<String>,
    pub color: Color,
}

impl MapScreen {
    pub fn new() -> Self { Self { zones: Vec::new(), current_zone: 0, discovered: Vec::new(), visible: false } }
}

impl Default for MapScreen {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// UIRenderer — orchestrator
// ---------------------------------------------------------------------------

pub struct UIRenderer {
    pub theme: UiTheme,
}

impl UIRenderer {
    pub fn new() -> Self { Self { theme: UiTheme::default_dark() } }
    pub fn with_theme(theme: UiTheme) -> Self { Self { theme } }

    pub fn render_panel(&self, panel: &Panel) -> Vec<DrawCommand> { panel.render(&self.theme) }
    pub fn render_button(&self, button: &Button) -> Vec<DrawCommand> { button.render(&self.theme) }
    pub fn render_text(&self, label: &TextLabel) -> Vec<DrawCommand> { label.render().map_or_else(Vec::new, |c| vec![c]) }
    pub fn render_bar(&self, bar: &Bar) -> Vec<DrawCommand> { bar.render(&self.theme) }
    pub fn render_inventory(&self, grid: &InventoryGrid) -> Vec<DrawCommand> { grid.render(&self.theme) }
    pub fn render_dialogue(&self, dialogue: &DialogueBox) -> Vec<DrawCommand> { dialogue.render(&self.theme) }
    pub fn render_minimap(&self, minimap: &UiMinimap, tile_colors: &[Color], map_width: usize, cam_rect: Option<Rect>, tile_size: f32) -> Vec<DrawCommand> {
        minimap.render(tile_colors, map_width, cam_rect, tile_size, &self.theme)
    }

    /// Render the full equipment screen
    pub fn render_equipment_screen(&self, screen: &EquipmentScreen, x: f32, y: f32, w: f32) -> Vec<DrawCommand> {
        if !screen.visible { return Vec::new(); }
        let mut cmds = Vec::new();
        let panel = Panel::new(x, y, w, 300.0).with_title("Equipment");
        cmds.extend(panel.render(&self.theme));
        for (i, slot) in screen.slots.iter().enumerate() {
            let sy = y + 30.0 + i as f32 * 50.0;
            let slot_rect = Rect::new(x + 10.0, sy, w - 20.0, 44.0);
            let bg = if screen.selected_slot == Some(i) { self.theme.inventory_slot_hover } else { self.theme.inventory_slot };
            cmds.push(DrawCommand::DrawRect { rect: slot_rect, color: bg });
            cmds.push(DrawCommand::DrawText {
                text: EquipmentScreen::slot_name(&slot.slot).to_string(),
                position: Vec2::new(x + 16.0, sy + 4.0),
                color: self.theme.text_secondary, size: 11.0,
            });
            if let Some(ref name) = slot.item_name {
                cmds.push(DrawCommand::DrawText {
                    text: name.clone(),
                    position: Vec2::new(x + 16.0, sy + 20.0),
                    color: self.theme.text_primary, size: 13.0,
                });
            } else {
                cmds.push(DrawCommand::DrawText {
                    text: "Empty".into(),
                    position: Vec2::new(x + 16.0, sy + 20.0),
                    color: self.theme.text_secondary, size: 11.0,
                });
            }
        }
        cmds
    }

    /// Render the full quest journal screen
    pub fn render_quest_journal(&self, screen: &QuestJournalScreen, x: f32, y: f32, w: f32, h: f32) -> Vec<DrawCommand> {
        if !screen.visible { return Vec::new(); }
        let mut cmds = vec![DrawCommand::DrawRect { rect: Rect::new(x, y, w, h), color: self.theme.menu_bg }];
        cmds.push(DrawCommand::DrawText { text: "Quest Journal".into(), position: Vec2::new(x + 10.0, y + 8.0), color: self.theme.text_primary, size: 16.0 });
        // Filter tabs
        let filters = ["All", "Active", "Done", "Failed"];
        for (i, f) in filters.iter().enumerate() {
            let fx = x + 10.0 + i as f32 * 70.0;
            let selected = match (&screen.filter, *f) {
                (QuestFilter::All, "All") => true,
                (QuestFilter::Active, "Active") => true,
                (QuestFilter::Completed, "Done") => true,
                (QuestFilter::Failed, "Failed") => true,
                _ => false,
            };
            let color = if selected { self.theme.dialogue_name } else { self.theme.text_secondary };
            cmds.push(DrawCommand::DrawText { text: f.to_string(), position: Vec2::new(fx, y + 30.0), color, size: 12.0 });
        }
        // Entries
        let entries = screen.filtered_entries();
        for (i, entry) in entries.iter().enumerate() {
            let ey = y + 55.0 + i as f32 * 24.0;
            let color = if entry.is_main { self.theme.dialogue_name } else { self.theme.text_primary };
            cmds.push(DrawCommand::DrawText { text: format!("{} [{}]", entry.name, entry.state), position: Vec2::new(x + 10.0, ey), color, size: 12.0 });
            cmds.push(DrawCommand::DrawText { text: format!("{:.0}%", entry.progress * 100.0), position: Vec2::new(x + w - 40.0, ey), color: self.theme.text_secondary, size: 10.0 });
        }
        cmds
    }

    /// Render the shop screen
    pub fn render_shop_screen(&self, screen: &ShopScreen, x: f32, y: f32, w: f32, h: f32) -> Vec<DrawCommand> {
        if !screen.visible { return Vec::new(); }
        let mut cmds = vec![DrawCommand::DrawRect { rect: Rect::new(x, y, w, h), color: self.theme.menu_bg }];
        cmds.push(DrawCommand::DrawText { text: "Shop".into(), position: Vec2::new(x + 10.0, y + 8.0), color: self.theme.text_primary, size: 16.0 });
        cmds.push(DrawCommand::DrawText { text: format!("Gold: {}", screen.player_gold), position: Vec2::new(x + w - 80.0, y + 8.0), color: self.theme.dialogue_name, size: 14.0 });
        for (i, item) in screen.items.iter().enumerate() {
            let iy = y + 35.0 + i as f32 * 30.0;
            let bg = if screen.selected_index == Some(i) { self.theme.inventory_slot_hover } else { self.theme.inventory_slot };
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(x + 10.0, iy, w - 20.0, 26.0), color: bg });
            cmds.push(DrawCommand::DrawText { text: item.name.clone(), position: Vec2::new(x + 16.0, iy + 5.0), color: self.theme.text_primary, size: 12.0 });
            cmds.push(DrawCommand::DrawText { text: format!("{}g", item.price), position: Vec2::new(x + w - 50.0, iy + 5.0), color: self.theme.dialogue_name, size: 12.0 });
        }
        cmds
    }

    /// Render the crafting screen
    pub fn render_crafting_screen(&self, screen: &CraftingScreen, x: f32, y: f32, w: f32, h: f32) -> Vec<DrawCommand> {
        if !screen.visible { return Vec::new(); }
        let mut cmds = vec![DrawCommand::DrawRect { rect: Rect::new(x, y, w, h), color: self.theme.menu_bg }];
        cmds.push(DrawCommand::DrawText { text: "Crafting".into(), position: Vec2::new(x + 10.0, y + 8.0), color: self.theme.text_primary, size: 16.0 });
        for (i, recipe) in screen.recipes.iter().enumerate() {
            let ry = y + 35.0 + i as f32 * 36.0;
            let bg = if screen.selected_index == Some(i) { self.theme.inventory_slot_hover } else { self.theme.inventory_slot };
            cmds.push(DrawCommand::DrawRect { rect: Rect::new(x + 10.0, ry, w - 20.0, 32.0), color: bg });
            let name_color = if recipe.craftable { self.theme.text_primary } else { self.theme.text_secondary };
            cmds.push(DrawCommand::DrawText { text: format!("{} x{}", recipe.result_name, recipe.result_count), position: Vec2::new(x + 16.0, ry + 4.0), color: name_color, size: 12.0 });
            let ings: Vec<String> = recipe.ingredients.iter().map(|(_, name, count)| format!("{}x{}", count, name)).collect();
            cmds.push(DrawCommand::DrawText { text: ings.join(", "), position: Vec2::new(x + 16.0, ry + 18.0), color: self.theme.text_secondary, size: 10.0 });
        }
        cmds
    }

    /// Render the settings screen
    pub fn render_settings_screen(&self, screen: &SettingsScreen, x: f32, y: f32, w: f32, h: f32) -> Vec<DrawCommand> {
        if !screen.visible { return Vec::new(); }
        let mut cmds = vec![DrawCommand::DrawRect { rect: Rect::new(x, y, w, h), color: self.theme.menu_bg }];
        cmds.push(DrawCommand::DrawText { text: "Settings".into(), position: Vec2::new(x + 10.0, y + 8.0), color: self.theme.text_primary, size: 16.0 });
        let labels = ["Master Volume", "Music Volume", "SFX Volume"];
        let values = [screen.volume_master, screen.volume_music, screen.volume_sfx];
        for (i, (label, &val)) in labels.iter().zip(values.iter()).enumerate() {
            let sy = y + 40.0 + i as f32 * 40.0;
            cmds.push(DrawCommand::DrawText { text: label.to_string(), position: Vec2::new(x + 16.0, sy), color: self.theme.text_primary, size: 12.0 });
            let bar = Bar::new(x + 160.0, sy, w - 180.0, 16.0, BarKind::Hp, val * 100.0, 100.0);
            cmds.extend(bar.render(&self.theme));
        }
        cmds
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
        assert_eq!(cmds.len(), 6);
    }

    #[test]
    fn test_button_hit_test() {
        let btn = Button::new(50.0, 50.0, 100.0, 30.0, "OK");
        assert!(btn.hit_test(Vec2::new(75.0, 65.0)));
        assert!(!btn.hit_test(Vec2::new(10.0, 10.0)));
    }

    #[test]
    fn test_bar_ratio() {
        let bar = Bar::new(0.0, 0.0, 100.0, 16.0, BarKind::Hp, 75.0, 100.0);
        assert!((bar.ratio() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_inventory_slot_at() {
        let grid = InventoryGrid::new(0.0, 0.0, 4, 4, 32.0);
        assert!(grid.slot_at(Vec2::new(5.0, 5.0)).is_some());
        assert!(grid.slot_at(Vec2::new(500.0, 500.0)).is_none());
    }

    #[test]
    fn test_inventory_drag_drop() {
        let mut grid = InventoryGrid::new(0.0, 0.0, 4, 4, 32.0);
        grid.set_item(0, 0, 1, 5);
        grid.set_item(2, 2, 2, 3);
        grid.begin_drag(0, 0);
        grid.update_drag_cursor(Vec2::new(70.0, 70.0));
        let result = grid.end_drag();
        assert!(result.is_some());
        let (src, dst) = result.unwrap();
        grid.swap_slots(src, dst);
        assert_eq!(grid.slots[0].item_id, Some(2));
        assert_eq!(grid.slots[10].item_id, Some(1));
    }

    #[test]
    fn test_dialogue_box() {
        let mut dialogue = DialogueBox::new(10.0, 400.0, 780.0, 180.0);
        dialogue.show("NPC", "Hello traveler!");
        assert!(dialogue.visible);
        dialogue.update(10.0);
        assert!(dialogue.is_text_complete());
        assert_eq!(dialogue.text, "Hello traveler!");
    }

    #[test]
    fn test_dialogue_choices() {
        let mut dialogue = DialogueBox::new(10.0, 400.0, 780.0, 180.0);
        dialogue.show_choices("NPC", "Choose:", vec!["Yes".into(), "No".into()]);
        dialogue.update(10.0);
        dialogue.select_next();
        assert_eq!(dialogue.selected_choice, 1);
        assert_eq!(dialogue.confirm_choice(), Some(1));
    }

    #[test]
    fn test_menu_system() {
        let mut menu = MenuSystem::new();
        assert!(menu.is_main_menu());
        menu.open_pause();
        assert!(!menu.is_main_menu());
        menu.close();
        assert!(menu.is_main_menu());
    }

    #[test]
    fn test_menu_open_close_chain() {
        let mut menu = MenuSystem::new();
        menu.open_inventory();
        menu.open_settings();
        assert_eq!(menu.current_screen, MenuScreen::Settings);
        menu.close();
        assert_eq!(menu.current_screen, MenuScreen::Inventory);
        menu.close();
        assert!(!menu.visible);
    }

    #[test]
    fn test_equipment_screen() {
        let mut screen = EquipmentScreen::new();
        assert_eq!(screen.slots.len(), 5);
        screen.slots[0].item_name = Some("Sword".into());
        assert_eq!(screen.slots[0].item_name, Some("Sword".into()));
    }

    #[test]
    fn test_quest_journal_filter() {
        let mut journal = QuestJournalScreen::new();
        journal.entries.push(QuestJournalEntry { quest_id: "q1".into(), name: "A".into(), state: "Active".into(), progress: 0.5, objectives: vec![], is_main: false });
        journal.entries.push(QuestJournalEntry { quest_id: "q2".into(), name: "B".into(), state: "Complete".into(), progress: 1.0, objectives: vec![], is_main: true });
        journal.filter = QuestFilter::Active;
        assert_eq!(journal.filtered_entries().len(), 1);
    }

    #[test]
    fn test_shop_can_afford() {
        let mut shop = ShopScreen::new();
        shop.player_gold = 50;
        shop.items.push(ShopItem { item_id: "potion".into(), name: "Potion".into(), price: 30, description: String::new(), in_stock: true, quantity: 10 });
        shop.selected_index = Some(0);
        assert!(shop.can_afford());
        shop.player_gold = 10;
        assert!(!shop.can_afford());
    }

    #[test]
    fn test_crafting_screen() {
        let mut craft = CraftingScreen::new();
        craft.recipes.push(Recipe { result_item: "sword".into(), result_name: "Iron Sword".into(), result_count: 1, ingredients: vec![("iron".into(), "Iron".into(), 3)], craftable: true });
        assert_eq!(craft.recipes.len(), 1);
        assert!(craft.recipes[0].craftable);
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
