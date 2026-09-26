//! 对线战斗原语 — 吸收 `everwar` 队伍对抗 + `War3GameFramework` 波次刷怪思想.
//!
//! MOBA/RTS 通用数据侧：队伍归属（蓝/红）/ 出兵时钟（固定间隔波次）/
//! 防御结构（射程/伤害/冷却）/ 范围索敌（最近敌方）. 无渲染无 ECS 依赖，
//! 游戏侧（WoW 召唤师峡谷 / 竞技场）以组件/状态机方式接入生产.
//!
//! 公理：`dt <= 0` 不推进；空候选索敌返回 `None`；结构开火后进入全冷却.

/// 对战队伍（蓝方西南 / 红方东北，180° 对称图通用）.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Blue,
    Red,
}

impl Team {
    /// 对手.
    pub fn enemy(self) -> Self {
        match self {
            Self::Blue => Self::Red,
            Self::Red => Self::Blue,
        }
    }

    /// 按横坐标归属（`x < mid_x` 蓝，否则红；渲染/刷怪/塔分组同源）.
    pub fn of_x(x: f32, mid_x: f32) -> Self {
        if x < mid_x {
            Self::Blue
        } else {
            Self::Red
        }
    }
}

/// 出兵时钟（固定间隔波次；首波立即出 Nokia 3310 式简单）.
///
/// `tick(dt)` 累积并在到期间隔返回 `Some(波次号)`（从 1 起），未到期 `None`.
#[derive(Debug, Clone)]
pub struct WaveClock {
    interval: f32,
    t: f32,
    wave: u32,
}

impl WaveClock {
    /// `interval <= 0` 按每 tick 一波处理（调用方保证正间隔）.
    pub fn new(interval: f32) -> Self {
        Self { interval: interval.max(0.0), t: 0.0, wave: 0 }
    }

    pub fn wave(&self) -> u32 {
        self.wave
    }

    pub fn tick(&mut self, dt: f32) -> Option<u32> {
        if dt <= 0.0 {
            return None;
        }
        // 首波立即（t 从 interval 起算，第一 tick 即到期）.
        self.t += dt;
        if self.t >= self.interval {
            self.t -= self.interval;
            self.wave += 1;
            Some(self.wave)
        } else {
            None
        }
    }
}

/// 防御结构（塔/水晶逻辑侧：射程内开火，冷却约束）.
#[derive(Debug, Clone)]
pub struct Structure {
    pub team: Team,
    pub range: f32,
    pub damage: f32,
    pub cooldown: f32,
    cd: f32,
}

impl Structure {
    pub fn new(team: Team, range: f32, damage: f32, cooldown: f32) -> Self {
        Self { team, range, damage, cooldown: cooldown.max(0.1), cd: 0.0 }
    }

    /// 推进冷却；就绪返回 `true`（调用方索敌命中后 `fire()` 消冷却）.
    pub fn ready(&self) -> bool {
        self.cd <= 0.0
    }

    pub fn tick(&mut self, dt: f32) {
        if dt > 0.0 {
            self.cd = (self.cd - dt).max(0.0);
        }
    }

    pub fn fire(&mut self) {
        self.cd = self.cooldown;
    }
}

/// 范围索敌：候选 `(id, x, y, team)` 中找离 `(fx, fy)` 最近的敌方（`range` 内）.
///
/// 空候选/全友军/全超距 → `None`；平局取首个（调用方传入顺序需确定性）.
pub fn acquire_target(
    from: (f32, f32),
    team: Team,
    range: f32,
    candidates: &[(u64, f32, f32, Team)],
) -> Option<u64> {
    let mut best: Option<(u64, f32)> = None;
    for (id, x, y, t) in candidates {
        if *t == team {
            continue;
        }
        let d = ((from.0 - x).powi(2) + (from.1 - y).powi(2)).sqrt();
        if d <= range && best.map_or(true, |(_, bd)| d < bd) {
            best = Some((*id, d));
        }
    }
    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_enemy_and_of_x() {
        assert_eq!(Team::Blue.enemy(), Team::Red);
        assert_eq!(Team::Red.enemy(), Team::Blue);
        assert_eq!(Team::of_x(10.0, 32.0), Team::Blue);
        assert_eq!(Team::of_x(50.0, 32.0), Team::Red);
        assert_eq!(Team::of_x(32.0, 32.0), Team::Red); // 线上归红，保证互斥完备
    }

    #[test]
    fn wave_clock_first_tick_fires() {
        let mut w = WaveClock::new(25.0);
        assert_eq!(w.wave(), 0);
        assert_eq!(w.tick(25.0), Some(1));
        assert_eq!(w.tick(10.0), None);
        assert_eq!(w.tick(15.0), Some(2));
    }

    #[test]
    fn wave_clock_non_positive_dt_never_fires() {
        let mut w = WaveClock::new(5.0);
        assert_eq!(w.tick(0.0), None);
        assert_eq!(w.tick(-1.0), None);
        assert_eq!(w.wave(), 0);
    }

    #[test]
    fn structure_cooldown_gates_fire() {
        let mut s = Structure::new(Team::Blue, 320.0, 40.0, 2.0);
        assert!(s.ready());
        s.fire();
        assert!(!s.ready());
        s.tick(1.0);
        assert!(!s.ready());
        s.tick(1.0);
        assert!(s.ready());
    }

    #[test]
    fn acquire_prefers_nearest_enemy_in_range() {
        let cands = vec![
            (1u64, 100.0, 0.0, Team::Red),
            (2u64, 50.0, 0.0, Team::Red),
            (3u64, 10.0, 0.0, Team::Blue), // 友军跳过
            (4u64, 1000.0, 0.0, Team::Red), // 超距跳过
        ];
        assert_eq!(acquire_target((0.0, 0.0), Team::Blue, 200.0, &cands), Some(2));
    }

    #[test]
    fn acquire_none_when_no_enemy() {
        assert_eq!(acquire_target((0.0, 0.0), Team::Blue, 200.0, &[]), None);
        let friends = vec![(1u64, 10.0, 0.0, Team::Blue)];
        assert_eq!(acquire_target((0.0, 0.0), Team::Blue, 200.0, &friends), None);
    }
}
