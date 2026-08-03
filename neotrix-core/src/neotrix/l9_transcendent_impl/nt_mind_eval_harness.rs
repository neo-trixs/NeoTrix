//! L9 Evaluation Harness — Model×Budget 质量-成本评测 (R2-Bench 式)
//!
//! 参考: R2-Router (ICML 2026), R2-Bench dataset
//! 核心指标: AUDC (Area Under Deferral Curve), QNC (Query-Normalized Cost), Peak Quality
//! 预算执行: prompt 注入 "use at most K tokens" (Lee et al. 2025)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use crate::neotrix::nt_io_provider::{LlmProvider, LlmRequest, LlmResponse, LlmError, LlmProviderType, create_provider_from_type};
use crate::core::nt_core_ttc::{EffortTier, EffortTierSelector};
use crate::core::nt_core_e8::domain_transition::E8TaskType;
use crate::core::nt_core_prm::ProcessScore;
use crate::neotrix::l9_transcendent_impl::nt_mind_consciousness_gold_standard::{ConsciousnessGoldStandard, ConsciousnessLevel, GoldStandardReport, E8HexagramState, derive_level};

/// 评测预算网格 (R2-Bench 16 点 + 我们努力分层对齐)
pub const DEFAULT_BUDGET_GRID: &[u32] = &[
    0,      // 直接回答 (EffortTier::Low, thinking_budget=0)
    512,    // 极低
    1024,   // Low-Medium 边界
    2048,   // Medium
    4096,   // High
    8192,   // XHigh
    16384,  // Max
    32768,  // Max+ (unlimited)
];

/// 基线模型规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub name: String,
    pub provider_type: String,      // "vllm" | "sglang" | "ollama" | "openai" | "anthropic" 等
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key_env: Option<String>,
    pub pricing_per_1m_in: f64,     // USD per 1M input tokens
    pub pricing_per_1m_out: f64,    // USD per 1M output tokens
}

/// 数据集规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSpec {
    pub name: String,
    pub queries: Vec<EvalQuery>,
    pub judge_model: String,        // e.g. "qwen3-80b-instruct" (LLM-as-judge)
    pub judge_base_url: Option<String>,
    pub judge_api_key_env: Option<String>,
    pub golden_answers: Option<HashMap<String, String>>, // query_id -> golden answer
}

/// 单条评测查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalQuery {
    pub id: String,
    pub prompt: String,
    pub category: String,           // "math" | "reasoning" | "coding" | "knowledge" | "rag" | "creative"
    pub difficulty: f64,            // 0.0~1.0
    pub expected_tokens: u32,       // 预估合理输出长度
}

/// 单次评测结果 (model × budget 点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalPoint {
    pub model_name: String,
    pub budget: u32,
    pub query_id: String,
    pub response: String,
    pub actual_tokens: u32,
    pub quality_score: f64,         // 0.0~1.0 (LLM judge)
    pub judge_justification: String,
    pub latency_ms: u64,
    pub cost_usd: f64,
    pub consciousness_phi: Option<f64>, // 可选：金标 φ 检测
    pub consciousness_level: Option<ConsciousnessLevel>,
}

/// 质量-成本矩阵行 (单模型全预算)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQualityCurve {
    pub model_name: String,
    pub points: Vec<EvalPoint>,
    // 插值后的连续曲线 (用于 AUDC 计算)
    pub interpolated_quality: Vec<f64>,  // 对应 DEFAULT_BUDGET_GRID
}

/// Pareto 前沿点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    pub model_name: String,
    pub budget: u32,
    pub quality: f64,
    pub cost_usd: f64,
}

/// 评测报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub timestamp: i64,
    pub dataset_name: String,
    pub curves: Vec<ModelQualityCurve>,
    pub audc_scores: HashMap<String, f64>,      // model -> AUDC
    pub qnc_scores: HashMap<String, f64>,       // model -> QNC
    pub peak_quality: HashMap<String, f64>,     // model -> Peak Quality
    pub pareto_frontier: Vec<ParetoPoint>,
    pub galaxy_vs_baseline: HashMap<String, GalaxyComparison>,
    pub summary: String,
}

/// 大阵 vs 基线对比
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyComparison {
    pub baseline_model: String,
    pub galaxy_effort_tier: EffortTier,
    pub quality_delta: f64,         // galaxy - baseline (同预算)
    pub token_savings_pct: f64,     // 同质量下 galaxy 省 token %
    pub audc_improvement: f64,
}

/// Judge 规格
#[derive(Clone)]
struct JudgeSpec {
    provider: Arc<dyn LlmProvider>,
    model_id: String,
}

/// 评测 Harness 主结构
pub struct EvalHarness {
    budget_grid: Vec<u32>,
    baselines: Vec<ModelSpec>,
    datasets: Vec<DatasetSpec>,
    judge: JudgeSpec,
    gold_standard: Arc<ConsciousnessGoldStandard>,
    concurrency_limit: usize,
}

impl EvalHarness {
    /// 创建默认 harness (使用 R2-Bench 标准预算网格)
    pub fn new_default(
        baselines: Vec<ModelSpec>,
        datasets: Vec<DatasetSpec>,
        judge_provider: Arc<dyn LlmProvider>,
        judge_model: String,
    ) -> Self {
        Self {
            budget_grid: DEFAULT_BUDGET_GRID.to_vec(),
            baselines,
            datasets,
            judge: JudgeSpec { provider: judge_provider, model_id: judge_model },
            gold_standard: Arc::new(ConsciousnessGoldStandard::new()),
            concurrency_limit: 4,
        }
    }

    /// 自定义预算网格
    pub fn with_budget_grid(mut self, grid: Vec<u32>) -> Self {
        self.budget_grid = grid;
        self
    }

    /// 设置并发限制
    pub fn with_concurrency(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit.max(1);
        self
    }

    /// 运行全量评测 (所有数据集 × 所有模型 × 所有预算)
    pub async fn run(&self) -> Result<Vec<EvalReport>, EvalError> {
        let mut reports = Vec::new();
        for dataset in &self.datasets {
            let report = self.run_dataset(dataset).await?;
            reports.push(report);
        }
        Ok(reports)
    }

    /// 运行单数据集评测
    pub async fn run_dataset(&self, dataset: &DatasetSpec) -> Result<EvalReport, EvalError> {
        let mut curves = Vec::new();
        
        // 为每个基线模型跑完整预算网格
        for model_spec in &self.baselines {
            let curve = self.eval_model_on_dataset(model_spec, dataset).await?;
            curves.push(curve);
        }

        // 计算指标
        let (audc_scores, qnc_scores, peak_quality, pareto_frontier) = self.compute_metrics(&curves);
        
        // 大阵对比 (若数据集包含同模型不同 effort_tier)
        let galaxy_vs_baseline = self.compare_galaxy_vs_baseline(&curves, dataset);

        let summary = self.generate_summary(&curves, &audc_scores, &qnc_scores, &peak_quality);

        Ok(EvalReport {
            timestamp: chrono::Utc::now().timestamp(),
            dataset_name: dataset.name.clone(),
            curves,
            audc_scores,
            qnc_scores,
            peak_quality,
            pareto_frontier,
            galaxy_vs_baseline,
            summary,
        })
    }

    /// 评测单模型在单数据集上的全预算曲线
    async fn eval_model_on_dataset(&self, model: &ModelSpec, dataset: &DatasetSpec) -> Result<ModelQualityCurve, EvalError> {
        let provider = self.build_provider(model)?;
        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));
        let mut points = Vec::new();

        for query in &dataset.queries {
            for &budget in &self.budget_grid {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => return Err(EvalError::ConcurrencyClosed),
                };
                let provider = provider.clone();
                let judge = self.judge.clone();
                let query = query.clone();
                let budget = budget;
                let gold_standard = self.gold_standard.clone();
                let model_name = model.name.clone();

                let point = tokio::spawn(async move {
                    let _permit = permit;
                    Self::eval_single_point(
                        provider, judge, gold_standard,
                        model_name, query, budget
                    ).await
                }).await.map_err(|e| EvalError::JoinError(e.to_string()))??;

                points.push(point);
            }
        }

        // 按预算聚合插值质量
        let interpolated = Self::interpolate_quality(&points, &self.budget_grid);

        Ok(ModelQualityCurve {
            model_name: model.name.clone(),
            points,
            interpolated_quality: interpolated,
        })
    }

    /// 单点评测: 发送请求 + Judge 打分 + 金标检测
    async fn eval_single_point(
        provider: Arc<dyn LlmProvider>,
        judge: JudgeSpec,
        gold_standard: Arc<ConsciousnessGoldStandard>,
        model_name: String,
        query: EvalQuery,
        budget: u32,
    ) -> Result<EvalPoint, EvalError> {
        let start = Instant::now();

        // 构建带预算约束的 prompt
        let budget_prompt = if budget > 0 {
            format!("\n\n[Constraint] Answer within {} tokens.", budget)
        } else {
            "\n\n[Constraint] Answer directly without extended reasoning.".to_string()
        };
        let full_prompt = format!("{}{}", query.prompt, budget_prompt);

        let request = LlmRequest::new(&model_name, &full_prompt)
            .with_max_tokens(budget.max(512))
            .with_thinking(budget);

        let response = provider.complete(&request).await.map_err(EvalError::ProviderError)?;
        let latency = start.elapsed().as_millis() as u64;

        // 计算成本
        let cost = (response.usage.prompt_tokens as f64 / 1_000_000.0) * 0.0  // 需要从 model_spec 取 pricing
            + (response.usage.completion_tokens as f64 / 1_000_000.0) * 0.0;

        // LLM Judge 打分
        let (quality, justification) = Self::judge_response(&judge, &query, &response.content).await?;

        // 金标意识检测 (可选，低频采样)
        let (phi, level) = if query.difficulty > 0.7 {
            // 简易评估：用响应长度启发式近似 phi，避免完整状态依赖
            let heuristic_phi = (response.content.len() as f64 / 2000.0).min(1.0);
            let dummy_report = GoldStandardReport {
                timestamp: chrono::Utc::now(),
                phi: heuristic_phi,
                coherence: 0.5,
                is_conscious_like: heuristic_phi > 0.33,
                is_phi_conscious: heuristic_phi > 0.33,
                is_coherent: true,
                phi_confidence: heuristic_phi,
                coherence_confidence: 0.5,
                detection_streak: 0,
                combined_confidence: heuristic_phi * 0.5,
            };
            (Some(dummy_report.phi), Some(derive_level(&dummy_report)))
        } else { (None, None) };

        Ok(EvalPoint {
            model_name,
            budget,
            query_id: query.id,
            response: response.content,
            actual_tokens: response.usage.completion_tokens,
            quality_score: quality,
            judge_justification: justification,
            latency_ms: latency,
            cost_usd: cost,
            consciousness_phi: phi,
            consciousness_level: level,
        })
    }

    /// LLM-as-Judge 打分 (参考 R2-Bench: Qwen3-80B-Instruct, Pearson r=0.82 vs human)
    async fn judge_response(
        judge: &JudgeSpec,
        query: &EvalQuery,
        response: &str,
    ) -> Result<(f64, String), EvalError> {
        let judge_prompt = format!(
            r#"You are an expert evaluator. Score the following response on a scale of 0.0 to 1.0 for correctness and quality.

Query (category: {}): {}
Response: {}

Provide your score and brief justification in JSON:
{{"score": <0.0-1.0>, "justification": "<brief>"}}"#,
            query.category, query.prompt, response
        );

        let request = LlmRequest::new(&judge.model_id, &judge_prompt)
            .with_max_tokens(512)
            .with_temperature(Some(0.0));

        let jr = judge.provider.complete(&request).await.map_err(EvalError::ProviderError)?;
        
        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(&jr.content)
            .map_err(|e| EvalError::JudgeParseError(e.to_string()))?;
        let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let justification = parsed.get("justification").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Ok((score.clamp(0.0, 1.0), justification))
    }

    /// 线性插值质量曲线到标准预算网格
    fn interpolate_quality(points: &[EvalPoint], grid: &[u32]) -> Vec<f64> {
        // 按 budget 分组取平均
        let mut budget_to_quality: HashMap<u32, Vec<f64>> = HashMap::new();
        for p in points {
            budget_to_quality.entry(p.budget).or_default().push(p.quality_score);
        }
        let avg_quality: HashMap<u32, f64> = budget_to_quality
            .into_iter()
            .map(|(b, qs)| (b, qs.iter().sum::<f64>() / qs.len() as f64))
            .collect();

        // 线性插值
        grid.iter().map(|&target_budget| {
            if let Some(&q) = avg_quality.get(&target_budget) {
                return q;
            }
            let mut lower = None;
            let mut upper = None;
            for &b in avg_quality.keys() {
                if b <= target_budget && lower.map_or(true, |l| b > l) {
                    lower = Some(b);
                }
                if b >= target_budget && upper.map_or(true, |u| b < u) {
                    upper = Some(b);
                }
            }
            match (lower, upper) {
                (Some(l), Some(u)) if l != u => {
                    let ql = avg_quality[&l];
                    let qu = avg_quality[&u];
                    let t = (target_budget - l) as f64 / (u - l) as f64;
                    ql + t * (qu - ql)
                }
                (Some(l), None) => avg_quality[&l],
                (None, Some(u)) => avg_quality[&u],
                _ => 0.0,
            }
        }).collect()
    }

    /// 计算 AUDC, QNC, Peak Quality, Pareto 前沿
    fn compute_metrics(&self, curves: &[ModelQualityCurve]) -> (
        HashMap<String, f64>, HashMap<String, f64>, HashMap<String, f64>, Vec<ParetoPoint>
    ) {
        let mut audc_scores = HashMap::new();
        let mut qnc_scores = HashMap::new();
        let mut peak_quality = HashMap::new();
        let mut all_points = Vec::new();

        for curve in curves {
            let qualities = &curve.interpolated_quality;
            let costs: Vec<f64> = self.budget_grid.iter()
                .map(|&b| b as f64 * 0.000001) // 简化：假设 $1/1M tokens，实际应用 model 定价
                .collect();

            // AUDC: 梯形积分 quality vs cost
            let mut audc = 0.0;
            for i in 1..qualities.len() {
                let q_avg = (qualities[i-1] + qualities[i]) / 2.0;
                let c_delta = costs[i] - costs[i-1];
                audc += q_avg * c_delta;
            }
            // 归一化到 [0,1] (除以 max cost)
            let max_cost = costs.last().copied().unwrap_or(1.0);
            audc_scores.insert(curve.model_name.clone(), audc / max_cost);

            // Peak Quality
            let peak = qualities.iter().copied().fold(0.0, f64::max);
            peak_quality.insert(curve.model_name.clone(), peak);

            // 收集 Pareto 候选
            for (i, &q) in qualities.iter().enumerate() {
                all_points.push(ParetoPoint {
                    model_name: curve.model_name.clone(),
                    budget: self.budget_grid[i],
                    quality: q,
                    cost_usd: costs[i],
                });
            }
        }

        // QNC: 相对最佳单模型的成本归一化
        let best_peak = peak_quality.values().copied().fold(0.0, f64::max);
        for curve in curves {
            let min_cost_for_best = all_points.iter()
                .filter(|p| p.model_name == curve.model_name && (p.quality - best_peak).abs() < 0.01)
                .map(|p| p.cost_usd)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(f64::INFINITY);
            let best_single_cost = all_points.iter()
                .filter(|p| (p.quality - best_peak).abs() < 0.01)
                .map(|p| p.cost_usd)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(1.0);
            qnc_scores.insert(curve.model_name.clone(), if best_single_cost > 0.0 { min_cost_for_best / best_single_cost } else { 1.0 });
        }

        // Pareto 前沿: 无其他点同时 quality >= 且 cost <=
        let mut pareto = Vec::new();
        for p in &all_points {
            let dominated = all_points.iter().any(|o| 
                o.quality >= p.quality && o.cost_usd <= p.cost_usd && 
                (o.quality > p.quality || o.cost_usd < p.cost_usd)
            );
            if !dominated {
                pareto.push(p.clone());
            }
        }
        pareto.sort_by(|a, b| a.cost_usd.total_cmp(&b.cost_usd));

        (audc_scores, qnc_scores, peak_quality, pareto)
    }

    /// 大阵 vs 基线对比: 同模型不同 effort_tier
    fn compare_galaxy_vs_baseline(
        &self, _curves: &[ModelQualityCurve], _dataset: &DatasetSpec
    ) -> HashMap<String, GalaxyComparison> {
        // 这里简化：实际需对比同模型在 effort_tier=Low/Max 下的曲线
        // 需要数据集包含 effort_tier 标注
        HashMap::new()
    }

    fn generate_summary(
        &self, curves: &[ModelQualityCurve], audc: &HashMap<String, f64>, 
        qnc: &HashMap<String, f64>, peak: &HashMap<String, f64>
    ) -> String {
        let mut lines = vec![format!("Eval Summary ({} models, {} budgets)", curves.len(), self.budget_grid.len())];
        for curve in curves {
            lines.push(format!(
                "  {}: AUDC={:.3} QNC={:.3} Peak={:.3}",
                curve.model_name,
                audc.get(&curve.model_name).unwrap_or(&0.0),
                qnc.get(&curve.model_name).unwrap_or(&1.0),
                peak.get(&curve.model_name).unwrap_or(&0.0)
            ));
        }
        lines.join("\n")
    }

    fn build_provider(&self, spec: &ModelSpec) -> Result<Arc<dyn LlmProvider>, EvalError> {
        let provider_type = LlmProviderType::from_name(&spec.provider_type)
            .ok_or_else(|| EvalError::ConfigError(format!("Unknown provider type: {}", spec.provider_type)))?;
        let api_key = spec.api_key_env.as_ref().and_then(|env| std::env::var(env).ok());
        Ok(Arc::from(create_provider_from_type(provider_type, api_key)))
    }
}

/// 评测错误类型
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("Provider error: {0}")]
    ProviderError(LlmError),
    #[error("Judge parse error: {0}")]
    JudgeParseError(String),
    #[error("Join error: {0}")]
    JoinError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Concurrency semaphore closed")]
    ConcurrencyClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_quality() {
        let points = vec![
            EvalPoint { model_name: "m".into(), budget: 1024, query_id: "q1".into(), response: "".into(), actual_tokens: 100, quality_score: 0.5, judge_justification: "".into(), latency_ms: 100, cost_usd: 0.0, consciousness_phi: None, consciousness_level: None },
            EvalPoint { model_name: "m".into(), budget: 4096, query_id: "q1".into(), response: "".into(), actual_tokens: 400, quality_score: 0.8, judge_justification: "".into(), latency_ms: 200, cost_usd: 0.0, consciousness_phi: None, consciousness_level: None },
        ];
        let grid = vec![1024, 2048, 4096];
        let interp = EvalHarness::interpolate_quality(&points, &grid);
        assert_eq!(interp[0], 0.5);
        assert!((interp[1] - 0.6).abs() < 0.01); // 线性插值: 2048 位于 1024→4096 的 1/3 处
        assert_eq!(interp[2], 0.8);
    }

    #[test]
    fn test_budget_grid_constants() {
        assert_eq!(DEFAULT_BUDGET_GRID.len(), 8);
        assert_eq!(DEFAULT_BUDGET_GRID[0], 0);    // Low: 无思考
        assert_eq!(DEFAULT_BUDGET_GRID[3], 2048); // Medium
        assert_eq!(DEFAULT_BUDGET_GRID[6], 16384); // Max
    }
}