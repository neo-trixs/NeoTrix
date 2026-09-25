//! Dispatcher reduce — Claim/Reduce/分级/Error（纯搬移，行为零变更）。

use super::nt_dispatcher_dto::{DecompositionResult, SubTask, SubTaskResult};
use crate::l5_cognition::nt_core::nt_crt::CrtTimeScale;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── 确定性 Reducer (H2-H6): 过滤 → 规范化 → 聚类 → 保留最高 → 标注一致性 ──
// 关注点分离: 代码做数据工程, 模型只做推理综合。零 LLM 调用, 零新依赖。

/// 聚类后的主张组: 组内保留最高分版本, 标注一致性 (成员数)。
#[derive(Debug, Clone)]
pub struct ClaimGroup {
    /// 组内最高分代表版本 (原始输出)
    pub representative: String,
    /// 组内成员 sub_task_id (溯源 keys)
    pub members: Vec<String>,
    /// 一致性: 多个 worker 独立确认数 (≥1)
    pub consensus: usize,
    /// 组内最佳质量分 (输出长度对数, tokens 未填时的代理)
    pub best_score: f64,
}

/// Reducer 全量报告 (L1 可观测): 过滤原因 + 合并统计。
#[derive(Debug, Clone, Default)]
pub struct ReduceReport {
    /// 输入原始条目数
    pub raw: usize,
    /// 因失败过滤
    pub filtered_failed: usize,
    /// 因 malformed (空/过短) 过滤
    pub filtered_malformed: usize,
    /// 聚类结果 (每簇一条代表 + consensus 标注)
    pub clusters: Vec<ClaimGroup>,
    /// 被合并的重复主张数 = 有效输入 - 簇数
    pub deduped: usize,
}

impl ReduceReport {
    /// 压缩率 0.0-1.0: 多少原始条目被确定性压缩
    pub fn compression_ratio(&self) -> f64 {
        if self.raw == 0 {
            0.0
        } else {
            self.clusters.len() as f64 / self.raw as f64
        }
    }
}

/// 轻量文本规范化 (std-only): 小写 + 非字母数字剥离 + 空白折叠。
fn lightweight_normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 高信号词: 去停用词后按词频降序取 top 12。
fn high_signal_words(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "a", "an", "and", "or", "of", "to", "in", "on", "for", "with", "as", "by", "is",
        "are", "was", "were", "be", "been", "that", "this", "these", "those", "it", "its", "from",
        "at", "into", "between", "about", "which", "will", "should", "can", "could", "would",
        "not", "no", "yes", "but", "if", "then", "so", "we", "you", "they", "he", "she", "our",
        "your", "their", "the", "的", "了", "是", "在", "和", "与", "及", "或", "对", "为", "从",
        "有", "被", "用", "于", "个", "也", "这", "那", "我", "你", "他", "她", "它", "们", "不",
    ];
    let mut counts: HashMap<String, usize> = HashMap::new();
    for w in text.split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric());
        if w.is_empty() || w.chars().count() < 2 || STOP.contains(&w) {
            continue;
        }
        *counts.entry(w.to_string()).or_insert(0) += 1;
    }
    let mut words: Vec<(String, usize)> = counts.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    words.into_iter().take(12).map(|(w, _)| w).collect()
}

/// 两词集 Jaccard 相似度 [0,1]。
fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.iter().filter(|w| b.contains(w)).count();
    inter as f64 / (a.len() + b.len() - inter) as f64
}

/// 确定性 Reducer 主入口: 过滤 → 规范化 → 聚类 → 组内保留最高分。
///
/// 对齐文章 H2-H6: malformed 过滤 / normalize 分组 / 组内保留最高 confidence /
/// 一致性 (consensus) 显式标注。纯函数, 无 LLM 调用。
pub fn reduce_subtask_results(results: &[SubTaskResult]) -> ReduceReport {
    let mut report = ReduceReport {
        raw: results.len(),
        ..Default::default()
    };
    let mut claims: Vec<(String, String, f64)> = Vec::new();

    for r in results {
        if !r.success {
            report.filtered_failed += 1;
            continue;
        }
        let out = r.output.trim();
        if out.is_empty() || out.chars().count() < 32 {
            report.filtered_malformed += 1;
            continue;
        }
        let score = (out.chars().count() as f64).ln();
        claims.push((out.to_string(), r.sub_task_id.clone(), score));
    }

    for (raw, key, score) in claims {
        let kws = high_signal_words(&lightweight_normalize(&raw));
        let mut best: Option<usize> = None;
        let mut best_j = 0.0;
        for (i, g) in report.clusters.iter().enumerate() {
            let gkws = high_signal_words(&lightweight_normalize(&g.representative));
            let j = jaccard(&kws, &gkws);
            if j >= 0.5 && j > best_j {
                best = Some(i);
                best_j = j;
            }
        }
        match best {
            Some(i) => {
                let g = &mut report.clusters[i];
                if score > g.best_score {
                    g.best_score = score;
                    g.representative = raw;
                }
                g.members.push(key);
                g.consensus += 1;
            }
            None => {
                report.clusters.push(ClaimGroup {
                    representative: raw,
                    members: vec![key],
                    consensus: 1,
                    best_score: score,
                });
            }
        }
    }
    report.deduped = report
        .clusters
        .iter()
        .map(|c| c.members.len().saturating_sub(1))
        .sum();
    report
}

/// 把 Reducer 报告格式化为聚合 LLM 的结构化输入信号 (H5: 一致性/矛盾一等公民)。
pub fn format_reducer_signals(report: &ReduceReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Reducer summary: raw={}, filtered_failed={}, filtered_malformed={}, clusters={}, compression={:.2}\n",
        report.raw,
        report.filtered_failed,
        report.filtered_malformed,
        report.clusters.len(),
        report.compression_ratio()
    ));
    for (i, g) in report.clusters.iter().enumerate() {
        let tag = if g.consensus >= 2 {
            format!("<CONSENSUS n={}>", g.consensus)
        } else {
            "<SINGLE>".to_string()
        };
        out.push_str(&format!("{}. {} {}\n", i + 1, tag, g.representative));
    }
    out
}

// ── 调度头分级 (H7-H9): 拆解打标 scalar → 确定性走代码, 推理走 LLM ──

/// 子任务分级: 确定性 (可结构化处理) vs 需要智能推理。
/// 由 `required_capabilities` 与 prompt 特征派生, 不改 SubTask schema。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubTaskClass {
    /// 确定性任务: 走 kernel/代码路径, 不耗费 LLM 推理预算
    Deterministic,
    /// 需要智能推理: 走 CoT/ReasoningEngine/LLM 链
    Reasoning,
}

/// 从子任务派生分级 (H7 打标)。
/// 推理特征: 深度分析/设计/写作/研究/代码生成 能力, 或长 prompt 隐含复杂度。
/// 确定性特征: 结构化/机械化能力 (testing/verification) + 短 prompt。
pub fn classify_sub_task(sub_task: &SubTask) -> SubTaskClass {
    let caps: Vec<&str> = sub_task
        .required_capabilities
        .iter()
        .map(|s| s.as_str())
        .collect();
    let reasoning_caps = [
        "code_generation",
        "design",
        "analysis",
        "research",
        "writing",
    ];
    let deterministic_caps = ["testing", "verification"];

    if caps.iter().any(|c| reasoning_caps.contains(c)) {
        return SubTaskClass::Reasoning;
    }
    if caps.iter().any(|c| deterministic_caps.contains(c)) && sub_task.prompt.len() < 200 {
        return SubTaskClass::Deterministic;
    }
    // 兜底: 长 prompt / 战略尺度 = 推理
    if sub_task.prompt.len() > 300 || matches!(sub_task.crt_scale, CrtTimeScale::Xuanye) {
        SubTaskClass::Reasoning
    } else {
        SubTaskClass::Deterministic
    }
}

/// 格式化拆解计划 (含每子任务分级), 供调度报告消费 (H7 显式化 + H10 可观测)。
pub fn format_dispatch_plan(decomposition: &DecompositionResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Dispatch plan: {} subtasks, order=[{}]\n",
        decomposition.sub_tasks.len(),
        decomposition.execution_order.join(",")
    ));
    for st in &decomposition.sub_tasks {
        let class = match classify_sub_task(st) {
            SubTaskClass::Deterministic => "DET",
            SubTaskClass::Reasoning => "RES",
        };
        out.push_str(&format!(
            "  [{}] {} (caps={}) deps=[{}]\n",
            class,
            st.title,
            st.required_capabilities.join("+"),
            st.dependencies.join(",")
        ));
    }
    out
}

/// 调度错误类型
#[derive(Debug, thiserror::Error)]
pub enum TaskDispatchError {
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("CoT error: {0}")]
    CotError(String),
    #[error("Reasoning error: {0}")]
    ReasoningError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_core_task_dispatcher::nt_dispatcher_dto::{DecompositionResult, SubTask};
    use crate::l5_cognition::nt_core::nt_crt::{CrtPlan, CrtTimeScale};
    use std::collections::HashMap;

    fn result(id: &str, success: bool, output: &str) -> SubTaskResult {
        SubTaskResult {
            sub_task_id: id.to_string(),
            success,
            output: output.to_string(),
            error: None,
            tokens_used: 0,
            duration_ms: 1,
            cot_output: None,
        }
    }

    #[test]
    fn test_reducer_happy_path_keeps_all() {
        let results = vec![result(
            "a",
            true,
            "The system uses a vector database for retrieval.",
        )];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.raw, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].consensus, 1);
        assert_eq!(report.filtered_failed, 0);
        assert_eq!(report.filtered_malformed, 0);
    }

    #[test]
    fn test_reducer_all_empty_filters_malformed() {
        let results = vec![result("a", true, ""), result("b", true, "short")];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_malformed, 2);
        assert!(report.clusters.is_empty());
        assert_eq!(report.raw, 2);
    }

    #[test]
    fn test_reducer_failed_filtered() {
        let results = vec![
            result("a", false, "should be dropped"),
            result(
                "b",
                true,
                "active finding that is long enough to keep as a valid claim",
            ),
        ];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_failed, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].consensus, 1);
    }

    #[test]
    fn test_reducer_duplicate_merged_with_consensus() {
        let r0 = result("w1", true, "The architecture uses a vector symbolic database for semantic retrieval of knowledge entries");
        let r1 = result("w2", true, "The architecture uses vector symbolic storage for knowledge retrieval with semantic lookup");
        let report = reduce_subtask_results(&[r0, r1]);
        assert_eq!(report.raw, 2);
        assert_eq!(
            report.deduped, 1,
            "two near-dup claims merge into one cluster"
        );
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(
            report.clusters[0].consensus, 2,
            "independent workers confirming same claim"
        );
    }

    #[test]
    fn test_reducer_short_output_filtered_while_valid_kept() {
        let valid = "A detailed finding about the reducer design that fully satisfies the minimum length requirement";
        let results = vec![result("a", true, "short"), result("b", true, valid)];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_malformed, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].representative, valid);
    }

    #[test]
    fn test_reducer_signals_format_consensus() {
        let r0 = result("w1", true, "The architecture uses a vector symbolic database for semantic retrieval of knowledge entries");
        let r1 = result("w2", true, "The architecture uses vector symbolic storage for knowledge retrieval with semantic lookup");
        let report = reduce_subtask_results(&[r0, r1]);
        let signals = format_reducer_signals(&report);
        assert!(
            signals.contains("<CONSENSUS n=2>"),
            "consensus marker for multi-worker confirmation"
        );
        assert!(signals.contains("clusters=1"));
    }

    fn sub_task(id: &str, title: &str, caps: Vec<&str>, prompt_len: usize) -> SubTask {
        SubTask {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            prompt: "x".repeat(prompt_len),
            context: HashMap::new(),
            priority: 5,
            estimated_complexity: 0.5,
            required_capabilities: caps.into_iter().map(|s| s.to_string()).collect(),
            dependencies: Vec::new(),
            crt_scale: CrtTimeScale::Gaitian,
            hexagram_bias: Some(3),
        }
    }

    #[test]
    fn test_classify_reasoning_capability() {
        let st = sub_task("s1", "Design the system architecture", vec!["design"], 80);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Reasoning);
    }

    #[test]
    fn test_classify_deterministic_verification() {
        let st = sub_task(
            "s2",
            "Verify output passes format check",
            vec!["testing"],
            100,
        );
        assert_eq!(classify_sub_task(&st), SubTaskClass::Deterministic);
    }

    #[test]
    fn test_classify_long_prompt_falls_back_reasoning() {
        let st = sub_task("s3", "Generic task", vec!["general"], 350);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Reasoning);
    }

    #[test]
    fn test_classify_default_deterministic() {
        let st = sub_task("s4", "Generic task", vec!["general"], 50);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Deterministic);
    }

    #[test]
    fn test_format_dispatch_plan_report() {
        let order = vec!["s2".to_string(), "s1".to_string()];
        let plan = DecompositionResult {
            original_task: "Build a system".to_string(),
            sub_tasks: vec![
                sub_task("s2", "Verify output", vec!["testing"], 100),
                sub_task("s1", "Design architecture", vec!["design"], 80),
            ],
            execution_order: order,
            crt_plan: CrtPlan::new(CrtTimeScale::Huntian, 120.0),
            estimated_total_time: 60.0,
            confidence: 0.7,
        };
        let report = format_dispatch_plan(&plan);
        assert!(report.contains("DET"), "deterministic subtask tagged");
        assert!(report.contains("RES"), "reasoning subtask tagged");
        assert!(report.contains("order=[s2,s1]"));
    }
}
