#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};

/// 梦境模式类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DreamCategory {
    /// 重复出现的错误/panic 模式
    ErrorPattern,
    /// 可优化的性能/效率机会
    OptimizationOpportunity,
    /// 知识缺口 — 尚未覆盖的领域
    KnowledgeGap,
    /// 行为层面的洞察 — 重复的用户/系统交互模式
    BehavioralInsight,
    /// 技能合并 — 多次成功的变异模式可结晶为技能
    SkillConsolidation,
    /// 架构层面的弱点
    ArchitectureWeakness,
}

impl DreamCategory {
    pub fn name(&self) -> &'static str {
        match self {
            DreamCategory::ErrorPattern => "error_pattern",
            DreamCategory::OptimizationOpportunity => "optimization_opportunity",
            DreamCategory::KnowledgeGap => "knowledge_gap",
            DreamCategory::BehavioralInsight => "behavioral_insight",
            DreamCategory::SkillConsolidation => "skill_consolidation",
            DreamCategory::ArchitectureWeakness => "architecture_weakness",
        }
    }
}

/// 梦境循环调度器的配置
#[derive(Debug, Clone)]
pub struct DreamConfig {
    /// 两次梦境循环之间的间隔 (cycle 数)
    pub interval_cycles: u64,
    /// 形成模式所需的最小迹文本数量
    pub min_traces_for_pattern: usize,
    /// 最大保留模式数
    pub max_patterns: usize,
    /// 置信度阈值 — 低于此值的模式被修剪
    pub confidence_threshold: f64,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            interval_cycles: 30,
            min_traces_for_pattern: 5,
            max_patterns: 100,
            confidence_threshold: 0.3,
        }
    }
}

/// 单个梦境模式 — 从执行迹中合成的抽象洞察
#[derive(Debug, Clone)]
pub struct DreamPattern {
    /// 唯一标识
    pub id: u64,
    /// 模式类别
    pub category: DreamCategory,
    /// 人类可读的模式描述
    pub description: String,
    /// 模式置信度 [0, 1]
    pub confidence: f64,
    /// 形成此模式的源迹文本
    pub source_traces: Vec<String>,
    /// 创建时的 cycle
    pub created_at: u64,
    /// 匹配命中次数
    pub hit_count: u32,
    /// 最近一次匹配的 cycle
    pub last_matched: u64,
}

/// 梦境统计快照
#[derive(Debug, Clone, Default)]
pub struct DreamStats {
    pub total_patterns: usize,
    pub by_category: HashMap<String, usize>,
    pub dream_count: u64,
}

/// 梦境循环调度器
///
/// 以 Claude Code Managed Agents "Dreaming" 的调度模式为参考，
/// 定期审阅执行迹，识别重复模式，合成新洞察并注入经验树/技能库。
///
/// 三阶段流程:
///   1. 记录迹 (record_trace) — 持续收集执行迹文本
///   2. 分析 (run_dream_cycle) — 按周期扫描迹，检测模式
///   3. 合成 (synthesize_reflection) — 将模式转为可读反思
#[derive(Debug)]
pub struct DreamCycleScheduler {
    /// 执行迹环形缓冲 (最大 500 条)
    trace_buffer: VecDeque<String>,
    /// 已合成的模式列表
    synthesized_patterns: Vec<DreamPattern>,
    /// 配置
    config: DreamConfig,
    /// 下一个模式 ID
    next_id: u64,
    /// 上次梦境 cycle
    last_dream_cycle: u64,
    /// 梦境运行次数
    dream_count: u64,
}

impl DreamCycleScheduler {
    pub fn new() -> Self {
        Self {
            trace_buffer: VecDeque::with_capacity(500),
            synthesized_patterns: Vec::new(),
            config: DreamConfig::default(),
            next_id: 1,
            last_dream_cycle: 0,
            dream_count: 0,
        }
    }

    /// 用自定义配置创建调度器
    pub fn with_config(config: DreamConfig) -> Self {
        Self {
            trace_buffer: VecDeque::with_capacity(500),
            synthesized_patterns: Vec::new(),
            config,
            next_id: 1,
            last_dream_cycle: 0,
            dream_count: 0,
        }
    }

    /// 记录一条执行迹文本。超过 500 条时淘汰最老的。
    pub fn record_trace(&mut self, text: String) {
        if self.trace_buffer.len() >= 500 {
            self.trace_buffer.pop_front();
        }
        self.trace_buffer.push_back(text);
    }

    /// 当前 cycle 是否应该运行梦境
    pub fn should_dream(&self, cycle: u64) -> bool {
        cycle >= self.last_dream_cycle + self.config.interval_cycles
    }

    /// 返回所有模式
    pub fn patterns(&self) -> &[DreamPattern] {
        &self.synthesized_patterns
    }

    /// 按类别筛选模式
    pub fn patterns_by_category(&self, cat: DreamCategory) -> Vec<&DreamPattern> {
        self.synthesized_patterns
            .iter()
            .filter(|p| p.category == cat)
            .collect()
    }

    /// 统计快照
    pub fn stats(&self) -> DreamStats {
        let mut by_category: HashMap<String, usize> = HashMap::new();
        for p in &self.synthesized_patterns {
            *by_category
                .entry(p.category.name().to_string())
                .or_insert(0) += 1;
        }
        DreamStats {
            total_patterns: self.synthesized_patterns.len(),
            by_category,
            dream_count: self.dream_count,
        }
    }

    /// 修剪置信度低于阈值的模式
    pub fn prune(&mut self) {
        let threshold = self.config.confidence_threshold;
        self.synthesized_patterns
            .retain(|p| p.confidence >= threshold);
    }

    /// 运行一次梦境循环
    ///
    /// 1. 对迹缓冲进行关键词频率分析
    /// 2. 按类别启发式分组相关迹
    /// 3. 对每组 >= min_traces_for_pattern 的迹创建/更新 DreamPattern
    /// 4. 新模式起始置信度 0.3，随 hit_count 递增
    /// 5. 已匹配的模式 hit_count++ 且 confidence +0.05 (上限 1.0)
    ///
    /// 返回本次新创建的所有模式
    pub fn run_dream_cycle(&mut self, cycle: u64) -> Vec<DreamPattern> {
        self.last_dream_cycle = cycle;
        self.dream_count += 1;

        let mut new_patterns: Vec<DreamPattern> = Vec::new();

        // 为每个类别计数和收集迹
        let grouped = self.group_traces_by_category();

        for (category, traces) in &grouped {
            if traces.len() < self.config.min_traces_for_pattern {
                continue;
            }

            let description = Self::generate_description(*category, traces.len());

            // 检查是否已有同类模式可以更新
            let existing = self
                .synthesized_patterns
                .iter_mut()
                .find(|p| p.category == *category && p.hit_count < 100);

            if let Some(pattern) = existing {
                pattern.hit_count += 1;
                pattern.last_matched = cycle;
                pattern.confidence = (pattern.confidence + 0.05).min(1.0);
                // 补充新的源迹，但避免无限增长
                for t in traces.iter().take(5) {
                    if pattern.source_traces.len() < 50 {
                        pattern.source_traces.push(t.clone());
                    }
                }
            } else if self.synthesized_patterns.len() < self.config.max_patterns {
                let pattern = DreamPattern {
                    id: self.next_id,
                    category: *category,
                    description,
                    confidence: 0.3,
                    source_traces: traces.iter().take(20).cloned().collect(),
                    created_at: cycle,
                    hit_count: 1,
                    last_matched: cycle,
                };
                self.next_id += 1;
                new_patterns.push(pattern.clone());
                self.synthesized_patterns.push(pattern);
            }
        }

        new_patterns
    }

    /// 按类别启发式将迹文本分组
    fn group_traces_by_category(&self) -> HashMap<DreamCategory, Vec<String>> {
        let mut groups: HashMap<DreamCategory, Vec<String>> = HashMap::new();
        groups.insert(DreamCategory::ErrorPattern, Vec::new());
        groups.insert(DreamCategory::OptimizationOpportunity, Vec::new());
        groups.insert(DreamCategory::KnowledgeGap, Vec::new());
        groups.insert(DreamCategory::BehavioralInsight, Vec::new());
        groups.insert(DreamCategory::SkillConsolidation, Vec::new());
        groups.insert(DreamCategory::ArchitectureWeakness, Vec::new());

        for trace in &self.trace_buffer {
            let lower = trace.to_lowercase();
            let mut categorized = false;

            // 优先匹配特异性高的类别
            if contains_any(&lower, &["dead code", "unused", "deprecated", "workaround"]) {
                groups
                    .get_mut(&DreamCategory::ArchitectureWeakness)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            } else if contains_any(
                &lower,
                &["slow", "o(n²)", "o(n^2)", "bottleneck", "latency"],
            ) {
                groups
                    .get_mut(&DreamCategory::OptimizationOpportunity)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            } else if contains_any(&lower, &["unknown", "not implemented", "todo", "missing"]) {
                groups
                    .get_mut(&DreamCategory::KnowledgeGap)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            } else if contains_any(&lower, &["mutation", "crystal", "skill"]) {
                groups
                    .get_mut(&DreamCategory::SkillConsolidation)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            } else if contains_any(&lower, &["intervention", "pattern", "insight", "observed"]) {
                groups
                    .get_mut(&DreamCategory::BehavioralInsight)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            } else if contains_any(
                &lower,
                &["panic", "unwrap", "crash", "timeout", "error", "failed"],
            ) {
                groups
                    .get_mut(&DreamCategory::ErrorPattern)
                    .unwrap()
                    .push(trace.clone());
                categorized = true;
            }

            // 未分类的痕记到 ErrorPattern 作为兜底
            if !categorized && contains_any(&lower, &["error", "fail", "exception"]) {
                groups
                    .get_mut(&DreamCategory::ErrorPattern)
                    .unwrap()
                    .push(trace.clone());
            }
        }

        groups
    }

    /// 根据类别和迹数量生成人类可读的描述
    fn generate_description(category: DreamCategory, trace_count: usize) -> String {
        match category {
            DreamCategory::ErrorPattern => {
                format!(
                    "Detected recurring error patterns across {} execution traces — suggest root cause analysis",
                    trace_count
                )
            }
            DreamCategory::OptimizationOpportunity => {
                format!(
                    "Identified {} traces with potential optimization targets — review latency and complexity bottlenecks",
                    trace_count
                )
            }
            DreamCategory::KnowledgeGap => {
                format!(
                    "Found {} traces indicating knowledge gaps — consider targeted exploration or curriculum learning",
                    trace_count
                )
            }
            DreamCategory::BehavioralInsight => {
                format!(
                    "Synthesized {} traces into behavioral insight — recurring interaction or intervention pattern detected",
                    trace_count
                )
            }
            DreamCategory::SkillConsolidation => {
                format!(
                    "Observed {} successful mutation traces eligible for skill crystallization",
                    trace_count
                )
            }
            DreamCategory::ArchitectureWeakness => {
                format!(
                    "Architecture weakness flagged across {} traces — dead code, deprecated paths, or workarounds accumulating",
                    trace_count
                )
            }
        }
    }

    /// 合成人类可读的梦境反思报告
    ///
    /// 输出格式:
    /// ```
    /// === Dream Reflection (cycle={cycle}) ===
    /// Patterns: 5 total | 3 new this cycle | dream_count={dream_count}
    /// - ErrorPattern (2): Detected recurring error patterns...
    /// - OptimizationOpportunity (1): Identified...
    /// ```
    pub fn synthesize_reflection(&self) -> String {
        let mut by_cat: HashMap<DreamCategory, Vec<&DreamPattern>> = HashMap::new();
        for p in &self.synthesized_patterns {
            by_cat.entry(p.category).or_default().push(p);
        }

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "=== Dream Reflection === Patterns: {} total | dream_count: {}",
            self.synthesized_patterns.len(),
            self.dream_count
        ));

        let mut cats: Vec<_> = by_cat.into_iter().collect();
        cats.sort_by_key(|(c, _)| *c as u8);

        for (category, patterns) in &cats {
            let max_desc = patterns
                .iter()
                .max_by(|a, b| {
                    a.confidence
                        .partial_cmp(&b.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|p| p.description.as_str())
                .unwrap_or("");
            lines.push(format!(
                "  - {} ({}): hit_count={}, confidence={:.2} | {}",
                category.name(),
                patterns.len(),
                patterns.iter().map(|p| p.hit_count).sum::<u32>(),
                patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64,
                max_desc,
            ));
        }

        lines.join("\n")
    }
}

impl Default for DreamCycleScheduler {
    fn default() -> Self {
        Self::new()
    }
}

fn contains_any(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scheduler() -> DreamCycleScheduler {
        DreamCycleScheduler::new()
    }

    fn make_scheduler_small() -> DreamCycleScheduler {
        DreamCycleScheduler::with_config(DreamConfig {
            interval_cycles: 5,
            min_traces_for_pattern: 2,
            max_patterns: 20,
            confidence_threshold: 0.3,
        })
    }

    #[test]
    fn test_trace_recording_and_eviction() {
        let mut s = make_scheduler();
        assert!(s.trace_buffer.is_empty());

        for i in 0..10 {
            s.record_trace(format!("trace text {}", i));
        }
        assert_eq!(s.trace_buffer.len(), 10);

        // Eviction: record 500+ should keep at 500
        let large_cfg = DreamConfig {
            interval_cycles: 30,
            min_traces_for_pattern: 5,
            max_patterns: 100,
            confidence_threshold: 0.3,
        };
        let mut s2 = DreamCycleScheduler::with_config(large_cfg);
        for i in 0..520 {
            s2.record_trace(format!("trace {}", i));
        }
        assert_eq!(s2.trace_buffer.len(), 500);
        // Oldest should be trace 20 (since indices 0..19 evicted)
        assert_eq!(s2.trace_buffer.front().unwrap(), "trace 20");
    }

    #[test]
    fn test_should_dream_timing() {
        let cfg = DreamConfig {
            interval_cycles: 10,
            min_traces_for_pattern: 5,
            max_patterns: 100,
            confidence_threshold: 0.3,
        };
        let s = DreamCycleScheduler::with_config(cfg);

        // cycle 0: should NOT dream (last_dream_cycle=0, interval=10)
        assert!(!s.should_dream(0));
        // cycle 5: still no
        assert!(!s.should_dream(5));
        // cycle 10: == 0 + 10, should dream
        assert!(s.should_dream(10));
        // cycle 15: should still dream (15 >= 0+10)
        assert!(s.should_dream(15));
    }

    #[test]
    fn test_should_dream_after_run() {
        let mut s = make_scheduler_small();
        assert!(!s.should_dream(0));

        s.run_dream_cycle(5);
        // last_dream_cycle = 5, interval = 5
        assert!(!s.should_dream(9));
        assert!(s.should_dream(10));
    }

    #[test]
    fn test_pattern_creation_from_traces() {
        let mut s = make_scheduler_small();

        // Feed enough error-pattern traces
        for i in 0..6 {
            s.record_trace(format!(
                "panic occurred in module {}: unwrap failed on None",
                i
            ));
        }
        for i in 0..4 {
            s.record_trace(format!("slow query detected: O(n²) behavior at line {}", i));
        }

        let new = s.run_dream_cycle(10);
        assert!(!new.is_empty(), "should create at least one pattern");

        let err_patterns = s.patterns_by_category(DreamCategory::ErrorPattern);
        assert!(!err_patterns.is_empty(), "should have error pattern");

        let opt_patterns = s.patterns_by_category(DreamCategory::OptimizationOpportunity);
        // Only 4 traces, min is 2, so should be created
        assert!(!opt_patterns.is_empty(), "should have optimization pattern");
    }

    #[test]
    fn test_pattern_hit_count_increment() {
        let mut s = make_scheduler_small();

        for i in 0..6 {
            s.record_trace(format!("panic error trace {}", i));
        }
        let _ = s.run_dream_cycle(10);

        // Verify hit_count = 1 for error pattern
        for p in &s.synthesized_patterns {
            if p.category == DreamCategory::ErrorPattern {
                assert_eq!(p.hit_count, 1, "first match should set hit_count=1");
                assert!(
                    (p.confidence - 0.3).abs() < 1e-6,
                    "initial confidence should be 0.3"
                );
            }
        }

        // Second dream cycle with more error traces
        for i in 0..3 {
            s.record_trace(format!("another panic unwrap crash {}", i));
        }
        let _ = s.run_dream_cycle(20);

        for p in &s.synthesized_patterns {
            if p.category == DreamCategory::ErrorPattern {
                assert_eq!(p.hit_count, 2, "second match should increment hit_count");
                assert!(
                    (p.confidence - 0.35).abs() < 1e-6,
                    "confidence should be 0.35 after +0.05"
                );
            }
        }
    }

    #[test]
    fn test_prune_removes_low_confidence() {
        let mut s = DreamCycleScheduler::with_config(DreamConfig {
            interval_cycles: 1,
            min_traces_for_pattern: 2,
            max_patterns: 100,
            confidence_threshold: 0.5,
        });

        // Create some patterns
        for i in 0..3 {
            s.record_trace(format!("panic crash error {}", i));
        }
        let _ = s.run_dream_cycle(5);

        assert!(
            !s.synthesized_patterns.is_empty(),
            "should have patterns before prune"
        );

        s.prune();

        // All patterns have confidence 0.3, threshold is 0.5, so all should be removed
        assert!(
            s.synthesized_patterns.is_empty(),
            "all patterns should be pruned"
        );
    }

    #[test]
    fn test_stats_accuracy() {
        let mut s = make_scheduler_small();

        for i in 0..6 {
            s.record_trace(format!("panic crash {}", i));
        }
        for i in 0..6 {
            s.record_trace(format!("slow bottleneck {}", i));
        }
        for i in 0..6 {
            s.record_trace(format!("unknown api {}", i));
        }

        let _ = s.run_dream_cycle(10);
        let stats = s.stats();

        assert!(
            stats.total_patterns >= 3,
            "should have at least 3 pattern categories"
        );
        assert_eq!(stats.dream_count, 1);
        assert!(stats.by_category.contains_key("error_pattern"));
        assert!(stats.by_category.contains_key("optimization_opportunity"));
        assert!(stats.by_category.contains_key("knowledge_gap"));
    }

    #[test]
    fn test_synthesize_reflection_output() {
        let mut s = make_scheduler_small();

        for i in 0..6 {
            s.record_trace(format!("panic crash error {}", i));
        }
        let _ = s.run_dream_cycle(10);

        let reflection = s.synthesize_reflection();
        assert!(reflection.contains("=== Dream Reflection ==="));
        assert!(reflection.contains("dream_count: 1"));
        assert!(reflection.contains("ErrorPattern") || reflection.contains("error_pattern"));
    }

    #[test]
    fn test_patterns_by_category() {
        let mut s = make_scheduler_small();

        for i in 0..6 {
            s.record_trace(format!("panic crash {}", i));
        }
        let _ = s.run_dream_cycle(10);

        let errs = s.patterns_by_category(DreamCategory::ErrorPattern);
        assert!(!errs.is_empty());

        let gaps = s.patterns_by_category(DreamCategory::KnowledgeGap);
        // Should be empty since we didn't add knowledge-gap traces
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_multiple_dream_cycles() {
        let mut s = make_scheduler_small();

        // Cycle 1
        for i in 0..3 {
            s.record_trace(format!("panic crash {}", i));
        }
        let new1 = s.run_dream_cycle(5);
        assert!(!new1.is_empty());

        // Cycle 2: more traces, existing pattern should be updated
        for i in 0..3 {
            s.record_trace(format!("unwrap error {}", i));
        }
        let new2 = s.run_dream_cycle(10);
        // No new pattern for same category
        let err_count = s.patterns_by_category(DreamCategory::ErrorPattern).len();
        assert_eq!(err_count, 1, "should still have 1 error pattern");

        // Cycle 3: new category
        for i in 0..3 {
            s.record_trace(format!("slow bottleneck {}", i));
        }
        let new3 = s.run_dream_cycle(15);
        assert!(new3
            .iter()
            .any(|p| p.category == DreamCategory::OptimizationOpportunity));
    }

    #[test]
    fn test_architecture_weakness_detection() {
        let mut s = make_scheduler_small();
        for i in 0..5 {
            s.record_trace(format!(
                "dead code in module_{}: unused function detected",
                i
            ));
        }

        let new = s.run_dream_cycle(10);
        let has_arch = new
            .iter()
            .any(|p| p.category == DreamCategory::ArchitectureWeakness);
        assert!(
            has_arch,
            "should detect architecture weakness from 'dead code' traces"
        );
    }

    #[test]
    fn test_skill_consolidation_detection() {
        let mut s = make_scheduler_small();
        for i in 0..5 {
            s.record_trace(format!(
                "mutation successful: crystallized skill variant {}",
                i
            ));
        }

        let new = s.run_dream_cycle(10);
        let has_skill = new
            .iter()
            .any(|p| p.category == DreamCategory::SkillConsolidation);
        assert!(
            has_skill,
            "should detect skill consolidation from mutation traces"
        );
    }

    #[test]
    fn test_behavioral_insight_detection() {
        let mut s = make_scheduler_small();
        for i in 0..5 {
            s.record_trace(format!("intervention pattern observed at cycle {}", i));
        }

        let new = s.run_dream_cycle(10);
        let has_behavioral = new
            .iter()
            .any(|p| p.category == DreamCategory::BehavioralInsight);
        assert!(
            has_behavioral,
            "should detect behavioral insight from intervention traces"
        );
    }

    #[test]
    fn test_max_patterns_respected() {
        let cfg = DreamConfig {
            interval_cycles: 1,
            min_traces_for_pattern: 2,
            max_patterns: 3,
            confidence_threshold: 0.1,
        };
        let mut s = DreamCycleScheduler::with_config(cfg);

        // Push enough traces for many categories across multiple cycles
        for _ in 0..2 {
            s.record_trace("panic error after restart".into());
        }
        let _ = s.run_dream_cycle(5);

        for _ in 0..2 {
            s.record_trace("slow bottleneck in data path".into());
        }
        let _ = s.run_dream_cycle(10);

        for _ in 0..2 {
            s.record_trace("unknown endpoint not implemented".into());
        }
        let _ = s.run_dream_cycle(15);

        for _ in 0..2 {
            s.record_trace("dead code found in legacy module".into());
        }
        let _ = s.run_dream_cycle(20);

        assert!(
            s.synthesized_patterns.len() <= 3,
            "should not exceed max_patterns=3, got {}",
            s.synthesized_patterns.len()
        );
    }

    #[test]
    fn test_new_returns_valid() {
        let s = make_scheduler();
        assert_eq!(s.trace_buffer.len(), 0);
        assert_eq!(s.synthesized_patterns.len(), 0);
        assert_eq!(s.dream_count, 0);
        assert_eq!(s.next_id, 1);
    }

    #[test]
    fn test_run_dream_no_traces() {
        let mut s = make_scheduler_small();
        let new = s.run_dream_cycle(10);
        assert!(new.is_empty(), "no traces should yield no patterns");
        assert_eq!(s.dream_count, 1);
    }
}
