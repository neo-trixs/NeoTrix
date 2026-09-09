//! Benchmark shared types — used by both L1 (gateway challenge) and L5 (benchmark suite).

use serde::{Deserialize, Serialize};

/// Ori-Eval test case for provider benchmarking.
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

/// 一个用例的 Ori-Eval 评分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriCaseScore {
    pub case_id: String,
    pub answer_grade: f64,
    pub tool_call_legit: bool,
    pub tool_call_necessary: bool,
}

impl OriCaseScore {
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
    pub tool_call_accuracy: f64,
    pub tool_necessity: f64,
    pub composite: f64,
}

/// Ori-Eval report with per-model scores and rankings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriEvalReport {
    pub per_model: Vec<OriModelScore>,
    pub ranking: Vec<String>,
    pub timestamp: String,
}

impl OriEvalReport {
    pub fn best_model(&self) -> Option<&str> {
        self.ranking.first().map(|s| s.as_str())
    }
}

/// Ori-Eval suite — holds test cases. Provider-specific methods live in L5.
pub struct OriEvalSuite {
    pub cases: Vec<OriEvalCase>,
}

impl OriEvalSuite {
    pub fn new(cases: Vec<OriEvalCase>) -> Self {
        Self { cases }
    }

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
