//! L4 进化层 — 实时变化: 生长周期、能力评分、适应记录

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 生长阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrowthPhase {
    /// 编译通过
    Compile,
    /// 单元测试通过
    UnitTest,
    /// 集成测试通过
    IntegrationTest,
    /// 基准测试通过
    Benchmark,
    /// 主流水线通过
    Mainline,
    /// 自愈/自适应
    SelfHealing,
    /// 完全自主
    Autonomous,
}

impl Default for GrowthPhase {
    fn default() -> Self {
        Self::Compile
    }
}

impl GrowthPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compile => "C0-Compile",
            Self::UnitTest => "C1-UnitTest",
            Self::IntegrationTest => "C2-Integration",
            Self::Benchmark => "C3-Benchmark",
            Self::Mainline => "C4-Mainline",
            Self::SelfHealing => "C5-SelfHealing",
            Self::Autonomous => "C6-Autonomous",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Compile => Self::UnitTest,
            Self::UnitTest => Self::IntegrationTest,
            Self::IntegrationTest => Self::Benchmark,
            Self::Benchmark => Self::Mainline,
            Self::Mainline => Self::SelfHealing,
            Self::SelfHealing => Self::Autonomous,
            Self::Autonomous => Self::Autonomous,
        }
    }

    pub fn index(&self) -> u8 {
        match self {
            Self::Compile => 0,
            Self::UnitTest => 1,
            Self::IntegrationTest => 2,
            Self::Benchmark => 3,
            Self::Mainline => 4,
            Self::SelfHealing => 5,
            Self::Autonomous => 6,
        }
    }
}

/// 单次生长周期记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthCycle {
    pub cycle_id: u64,
    pub phase: GrowthPhase,
    pub changes: Vec<String>,
    pub score_before: f64,
    pub score_after: f64,
    pub duration_ms: u64,
    pub timestamp: String,
}

/// 能力评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScores {
    pub scores: HashMap<String, f64>,
}

impl CapabilityScores {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        scores.insert("consciousness".into(), 0.3);
        scores.insert("emotion".into(), 0.2);
        scores.insert("creativity".into(), 0.1);
        scores.insert("reasoning".into(), 0.3);
        scores.insert("memory".into(), 0.4);
        scores.insert("safety".into(), 0.8);
        scores.insert("evolution".into(), 0.1);
        Self { scores }
    }

    pub fn get(&self, key: &str) -> f64 {
        self.scores.get(key).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, key: impl Into<String>, value: f64) {
        self.scores.insert(key.into(), value.clamp(0.0, 1.0));
    }

    /// 总体评分 (加权平均)
    pub fn overall(&self) -> f64 {
        if self.scores.is_empty() {
            return 0.0;
        }
        let weights: HashMap<&str, f64> = [
            ("consciousness", 0.2),
            ("emotion", 0.1),
            ("creativity", 0.1),
            ("reasoning", 0.2),
            ("memory", 0.1),
            ("safety", 0.2),
            ("evolution", 0.1),
        ]
        .into_iter()
        .collect();

        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for (k, v) in &self.scores {
            let w = weights.get(k.as_str()).unwrap_or(&0.1);
            weighted_sum += v * w;
            total_weight += w;
        }
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// 识别能力差距 (低于阈值的)
    pub fn gaps(&self, threshold: f64) -> Vec<(String, f64)> {
        self.scores
            .iter()
            .filter(|(_, v)| **v < threshold)
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}

/// 适应记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adaptation {
    pub id: String,
    pub trigger: String,      // 什么触发了适应
    pub change: String,       // 做了什么改变
    pub effect: String,       // 效果如何
    pub score_delta: f64,     // 评分变化
    pub timestamp: String,
}

/// L4 进化层 — 实时变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalEvolution {
    pub growth_cycles: Vec<GrowthCycle>,
    pub capability_scores: CapabilityScores,
    pub adaptations: Vec<Adaptation>,
    pub current_phase: GrowthPhase,
    next_cycle_id: u64,
}

impl CrystalEvolution {
    pub fn new() -> Self {
        Self {
            growth_cycles: Vec::new(),
            capability_scores: CapabilityScores::new(),
            adaptations: Vec::new(),
            current_phase: GrowthPhase::Compile,
            next_cycle_id: 1,
        }
    }

    /// 记录一次生长周期
    pub fn record_cycle(
        &mut self,
        phase: GrowthPhase,
        changes: Vec<String>,
        score_before: f64,
        score_after: f64,
        duration_ms: u64,
    ) {
        let cycle_id = self.next_cycle_id;
        self.next_cycle_id += 1;

        self.growth_cycles.push(GrowthCycle {
            cycle_id,
            phase: phase.clone(),
            changes,
            score_before,
            score_after,
            duration_ms,
            timestamp: timestamp_now(),
        });

        // 自动推进阶段
        if score_after > score_before && score_after > 0.5 {
            self.current_phase = self.current_phase.next();
        }
    }

    /// 记录适应
    pub fn record_adaptation(
        &mut self,
        trigger: impl Into<String>,
        change: impl Into<String>,
        effect: impl Into<String>,
        score_delta: f64,
    ) {
        let id = format!("ADP-{:04}", self.adaptations.len() + 1);
        self.adaptations.push(Adaptation {
            id,
            trigger: trigger.into(),
            change: change.into(),
            effect: effect.into(),
            score_delta,
            timestamp: timestamp_now(),
        });
    }

    /// 更新能力评分
    pub fn update_score(&mut self, key: &str, value: f64) {
        self.capability_scores.set(key, value);
    }

    /// 获取进化趋势 (最近N个周期的评分变化)
    pub fn trend(&self, n: usize) -> Vec<(f64, f64)> {
        self.growth_cycles
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(|c| (c.score_before, c.score_after))
            .collect()
    }
}

fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}
