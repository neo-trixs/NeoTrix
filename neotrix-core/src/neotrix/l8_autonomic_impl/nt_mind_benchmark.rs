use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use crate::core::CapabilityVector;
use crate::neotrix::nt_world_model::TaskType;
use crate::neotrix::nt_mind::memory::ReasoningBank;
use crate::neotrix::nt_mind::ReasoningBrain;
use crate::core::nt_core_knowledge::KnowledgeSource;
use crate::core::nt_core_bank::ReasoningMemory;
use crate::neotrix::nt_io_provider::{LlmError, LlmProvider, LlmRequest, LlmResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub category: String,
    pub score: f64,
    pub max_score: f64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub results: Vec<BenchmarkResult>,
    pub overall_score: f64,
    pub timestamp: String,
    pub iteration: u64,
}

pub struct BenchmarkSuite;

impl BenchmarkSuite {
    pub fn run_all(cap: &CapabilityVector) -> BenchmarkReport {
        let mut results = vec![
            BenchmarkResult {
                name: "general_intelligence".into(),
                category: "core".into(),
                score: cap.arr().iter().sum::<f64>() / cap.arr().len() as f64,
                max_score: 1.0,
                metadata: None,
            },
            BenchmarkResult {
                name: "quality_gates".into(),
                category: "core".into(),
                score: cap.quality_gates(),
                max_score: 1.0,
                metadata: None,
            },
            BenchmarkResult {
                name: "extension_diversity".into(),
                category: "knowledge".into(),
                score: (cap.extension.len() as f64).min(22.0) / 22.0,
                max_score: 1.0,
                metadata: None,
            },
        ];

        for (name, tt) in &[
            ("design", TaskType::Design),
            ("code_analysis", TaskType::CodeAnalysis),
            ("code_review", TaskType::CodeReview),
            ("nt_shield", TaskType::Security),
        ] {
            let score = crate::neotrix::nt_mind::core::PerformanceEvaluator::evaluate(tt, cap);
            results.push(BenchmarkResult {
                name: format!("task_{}", name),
                category: "task".into(),
                score,
                max_score: 1.0,
                metadata: None,
            });
        }

        let overall = results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64;

        BenchmarkReport {
            overall_score: overall,
            timestamp: chrono::Utc::now().to_rfc3339(),
            results,
            iteration: 0,
        }
    }

    pub fn run_category(cap: &CapabilityVector, category: &str) -> Vec<BenchmarkResult> {
        let report = Self::run_all(cap);
        report.results.into_iter().filter(|r| r.category == category).collect()
    }

    pub fn run_all_extended(cap: &CapabilityVector, bank: &mut ReasoningBank) -> BenchmarkReport {
        let mut base = Self::run_all(cap);

        let knowledge = Self::run_knowledge_benchmarks(cap);
        let memory = Self::run_memory_benchmarks(bank);
        let convergence = Self::run_convergence_benchmarks(cap);

        base.results.extend(knowledge);
        base.results.extend(memory);
        base.results.extend(convergence);

        let overall = base.results.iter().map(|r| r.score / r.max_score).sum::<f64>() / base.results.len() as f64;
        base.overall_score = overall;
        base
    }

    pub fn run_knowledge_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        use KnowledgeSource::*;
        let sources = vec![
            HeroUI, BaseUI, ArcUI, CortexUI, AgenticDS, DesignPhilosophy,
            Hyperframes, Betterleaks, YaoWebsecurity, Botasaurus, ReactDoctor,
            OpenPencil, AiTrader, SesameRobot, EverOS, MattPocockSkills,
            NestedLearning, AutonomousGoal, AwesomeDesignSkills,
            DeepSeekTui, Codebuff, OpenClaude, Cairn, Orca, RedRun,
            AutonomousSpeedrunning, Synesis, MemOS, Reflexio, Mem0,
            Mnemosyne, OriMnemos, OPSD,
        ];

        let non_zero = sources.iter().filter(|s: &&KnowledgeSource| {
            s.capability_vector().arr().iter().any(|&v| v > 0.0)
        }).count();
        let coverage = non_zero as f64 / sources.len() as f64;

        let total: f64 = cap.arr().iter().sum();
        let entropy = if total > 0.0 {
            -cap.arr().iter().filter(|&&v| v > 0.0).map(|&v| {
                let p = v / total;
                p * p.log2()
            }).sum::<f64>()
        } else {
            0.0
        };
        let max_entropy = (cap.arr().len() as f64).log2();
        let diversity = if max_entropy > 0.0 { (entropy / max_entropy).min(1.0) } else { 0.0 };

        let richness = (cap.extension.len() as f64 / 50.0).min(1.0);

        vec![
            BenchmarkResult {
                name: "knowledge_coverage".into(),
                category: "knowledge".into(),
                score: coverage,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("total_sources".into(), sources.len().to_string()),
                    ("non_zero_sources".into(), non_zero.to_string()),
                ])),
            },
            BenchmarkResult {
                name: "knowledge_diversity".into(),
                category: "knowledge".into(),
                score: diversity,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("entropy".into(), format!("{:.4}", entropy)),
                    ("max_entropy".into(), format!("{:.4}", max_entropy)),
                ])),
            },
            BenchmarkResult {
                name: "extension_richness".into(),
                category: "knowledge".into(),
                score: richness,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("extension_count".into(), cap.extension.len().to_string()),
                ])),
            },
        ]
    }

    pub fn run_memory_benchmarks(bank: &mut ReasoningBank) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let memory_retention_result = {
            let count_before = bank.stats().total_memories;
            let test_memories: Vec<ReasoningMemory> = (0..5).map(|i| {
                ReasoningMemory::new(
                    &format!("benchmark_retention_test_{}", i),
                    TaskType::General,
                    &[],
                    0.8,
                )
            }).collect();

            for mem in &test_memories {
                bank.store(mem.clone());
            }

            let recalled = test_memories.iter().filter(|m| {
                let retrieved = bank.retrieve_relevant(&m.task_description, None, 10);
                retrieved.iter().any(|r| r.id == m.id)
            }).count();
            let retention = if test_memories.is_empty() { 0.0 } else { recalled as f64 / test_memories.len() as f64 };

            BenchmarkResult {
                name: "memory_retention".into(),
                category: "memory".into(),
                score: retention,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("stored".into(), test_memories.len().to_string()),
                    ("recalled".into(), recalled.to_string()),
                    ("count_before".into(), count_before.to_string()),
                ])),
            }
        };
        results.push(memory_retention_result);

        let memory_capacity_result = {
            let fill_count = 200usize;
            let mut panic_occurred = false;
            let _before_stats = bank.stats().total_memories;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                for i in 0..fill_count {
                    let mem = ReasoningMemory::new(
                        &format!("benchmark_capacity_fill_{}", i),
                        TaskType::General,
                        &[],
                        0.1,
                    );
                    bank.store(mem);
                }
                let _ = bank.stats();
            }));
            if result.is_err() {
                panic_occurred = true;
            }

            BenchmarkResult {
                name: "memory_capacity".into(),
                category: "memory".into(),
                score: if panic_occurred { 0.0 } else { 1.0 },
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("panic".into(), panic_occurred.to_string()),
                    ("fill_attempted".into(), fill_count.to_string()),
                    ("total_after".into(), bank.stats().total_memories.to_string()),
                ])),
            }
        };
        results.push(memory_capacity_result);

        let memory_retrieval_speed_result = {
            let start = Instant::now();
            let trials = 5;
            for _ in 0..trials {
                let _ = bank.retrieve_relevant("benchmark speed test query", None, 5);
            }
            let elapsed = start.elapsed();
            let avg = elapsed / trials as u32;
            let speed_score = if avg.as_micros() < 1000 {
                1.0
            } else if avg.as_micros() < 5000 {
                0.8
            } else if avg.as_micros() < 10000 {
                0.5
            } else {
                0.2
            };

            BenchmarkResult {
                name: "memory_retrieval_speed".into(),
                category: "memory".into(),
                score: speed_score,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("avg_micros".into(), avg.as_micros().to_string()),
                    ("trials".into(), trials.to_string()),
                ])),
            }
        };
        results.push(memory_retrieval_speed_result);

        results
    }

    pub fn run_convergence_benchmarks(cap: &CapabilityVector) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        let vector_stability_result = {
            let mut brain = ReasoningBrain::new();
            brain.capability = cap.clone();

            let absorb_sources = [KnowledgeSource::HeroUI,
                KnowledgeSource::BaseUI,
                KnowledgeSource::ArcUI,
                KnowledgeSource::CortexUI,
                KnowledgeSource::AgenticDS];

            let mut snapshots: Vec<Vec<f64>> = Vec::new();
            for i in 0..10 {
                brain.absorb(absorb_sources[i % absorb_sources.len()]);
                snapshots.push(brain.capability.arr.clone());
            }

            let n = snapshots.len() as f64;
            let per_dim_variance: Vec<f64> = if n > 0.0 {
                (0..snapshots[0].len()).map(|dim| {
                    let mean: f64 = snapshots.iter().map(|s| s[dim]).sum::<f64>() / n;
                    snapshots.iter().map(|s| (s[dim] - mean).powi(2)).sum::<f64>() / n
                }).collect()
            } else {
                vec![0.0]
            };
            let avg_variance = if per_dim_variance.is_empty() {
                0.0
            } else {
                per_dim_variance.iter().sum::<f64>() / per_dim_variance.len() as f64
            };
            let stability = 1.0 / (1.0 + avg_variance * 100.0);

            BenchmarkResult {
                name: "vector_stability".into(),
                category: "convergence".into(),
                score: stability,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("avg_variance".into(), format!("{:.6}", avg_variance)),
                    ("iterations".into(), "10".into()),
                    ("dimensions".into(), per_dim_variance.len().to_string()),
                ])),
            }
        };
        results.push(vector_stability_result);

        let absorption_efficiency_result = {
            let mut brain = ReasoningBrain::new();
            brain.capability = cap.clone();

            let sources = [KnowledgeSource::DesignPhilosophy,
                KnowledgeSource::Hyperframes,
                KnowledgeSource::Betterleaks,
                KnowledgeSource::ReactDoctor,
                KnowledgeSource::OpenPencil];

            let start = Instant::now();
            for i in 0..10 {
                brain.absorb(sources[i % sources.len()]);
            }
            let elapsed = start.elapsed();
            let avg_us = elapsed.as_micros() as f64 / 10.0;
            let efficiency = if avg_us < 50.0 {
                1.0
            } else if avg_us < 200.0 {
                0.8
            } else if avg_us < 500.0 {
                0.5
            } else {
                0.2
            };

            BenchmarkResult {
                name: "absorption_efficiency".into(),
                category: "convergence".into(),
                score: efficiency,
                max_score: 1.0,
                metadata: Some(HashMap::from([
                    ("total_micros".into(), elapsed.as_micros().to_string()),
                    ("avg_micros".into(), format!("{:.1}", avg_us)),
                ])),
            }
        };
        results.push(absorption_efficiency_result);

        results
    }
}

// ═══════════════════════════════════════════════════════════════════
// F7: Ori-Eval — model-selection proof loop (R-P79 接线到生产路径)
//
// 不同于 BenchmarkSuite 的内置能力向量评测 (基准自身内部向量, 无真实模型),
// Ori-Eval 运行"我们自己的 agent 提示词" → 检查 tool call 正确/必要性 →
// 按 rubric 打分答案 → 输出 per-model 分数表 → 选出最优模型。
//
// 设计 (R-P42 强化既有 BenchmarkSuite 节点):
// - `OriEvalModel` trait: 让测试注入 test-double; 生产走 `Arc<dyn LlmProvider>`
//   (GatewayV2 实现该 trait, 因此可复用真实网关请求)。
// - 评分纯函数化: `grade_case` / `tool_call_legitimacy` 可单测。
// ═══════════════════════════════════════════════════════════════════

/// 一个 Ori-Eval 用例 — 我们自己的 agent 提示词 + 期望行为
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriEvalCase {
    pub id: String,
    pub prompt: String,
    /// 期望调用的工具名 (正确性检查: 命中任意一个即合法)
    pub expected_tool: Option<String>,
    /// 期望回答包含的关键词 (rubric 评分)
    pub rubric_keywords: Vec<String>,
    /// 是否明确要求调用工具 (必要性检查: 若 false 则调用工具视为不必要)
    pub requires_tool: bool,
}

impl OriEvalCase {
    pub fn new(
        id: &str,
        prompt: &str,
        expected_tool: Option<&str>,
        rubric_keywords: &[&str],
        requires_tool: bool,
    ) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            expected_tool: expected_tool.map(|s| s.to_string()),
            rubric_keywords: rubric_keywords.iter().map(|s| s.to_string()).collect(),
            requires_tool,
        }
    }
}

/// Ori-Eval 模型执行接口 — 让测试注入 test-double, 生产用真实 provider
#[async_trait::async_trait]
pub trait OriEvalModel: Send + Sync {
    /// 以指定模型名运行一次完整 prompt, 返回 (内容, tool 调用名列表)
    async fn run(&self, model: &str, prompt: &str) -> Result<(String, Vec<String>), LlmError>;
}

/// 一个用例的 Ori-Eval 评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriCaseScore {
    pub case_id: String,
    pub answer_grade: f64,         // rubric 关键词命中率 0.0-1.0
    pub tool_call_legit: bool,     // 调用的工具是否在期望集内
    pub tool_call_necessary: bool, // 是否需要调用了 / 不需时未调用
}

impl OriCaseScore {
    /// 该用例综合分: answer_grade 为主, tool 合法性/必要性为修正
    pub fn composite(&self) -> f64 {
        let tool_factor = if self.tool_call_legit && self.tool_call_necessary {
            1.0
        } else if self.tool_call_legit || self.tool_call_necessary {
            0.5
        } else {
            0.0
        };
        self.answer_grade * 0.6 + tool_factor * 0.4
    }
}

/// 单个模型的总分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriModelScore {
    pub model: String,
    pub case_scores: Vec<OriCaseScore>,
    pub avg_answer_grade: f64,
    pub tool_call_accuracy: f64, // 合法 tool 调用用例占比
    pub tool_necessity: f64,     // 必要性达标的用例占比
    pub composite: f64,          // 综合分 (用于排序选模型)
}

/// Ori-Eval 报告 — per-model 分数表 + 排名
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriEvalReport {
    pub per_model: Vec<OriModelScore>,
    /// 按 composite 降序的模型名排名 (第 0 位 = 最优模型)
    pub ranking: Vec<String>,
    pub timestamp: String,
}

impl OriEvalReport {
    /// 最优模型 (排名第 0 位)
    pub fn best_model(&self) -> Option<&str> {
        self.ranking.first().map(|s| s.as_str())
    }
}

/// F7: Ori-Eval 引擎 — 运行自己 agent 提示词 → 评分 → 选出最优模型
#[derive(Debug, Default)]
pub struct OriEvalSuite {
    pub cases: Vec<OriEvalCase>,
}

impl OriEvalSuite {
    pub fn new(cases: Vec<OriEvalCase>) -> Self {
        Self { cases }
    }

    /// 用真实 provider (如 GatewayV2, 其实现 LlmProvider) 跑一批模型。
    /// `models`: (展示名, provider) 列表 — 生产 R-P79 接线点。
    pub async fn run_with_provider(
        &self,
        models: &[(&str, Arc<dyn LlmProvider>)],
    ) -> Result<OriEvalReport, LlmError> {
        let mut scores = Vec::new();
        for (name, provider) in models {
            let model_scores = self.run_one(&name, provider.as_ref()).await?;
            scores.push(model_scores);
        }
        Ok(Self::finalize_report(scores))
    }

    /// 用 test-double (OriEvalModel trait) 跑一批模型 — 单测/离线路径
    pub async fn run_with_model(
        &self,
        models: &[(&str, &dyn OriEvalModel)],
    ) -> Result<OriEvalReport, LlmError> {
        let mut scores = Vec::new();
        for (name, model) in models {
            let mut case_scores = Vec::new();
            for case in &self.cases {
                let (content, calls) = model.run(name, &case.prompt).await?;
                case_scores.push(Self::grade_case(case, &content, &calls));
            }
            scores.push(Self::aggregate_model(name.to_string(), case_scores));
        }
        Ok(Self::finalize_report(scores))
    }

    /// 用 `&dyn LlmProvider` 跑单个模型所有用例 — 生产接线 (如 GatewayV2 直接传入,
    /// 不经 Arc 包装; R-P79)。
    pub async fn score_with_provider(
        &self,
        model_name: &str,
        provider: &dyn LlmProvider,
    ) -> Result<OriModelScore, LlmError> {
        self.run_one(model_name, provider).await
    }

    /// 运行单个模型的所有用例
    async fn run_one(
        &self,
        model_name: &str,
        provider: &dyn LlmProvider,
    ) -> Result<OriModelScore, LlmError> {
        let mut case_scores = Vec::new();
        for case in &self.cases {
            let request = LlmRequest::new(model_name, &case.prompt)
                .with_temperature(Some(0.0))
                .with_max_tokens(1024);
            let response: LlmResponse = provider.complete(&request).await?;
            let calls: Vec<String> = response
                .tool_calls
                .unwrap_or_default()
                .iter()
                .map(|tc| tc.function.name.clone())
                .collect();
            case_scores.push(Self::grade_case(case, &response.content, &calls));
        }
        Ok(Self::aggregate_model(model_name.to_string(), case_scores))
    }

    /// 纯函数评分 — 单个用例: rubric 关键词 + tool 合法性/必要性
    pub fn grade_case(case: &OriEvalCase, content: &str, tool_calls: &[String]) -> OriCaseScore {
        let lower = content.to_lowercase();
        let answer_grade = if case.rubric_keywords.is_empty() {
            if content.trim().is_empty() { 0.0 } else { 1.0 }
        } else {
            let hits = case
                .rubric_keywords
                .iter()
                .filter(|kw| lower.contains(&kw.to_lowercase()))
                .count();
            hits as f64 / case.rubric_keywords.len() as f64
        };

        // 合法性: 所有调用的工具都命中期望工具 (或期望工具为 None 时不调用工具)
        let tool_call_legit = match &case.expected_tool {
            Some(expected) => tool_calls.iter().any(|c| c == expected),
            None => tool_calls.is_empty(),
        };
        // 必要性: requires_tool=true 时调用了工具; false 时未调用工具
        let tool_call_necessary = if case.requires_tool {
            !tool_calls.is_empty()
        } else {
            tool_calls.is_empty()
        };

        OriCaseScore {
            case_id: case.id.clone(),
            answer_grade,
            tool_call_legit,
            tool_call_necessary,
        }
    }

    /// 聚合一个模型的全部用例分数
    pub fn aggregate_model(model: String, case_scores: Vec<OriCaseScore>) -> OriModelScore {
        let n = case_scores.len().max(1) as f64;
        let avg_answer_grade = case_scores.iter().map(|s| s.answer_grade).sum::<f64>() / n;
        let tool_call_accuracy =
            case_scores.iter().filter(|s| s.tool_call_legit).count() as f64 / n;
        let tool_necessity =
            case_scores.iter().filter(|s| s.tool_call_necessary).count() as f64 / n;
        let composite = avg_answer_grade * 0.5 + tool_call_accuracy * 0.3 + tool_necessity * 0.2;
        OriModelScore {
            model,
            case_scores,
            avg_answer_grade,
            tool_call_accuracy,
            tool_necessity,
            composite,
        }
    }

    /// 生成报告 + 排名 (composite 降序)
    pub fn finalize_report(mut scores: Vec<OriModelScore>) -> OriEvalReport {
        scores.sort_by(|a, b| b.composite.partial_cmp(&a.composite).unwrap_or(std::cmp::Ordering::Equal));
        let ranking = scores.iter().map(|s| s.model.clone()).collect();
        OriEvalReport {
            per_model: scores,
            ranking,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_cap() -> CapabilityVector {
        let mut cap = CapabilityVector::default();
        for v in cap.arr_mut().iter_mut() {
            *v = 0.5;
        }
        cap
    }

    #[test]
    fn test_benchmark_runs_without_panic() {
        let cap = CapabilityVector::default();
        let report = BenchmarkSuite::run_all(&cap);
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 1.0);
        assert!(!report.results.is_empty());
    }

    #[test]
    fn test_benchmark_category_filter() {
        let cap = CapabilityVector::default();
        let core = BenchmarkSuite::run_category(&cap, "core");
        assert!(!core.is_empty());
        let task = BenchmarkSuite::run_category(&cap, "task");
        assert_eq!(task.len(), 4);
    }

    #[test]
    fn test_knowledge_benchmarks() {
        let cap = make_test_cap();
        let results = BenchmarkSuite::run_knowledge_benchmarks(&cap);
        assert_eq!(results.len(), 3);
        for r in &results {
            assert_eq!(r.category, "knowledge");
            assert!(r.score >= 0.0 && r.score <= 1.0);
            assert!(r.metadata.is_some());
        }
        let coverage = results.iter().find(|r| r.name == "knowledge_coverage").expect("knowledge_coverage result should be present");
        assert!(coverage.score > 0.0);
        let diversity = results.iter().find(|r| r.name == "knowledge_diversity").expect("knowledge_diversity result should be present");
        assert!(diversity.score >= 0.0);
    }

    #[test]
    fn test_memory_retention() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let retention = results.iter().find(|r| r.name == "memory_retention").expect("memory_retention result should be present");
        assert_eq!(retention.category, "memory");
        assert!(retention.score >= 0.0 && retention.score <= 1.0);
    }

    #[test]
    fn test_memory_capacity() {
        let mut bank = ReasoningBank::new(100);
        let results = BenchmarkSuite::run_memory_benchmarks(&mut bank);
        let capacity = results.iter().find(|r| r.name == "memory_capacity").expect("memory_capacity result should be present");
        assert_eq!(capacity.max_score, 1.0);
        let meta = capacity.metadata.as_ref().expect("memory_capacity should have metadata");
        assert_eq!(meta.get("panic").expect("metadata should contain 'panic' key"), "false");
    }

    #[test]
    fn test_vector_stability() {
        let cap = make_test_cap();
        let results = BenchmarkSuite::run_convergence_benchmarks(&cap);
        let stability = results.iter().find(|r| r.name == "vector_stability").expect("vector_stability result should be present");
        assert_eq!(stability.category, "convergence");
        assert!(stability.score > 0.0 && stability.score <= 1.0);
    }

    #[test]
    fn test_benchmark_report_format() {
        let cap = make_test_cap();
        let mut bank = ReasoningBank::new(100);
        let report = BenchmarkSuite::run_all_extended(&cap, &mut bank);
        assert!(!report.results.is_empty());
        assert!(report.overall_score >= 0.0 && report.overall_score <= 1.0);
        assert!(!report.timestamp.is_empty());
        let categories: std::collections::HashSet<&str> = report.results.iter().map(|r| r.category.as_str()).collect();
        assert!(categories.contains("core"));
        assert!(categories.contains("task"));
        assert!(categories.contains("knowledge"));
        assert!(categories.contains("memory"));
        assert!(categories.contains("convergence"));
    }

    #[test]
    fn test_backward_compatible_run_all() {
        let cap = make_test_cap();
        let report = BenchmarkSuite::run_all(&cap);
        assert_eq!(report.results.len(), 7);
        for r in &report.results {
            assert!(r.metadata.is_none());
        }
    }

    // ── F7: Ori-Eval ─────────────────────────────────────────────

    #[test]
    fn test_ori_grade_rubric_keyword_hit() {
        let case = OriEvalCase::new("q1", "write a rust fn", None, &["fn", "rust"], false);
        let score = OriEvalSuite::grade_case(&case, "fn add(a: u32) -> u32 { a } in rust", &[]);
        assert!((score.answer_grade - 1.0).abs() < 1e-9, "两个关键词都命中: {}", score.answer_grade);
        let partial = OriEvalSuite::grade_case(&case, "just rust", &[]);
        assert!((partial.answer_grade - 0.5).abs() < 1e-9);
        let none = OriEvalSuite::grade_case(&case, "nothing here", &[]);
        assert_eq!(none.answer_grade, 0.0);
    }

    #[test]
    fn test_ori_tool_call_legitimacy() {
        let case = OriEvalCase::new("q1", "search the web", Some("web_search"), &["result"], true);
        // 命中期望工具 → 合法
        let ok = OriEvalSuite::grade_case(&case, "found result", &["web_search".to_string()]);
        assert!(ok.tool_call_legit);
        assert!(ok.tool_call_necessary);
        // 调用非期望工具 → 不合法
        let bad = OriEvalSuite::grade_case(&case, "result", &["execute_command".to_string()]);
        assert!(!bad.tool_call_legit);
        // requires_tool=true 但未调用 → 不必要
        let no_call = OriEvalSuite::grade_case(&case, "result", &[]);
        assert!(!no_call.tool_call_necessary);
    }

    #[test]
    fn test_ori_tool_necessity_when_not_required() {
        let case = OriEvalCase::new("q1", "just answer", None, &["answer"], false);
        // 不需要工具时未调用 → 必要
        let no_call = OriEvalSuite::grade_case(&case, "the answer is 42", &[]);
        assert!(no_call.tool_call_necessary);
        assert!(no_call.tool_call_legit);
        // 不需要工具时却调用 → 不必要
        let over = OriEvalSuite::grade_case(&case, "answer", &["web_search".to_string()]);
        assert!(!over.tool_call_necessary);
        assert!(!over.tool_call_legit);
    }

    #[test]
    fn test_ori_case_composite_factor() {
        let case = OriEvalCase::new("q1", "search", Some("web_search"), &["hit"], true);
        let perfect = OriEvalSuite::grade_case(&case, "hit", &["web_search".to_string()]);
        assert!((perfect.composite() - 1.0).abs() < 1e-9);
        let tool_bad = OriEvalSuite::grade_case(&case, "hit", &[]);
        assert!(tool_bad.composite() < perfect.composite());
    }

    #[tokio::test]
    async fn test_ori_run_with_test_double_ranks_best_model() {
        struct Double {
            good: bool,
        }
        #[async_trait::async_trait]
        impl OriEvalModel for Double {
            async fn run(&self, _model: &str, prompt: &str) -> Result<(String, Vec<String>), LlmError> {
                if self.good {
                    Ok(("answer with correct result".into(), vec!["web_search".into()]))
                } else {
                    let _ = prompt;
                    Ok(("wrong".into(), vec![]))
                }
            }
        }
        let cases = vec![
            OriEvalCase::new("q1", "search and answer", Some("web_search"), &["correct"], true),
            OriEvalCase::new("q2", "search and answer", Some("web_search"), &["result"], true),
        ];
        let suite = OriEvalSuite::new(cases);
        let good = Double { good: true };
        let bad = Double { good: false };
        let report = suite
            .run_with_model(&[("good-model", &good), ("bad-model", &bad)])
            .await
            .expect("report");
        assert_eq!(report.ranking.len(), 2);
        assert_eq!(report.best_model(), Some("good-model"), "最优模型应排第 0 位");
        assert_eq!(report.ranking[1], "bad-model");
        // composite 单调递减
        assert!(
            report.per_model[0].composite > report.per_model[1].composite
        );
    }

    #[test]
    fn test_ori_aggregate_model_and_tool_accuracy() {
        let case = OriEvalCase::new("q", "search", Some("web_search"), &["result"], true);
        let good = OriEvalSuite::grade_case(&case, "result", &["web_search".into()]);
        let bad = OriEvalSuite::grade_case(&case, "result", &[]);
        let score = OriEvalSuite::aggregate_model("m1".into(), vec![good, bad]);
        assert_eq!(score.case_scores.len(), 2);
        assert!((score.tool_call_accuracy - 0.5).abs() < 1e-9);
        assert!((score.tool_necessity - 0.5).abs() < 1e-9);
        assert!((score.avg_answer_grade - 1.0).abs() < 1e-9);
        assert!(score.composite > 0.0 && score.composite <= 1.0);
    }

    #[test]
    fn test_ori_empty_rubric_grades_nonempty_content() {
        let case = OriEvalCase::new("q", "hi", None, &[], false);
        let score = OriEvalSuite::grade_case(&case, "hello world", &[]);
        assert_eq!(score.answer_grade, 1.0);
        let empty = OriEvalSuite::grade_case(&case, "", &[]);
        assert_eq!(empty.answer_grade, 0.0);
    }
}
