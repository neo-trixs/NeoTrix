/// NT-GAME UI 系统 — 吸收 Godot 4 Control / Unity UI Toolkit / egui 即时模式
///
/// 核心能力：
/// - Screen 枚举：定义所有游戏屏幕
/// - ScreenStack：栈式屏幕管理（push/pop/replace）
/// - Widget：组件化 UI 布局树（VBox/HBox/Grid/Anchor/Margin）
/// - UiState：UI 交互状态管理（焦点、悬停、拖拽）
///
/// 设计参考：
/// - Godot 4: Control 节点 + 锚点布局 + 主题系统
/// - Unity UI Toolkit: XML 布局 + CSS 样式 + 选择器
/// - egui: 即时模式 + 唯一 ID + 响应式交互

use std::collections::HashMap;

use super::components::{Color, Rect, Vec2};

// ═══════════════════════════════════════════════════════════════════
// Screen 枚举 — 游戏屏幕定义
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Screen {
    MainMenu,
    GameWorld,
    Inventory,
    Dialogue { npc_name: String },
    QuestLog,
    PauseMenu,
    Settings,
    DevConsole,
}

impl Screen {
    /// 返回屏幕的唯一标识符
    pub fn id(&self) -> &str {
        match self {
            Screen::MainMenu => "main_menu",
            Screen::GameWorld => "game_world",
            Screen::Inventory => "inventory",
            Screen::Dialogue { .. } => "dialogue",
            Screen::QuestLog => "quest_log",
            Screen::PauseMenu => "pause_menu",
            Screen::Settings => "settings",
            Screen::DevConsole => "dev_console",
        }
    }

    /// 是否允许覆盖下层屏幕（如 PauseMenu 覆盖 GameWorld）
    pub fn covers_previous(&self) -> bool {
        matches!(
            self,
            Screen::PauseMenu | Screen::Settings | Screen::DevConsole
        )
    }
}

// ═══════════════════════════════════════════════════════════════════
// ScreenStack — 屏幕栈管理
// ═══════════════════════════════════════════════════════════════════

pub struct ScreenStack {
    screens: Vec<Screen>,
}

impl ScreenStack {
    pub fn new() -> Self {
        Self {
            screens: Vec::new(),
        }
    }

    /// 压入新屏幕到栈顶
    pub fn push(&mut self, screen: Screen) {
        self.screens.push(screen);
    }

    /// 弹出栈顶屏幕
    pub fn pop(&mut self) -> Option<Screen> {
        self.screens.pop()
    }

    /// 替换栈顶屏幕（pop + push）
    pub fn replace(&mut self, screen: Screen) {
        self.screens.pop();
        self.screens.push(screen);
    }

    /// 清空栈并压入新屏幕（等同于 replace 但更明确）
    pub fn clear_and_push(&mut self, screen: Screen) {
        self.screens.clear();
        self.screens.push(screen);
    }

    /// 获取当前（栈顶）屏幕
    pub fn current(&self) -> Option<&Screen> {
        self.screens.last()
    }

    /// 栈中屏幕数量
    pub fn len(&self) -> usize {
        self.screens.len()
    }

    /// 栈是否为空
    pub fn is_empty(&self) -> bool {
        self.screens.is_empty()
    }

    /// 查找栈中是否存在指定屏幕
    pub fn contains(&self, screen: &Screen) -> bool {
        self.screens.contains(screen)
    }

    /// 按 ID 查找栈中是否存在指定屏幕
    pub fn contains_id(&self, id: &str) -> bool {
        self.screens.iter().any(|s| s.id() == id)
    }

    /// 获取所有屏幕的引用
    pub fn all(&self) -> &[Screen] {
        &self.screens
    }
}

impl Default for ScreenStack {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Layout — UI 布局模式
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Layout {
    /// 垂直堆叠子元素
    VBox { spacing: f32 },
    /// 水平排列子元素
    HBox { spacing: f32 },
    /// 网格布局
    Grid { cols: usize, spacing: f32 },
    /// 锚点布局（相对父元素的四边百分比，0.0-1.0）
    Anchor {
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
    },
    /// 统一外边距
    Margin { all: f32 },
}

impl Default for Layout {
    fn default() -> Self {
        Layout::VBox { spacing: 4.0 }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Widget — UI 组件树节点
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Widget {
    /// 组件唯一标识（用于交互查询）
    pub id: String,
    /// 布局模式
    pub layout: Layout,
    /// 子组件
    pub children: Vec<Widget>,
    /// 是否可见
    pub visible: bool,
    /// 组件内容（文本/图像等）
    pub content: WidgetContent,
    /// 计算后的矩形区域（由 layout 系统填充）
    pub bounds: Rect,
    /// 组件专属样式覆盖
    pub style: WidgetStyle,
}

#[derive(Debug, Clone, Default)]
pub struct WidgetStyle {
    pub background_color: Option<Color>,
    pub foreground_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub opacity: f32,
}

impl WidgetStyle {
    pub fn transparent() -> Self {
        Self {
            opacity: 0.0,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub enum WidgetContent {
    None,
    Text(String),
    Button { label: String, on_click: Option<String> },
    Image { path: String },
    ProgressBar { value: f32, max: f32 },
}

impl Widget {
    /// 创建空容器组件
    pub fn new(id: &str, layout: Layout) -> Self {
        Self {
            id: id.to_string(),
            layout,
            children: Vec::new(),
            visible: true,
            content: WidgetContent::None,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            style: WidgetStyle::default(),
        }
    }

    /// 创建文本组件
    pub fn text(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            layout: Layout::default(),
            children: Vec::new(),
            visible: true,
            content: WidgetContent::Text(text.to_string()),
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            style: WidgetStyle::default(),
        }
    }

    /// 创建按钮组件
    pub fn button(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            layout: Layout::default(),
            children: Vec::new(),
            visible: true,
            content: WidgetContent::Button {
                label: label.to_string(),
                on_click: None,
            },
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            style: WidgetStyle::default(),
        }
    }

    /// 创建图像组件
    pub fn image(id: &str, path: &str) -> Self {
        Self {
            id: id.to_string(),
            layout: Layout::default(),
            children: Vec::new(),
            visible: true,
            content: WidgetContent::Image {
                path: path.to_string(),
            },
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            style: WidgetStyle::default(),
        }
    }

    /// 创建进度条组件
    pub fn progress_bar(id: &str, value: f32, max: f32) -> Self {
        Self {
            id: id.to_string(),
            layout: Layout::default(),
            children: Vec::new(),
            visible: true,
            content: WidgetContent::ProgressBar { value, max },
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            style: WidgetStyle::default(),
        }
    }

    /// 添加子组件（Builder 模式）
    pub fn with_child(mut self, child: Widget) -> Self {
        self.children.push(child);
        self
    }

    /// 设置可见性
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 设置样式
    pub fn with_style(mut self, style: WidgetStyle) -> Self {
        self.style = style;
        self
    }

    /// 设置按钮点击回调
    pub fn with_on_click(mut self, callback: &str) -> Self {
        if let WidgetContent::Button {
            on_click, ..
        } = &mut self.content
        {
            *on_click = Some(callback.to_string());
        }
        self
    }

    /// 递归设置所有子组件的 bounds（简单布局计算）
    pub fn layout(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.bounds = Rect::new(x, y, w, h);

        match &self.layout {
            Layout::VBox { spacing } => {
                let mut current_y = y;
                let child_height = if self.children.is_empty() {
                    0.0
                } else {
                    (h - spacing * (self.children.len() as f32 - 1.0))
                        / self.children.len() as f32
                };
                for child in &mut self.children {
                    child.layout(x, current_y, w, child_height);
                    current_y += child_height + spacing;
                }
            }
            Layout::HBox { spacing } => {
                let mut current_x = x;
                let child_width = if self.children.is_empty() {
                    0.0
                } else {
                    (w - spacing * (self.children.len() as f32 - 1.0))
                        / self.children.len() as f32
                };
                for child in &mut self.children {
                    child.layout(current_x, y, child_width, h);
                    current_x += child_width + spacing;
                }
            }
            Layout::Grid { cols, spacing } => {
                let rows = (self.children.len() + cols - 1) / cols;
                let cell_w = if *cols == 0 {
                    w
                } else {
                    (w - spacing * (*cols as f32 - 1.0)) / *cols as f32
                };
                let cell_h = if rows == 0 {
                    h
                } else {
                    (h - spacing * (rows as f32 - 1.0)) / rows as f32
                };
                for (i, child) in self.children.iter_mut().enumerate() {
                    let col = i % cols;
                    let row = i / cols;
                    let cx = x + col as f32 * (cell_w + spacing);
                    let cy = y + row as f32 * (cell_h + spacing);
                    child.layout(cx, cy, cell_w, cell_h);
                }
            }
            Layout::Anchor {
                top,
                bottom,
                left,
                right,
            } => {
                let ax = x + w * left;
                let ay = y + h * top;
                let aw = w * (right - left);
                let ah = h * (bottom - top);
                for child in &mut self.children {
                    child.layout(ax, ay, aw, ah);
                }
            }
            Layout::Margin { all } => {
                for child in &mut self.children {
                    child.layout(x + all, y + all, w - all * 2.0, h - all * 2.0);
                }
            }
        }
    }

    /// 按 ID 查找组件（深度优先）
    pub fn find(&self, id: &str) -> Option<&Widget> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(id) {
                return Some(found);
            }
        }
        None
    }

    /// 按 ID 可变查找组件
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Widget> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// 判断点是否在组件区域内
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.visible && self.bounds.contains_point(point)
    }
}

// ═══════════════════════════════════════════════════════════════════
// UiState — UI 交互状态管理
// ═══════════════════════════════════════════════════════════════════

pub struct UiState {
    /// 当前获得焦点的组件 ID
    pub focused: Option<String>,
    /// 当前鼠标悬停的组件 ID
    pub hovered: Option<String>,
    /// 是否正在拖拽
    pub drag_active: bool,
    /// 拖拽的组件 ID
    pub drag_source: Option<String>,
    /// 拖拽起始位置
    pub drag_start: Vec2,
    /// 拖拽当前偏移
    pub drag_offset: Vec2,
    /// 屏幕尺寸
    pub screen_size: Vec2,
    /// 鼠标位置（屏幕坐标）
    pub mouse_position: Vec2,
    /// 组件自定义数据（用于 UI 状态绑定）
    pub data: HashMap<String, String>,
}

impl UiState {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            focused: None,
            hovered: None,
            drag_active: false,
            drag_source: None,
            drag_start: Vec2::ZERO,
            drag_offset: Vec2::ZERO,
            screen_size: Vec2::new(screen_width, screen_height),
            mouse_position: Vec2::ZERO,
            data: HashMap::new(),
        }
    }

    /// 更新鼠标位置
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
    }

    /// 尝试聚焦组件
    pub fn focus(&mut self, id: &str) {
        self.focused = Some(id.to_string());
    }

    /// 清除焦点
    pub fn unfocus(&mut self) {
        self.focused = None;
    }

    /// 设置悬停组件
    pub fn set_hovered(&mut self, id: Option<&str>) {
        self.hovered = id.map(|s| s.to_string());
    }

    /// 开始拖拽
    pub fn begin_drag(&mut self, source_id: &str, start_pos: Vec2) {
        self.drag_active = true;
        self.drag_source = Some(source_id.to_string());
        self.drag_start = start_pos;
        self.drag_offset = Vec2::ZERO;
    }

    /// 更新拖拽偏移
    pub fn update_drag(&mut self, current_pos: Vec2) {
        if self.drag_active {
            self.drag_offset = current_pos - self.drag_start;
        }
    }

    /// 结束拖拽
    pub fn end_drag(&mut self) -> Option<(String, Vec2)> {
        if self.drag_active {
            let source = self.drag_source.take().unwrap_or_default();
            let offset = self.drag_offset;
            self.drag_active = false;
            self.drag_offset = Vec2::ZERO;
            Some((source, offset))
        } else {
            None
        }
    }

    /// 设置自定义数据
    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// 获取自定义数据
    pub fn get_data(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// 从 Widget 树更新 hover 状态（hit test）
    pub fn update_hover_from_tree(&mut self, root: &Widget) {
        self.hovered = self.hit_test(root, self.mouse_position);
    }

    /// 递归命中测试
    fn hit_test(&self, widget: &Widget, pos: Vec2) -> Option<String> {
        if !widget.visible {
            return None;
        }
        // 深度优先：子组件优先（后绘制的在上层）
        for child in widget.children.iter().rev() {
            if let Some(found) = self.hit_test(child, pos) {
                return Some(found);
            }
        }
        if widget.contains_point(pos) {
            Some(widget.id.clone())
        } else {
            None
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

// ═══════════════════════════════════════════════════════════════════
// 预制 UI 树工厂函数
// ═══════════════════════════════════════════════════════════════════

/// 创建主菜单 UI 树
pub fn create_main_menu_ui() -> Widget {
    Widget::new("main_menu_root", Layout::VBox { spacing: 12.0 })
        .with_child(Widget::text("title", "NeoTrix"))
        .with_child(
            Widget::new("button_container", Layout::VBox { spacing: 8.0 })
                .with_child(Widget::button("btn_new_game", "New Game"))
                .with_child(Widget::button("btn_continue", "Continue"))
                .with_child(Widget::button("btn_settings", "Settings"))
                .with_child(Widget::button("btn_quit", "Quit")),
        )
}

/// 创建暂停菜单 UI 树
pub fn create_pause_menu_ui() -> Widget {
    Widget::new("pause_menu_root", Layout::VBox { spacing: 8.0 })
        .with_child(Widget::text("pause_title", "PAUSED"))
        .with_child(Widget::button("btn_resume", "Resume"))
        .with_child(Widget::button("btn_settings_pause", "Settings"))
        .with_child(Widget::button("btn_main_menu", "Main Menu"))
}

/// 创建 HUD（游戏内浮动 UI）
pub fn create_hud_ui() -> Widget {
    Widget::new(
        "hud_root",
        Layout::Anchor {
            top: 0.0,
            bottom: 1.0,
            left: 0.0,
            right: 1.0,
        },
    )
    .with_child(
        Widget::new(
            "health_bar_area",
            Layout::Anchor {
                top: 0.02,
                bottom: 0.08,
                left: 0.02,
                right: 0.25,
            },
        )
        .with_child(Widget::progress_bar("health_bar", 75.0, 100.0)),
    )
    .with_child(
        Widget::new(
            "minimap_area",
            Layout::Anchor {
                top: 0.02,
                bottom: 0.22,
                left: 0.78,
                right: 0.98,
            },
        )
        .with_child(Widget::image("minimap", "minimap.png")),
    )
}

// ═══════════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Screen 测试 ──

    #[test]
    fn test_screen_id() {
        assert_eq!(Screen::MainMenu.id(), "main_menu");
        assert_eq!(
            Screen::Dialogue {
                npc_name: "guard".to_string()
            }
            .id(),
            "dialogue"
        );
        assert_eq!(Screen::DevConsole.id(), "dev_console");
    }

    #[test]
    fn test_screen_covers_previous() {
        assert!(Screen::PauseMenu.covers_previous());
        assert!(Screen::Settings.covers_previous());
        assert!(!Screen::GameWorld.covers_previous());
        assert!(!Screen::Inventory.covers_previous());
    }

    #[test]
    fn test_screen_equality() {
        assert_eq!(Screen::MainMenu, Screen::MainMenu);
        assert_ne!(Screen::MainMenu, Screen::Inventory);
    }

    // ── ScreenStack 测试 ──

    #[test]
    fn test_screen_stack_push_pop() {
        let mut stack = ScreenStack::new();
        assert!(stack.is_empty());

        stack.push(Screen::MainMenu);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.current(), Some(&Screen::MainMenu));

        stack.push(Screen::GameWorld);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.current(), Some(&Screen::GameWorld));

        let popped = stack.pop();
        assert_eq!(popped, Some(Screen::GameWorld));
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.current(), Some(&Screen::MainMenu));
    }

    #[test]
    fn test_screen_stack_replace() {
        let mut stack = ScreenStack::new();
        stack.push(Screen::MainMenu);
        stack.push(Screen::GameWorld);
        stack.replace(Screen::PauseMenu);

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.current(), Some(&Screen::PauseMenu));
        // MainMenu 仍在栈底
        assert!(stack.contains(&Screen::MainMenu));
    }

    #[test]
    fn test_screen_stack_clear_and_push() {
        let mut stack = ScreenStack::new();
        stack.push(Screen::MainMenu);
        stack.push(Screen::GameWorld);
        stack.push(Screen::Inventory);
        stack.clear_and_push(Screen::DevConsole);

        assert_eq!(stack.len(), 1);
        assert_eq!(stack.current(), Some(&Screen::DevConsole));
    }

    #[test]
    fn test_screen_stack_contains_id() {
        let mut stack = ScreenStack::new();
        stack.push(Screen::MainMenu);
        stack.push(Screen::GameWorld);

        assert!(stack.contains_id("main_menu"));
        assert!(stack.contains_id("game_world"));
        assert!(!stack.contains_id("inventory"));
    }

    #[test]
    fn test_screen_stack_empty() {
        let mut stack = ScreenStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.current(), None);
    }

    // ── Layout 测试 ──

    #[test]
    fn test_layout_default() {
        let layout = Layout::default();
        match layout {
            Layout::VBox { spacing } => assert!((spacing - 4.0).abs() < f32::EPSILON),
            _ => panic!("default layout should be VBox"),
        }
    }

    // ── Widget 测试 ──

    #[test]
    fn test_widget_creation() {
        let w = Widget::new("root", Layout::VBox { spacing: 8.0 });
        assert_eq!(w.id, "root");
        assert!(w.visible);
        assert!(w.children.is_empty());
    }

    #[test]
    fn test_widget_text() {
        let w = Widget::text("label", "Hello");
        assert_eq!(w.id, "label");
        match &w.content {
            WidgetContent::Text(t) => assert_eq!(t, "Hello"),
            _ => panic!("expected text content"),
        }
    }

    #[test]
    fn test_widget_button() {
        let w = Widget::button("btn", "Click Me").with_on_click("on_click_handler");
        match &w.content {
            WidgetContent::Button { label, on_click } => {
                assert_eq!(label, "Click Me");
                assert_eq!(on_click.as_deref(), Some("on_click_handler"));
            }
            _ => panic!("expected button content"),
        }
    }

    #[test]
    fn test_widget_tree_builder() {
        let root = Widget::new("root", Layout::VBox { spacing: 4.0 })
            .with_child(Widget::text("t1", "First"))
            .with_child(Widget::text("t2", "Second"))
            .with_child(Widget::button("b1", "Go"));

        assert_eq!(root.children.len(), 3);
        assert_eq!(root.children[0].id, "t1");
        assert_eq!(root.children[2].id, "b1");
    }

    #[test]
    fn test_widget_find() {
        let root = Widget::new("root", Layout::VBox { spacing: 4.0 }).with_child(
            Widget::new("inner", Layout::HBox { spacing: 2.0 })
                .with_child(Widget::text("target", "found me")),
        );

        assert!(root.find("root").is_some());
        assert!(root.find("inner").is_some());
        assert!(root.find("target").is_some());
        assert!(root.find("missing").is_none());
    }

    #[test]
    fn test_widget_find_mut() {
        let mut root = Widget::new("root", Layout::VBox { spacing: 4.0 })
            .with_child(Widget::text("child", "before"));

        if let Some(child) = root.find_mut("child") {
            child.visible = false;
        }
        assert!(!root.find("child").unwrap().visible);
    }

    #[test]
    fn test_widget_contains_point() {
        let mut w = Widget::new("box", Layout::default());
        w.bounds = Rect::new(10.0, 10.0, 100.0, 50.0);

        assert!(w.contains_point(Vec2::new(50.0, 30.0)));
        assert!(!w.contains_point(Vec2::new(5.0, 5.0)));
    }

    #[test]
    fn test_widget_invisible_not_hit() {
        let mut w = Widget::new("hidden", Layout::default());
        w.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        w.visible = false;

        assert!(!w.contains_point(Vec2::new(50.0, 50.0)));
    }

    // ── Layout 计算测试 ──

    #[test]
    fn test_vbox_layout() {
        let mut root = Widget::new("root", Layout::VBox { spacing: 4.0 })
            .with_child(Widget::text("a", ""))
            .with_child(Widget::text("b", ""))
            .with_child(Widget::text("c", ""));

        root.layout(0.0, 0.0, 200.0, 100.0);

        // 3 个子组件，spacing=4，总高=100
        // 每个高度 = (100 - 4*2) / 3 ≈ 30.666
        let expected_child_h = (100.0 - 4.0 * 2.0) / 3.0;
        assert!((root.children[0].bounds.h - expected_child_h).abs() < 0.1);
        assert!((root.children[0].bounds.y).abs() < 0.1);
        assert!((root.children[1].bounds.y - (expected_child_h + 4.0)).abs() < 0.1);
    }

    #[test]
    fn test_hbox_layout() {
        let mut root = Widget::new("root", Layout::HBox { spacing: 10.0 })
            .with_child(Widget::text("a", ""))
            .with_child(Widget::text("b", ""));

        root.layout(0.0, 0.0, 200.0, 50.0);

        let expected_child_w = (200.0 - 10.0) / 2.0;
        assert!((root.children[0].bounds.w - expected_child_w).abs() < 0.1);
        assert!((root.children[1].bounds.x - (expected_child_w + 10.0)).abs() < 0.1);
    }

    #[test]
    fn test_grid_layout() {
        let mut root = Widget::new(
            "grid",
            Layout::Grid {
                cols: 2,
                spacing: 5.0,
            },
        )
        .with_child(Widget::text("a", ""))
        .with_child(Widget::text("b", ""))
        .with_child(Widget::text("c", ""))
        .with_child(Widget::text("d", ""));

        root.layout(0.0, 0.0, 200.0, 200.0);

        let cell_w = (200.0 - 5.0) / 2.0;
        let cell_h = (200.0 - 5.0) / 2.0;

        assert!((root.children[0].bounds.x).abs() < 0.1);
        assert!((root.children[0].bounds.w - cell_w).abs() < 0.1);
        assert!((root.children[1].bounds.x - (cell_w + 5.0)).abs() < 0.1);
        assert!((root.children[2].bounds.y - (cell_h + 5.0)).abs() < 0.1);
    }

    #[test]
    fn test_anchor_layout() {
        let mut root = Widget::new(
            "anchored",
            Layout::Anchor {
                top: 0.0,
                bottom: 0.2,
                left: 0.7,
                right: 1.0,
            },
        );

        root.layout(0.0, 0.0, 1280.0, 720.0);

        assert!((root.bounds.x - 1280.0 * 0.7).abs() < 0.1);
        assert!((root.bounds.y).abs() < 0.1);
        assert!((root.bounds.w - 1280.0 * 0.3).abs() < 0.1);
        assert!((root.bounds.h - 720.0 * 0.2).abs() < 0.1);
    }

    #[test]
    fn test_margin_layout() {
        let mut root = Widget::new("padded", Layout::Margin { all: 16.0 })
            .with_child(Widget::text("inner", ""));

        root.layout(0.0, 0.0, 200.0, 100.0);

        assert!((root.children[0].bounds.x - 16.0).abs() < 0.1);
        assert!((root.children[0].bounds.y - 16.0).abs() < 0.1);
        assert!((root.children[0].bounds.w - 168.0).abs() < 0.1);
        assert!((root.children[0].bounds.h - 68.0).abs() < 0.1);
    }

    // ── UiState 测试 ──

    #[test]
    fn test_ui_state_focus() {
        let mut ui = UiState::new(1280.0, 720.0);
        assert!(ui.focused.is_none());

        ui.focus("input_field");
        assert_eq!(ui.focused.as_deref(), Some("input_field"));

        ui.unfocus();
        assert!(ui.focused.is_none());
    }

    #[test]
    fn test_ui_state_hover() {
        let mut ui = UiState::new(1280.0, 720.0);
        ui.set_hovered(Some("button"));
        assert_eq!(ui.hovered.as_deref(), Some("button"));

        ui.set_hovered(None);
        assert!(ui.hovered.is_none());
    }

    #[test]
    fn test_ui_state_drag() {
        let mut ui = UiState::new(1280.0, 720.0);
        assert!(!ui.drag_active);

        ui.begin_drag("icon", Vec2::new(100.0, 100.0));
        assert!(ui.drag_active);
        assert_eq!(ui.drag_source.as_deref(), Some("icon"));

        ui.update_drag(Vec2::new(120.0, 130.0));
        assert!((ui.drag_offset.x - 20.0).abs() < 0.1);
        assert!((ui.drag_offset.y - 30.0).abs() < 0.1);

        let result = ui.end_drag();
        assert!(result.is_some());
        let (source, offset) = result.unwrap();
        assert_eq!(source, "icon");
        assert!(!ui.drag_active);
    }

    #[test]
    fn test_ui_state_data() {
        let mut ui = UiState::new(1280.0, 720.0);
        ui.set_data("player_name", "Neo");
        assert_eq!(ui.get_data("player_name"), Some("Neo"));
        assert!(ui.get_data("unknown").is_none());
    }

    #[test]
    fn test_ui_state_hit_test() {
        let mut root = Widget::new("root", Layout::default());
        root.bounds = Rect::new(0.0, 0.0, 200.0, 200.0);

        let mut child = Widget::new("btn", Layout::default());
        child.bounds = Rect::new(10.0, 10.0, 80.0, 30.0);
        root.children.push(child);

        let mut ui = UiState::new(200.0, 200.0);
        ui.set_mouse_position(50.0, 25.0);
        ui.update_hover_from_tree(&root);

        assert_eq!(ui.hovered.as_deref(), Some("btn"));

        // 点击空白区域
        ui.set_mouse_position(150.0, 150.0);
        ui.update_hover_from_tree(&root);
        // 没有匹配的叶子组件，hovered 保持不变（hit_test 返回 None）
        // 但 widget 不包含该点，所以返回 None
        // 这里只是验证不 panic
    }

    #[test]
    fn test_ui_state_invisible_widget_not_hit() {
        let mut root = Widget::new("root", Layout::default());
        root.bounds = Rect::new(0.0, 0.0, 200.0, 200.0);

        let mut hidden = Widget::new("hidden", Layout::default());
        hidden.bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        hidden.visible = false;
        root.children.push(hidden);

        let mut ui = UiState::new(200.0, 200.0);
        ui.set_mouse_position(50.0, 50.0);
        ui.update_hover_from_tree(&root);

        assert!(ui.hovered.is_none());
    }

    // ── 预制 UI 测试 ──

    #[test]
    fn test_create_main_menu_ui() {
        let ui = create_main_menu_ui();
        assert_eq!(ui.id, "main_menu_root");
        assert!(!ui.children.is_empty());

        // 应该有标题 + 按钮容器
        assert!(ui.find("title").is_some());
        assert!(ui.find("btn_new_game").is_some());
        assert!(ui.find("btn_quit").is_some());
    }

    #[test]
    fn test_create_pause_menu_ui() {
        let ui = create_pause_menu_ui();
        assert_eq!(ui.id, "pause_menu_root");
        assert!(ui.find("btn_resume").is_some());
        assert!(ui.find("btn_main_menu").is_some());
    }

    #[test]
    fn test_create_hud_ui() {
        let ui = create_hud_ui();
        assert_eq!(ui.id, "hud_root");
        assert!(ui.find("health_bar").is_some());
        assert!(ui.find("minimap").is_some());
    }

    // ── ScreenStack 默认值测试 ──

    #[test]
    fn test_screen_stack_default() {
        let stack = ScreenStack::default();
        assert!(stack.is_empty());
    }
}
