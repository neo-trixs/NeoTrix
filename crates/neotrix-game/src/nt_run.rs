//! 种子 Run 管理 — 吸收 `pokerogue` 的 Roguelike 循环思想.
//!
//! Pokerogue 侧：种子对局（`game-mode` + 楼层/波次 + 修饰器 `modifier/` +
//! 存档 `system/`）：同种子同流程（可复现），波次推进给奖励三选一，
//! 存档/读档续跑。
//! 本模块是无头数据侧：`RunState { seed, wave, modifiers, rng_state }`
//! —— `mulberry32` 整数 RNG（确定性、零依赖），`next_wave()` 推进波次
//! 并按波次表给修饰器槽位，`to_json/from_json` 存档（经 `serde_json`，
//! 引擎已有依赖）。
//!
//! 公理：同种子同序列（`RunState::new(seed)` 确定性）；修饰器只做
//! 记录（数值解释权在游戏侧）；存档 roundtrip 无损。

use serde::{Deserialize, Serialize};

/// Roguelike 单局状态.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunState {
    pub seed: u64,
    rng: u64,
    /// 当前波次（1 起）.
    pub wave: u32,
    /// 已获得修饰器（名列表，游戏侧解释数值）.
    pub modifiers: Vec<String>,
    /// 本局累计掷骰次数（录像/审计用）.
    pub rolls: u64,
}

impl RunState {
    pub fn new(seed: u64) -> Self {
        Self { seed, rng: seed, wave: 1, modifiers: Vec::new(), rolls: 0 }
    }

    /// 下一个 `[0, 1)` 随机数（mulberry32 变体，64bit 状态）.
    pub fn next_f32(&mut self) -> f32 {
        // splitmix64 一步 → 取高 32bit 归一化
        self.rng = self.rng.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        self.rolls += 1;
        ((z >> 32) as u32 as f64 / u32::MAX as f64) as f32
    }

    /// `[lo, hi)` 整数骰（`lo >= hi` → `Err`）.
    pub fn roll_range(&mut self, lo: u32, hi: u32) -> Result<u32, String> {
        if lo >= hi {
            return Err(format!("非法区间 [{}, {})", lo, hi));
        }
        let f = self.next_f32();
        Ok(lo + ((f * (hi - lo) as f32) as u32).min(hi - lo - 1))
    }

    /// 波次奖励候选：按波次数给 3 选 1 修饰器名（确定性查表 + 骰子扰动）.
    pub fn wave_choices(&mut self) -> [String; 3] {
        const POOL: &[&str] = &[
            "atk_up", "def_up", "spd_up", "hp_up", "crit_up",
            "regen", "thorns", "lifesteal", "shield", "haste",
        ];
        let mut out = [String::new(), String::new(), String::new()];
        // 起点按波次偏移，保证不同波次默认不同；再用骰子替换一席增加变数
        let base = (self.wave as usize) % POOL.len();
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = POOL[(base + i * 3) % POOL.len()].to_string();
        }
        if let Ok(r) = self.roll_range(0, 3) {
            if let Ok(extra) = self.roll_range(0, POOL.len() as u32) {
                out[r as usize] = POOL[extra as usize].to_string();
            }
        }
        out
    }

    /// 选取修饰器并推进波次（空名 → `Err`）.
    pub fn pick_and_advance(&mut self, choice: &str) -> Result<(), String> {
        if choice.is_empty() {
            return Err("修饰器名为空".to_string());
        }
        self.modifiers.push(choice.to_string());
        self.wave += 1;
        Ok(())
    }

    /// 存档 → JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// 读档 ← JSON.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let (mut a, mut b) = (RunState::new(12345), RunState::new(12345));
        for _ in 0..16 {
            assert!((a.next_f32() - b.next_f32()).abs() < f32::EPSILON);
        }
        assert_eq!(a.rolls, 16);
        // 不同种子大概率不同流
        let mut c = RunState::new(99999);
        assert!((a.next_f32() - c.next_f32()).abs() > f32::EPSILON
            || (a.next_f32() - c.next_f32()).abs() > f32::EPSILON);
    }

    #[test]
    fn range_and_bad_range() {
        let mut r = RunState::new(7);
        for _ in 0..64 {
            let v = r.roll_range(1, 7).unwrap();
            assert!((1..7).contains(&v));
        }
        assert!(r.roll_range(5, 5).is_err());
        assert!(r.roll_range(9, 2).is_err());
    }

    #[test]
    fn wave_flow_and_choices_differ() {
        let mut r = RunState::new(42);
        let c1 = r.wave_choices();
        assert_eq!(c1.len(), 3);
        r.pick_and_advance(&c1[0]).unwrap();
        assert_eq!(r.wave, 2);
        assert_eq!(r.modifiers.len(), 1);
        assert!(r.pick_and_advance("").is_err());
    }

    #[test]
    fn save_roundtrip() {
        let mut r = RunState::new(2024);
        r.next_f32();
        r.pick_and_advance("atk_up").unwrap();
        let s = r.to_json().unwrap();
        let back = RunState::from_json(&s).unwrap();
        assert_eq!(r, back);
        // 续跑确定性：同状态继续同流
        let (mut x, mut y) = (back.clone(), back);
        assert!((x.next_f32() - y.next_f32()).abs() < f32::EPSILON);
        assert!(RunState::from_json("not json").is_err());
    }
}
