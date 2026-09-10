//! L1-local benchmark types for challenge evaluation.
//!
//! Breaks the upward L1→L5 dependency by defining the types here in L1.
//! L5's benchmark module can adapt these types for its own use.

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
        use super::super::LlmRequest;

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