// L1 状态栈 — push / pop / replace
// 借鉴: Bevy State + macroquad 架构

use std::any::{Any, TypeId};
use std::collections::HashMap;
use macroquad::prelude::{screen_width, screen_height};

/// 游戏状态 trait — 借鉴 solstack 的 6 个生命周期回调
pub trait GameState {
    /// 进入栈顶时调用 (首次 push 或从下方恢复)
    fn on_start(&mut self) {}
    /// 从栈中移除时调用 (pop 或 replace)
    fn on_stop(&mut self) {}
    /// 被新状态覆盖时调用 (push 了新状态)
    fn on_pause(&mut self) {}
    /// 重新成为栈顶时调用 (上方状态 pop 了)
    fn on_resume(&mut self) {}
    /// 每帧更新 — 返回 StateCommand 驱动状态转换
    fn on_tick(&mut self, ctx: &mut StateContext) -> StateCommand {
        let _ = ctx;
        StateCommand::None
    }
    /// 下方状态也执行的 tick (阴影状态)
    /// （框架缝：当前无状态覆写，保留供未来叠层状态用）
    #[allow(dead_code)]
    fn on_shadow_tick(&mut self, ctx: &mut StateContext) {
        let _ = ctx;
    }
    /// 渲染
    /// （框架缝：渲染已收敛到 render.rs，保留回调位）
    #[allow(dead_code)]
    fn render(&self, ctx: &StateContext) {
        let _ = ctx;
    }
}

/// 状态命令 — 由状态返回，驱动状态栈操作
pub enum StateCommand {
    None,
    Push(Box<dyn GameState>),
    Pop,
    // 框架缝：当前无构造位（apply_command 已就绪），供未来状态替换用
    #[allow(dead_code)]
    Replace(Box<dyn GameState>),
}

/// 状态上下文 — 传递给状态的共享数据
/// （标量字段当前由 ctx.get::<T>() 通道代替读取，保留载荷位）
pub struct StateContext {
    #[allow(dead_code)]
    pub delta_time: f32,
    #[allow(dead_code)]
    pub screen_width: f32,
    #[allow(dead_code)]
    pub screen_height: f32,
    pub data: HashMap<TypeId, Box<dyn Any>>,
}

impl StateContext {
    pub fn new(dt: f32) -> Self {
        Self {
            delta_time: dt,
            screen_width: screen_width(),
            screen_height: screen_height(),
            data: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    // 框架缝：与 get 配对，当前状态实现只读，供未来可写状态用
    #[allow(dead_code)]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }
}

/// 状态栈管理器
pub struct StateStack {
    stack: Vec<Box<dyn GameState>>,
}

impl StateStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// 推入新状态 (当前状态暂停)
    pub fn push(&mut self, mut state: Box<dyn GameState>) {
        if let Some(current) = self.stack.last_mut() {
            current.on_pause();
        }
        state.on_start();
        self.stack.push(state);
    }

    /// 弹出顶部状态 (下方状态恢复)
    pub fn pop(&mut self) -> Option<Box<dyn GameState>> {
        if let Some(mut state) = self.stack.pop() {
            state.on_stop();
            if let Some(current) = self.stack.last_mut() {
                current.on_resume();
            }
            Some(state)
        } else {
            None
        }
    }

    /// 替换顶部状态 (不触发下方的 pause/resume)
    pub fn replace(&mut self, mut state: Box<dyn GameState>) {
        if let Some(mut old) = self.stack.pop() {
            old.on_stop();
        }
        state.on_start();
        self.stack.push(state);
    }

    pub fn current_mut(&mut self) -> Option<&mut dyn GameState> {
        self.stack.last_mut().map(|s| s.as_mut() as &mut dyn GameState)
    }

    pub fn depth(&self) -> usize { self.stack.len() }

    pub fn apply_command(&mut self, cmd: StateCommand) {
        match cmd {
            StateCommand::None => {}
            StateCommand::Push(state) => self.push(state),
            StateCommand::Pop => { self.pop(); }
            StateCommand::Replace(state) => self.replace(state),
        }
    }
}

impl Default for StateStack {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Probe {
        id: u8,
        log: Rc<RefCell<Vec<String>>>,
        /// tick 后续命令：None 无动作，Some(id) 推新屏
        then: RefCell<Option<u8>>,
    }

    impl Probe {
        fn new(id: u8, log: &Rc<RefCell<Vec<String>>>) -> Self {
            Self { id, log: Rc::clone(log), then: RefCell::new(None) }
        }
    }

    impl GameState for Probe {
        fn on_start(&mut self) {
            self.log.borrow_mut().push(format!("start{}", self.id));
        }
        fn on_stop(&mut self) {
            self.log.borrow_mut().push(format!("stop{}", self.id));
        }
        fn on_pause(&mut self) {
            self.log.borrow_mut().push(format!("pause{}", self.id));
        }
        fn on_resume(&mut self) {
            self.log.borrow_mut().push(format!("resume{}", self.id));
        }
        fn on_tick(&mut self, _ctx: &mut StateContext) -> StateCommand {
            match self.then.borrow().as_ref() {
                Some(id) => {
                    let id = *id;
                    StateCommand::Push(Box::new(Probe::new(id, &self.log)))
                }
                None => StateCommand::None,
            }
        }
    }

    fn ctx() -> StateContext {
        StateContext {
            delta_time: 0.016,
            screen_width: 1280.0,
            screen_height: 720.0,
            data: HashMap::new(),
        }
    }

    #[test]
    fn push_pop_lifecycle_order() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut st = StateStack::new();
        st.push(Box::new(Probe::new(1, &log)));
        st.push(Box::new(Probe::new(2, &log)));
        assert_eq!(st.depth(), 2);
        st.pop();
        assert_eq!(st.depth(), 1);
        assert_eq!(
            *log.borrow(),
            vec!["start1", "pause1", "start2", "stop2", "resume1"]
        );
    }

    #[test]
    fn replace_skips_resume() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut st = StateStack::new();
        st.push(Box::new(Probe::new(1, &log)));
        st.replace(Box::new(Probe::new(2, &log)));
        assert_eq!(st.depth(), 1);
        assert_eq!(*log.borrow(), vec!["start1", "stop1", "start2"]);
    }

    #[test]
    fn apply_command_drives_tick_commands() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut st = StateStack::new();
        let p1 = Probe::new(1, &log);
        p1.then.replace(Some(9));
        st.push(Box::new(p1));
        // tick 发 Push(9) → apply 落栈
        let cmd = {
            let mut c = ctx();
            st.current_mut().unwrap().on_tick(&mut c)
        };
        assert!(matches!(cmd, StateCommand::Push(_)));
        st.apply_command(cmd);
        assert_eq!(st.depth(), 2);
        st.apply_command(StateCommand::Pop);
        assert_eq!(st.depth(), 1);
    }

    #[test]
    fn context_insert_get_roundtrip() {
        let mut c = ctx();
        assert!(c.get::<u32>().is_none());
        c.insert(42u32);
        assert_eq!(c.get::<u32>(), Some(&42));
    }
}
