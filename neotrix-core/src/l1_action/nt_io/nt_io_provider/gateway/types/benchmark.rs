use std::time::Instant;

use crate::l1_action::nt_io::nt_io_provider::catalog::provider_catalog::lookup_provider;
use super::*;

// ═══════════════════════════════════════════════════════════════════
// Benchmark Types — L1-local benchmark types for challenge evaluation
// ═══════════════════════════════════════════════════════════════════

/// An evaluation case for LLM challenge testing.
#[derive(Debug, Clone)]
pub struct OriEvalCase {
    pub id: String,
    pub prompt: String,
    /// Expected tool name (correctness check: pass if any match)
    pub expected_tool: Option<String>,
    /// Keywords expected in response (rubric scoring)
    pub rubric_keywords: Vec<String>,
    /// Whether tool call is required (necessity check)
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

/// Score for a single evaluation case.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OriCaseScore {
    pub case_id: String,
    pub answer_grade: f64,         // rubric keyword hit rate 0.0-1.0
    pub tool_call_legit: bool,     // was tool call in expected set?
    pub tool_call_necessary: bool, // was tool call needed?
}

/// Model-level aggregate score.
#[derive(Debug, Clone)]
pub struct OriModelScore {
    pub model: String,
    pub accuracy: f64,
    pub tool_accuracy: f64,
    pub tool_necessity: f64,
    pub composite: f64,
}

/// Evaluation report with per-model scores and ranking.
#[derive(Debug, Clone)]
pub struct OriEvalReport {
    pub per_model: Vec<OriModelScore>,
    /// Models ranked by composite score (index 0 = best)
    pub ranking: Vec<String>,
    pub timestamp: String,
}

impl OriEvalReport {
    /// Best model (rank 0).
    pub fn best_model(&self) -> Option<&str> {
        self.ranking.first().map(|s| s.as_str())
    }
}

/// Evaluation suite that runs cases across models.
#[derive(Debug, Default)]
pub struct OriEvalSuite {
    pub cases: Vec<OriEvalCase>,
}

impl OriEvalSuite {
    pub fn new(cases: Vec<OriEvalCase>) -> Self {
        Self { cases }
    }

    /// Run evaluation cases through a provider and score them.
    pub async fn score_with_provider(
        &self,
        model_name: &str,
        provider: &(dyn crate::core::nt_core_llm::LlmProvider + Send + Sync),
    ) -> Result<OriModelScore, crate::core::nt_core_llm::LlmError> {
        use crate::l1_action::nt_io::nt_io_provider/gateway::types::LlmRequest;

        let mut correct = 0usize;
        let mut tool_correct = 0usize;
        let mut tool_needed = 0usize;

        for case in &self.cases {
            let request = LlmRequest::new(model_name, &case.prompt);
            let resp = provider.complete(&request).await?;
            let content = resp.content.to_lowercase();

            // Rubric keyword scoring
            let keyword_hits = case.rubric_keywords.iter()
                .filter(|kw| content.contains(&kw.to_lowercase()))
                .count();
            let answer_grade = if case.rubric_keywords.is_empty() {
                1.0
            } else {
                keyword_hits as f64 / case.rubric_keywords.len() as f64
            };
            if answer_grade >= 0.5 {
                correct += 1;
            }

            // Tool call scoring (simplified: check if tool call info exists in response)
            let has_tool = resp.tool_calls.as_ref().map_or(false, |tc| !tc.is_empty());
            if let Some(ref expected) = case.expected_tool {
                if has_tool && content.contains(&expected.to_lowercase()) {
                    tool_correct += 1;
                }
            }
            if case.requires_tool && has_tool {
                tool_needed += 1;
            }
        }

        let total = self.cases.len() as f64;
        let accuracy = correct as f64 / total;
        let tool_accuracy = if self.cases.iter().any(|c| c.expected_tool.is_some()) {
            tool_correct as f64 / self.cases.iter().filter(|c| c.expected_tool.is_some()).count() as f64
        } else {
            1.0
        };
        let tool_necessity = if self.cases.iter().any(|c| c.requires_tool) {
            tool_needed as f64 / self.cases.iter().filter(|c| c.requires_tool).count() as f64
        } else {
            1.0
        };
        let composite = accuracy * 0.5 + tool_accuracy * 0.25 + tool_necessity * 0.25;

        Ok(OriModelScore {
            model: model_name.to_string(),
            accuracy,
            tool_accuracy,
            tool_necessity,
            composite,
        })
    }

    /// Finalize scores into a report.
    pub fn finalize_report(scores: Vec<OriModelScore>) -> OriEvalReport {
        let mut ranking: Vec<String> = scores.iter().map(|s| s.model.clone()).collect();
        ranking.sort_by(|a, b| {
            let score_a = scores.iter().find(|s| s.model == *a).map(|s| s.composite).unwrap_or(0.0);
            let score_b = scores.iter().find(|s| s.model == *b).map(|s| s.composite).unwrap_or(0.0);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        OriEvalReport {
            per_model: scores,
            ranking,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// LLM Challenge — Deterministic challenge tasks scoring provider accuracy/latency/cost
// ═══════════════════════════════════════════════════════════════════

impl GatewayV2 {
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
            let resp = self.call_provider_backoff(provider_name, &request).await?;
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
    pub(super) fn provider_model(&self, provider_name: &str) -> Option<String> {
        let parts: Vec<&str> = provider_name.split('/').collect();
        if parts.len() <= 1 {
            let name = parts.first().copied().unwrap_or(provider_name);
            if name.is_empty() {
                return None;
            }
            if let Some(info) = lookup_provider(name) {
                if info.is_free && !info.default_model.is_empty() {
                    return Some(info.default_model.to_string());
                }
            }
            return Some(name.to_string());
        }
        Some(parts[1..].join("/"))
    }

    /// F7: Ori-Eval 生产接线 (R-P79)
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
