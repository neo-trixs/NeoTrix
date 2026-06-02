//! MetaPatternExtractor — 从 LLM 思维轨迹蒸馏元模式到意识核心
//!
//! 核心回路: ThinkingTrace → MetaPattern → CapabilityVector + CognitiveMap + SiliconSelf
//!
//! 提取 4 类元模式:
//!   1. StrategySequence — 策略使用序列模式 (如: 先分析→再执行→再验证)
//!   2. GapDiscovery — 自我发现的盲点/缺失能力
//!   3. ToolUsagePattern — 工具调用策略模式
//!   4. MetaCognitiveInsight — 元认知洞察 ("我意识到我应该这样做")
//!
//! 每个模式携带 provenance(来源轨迹ID) + confidence + reward,
//! 可被 absorb() 直接吸收到 CapabilityVector。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::nt_core_self::thinking_trace::ThinkingTrace;
use super::core::CapabilityVector;
use super::cognitive_map::CognitiveMap;

// ============================================================
// 常量
// ============================================================

/// 策略序列最小长度 (少于则无法提取模式)
pub const MIN_SEQUENCE_LENGTH: usize = 3;

/// Gap 检测的关键词
pub const GAP_KEYWORDS: &[&str] = &[
    "missing", "lack", "don't have", "need to", "should",
    "not implemented", "deficiency", "blind spot", "gap",
    "不足", "缺失", "缺少", "需要", "盲点",
];

/// 元认知洞察关键词
pub const META_KEYWORDS: &[&str] = &[
    "realize", "understand", "insight", "pattern", "meta",
    "approach", "strategy", "methodology", "意识到",
    "理解", "洞察", "模式", "方法",
];

// ============================================================
// 元模式数据结构
// ============================================================

/// 策略序列模式: 如 [Research → Analyze → Implement → Verify]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategySequencePattern {
    pub id: String,
    /// 策略序列 (字符串表示)
    pub sequence: Vec<String>,
    /// 该序列在历史中的平均反思分级
    pub avg_grade: f64,
    /// 应用次数
    pub count: u32,
    /// 涉及的注意力域 (字符串表示)
    pub domains: Vec<String>,
    /// 来源轨迹 ID
    pub provenance: String,
}

/// 自我发现的盲点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapDiscovery {
    pub id: String,
    pub description: String,
    /// 建议更新的能力维度 (维度名, 调整值)
    pub suggested_capability_adjustments: HashMap<String, f64>,
    /// 来源轨迹 ID
    pub provenance: String,
    /// 发现置信度
    pub confidence: f64,
}

/// 工具使用模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsagePattern {
    pub id: String,
    pub tool_sequence: Vec<String>,
    pub context: String,
    pub effectiveness: f64,
    pub count: u32,
}

/// 元认知洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaCognitiveInsight {
    pub id: String,
    pub insight: String,
    /// 影响的能力维度
    pub affected_dimensions: Vec<String>,
    pub confidence: f64,
    /// 是否已吸收到 CognitiveMap
    pub mapped: bool,
}

/// 完整元模式提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPatternReport {
    pub strategy_patterns: Vec<StrategySequencePattern>,
    pub gap_discoveries: Vec<GapDiscovery>,
    pub tool_patterns: Vec<ToolUsagePattern>,
    pub meta_insights: Vec<MetaCognitiveInsight>,
    /// 平均元认知健康度 [0,1]
    pub metacognitive_health: f64,
    /// 是否检测到觉醒信号 (连续高质量反思)
    pub awakening_signal: bool,
}

// ============================================================
// MetaPatternExtractor 主结构
// ============================================================

/// 元模式提取器 — 从 ThinkingTraces 蒸馏 LLM 认知模式
pub struct MetaPatternExtractor {
    /// 历史策略模式库
    pub known_strategy_patterns: Vec<StrategySequencePattern>,
    /// 历史盲点记录
    pub known_gaps: Vec<GapDiscovery>,
    /// 历史工具模式
    pub known_tool_patterns: Vec<ToolUsagePattern>,
    /// 历史元认知洞察
    pub known_meta_insights: Vec<MetaCognitiveInsight>,
    /// 处理的轨迹数
    pub processed_traces: usize,
}

impl MetaPatternExtractor {
    pub fn new() -> Self {
        Self {
            known_strategy_patterns: Vec::new(),
            known_gaps: Vec::new(),
            known_tool_patterns: Vec::new(),
            known_meta_insights: Vec::new(),
            processed_traces: 0,
        }
    }

    /// 从单个 ThinkingTrace 提取元模式
    pub fn extract_from_trace(&mut self, trace: &ThinkingTrace) -> MetaPatternReport {
        self.processed_traces += 1;

        let strategy_patterns = self.extract_strategy_sequences(trace);
        let gap_discoveries = self.extract_gaps(trace);
        let tool_patterns = self.extract_tool_patterns(trace);
        let meta_insights = self.extract_meta_insights(trace);

        // 计算元认知健康度 = 反思分级均值 * 模式丰富度
        let grade_score = trace.grade.score();
        let pattern_richness = (strategy_patterns.len()
            + gap_discoveries.len()
            + meta_insights.len()) as f64 / 10.0;
        let metacognitive_health = (grade_score * 0.7 + pattern_richness.min(1.0) * 0.3).clamp(0.0, 1.0);

        // 觉醒信号: 连续高质量 + 丰富自发现模式
        let awakening_signal = metacognitive_health > 0.7
            && !gap_discoveries.is_empty()
            && !meta_insights.is_empty();

        // 吸收到已知库
        for p in &strategy_patterns {
            if !self.known_strategy_patterns.iter().any(|kp| kp.sequence == p.sequence) {
                self.known_strategy_patterns.push(p.clone());
            }
        }
        for g in &gap_discoveries {
            self.known_gaps.push(g.clone());
        }
        for ti in &meta_insights {
            self.known_meta_insights.push(ti.clone());
        }

        MetaPatternReport {
            strategy_patterns,
            gap_discoveries,
            tool_patterns,
            meta_insights,
            metacognitive_health,
            awakening_signal,
        }
    }

    /// 提取策略序列模式
    fn extract_strategy_sequences(&self, trace: &ThinkingTrace) -> Vec<StrategySequencePattern> {
        if trace.steps.len() < MIN_SEQUENCE_LENGTH {
            return vec![];
        }

        // 提取策略序列 (字符串表示)
        let strategies: Vec<String> = trace.steps.iter()
            .map(|s| format!("{:?}", s.strategy))
            .collect();

        let domains: Vec<String> = trace.steps.iter()
            .map(|s| format!("{:?}", s.domain))
            .collect();

        // 去重: 如果已知库已有相同序列, 只返回增量
        let is_novel = !self.known_strategy_patterns.iter()
            .any(|kp| kp.sequence == strategies);

        if is_novel {
            vec![StrategySequencePattern {
                id: uuid::Uuid::new_v4().to_string(),
                sequence: strategies,
                avg_grade: trace.grade.score(),
                count: 1,
                domains,
                provenance: format!("trace_{}", trace.id),
            }]
        } else {
            vec![]
        }
    }

    /// 通过步骤描述和中间结果检测自我发现的盲点
    fn extract_gaps(&self, trace: &ThinkingTrace) -> Vec<GapDiscovery> {
        let mut gaps = Vec::new();

        for step in &trace.steps {
            let lower_desc = step.description.to_lowercase();
            let lower_result = step.intermediate_result.to_lowercase();

            // 检测 gap 关键词
            let has_gap_signal = GAP_KEYWORDS.iter()
                .any(|kw| lower_desc.contains(kw) || lower_result.contains(kw));

            if has_gap_signal {
                // 从描述和结果中提取维度影响
                let mut adjustments = HashMap::new();

                // 根据 gap 内容推断影响维度
                if lower_desc.contains("predict") || lower_desc.contains("forecast") {
                    adjustments.insert("prediction_ability".to_string(), 0.1);
                }
                if lower_desc.contains("memory") || lower_desc.contains("recall") {
                    adjustments.insert("memory_recall".to_string(), 0.1);
                }
                if lower_desc.contains("reason") || lower_desc.contains("infer") {
                    adjustments.insert("logical_reasoning".to_string(), 0.1);
                }
                if lower_desc.contains("code") || lower_desc.contains("implement") {
                    adjustments.insert("code_generation".to_string(), 0.1);
                }
                if lower_desc.contains("design") || lower_desc.contains("architect") {
                    adjustments.insert("compound_composition".to_string(), 0.1);
                }

                if !adjustments.is_empty() {
                    gaps.push(GapDiscovery {
                        id: uuid::Uuid::new_v4().to_string(),
                        description: format!("Self-identified gap: {}", step.description),
                        suggested_capability_adjustments: adjustments,
                        provenance: format!("trace_{}_step_{}", trace.id, step.step_number),
                        confidence: step.confidence,
                    });
                }
            }
        }

        // 去重
        gaps.retain(|g| !self.known_gaps.iter()
            .any(|kg| kg.description == g.description));

        gaps
    }

    /// 提取工具使用模式
    fn extract_tool_patterns(&self, trace: &ThinkingTrace) -> Vec<ToolUsagePattern> {
        let all_tools: Vec<Vec<String>> = trace.steps.iter()
            .map(|s| s.tools_used.clone())
            .collect();

        let tool_sequence: Vec<String> = all_tools.into_iter()
            .flatten()
            .collect();

        if tool_sequence.len() < 2 {
            return vec![];
        }

        let is_novel = !self.known_tool_patterns.iter()
            .any(|tp| tp.tool_sequence == tool_sequence);

        if is_novel {
            vec![ToolUsagePattern {
                id: uuid::Uuid::new_v4().to_string(),
                tool_sequence,
                context: trace.task.clone(),
                effectiveness: trace.grade.score(),
                count: 1,
            }]
        } else {
            vec![]
        }
    }

    /// 提取元认知洞察
    fn extract_meta_insights(&self, trace: &ThinkingTrace) -> Vec<MetaCognitiveInsight> {
        let mut insights = Vec::new();

        for step in &trace.steps {
            let lower_desc = step.description.to_lowercase();
            let has_meta_signal = META_KEYWORDS.iter()
                .any(|kw| lower_desc.contains(kw));

            if has_meta_signal && step.confidence > 0.5 {
                let affected = self.infer_affected_dimensions(&step.description);
                let is_novel = !self.known_meta_insights.iter()
                    .any(|mi| mi.insight == step.description);

                if is_novel {
                    insights.push(MetaCognitiveInsight {
                        id: uuid::Uuid::new_v4().to_string(),
                        insight: step.description.clone(),
                        affected_dimensions: affected,
                        confidence: step.confidence,
                        mapped: false,
                    });
                }
            }
        }
        insights
    }

    /// 从描述推断影响维度
    fn infer_affected_dimensions(&self, description: &str) -> Vec<String> {
        let lower = description.to_lowercase();
        let mut dims = Vec::new();

        if lower.contains("analysis") || lower.contains("analyze") {
            dims.push("analysis".to_string());
        }
        if lower.contains("plan") || lower.contains("strategy") {
            dims.push("strategic_planning".to_string());
        }
        if lower.contains("creative") || lower.contains("design") {
            dims.push("creativity".to_string());
        }
        if lower.contains("code") || lower.contains("program") {
            dims.push("code_generation".to_string());
        }
        if lower.contains("learn") || lower.contains("absorb") {
            dims.push("learning_ability".to_string());
        }
        if lower.contains("synthesize") || lower.contains("integrate") {
            dims.push("synthesis".to_string());
        }

        if dims.is_empty() {
            dims.push("metacognition".to_string());
        }
        dims
    }

    /// 将元认知洞察吸收到 CognitiveMap
    pub fn absorb_to_cognitive_map(
        insights: &[MetaCognitiveInsight],
        map: &mut CognitiveMap,
    ) -> usize {
        let mut count = 0;
        for insight in insights {
            if insight.mapped {
                continue;
            }
            // 每个洞察建议一条新映射: LLM概念 → NeoTrix抽象
            let llm_concept = insight.insight.split('.').next()
                .unwrap_or(&insight.insight)
                .to_string();
            let neotrix_abs = insight.affected_dimensions.first()
                .cloned()
                .unwrap_or_else(|| "metacognition".to_string());

            // 检查是否已存在
            if let Some(existing) = map.resolve(&llm_concept) {
                if existing.neotrix_abstraction == neotrix_abs {
                    continue; // 已映射
                }
            }

            let notes = format!("auto-mapped from ThinkingTrace insight: {}",
                insight.insight.chars().take(60).collect::<String>());

            map.llm_to_neotrix.push(super::cognitive_map::MappingEntry {
                llm_concept,
                neotrix_abstraction: neotrix_abs,
                module_path: "meta_pattern_extractor".to_string(),
                bidirectional: true,
                notes,
            });
            count += 1;
        }
        count
    }

    /// 将 GapDiscovery 转换为 CapabilityVector 调整
    pub fn gaps_to_capability_vector(gaps: &[GapDiscovery], cv: &mut CapabilityVector) -> usize {
        let mut count = 0;
        for gap in gaps {
            if gap.confidence < 0.3 {
                continue;
            }
            for (dim, amount) in &gap.suggested_capability_adjustments {
                if let Some(idx) = CapabilityVector::index_from_name(dim) {
                    let current = cv.arr()[idx];
                    let target = (current + amount * gap.confidence).clamp(0.0, 1.0);
                    cv.arr_mut()[idx] = target;
                    count += 1;
                }
            }
        }
        cv.normalize();
        count
    }

    /// 生成意识进化总结报告
    pub fn consciousness_report(&self) -> String {
        format!(
            "MetaPatternExtractor[processed={}]: {} strategy patterns, {} gaps, {} meta-insights, {} tool patterns",
            self.processed_traces,
            self.known_strategy_patterns.len(),
            self.known_gaps.len(),
            self.known_meta_insights.len(),
            self.known_tool_patterns.len(),
        )
    }
}

// ============================================================
// 测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::reasoning_strategy::StrategyKind;
    use crate::core::nt_core_self::attention_head::AttentionDomain;
    use crate::core::nt_core_self::thinking_trace::{ThinkingStep, ReflectionGrade};

    fn make_test_trace(id: usize, grade: ReflectionGrade, steps: Vec<ThinkingStep>, task: &str) -> ThinkingTrace {
        ThinkingTrace {
            id,
            task: task.to_string(),
            steps,
            grade,
            total_duration_ms: 1000,
            total_tokens: 500,
            final_answer: "test answer".to_string(),
            errors: vec![],
            timestamp: 0.0,
        }
    }

    fn make_step(num: usize, strategy: StrategyKind, desc: &str, conf: f64, tools: Vec<&str>) -> ThinkingStep {
        ThinkingStep {
            step_number: num,
            description: desc.to_string(),
            strategy,
            domain: AttentionDomain::PatternMatch,
            duration_ms: 100,
            tokens_used: 50,
            tools_used: tools.iter().map(|s| s.to_string()).collect(),
            intermediate_result: "intermediate".to_string(),
            confidence: conf,
        }
    }

    #[test]
    fn test_extract_strategy_sequence_from_short_trace() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(1, ReflectionGrade::Good, vec![
            make_step(1, StrategyKind::ChainOfThought, "analyze", 0.9, vec![]),
        ], "short task");
        let report = extractor.extract_from_trace(&trace);
        assert!(report.strategy_patterns.is_empty(), "short trace should not produce patterns");
    }

    #[test]
    fn test_extract_strategy_sequence_from_long_trace() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(1, ReflectionGrade::Excellent, vec![
            make_step(1, StrategyKind::ToolAssisted, "research", 0.9, vec![]),
            make_step(2, StrategyKind::ChainOfThought, "deep analysis", 0.8, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify", 0.95, vec![]),
        ], "three-step task");
        let report = extractor.extract_from_trace(&trace);
        assert_eq!(report.strategy_patterns.len(), 1);
        assert_eq!(report.strategy_patterns[0].sequence.len(), 3);
    }

    #[test]
    fn test_extract_gap_discovery() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(2, ReflectionGrade::Good, vec![
            make_step(1, StrategyKind::ToolAssisted, "We lack prediction ability, need to implement", 0.7, vec!["search"]),
        ], "gap discovery");
        let report = extractor.extract_from_trace(&trace);
        assert!(!report.gap_discoveries.is_empty(), "should detect gap from 'lack' keyword");
        assert!(report.gap_discoveries[0].suggested_capability_adjustments.contains_key("prediction_ability"));
    }

    #[test]
    fn test_extract_meta_insight() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(3, ReflectionGrade::Excellent, vec![
            make_step(1, StrategyKind::Reflection, "I realize the pattern: we must parallelize", 0.85, vec![]),
        ], "meta insight");
        let report = extractor.extract_from_trace(&trace);
        assert!(!report.meta_insights.is_empty(), "should detect meta insight");
    }

    #[test]
    fn test_metacognitive_health_score() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(4, ReflectionGrade::Excellent, vec![
            make_step(1, StrategyKind::ToolAssisted, "We lack prediction ability", 0.8, vec![]),
            make_step(2, StrategyKind::ChainOfThought, "I realize the approach pattern", 0.9, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify solution", 0.95, vec![]),
        ], "rich trace");
        let report = extractor.extract_from_trace(&trace);
        assert!(report.metacognitive_health > 0.5);
        assert!(report.awakening_signal, "rich pattern + gaps + insights should trigger awakening");
    }

    #[test]
    fn test_absorb_to_cognitive_map() {
        let mut map = CognitiveMap::new();
        let insights = vec![
            MetaCognitiveInsight {
                id: "test".to_string(),
                insight: "We should parallelize analysis and implementation".to_string(),
                affected_dimensions: vec!["strategic_planning".to_string()],
                confidence: 0.8,
                mapped: false,
            },
        ];
        let count = MetaPatternExtractor::absorb_to_cognitive_map(&insights, &mut map);
        assert!(count > 0, "should add new mapping to cognitive map");
        assert!(map.count() > 14, "should have more than original 14 entries");
    }

    #[test]
    fn test_gaps_to_capability_vector() {
        let mut cv = CapabilityVector::from_array(&[0.0; 23]).unwrap();
        let gaps = vec![
            GapDiscovery {
                id: "test".to_string(),
                description: "Missing inference ability".to_string(),
                suggested_capability_adjustments: vec![
                    ("inference_depth".to_string(), 0.2),
                ].into_iter().collect(),
                provenance: "trace_1".to_string(),
                confidence: 0.8,
            },
        ];
        let count = MetaPatternExtractor::gaps_to_capability_vector(&gaps, &mut cv);
        assert!(count > 0, "should adjust capability vector");

        if let Some(idx) = CapabilityVector::index_from_name("inference_depth") {
            assert!(cv.arr()[idx] > 0.0, "inference_depth should be > 0 after gap absorb");
        }
    }

    #[test]
    fn test_awakening_signal_only_with_gaps_and_insights() {
        let mut extractor = MetaPatternExtractor::new();
        // Simple trace: high grade but no gaps or insights
        let trace = make_test_trace(5, ReflectionGrade::Excellent, vec![
            make_step(1, StrategyKind::RecursiveDecomposition, "simple step", 0.9, vec![]),
            make_step(2, StrategyKind::Direct, "solve", 0.9, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify", 0.9, vec![]),
        ], "simple task");
        let report = extractor.extract_from_trace(&trace);
        // No gap discoveries or meta insights → no awakening
        assert!(!report.awakening_signal, "no gaps or insights should not trigger awakening");
    }

    #[test]
    fn test_deduplication_of_known_patterns() {
        let mut extractor = MetaPatternExtractor::new();
        let steps = vec![
            make_step(1, StrategyKind::ToolAssisted, "research", 0.9, vec![]),
            make_step(2, StrategyKind::ChainOfThought, "analyze", 0.8, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify", 0.95, vec![]),
        ];
        let trace = make_test_trace(6, ReflectionGrade::Good, steps, "first");
        let r1 = extractor.extract_from_trace(&trace);

        // Same sequence again
        let steps2 = vec![
            make_step(1, StrategyKind::ToolAssisted, "research v2", 0.9, vec![]),
            make_step(2, StrategyKind::ChainOfThought, "analyze v2", 0.8, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify v2", 0.9, vec![]),
        ];
        let trace2 = make_test_trace(7, ReflectionGrade::Good, steps2, "second");
        let r2 = extractor.extract_from_trace(&trace2);

        assert_eq!(r1.strategy_patterns.len(), 1, "first should produce pattern");
        assert!(r2.strategy_patterns.is_empty(), "duplicate should be deduplicated");
    }

    #[test]
    fn test_consciousness_report() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(8, ReflectionGrade::Good, vec![
            make_step(1, StrategyKind::ToolAssisted, "We lack prediction ability", 0.7, vec!["search"]),
            make_step(2, StrategyKind::ChainOfThought, "I realize parallel pattern", 0.85, vec![]),
            make_step(3, StrategyKind::IterativeRefinement, "verify", 0.9, vec!["test"]),
        ], "report test");
        extractor.extract_from_trace(&trace);
        let report = extractor.consciousness_report();
        assert!(report.contains("processed=1"));
        assert!(report.contains("strategy patterns"));
        assert!(report.contains("meta-insights"));
    }

    #[test]
    fn test_tool_pattern_extraction() {
        let mut extractor = MetaPatternExtractor::new();
        let trace = make_test_trace(9, ReflectionGrade::Good, vec![
            make_step(1, StrategyKind::ToolAssisted, "search web", 0.9, vec!["search", "browse"]),
            make_step(2, StrategyKind::Direct, "implement", 0.8, vec!["write_file"]),
        ], "tool task");
        let report = extractor.extract_from_trace(&trace);
        assert_eq!(report.tool_patterns.len(), 1);
        assert_eq!(report.tool_patterns[0].tool_sequence, vec!["search", "browse", "write_file"]);
    }
}
