/// NT-GAME 输入系统 — UE5 Enhanced Input + Godot 4 InputMap 混合架构
///
/// 设计参考：
/// - UE5 Enhanced Input System: InputAction + InputMappingContext
/// - Godot 4 InputMap: 预定义动作 + 优先级上下文
///
/// 核心概念：
/// - InputBinding: 原始输入绑定（键/鼠标/手柄）
/// - InputAction: 逻辑动作定义（包含多个绑定）
/// - InputMappingContext: 动作映射上下文（优先级 + 动作集合）
/// - InputState: 当前帧输入状态
/// - InputManager: 多上下文管理器

use std::collections::{HashMap, HashSet};

// ============================================================================
// InputBinding — 原始输入绑定
// ============================================================================

/// 输入绑定类型 — 支持键盘/鼠标/手柄
#[derive(Debug, Clone, PartialEq)]
pub enum InputBinding {
    /// 键盘按键（扫描码）
    Key(u32),
    /// 鼠标按钮（0=左键, 1=右键, 2=中键, 3+=侧键）
    MouseButton(u8),
    /// 手柄按钮（参考 GamepadButton 常量）
    GamepadButton(u8),
    /// 手柄轴（轴索引 + 死区阈值，正值=右/下，负值=左/上）
    GamepadAxis(u8, f32),
}

impl InputBinding {
    /// 判断绑定是否为轴类型
    pub fn is_axis(&self) -> bool {
        matches!(self, InputBinding::GamepadAxis(_, _))
    }
}

// ============================================================================
// InputAction — 逻辑动作定义
// ============================================================================

/// 输入动作 — 将多个物理绑定映射到逻辑动作
#[derive(Debug, Clone)]
pub struct InputAction {
    /// 动作名称（如 "move_forward", "jump", "attack"）
    pub name: String,
    /// 绑定列表（同一动作可绑定多个输入）
    pub bindings: Vec<InputBinding>,
}

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bindings: Vec::new(),
        }
    }

    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    pub fn with_key(mut self, key: u32) -> Self {
        self.bindings.push(InputBinding::Key(key));
        self
    }

    pub fn with_mouse_button(mut self, button: u8) -> Self {
        self.bindings
            .push(InputBinding::MouseButton(button));
        self
    }

    pub fn with_gamepad_button(mut self, button: u8) -> Self {
        self.bindings
            .push(InputBinding::GamepadButton(button));
        self
    }

    pub fn with_gamepad_axis(mut self, axis: u8, threshold: f32) -> Self {
        self.bindings
            .push(InputBinding::GamepadAxis(axis, threshold));
        self
    }
}

// ============================================================================
// InputMappingContext — 输入映射上下文
// ============================================================================

/// 输入映射上下文 — 优先级 + 动作集合
///
/// 类似 UE5 的 InputMappingContext，支持多上下文叠加
/// 高优先级上下文可以屏蔽低优先级上下文的同名动作
#[derive(Debug, Clone)]
pub struct InputMappingContext {
    /// 上下文名称
    pub name: String,
    /// 动作映射表（动作名 → InputAction）
    pub actions: HashMap<String, InputAction>,
    /// 优先级（数值越大优先级越高）
    pub priority: i32,
}

impl InputMappingContext {
    pub fn new(name: impl Into<String>, priority: i32) -> Self {
        Self {
            name: name.into(),
            actions: HashMap::new(),
            priority,
        }
    }

    /// 添加动作到上下文
    pub fn add_action(&mut self, action: InputAction) {
        self.actions.insert(action.name.clone(), action);
    }

    /// 获取动作
    pub fn get_action(&self, name: &str) -> Option<&InputAction> {
        self.actions.get(name)
    }

    /// 移除动作
    pub fn remove_action(&mut self, name: &str) -> Option<InputAction> {
        self.actions.remove(name)
    }
}

// ============================================================================
// InputState — 当前帧输入状态
// ============================================================================

/// 输入状态 — 存储当前帧的输入数据
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// 当前按下的动作集合
    pub pressed: HashSet<String>,
    /// 本帧刚按下的动作集合
    pub just_pressed: HashSet<String>,
    /// 本帧刚释放的动作集合
    pub just_released: HashSet<String>,
    /// 轴值（支持 analog 输入）
    pub axes: HashMap<String, f32>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空帧状态（保留 pressed 用于下帧比较）
    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

// ============================================================================
// InputManager — 多上下文管理器
// ============================================================================

/// 输入管理器 — 管理多个映射上下文，提供统一查询接口
pub struct InputManager {
    /// 已注册的映射上下文（按优先级降序排列）
    contexts: Vec<InputMappingContext>,
    /// 当前帧输入状态
    state: InputState,
    /// 上一帧按下的动作（用于计算 just_pressed/just_released）
    previous_pressed: HashSet<String>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
            state: InputState::new(),
            previous_pressed: HashSet::new(),
        }
    }

    /// 添加映射上下文（自动按优先级降序排序）
    pub fn add_context(&mut self, ctx: InputMappingContext) {
        self.contexts.push(ctx);
        self.contexts.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 移除映射上下文
    pub fn remove_context(&mut self, name: &str) -> bool {
        let len_before = self.contexts.len();
        self.contexts.retain(|c| c.name != name);
        self.contexts.len() < len_before
    }

    /// 获取上下文引用
    pub fn get_context(&self, name: &str) -> Option<&InputMappingContext> {
        self.contexts.iter().find(|c| c.name == name)
    }

    /// 获取上下文可变引用
    pub fn get_context_mut(&mut self, name: &str) -> Option<&mut InputMappingContext> {
        self.contexts.iter_mut().find(|c| c.name == name)
    }

    /// 获取所有上下文
    pub fn contexts(&self) -> &[InputMappingContext] {
        &self.contexts
    }

    /// 查找动作绑定（按优先级顺序查找第一个匹配的动作）
    pub fn find_action(&self, action_name: &str) -> Option<&InputAction> {
        for ctx in &self.contexts {
            if let Some(action) = ctx.actions.get(action_name) {
                return Some(action);
            }
        }
        None
    }

    /// 更新输入状态
    ///
    /// 需要外部提供原始输入状态（来自平台层：macroquad/gilrs/winit）
    /// `current_pressed` - 当前帧按下的动作名集合
    /// `current_axes` - 当前帧的轴值
    pub fn update(&mut self, current_pressed: &HashSet<String>, current_axes: &HashMap<String, f32>) {
        // 保存当前状态
        self.state.pressed = current_pressed.clone();
        self.state.axes = current_axes.clone();

        // 计算 just_pressed（本帧按下 & 上帧未按下）
        self.state.just_pressed = current_pressed
            .difference(&self.previous_pressed)
            .cloned()
            .collect();

        // 计算 just_released（上帧按下 & 本帧未按下）
        self.state.just_released = self
            .previous_pressed
            .difference(current_pressed)
            .cloned()
            .collect();

        // 保存当前状态供下帧使用
        self.previous_pressed = current_pressed.clone();
    }

    /// 检查动作是否正在按下
    pub fn is_pressed(&self, action: &str) -> bool {
        self.state.pressed.contains(action)
    }

    /// 检查动作是否本帧刚按下
    pub fn is_just_pressed(&self, action: &str) -> bool {
        self.state.just_pressed.contains(action)
    }

    /// 检查动作是否本帧刚释放
    pub fn is_just_released(&self, action: &str) -> bool {
        self.state.just_released.contains(action)
    }

    /// 获取轴值
    pub fn axis(&self, action: &str) -> f32 {
        self.state.axes.get(action).copied().unwrap_or(0.0)
    }

    /// 获取输入状态引用
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// 直接设置输入状态（用于测试或回放）
    pub fn set_state(&mut self, state: InputState) {
        self.state = state;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 常用键码常量（参考 winit/macroquad）
// ============================================================================

/// 键盘码常量
pub mod keys {
    /// 字母键
    pub const KEY_A: u32 = 65;
    pub const KEY_B: u32 = 66;
    pub const KEY_C: u32 = 67;
    pub const KEY_D: u32 = 68;
    pub const KEY_E: u32 = 69;
    pub const KEY_F: u32 = 70;
    pub const KEY_G: u32 = 71;
    pub const KEY_H: u32 = 72;
    pub const KEY_I: u32 = 73;
    pub const KEY_J: u32 = 74;
    pub const KEY_K: u32 = 75;
    pub const KEY_L: u32 = 76;
    pub const KEY_M: u32 = 77;
    pub const KEY_N: u32 = 78;
    pub const KEY_O: u32 = 79;
    pub const KEY_P: u32 = 80;
    pub const KEY_Q: u32 = 81;
    pub const KEY_R: u32 = 82;
    pub const KEY_S: u32 = 83;
    pub const KEY_T: u32 = 84;
    pub const KEY_U: u32 = 85;
    pub const KEY_V: u32 = 86;
    pub const KEY_W: u32 = 87;
    pub const KEY_X: u32 = 88;
    pub const KEY_Y: u32 = 89;
    pub const KEY_Z: u32 = 90;

    /// 功能键
    pub const KEY_SPACE: u32 = 32;
    pub const KEY_ENTER: u32 = 13;
    pub const KEY_ESCAPE: u32 = 27;
    pub const KEY_TAB: u32 = 9;
    pub const KEY_BACKSPACE: u32 = 8;
    pub const KEY_DELETE: u32 = 46;

    /// 方向键
    pub const KEY_UP: u32 = 38;
    pub const KEY_DOWN: u32 = 40;
    pub const KEY_LEFT: u32 = 37;
    pub const KEY_RIGHT: u32 = 39;

    /// Shift/Ctrl/Alt
    pub const KEY_LEFT_SHIFT: u32 = 16;
    pub const KEY_LEFT_CONTROL: u32 = 17;
    pub const KEY_LEFT_ALT: u32 = 18;

    /// 数字键（顶部）
    pub const KEY_0: u32 = 48;
    pub const KEY_1: u32 = 49;
    pub const KEY_2: u32 = 50;
    pub const KEY_3: u32 = 51;
    pub const KEY_4: u32 = 52;
    pub const KEY_5: u32 = 53;
    pub const KEY_6: u32 = 54;
    pub const KEY_7: u32 = 55;
    pub const KEY_8: u32 = 56;
    pub const KEY_9: u32 = 57;
}

/// 手柄按钮常量
pub mod gamepad {
    pub const BTN_SOUTH: u8 = 0;   // A / Cross
    pub const BTN_EAST: u8 = 1;    // B / Circle
    pub const BTN_WEST: u8 = 2;    // X / Square
    pub const BTN_NORTH: u8 = 3;   // Y / Triangle
    pub const BTN_SHOULDER_LEFT: u8 = 4;   // LB / L1
    pub const BTN_SHOULDER_RIGHT: u8 = 5;  // RB / R1
    pub const BTN_BACK: u8 = 6;     // Select / Back
    pub const BTN_START: u8 = 7;    // Start
    pub const BTN_HOME: u8 = 8;     // Guide / Home
    pub const BTN_LEFT_STICK: u8 = 9;
    pub const BTN_RIGHT_STICK: u8 = 10;
    pub const DPAD_UP: u8 = 11;
    pub const DPAD_DOWN: u8 = 12;
    pub const DPAD_LEFT: u8 = 13;
    pub const DPAD_RIGHT: u8 = 14;

    /// 手柄轴索引
    pub const AXIS_LEFT_X: u8 = 0;
    pub const AXIS_LEFT_Y: u8 = 1;
    pub const AXIS_RIGHT_X: u8 = 2;
    pub const AXIS_RIGHT_Y: u8 = 3;
    pub const AXIS_LEFT_TRIGGER: u8 = 4;
    pub const AXIS_RIGHT_TRIGGER: u8 = 5;
}

// ============================================================================
// 便捷构建函数
// ============================================================================

/// 创建一个简单的按键绑定的动作
pub fn key_action(name: impl Into<String>, key: u32) -> InputAction {
    InputAction::new(name).with_key(key)
}

/// 创建一个简单的手柄按钮绑定的动作
pub fn gamepad_action(name: impl Into<String>, button: u8) -> InputAction {
    InputAction::new(name).with_gamepad_button(button)
}

/// 创建一个 2D 移动动作（WASD + 手柄左摇杆）
pub fn movement_2d_action(name: impl Into<String>) -> InputAction {
    InputAction::new(name)
        .with_key(keys::KEY_W)
        .with_key(keys::KEY_UP)
        .with_gamepad_axis(gamepad::AXIS_LEFT_Y, 0.2)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_action_creation() {
        let action = InputAction::new("jump")
            .with_key(keys::KEY_SPACE)
            .with_gamepad_button(gamepad::BTN_SOUTH);

        assert_eq!(action.name, "jump");
        assert_eq!(action.bindings.len(), 2);
        assert_eq!(action.bindings[0], InputBinding::Key(keys::KEY_SPACE));
        assert_eq!(
            action.bindings[1],
            InputBinding::GamepadButton(gamepad::BTN_SOUTH)
        );
    }

    #[test]
    fn test_input_mapping_context() {
        let mut ctx = InputMappingContext::new("gameplay", 10);
        ctx.add_action(InputAction::new("jump").with_key(keys::KEY_SPACE));
        ctx.add_action(InputAction::new("attack").with_key(keys::KEY_Z));

        assert_eq!(ctx.actions.len(), 2);
        assert!(ctx.get_action("jump").is_some());
        assert!(ctx.get_action("nonexistent").is_none());

        ctx.remove_action("jump");
        assert_eq!(ctx.actions.len(), 1);
    }

    #[test]
    fn test_input_manager_context_priority() {
        let mut manager = InputManager::new();

        let low_ctx = InputMappingContext::new("low", 1);
        let high_ctx = InputMappingContext::new("high", 10);

        manager.add_context(low_ctx);
        manager.add_context(high_ctx);

        // 高优先级应该在前面
        assert_eq!(manager.contexts()[0].name, "high");
        assert_eq!(manager.contexts()[1].name, "low");
    }

    #[test]
    fn test_input_manager_find_action() {
        let mut manager = InputManager::new();

        let mut ctx1 = InputMappingContext::new("base", 1);
        ctx1.add_action(InputAction::new("jump").with_key(keys::KEY_SPACE));

        let mut ctx2 = InputMappingContext::new("override", 10);
        ctx2.add_action(InputAction::new("jump").with_key(keys::KEY_UP));

        manager.add_context(ctx1);
        manager.add_context(ctx2);

        // 应该找到高优先级上下文的动作
        let action = manager.find_action("jump").unwrap();
        assert_eq!(action.bindings[0], InputBinding::Key(keys::KEY_UP));
    }

    #[test]
    fn test_input_state_pressed_just_pressed_just_released() {
        let mut manager = InputManager::new();

        let mut pressed = HashSet::new();
        let axes = HashMap::new();

        // 第一帧：按空格
        pressed.insert("jump".to_string());
        manager.update(&pressed, &axes);
        assert!(manager.is_pressed("jump"));
        assert!(manager.is_just_pressed("jump"));
        assert!(!manager.is_just_released("jump"));

        // 第二帧：继续按
        manager.update(&pressed, &axes);
        assert!(manager.is_pressed("jump"));
        assert!(!manager.is_just_pressed("jump"));
        assert!(!manager.is_just_released("jump"));

        // 第三帧：释放
        pressed.clear();
        manager.update(&pressed, &axes);
        assert!(!manager.is_pressed("jump"));
        assert!(!manager.is_just_pressed("jump"));
        assert!(manager.is_just_released("jump"));
    }

    #[test]
    fn test_axis_values() {
        let mut manager = InputManager::new();
        let mut pressed = HashSet::new();
        let mut axes = HashMap::new();

        axes.insert("move_x".to_string(), 0.75);
        axes.insert("move_y".to_string(), -0.5);

        manager.update(&pressed, &axes);

        assert!((manager.axis("move_x") - 0.75).abs() < f32::EPSILON);
        assert!((manager.axis("move_y") - (-0.5)).abs() < f32::EPSILON);
        assert!((manager.axis("nonexistent")).abs() < f32::EPSILON);
    }

    #[test]
    fn test_remove_context() {
        let mut manager = InputManager::new();

        let ctx = InputMappingContext::new("temp", 5);
        manager.add_context(ctx);

        assert_eq!(manager.contexts().len(), 1);
        assert!(manager.remove_context("temp"));
        assert_eq!(manager.contexts().len(), 0);
        assert!(!manager.remove_context("nonexistent"));
    }

    #[test]
    fn test_gamepad_binding() {
        let action = InputAction::new("attack")
            .with_gamepad_button(gamepad::BTN_EAST)
            .with_gamepad_axis(gamepad::AXIS_RIGHT_TRIGGER, 0.5);

        assert_eq!(action.bindings.len(), 2);
        assert!(!action.bindings[0].is_axis());
        assert!(action.bindings[1].is_axis());
    }

    #[test]
    fn test_default_state() {
        let state = InputState::new();
        assert!(state.pressed.is_empty());
        assert!(state.just_pressed.is_empty());
        assert!(state.just_released.is_empty());
        assert!(state.axes.is_empty());
    }
}
