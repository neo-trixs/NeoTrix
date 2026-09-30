//! NT-JEV-AGENTJEV — AgentJev-0.6B sidecar 桥（TypeSafe JEV 系决策模型生产落地）
//!
//! 上游：malevrigns/agent-jev（Apache-2.0），Qwen3-0.6B 去 LM 头 + 候选头，
//! 三原语 boolean/choice/score，单次前向零解码出分布。权重已归档至
//! `~/Downloads/Neo/neotrix-archive/models/agent-jev/`，sidecar 照其 README
//! 跑在 127.0.0.1:8149。
//!
//! 安全说明：nt_http 的 SSRF guard 拒绝回环地址，而 sidecar 按上游设计只绑
//! 127.0.0.1。本桥**不走** nt_http，改用直连 reqwest blocking，但 URL 主机
//! 写死 allowlist（127.0.0.1/::1 + 端口），主机名永不来自外部输入，
//! 因此不存在 SSRF 向量（攻击者无法把请求指到别处）。
//! 无 unwrap / expect / panic；无 `[]` 索引。

use std::collections::HashMap;
use std::time::Duration;

use crate::l5_cognition::nt_jev::primitives::{
    ChoiceAnswer, DecisionStatus, JevDecision, NoulAnswer, ScoreAnswer,
};

/// 默认 sidecar 端口（上游 README 默认值）
pub const DEFAULT_PORT: u16 = 8149;
/// 单次 evaluate 超时（上游 P90 ~85ms CUDA，CPU 放宽到 30s）
pub const EVAL_TIMEOUT_SECS: u64 = 30;

/// 提问（对齐上游 /api/evaluate 契约）
#[derive(Debug, Clone)]
pub enum AgentJevQuestion {
    Boolean {
        id: String,
        question: String,
        criteria_true: String,
        criteria_false: String,
    },
    Choice {
        id: String,
        question: String,
        options: Vec<(String, String)>,
    },
    Score {
        id: String,
        question: String,
        levels: Vec<String>,
    },
}

impl AgentJevQuestion {
    fn qid(&self) -> &str {
        match self {
            Self::Boolean { id, .. } => id,
            Self::Choice { id, .. } => id,
            Self::Score { id, .. } => id,
        }
    }

    fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Boolean {
                id,
                question,
                criteria_true,
                criteria_false,
            } => serde_json::json!({
                "id": id,
                "type": "boolean",
                "question": question,
                "criteria": {"true": criteria_true, "false": criteria_false},
            }),
            Self::Choice {
                id,
                question,
                options,
            } => {
                let map: HashMap<&str, &str> = options
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                serde_json::json!({
                    "id": id,
                    "type": "choice",
                    "question": question,
                    "options": map,
                })
            }
            Self::Score {
                id,
                question,
                levels,
            } => serde_json::json!({
                "id": id,
                "type": "score",
                "question": question,
                "levels": levels,
            }),
        }
    }
}

pub struct NtJevAgentJev;

impl NtJevAgentJev {
    /// 仅回环主机 allowlist（防 SSRF：sidecar 只绑本地，主机名不接受外部输入）
    fn base_url(port: u16) -> Result<String, String> {
        if port == 0 {
            return Err("agentjev: port 0 rejected".to_string());
        }
        Ok(format!("http://127.0.0.1:{port}"))
    }

    fn client() -> Result<reqwest::blocking::Client, String> {
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(EVAL_TIMEOUT_SECS))
            .build()
            .map_err(|e| format!("agentjev client: {e}"))
    }

    /// 健康检查（GET /health）
    pub fn health(port: u16) -> Result<bool, String> {
        let url = format!("{}/health", Self::base_url(port)?);
        let body = Self::client()?
            .get(&url)
            .send()
            .map_err(|e| format!("agentjev health: {e}"))?
            .text()
            .map_err(|e| format!("agentjev health body: {e}"))?;
        Ok(body.contains("ok") || body.contains("healthy") || body.contains("status"))
    }

    /// 批量求值（POST /api/evaluate）→ 按 id 配对的决策表
    pub fn evaluate(
        port: u16,
        state: &serde_json::Value,
        questions: &[AgentJevQuestion],
    ) -> Result<HashMap<String, JevDecision>, String> {
        if questions.is_empty() {
            return Ok(HashMap::new());
        }
        let url = format!("{}/api/evaluate", Self::base_url(port)?);
        let qs: Vec<serde_json::Value> = questions.iter().map(|q| q.to_json()).collect();
        let body = serde_json::json!({"state": state, "questions": qs});
        let resp: serde_json::Value = Self::client()?
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("agentjev evaluate: {e}"))?
            .json()
            .map_err(|e| format!("agentjev evaluate body: {e}"))?;
        Self::parse_response(&resp, questions)
    }

    /// 响应解析（纯函数，fixture 可测）
    pub fn parse_response(
        resp: &serde_json::Value,
        questions: &[AgentJevQuestion],
    ) -> Result<HashMap<String, JevDecision>, String> {
        let mut out = HashMap::new();
        let results = resp
            .get("results")
            .and_then(|r| r.as_array())
            .ok_or("agentjev: missing results[]")?;
        // 按问题 id 建表（上游 answers 与 questions 按 id 配对）
        let mut by_id: HashMap<&str, &AgentJevQuestion> = HashMap::new();
        for q in questions {
            by_id.insert(q.qid(), q);
        }
        for req in results {
            let answers = req
                .get("answers")
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default();
            for ans in &answers {
                let id = ans
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let q = match by_id.get(id) {
                    Some(q) => q,
                    None => continue,
                };
                if let Some(d) = Self::parse_answer(ans, q) {
                    out.insert(id.to_string(), d);
                }
            }
        }
        Ok(out)
    }

    fn parse_answer(ans: &serde_json::Value, q: &AgentJevQuestion) -> Option<JevDecision> {
        match q {
            AgentJevQuestion::Boolean { .. } => {
                let p = ans.get("probability").and_then(|v| v.as_f64())?;
                let v = ans.get("value").and_then(|x| x.as_bool()).unwrap_or(p > 0.5);
                Some(JevDecision::Noul(NoulAnswer {
                    noul: if v { p.clamp(0.0, 1.0) } else { (1.0 - p).clamp(0.0, 1.0) },
                    needs_review: false,
                    reason: None,
                    status: DecisionStatus::Selected,
                }))
            }
            AgentJevQuestion::Choice { options, .. } => {
                let dist_src = ans.get("distribution")?;
                let mut probs = HashMap::new();
                for (k, desc) in options {
                    let _ = desc;
                    if let Some(p) = dist_src.get(k).and_then(|v| v.as_f64()) {
                        probs.insert(k.clone(), p.clamp(0.0, 1.0));
                    }
                }
                if probs.is_empty() {
                    return None;
                }
                let top = ans
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        probs
                            .iter()
                            .max_by(|a, b| {
                                a.1.partial_cmp(b.1)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            })
                            .map(|(k, _)| k.clone())
                    })?;
                let mut sorted: Vec<f64> = probs.values().cloned().collect();
                sorted.sort_by(|a, b| {
                    b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
                });
                let first = sorted.first().copied().unwrap_or(0.0);
                let second = sorted.get(1).copied().unwrap_or(0.0);
                Some(JevDecision::Choice(ChoiceAnswer {
                    choice: top,
                    probabilities: probs,
                    confidence: first,
                    margin: (first - second).max(0.0),
                    needs_review: first < 0.7,
                    reason: None,
                    status: if first < 0.7 {
                        DecisionStatus::Review
                    } else {
                        DecisionStatus::Selected
                    },
                }))
            }
            AgentJevQuestion::Score { levels, .. } => {
                let level = ans.get("level").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let score = ans.get("score").and_then(|v| v.as_f64()).unwrap_or(level as f64);
                let mut probs = HashMap::new();
                if let Some(dist) = ans.get("distribution").and_then(|v| v.as_object()) {
                    for (k, v) in dist {
                        if let Some(p) = v.as_f64() {
                            probs.insert(k.clone(), p.clamp(0.0, 1.0));
                        }
                    }
                }
                if probs.is_empty() {
                    // 无分布时退化为 one-hot（保持可评估）
                    probs.insert(level.to_string(), 1.0);
                }
                let conf = probs.values().cloned().fold(0.0f64, f64::max);
                Some(JevDecision::Score(ScoreAnswer {
                    score,
                    probabilities: probs,
                    confidence: conf,
                    legend: levels.clone(),
                    needs_review: conf < 0.7,
                    reason: None,
                    status: if conf < 0.7 {
                        DecisionStatus::Review
                    } else {
                        DecisionStatus::Selected
                    },
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boolean_q() -> AgentJevQuestion {
        AgentJevQuestion::Boolean {
            id: "done".to_string(),
            question: "Are all tests passing?".to_string(),
            criteria_true: "suite green".to_string(),
            criteria_false: "failing".to_string(),
        }
    }

    fn choice_q() -> AgentJevQuestion {
        AgentJevQuestion::Choice {
            id: "next".to_string(),
            question: "What next?".to_string(),
            options: vec![
                ("debug_failure".to_string(), "Read assertion.".to_string()),
                ("submit_patch".to_string(), "Open PR.".to_string()),
            ],
        }
    }

    #[test]
    fn test_base_url_rejects_zero_port() {
        assert!(NtJevAgentJev::base_url(0).is_err());
        assert_eq!(
            NtJevAgentJev::base_url(8149).unwrap(),
            "http://127.0.0.1:8149"
        );
    }

    #[test]
    fn test_question_json_shapes() {
        let b = boolean_q().to_json();
        assert_eq!(b.get("type").and_then(|v| v.as_str()), Some("boolean"));
        assert!(b.get("criteria").is_some());
        let c = choice_q().to_json();
        assert_eq!(c.get("type").and_then(|v| v.as_str()), Some("choice"));
        let s = AgentJevQuestion::Score {
            id: "r".to_string(),
            question: "risk?".to_string(),
            levels: vec!["low".to_string(), "high".to_string()],
        }
        .to_json();
        assert_eq!(s.get("type").and_then(|v| v.as_str()), Some("score"));
    }

    #[test]
    fn test_parse_boolean_and_choice_fixture() {
        // 上游 README 示例响应形状
        let resp: serde_json::Value = serde_json::from_str(
            r#"{"api_version":"agentjev.decision.v1","results":[{"id":"0","answers":[
                {"id":"done","type":"boolean","probability":0.08,"value":false,"distribution":{"true":0.08,"false":0.92}},
                {"id":"next","type":"choice","value":"debug_failure","top_probability":0.87,"margin":0.74,"distribution":{"debug_failure":0.87,"submit_patch":0.13}}
            ]}],"usage":{"generated_tokens":0}}"#,
        )
        .unwrap();
        let qs = vec![boolean_q(), choice_q()];
        let out = NtJevAgentJev::parse_response(&resp, &qs).unwrap();
        assert_eq!(out.len(), 2);
        match out.get("done").unwrap() {
            JevDecision::Noul(n) => assert!((n.noul - 0.92).abs() < 1e-9),
            _ => panic!("must be noul"),
        }
        match out.get("next").unwrap() {
            JevDecision::Choice(c) => {
                assert_eq!(c.choice, "debug_failure");
                assert!((c.margin - 0.74).abs() < 1e-9);
            }
            _ => panic!("must be choice"),
        }
    }

    #[test]
    fn test_parse_missing_results_errors() {
        let resp = serde_json::json!({"nope": 1});
        assert!(NtJevAgentJev::parse_response(&resp, &[boolean_q()]).is_err());
    }

    #[test]
    fn test_parse_unknown_question_id_skipped() {
        let resp = serde_json::json!({"results": [{"id": "0", "answers": [
            {"id": "ghost", "type": "boolean", "probability": 0.5, "value": true}
        ]}]});
        let out = NtJevAgentJev::parse_response(&resp, &[boolean_q()]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn test_score_fallback_one_hot() {
        let q = AgentJevQuestion::Score {
            id: "r".to_string(),
            question: "risk?".to_string(),
            levels: vec!["low".to_string(), "high".to_string()],
        };
        let resp =
            serde_json::json!({"results": [{"id": "0", "answers": [{"id": "r", "level": 1, "score": 1.0}]}]});
        let out = NtJevAgentJev::parse_response(&resp, std::slice::from_ref(&q)).unwrap();
        match out.get("r").unwrap() {
            JevDecision::Score(s) => {
                assert_eq!(s.legend.len(), 2);
                assert!((s.confidence - 1.0).abs() < 1e-9);
            }
            _ => panic!("must be score"),
        }
    }

    /// 活服务冒烟（默认忽略）：sidecar 跑在 :8149 即激活。
    /// `cargo test -p neotrix --lib live_agentjev -- --ignored --nocapture`
    #[test]
    #[ignore = "needs agentjev sidecar on 127.0.0.1:8149"]
    fn live_agentjev_health() {
        if !NtJevAgentJev::health(DEFAULT_PORT).unwrap_or(false) {
            return;
        }
        let qs = vec![boolean_q()];
        let out = NtJevAgentJev::evaluate(
            DEFAULT_PORT,
            &serde_json::json!("23 tests passed, 1 failed."),
            &qs,
        )
        .expect("evaluate must succeed");
        assert!(out.contains_key("done"));
    }
}
