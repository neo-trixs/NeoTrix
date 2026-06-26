/// DecisionChain — 进化决策上下文追踪
///
/// 每个自我修改提案（MutationOp / SelfEvolutionStep）执行时，
/// 记录其上下文：当时的指标状态、备选方案、选择理由、预期收益。
/// 这是意识体"知道自己为什么做了那个决定"的结构化记忆。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// 决策上下文快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// 当时的 ECE
    pub ece: f64,
    /// 当时的元精度
    pub meta_accuracy: f64,
    /// 当时的复合损失
    pub composite_loss: f64,
    /// 当时的 arousal
    pub arousal: f64,
    /// 正在执行的 cycle
    pub cycle: u64,
}

/// 一个决策条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEntry {
    /// 决策唯一 ID
    pub id: u64,
    /// 提案名称（"G348 MCTS wiring"）
    pub proposal_name: String,
    /// 提案类型（"ModuleWiring", "TuneMutation" 等）
    pub proposal_type: String,
    /// 执行前的指标快照
    pub context_before: DecisionContext,
    /// 备选方案（其他被考虑的方案名称）
    pub alternatives: Vec<String>,
    /// 选择本方案的理由
    pub rationale: String,
    /// 预期收益描述
    pub expected_gain: String,
    /// 执行后的指标快照
    pub context_after: Option<DecisionContext>,
    /// 实际收益（delta）
    pub actual_delta: Option<f64>,
    /// 是否成功
    pub success: bool,
    /// 创建 cycle
    pub created_at: u64,
}

/// 决策链追踪器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionChain {
    /// 所有决策记录（最近 200 条）
    pub entries: VecDeque<DecisionEntry>,
    next_id: u64,
}

impl DecisionChain {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(200),
            next_id: 1,
        }
    }

    /// 开始一个决策（执行前调用）。
    pub fn begin_decision(
        &mut self,
        proposal_name: String,
        proposal_type: String,
        context: DecisionContext,
        alternatives: Vec<String>,
        rationale: String,
        expected_gain: String,
    ) -> u64 {
        let id = self.next_id;
        let cycle = context.cycle;
        self.next_id += 1;
        if self.entries.len() >= 200 {
            self.entries.pop_front();
        }
        self.entries.push_back(DecisionEntry {
            id,
            proposal_name,
            proposal_type,
            context_before: context,
            alternatives,
            rationale,
            expected_gain,
            context_after: None,
            actual_delta: None,
            success: false,
            created_at: cycle,
        });
        id
    }

    /// 完成决策（执行后调用）。
    pub fn complete_decision(
        &mut self,
        id: u64,
        context_after: DecisionContext,
        delta: f64,
        success: bool,
    ) {
        if let Some(entry) = self.entries.iter_mut().rev().find(|e| e.id == id) {
            entry.context_after = Some(context_after);
            entry.actual_delta = Some(delta);
            entry.success = success;
        }
    }

    /// 获取最近的决策（按成功/失败筛选）。
    pub fn recent_decisions(&self, n: usize, only_successful: bool) -> Vec<&DecisionEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| !only_successful || e.success)
            .take(n)
            .collect()
    }

    /// 计算近期成功率。
    pub fn recent_success_rate(&self, n: usize) -> f64 {
        let recent: Vec<_> = self.entries.iter().rev().take(n).collect();
        if recent.is_empty() {
            return 0.0;
        }
        let success_count = recent.iter().filter(|e| e.success).count();
        success_count as f64 / recent.len() as f64
    }

    /// 获取总统计。
    pub fn stats(&self) -> String {
        let total = self.entries.len();
        let successful = self.entries.iter().filter(|e| e.success).count();
        format!(
            "decision_chain: {} total, {} success ({:.1}%), {} recent",
            total,
            successful,
            if total > 0 { successful as f64 / total as f64 * 100.0 } else { 0.0 },
            self.entries.len().min(200),
        )
    }

    /// 获取总条目数。
    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    /// 获取成功条目数。
    pub fn success_count(&self) -> usize {
        self.entries.iter().filter(|e| e.success).count()
    }
}

impl Default for DecisionChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context(cycle: u64) -> DecisionContext {
        DecisionContext {
            ece: 0.15,
            meta_accuracy: 0.7,
            composite_loss: 0.4,
            arousal: 0.5,
            cycle,
        }
    }

    #[test]
    fn test_begin_and_complete_decision() {
        let mut chain = DecisionChain::new();
        let id = chain.begin_decision(
            "test_proposal".into(),
            "ModuleWiring".into(),
            sample_context(1),
            vec!["alt_a".into(), "alt_b".into()],
            "this is the best option".into(),
            "reduce ECE by 0.02".into(),
        );
        assert!(id > 0);
        assert_eq!(chain.entries.len(), 1);

        chain.complete_decision(id, sample_context(10), -0.02, true);
        let entry = chain.entries.back().unwrap();
        assert!(entry.success);
        assert_eq!(entry.actual_delta, Some(-0.02));
    }

    #[test]
    fn test_recent_success_rate() {
        let mut chain = DecisionChain::new();
        for i in 0..5 {
            let id = chain.begin_decision(
                format!("p_{}", i), "test".into(), sample_context(i),
                vec![], "ok".into(), "gain".into(),
            );
            chain.complete_decision(id, sample_context(i + 1), 0.1, i % 2 == 0);
        }
        let rate = chain.recent_success_rate(5);
        assert!((rate - 0.6).abs() < 1e-6); // 3/5 = 0.6
    }

    #[test]
    fn test_capacity_bound() {
        let mut chain = DecisionChain::new();
        for i in 0..210 {
            let id = chain.begin_decision(
                format!("p_{}", i), "test".into(), sample_context(i as u64),
                vec![], "ok".into(), "gain".into(),
            );
            chain.complete_decision(id, sample_context(i as u64 + 1), 0.0, true);
        }
        assert_eq!(chain.entries.len(), 200);
    }

    #[test]
    fn test_recent_decisions_filter() {
        let mut chain = DecisionChain::new();
        for i in 0..10 {
            let id = chain.begin_decision(
                format!("p_{}", i), "test".into(), sample_context(i as u64),
                vec![], "".into(), "".into(),
            );
            chain.complete_decision(id, sample_context(i as u64 + 1), 0.0, i < 5);
        }
        let successful = chain.recent_decisions(10, true);
        assert_eq!(successful.len(), 5);
    }

    #[test]
    fn test_stats() {
        let chain = DecisionChain::new();
        assert!(chain.stats().contains("0 total"));
    }
}
