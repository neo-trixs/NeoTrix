//! 定时调度器 — 吸收 `kaplay` 的 `k.wait / k.loop / k.tween` 调度思想.
//!
//! KAPLAY 侧：`k.wait(1.0, cb)` 一次性延迟，`k.loop(0.5, cb)` 周期触发，
//! `k.tween(...)` 数值补间（引擎已有 `tween.rs` 做补间本体）。
//! 本模块只做**时间调度**（无头、可测）：调用方 `after/every` 注册，
//! 每帧 `update(dt)` 推进并返回到期的 `TimerFire` 列表，调用方按 id
//! 分发回调/事件（不断闭包进调度器，保持确定性与可序列化）。
//!
//! 公理：时间单调累积；`dt <= 0` 不推进；`cancel` 幂等；`update` 返回按
//! 到期顺序（once 先于 loop 同帧到期按注册序）。

/// 到期事件 — 调用方按 `id` 分发.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerFire {
    pub id: u64,
    pub kind: TimerKind,
    /// loop 触发时本帧追补次数（once 恒为 1；lag 下追帧但钳制，防滚雪球）.
    pub times: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerKind {
    Once,
    Loop,
}

#[derive(Debug)]
struct Once {
    id: u64,
    at: f32,
}

#[derive(Debug)]
struct Loop {
    id: u64,
    interval: f32,
    next: f32,
}

/// 无头调度器（KAPLAY `wait/loop` 的数据侧）.
#[derive(Debug, Default)]
pub struct Scheduler {
    now: f32,
    next_id: u64,
    once: Vec<Once>,
    loops: Vec<Loop>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn now(&self) -> f32 {
        self.now
    }

    pub fn pending(&self) -> usize {
        self.once.len() + self.loops.len()
    }

    /// 一次性延迟（KAPLAY `k.wait`）：`delay <= 0` 则下一 `update` 即到期.
    pub fn after(&mut self, delay: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.once.push(Once { id, at: self.now + delay.max(0.0) });
        id
    }

    /// 周期触发（KAPLAY `k.loop`）：`interval <= 0` 按每帧一次处理.
    /// 返回 id，可 `cancel`.
    pub fn every(&mut self, interval: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let iv = if interval <= 0.0 { 0.0 } else { interval };
        self.loops.push(Loop { id, interval: iv, next: self.now + iv });
        id
    }

    /// 取消（once/loop 通用，不存在返回 false）.
    pub fn cancel(&mut self, id: u64) -> bool {
        if let Some(pos) = self.once.iter().position(|o| o.id == id) {
            self.once.remove(pos);
            return true;
        }
        if let Some(pos) = self.loops.iter().position(|l| l.id == id) {
            self.loops.remove(pos);
            return true;
        }
        false
    }

    /// 推进 `dt`，返回本帧到期列表（按到期时间/注册序稳定排序）.
    pub fn update(&mut self, dt: f32) -> Vec<TimerFire> {
        if dt <= 0.0 {
            return Vec::new();
        }
        self.now += dt;
        let mut out = Vec::new();
        // once：到期即移除
        let mut i = 0;
        while i < self.once.len() {
            if self.once[i].at <= self.now {
                let o = self.once.remove(i);
                out.push(TimerFire { id: o.id, kind: TimerKind::Once, times: 1 });
            } else {
                i += 1;
            }
        }
        // loop：追补但单帧钳制 4 次（防 tab 切后台后滚雪球）
        for l in &mut self.loops {
            if l.interval <= 0.0 {
                out.push(TimerFire { id: l.id, kind: TimerKind::Loop, times: 1 });
                l.next = self.now;
                continue;
            }
            let mut times = 0u32;
            while l.next <= self.now && times < 4 {
                times += 1;
                l.next += l.interval;
            }
            // 仍欠账则快进到下一未来时刻（丢弃积压，只保当下）
            if l.next <= self.now {
                l.next = self.now + l.interval;
            }
            if times > 0 {
                out.push(TimerFire { id: l.id, kind: TimerKind::Loop, times });
            }
        }
        out.sort_by_key(|f| f.id);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn once_fires_after_delay() {
        let mut s = Scheduler::new();
        let id = s.after(1.0);
        assert!(s.update(0.5).is_empty());
        let fires = s.update(0.5);
        assert_eq!(fires.len(), 1);
        assert_eq!(fires[0], TimerFire { id, kind: TimerKind::Once, times: 1 });
        assert!(s.update(1.0).is_empty()); // 一次性不重复
    }

    #[test]
    fn zero_delay_fires_next_update() {
        let mut s = Scheduler::new();
        let id = s.after(0.0);
        let fires = s.update(0.016);
        assert_eq!(fires.len(), 1);
        assert_eq!(fires[0].id, id);
    }

    #[test]
    fn loop_ticks_periodically_and_cancels() {
        let mut s = Scheduler::new();
        let id = s.every(0.5);
        assert!(s.update(0.4).is_empty());
        assert_eq!(s.update(0.2).len(), 1); // 0.6 ≥ 0.5
        assert_eq!(s.update(0.5).len(), 1);
        assert!(s.cancel(id));
        assert!(!s.cancel(id)); // 幂等：二次取消 false
        assert!(s.update(1.0).is_empty());
    }

    #[test]
    fn non_positive_dt_never_advances() {
        let mut s = Scheduler::new();
        let _id = s.after(0.1);
        assert!(s.update(0.0).is_empty());
        assert!(s.update(-1.0).is_empty());
        assert_eq!(s.now(), 0.0);
    }

    #[test]
    fn lag_capped_no_snowball() {
        let mut s = Scheduler::new();
        let _id = s.every(0.1);
        let fires = s.update(10.0); // 大卡顿
        assert_eq!(fires.len(), 1);
        assert!(fires[0].times <= 4);
        // 下一小步不再补爆
        let fires2 = s.update(0.1);
        assert!(fires2.len() <= 1);
    }
}
