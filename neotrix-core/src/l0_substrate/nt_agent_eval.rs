//! `nt_agent_eval` — 极简 agent 评测台（harness-evals 式：Score 0-1 + threshold pass/fail）。
//!
//! 与吞吐型 `memory_bench` 基准互补：此处只定义评测契约
//! （`EvalCase`/`Score`/`evaluate`），真实评测用例在调用方填充。
//! 纯逻辑、零 I/O、零时钟，可在任何层经 L0 facade 取用。

/// 单条评测用例：输入、标准答案、评分器。
pub struct EvalCase<'a> {
    /// 喂给 agent 的 prompt。
    pub prompt: &'a str,
    /// 期望命中的 gold 片段（空串 = 不做 gold 匹配）。
    pub gold: &'a str,
    /// 评分器：对 agent 输出打分并归一化到 0..=1。
    pub scorer: fn(&str, &str) -> f64,
}

/// 归一化分数（0..=1）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Score(pub f64);

impl Score {
    /// 阈值 pass/fail（harness-evals 语义：无魔法、可配置阈值）。
    #[must_use]
    pub fn pass(self, threshold: f64) -> bool {
        self.0 >= threshold
    }
}

/// 内置 scorer：输出是否包含 gold 片段（二值 1.0/0.0）。
#[must_use]
pub fn contains_gold(output: &str, gold: &str) -> f64 {
    if gold.is_empty() || output.contains(gold) {
        1.0
    } else {
        0.0
    }
}

/// karotte 式 transcript 判定：把输出当 JSON 解析，检查 `status` 字段
/// 等于 gold（如 `"completed"`）。非 JSON / 缺字段 ⇒ 0.0。
/// 对齐 karotte 的「task 模板 + transcript.json + 终态判定」契约，
/// 本层不跑 VM，只吃 transcript 文本。
#[must_use]
pub fn transcript_success_scorer(output: &str, gold: &str) -> f64 {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(output);
    match parsed {
        Ok(v) => match v.get("status").and_then(|s| s.as_str()) {
            Some(s) if gold.is_empty() || s == gold => 1.0,
            _ => 0.0,
        },
        Err(_) => 0.0,
    }
}

/// 跑一组 case：返回 (mean_score, pass_rate)。
#[must_use]
pub fn evaluate(cases: &[EvalCase<'_>], outputs: &[&str], threshold: f64) -> (f64, f64) {
    let mut sum = 0.0;
    let mut passed = 0usize;
    for (case, output) in cases.iter().zip(outputs.iter()) {
        let score = (case.scorer)(output, case.gold);
        sum += score;
        if Score(score).pass(threshold) {
            passed += 1;
        }
    }
    let n = cases.len().max(1) as f64;
    (sum / n, passed as f64 / n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_threshold_pass_fail() {
        assert!(Score(0.8).pass(0.5));
        assert!(!Score(0.4).pass(0.5));
    }

    #[test]
    fn contains_gold_scorer_binary() {
        assert_eq!(contains_gold("the answer is 2", "2"), 1.0);
        assert_eq!(contains_gold("no idea", "2"), 0.0);
        assert_eq!(contains_gold("anything", ""), 1.0);
    }

    #[test]
    fn transcript_success_scorer_parses_status() {
        assert_eq!(transcript_success_scorer(r#"{"status":"completed"}"#, "completed"), 1.0);
        assert_eq!(transcript_success_scorer(r#"{"status":"failed"}"#, "completed"), 0.0);
        assert_eq!(transcript_success_scorer("not json", "completed"), 0.0);
        assert_eq!(transcript_success_scorer(r#"{"other":1}"#, "completed"), 0.0);
    }

    #[test]
    fn evaluate_mean_and_pass_rate() {
        let cases = [
            EvalCase { prompt: "p1", gold: "a", scorer: contains_gold },
            EvalCase { prompt: "p2", gold: "b", scorer: contains_gold },
        ];
        let (mean, rate) = evaluate(&cases, &["a", "nope"], 0.5);
        assert_eq!(mean, 0.5);
        assert_eq!(rate, 0.5);
    }
}
