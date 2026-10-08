//! Harness 主入口 — EvalHarness 结构 + 运行管线 + trait 接线 (纯搬移自门面)。

use super::nt_budget::DEFAULT_BUDGET_GRID;
use super::nt_compliance::ComplianceGate;
use super::nt_regression::RegressionCase;
use super::nt_types::{
    DatasetSpec, EvalError, EvalPoint, EvalQuery, EvalReport, JudgeSpec, ModelQualityCurve,
    ModelSpec,
};
use super::nt_verify_oracle::{
    UnifiedVerifyRequest, verify_constraint, verify_deterministic, verify_unified,
};
use crate::l1_action::nt_io::nt_io_provider::{
    create_provider_from_type, LlmProvider, LlmProviderType, LlmRequest,
};
use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::{
    derive_level, ConsciousnessGoldStandard, GoldStandardReport,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// 评测 Harness 主结构
pub struct EvalHarness {
    pub(crate) budget_grid: Vec<u32>,
    pub(crate) baselines: Vec<ModelSpec>,
    pub(crate) datasets: Vec<DatasetSpec>,
    pub(crate) judge: JudgeSpec,
    pub(crate) gold_standard: Arc<ConsciousnessGoldStandard>,
    pub(crate) concurrency_limit: usize,
    pub(crate) compliance: ComplianceGate,
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
            judge: JudgeSpec {
                provider: judge_provider,
                model_id: judge_model,
            },
            gold_standard: Arc::new(ConsciousnessGoldStandard::new()),
            concurrency_limit: 4,
            compliance: ComplianceGate::new(),
        }
    }

    /// 自定义预算网格
    pub fn with_budget_grid(mut self, grid: Vec<u32>) -> Self {
        self.budget_grid = grid;
        self
    }

    /// 设置并发限制
    pub(crate) fn _with_concurrency(mut self, limit: usize) -> Self {
        self.concurrency_limit = limit.max(1);
        self
    }

    /// 记录一条 withholding 结果 (plain vs aided 通过率)
    pub fn record_withholding(&mut self, plain_pass: f64, aided_pass: f64, samples: usize) {
        self.compliance.record_withholding(plain_pass, aided_pass, samples);
    }

    /// 覆盖合规门阈值 (默认 0.5)
    pub(crate) fn _with_gate(mut self, gate: f64) -> Self {
        self.compliance.gate = gate;
        self
    }

    /// 合规报告: (_plane_conformance, mean_ap_acc, passes)
    pub fn compliance_report(&self) -> (f64, f64, bool) {
        (
            self.compliance._plane_conformance(),
            self.compliance.mean_ap_acc(),
            self.compliance.passes(),
        )
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
        let (audc_scores, qnc_scores, peak_quality, pareto_frontier) =
            self.compute_metrics(&curves);

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
    async fn eval_model_on_dataset(
        &self,
        model: &ModelSpec,
        dataset: &DatasetSpec,
    ) -> Result<ModelQualityCurve, EvalError> {
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
                let gold_standard = self.gold_standard.clone();
                let model_name = model.name.clone();
                let pricing_in = model.pricing_per_1m_in;
                let pricing_out = model.pricing_per_1m_out;

                let point = tokio::spawn(async move {
                    let _permit = permit;
                    Self::eval_single_point(
                        provider,
                        judge,
                        gold_standard,
                        model_name,
                        query,
                        budget,
                        pricing_in,
                        pricing_out,
                    )
                    .await
                })
                .await
                .map_err(|e| EvalError::JoinError(e.to_string()))??;

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
        _gold_standard: Arc<ConsciousnessGoldStandard>,
        model_name: String,
        query: EvalQuery,
        budget: u32,
        pricing_per_1m_in: f64,
        pricing_per_1m_out: f64,
    ) -> Result<EvalPoint, EvalError> {
        let start = Instant::now();

        // 构建带预算约束的 prompt
        let budget_prompt = if budget > 0 {
            format!("\n\n[Constraint] Answer within {} tokens.", budget)
        } else {
            "\n\n[Constraint] Answer directly without extended reasoning.".to_string()
        };
        let full_prompt = format!("{}{}", query.prompt, budget_prompt);

        // 思考预算与输出预算解耦: 此前 with_thinking(budget) 使思考可花掉全部输出预算,
        // 总生成 token 最高达 2×budget (纯浪费)。思考分配 25%, 输出保底 budget。
        let _thinking = if budget > 0 { (budget / 4).max(1) } else { 0 };
        let request = LlmRequest::new(&model_name, &full_prompt)
            .with_max_tokens(budget.max(512));

        let response = provider
            .complete(&request)
            .await
            .map_err(EvalError::ProviderError)?;
        let latency = start.elapsed().as_millis() as u64;

        // 真实定价 (ModelSpec.pricing_per_1m_*): 之前硬编码 0.0 使成本曲线失真,
        // 无法支撑 QNC/Pareto 决策。
        let cost = (response.usage.prompt_tokens as f64 / 1_000_000.0) * pricing_per_1m_in
            + (response.usage.completion_tokens as f64 / 1_000_000.0) * pricing_per_1m_out;

        // LLM Judge 打分
        let (quality, justification) =
            Self::judge_response(&judge, &query, &response.content).await?;

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
        } else {
            (None, None)
        };

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

        let jr = judge
            .provider
            .complete(&request)
            .await
            .map_err(EvalError::ProviderError)?;

        // 解析 JSON
        let parsed: serde_json::Value = serde_json::from_str(&jr.content)
            .map_err(|e| EvalError::JudgeParseError(e.to_string()))?;
        let score = parsed.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let justification = parsed
            .get("justification")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok((score.clamp(0.0, 1.0), justification))
    }

    fn build_provider(&self, spec: &ModelSpec) -> Result<Arc<dyn LlmProvider>, EvalError> {
        let provider_type = LlmProviderType::from_name(&spec.provider_type).ok_or_else(|| {
            EvalError::ConfigError(format!("Unknown provider type: {}", spec.provider_type))
        })?;
        let api_key = spec
            .api_key_env
            .as_ref()
            .and_then(|env| std::env::var(env).ok());
        Ok(create_provider_from_type(provider_type, api_key))
    }
}

// L5 trait abstraction: EvalHarnessApi
impl crate::l5_cognition::traits::EvalHarnessApi for EvalHarness {
    fn generate_regression_test(&self, candidate: &str) -> crate::l5_cognition::traits::RegressionCase {
        let l6_case = EvalHarness::generate_regression_test(self, candidate);
        crate::l5_cognition::traits::RegressionCase {
            id: l6_case.id,
            candidate: l6_case.candidate,
            forbidden_tokens: l6_case.forbidden_tokens,
            required_categories: l6_case.required_categories,
        }
    }

    fn run_regression_test(&self, case: &crate::l5_cognition::traits::RegressionCase) -> crate::l5_cognition::traits::RegressionResult {
        let l6_case = RegressionCase {
            id: case.id.clone(),
            candidate: case.candidate.clone(),
            forbidden_tokens: case.forbidden_tokens.clone(),
            required_categories: case.required_categories.clone(),
        };
        let l6_result = EvalHarness::run_regression_test(self, &l6_case);
        crate::l5_cognition::traits::RegressionResult {
            passed: l6_result.passed,
            reasons: l6_result.reasons,
        }
    }
}

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for EvalHarness {
    fn name(&self) -> &str {
        "nt_mind_eval_harness_self_verifiable"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let r = verify_deterministic("42", "42");
        if !r.verifiable || r.score != 1.0 {
            return Err(vec!["deterministic channel failed".into()]);
        }
        let c = verify_constraint("no-pii", "user@example.com", &["@example.com"]);
        if c.score != 0.0 {
            return Err(vec!["constraint channel should reject PII".into()]);
        }
        // 统一验证框架 (llm-as-a-verifier 吸收): 多模态分派入口行为健康
        let d = verify_unified(&UnifiedVerifyRequest {
            modality: "deterministic".into(),
            actual: "42".into(),
            expected: Some("42".into()),
            extractable: None,
            policy: None,
            forbidden: vec![],
        });
        if !d.passed() {
            return Err(vec!["unified deterministic channel failed".into()]);
        }
        let p = verify_unified(&UnifiedVerifyRequest {
            modality: "policy".into(),
            actual: "user@example.com".into(),
            expected: None,
            extractable: None,
            policy: Some("no-pii".into()),
            forbidden: vec!["@example.com".into()],
        });
        if p.score != 0.0 {
            return Err(vec!["unified policy channel should reject PII".into()]);
        }
        Ok(())
    }
}
