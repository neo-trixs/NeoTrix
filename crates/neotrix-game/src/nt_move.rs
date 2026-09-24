//! 平台动作套件 — swordgame.ai 类武侠动作的引擎底座.
//!
//! 公理（平台手感 canon）：土狼时间（离崖后仍可跳）+ 跳跃预输入缓冲（提前按也算）+
//! 二段跳 + 可变跳高（松开即截断上升速度）+ 冲刺（cd 独立、短时无敌由调用方定）。
//! 全纯函数/纯状态，调用方每帧：`tick` 衰计时 → `want_jump` 存缓冲 →
//! `consume_jump(grounded)` 取跳跃 → 按需 `cut_jump` / `dash`。
//! 单测无头可跑；`games/neotrix-swords` 首个消费者。

/// 调参表（数值即数据，改数不改码；默认按 60Hz 手感标定）
#[derive(Debug, Clone, Copy)]
pub struct MoveTune {
    /// 土狼窗口（s）
    pub coyote_time: f32,
    /// 预输入窗口（s）
    pub buffer_time: f32,
    /// 总跳段数（含地面一段；2 = 二段跳）
    pub max_jumps: u32,
    /// 起跳初速（px/s，上为负由调用方取反——此处返回标量速度）
    pub jump_speed: f32,
    /// 松开截断系数（上升速度 × 该值）
    pub cut_mult: f32,
    /// 冲刺速度（px/s）
    pub dash_speed: f32,
    /// 冲刺持续（s）
    pub dash_time: f32,
    /// 冲刺冷却（s）
    pub dash_cool: f32,
}

impl MoveTune {
    pub fn platformer() -> Self {
        Self {
            coyote_time: 0.10,
            buffer_time: 0.12,
            max_jumps: 2,
            jump_speed: 520.0,
            cut_mult: 0.45,
            dash_speed: 620.0,
            dash_time: 0.14,
            dash_cool: 0.55,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpKind {
    Ground,
    Air,
}

#[derive(Debug, Clone)]
pub struct MoveState {
    coyote_left: f32,
    buffer_left: f32,
    jumps_used: u32,
    dash_cd_left: f32,
    dash_left: f32,
    drop_left: f32,
}

impl MoveState {
    pub fn new() -> Self {
        Self {
            coyote_left: 0.0,
            buffer_left: 0.0,
            jumps_used: 0,
            dash_cd_left: 0.0,
            dash_left: 0.0,
            drop_left: 0.0,
        }
    }

    /// 落地调用：重置土狼与段数
    pub fn land(&mut self, tune: &MoveTune) {
        self.coyote_left = tune.coyote_time;
        self.jumps_used = 0;
    }

    /// 离崖调用（不再接地且非跳跃起跳）：只刷土狼，不清段数
    pub fn walk_off(&mut self, tune: &MoveTune) {
        self.coyote_left = tune.coyote_time;
    }

    /// 每帧衰减计时
    pub fn tick(&mut self, dt: f32) {
        self.coyote_left = (self.coyote_left - dt).max(0.0);
        self.buffer_left = (self.buffer_left - dt).max(0.0);
        self.dash_cd_left = (self.dash_cd_left - dt).max(0.0);
        self.dash_left = (self.dash_left - dt).max(0.0);
        self.drop_left = (self.drop_left - dt).max(0.0);
    }

    /// 玩家按下跳跃：存入缓冲（消费在 consume_jump）
    pub fn want_jump(&mut self, tune: &MoveTune) {
        self.buffer_left = tune.buffer_time;
    }

    /// 尝试起跳：地面段（在地或土狼内）优先，否则空中段；返回跳跃种类
    pub fn consume_jump(&mut self, tune: &MoveTune, grounded: bool) -> Option<JumpKind> {
        if self.buffer_left <= 0.0 {
            return None;
        }
        if grounded || self.coyote_left > 0.0 {
            self.buffer_left = 0.0;
            self.coyote_left = 0.0;
            self.jumps_used = 1;
            return Some(JumpKind::Ground);
        }
        if self.jumps_used < tune.max_jumps {
            self.buffer_left = 0.0;
            self.jumps_used += 1;
            return Some(JumpKind::Air);
        }
        None
    }

    /// 可变跳高：松开跳跃键且仍在上升 → 截断
    pub fn cut_jump(vy: f32, tune: &MoveTune) -> f32 {
        if vy < 0.0 {
            vy * tune.cut_mult
        } else {
            vy
        }
    }

    /// 冲刺：cd 就绪则触发，返回持续时间（调用方在此期间覆写速度）
    pub fn dash(&mut self, tune: &MoveTune) -> Option<f32> {
        if self.dash_cd_left > 0.0 {
            return None;
        }
        self.dash_cd_left = tune.dash_cool;
        self.dash_left = tune.dash_time;
        Some(tune.dash_time)
    }

    /// 冲刺中
    pub fn dashing(&self) -> bool {
        self.dash_left > 0.0
    }

    /// 跳穿下降：下 0.25s 忽略顶面（Celeste descent；下+跳触发）
    pub fn drop(&mut self) {
        self.drop_left = 0.25;
    }

    /// 下降穿行中（调用方透传给 move_on_platforms 的 drop 参数）
    pub fn dropping(&self) -> bool {
        self.drop_left > 0.0
    }
}

impl Default for MoveState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tune() -> MoveTune {
        MoveTune::platformer()
    }

    #[test]
    fn ground_jump_consumes_buffer() {
        let mut s = MoveState::new();
        let t = tune();
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, true), Some(JumpKind::Ground));
        assert_eq!(s.consume_jump(&t, true), None); // 缓冲已消费
    }

    #[test]
    fn coyote_grace_after_walk_off() {
        let mut s = MoveState::new();
        let t = tune();
        s.land(&t);
        s.walk_off(&t);
        s.tick(0.05); // 土狼窗内
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, false), Some(JumpKind::Ground));
    }

    #[test]
    fn coyote_expired_falls_to_air_jump() {
        let mut s = MoveState::new();
        let t = tune();
        s.land(&t);
        s.walk_off(&t);
        s.tick(0.5); // 土狼过期
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, false), Some(JumpKind::Air)); // 二段跳接住
    }

    #[test]
    fn double_jump_only_once() {
        let mut s = MoveState::new();
        let t = tune();
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, true), Some(JumpKind::Ground));
        s.tick(0.2);
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, false), Some(JumpKind::Air));
        s.tick(0.2);
        s.want_jump(&t);
        assert_eq!(s.consume_jump(&t, false), None); // 三段无门
    }

    #[test]
    fn buffer_fires_late_landing() {
        // 空中提前按，落地瞬间（3 帧内）仍触发——手感核心
        let mut s = MoveState::new();
        let t = tune();
        s.want_jump(&t);
        s.tick(0.05);
        s.tick(0.05);
        s.land(&t);
        assert_eq!(s.consume_jump(&t, true), Some(JumpKind::Ground));
    }

    #[test]
    fn cut_only_when_rising() {
        let t = tune();
        assert!((MoveState::cut_jump(-500.0, &t) + 225.0).abs() < 1e-3);
        assert_eq!(MoveState::cut_jump(100.0, &t), 100.0); // 下落不截
    }

    #[test]
    fn dash_cooldown_gates() {
        let mut s = MoveState::new();
        let t = tune();
        assert!(s.dash(&t).is_some());
        assert!(s.dashing());
        assert!(s.dash(&t).is_none()); // cd 中
        s.tick(1.0);
        assert!(!s.dashing());
        assert!(s.dash(&t).is_some()); // cd 好
    }

    #[test]
    fn drop_window_opens_and_closes() {
        let mut s = MoveState::new();
        assert!(!s.dropping());
        s.drop();
        assert!(s.dropping());
        s.tick(0.1);
        assert!(s.dropping());
        s.tick(0.2);
        assert!(!s.dropping()); // 0.25s 窗关闭
    }
}
