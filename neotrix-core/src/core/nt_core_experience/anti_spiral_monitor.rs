use std::collections::{HashMap, VecDeque};

/// 检测到的螺旋/振荡模式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiralPattern {
    /// 相同任务被重复创建 3+ 次
    RepeatedProposals,
    /// 任务类型在 A↔B 之间来回切换
    TaskTypeOscillation,
    /// meta_accuracy 在 50 cycle 内无改善
    MetaStagnation,
    /// 同一 weakness 被连续报告 5+ 次
    RepeatedWeakness,
}

impl SpiralPattern {
    pub fn name(&self) -> &'static str {
        match self {
            SpiralPattern::RepeatedProposals => "repeated_proposals",
            SpiralPattern::TaskTypeOscillation => "task_type_oscillation",
            SpiralPattern::MetaStagnation => "meta_stagnation",
            SpiralPattern::RepeatedWeakness => "repeated_weakness",
        }
    }

    pub fn severity(&self) -> f64 {
        match self {
            SpiralPattern::RepeatedProposals => 0.5,
            SpiralPattern::TaskTypeOscillation => 0.7,
            SpiralPattern::MetaStagnation => 0.6,
            SpiralPattern::RepeatedWeakness => 0.4,
        }
    }
}

/// 单次反螺旋检测结果
#[derive(Debug, Clone)]
pub struct SpiralDetection {
    pub pattern: SpiralPattern,
    pub cycle: u64,
    pub severity: f64,
    pub description: String,
    /// 建议的恢复动作
    pub suggestion: String,
}

/// HIVE 风格反螺旋监控器
///
/// 检测 4 种推理螺旋模式:
/// 1. RepeatedProposals — 相同任务重复创建
/// 2. TaskTypeOscillation — 类型间来回切换
/// 3. MetaStagnation — 元精度长期停滞
/// 4. RepeatedWeakness — 同一弱点反复报告
#[derive(Debug)]
pub struct AntiSpiralMonitor {
    /// 最近创建的任务类型历史 (type_name, cycle)
    task_history: VecDeque<(String, u64)>,
    /// 最近报告的 weakness 描述历史
    weakness_history: VecDeque<String>,
    /// cycle → meta_accuracy 快照
    meta_snapshots: VecDeque<(u64, f64)>,
    /// 已检测到的螺旋事件
    pub detections: Vec<SpiralDetection>,
    /// 上次检测的 cycle (防同一 cycle 重复报告)
    last_detection_cycle: u64,
    /// 配置参数
    pub config: AntiSpiralConfig,
}

#[derive(Debug, Clone)]
pub struct AntiSpiralConfig {
    /// 任务历史保留长度
    pub task_history_capacity: usize,
    /// Weakness 历史保留长度
    pub weakness_capacity: usize,
    /// Meta 快照保留数
    pub meta_capacity: usize,
    /// 重复提案检测阈值 (相同 title 出现 N 次)
    pub repeated_proposal_threshold: usize,
    /// 振荡检测窗口
    pub oscillation_window: usize,
    /// 停滞检测窗口 (cycle)
    pub stagnation_window: u64,
    /// 停滞改善阈值 (meta_accuracy 变化 < this)
    pub stagnation_improvement_threshold: f64,
}

impl Default for AntiSpiralConfig {
    fn default() -> Self {
        Self {
            task_history_capacity: 50,
            weakness_capacity: 30,
            meta_capacity: 100,
            repeated_proposal_threshold: 3,
            oscillation_window: 6,
            stagnation_window: 50,
            stagnation_improvement_threshold: 0.02,
        }
    }
}

impl AntiSpiralMonitor {
    pub fn new(config: AntiSpiralConfig) -> Self {
        Self {
            task_history: VecDeque::with_capacity(config.task_history_capacity),
            weakness_history: VecDeque::with_capacity(config.weakness_capacity),
            meta_snapshots: VecDeque::with_capacity(config.meta_capacity),
            detections: Vec::new(),
            last_detection_cycle: 0,
            config,
        }
    }

    /// 记录一次任务创建事件
    pub fn record_task_creation(&mut self, task_type: &str, cycle: u64) {
        if self.task_history.len() >= self.config.task_history_capacity {
            self.task_history.pop_front();
        }
        self.task_history.push_back((task_type.to_string(), cycle));
    }

    /// 记录一次 weakness 报告
    pub fn record_weakness(&mut self, description: &str) {
        if self.weakness_history.len() >= self.config.weakness_capacity {
            self.weakness_history.pop_front();
        }
        self.weakness_history.push_back(description.to_string());
    }

    /// 记录 meta_accuracy 快照
    pub fn record_meta_snapshot(&mut self, cycle: u64, meta_accuracy: f64) {
        if self.meta_snapshots.len() >= self.config.meta_capacity {
            self.meta_snapshots.pop_front();
        }
        self.meta_snapshots.push_back((cycle, meta_accuracy));
    }

    /// 执行一次检测扫描, 返回本次发现的新螺旋模式
    pub fn scan(&mut self, cycle: u64) -> Vec<SpiralDetection> {
        if cycle == self.last_detection_cycle {
            return Vec::new();
        }
        self.last_detection_cycle = cycle;

        let mut findings = Vec::new();

        // 1. 重复提案检测
        if let Some(detection) = self.detect_repeated_proposals(cycle) {
            findings.push(detection);
        }

        // 2. 振荡检测
        if let Some(detection) = self.detect_oscillation(cycle) {
            findings.push(detection);
        }

        // 3. 停滞检测
        if let Some(detection) = self.detect_stagnation(cycle) {
            findings.push(detection);
        }

        // 4. 重复 weakness 检测
        if let Some(detection) = self.detect_repeated_weakness(cycle) {
            findings.push(detection);
        }

        for f in &findings {
            self.detections.push(f.clone());
        }
        // Keep only last 50 detections
        if self.detections.len() > 50 {
            self.detections.drain(0..self.detections.len() - 50);
        }

        findings
    }

    /// 检测相同 type 的任务是否被重复创建阈值次数
    fn detect_repeated_proposals(&self, cycle: u64) -> Option<SpiralDetection> {
        let type_counts: HashMap<&str, usize> =
            self.task_history
                .iter()
                .fold(HashMap::new(), |mut acc, (t, _)| {
                    *acc.entry(t.as_str()).or_insert(0) += 1;
                    acc
                });
        for (type_name, count) in &type_counts {
            if *count >= self.config.repeated_proposal_threshold {
                return Some(SpiralDetection {
                    pattern: SpiralPattern::RepeatedProposals,
                    cycle,
                    severity: SpiralPattern::RepeatedProposals.severity()
                        + (*count as f64 - 2.0) * 0.1,
                    description: format!(
                        "task type '{}' created {} times in recent history (threshold: {})",
                        type_name, count, self.config.repeated_proposal_threshold
                    ),
                    suggestion: format!(
                        "halt new '{}' tasks, investigate root cause instead of creating more tasks of same type",
                        type_name
                    ),
                });
            }
        }
        None
    }

    /// 检测任务类型是否在振荡 (A→B→A→B 模式)
    fn detect_oscillation(&self, cycle: u64) -> Option<SpiralDetection> {
        if self.task_history.len() < 4 {
            return None;
        }
        let recent: Vec<&str> = self
            .task_history
            .iter()
            .rev()
            .take(self.config.oscillation_window)
            .map(|(t, _)| t.as_str())
            .collect();

        // Check for alternating pattern: A, B, A, B, A, B
        if recent.len() >= 4 {
            let mut alternating = true;
            for i in 2..recent.len() {
                if recent[i] != recent[i - 2] {
                    alternating = false;
                    break;
                }
            }
            if alternating && recent[0] != recent[1] {
                return Some(SpiralDetection {
                    pattern: SpiralPattern::TaskTypeOscillation,
                    cycle,
                    severity: SpiralPattern::TaskTypeOscillation.severity(),
                    description: format!(
                        "task types oscillating between '{}' and '{}' in last {} entries",
                        recent[0],
                        recent[1],
                        recent.len()
                    ),
                    suggestion: format!(
                        "break oscillation: consolidate '{}' and '{}' into a single composite task",
                        recent[0], recent[1]
                    ),
                });
            }
        }
        None
    }

    /// 检测 meta_accuracy 长期停滞
    fn detect_stagnation(&self, cycle: u64) -> Option<SpiralDetection> {
        if self.meta_snapshots.len() < 10 {
            return None;
        }
        let window_start = cycle.saturating_sub(self.config.stagnation_window);
        let in_window: Vec<&(u64, f64)> = self
            .meta_snapshots
            .iter()
            .filter(|(c, _)| *c >= window_start)
            .collect();

        if in_window.len() < 5 {
            return None;
        }

        let earliest = in_window.first().map(|(_, v)| *v).unwrap_or(0.5);
        let latest = in_window.last().map(|(_, v)| *v).unwrap_or(0.5);
        let improvement = latest - earliest;

        if improvement.abs() < self.config.stagnation_improvement_threshold {
            Some(SpiralDetection {
                pattern: SpiralPattern::MetaStagnation,
                cycle,
                severity: SpiralPattern::MetaStagnation.severity()
                    + (1.0 - improvement.abs() / 0.1).clamp(0.0, 0.3),
                description: format!(
                    "meta_accuracy stagnant over {} cycles: {:.4} -> {:.4} (Δ={:.4})",
                    self.config.stagnation_window, earliest, latest, improvement
                ),
                suggestion: "inject novelty: explore a new domain or mutate existing strategy more aggressively"
                    .to_string(),
            })
        } else {
            None
        }
    }

    /// 检测相同的 weakness 描述是否被反复报告
    fn detect_repeated_weakness(&self, cycle: u64) -> Option<SpiralDetection> {
        if self.weakness_history.len() < 5 {
            return None;
        }
        let recent: Vec<&str> = self
            .weakness_history
            .iter()
            .rev()
            .take(10)
            .map(|s| s.as_str())
            .collect();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for w in &recent {
            *counts.entry(*w).or_insert(0) += 1;
        }
        for (weakness, count) in &counts {
            if *count >= 5 {
                return Some(SpiralDetection {
                    pattern: SpiralPattern::RepeatedWeakness,
                    cycle,
                    severity: SpiralPattern::RepeatedWeakness.severity(),
                    description: format!(
                        "weakness '{}' reported {} times in last {} entries",
                        weakness, count, recent.len()
                    ),
                    suggestion: format!(
                        "the fix for '{}' is ineffective; try a different approach or escalate to a higher-priority task",
                        weakness
                    ),
                });
            }
        }
        None
    }

    /// 统计数据
    pub fn stats(&self) -> AntiSpiralStats {
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for d in &self.detections {
            *by_type.entry(d.pattern.name().to_string()).or_insert(0) += 1;
        }
        AntiSpiralStats {
            total_detections: self.detections.len(),
            detections_by_type: by_type,
            active_patterns: self
                .detections
                .iter()
                .rev()
                .take(5)
                .map(|d| d.pattern.name().to_string())
                .collect(),
        }
    }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "anti_spiral: {} total detections (types: {:?}) last_5={:?}",
            s.total_detections, s.detections_by_type, s.active_patterns
        )
    }
}

#[derive(Debug, Clone)]
pub struct AntiSpiralStats {
    pub total_detections: usize,
    pub detections_by_type: HashMap<String, usize>,
    pub active_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_monitor() -> AntiSpiralMonitor {
        AntiSpiralMonitor::new(AntiSpiralConfig {
            repeated_proposal_threshold: 3,
            oscillation_window: 4,
            stagnation_window: 10,
            stagnation_improvement_threshold: 0.02,
            ..Default::default()
        })
    }

    #[test]
    fn test_clean_history_no_detection() {
        let mut m = make_monitor();
        m.record_meta_snapshot(0, 0.8);
        m.record_meta_snapshot(5, 0.81);
        let findings = m.scan(10);
        assert!(
            findings.is_empty(),
            "clean history should produce no detections"
        );
    }

    #[test]
    fn test_repeated_proposal_detection() {
        let mut m = make_monitor();
        for i in 0..4 {
            m.record_task_creation("module_wiring", i);
        }
        let findings = m.scan(5);
        let has_repeated = findings
            .iter()
            .any(|d| d.pattern == SpiralPattern::RepeatedProposals);
        assert!(
            has_repeated,
            "4 identical task types should trigger repeated_proposals"
        );
    }

    #[test]
    fn test_oscillation_detection() {
        let mut m = make_monitor();
        let pattern = vec!["a", "b", "a", "b", "a", "b"];
        for (i, t) in pattern.iter().enumerate() {
            m.record_task_creation(t, i as u64);
        }
        let findings = m.scan(10);
        let has_osc = findings
            .iter()
            .any(|d| d.pattern == SpiralPattern::TaskTypeOscillation);
        assert!(
            has_osc,
            "A-B-A-B pattern should trigger oscillation detection"
        );
    }

    #[test]
    fn test_same_cycle_de_dup() {
        let mut m = make_monitor();
        for i in 0..4 {
            m.record_task_creation("module_wiring", i);
        }
        let _ = m.scan(10);
        let findings2 = m.scan(10);
        assert!(findings2.is_empty(), "same cycle should return empty");
    }

    #[test]
    fn test_stagnation_detection() {
        let mut m = make_monitor();
        for i in 0..12 {
            // All roughly the same meta_accuracy
            m.record_meta_snapshot(i as u64, 0.50 + (i as f64 * 0.001));
        }
        let findings = m.scan(15);
        let has_stag = findings
            .iter()
            .any(|d| d.pattern == SpiralPattern::MetaStagnation);
        assert!(
            has_stag,
            "flat meta_accuracy should trigger stagnation detection"
        );
    }

    #[test]
    fn test_repeated_weakness() {
        let mut m = make_monitor();
        for _ in 0..6 {
            m.record_weakness("calibration_ece_too_high");
        }
        let findings = m.scan(10);
        let has_weak = findings
            .iter()
            .any(|d| d.pattern == SpiralPattern::RepeatedWeakness);
        assert!(
            has_weak,
            "6 identical weaknesses should trigger repeated_weakness"
        );
    }

    #[test]
    fn test_no_oscillation_non_alternating() {
        let mut m = make_monitor();
        let pattern = vec!["a", "a", "a", "b", "b", "b"];
        for (i, t) in pattern.iter().enumerate() {
            m.record_task_creation(t, i as u64);
        }
        let findings = m.scan(10);
        let has_osc = findings
            .iter()
            .any(|d| d.pattern == SpiralPattern::TaskTypeOscillation);
        assert!(
            !has_osc,
            "non-alternating pattern should not trigger oscillation"
        );
    }

    #[test]
    fn test_spiral_pattern_name() {
        assert_eq!(
            SpiralPattern::RepeatedProposals.name(),
            "repeated_proposals"
        );
        assert_eq!(
            SpiralPattern::TaskTypeOscillation.name(),
            "task_type_oscillation"
        );
        assert_eq!(SpiralPattern::MetaStagnation.name(), "meta_stagnation");
        assert_eq!(SpiralPattern::RepeatedWeakness.name(), "repeated_weakness");
    }

    #[test]
    fn test_stats_tracking() {
        let mut m = make_monitor();
        for i in 0..6 {
            m.record_task_creation("module_wiring", i);
        }
        let _ = m.scan(10);
        let s = m.stats();
        assert!(s.total_detections > 0);
    }
}
