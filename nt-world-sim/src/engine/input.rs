use super::renderer::Vec2;

/// 键盘按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // 字母键
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // 数字键
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // 功能键
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // 控制键
    Space, Enter, Escape, Backspace, Tab, CapsLock,
    Shift, Ctrl, Alt, Meta,
    
    // 方向键
    Up, Down, Left, Right,
    
    // 其他
    Insert, Delete, Home, End, PageUp, PageDown,
    PrintScreen, ScrollLock, Pause,
}

/// 鼠标按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

/// 游戏手柄轴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

/// 游戏手柄按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    A, B, X, Y,
    LeftBumper, RightBumper,
    Back, Start,
    LeftStick, RightStick,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

/// 输入状态
#[derive(Debug, Default)]
pub struct InputState {
    /// 当前按下的键
    pub keys_down: std::collections::HashSet<KeyCode>,
    /// 刚按下的键 (本帧)
    pub keys_just_pressed: std::collections::HashSet<KeyCode>,
    /// 刚释放的键 (本帧)
    pub keys_just_released: std::collections::HashSet<KeyCode>,
    
    /// 鼠标位置
    pub mouse_position: Vec2,
    /// 鼠标移动
    pub mouse_delta: Vec2,
    /// 鼠标滚轮
    pub mouse_scroll: f32,
    /// 当前按下的鼠标按钮
    pub mouse_buttons_down: std::collections::HashSet<MouseButton>,
    /// 刚按下的鼠标按钮
    pub mouse_buttons_just_pressed: std::collections::HashSet<MouseButton>,
    /// 刚释放的鼠标按钮
    pub mouse_buttons_just_released: std::collections::HashSet<MouseButton>,
    
    /// 游戏手柄状态
    pub gamepad_axes: std::collections::HashMap<(usize, GamepadAxis), f32>,
    pub gamepad_buttons_down: std::collections::HashSet<(usize, GamepadButton)>,
    pub gamepad_buttons_just_pressed: std::collections::HashSet<(usize, GamepadButton)>,
    pub gamepad_buttons_just_released: std::collections::HashSet<(usize, GamepadButton)>,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// 清除本帧状态
    pub fn clear_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_delta = Vec2::zero();
        self.mouse_scroll = 0.0;
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.gamepad_buttons_just_pressed.clear();
        self.gamepad_buttons_just_released.clear();
    }
}

/// 输入提供者 trait
pub trait InputProvider {
    /// 键盘是否按下
    fn is_key_pressed(&self, key: KeyCode) -> bool;

    /// 键盘是否刚按下
    fn is_key_just_pressed(&self, key: KeyCode) -> bool;

    /// 键盘是否刚释放
    fn is_key_just_released(&self, key: KeyCode) -> bool;

    /// 鼠标位置
    fn get_mouse_position(&self) -> Vec2;

    /// 鼠标移动
    fn get_mouse_delta(&self) -> Vec2;

    /// 鼠标滚轮
    fn get_mouse_scroll(&self) -> f32;

    /// 鼠标按钮是否按下
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;

    /// 鼠标按钮是否刚按下
    fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool;

    /// 鼠标按钮是否刚释放
    fn is_mouse_button_just_released(&self, button: MouseButton) -> bool;

    /// 游戏手柄轴值
    fn get_gamepad_axis(&self, gamepad: usize, axis: GamepadAxis) -> f32;

    /// 游戏手柄按钮是否按下
    fn is_gamepad_button_pressed(&self, gamepad: usize, button: GamepadButton) -> bool;

    /// 游戏手柄按钮是否刚按下
    fn is_gamepad_button_just_pressed(&self, gamepad: usize, button: GamepadButton) -> bool;

    /// 游戏手柄按钮是否刚释放
    fn is_gamepad_button_just_released(&self, gamepad: usize, button: GamepadButton) -> bool;
}

/// 简单输入状态实现
pub struct SimpleInputProvider {
    state: InputState,
}

impl SimpleInputProvider {
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
        }
    }

    /// 更新输入状态 (由平台层调用)
    pub fn update(&mut self) {
        self.state.clear_frame();
    }

    /// 按下键
    pub fn key_down(&mut self, key: KeyCode) {
        if !self.state.keys_down.contains(&key) {
            self.state.keys_just_pressed.insert(key);
        }
        self.state.keys_down.insert(key);
    }

    /// 释放键
    pub fn key_up(&mut self, key: KeyCode) {
        self.state.keys_down.remove(&key);
        self.state.keys_just_released.insert(key);
    }

    /// 设置鼠标位置
    pub fn set_mouse_position(&mut self, position: Vec2) {
        self.state.mouse_delta = position - self.state.mouse_position;
        self.state.mouse_position = position;
    }

    /// 设置鼠标滚轮
    pub fn set_mouse_scroll(&mut self, scroll: f32) {
        self.state.mouse_scroll = scroll;
    }

    /// 按下鼠标按钮
    pub fn mouse_button_down(&mut self, button: MouseButton) {
        if !self.state.mouse_buttons_down.contains(&button) {
            self.state.mouse_buttons_just_pressed.insert(button);
        }
        self.state.mouse_buttons_down.insert(button);
    }

    /// 释放鼠标按钮
    pub fn mouse_button_up(&mut self, button: MouseButton) {
        self.state.mouse_buttons_down.remove(&button);
        self.state.mouse_buttons_just_released.insert(button);
    }

    /// 设置游戏手柄轴值
    pub fn set_gamepad_axis(&mut self, gamepad: usize, axis: GamepadAxis, value: f32) {
        self.state.gamepad_axes.insert((gamepad, axis), value);
    }

    /// 按下游戏手柄按钮
    pub fn gamepad_button_down(&mut self, gamepad: usize, button: GamepadButton) {
        if !self.state
            .gamepad_buttons_down
            .contains(&(gamepad, button))
        {
            self.state
                .gamepad_buttons_just_pressed
                .insert((gamepad, button));
        }
        self.state
            .gamepad_buttons_down
            .insert((gamepad, button));
    }

    /// 释放游戏手柄按钮
    pub fn gamepad_button_up(&mut self, gamepad: usize, button: GamepadButton) {
        self.state
            .gamepad_buttons_down
            .remove(&(gamepad, button));
        self.state
            .gamepad_buttons_just_released
            .insert((gamepad, button));
    }
}

impl InputProvider for SimpleInputProvider {
    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.state.keys_down.contains(&key)
    }

    fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.state.keys_just_pressed.contains(&key)
    }

    fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.state.keys_just_released.contains(&key)
    }

    fn get_mouse_position(&self) -> Vec2 {
        self.state.mouse_position
    }

    fn get_mouse_delta(&self) -> Vec2 {
        self.state.mouse_delta
    }

    fn get_mouse_scroll(&self) -> f32 {
        self.state.mouse_scroll
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.state.mouse_buttons_down.contains(&button)
    }

    fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.state.mouse_buttons_just_pressed.contains(&button)
    }

    fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.state.mouse_buttons_just_released.contains(&button)
    }

    fn get_gamepad_axis(&self, gamepad: usize, axis: GamepadAxis) -> f32 {
        self.state
            .gamepad_axes
            .get(&(gamepad, axis))
            .copied()
            .unwrap_or(0.0)
    }

    fn is_gamepad_button_pressed(&self, gamepad: usize, button: GamepadButton) -> bool {
        self.state
            .gamepad_buttons_down
            .contains(&(gamepad, button))
    }

    fn is_gamepad_button_just_pressed(&self, gamepad: usize, button: GamepadButton) -> bool {
        self.state
            .gamepad_buttons_just_pressed
            .contains(&(gamepad, button))
    }

    fn is_gamepad_button_just_released(&self, gamepad: usize, button: GamepadButton) -> bool {
        self.state
            .gamepad_buttons_just_released
            .contains(&(gamepad, button))
    }
}

impl Default for SimpleInputProvider {
    fn default() -> Self {
        Self::new()
    }
}
