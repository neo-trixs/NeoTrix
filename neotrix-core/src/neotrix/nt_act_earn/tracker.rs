use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单条收益记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EarningsRecord {
    pub date: String,
    pub platform: String,
    pub amount: f64,
    pub currency: String,
    pub content_title: String,
}

/// RL 奖励信号
#[derive(Clone, Debug)]
pub struct RewardSignal {
    pub value: f64,
    pub source: String,
}

/// 收益统计
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EarnStats {
    pub total_earnings: f64,
    pub daily_earnings: Vec<(String, f64)>,
    pub platform_breakdown: HashMap<String, f64>,
    pub best_platform: String,
    pub avg_daily_earnings: f64,
    pub records: Vec<EarningsRecord>,
}

impl Default for EarnStats {
    fn default() -> Self {
        Self {
            total_earnings: 0.0,
            daily_earnings: vec![],
            platform_breakdown: HashMap::new(),
            best_platform: "none".to_string(),
            avg_daily_earnings: 0.0,
            records: vec![],
        }
    }
}

/// 收益追踪器
pub struct EarnTracker {
    stats: EarnStats,
    reward_history: Vec<RewardSignal>,
}

impl Default for EarnTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EarnTracker {
    pub fn new() -> Self {
        Self {
            stats: EarnStats::default(),
            reward_history: Vec::new(),
        }
    }

    /// 记录一笔收益
    pub fn record_earning(&mut self, record: EarningsRecord) {
        self.stats.total_earnings += record.amount;
        self.stats.records.push(record.clone());

        let entry = self
            .stats
            .platform_breakdown
            .entry(record.platform.clone())
            .or_insert(0.0);
        *entry += record.amount;

        self.update_best_platform();
        self.recalc_average();
    }

    /// 批量记录收益
    pub fn record_batch(&mut self, records: Vec<EarningsRecord>) {
        for r in records {
            self.record_earning(r);
        }
    }

    /// 生成 RL 奖励信号
    pub fn compute_reward(&mut self, recent_delta: f64) -> RewardSignal {
        let value = recent_delta.clamp(-1.0, 1.0) * 0.5 + 0.5;
        let signal = RewardSignal {
            value: value.max(0.0),
            source: "nt_act_earn".to_string(),
        };
        self.reward_history.push(signal.clone());
        signal
    }

    /// 获取可用于 RL 反馈的平均奖励
    pub fn smoothed_reward(&self, window: usize) -> f64 {
        let n = self.reward_history.len().min(window);
        if n == 0 {
            return 0.5;
        }
        self.reward_history
            .iter()
            .rev()
            .take(n)
            .map(|r| r.value)
            .sum::<f64>()
            / n as f64
    }

    pub fn stats(&self) -> &EarnStats {
        &self.stats
    }

    fn update_best_platform(&mut self) {
        let best = self
            .stats
            .platform_breakdown
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| "none".to_string());
        self.stats.best_platform = best;
    }

    fn recalc_average(&mut self) {
        let days = self.stats.daily_earnings.len().max(1);
        self.stats.avg_daily_earnings = self.stats.total_earnings / days as f64;
    }

    /// 从已持久化的统计重建追踪器
    pub fn from_stats(stats: EarnStats) -> Self {
        Self {
            stats,
            reward_history: Vec::new(),
        }
    }
}
