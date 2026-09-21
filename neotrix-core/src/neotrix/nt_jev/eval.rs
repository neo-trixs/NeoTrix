//! JEV Eval — S1Bench-style metrics: accuracy, ECE, Brier, coverage@precision, latency.
//!
//! Measures JEV decision quality against a labelled gold set, mirroring the
//! public S1Bench board: macro accuracy, expected calibration error (ECE),
//! Brier score, coverage at a precision target, and throughput (decisions/s
//! via mean latency).
//!
//! Only `Noul` and `Choice` decisions are scored: `Score` decisions always
//! report `predicted_prob == None` and `is_correct == false` (type mismatch
//! against [`GoldAnswer`], which has no score variant).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::primitives::JevDecision;

// ═══════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════

/// Gold-standard answer for one eval case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoldAnswer {
    /// Expected boolean outcome.
    Noul(bool),
    /// Expected winning option key.
    Choice(String),
}

/// One labelled eval case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    /// Case id; predictions join on this.
    pub id: String,
    /// Gold-standard answer.
    pub gold: GoldAnswer,
}

/// One model prediction for a case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalPrediction {
    /// Must match an [`EvalCase::id`]; unknown ids are ignored.
    pub case_id: String,
    /// Model decision.
    pub decision: JevDecision,
    /// Wall-clock latency for this decision.
    pub latency_ms: u64,
}

/// Aggregate S1Bench-style report over joined case/prediction pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    /// Number of joined pairs (cases with a matching prediction).
    pub n: usize,
    /// Fraction correct.
    pub accuracy: f64,
    /// Mean squared error of gold-outcome probabilities (skips `None`s).
    pub brier: f64,
    /// Expected calibration error over `(confidence, correct)` (10 bins).
    pub ece: f64,
    /// Max fraction of items retained at precision >= 0.9.
    pub coverage_at_p90: f64,
    /// Mean prediction latency in milliseconds.
    pub mean_latency_ms: f64,
}

// ═══════════════════════════════════════════════════════════════════
// Per-decision scoring
// ═══════════════════════════════════════════════════════════════════

/// Probability the decision assigned to the gold outcome.
///
/// - Noul: `p` if gold is true, else `1 - p`.
/// - Choice: `probabilities[gold]`; `None` if the key is missing.
/// - Anything else (type mismatch, `Score`): `None`.
pub fn predicted_prob(decision: &JevDecision, gold: &GoldAnswer) -> Option<f64> {
    match (decision, gold) {
        (JevDecision::Noul(n), GoldAnswer::Noul(g)) => Some(if *g { n.noul } else { 1.0 - n.noul }),
        (JevDecision::Choice(c), GoldAnswer::Choice(g)) => c.probabilities.get(g).copied(),
        _ => None,
    }
}

/// Whether the decision's top answer matches gold.
///
/// - Noul: `(p > 0.5) == gold` (p == 0.5 predicts false).
/// - Choice: `choice == gold`.
/// - Anything else (type mismatch, `Score`): false.
pub fn is_correct(decision: &JevDecision, gold: &GoldAnswer) -> bool {
    match (decision, gold) {
        (JevDecision::Noul(n), GoldAnswer::Noul(g)) => (n.noul > 0.5) == *g,
        (JevDecision::Choice(c), GoldAnswer::Choice(g)) => &c.choice == g,
        _ => false,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Aggregate metrics
// ═══════════════════════════════════════════════════════════════════

/// Brier score: mean of `(p - y)^2` with `y` in {0, 1}.
///
/// `items` holds `(prob of gold outcome, correct?)`. Empty input → 0.0.
pub fn brier_score(items: &[(f64 /* prob of gold */, bool /* correct? */)]) -> f64 {
    if items.is_empty() {
        return 0.0;
    }
    let sum: f64 = items
        .iter()
        .map(|(p, correct)| {
            let y = if *correct { 1.0 } else { 0.0 };
            (p - y) * (p - y)
        })
        .sum();
    sum / items.len() as f64
}

/// Expected calibration error with EQUAL-MASS bins.
///
/// Sorts by confidence, splits into `bins.min(n)` groups of ~equal count
/// (first groups take the remainder), and sums `|acc - conf| * (count / n)`.
/// Empty input → 0.0; `bins < 1` is treated as 1.
pub fn ece(conf_correct: &[(f64 /* confidence */, bool /* correct */)], bins: usize) -> f64 {
    let n = conf_correct.len();
    if n == 0 {
        return 0.0;
    }
    let nb = bins.max(1).min(n);
    let mut sorted: Vec<(f64, bool)> = conf_correct.to_vec();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let base = n / nb;
    let rem = n % nb;
    let mut total = 0.0;
    let mut start = 0;
    for b in 0..nb {
        let size = base + if b < rem { 1 } else { 0 };
        let end = start + size;
        let slice = &sorted[start..end];
        let acc = slice.iter().filter(|(_, c)| *c).count() as f64 / slice.len() as f64;
        let conf: f64 = slice.iter().map(|(c, _)| c).sum::<f64>() / slice.len() as f64;
        total += (acc - conf).abs() * (slice.len() as f64 / n as f64);
        start = end;
    }
    total
}

/// Max fraction of items retained at precision >= `target`.
///
/// Scans thresholds over the unique confidence values (keep items with
/// `confidence >= t`); returns the largest kept fraction whose precision
/// meets the target, or 0.0 if none does.
pub fn coverage_at_precision(conf_correct: &[(f64, bool)], target: f64) -> f64 {
    let n = conf_correct.len();
    if n == 0 {
        return 0.0;
    }
    let mut thresholds: Vec<f64> = conf_correct.iter().map(|(c, _)| *c).collect();
    thresholds.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    thresholds.dedup();
    let mut best: f64 = 0.0;
    for t in thresholds {
        let mut kept = 0usize;
        let mut hits = 0usize;
        for (c, correct) in conf_correct {
            if *c >= t {
                kept += 1;
                if *correct {
                    hits += 1;
                }
            }
        }
        if kept > 0 && hits as f64 / kept as f64 >= target {
            best = best.max(kept as f64 / n as f64);
        }
    }
    best
}

/// Join cases with predictions by `case_id` and score the aggregate.
///
/// - Predictions with unknown ids are ignored; cases without predictions
///   are skipped (no panics on any input; `n == 0` → all zeros).
/// - `accuracy` = correct / n.
/// - `brier` over [`predicted_prob`] values, skipping `None`s.
/// Confidence in the *predicted* outcome for calibration purposes.
///
/// For `Noul` this is `max(p, 1-p)` — note [`JevDecision::confidence`]
/// returns raw `p`, which understates confidence for confident-false
/// predictions and would corrupt ECE/coverage. Choice/Score pass through.
pub fn prediction_confidence(decision: &JevDecision) -> f64 {
    match decision {
        JevDecision::Noul(n) => n.noul.max(1.0 - n.noul),
        other => other.confidence(),
    }
}

/// - `ece` over `(prediction_confidence(), correct)` with 10 bins.
/// - `coverage_at_p90` = [`coverage_at_precision`] at target 0.9.
/// - `mean_latency_ms` over joined predictions.
pub fn evaluate(cases: &[EvalCase], preds: &[EvalPrediction]) -> EvalReport {
    let zeros = EvalReport {
        n: 0,
        accuracy: 0.0,
        brier: 0.0,
        ece: 0.0,
        coverage_at_p90: 0.0,
        mean_latency_ms: 0.0,
    };
    let gold_by_id: HashMap<&str, &GoldAnswer> =
        cases.iter().map(|c| (c.id.as_str(), &c.gold)).collect();
    let mut joined: Vec<(&GoldAnswer, &JevDecision, u64)> = Vec::new();
    for p in preds {
        if let Some(gold) = gold_by_id.get(p.case_id.as_str()) {
            joined.push((gold, &p.decision, p.latency_ms));
        }
    }
    let n = joined.len();
    if n == 0 {
        return zeros;
    }
    let correct: Vec<bool> = joined
        .iter()
        .map(|(gold, decision, _)| is_correct(decision, gold))
        .collect();
    let accuracy = correct.iter().filter(|&&c| c).count() as f64 / n as f64;
    let brier_items: Vec<(f64, bool)> = joined
        .iter()
        .zip(correct.iter())
        .filter_map(|((gold, decision, _), &c)| predicted_prob(decision, gold).map(|p| (p, c)))
        .collect();
    let brier = brier_score(&brier_items);
    let conf_items: Vec<(f64, bool)> = joined
        .iter()
        .zip(correct.iter())
        .map(|((_, decision, _), &c)| (prediction_confidence(decision), c))
        .collect();
    let ece_value = ece(&conf_items, 10);
    let coverage_at_p90 = coverage_at_precision(&conf_items, 0.9);
    let mean_latency_ms = joined.iter().map(|(_, _, l)| *l as f64).sum::<f64>() / n as f64;
    EvalReport {
        n,
        accuracy,
        brier,
        ece: ece_value,
        coverage_at_p90,
        mean_latency_ms,
    }
}

// ═══════════════════════════════════════════════════════════════════
// Golden-pack persistence (JSONL, string-level — callers own the fs)
// ═══════════════════════════════════════════════════════════════════

/// Serialize cases to JSONL (one object per line) for golden-pack storage.
pub fn cases_to_jsonl(cases: &[EvalCase]) -> Result<String, serde_json::Error> {
    let mut out = String::new();
    for c in cases {
        out.push_str(&serde_json::to_string(c)?);
        out.push('\n');
    }
    Ok(out)
}

/// Parse JSONL back into cases (blank lines skipped).
pub fn cases_from_jsonl(s: &str) -> Result<Vec<EvalCase>, serde_json::Error> {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str)
        .collect()
}

/// Serialize predictions to JSONL (record/replay for CI: run `check` on the
/// recording without calling any model).
pub fn preds_to_jsonl(preds: &[EvalPrediction]) -> Result<String, serde_json::Error> {
    let mut out = String::new();
    for p in preds {
        out.push_str(&serde_json::to_string(p)?);
        out.push('\n');
    }
    Ok(out)
}

/// Parse JSONL back into predictions (blank lines skipped).
pub fn preds_from_jsonl(s: &str) -> Result<Vec<EvalPrediction>, serde_json::Error> {
    s.lines()
        .filter(|l| !l.trim().is_empty())
        .map(serde_json::from_str)
        .collect()
}

/// Serialize a report to compact JSON for nightly archives.
pub fn report_to_json(report: &EvalReport) -> Result<String, serde_json::Error> {
    serde_json::to_string(report)
}

#[cfg(test)]
mod tests {
    use super::super::primitives::{ChoiceAnswer, DecisionStatus, NoulAnswer};
    use super::*;

    fn noul_decision(p: f64) -> JevDecision {
        JevDecision::Noul(NoulAnswer {
            noul: p,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        })
    }

    fn choice_decision(choice: &str, probs: &[(&str, f64)], confidence: f64) -> JevDecision {
        let mut probabilities = HashMap::new();
        for (k, v) in probs {
            probabilities.insert((*k).to_string(), *v);
        }
        JevDecision::Choice(ChoiceAnswer {
            choice: choice.to_string(),
            probabilities,
            confidence,
            margin: 0.0,
            needs_review: false,
            reason: None,
            status: DecisionStatus::Selected,
        })
    }

    #[test]
    fn test_perfect_predictor() {
        let cases = vec![
            EvalCase {
                id: "c1".into(),
                gold: GoldAnswer::Noul(true),
            },
            EvalCase {
                id: "c2".into(),
                gold: GoldAnswer::Choice("a".into()),
            },
        ];
        let preds = vec![
            EvalPrediction {
                case_id: "c1".into(),
                decision: noul_decision(1.0),
                latency_ms: 10,
            },
            EvalPrediction {
                case_id: "c2".into(),
                decision: choice_decision("a", &[("a", 1.0), ("b", 0.0)], 1.0),
                latency_ms: 20,
            },
        ];
        let r = evaluate(&cases, &preds);
        assert_eq!(r.n, 2);
        assert!((r.accuracy - 1.0).abs() < 1e-12);
        assert!((r.brier - 0.0).abs() < 1e-12);
        assert!((r.ece - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_uncertain_half_correct_brier_quarter() {
        // Noul p=0.5 always; p == 0.5 predicts false, so gold=false is
        // correct and gold=true is wrong → accuracy 0.5.
        let cases: Vec<EvalCase> = (0..10)
            .map(|i| EvalCase {
                id: format!("c{}", i),
                gold: GoldAnswer::Noul(i % 2 == 0),
            })
            .collect();
        let preds: Vec<EvalPrediction> = (0..10)
            .map(|i| EvalPrediction {
                case_id: format!("c{}", i),
                decision: noul_decision(0.5),
                latency_ms: 5,
            })
            .collect();
        let r = evaluate(&cases, &preds);
        assert!((r.accuracy - 0.5).abs() < 1e-12);
        // Every item has prob-of-gold 0.5 → (0.5-y)^2 = 0.25 each.
        assert!((r.brier - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_ece_detects_miscalibration() {
        // 10 items at conf 0.8, 5 correct → |0.5 - 0.8| = 0.3 (single bin).
        let items: Vec<(f64, bool)> = (0..10).map(|i| (0.8, i < 5)).collect();
        let got = ece(&items, 1);
        assert!((got - 0.3).abs() < 1e-9, "got {}", got);
    }

    #[test]
    fn test_ece_empty_and_zero_bins() {
        assert_eq!(ece(&[], 10), 0.0);
        // bins < 1 → treated as 1: single bin acc=1.0, conf=0.8.
        let got = ece(&[(0.8, true)], 0);
        assert!((got - 0.2).abs() < 1e-9, "got {}", got);
    }

    #[test]
    fn test_coverage_separable() {
        // 5 items at conf 0.95 all correct; 5 at conf 0.4 with 2 correct.
        let mut items = vec![(0.95, true); 5];
        items.extend(vec![
            (0.4, true),
            (0.4, true),
            (0.4, false),
            (0.4, false),
            (0.4, false),
        ]);
        // t=0.95 → precision 1.0, coverage 0.5; t=0.4 → precision 0.7 < 0.9.
        let got = coverage_at_precision(&items, 0.9);
        assert!((got - 0.5).abs() < 1e-12, "got {}", got);
    }

    #[test]
    fn test_coverage_none_meets_target() {
        assert_eq!(
            coverage_at_precision(&[(0.5, false), (0.6, false)], 0.9),
            0.0
        );
        assert_eq!(coverage_at_precision(&[], 0.9), 0.0);
    }

    #[test]
    fn test_empty_inputs_no_panic() {
        let r = evaluate(&[], &[]);
        assert_eq!(r.n, 0);
        assert_eq!(r.accuracy, 0.0);
        assert_eq!(r.brier, 0.0);
        assert_eq!(r.ece, 0.0);
        assert_eq!(r.coverage_at_p90, 0.0);
        assert_eq!(r.mean_latency_ms, 0.0);
        assert_eq!(brier_score(&[]), 0.0);
    }

    #[test]
    fn test_type_mismatch_skipped_gracefully() {
        // Choice prediction vs Noul gold: not correct, prob None.
        let d = choice_decision("a", &[("a", 0.9), ("b", 0.1)], 0.8);
        let g = GoldAnswer::Noul(true);
        assert_eq!(predicted_prob(&d, &g), None);
        assert!(!is_correct(&d, &g));
        // Missing key in distribution → None.
        let g2 = GoldAnswer::Choice("zzz".into());
        assert_eq!(predicted_prob(&d, &g2), None);
        // evaluate() skips the None in brier instead of panicking.
        let cases = vec![EvalCase {
            id: "c1".into(),
            gold: GoldAnswer::Noul(true),
        }];
        let preds = vec![EvalPrediction {
            case_id: "c1".into(),
            decision: d,
            latency_ms: 7,
        }];
        let r = evaluate(&cases, &preds);
        assert_eq!(r.n, 1);
        assert_eq!(r.accuracy, 0.0);
        assert_eq!(r.brier, 0.0);
    }

    #[test]
    fn test_predicted_prob_noul_both_sides() {
        assert_eq!(
            predicted_prob(&noul_decision(0.8), &GoldAnswer::Noul(true)),
            Some(0.8)
        );
        // Note: 1.0 - 0.8 is not exactly 0.2 in binary floating point.
        let p = predicted_prob(&noul_decision(0.8), &GoldAnswer::Noul(false)).unwrap();
        assert!((p - 0.2).abs() < 1e-12, "got {}", p);
        assert!(is_correct(&noul_decision(0.8), &GoldAnswer::Noul(true)));
        assert!(!is_correct(&noul_decision(0.8), &GoldAnswer::Noul(false)));
    }

    #[test]
    fn test_latency_mean_and_unknown_ids() {
        let cases = vec![
            EvalCase {
                id: "c1".into(),
                gold: GoldAnswer::Noul(true),
            },
            EvalCase {
                id: "c2".into(),
                gold: GoldAnswer::Noul(false),
            },
        ];
        let preds = vec![
            EvalPrediction {
                case_id: "c1".into(),
                decision: noul_decision(0.9),
                latency_ms: 100,
            },
            EvalPrediction {
                case_id: "c2".into(),
                decision: noul_decision(0.1),
                latency_ms: 300,
            },
            // Unknown id → ignored; does not affect n or the mean.
            EvalPrediction {
                case_id: "ghost".into(),
                decision: noul_decision(0.9),
                latency_ms: 9999,
            },
        ];
        let r = evaluate(&cases, &preds);
        assert_eq!(r.n, 2);
        assert!((r.accuracy - 1.0).abs() < 1e-12);
        assert!((r.mean_latency_ms - 200.0).abs() < 1e-12);
    }

    #[test]
    fn test_prediction_confidence_confident_false() {
        // p=0.1 predicts false; confidence in that prediction is 0.9, not 0.1.
        assert!((prediction_confidence(&noul_decision(0.1)) - 0.9).abs() < 1e-12);
        assert!((prediction_confidence(&noul_decision(0.9)) - 0.9).abs() < 1e-12);
    }

    #[test]
    fn test_evaluate_uses_prediction_confidence() {
        // Two confident-correct Nouls (0.9 true, 0.1 false): both enter ECE
        // with confidence 0.9 against accuracy 1.0 → ece = |1.0-0.9| = 0.1.
        // With raw p as confidence the second item would enter as (0.1, ✓)
        // and report ece = 0.5 instead.
        let cases = vec![
            EvalCase { id: "c1".into(), gold: GoldAnswer::Noul(true) },
            EvalCase { id: "c2".into(), gold: GoldAnswer::Noul(false) },
        ];
        let preds = vec![
            EvalPrediction { case_id: "c1".into(), decision: noul_decision(0.9), latency_ms: 10 },
            EvalPrediction { case_id: "c2".into(), decision: noul_decision(0.1), latency_ms: 10 },
        ];
        let r = evaluate(&cases, &preds);
        assert!((r.accuracy - 1.0).abs() < 1e-12);
        assert!((r.ece - 0.1).abs() < 1e-12, "ece={}", r.ece);
    }

    #[test]
    fn test_golden_pack_jsonl_roundtrip() {
        let cases = vec![
            EvalCase { id: "c1".into(), gold: GoldAnswer::Noul(true) },
            EvalCase { id: "c2".into(), gold: GoldAnswer::Choice("b".into()) },
        ];
        let s = cases_to_jsonl(&cases).unwrap();
        assert_eq!(s.lines().count(), 2);
        let back = cases_from_jsonl(&format!("\n{}\n", s)).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].id, "c1");
        assert!(matches!(back[1].gold, GoldAnswer::Choice(ref o) if o == "b"));

        let preds = vec![EvalPrediction {
            case_id: "c1".into(),
            decision: noul_decision(0.9),
            latency_ms: 12,
        }];
        let ps = preds_to_jsonl(&preds).unwrap();
        let pback = preds_from_jsonl(&ps).unwrap();
        assert_eq!(pback.len(), 1);
        assert_eq!(pback[0].latency_ms, 12);
    }

    #[test]
    fn test_report_to_json() {
        let r = evaluate(
            &[EvalCase { id: "c1".into(), gold: GoldAnswer::Noul(true) }],
            &[EvalPrediction { case_id: "c1".into(), decision: noul_decision(0.9), latency_ms: 10 }],
        );
        let j = report_to_json(&r).unwrap();
        assert!(j.contains("\"accuracy\""), "got {}", j);
        assert!(j.contains("\"ece\""), "got {}", j);
    }
}
