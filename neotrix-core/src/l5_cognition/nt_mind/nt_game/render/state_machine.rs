/// NT-GAME 状态机 — 吸收 Unity StatePattern + Godot StateMachine
///
/// 泛型状态机，支持 enter/update/exit 生命周期
/// 用于管理游戏状态（主菜单/游戏/对话/暂停/战斗）

use std::collections::HashMap;

/// 游戏状态 trait
pub trait GameState: Send + Sync {
    fn name(&self) -> &str;
    fn enter(&mut self, ctx: &mut StateContext);
    fn update(&mut self, ctx: &mut StateContext, dt: f32);
    fn exit(&mut self, ctx: &mut StateContext);
}

/// 状态机上下文 — 传递给状态的共享数据
pub struct StateContext {
    pub dt: f32,
    pub should_transition: Option<String>,
    pub data: HashMap<String, Box<dyn std::any::Any + Send>>,
}

impl StateContext {
    pub fn new(dt: f32) -> Self {
        Self {
            dt,
            should_transition: None,
            data: HashMap::new(),
        }
    }

    pub fn set_data<T: Send + 'static>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    pub fn get_data<T: Send + 'static>(&self, key: &str) -> Option<&T> {
        self.data
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub fn get_data_mut<T: Send + 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.data
            .get_mut(key)
            .and_then(|v| v.downcast_mut::<T>())
    }
}

/// 游戏状态机
pub struct StateMachine {
    states: HashMap<String, Box<dyn GameState>>,
    current: Option<String>,
    previous: Option<String>,
    pending_transition: Option<String>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current: None,
            previous: None,
            pending_transition: None,
        }
    }

    /// 注册一个状态
    pub fn add_state(&mut self, state: Box<dyn GameState>) {
        let name = state.name().to_string();
        self.states.insert(name, state);
    }

    /// 切换到指定状态
    pub fn transition_to(&mut self, state_name: &str, ctx: &mut StateContext) {
        // 先退出当前状态
        if let Some(ref current) = self.current {
            if let Some(state) = self.states.get_mut(current) {
                state.exit(ctx);
            }
            self.previous = Some(current.clone());
        }

        // 进入新状态
        self.current = Some(state_name.to_string());
        if let Some(state) = self.states.get_mut(state_name) {
            state.enter(ctx);
        }
    }

    /// 请求延迟切换（在下一个 update 中执行）
    pub fn request_transition(&mut self, state_name: &str) {
        self.pending_transition = Some(state_name.to_string());
    }

    /// 更新当前状态
    pub fn update(&mut self, ctx: &mut StateContext) {
        // 执行待处理的切换
        if let Some(target) = self.pending_transition.take() {
            self.transition_to(&target, ctx);
        }

        // 更新当前状态
        if let Some(ref current) = self.current {
            if let Some(state) = self.states.get_mut(current) {
                state.update(ctx, ctx.dt);
            }
        }

        // 检查状态请求的切换
        if let Some(target) = ctx.should_transition.take() {
            self.transition_to(&target, ctx);
        }
    }

    /// 获取当前状态名
    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// 获取上一个状态名
    pub fn previous(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    /// 是否处于指定状态
    pub fn is_current(&self, name: &str) -> bool {
        self.current.as_deref() == Some(name)
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// 内置游戏状态：主菜单
pub struct MainMenuState;

impl GameState for MainMenuState {
    fn name(&self) -> &str {
        "main_menu"
    }
    fn enter(&mut self, _ctx: &mut StateContext) {
        // 初始化主菜单
    }
    fn update(&mut self, _ctx: &mut StateContext, _dt: f32) {
        // 等待玩家输入
    }
    fn exit(&mut self, _ctx: &mut StateContext) {}
}

/// 内置游戏状态：游戏中
pub struct PlayingState;

impl GameState for PlayingState {
    fn name(&self) -> &str {
        "playing"
    }
    fn enter(&mut self, _ctx: &mut StateContext) {}
    fn update(&mut self, _ctx: &mut StateContext, _dt: f32) {
        // 更新游戏世界
    }
    fn exit(&mut self, _ctx: &mut StateContext) {}
}

/// 内置游戏状态：暂停
pub struct PausedState;

impl GameState for PausedState {
    fn name(&self) -> &str {
        "paused"
    }
    fn enter(&mut self, _ctx: &mut StateContext) {}
    fn update(&mut self, _ctx: &mut StateContext, _dt: f32) {}
    fn exit(&mut self, _ctx: &mut StateContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestState {
        name: String,
        enter_count: u32,
        exit_count: u32,
    }

    impl TestState {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                enter_count: 0,
                exit_count: 0,
            }
        }
    }

    impl GameState for TestState {
        fn name(&self) -> &str {
            &self.name
        }
        fn enter(&mut self, _ctx: &mut StateContext) {
            self.enter_count += 1;
        }
        fn update(&mut self, _ctx: &mut StateContext, _dt: f32) {}
        fn exit(&mut self, _ctx: &mut StateContext) {
            self.exit_count += 1;
        }
    }

    #[test]
    fn test_state_machine_transition() {
        let mut sm = StateMachine::new();
        sm.add_state(Box::new(TestState::new("a")));
        sm.add_state(Box::new(TestState::new("b")));

        let mut ctx = StateContext::new(0.016);
        sm.transition_to("a", &mut ctx);
        assert!(sm.is_current("a"));

        sm.transition_to("b", &mut ctx);
        assert!(sm.is_current("b"));
        assert_eq!(sm.previous(), Some("a"));
    }

    #[test]
    fn test_pending_transition() {
        let mut sm = StateMachine::new();
        sm.add_state(Box::new(TestState::new("a")));
        sm.add_state(Box::new(TestState::new("b")));

        let mut ctx = StateContext::new(0.016);
        sm.transition_to("a", &mut ctx);
        sm.request_transition("b");

        sm.update(&mut ctx);
        assert!(sm.is_current("b"));
    }
}
