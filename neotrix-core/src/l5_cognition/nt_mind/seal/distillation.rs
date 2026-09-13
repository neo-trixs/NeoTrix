#![allow(dead_code)]

use std::collections::HashMap;

/// 执行轨迹摘要（蒸馏输入）
#[derive(Debug, Clone)]
pub struct _TraceSummary {
    /// 轨迹唯一标识
    pub trace_id: String,
    /// 任务描述
    pub task_description: String,
    /// 执行步骤摘要
    pub steps: Vec<_StepSummary>,
    /// 轨迹是否成功
    pub success: bool,
    /// 总 token 消耗
    pub total_tokens: u32,
}

/// 单步执行摘要
#[derive(Debug, Clone)]
pub struct _StepSummary {
    /// 使用的技能名称
    pub skill_name: String,
    /// 输入模式摘要
    pub input_pattern: String,
    /// 输出模式摘要
    pub output_pattern: String,
    /// 步骤是否成功
    pub success: bool,
    /// 错误模式（失败时填充）
    pub error_pattern: Option<String>,
}

/// 蒸馏出的技能模板
#[derive(Debug, Clone)]
pub struct SkillTemplate {
    /// 模板唯一标识
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 输入模式
    pub input_pattern: String,
    /// 输出模式
    pub output_pattern: String,
    /// 所需技能列表
    pub required_skills: Vec<String>,
    /// 成功率（0.0-1.0）
    pub success_rate: f64,
    /// 来源轨迹 ID 列表
    pub source_traces: Vec<String>,
    /// 创建时间戳
    pub created_at: i64,
}

/// 失败反模式（避免重复犯错）
#[derive(Debug, Clone)]
pub struct _FailureAntiPattern {
    /// 失败模式描述
    pub pattern: String,
    /// 错误签名（用于快速匹配）
    pub error_signature: String,
    /// 出现次数
    pub occurrences: u32,
    /// 避免策略
    pub avoidance_strategy: String,
}

/// 模式差异（成功 vs 失败对比）
#[derive(Debug, Clone)]
struct PatternDiff {
    /// 技能名称
    skill_name: String,
    /// 成功模式
    success_pattern: String,
    /// 失败模式
    failure_pattern: String,
    /// 偏离分数（0.0-1.0，越大差异越大）
    divergence_score: f64,
}

/// 蒸馏结果摘要
#[derive(Debug, Clone)]
pub struct DistillationResult {
    /// 新生成的模板数量
    pub new_templates: usize,
    /// 新生成的反模式数量
    pub new_anti_patterns: usize,
    /// 分析的轨迹数量
    pub traces_analyzed: usize,
    /// 整体偏离分数
    pub divergence_score: f64,
}

/// 蒸馏引擎 — 从执行轨迹中提取可复用技能模板
///
/// 核心思路：对比成功轨迹与失败轨迹的步骤模式，
/// 提取成功特征作为模板，提取失败特征作为反模式。
#[derive(Debug)]
pub struct DistillationEngine {
    /// 已生成的技能模板
    templates: Vec<SkillTemplate>,
    /// 已提取的失败反模式
    anti_patterns: Vec<_FailureAntiPattern>,
    /// 待处理的轨迹摘要
    trace_summaries: Vec<_TraceSummary>,
    /// 下一个模板 ID 计数器
    next_template_id: u32,
}

impl DistillationEngine {
    /// 创建新的蒸馏引擎实例
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            anti_patterns: Vec::new(),
            trace_summaries: Vec::new(),
            next_template_id: 1,
        }
    }

    /// 添加执行轨迹摘要到待处理队列
    pub(crate) fn _add_trace(&mut self, summary: _TraceSummary) {
        self.trace_summaries.push(summary);
    }

    /// 运行蒸馏 — 对比成功/失败轨迹，提取模式
    ///
    /// 流程：分离成功/失败轨迹 → 对比模式差异 → 生成模板 + 反模式
    pub fn distill(&mut self) -> DistillationResult {
        let traces: Vec<_TraceSummary> = self.trace_summaries.drain(..).collect();
        let traces_analyzed = traces.len();

        let success_traces: Vec<&_TraceSummary> = traces.iter().filter(|t| t.success).collect();
        let failure_traces: Vec<&_TraceSummary> = traces.iter().filter(|t| !t.success).collect();

        let diffs = self.compare_traces_owned(&success_traces, &failure_traces);

        let avg_divergence = if diffs.is_empty() {
            0.0
        } else {
            diffs.iter().map(|d| d.divergence_score).sum::<f64>() / diffs.len() as f64
        };

        let new_templates = self.generate_templates(&diffs);

        let failure_refs: Vec<&_TraceSummary> = failure_traces;
        let new_anti_patterns = self.extract_anti_patterns(&failure_refs);

        DistillationResult {
            new_templates,
            new_anti_patterns,
            traces_analyzed,
            divergence_score: avg_divergence,
        }
    }

    /// 比较成功与失败轨迹，找出差异模式
    fn compare_traces_owned(
        &self,
        success: &[&_TraceSummary],
        failure: &[&_TraceSummary],
    ) -> Vec<PatternDiff> {
        let mut success_skill_map: HashMap<String, Vec<&_StepSummary>> = HashMap::new();
        for trace in success {
            for step in &trace.steps {
                success_skill_map
                    .entry(step.skill_name.clone())
                    .or_default()
                    .push(step);
            }
        }

        let mut failure_skill_map: HashMap<String, Vec<&_StepSummary>> = HashMap::new();
        for trace in failure {
            for step in &trace.steps {
                failure_skill_map
                    .entry(step.skill_name.clone())
                    .or_default()
                    .push(step);
            }
        }

        let mut diffs = Vec::new();
        for (skill, success_steps) in &success_skill_map {
            if let Some(failure_steps) = failure_skill_map.get(skill) {
                let success_patterns: Vec<&str> = success_steps
                    .iter()
                    .map(|s| s.input_pattern.as_str())
                    .collect();
                let failure_patterns: Vec<&str> = failure_steps
                    .iter()
                    .map(|s| s.input_pattern.as_str())
                    .collect();

                let divergence = compute_pattern_divergence(&success_patterns, &failure_patterns);

                let avg_success_output = average_output_pattern(success_steps);
                let avg_failure_output = average_output_pattern(failure_steps);

                diffs.push(PatternDiff {
                    skill_name: skill.clone(),
                    success_pattern: avg_success_output,
                    failure_pattern: avg_failure_output,
                    divergence_score: divergence,
                });
            }
        }

        diffs.sort_by(|a, b| b.divergence_score.partial_cmp(&a.divergence_score).unwrap());
        diffs
    }

    /// 从差异模式生成技能模板
    fn generate_templates(&mut self, diffs: &[PatternDiff]) -> usize {
        let mut count = 0;
        let now = timestamp_now();

        for diff in diffs {
            if diff.divergence_score < 0.2 {
                continue;
            }

            let template = SkillTemplate {
                id: format!("tpl_{}", self.next_template_id),
                name: format!("skill_{}", diff.skill_name),
                description: format!(
                    "从成功轨迹蒸馏的 {} 模板（偏离分: {:.2}）",
                    diff.skill_name, diff.divergence_score
                ),
                input_pattern: diff.success_pattern.clone(),
                output_pattern: diff.success_pattern.clone(),
                required_skills: vec![diff.skill_name.clone()],
                success_rate: 1.0 - diff.divergence_score,
                source_traces: Vec::new(),
                created_at: now,
            };

            self.templates.push(template);
            self.next_template_id += 1;
            count += 1;
        }

        count
    }

    /// 从失败轨迹提取反模式
    fn extract_anti_patterns(&mut self, failure: &[&_TraceSummary]) -> usize {
        let mut error_groups: HashMap<String, Vec<&str>> = HashMap::new();

        for trace in failure {
            for step in &trace.steps {
                if let Some(ref err) = step.error_pattern {
                    error_groups
                        .entry(err.clone())
                        .or_default()
                        .push(&step.skill_name);
                }
            }
        }

        let mut count = 0;
        let mut seen = std::collections::HashSet::new();

        for (error_sig, skills) in &error_groups {
            if seen.contains(error_sig) {
                continue;
            }
            seen.insert(error_sig.clone());

            let occurrences = skills.len() as u32;
            let skills_involved: Vec<String> = skills.iter().map(|s| s.to_string()).collect();

            let anti_pattern = _FailureAntiPattern {
                pattern: format!("技能 {:?} 执行时出现错误: {}", skills_involved, error_sig),
                error_signature: error_sig.clone(),
                occurrences,
                avoidance_strategy: format!(
                    "在 {:?} 技能中避免以下错误模式: {}",
                    skills_involved, error_sig
                ),
            };

            self.anti_patterns.push(anti_pattern);
            count += 1;
        }

        count
    }

    /// 获取所有已生成的模板
    pub fn templates(&self) -> &[SkillTemplate] {
        &self.templates
    }

    /// 获取所有反模式
    pub fn anti_patterns(&self) -> &[_FailureAntiPattern] {
        &self.anti_patterns
    }

    /// 清理旧轨迹，仅保留最近 N 条
    pub(crate) fn _prune_traces(&mut self, keep_recent: usize) {
        let len = self.trace_summaries.len();
        if len > keep_recent {
            let drain_start = len - keep_recent;
            self.trace_summaries.drain(..drain_start);
        }
    }
}

/// 计算两组模式的偏离分数（基于集合差异的简单实现）
fn compute_pattern_divergence(success: &[&str], failure: &[&str]) -> f64 {
    if success.is_empty() && failure.is_empty() {
        return 0.0;
    }

    let success_set: std::collections::HashSet<&str> = success.iter().copied().collect();
    let failure_set: std::collections::HashSet<&str> = failure.iter().copied().collect();

    let intersection = success_set.intersection(&failure_set).count();
    let union = success_set.union(&failure_set).count();

    if union == 0 {
        return 0.0;
    }

    1.0 - (intersection as f64 / union as f64)
}

/// 提取步骤列表中出现频率最高的 output_pattern 作为代表
fn average_output_pattern(steps: &[&_StepSummary]) -> String {
    if steps.is_empty() {
        return String::new();
    }

    let mut freq: HashMap<&str, u32> = HashMap::new();
    for step in steps {
        *freq.entry(&step.output_pattern).or_insert(0) += 1;
    }

    freq.into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(pattern, _)| pattern.to_string())
        .unwrap_or_default()
}

/// 获取当前时间戳（秒级）
fn timestamp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distillation_engine_new() {
        let engine = DistillationEngine::new();
        assert!(engine.templates.is_empty());
        assert!(engine.anti_patterns.is_empty());
        assert!(engine.trace_summaries.is_empty());
    }

    #[test]
    fn test_add_and_prune_traces() {
        let mut engine = DistillationEngine::new();
        for i in 0..10 {
            engine._add_trace(_TraceSummary {
                trace_id: format!("t{}", i),
                task_description: "task".into(),
                steps: vec![],
                success: true,
                total_tokens: 100,
            });
        }
        assert_eq!(engine.trace_summaries.len(), 10);

        engine._prune_traces(3);
        assert_eq!(engine.trace_summaries.len(), 3);
        assert_eq!(engine.trace_summaries[0].trace_id, "t7");
    }

    #[test]
    fn test_distill_with_traces() {
        let mut engine = DistillationEngine::new();

        let success_trace = _TraceSummary {
            trace_id: "s1".into(),
            task_description: "测试成功".into(),
            steps: vec![_StepSummary {
                skill_name: "code_gen".into(),
                input_pattern: "函数定义".into(),
                output_pattern: "完整实现".into(),
                success: true,
                error_pattern: None,
            }],
            success: true,
            total_tokens: 200,
        };

        let failure_trace = _TraceSummary {
            trace_id: "f1".into(),
            task_description: "测试失败".into(),
            steps: vec![_StepSummary {
                skill_name: "code_gen".into(),
                input_pattern: "类型注解".into(),
                output_pattern: "编译错误".into(),
                success: false,
                error_pattern: Some("borrow_checker".into()),
            }],
            success: false,
            total_tokens: 150,
        };

        engine._add_trace(success_trace);
        engine._add_trace(failure_trace);

        let result = engine.distill();
        assert_eq!(result.traces_analyzed, 2);
        assert!(result.divergence_score > 0.0);
    }

    #[test]
    fn test_divergence_computation() {
        let s = vec!["a", "b", "c"];
        let f = vec!["b", "c", "d"];

        let score =
            compute_pattern_divergence(&s.iter().copied().collect(), &f.iter().copied().collect());

        assert!(score > 0.0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_empty_traces_distill() {
        let mut engine = DistillationEngine::new();
        let result = engine.distill();
        assert_eq!(result.new_templates, 0);
        assert_eq!(result.new_anti_patterns, 0);
        assert_eq!(result.traces_analyzed, 0);
    }
}
