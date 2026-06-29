#![allow(dead_code)]

//! 结果追踪器 — 基于量规的执行结果评估系统。
//!
//! 受 Claude Code Managed Agents "Outcomes" 功能启发，在任务执行前定义
//! 成功标准（量规），执行后对照评估结果，将差值反馈到进化循环中。
//! 每个量规包含多个带权重的评判标准，综合评分后与通过阈值比较。

use std::collections::{HashMap, VecDeque};

/// 单个量规评判标准。
#[derive(Debug, Clone)]
pub struct RubricCriterion {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub weight: f64,
    pub pass_threshold: f64,
}

/// 量规集合 — 一组带权重的评判标准，定义一次执行的成功基准。
#[derive(Debug, Clone)]
pub struct RubricSet {
    pub id: u64,
    pub name: String,
    pub criteria: Vec<RubricCriterion>,
    pub created_at: u64,
    pub version: u32,
}

/// 单次评估结果。
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub rubric_set_id: u64,
    pub criterion_scores: HashMap<u64, f64>,
    pub overall_score: f64,
    pub evidence: Vec<String>,
    pub evaluated_at: u64,
}

/// 结果追踪器的配置。
#[derive(Debug, Clone)]
pub struct OutcomeTrackerConfig {
    pub max_rubric_sets: usize,
    pub default_pass_threshold: f64,
}

impl Default for OutcomeTrackerConfig {
    fn default() -> Self {
        Self {
            max_rubric_sets: 50,
            default_pass_threshold: 0.7,
        }
    }
}

/// 结果追踪器 — 管理量规定义、执行评估、跟踪历史。
pub struct OutcomeTracker {
    rubric_sets: HashMap<u64, RubricSet>,
    evaluation_history: VecDeque<EvaluationResult>,
    config: OutcomeTrackerConfig,
    next_id: u64,
}

impl OutcomeTracker {
    pub fn new() -> Self {
        Self {
            rubric_sets: HashMap::new(),
            evaluation_history: VecDeque::new(),
            config: OutcomeTrackerConfig::default(),
            next_id: 1,
        }
    }

    pub fn with_config(config: OutcomeTrackerConfig) -> Self {
        Self {
            rubric_sets: HashMap::new(),
            evaluation_history: VecDeque::new(),
            config,
            next_id: 1,
        }
    }

    /// 创建新的量规集合，自动分配 id。返回该 id。
    pub fn create_rubric_set(&mut self, name: &str, criteria: Vec<RubricCriterion>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        if self.rubric_sets.len() >= self.config.max_rubric_sets {
            return 0;
        }

        let set = RubricSet {
            id,
            name: name.to_string(),
            criteria,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            version: 1,
        };
        self.rubric_sets.insert(id, set);
        id
    }

    pub fn get_rubric_set(&self, id: u64) -> Option<&RubricSet> {
        self.rubric_sets.get(&id)
    }

    pub fn list_rubric_sets(&self) -> Vec<&RubricSet> {
        self.rubric_sets.values().collect()
    }

    /// 对指定量规集合执行评估。
    ///
    /// 计算加权 overall_score，检查是否达到集合整体的 pass_threshold
    /// （若未单独设置，取第一个标准的 pass_threshold）。
    pub fn evaluate(
        &mut self,
        rubric_set_id: u64,
        scores: HashMap<u64, f64>,
        evidence: Vec<String>,
    ) -> Option<EvaluationResult> {
        let set = self.rubric_sets.get(&rubric_set_id)?;

        let total_weight: f64 = set.criteria.iter().map(|c| c.weight).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut weighted_sum = 0.0;
        let mut applied_weight = 0.0;

        for criterion in &set.criteria {
            if let Some(&score) = scores.get(&criterion.id) {
                weighted_sum += score * criterion.weight;
                applied_weight += criterion.weight;
            }
        }

        if applied_weight <= 0.0 {
            return None;
        }

        let overall_score = weighted_sum / applied_weight;
        let result = EvaluationResult {
            rubric_set_id,
            criterion_scores: scores,
            overall_score,
            evidence,
            evaluated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.evaluation_history.push_back(result.clone());
        Some(result)
    }

    pub fn evaluation_history(&self) -> &VecDeque<EvaluationResult> {
        &self.evaluation_history
    }

    /// 通过率 — 达到 pass_threshold 的评估占比。
    pub fn success_rate(&self) -> f64 {
        if self.evaluation_history.is_empty() {
            return 0.0;
        }
        let passed = self
            .evaluation_history
            .iter()
            .filter(|r| {
                let set = self.rubric_sets.get(&r.rubric_set_id);
        let _threshold = set
                    .and_then(|s| s.criteria.first())
                    .map(|c| c.pass_threshold)
                    .unwrap_or(self.config.default_pass_threshold);
                r.overall_score >= _threshold
            })
            .count();
        passed as f64 / self.evaluation_history.len() as f64
    }

    /// 指定量规集合的平均得分。
    pub fn avg_score(&self, rubric_set_id: u64) -> f64 {
        let scores: Vec<f64> = self
            .evaluation_history
            .iter()
            .filter(|r| r.rubric_set_id == rubric_set_id)
            .map(|r| r.overall_score)
            .collect();
        if scores.is_empty() {
            return 0.0;
        }
        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// 生成人类可读的摘要报告。
    pub fn export_report(&self) -> String {
        let mut lines = Vec::new();
        lines.push("=== OutcomeTracker Report ===".to_string());
        lines.push(format!("Total rubric sets: {}", self.rubric_sets.len()));
        lines.push(format!(
            "Total evaluations: {}",
            self.evaluation_history.len()
        ));
        lines.push(format!("Success rate: {:.2}%", self.success_rate() * 100.0));
        lines.push(String::new());

        lines.push("--- Rubric Sets ---".to_string());
        for set in self.rubric_sets.values() {
            lines.push(format!(
                "  [{}] {} (v{}, {} criteria)",
                set.id,
                set.name,
                set.version,
                set.criteria.len()
            ));
            for c in &set.criteria {
                lines.push(format!(
                    "    - {}: weight={:.2}, threshold={:.2}",
                    c.name, c.weight, c.pass_threshold
                ));
            }
            let avg = self.avg_score(set.id);
            lines.push(format!("    avg score: {:.3}", avg));
        }

        lines.push(String::new());
        lines.push("--- Recent Evaluations ---".to_string());
        let recent: Vec<_> = self.evaluation_history.iter().rev().take(10).collect();
        for r in &recent {
            let set_name = self
                .rubric_sets
                .get(&r.rubric_set_id)
                .map(|s| s.name.as_str())
                .unwrap_or("(deleted)");
            lines.push(format!(
                "  set=[{}] {}, score={:.3}, evidence_count={}",
                r.rubric_set_id,
                set_name,
                r.overall_score,
                r.evidence.len()
            ));
        }

        lines.join("\n")
    }

    /// 移除最旧的评估记录，直到历史不超过 200 条。
    pub fn prune(&mut self) {
        while self.evaluation_history.len() > 200 {
            self.evaluation_history.pop_front();
        }
    }
}

impl Default for OutcomeTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_criterion(id: u64, name: &str, weight: f64, threshold: f64) -> RubricCriterion {
        RubricCriterion {
            id,
            name: name.to_string(),
            description: String::new(),
            weight,
            pass_threshold: threshold,
        }
    }

    #[test]
    fn test_create_rubric_set() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![
            sample_criterion(1, "accuracy", 0.6, 0.7),
            sample_criterion(2, "coherence", 0.4, 0.5),
        ];
        let id = tracker.create_rubric_set("code quality", criteria);
        assert_ne!(id, 0);
        let set = tracker.get_rubric_set(id);
        assert!(set.is_some());
        assert_eq!(set.unwrap().name, "code quality");
    }

    #[test]
    fn test_evaluate_passing() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "accuracy", 1.0, 0.5)];
        let id = tracker.create_rubric_set("test pass", criteria);

        let mut scores = HashMap::new();
        scores.insert(1, 0.9);
        let result = tracker.evaluate(id, scores, vec!["test passed".to_string()]);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.overall_score - 0.9).abs() < 1e-6);
        assert!(r.overall_score >= 0.5);
    }

    #[test]
    fn test_evaluate_failing() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "accuracy", 1.0, 0.8)];
        let id = tracker.create_rubric_set("test fail", criteria);

        let mut scores = HashMap::new();
        scores.insert(1, 0.3);
        let result = tracker.evaluate(id, scores, vec![]);
        assert!(result.is_some());
        let r = result.unwrap();
        assert!((r.overall_score - 0.3).abs() < 1e-6);
        assert!(r.overall_score < 0.8);
    }

    #[test]
    fn test_multiple_rubric_sets() {
        let mut tracker = OutcomeTracker::new();
        let c1 = vec![sample_criterion(1, "a", 1.0, 0.5)];
        let c2 = vec![sample_criterion(1, "b", 1.0, 0.5)];
        let id1 = tracker.create_rubric_set("set A", c1);
        let id2 = tracker.create_rubric_set("set B", c2);
        assert_ne!(id1, id2);

        let mut s1 = HashMap::new();
        s1.insert(1, 0.8);
        let mut s2 = HashMap::new();
        s2.insert(1, 0.6);
        assert!(tracker.evaluate(id1, s1, vec![]).is_some());
        assert!(tracker.evaluate(id2, s2, vec![]).is_some());

        assert_eq!(tracker.list_rubric_sets().len(), 2);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "accuracy", 1.0, 0.5)];
        let id = tracker.create_rubric_set("rate test", criteria);

        let mut s_pass = HashMap::new();
        s_pass.insert(1, 0.9);
        let mut s_fail = HashMap::new();
        s_fail.insert(1, 0.1);

        tracker.evaluate(id, s_pass, vec![]);
        tracker.evaluate(id, s_fail.clone(), vec![]);
        tracker.evaluate(id, s_fail, vec![]);

        assert!((tracker.success_rate() - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_avg_score() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "accuracy", 1.0, 0.0)];
        let id = tracker.create_rubric_set("avg test", criteria);

        let mut s1 = HashMap::new();
        s1.insert(1, 0.8);
        let mut s2 = HashMap::new();
        s2.insert(1, 0.6);
        tracker.evaluate(id, s1, vec![]);
        tracker.evaluate(id, s2, vec![]);

        assert!((tracker.avg_score(id) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_avg_score_no_evaluations() {
        let tracker = OutcomeTracker::new();
        assert!((tracker.avg_score(999) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_export_report_format() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "accuracy", 1.0, 0.5)];
        let id = tracker.create_rubric_set("report test", criteria);
        let mut scores = HashMap::new();
        scores.insert(1, 0.9);
        tracker.evaluate(id, scores, vec!["good".to_string()]);

        let report = tracker.export_report();
        assert!(report.contains("OutcomeTracker Report"));
        assert!(report.contains("report test"));
        assert!(report.contains("1 rubric sets"));
        assert!(report.contains("1 evaluations"));
    }

    #[test]
    fn test_prune_removes_oldest() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "acc", 1.0, 0.0)];
        let id = tracker.create_rubric_set("prune test", criteria);

        for i in 0..210 {
            let mut scores = HashMap::new();
            scores.insert(1, 0.5);
            tracker.evaluate(
                id,
                scores,
                vec![format!("eval {}", i)],
            );
        }

        assert_eq!(tracker.evaluation_history.len(), 210);
        tracker.prune();
        assert_eq!(tracker.evaluation_history.len(), 200);
        // first evidence should be "eval 10" after prune
        let first = tracker.evaluation_history.front().unwrap();
        assert_eq!(first.evidence[0], "eval 10");
    }

    #[test]
    fn test_weighted_scoring() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![
            sample_criterion(1, "important", 0.75, 0.5),
            sample_criterion(2, "minor", 0.25, 0.5),
        ];
        let id = tracker.create_rubric_set("weighted", criteria);

        let mut scores = HashMap::new();
        scores.insert(1, 1.0);
        scores.insert(2, 0.0);
        let result = tracker.evaluate(id, scores, vec![]).unwrap();

        assert!((result.overall_score - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_invalid_id_returns_none() {
        let mut tracker = OutcomeTracker::new();
        let scores = HashMap::new();
        assert!(tracker.evaluate(999, scores, vec![]).is_none());
    }

    #[test]
    fn test_evaluate_missing_all_scores_returns_none() {
        let mut tracker = OutcomeTracker::new();
        let criteria = vec![sample_criterion(1, "acc", 1.0, 0.5)];
        let id = tracker.create_rubric_set("no scores", criteria);
        let scores = HashMap::new();
        assert!(tracker.evaluate(id, scores, vec![]).is_none());
    }

    #[test]
    fn test_success_rate_empty() {
        let tracker = OutcomeTracker::new();
        assert!((tracker.success_rate() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_export_report_empty() {
        let tracker = OutcomeTracker::new();
        let report = tracker.export_report();
        assert!(report.contains("0 rubric sets"));
        assert!(report.contains("0 evaluations"));
    }
}
