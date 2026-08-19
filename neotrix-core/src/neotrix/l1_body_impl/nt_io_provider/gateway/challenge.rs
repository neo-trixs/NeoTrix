use std::time::Instant;

use crate::neotrix::l8_autonomic_impl::nt_mind_benchmark::{OriEvalCase, OriEvalReport, OriEvalSuite};

use super::super::provider_catalog::lookup_provider;
use super::*;

impl GatewayV2 {
    // ═══════════════════════════════════════════════════════════════════
    // LLM Challenge (P0-3, Cycle 159) — Unstract/LLM-Challenge pattern
    // Deterministic challenge tasks scoring provider accuracy/latency/cost.
    // ═══════════════════════════════════════════════════════════════════

    /// Run the deterministic challenge suite against a provider. Returns a
    /// scored benchmark (accuracy, latency, cost) for the EvolutionFruit
    /// evidence chain and GatewayV2 provider selection.
    pub async fn run_llm_challenge(
        &self,
        provider_name: &str,
        task_type: &str,
    ) -> Result<crate::core::nt_core_consciousness_tree::ProviderBenchmark, LlmError> {
        let tasks = self.challenge_tasks(task_type);
        let mut correct = 0usize;
        let mut total_latency_ms = 0u64;
        let mut total_cost = 0.0f64;

        for task in tasks {
            let request = LlmRequest::new(
                &self.provider_model(provider_name).unwrap_or_default(),
                &task.prompt,
            );
            let start = Instant::now();
            let resp = self.call_provider(provider_name, &request).await?;
            total_latency_ms += start.elapsed().as_millis() as u64;
            total_cost += (resp.usage.total_tokens as f64 / 1000.0) * 0.002;
            if task.check(&resp.content) {
                correct += 1;
            }
        }

        let task_count = 4usize;
        Ok(crate::core::nt_core_consciousness_tree::ProviderBenchmark {
            provider: provider_name.to_string(),
            model: self
                .provider_model(provider_name)
                .unwrap_or_else(|| provider_name.to_string()),
            accuracy: correct as f64 / task_count as f64,
            latency_ms: total_latency_ms / task_count as u64,
            cost_usd: total_cost,
            task_type: task_type.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Deterministic challenge suite — answers are exact-match scored.
    pub(super) fn challenge_tasks(&self, task_type: &str) -> Vec<ChallengeTask> {
        match task_type {
            "arithmetic" => vec![
                ChallengeTask { prompt: "What is 17 + 25? Answer with the number only.".into(), expected: "42".into() },
                ChallengeTask { prompt: "What is 9 * 8? Answer with the number only.".into(), expected: "72".into() },
                ChallengeTask { prompt: "What is 100 - 37? Answer with the number only.".into(), expected: "63".into() },
                ChallengeTask { prompt: "What is 15 + 15 + 15? Answer with the number only.".into(), expected: "45".into() },
            ],
            "extraction" => vec![
                ChallengeTask { prompt: "Extract the email from: 'Contact alice@example.com for info'. Reply with the email only.".into(), expected: "alice@example.com".into() },
                ChallengeTask { prompt: "Extract the date from: 'The event is on 2026-07-31'. Reply with the date only.".into(), expected: "2026-07-31".into() },
                ChallengeTask { prompt: "Extract the city from: 'She lives in Shanghai, China'. Reply with the city only.".into(), expected: "Shanghai".into() },
                ChallengeTask { prompt: "Extract the number from: 'There are 42 apples'. Reply with the number only.".into(), expected: "42".into() },
            ],
            _ => vec![
                ChallengeTask { prompt: "Is 2 + 2 equal to 4? Answer yes or no.".into(), expected: "yes".into() },
                ChallengeTask { prompt: "Is 3 + 3 equal to 7? Answer yes or no.".into(), expected: "no".into() },
                ChallengeTask { prompt: "What color is the sky on a clear day? One word.".into(), expected: "blue".into() },
                ChallengeTask { prompt: "How many legs does a dog have? One digit.".into(), expected: "4".into() },
            ],
        }
    }

    /// Extract model id from `{provider}/{model_id}` registration names.
    /// 无 `/` 的 keyless 注册名 (如 `pollinations`) 回退到 catalog 默认模型,
    /// 避免把注册名当模型名发给端点 (pollinations 会 404 "Model not found")。
    /// 非 keyless 的 provider (如 `openai`) 保持返回注册名本身。
    pub(super) fn provider_model(&self, provider_name: &str) -> Option<String> {
        let model = provider_name
            .split('/')
            .next_back()
            .unwrap_or(provider_name);
        if model.is_empty() {
            return None;
        }
        if model == provider_name {
            // 无 `/` → 仅 keyless provider 回退到 catalog 默认模型
            if let Some(info) = lookup_provider(provider_name) {
                if info.is_free && !info.default_model.is_empty() {
                    return Some(info.default_model.to_string());
                }
            }
        }
        Some(model.to_string())
    }

    /// F7: Ori-Eval 生产接线 (R-P79) — 以自身 (GatewayV2 实现 LlmProvider) 作为
    /// 候选模型执行 Ori-Eval 用例集, 返回 per-model 分数表 + 排名 (选模型依据)。
    /// `cases`: 我们的 agent 提示词集 (F7 OriEvalSuite)。每条用例经完整网关链路
    /// (候选链 → 重试 → 修复/缓存) 执行, 使评估反映真实生产质量。
    pub async fn run_ori_eval_self(
        &self,
        cases: Vec<OriEvalCase>,
        model_names: &[&str],
    ) -> Result<OriEvalReport, LlmError> {
        let suite = OriEvalSuite::new(cases);
        let mut scores = Vec::new();
        for name in model_names {
            let score = suite.score_with_provider(name, self).await?;
            scores.push(score);
        }
        Ok(OriEvalSuite::finalize_report(scores))
    }
}

/// LLM Challenge deterministic task — exact-match scored benchmark item.
pub(super) struct ChallengeTask {
    prompt: String,
    expected: String,
}

impl ChallengeTask {
    pub(super) fn check(&self, response: &str) -> bool {
        response
            .to_lowercase()
            .contains(&self.expected.to_lowercase())
    }
}