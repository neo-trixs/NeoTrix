//! Temperature scaling for logits
//!
//! Supports two config formats:
//! 1. Laya format: `"1,3": 0.7` (qtype_idx, num_options → temp)
//! 2. JEV format: `"choice:3-5": 1.251` (qtype_name:option_range → temp)

use crate::types::QuestionType;
use crate::error::Result;
use std::collections::HashMap;
use std::path::Path;

/// Temperature scaler for logits
pub struct TemperatureScaler {
    default: [f64; 3],  // [noul, choice, score]
    by_options: HashMap<(usize, usize), f64>,
}

/// Parsed rl_agent_config.json structure
#[derive(serde::Deserialize)]
struct LayaAgentConfig {
    /// Default temperatures [noul, choice, score]
    temperature: Option<Vec<f64>>,
    /// Per-option-count temperature overrides (Laya format: "qtype_idx,num_options")
    temperature_by_options: Option<HashMap<String, f64>>,
}

/// JEV-style config with bucketed temperatures
#[derive(serde::Deserialize)]
pub struct JevConfig {
    /// Default temperatures [noul, choice, score]
    temperature: Option<Vec<f64>>,
    /// Per-option-count temperature overrides (JEV format: "qtype:range")
    temperature_by_options: Option<HashMap<String, f64>>,
}

/// Parse JEV format bucket spec like "choice:3-5" or "noul:2"
/// Returns (qtype_idx, min_options, max_options)
fn parse_jev_bucket(spec: &str) -> Option<(usize, usize, usize)> {
    let parts: Vec<&str> = spec.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let qtype_idx = match parts[0] {
        "noul" => 0,
        "choice" => 1,
        "score" => 2,
        _ => return None,
    };
    
    let range_str = parts[1];
    if let Some(dash_pos) = range_str.find('-') {
        // Range format: "3-5"
        let min: usize = range_str[..dash_pos].parse().ok()?;
        let max: usize = range_str[dash_pos + 1..].parse().ok()?;
        Some((qtype_idx, min, max))
    } else {
        // Exact format: "2"
        let n: usize = range_str.parse().ok()?;
        Some((qtype_idx, n, n))
    }
}

impl TemperatureScaler {
    /// Create with default temperatures
    pub fn new(default: [f64; 3], by_options: HashMap<(usize, usize), f64>) -> Self {
        Self {
            default,
            by_options,
        }
    }
    
    /// Load from rl_agent_config.json (supports both Laya and JEV formats)
    pub fn from_config_path(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: LayaAgentConfig = serde_json::from_str(&content)?;
        Self::from_config(config)
    }

    /// Load from rl_agent_config.json bytes
    pub fn from_config_bytes(data: &[u8]) -> Result<Self> {
        let config: LayaAgentConfig = serde_json::from_slice(data)?;
        Self::from_config(config)
    }

    /// Load from JEV-style config JSON with bucketed temperatures
    pub fn from_jev_config(config: JevConfig) -> Result<Self> {
        let default = config.temperature
            .and_then(|v| {
                if v.len() >= 3 {
                    Some([v[0], v[1], v[2]])
                } else {
                    None
                }
            })
            .unwrap_or([1.0, 1.0, 1.0]);

        for (i, &t) in default.iter().enumerate() {
            if t <= 0.0 {
                return Err(crate::error::Error::WeightLoadError(format!(
                    "Temperature must be positive, got {} for index {}", t, i
                )));
            }
        }

        let mut by_options = HashMap::new();
        if let Some(overrides) = config.temperature_by_options {
            for (key, temp) in overrides {
                if temp <= 0.0 {
                    return Err(crate::error::Error::WeightLoadError(format!(
                        "Temperature must be positive, got {} for key '{}'", temp, key
                    )));
                }
                
                // Try JEV format first: "choice:3-5"
                if let Some((qtype_idx, min, max)) = parse_jev_bucket(&key) {
                    for n in min..=max {
                        by_options.insert((qtype_idx, n), temp);
                    }
                }
                // Fall back to Laya format: "1,3"
                else if let Some((qtype_idx, num_opts)) = parse_laya_bucket(&key) {
                    by_options.insert((qtype_idx, num_opts), temp);
                }
            }
        }

        Ok(Self { default, by_options })
    }

    fn from_config(config: LayaAgentConfig) -> Result<Self> {
        let default = config.temperature
            .and_then(|v| {
                if v.len() >= 3 {
                    Some([v[0], v[1], v[2]])
                } else {
                    None
                }
            })
            .unwrap_or([1.0, 1.0, 1.0]);

        for (i, &t) in default.iter().enumerate() {
            if t <= 0.0 {
                return Err(crate::error::Error::WeightLoadError(format!(
                    "Temperature must be positive, got {} for index {}", t, i
                )));
            }
        }

        let mut by_options = HashMap::new();
        if let Some(overrides) = config.temperature_by_options {
            for (key, temp) in overrides {
                if temp <= 0.0 {
                    return Err(crate::error::Error::WeightLoadError(format!(
                        "Temperature must be positive, got {} for key '{}'", temp, key
                    )));
                }
                
                // Try JEV format first: "choice:3-5"
                if let Some((qtype_idx, min, max)) = parse_jev_bucket(&key) {
                    for n in min..=max {
                        by_options.insert((qtype_idx, n), temp);
                    }
                }
                // Fall back to Laya format: "1,3"
                else if let Some((qtype_idx, num_opts)) = parse_laya_bucket(&key) {
                    by_options.insert((qtype_idx, num_opts), temp);
                }
            }
        }

        Ok(Self { default, by_options })
    }

    /// Scale logits in place
    pub fn scale(&self, logits: &mut [f64], qtype: &QuestionType, num_options: usize) {
        let temp = self.get_temperature(qtype, num_options);
        for logit in logits.iter_mut() {
            *logit /= temp;
        }
    }
    
    /// Get temperature for question type
    pub fn get_temperature(&self, qtype: &QuestionType, num_options: usize) -> f64 {
        let qtype_idx = match qtype {
            QuestionType::Noul { .. } => 0,
            QuestionType::Choice { .. } => 1,
            QuestionType::Score { .. } => 2,
        };

        let bucket = (qtype_idx, num_options);
        self.by_options.get(&bucket)
            .copied()
            .unwrap_or(self.default[qtype_idx])
    }

    /// Single-case negative log-likelihood under temperature scaling:
    /// `-ln softmax(logits / temp)[gold]`.
    ///
    /// Numerically stable: the max scaled logit is subtracted before `exp`.
    /// Returns `f64::INFINITY` instead of panicking on unusable input
    /// (empty `logits`, `gold` out of range, non-positive/non-finite `temp`,
    /// or NaN-poisoned arithmetic).
    pub fn nll(logits: &[f64], gold: usize, temp: f64) -> f64 {
        if logits.is_empty() || gold >= logits.len() {
            return f64::INFINITY;
        }
        if !(temp > 0.0) || !temp.is_finite() {
            return f64::INFINITY;
        }
        let mut max = f64::NEG_INFINITY;
        for &z in logits {
            let s = z / temp;
            if s > max {
                max = s;
            }
        }
        if !max.is_finite() {
            return f64::INFINITY;
        }
        let mut sum = 0.0;
        for &z in logits {
            sum += ((z / temp) - max).exp();
        }
        if !(sum > 0.0) || !sum.is_finite() {
            return f64::INFINITY;
        }
        let v = (max - logits[gold] / temp) + sum.ln();
        if v.is_finite() {
            v
        } else {
            f64::INFINITY
        }
    }

    /// Mean NLL over a case set, skipping cases whose per-case NLL
    /// ([`TemperatureScaler::nll`]) is non-finite.
    /// Returns `f64::INFINITY` when no case is usable.
    fn mean_nll(cases: &[(Vec<f64>, usize)], temp: f64) -> f64 {
        let mut sum = 0.0;
        let mut n = 0usize;
        for (logits, gold) in cases {
            let v = Self::nll(logits, *gold, temp);
            if v.is_finite() {
                sum += v;
                n += 1;
            }
        }
        if n == 0 {
            f64::INFINITY
        } else {
            sum / n as f64
        }
    }

    /// Coarse-to-fine grid search for the NLL-optimal temperature.
    ///
    /// Coarse pass: 40 log-spaced points `T = 0.05 * 200^(i/39)` covering
    /// `[0.05, 10.0]`. Then two refinement rounds, each 20 log-spaced points
    /// over ±30% of the current best (clamped to `[0.05, 10.0]`). The best
    /// value seen is carried forward so refinement never regresses.
    /// Returns `None` when no case is usable. Fully deterministic.
    fn search_best_temperature(cases: &[(Vec<f64>, usize)]) -> Option<f64> {
        let usable = cases
            .iter()
            .any(|(logits, gold)| !logits.is_empty() && *gold < logits.len());
        if !usable {
            return None;
        }
        let mut best_t = 0.05;
        let mut best_v = f64::INFINITY;
        for i in 0..40 {
            let t = 0.05 * 200.0f64.powf(i as f64 / 39.0);
            let v = Self::mean_nll(cases, t);
            if v < best_v {
                best_v = v;
                best_t = t;
            }
        }
        for _ in 0..2 {
            let lo = (best_t * 0.7).clamp(0.05, 10.0);
            let hi = (best_t * 1.3).clamp(0.05, 10.0);
            for j in 0..20 {
                let t = if hi > lo {
                    lo * (hi / lo).powf(j as f64 / 19.0)
                } else {
                    best_t
                };
                let v = Self::mean_nll(cases, t);
                if v < best_v {
                    best_v = v;
                    best_t = t;
                }
            }
        }
        Some(best_t)
    }

    /// Fit one bucket's temperature by minimizing mean NLL over `cases`.
    ///
    /// `bucket` uses the same spec language as config keys: JEV form
    /// (`"choice:3-5"`, `"noul:2"`) or Laya form (`"1,3"`). A JEV range
    /// writes the fitted `T` into every expanded `(qtype_idx, num_options)`
    /// entry of the bucket map, so [`TemperatureScaler::scale`] picks it up immediately.
    ///
    /// Leaves the scaler unchanged (no panic) when `cases` is empty, when no
    /// case is usable (all empty logits / out-of-range gold), or when
    /// `bucket` is unparseable.
    pub fn fit_nll(&mut self, bucket: &str, cases: &[(Vec<f64>, usize)]) {
        if cases.is_empty() {
            return;
        }
        let keys = expand_bucket(bucket);
        if keys.is_empty() {
            return;
        }
        let Some(best) = Self::search_best_temperature(cases) else {
            return;
        };
        for k in keys {
            self.by_options.insert(k, best);
        }
    }

    /// Fit every bucket independently: group `cases` by bucket name, then
    /// call [`TemperatureScaler::fit_nll`] on each group. Empty input, groups
    /// case, and unparseable bucket names are skipped without panicking.
    pub fn fit_all_nll(&mut self, cases: &[(String, Vec<f64>, usize)]) {
        if cases.is_empty() {
            return;
        }
        let mut groups: HashMap<&str, Vec<(Vec<f64>, usize)>> = HashMap::new();
        for (bucket, logits, gold) in cases {
            groups
                .entry(bucket.as_str())
                .or_default()
                .push((logits.clone(), *gold));
        }
        for (bucket, group) in groups {
            self.fit_nll(bucket, &group);
        }
    }
}

/// Parse Laya format bucket spec like "1,3"
fn parse_laya_bucket(key: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = key.split(',').collect();
    if parts.len() == 2 {
        let qtype_idx = parts[0].parse::<usize>().ok()?;
        let num_opts = parts[1].parse::<usize>().ok()?;
        Some((qtype_idx, num_opts))
    } else {
        None
    }
}

/// Expand a bucket spec into concrete `(qtype_idx, num_options)` keys.
/// Accepts JEV (`"choice:3-5"`, `"noul:2"`) and Laya (`"1,3"`) forms.
/// Returns an empty vec for unparseable specs.
fn expand_bucket(spec: &str) -> Vec<(usize, usize)> {
    if let Some((qtype_idx, min, max)) = parse_jev_bucket(spec) {
        return (min..=max).map(|n| (qtype_idx, n)).collect();
    }
    if let Some(key) = parse_laya_bucket(spec) {
        return vec![key];
    }
    Vec::new()
}

impl Default for TemperatureScaler {
    fn default() -> Self {
        Self {
            default: [1.0, 1.0, 1.0],
            by_options: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config() {
        let config_json = r#"{
            "temperature": [0.8, 1.0, 0.9],
            "temperature_by_options": {
                "1,3": 0.7,
                "2,4": 0.6
            }
        }"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let scaler = TemperatureScaler::from_config(config).unwrap();

        // Default temps
        let mut logits = vec![1.0, 2.0, 3.0];
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        scaler.scale(&mut logits, &noul, 2);
        assert!((logits[0] - 1.25).abs() < 0.001); // 1.0 / 0.8

        // Override: (Choice, 3 options) -> 0.7
        let mut logits = vec![1.0, 2.0, 3.0];
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        scaler.scale(&mut logits, &choice, 3);
        assert!((logits[0] - 1.0 / 0.7).abs() < 0.001);
    }

    #[test]
    fn test_jev_bucket_parsing() {
        // "choice:3-5" should create buckets for (Choice, 3), (Choice, 4), (Choice, 5)
        let config = JevConfig {
            temperature: Some(vec![1.0, 1.0, 1.0]),
            temperature_by_options: Some(HashMap::from([
                ("choice:3-5".to_string(), 1.251),
                ("noul:2".to_string(), 1.637),
            ])),
        };
        let scaler = TemperatureScaler::from_jev_config(config).unwrap();

        // Choice with 4 options should use 1.251
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let temp = scaler.get_temperature(&choice, 4);
        assert!((temp - 1.251).abs() < 0.001);

        // Choice with 2 options should use default
        let temp = scaler.get_temperature(&choice, 2);
        assert!((temp - 1.0).abs() < 0.001);

        // Noul with 2 options should use 1.637
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        let temp = scaler.get_temperature(&noul, 2);
        assert!((temp - 1.637).abs() < 0.001);
    }

    #[test]
    fn test_default() {
        let scaler = TemperatureScaler::default();
        let mut logits = vec![1.0, 2.0];
        let noul = QuestionType::Noul { instructions: "test".into(), criteria: None };
        scaler.scale(&mut logits, &noul, 2);
        // Default temp is 1.0, so logits unchanged
        assert!((logits[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_invalid_temperature_rejected() {
        let config_json = r#"{"temperature": [-0.5, 1.0, 0.9]}"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let result = TemperatureScaler::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_temperature_rejected() {
        let config_json = r#"{"temperature": [0.0, 1.0, 0.9]}"#;
        let config: LayaAgentConfig = serde_json::from_str(config_json).unwrap();
        let result = TemperatureScaler::from_config(config);
        assert!(result.is_err());
    }

    // ── Post-hoc NLL temperature fitting ─────────────────────────

    #[test]
    fn test_nll_sanity() {
        // Hand-check: exp(2)=7.389, exp(1)=2.718, exp(0.1)=1.105,
        // sum≈11.2125, p(gold)=0.659, NLL≈0.4170.
        let logits = vec![2.0, 1.0, 0.1];
        let got = TemperatureScaler::nll(&logits, 0, 1.0);
        // Independent direct computation (no max-subtraction trick).
        let e0 = 2.0f64.exp();
        let e1 = 1.0f64.exp();
        let e2 = 0.1f64.exp();
        let direct = -(e0 / (e0 + e1 + e2)).ln();
        assert!(
            (got - direct).abs() < 1e-12,
            "stable nll {} vs direct {}",
            got,
            direct
        );
        // Precomputed constant (float64: 0.4170300162778335).
        assert!(
            (got - 0.41703).abs() < 1e-6,
            "nll {} vs precomputed 0.41703",
            got
        );
    }

    #[test]
    fn test_fit_recovers_known_temperature() {
        // Binary calibration design: with n0 gold-0 and n1 gold-1 copies of
        // gap-`a` logits, mean NLL is minimized exactly where
        // sigmoid(a/T) = n0/(n0+n1), i.e. T = a / ln(n0/n1).
        // Group A: a = 2*ln3, 3:1 split -> T* = 2.0.
        // Group B: b = 2*ln2, gold 1 twice + gold 0 once -> T* = 2.0.
        // Both groups are minimized at T=2 and each is strictly convex in
        // 1/T, so the joint optimum is exactly T* = 2.0.
        let a = 2.0 * 3.0f64.ln();
        let b = 2.0 * 2.0f64.ln();
        let cases = vec![
            (vec![a, 0.0], 0),
            (vec![a, 0.0], 0),
            (vec![a, 0.0], 0),
            (vec![a, 0.0], 1),
            (vec![0.0, b], 1),
            (vec![0.0, b], 1),
            (vec![0.0, b], 0),
        ];
        let mut scaler = TemperatureScaler::default();
        scaler.fit_nll("choice:2", &cases);
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let fitted = scaler.get_temperature(&choice, 2);
        assert!(
            (fitted - 2.0).abs() < 0.5,
            "fitted T {} should recover T*=2.0 within 0.5",
            fitted
        );
    }

    #[test]
    fn test_fit_lowers_nll_for_overconfident_wrong() {
        // Model confidently predicts the wrong class: softening (T > 1)
        // must reduce mean NLL below the T=1.0 baseline.
        let cases = vec![
            (vec![5.0, 1.0, 0.0], 1),
            (vec![4.0, 0.5, 0.2], 2),
            (vec![1.0, 4.5, 0.3], 0),
        ];
        let mean_at = |t: f64| {
            cases
                .iter()
                .map(|(l, g)| TemperatureScaler::nll(l, *g, t))
                .sum::<f64>()
                / cases.len() as f64
        };
        let baseline = mean_at(1.0);
        let mut scaler = TemperatureScaler::default();
        scaler.fit_nll("choice:3", &cases);
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let fitted = scaler.get_temperature(&choice, 3);
        assert!(fitted > 1.0, "overconfident-wrong fit T {} should exceed 1.0", fitted);
        assert!(
            mean_at(fitted) <= baseline + 1e-9,
            "fitted NLL {} should not exceed baseline {}",
            mean_at(fitted),
            baseline
        );
    }

    #[test]
    fn test_fit_empty_input_leaves_bucket_unchanged() {
        let mut scaler = TemperatureScaler::default();
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let before = scaler.get_temperature(&choice, 3);
        scaler.fit_nll("choice:3", &[]);
        assert_eq!(scaler.get_temperature(&choice, 3), before);
        scaler.fit_all_nll(&[]);
        assert_eq!(scaler.get_temperature(&choice, 3), before);
        // Unparseable bucket spec: no panic, nothing stored.
        scaler.fit_nll("not-a-bucket", &[(vec![1.0, 2.0], 0)]);
        assert_eq!(scaler.get_temperature(&choice, 3), before);
    }

    #[test]
    fn test_nll_out_of_range_gold() {
        // Degenerate inputs yield +inf instead of panicking.
        assert_eq!(TemperatureScaler::nll(&[1.0, 2.0], 5, 1.0), f64::INFINITY);
        assert_eq!(TemperatureScaler::nll(&[], 0, 1.0), f64::INFINITY);
        assert_eq!(TemperatureScaler::nll(&[1.0, 2.0], 0, 0.0), f64::INFINITY);
        assert_eq!(TemperatureScaler::nll(&[1.0, 2.0], 0, -1.0), f64::INFINITY);
        // Fitting over only-invalid cases leaves the bucket unchanged.
        let mut scaler = TemperatureScaler::default();
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let before = scaler.get_temperature(&choice, 2);
        scaler.fit_nll("choice:2", &[(vec![1.0, 2.0], 7)]);
        assert_eq!(scaler.get_temperature(&choice, 2), before);
        // Mixed valid + invalid: invalid skipped, valid case still fits
        // (gold 1 is not top logit, so optimum softens past 1.0).
        scaler.fit_nll("choice:2", &[(vec![1.0, 2.0], 9), (vec![5.0, 1.0], 1)]);
        assert!(scaler.get_temperature(&choice, 2) > 1.0);
    }

    #[test]
    fn test_fit_all_nll_groups_by_bucket() {
        // "choice:2": sharp-but-mostly-right (gap 1, 5:1 split ->
        // T* = 1/ln5 ≈ 0.62 < 1). "choice:3": overconfident-wrong
        // (T* at the 10.0 cap > 1). Each bucket must fit independently.
        let mut cases: Vec<(String, Vec<f64>, usize)> = Vec::new();
        for _ in 0..5 {
            cases.push(("choice:2".to_string(), vec![2.0, 1.0], 0));
        }
        cases.push(("choice:2".to_string(), vec![2.0, 1.0], 1));
        cases.push(("choice:3".to_string(), vec![5.0, 1.0, 0.0], 1));
        cases.push(("choice:3".to_string(), vec![4.0, 0.5, 0.2], 2));
        cases.push(("choice:3".to_string(), vec![1.0, 4.5, 0.3], 0));
        let mut scaler = TemperatureScaler::default();
        scaler.fit_all_nll(&cases);
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let t2 = scaler.get_temperature(&choice, 2);
        let t3 = scaler.get_temperature(&choice, 3);
        assert!(t2 < 1.0, "choice:2 fitted T {} should sharpen (< 1.0)", t2);
        assert!(t3 > 1.0, "choice:3 fitted T {} should soften (> 1.0)", t3);
    }

    #[test]
    fn test_fit_jev_range_and_laya_bucket_forms() {
        let cases = vec![
            (vec![5.0, 1.0, 0.0], 1),
            (vec![4.0, 0.5, 0.2], 2),
        ];
        let mut scaler = TemperatureScaler::default();
        scaler.fit_nll("choice:3-5", &cases);
        let choice = QuestionType::Choice { instructions: "test".into(), criteria: HashMap::new() };
        let t3 = scaler.get_temperature(&choice, 3);
        assert!(t3 > 1.0, "range fit T {} should soften (> 1.0)", t3);
        // JEV range writes every expanded key.
        assert_eq!(scaler.get_temperature(&choice, 4), t3);
        assert_eq!(scaler.get_temperature(&choice, 5), t3);
        // Outside the range stays default.
        assert_eq!(scaler.get_temperature(&choice, 2), 1.0);
        // Laya "qtype_idx,num_options" form addresses the same key.
        let mut scaler2 = TemperatureScaler::default();
        scaler2.fit_nll("1,3", &cases);
        assert_eq!(scaler2.get_temperature(&choice, 3), t3);
    }
}
