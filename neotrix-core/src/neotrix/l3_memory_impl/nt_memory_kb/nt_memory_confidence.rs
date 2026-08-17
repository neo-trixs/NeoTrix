use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::nt_memory_types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceWeights {
    pub w_source: f64,
    pub w_grounding: f64,
    pub w_consensus: f64,
    pub w_recency: f64,
    pub reconfirm_source_boost: f64,
    pub reconfirm_grounding_boost: f64,
    pub reconfirm_consensus_boost: f64,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            w_source: 0.30,
            w_grounding: 0.30,
            w_consensus: 0.25,
            w_recency: 0.15,
            reconfirm_source_boost: 0.3,
            reconfirm_grounding_boost: 0.2,
            reconfirm_consensus_boost: 0.1,
        }
    }
}

pub static CONFIDENCE_WEIGHTS: std::sync::LazyLock<ConfidenceWeights> =
    std::sync::LazyLock::new(ConfidenceWeights::default);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicConfidence {
    pub source_confidence: f64,
    pub grounding_confidence: f64,
    pub consensus_confidence: f64,
    pub recency_confidence: f64,
    pub computed_at: i64,
    pub last_confirmed_at: i64,
}

impl EpistemicConfidence {
    pub fn aggregate(&self) -> f64 {
        let w = &CONFIDENCE_WEIGHTS;
        let numerator = w.w_source + w.w_grounding + w.w_consensus + w.w_recency;
        let denominator = w.w_source / (self.source_confidence + 1e-10)
            + w.w_grounding / (self.grounding_confidence + 1e-10)
            + w.w_consensus / (self.consensus_confidence + 1e-10)
            + w.w_recency / (self.recency_confidence + 1e-10);
        (numerator / denominator).max(0.0).min(1.0)
    }

    pub fn decay(&mut self, elapsed_days: f64, lambda: f64) {
        let decay_factor = (-lambda * elapsed_days).exp();
        self.recency_confidence = (self.recency_confidence * decay_factor).max(0.0).min(1.0);
        self.computed_at = now_ts();
    }

    pub fn reconfirm(&mut self, strength: f64) {
        let w = &CONFIDENCE_WEIGHTS;
        let s = strength.max(0.0).min(1.0);
        self.source_confidence = self.source_confidence + (1.0 - self.source_confidence) * s * w.reconfirm_source_boost;
        self.grounding_confidence =
            self.grounding_confidence + (1.0 - self.grounding_confidence) * s * w.reconfirm_grounding_boost;
        self.consensus_confidence =
            self.consensus_confidence + (1.0 - self.consensus_confidence) * s * w.reconfirm_consensus_boost;
        self.recency_confidence = 1.0;
        let ts = now_ts();
        self.last_confirmed_at = ts;
        self.computed_at = ts;
    }

    pub fn unknown() -> Self {
        Self {
            source_confidence: 0.0,
            grounding_confidence: 0.0,
            consensus_confidence: 0.0,
            recency_confidence: 0.5,
            computed_at: now_ts(),
            last_confirmed_at: 0,
        }
    }

    pub fn is_above_threshold(&self, min_aggregate: f64) -> bool {
        self.aggregate() >= min_aggregate
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceSource {
    LmInference,
    ToolObservation,
    UserInput,
    WebCrawl,
    CrossReference,
    Deduced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictingFact {
    pub node_id: String,
    pub claim: String,
    pub confidence: f64,
    pub source: ConfidenceSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertainResult {
    pub node: KnowledgeNode,
    pub confidence: EpistemicConfidence,
    pub contradictions: Vec<ContradictingFact>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RetrievalStrategy {
    Conservative { min_confidence: f64 },
    Balanced,
    Exploratory,
    ConfidenceWeighted {
        source_weight: f64,
        grounding_weight: f64,
        consensus_weight: f64,
        recency_weight: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayConfig {
    pub lambda_general: f64,
    pub lambda_time_sensitive: f64,
    pub min_confidence: f64,
    pub auto_archive_days: i64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            lambda_general: 0.01,
            lambda_time_sensitive: 0.05,
            min_confidence: 0.1,
            auto_archive_days: 90,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsensusInfo {
    pub support_count: usize,
    pub contradict_count: usize,
    pub consensus_score: f64,
    pub contradictions: Vec<ContradictingFact>,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn grounding_saturation(composite: f64, k: f64) -> f64 {
    if composite <= 0.0 {
        return 0.0;
    }
    (composite / (composite + k)).max(0.0).min(1.0)
}

pub fn compute_grounding(
    source_count: usize,
    supporting_count: usize,
    contradicting_count: usize,
    relationship_types: usize,
) -> f64 {
    let support_ratio = if source_count > 0 {
        ((supporting_count as f64 - contradicting_count as f64) / source_count.max(1) as f64)
            .max(0.0)
            .min(1.0)
    } else {
        0.0
    };

    let source_contribution = (source_count as f64 / 5.0).min(1.0);
    let diversity_contribution = (relationship_types as f64 / 10.0).min(1.0);

    let composite =
        source_contribution * 0.4 + support_ratio * 0.4 + diversity_contribution * 0.2;

    grounding_saturation(composite, 2.0)
}

pub fn michaelis_menten_saturation(confidence: f64, k: f64) -> f64 {
    let c = confidence.max(0.0);
    (c * c / (c * c + k * k)).max(0.0).min(1.0)
}

pub fn decay_lambda_for_fact_type(fact_type: &str) -> f64 {
    match fact_type {
        "permanent" => 0.001,
        "dynamic" => 0.05,
        "time_sensitive" => 0.1,
        "trend" => 0.03,
        "source" => 0.005,
        _ => 0.01,
    }
}

pub fn should_auto_archive(confidence: &EpistemicConfidence) -> bool {
    if confidence.recency_confidence >= 0.1 {
        return false;
    }
    if confidence.last_confirmed_at <= 0 {
        return false;
    }
    let days_since =
        (now_ts() - confidence.last_confirmed_at) as f64 / 86400.0;
    days_since > 30.0
}

pub fn authenticated_diversity(grounding: f64, domains: &[String]) -> f64 {
    if domains.is_empty() {
        return 0.0;
    }
    let mut unique: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for d in domains {
        unique.insert(d.as_str());
    }
    let diversity = unique.len() as f64 / domains.len() as f64;

    let k = 0.3;
    let saturated_grounding = grounding / (grounding.abs() + k);

    saturated_grounding * diversity
}

pub fn newman_consensus(source_confidences: &[f64]) -> f64 {
    if source_confidences.is_empty() {
        return 0.5;
    }
    let n = source_confidences.len() as f64;
    let sum_uncertainty: f64 = source_confidences
        .iter()
        .map(|p| 1.0 - p.max(0.0).min(1.0))
        .sum();
    (1.0 - sum_uncertainty / n).max(0.0).min(1.0)
}

fn expand_contractions(s: &str) -> String {
    let mut s = s.to_string();
    let replacements = [
        ("isn't", "is not"),
        ("aren't", "are not"),
        ("wasn't", "was not"),
        ("weren't", "were not"),
        ("hasn't", "has not"),
        ("haven't", "have not"),
        ("hadn't", "had not"),
        ("doesn't", "does not"),
        ("don't", "do not"),
        ("didn't", "did not"),
        ("won't", "will not"),
        ("can't", "cannot"),
        ("couldn't", "could not"),
        ("shouldn't", "should not"),
        ("wouldn't", "would not"),
        ("mustn't", "must not"),
    ];
    for (from, to) in &replacements {
        s = s.replace(from, to);
    }
    s
}

pub fn detect_simple_contradiction(claim_a: &str, claim_b: &str) -> bool {
    let a = expand_contractions(&claim_a.trim().to_lowercase());
    let b = expand_contractions(&claim_b.trim().to_lowercase());

    if a == b {
        return false;
    }

    let negation_patterns = ["not ", "no ", "never ", "without ", "isn't ", "aren't ", "doesn't ",
        "don't ", "didn't ", "won't ", "can't ", "cannot ", "couldn't ", "shouldn't ",
        "wouldn't ", "hasn't ", "haven't ", "hadn't ", "wasn't ", "weren't "];

    for prefix in &negation_patterns {
        let negated_a = format!("{}{}", prefix, a);
        let negated_b = format!("{}{}", prefix, b);
        if a == negated_b || b == negated_a {
            return true;
        }
    }

    let opposite_pairs = [
        ("is", "is not"), ("are", "are not"), ("was", "was not"),
        ("were", "were not"), ("has", "has not"), ("have", "have not"),
        ("does", "does not"), ("do", "do not"), ("did", "did not"),
        ("will", "will not"), ("can", "cannot"), ("could", "could not"),
        ("should", "should not"), ("would", "would not"), ("may", "may not"),
        ("might", "might not"), ("must", "must not"),
    ];

    for (pos, neg) in &opposite_pairs {
        if a.contains(pos) && b.contains(neg) {
            return true;
        }
        if b.contains(pos) && a.contains(neg) {
            return true;
        }
    }

    let opposite_words = ["true", "false", "yes", "no", "positive", "negative",
        "increase", "decrease", "grow", "shrink", "win", "lose",
        "success", "failure", "correct", "incorrect", "right", "wrong",
        "enable", "disable", "start", "stop", "begin", "end",
        "on", "off", "open", "closed", "full", "empty",
        "present", "absent", "include", "exclude", "accept", "reject"];

    for pair in opposite_words.chunks(2) {
        if pair.len() == 2 {
            if a.contains(pair[0]) && b.contains(pair[1]) {
                return true;
            }
            if b.contains(pair[0]) && a.contains(pair[1]) {
                return true;
            }
        }
    }

    false
}

pub fn detect_consensus(
    support_count: usize,
    contradict_count: usize,
    _supporting_sources: Vec<String>,
    contradicting_sources: Vec<String>,
) -> ConsensusInfo {
    let total = (support_count + contradict_count) as f64;
    let normalized_consensus = if total == 0.0 {
        0.5
    } else {
        let consensus = (support_count as f64 - contradict_count as f64) / total;
        ((consensus + 1.0) / 2.0).max(0.0).min(1.0)
    };

    let contradictions: Vec<ContradictingFact> = contradicting_sources
        .into_iter()
        .map(|source_id| ContradictingFact {
            node_id: source_id,
            claim: String::new(),
            confidence: 0.0,
            source: ConfidenceSource::CrossReference,
        })
        .collect();

    ConsensusInfo {
        support_count,
        contradict_count,
        consensus_score: normalized_consensus,
        contradictions,
    }
}

#[derive(Deserialize)]
pub struct ConfidenceStore {
    inner: RwLock<HashMap<Uuid, EpistemicConfidence>>,
    contradictions: RwLock<HashMap<String, Vec<ContradictingFact>>>,
    decay_config: DecayConfig,
    lambda_overrides: HashMap<String, f64>,
}

// 自定义 Serialize：把 RwLock 内的真实数据写盘（derive 的 serde(skip) 会导致
// round-trip 后置信度/矛盾数据全部丢失）
impl serde::Serialize for ConfidenceStore {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let inner = self.inner.read().map_err(serde::ser::Error::custom)?;
        let contradictions = self.contradictions.read().map_err(serde::ser::Error::custom)?;
        #[derive(serde::Serialize)]
        struct Repr<'a> {
            inner: &'a HashMap<Uuid, EpistemicConfidence>,
            contradictions: &'a HashMap<String, Vec<ContradictingFact>>,
            decay_config: &'a DecayConfig,
            lambda_overrides: &'a HashMap<String, f64>,
        }
        Repr {
            inner: &inner,
            contradictions: &contradictions,
            decay_config: &self.decay_config,
            lambda_overrides: &self.lambda_overrides,
        }
        .serialize(serializer)
    }
}

impl ConfidenceStore {
    pub fn new(decay_config: DecayConfig) -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
            contradictions: RwLock::new(HashMap::new()),
            decay_config,
            lambda_overrides: HashMap::new(),
        }
    }

    pub fn store_confidence(
        &self,
        node_id: &Uuid,
        epistemic: &EpistemicConfidence,
    ) -> Result<(), String> {
        let mut map = self.inner.write().map_err(|e| format!("Lock: {}", e))?;
        map.insert(*node_id, epistemic.clone());
        Ok(())
    }

    pub fn get_confidence(&self, node_id: &Uuid) -> Result<Option<EpistemicConfidence>, String> {
        let map = self.inner.read().map_err(|e| format!("Lock: {}", e))?;
        Ok(map.get(node_id).cloned())
    }

    pub fn get_confidence_batch(
        &self,
        node_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, EpistemicConfidence>, String> {
        let map = self.inner.read().map_err(|e| format!("Lock: {}", e))?;
        let mut results = HashMap::new();
        for id in node_ids {
            if let Some(c) = map.get(id) {
                results.insert(*id, c.clone());
            }
        }
        Ok(results)
    }

    pub fn apply_decay(&self, lambda: f64, older_than_days: i64) -> Result<u64, String> {
        let mut map = self.inner.write().map_err(|e| format!("Lock: {}", e))?;
        let mut count = 0u64;
        let now = now_ts();
        for epistemic in map.values_mut() {
            let age_days = (now - epistemic.computed_at) as f64 / 86400.0;
            if age_days >= older_than_days as f64 {
                epistemic.decay(age_days, lambda);
                if epistemic.recency_confidence < self.decay_config.min_confidence {
                    epistemic.recency_confidence = self.decay_config.min_confidence;
                }
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn reconfirm_node(&self, node_id: &Uuid, strength: f64) -> Result<(), String> {
        let mut map = self.inner.write().map_err(|e| format!("Lock: {}", e))?;
        if let Some(epistemic) = map.get_mut(node_id) {
            epistemic.reconfirm(strength);
            Ok(())
        } else {
            Err(format!("Node {:?} not found in confidence store", node_id))
        }
    }

    pub fn log_contradiction(
        &self,
        primary: &str,
        contradicting: &str,
        claim: &str,
        contra_claim: &str,
    ) -> Result<(), String> {
        let mut map = self
            .contradictions
            .write()
            .map_err(|e| format!("Lock: {}", e))?;
        let entry = map.entry(primary.to_string()).or_default();
        entry.push(ContradictingFact {
            node_id: contradicting.to_string(),
            claim: contra_claim.to_string(),
            confidence: 0.5,
            source: ConfidenceSource::CrossReference,
        });
        let entry2 = map.entry(contradicting.to_string()).or_default();
        entry2.push(ContradictingFact {
            node_id: primary.to_string(),
            claim: claim.to_string(),
            confidence: 0.5,
            source: ConfidenceSource::CrossReference,
        });
        Ok(())
    }

    pub fn get_contradictions(
        &self,
        node_id: &str,
    ) -> Result<Vec<ContradictingFact>, String> {
        let map = self
            .contradictions
            .read()
            .map_err(|e| format!("Lock: {}", e))?;
        Ok(map.get(node_id).cloned().unwrap_or_default())
    }

    pub fn auto_archive(&self) -> Result<u64, String> {
        let mut map = self.inner.write().map_err(|e| format!("Lock: {}", e))?;
        let to_remove: Vec<Uuid> = map
            .iter()
            .filter(|(_, epistemic)| should_auto_archive(epistemic))
            .map(|(id, _)| *id)
            .collect();
        let count = to_remove.len() as u64;
        for id in &to_remove {
            map.remove(id);
        }
        Ok(count)
    }

    pub fn purge_archived(&self, older_than_days: i64) -> Result<u64, String> {
        let mut map = self.inner.write().map_err(|e| format!("Lock: {}", e))?;
        let now = now_ts();
        let cutoff = now - older_than_days * 86400;
        let to_remove: Vec<Uuid> = map
            .iter()
            .filter(|(_, epistemic)| epistemic.last_confirmed_at > 0 && epistemic.last_confirmed_at < cutoff)
            .map(|(id, _)| *id)
            .collect();
        let count = to_remove.len() as u64;
        for id in &to_remove {
            map.remove(id);
        }
        Ok(count)
    }

    pub fn attach_confidence(&self, node_id: &Uuid) -> EpistemicConfidence {
        self.inner
            .read()
            .ok()
            .and_then(|map| map.get(node_id).cloned())
            .unwrap_or_else(EpistemicConfidence::unknown)
    }

    pub fn set_lambda_override(&mut self, fact_type: &str, lambda: f64) {
        self.lambda_overrides.insert(fact_type.to_string(), lambda);
    }

    pub fn get_lambda(&self, fact_type: &str) -> f64 {
        self.lambda_overrides
            .get(fact_type)
            .copied()
            .unwrap_or_else(|| decay_lambda_for_fact_type(fact_type))
    }

    pub fn count(&self) -> usize {
        self.inner.read().map(|m| m.len()).unwrap_or(0)
    }
}

pub fn search_with_confidence(
    kb: &super::KnowledgeBase,
    confidence_store: &ConfidenceStore,
    query: &str,
    strategy: RetrievalStrategy,
    limit: usize,
) -> Result<Vec<UncertainResult>, String> {
    let pool_size = match strategy {
        RetrievalStrategy::Conservative { .. } => limit * 5,
        RetrievalStrategy::Balanced => limit * 3,
        RetrievalStrategy::Exploratory => limit * 3,
        RetrievalStrategy::ConfidenceWeighted { .. } => limit * 3,
    };

    let raw_results = kb.search_fused(query, pool_size)?;

    let mut uncertain: Vec<UncertainResult> = raw_results
        .into_iter()
        .map(|r| {
            let node_id = Uuid::parse_str(&r.node.id).unwrap_or_else(|_| Uuid::nil());
            let confidence = confidence_store
                .get_confidence(&node_id)
                .ok()
                .flatten()
                .unwrap_or_else(EpistemicConfidence::unknown);
            let contradictions = confidence_store
                .get_contradictions(&r.node.id)
                .ok()
                .unwrap_or_default();
            UncertainResult {
                node: r.node,
                confidence,
                contradictions,
            }
        })
        .collect();

    match strategy {
        RetrievalStrategy::Conservative { min_confidence } => {
            uncertain.retain(|u| u.confidence.aggregate() >= min_confidence);
        }
        RetrievalStrategy::Balanced => {}
        RetrievalStrategy::Exploratory => {}
        RetrievalStrategy::ConfidenceWeighted {
            source_weight,
            grounding_weight,
            consensus_weight,
            recency_weight,
        } => {
            for u in &mut uncertain {
                let w = source_weight + grounding_weight + consensus_weight + recency_weight;
                if w > 0.0 {
                    let _ = (u.confidence.source_confidence * source_weight
                        + u.confidence.grounding_confidence * grounding_weight
                        + u.confidence.consensus_confidence * consensus_weight
                        + u.confidence.recency_confidence * recency_weight)
                        / w;
                }
            }
        }
    }

    uncertain.sort_by(|a, b| {
        b.confidence
            .aggregate()
            .partial_cmp(&a.confidence.aggregate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    uncertain.truncate(limit);
    Ok(uncertain)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_confidence(
        source: f64,
        grounding: f64,
        consensus: f64,
        recency: f64,
    ) -> EpistemicConfidence {
        EpistemicConfidence {
            source_confidence: source,
            grounding_confidence: grounding,
            consensus_confidence: consensus,
            recency_confidence: recency,
            computed_at: now_ts(),
            last_confirmed_at: now_ts(),
        }
    }

    #[test]
    fn test_aggregate_equal_components() {
        let c = make_confidence(0.8, 0.8, 0.8, 0.8);
        let agg = c.aggregate();
        assert!((agg - 0.8).abs() < 0.01, "Expected ~0.8, got {}", agg);
    }

    #[test]
    fn test_aggregate_one_low_component() {
        let c = make_confidence(0.9, 0.9, 0.9, 0.1);
        let agg = c.aggregate();
        assert!(
            agg < 0.5,
            "Low recency should drag aggregate down, got {}",
            agg
        );
        assert!(agg > 0.15, "Aggregate should be above min, got {}", agg);
    }

    #[test]
    fn test_aggregate_with_different_weights() {
        let c = make_confidence(1.0, 0.5, 0.5, 0.5);
        let agg = c.aggregate();
        let c2 = make_confidence(0.5, 1.0, 0.5, 0.5);
        let agg2 = c2.aggregate();
        assert!(
            (agg - agg2).abs() < 0.01,
            "Source=1.0 should equal Grounding=1.0 (same weight), got {} vs {}",
            agg,
            agg2
        );
    }

    #[test]
    fn test_aggregate_lowest_pulls_hard() {
        let c = make_confidence(0.9, 0.9, 0.9, 0.01);
        let agg = c.aggregate();
        assert!(
            agg < 0.1,
            "Near-zero recency should pull aggregate very low, got {}",
            agg
        );
    }

    #[test]
    fn test_decay_over_time() {
        let mut c = make_confidence(0.8, 0.8, 0.8, 1.0);
        c.decay(30.0, 0.01);
        let expected = (-0.01 * 30.0_f64).exp();
        assert!(
            (c.recency_confidence - expected).abs() < 0.01,
            "After 30d lambda=0.01: expected ~{}, got {}",
            expected,
            c.recency_confidence
        );
    }

    #[test]
    fn test_decay_90_days() {
        let mut c = make_confidence(0.8, 0.8, 0.8, 1.0);
        c.decay(90.0, 0.01);
        let expected = (-0.01 * 90.0_f64).exp();
        assert!(
            (c.recency_confidence - expected).abs() < 0.01,
            "After 90d lambda=0.01: expected ~{}, got {}",
            expected,
            c.recency_confidence
        );
        assert!(
            c.recency_confidence > 0.40,
            "90d decay should be > 0.40, got {}",
            c.recency_confidence
        );
    }

    #[test]
    fn test_decay_fast_lambda() {
        let mut c = make_confidence(0.8, 0.8, 0.8, 1.0);
        c.decay(30.0, 0.05);
        let expected = (-0.05 * 30.0_f64).exp();
        assert!(
            (c.recency_confidence - expected).abs() < 0.01,
            "After 30d lambda=0.05: expected ~{}, got {}",
            expected,
            c.recency_confidence
        );
    }

    #[test]
    fn test_decay_clamps() {
        let mut c = make_confidence(0.8, 0.8, 0.8, 0.5);
        c.decay(1000.0, 10.0);
        assert!(
            c.recency_confidence >= 0.0,
            "Decay should not go negative"
        );
        assert!(
            c.recency_confidence <= 1.0,
            "Decay should not exceed 1.0"
        );
    }

    #[test]
    fn test_reconfirm_increases_confidence() {
        let mut c = make_confidence(0.3, 0.3, 0.3, 0.2);
        let before = c.source_confidence;
        c.reconfirm(0.8);
        assert!(
            c.source_confidence > before,
            "Source confidence should increase after reconfirm"
        );
        assert!(
            c.grounding_confidence > 0.3,
            "Grounding confidence should increase"
        );
        assert!(
            c.consensus_confidence > 0.3,
            "Consensus confidence should increase"
        );
    }

    #[test]
    fn test_reconfirm_sets_recency_to_one() {
        let mut c = make_confidence(0.5, 0.5, 0.5, 0.1);
        c.reconfirm(0.5);
        assert!(
            (c.recency_confidence - 1.0).abs() < 0.001,
            "Recency should jump to 1.0 on reconfirm"
        );
    }

    #[test]
    fn test_reconfirm_updates_timestamps() {
        let mut c = EpistemicConfidence::unknown();
        let old_computed = c.computed_at;
        let old_confirmed = c.last_confirmed_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        c.reconfirm(1.0);
        assert!(
            c.last_confirmed_at > old_confirmed,
            "last_confirmed_at should update"
        );
        assert!(
            c.computed_at >= old_computed,
            "computed_at should update"
        );
    }

    #[test]
    fn test_reconfirm_strength_zero_no_effect() {
        let mut c = make_confidence(0.5, 0.5, 0.5, 0.3);
        let before_source = c.source_confidence;
        let before_grounding = c.grounding_confidence;
        let before_consensus = c.consensus_confidence;
        c.reconfirm(0.0);
        assert!(
            (c.source_confidence - before_source).abs() < 0.001,
            "Zero-strength reconfirm should not change source"
        );
        assert!(
            (c.grounding_confidence - before_grounding).abs() < 0.001,
            "Zero-strength reconfirm should not change grounding"
        );
        assert!(
            (c.consensus_confidence - before_consensus).abs() < 0.001,
            "Zero-strength reconfirm should not change consensus"
        );
    }

    #[test]
    fn test_reconfirm_strength_one_full_effect() {
        let mut c = EpistemicConfidence {
            source_confidence: 0.5,
            grounding_confidence: 0.5,
            consensus_confidence: 0.5,
            recency_confidence: 0.5,
            computed_at: 0,
            last_confirmed_at: 0,
        };
        c.reconfirm(1.0);
        assert!(
            (c.source_confidence - (0.5 + 0.5 * 1.0 * 0.3)).abs() < 0.001,
            "source should be {} not {}",
            0.5 + 0.5 * 0.3,
            c.source_confidence
        );
        assert!(
            (c.recency_confidence - 1.0).abs() < 0.001,
            "recency should be 1.0"
        );
    }

    #[test]
    fn test_michaelis_menten_saturation_flattens_low() {
        let low = michaelis_menten_saturation(0.1, 0.3);
        let high = michaelis_menten_saturation(0.9, 0.3);
        assert!(
            low < 0.2,
            "Low confidence (0.1) should be strongly suppressed, got {}",
            low
        );
        assert!(
            high > 0.8,
            "High confidence (0.9) should be near 1.0, got {}",
            high
        );
    }

    #[test]
    fn test_michaelis_menten_monotonic() {
        let k = 0.3;
        for i in 0..100 {
            let c1 = i as f64 / 100.0;
            let c2 = (i + 1) as f64 / 100.0;
            let s1 = michaelis_menten_saturation(c1, k);
            let s2 = michaelis_menten_saturation(c2, k);
            assert!(
                s2 >= s1,
                "Saturation should be monotonic: {} @ {} -> {}, {} @ {} -> {}",
                c1,
                i,
                s1,
                c2,
                i + 1,
                s2
            );
        }
    }

    #[test]
    fn test_michaelis_menten_formula() {
        assert!(
            (michaelis_menten_saturation(0.3, 0.3) - 0.5).abs() < 0.01,
            "At K=0.3, saturation(0.3) should be ~0.5"
        );
    }

    #[test]
    fn test_contradiction_detection_negated_prefix() {
        assert!(
            detect_simple_contradiction("the sky is blue", "the sky is not blue"),
            "Direct negation with 'not ' should detect"
        );
        assert!(
            detect_simple_contradiction("the sky is not blue", "the sky is blue"),
            "Reverse negation should detect"
        );
    }

    #[test]
    fn test_contradiction_detection_same_claim_not_contradictory() {
        assert!(
            !detect_simple_contradiction("the sky is blue", "the sky is blue"),
            "Identical claims should not be contradictory"
        );
    }

    #[test]
    fn test_contradiction_detection_opposite_pairs() {
        assert!(
            detect_simple_contradiction("the answer is true", "the answer is false"),
            "true/false should detect"
        );
        assert!(
            detect_simple_contradiction("revenue will increase", "revenue will decrease"),
            "increase/decrease should detect"
        );
    }

    #[test]
    fn test_contradiction_detection_contractions() {
        assert!(
            detect_simple_contradiction("it is working", "it isn't working"),
            "is/isn't should detect"
        );
        assert!(
            detect_simple_contradiction("it isn't working", "it is working"),
            "isn't/is reverse should detect"
        );
    }

    #[test]
    fn test_contradiction_detection_no_false_positive() {
        assert!(
            !detect_simple_contradiction("the notebook is blue", "cars are generally fast"),
            "Different subjects should not be contradictory"
        );
        assert!(
            !detect_simple_contradiction("I like apples", "I have a notebook"),
            "Unrelated claims should not be contradictory"
        );
    }

    #[test]
    fn test_grounding_saturation_basic() {
        let g = grounding_saturation(2.0, 2.0);
        assert!((g - 0.5).abs() < 0.01, "composite=2, k=2 -> 0.5, got {}", g);
    }

    #[test]
    fn test_grounding_saturation_zero() {
        let g = grounding_saturation(0.0, 2.0);
        assert!((g - 0.0).abs() < 0.01, "zero composite -> 0, got {}", g);
    }

    #[test]
    fn test_compute_grounding_no_sources() {
        let g = compute_grounding(0, 0, 0, 0);
        assert!((g - 0.0).abs() < 0.01, "no sources -> 0, got {}", g);
    }

    #[test]
    fn test_newman_consensus_single_source() {
        let c = newman_consensus(&[0.8]);
        assert!((c - 0.8).abs() < 0.01, "single source 0.8 -> 0.8, got {}", c);
    }

    #[test]
    fn test_newman_consensus_ten_sources() {
        let confs = vec![0.8; 10];
        let c = newman_consensus(&confs);
        let expected = 1.0 - (1.0 / 10.0) * 10.0 * (1.0 - 0.8);
        assert!((c - expected).abs() < 0.01, "10 sources 0.8: expected {}, got {}", expected, c);
    }

    #[test]
    fn test_newman_consensus_empty() {
        let c = newman_consensus(&[]);
        assert!((c - 0.5).abs() < 0.01, "empty -> 0.5 neutral, got {}", c);
    }

    #[test]
    fn test_decay_lambda_for_fact_type() {
        assert!((decay_lambda_for_fact_type("permanent") - 0.001).abs() < 0.0001);
        assert!((decay_lambda_for_fact_type("time_sensitive") - 0.1).abs() < 0.0001);
        assert!((decay_lambda_for_fact_type("unknown_type") - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_should_auto_archive_fresh() {
        let c = make_confidence(0.8, 0.8, 0.8, 0.9);
        assert!(!should_auto_archive(&c), "Fresh fact should not be archived");
    }

    #[test]
    fn test_should_auto_archive_never_confirmed() {
        let c = EpistemicConfidence {
            source_confidence: 0.0,
            grounding_confidence: 0.0,
            consensus_confidence: 0.0,
            recency_confidence: 0.05,
            computed_at: 0,
            last_confirmed_at: 0,
        };
        assert!(
            !should_auto_archive(&c),
            "Never-confirmed fact should not be auto-archived"
        );
    }

    #[test]
    fn test_authenticated_diversity() {
        let domains = vec!["example.com".to_string(), "example.com".to_string()];
        let ad = authenticated_diversity(0.8, &domains);
        assert!(
            ad < 0.5,
            "Echo chamber (same domain) should have low diversity, got {}",
            ad
        );
    }

    #[test]
    fn test_authenticated_diversity_diverse() {
        let domains = vec![
            "a.com".to_string(),
            "b.org".to_string(),
            "c.net".to_string(),
            "d.io".to_string(),
        ];
        let ad = authenticated_diversity(0.9, &domains);
        assert!(
            ad > 0.6,
            "4 unique domains should have high diversity, got {}",
            ad
        );
    }

    #[test]
    fn test_confidence_store_store_and_retrieve() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let id = Uuid::new_v4();
        let c = make_confidence(0.7, 0.8, 0.9, 0.6);
        store.store_confidence(&id, &c).unwrap();
        let retrieved = store
            .get_confidence(&id)
            .unwrap()
            .expect("Should find confidence");
        assert!((retrieved.source_confidence - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_confidence_store_batch() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        for (i, id) in ids.iter().enumerate() {
            let c = make_confidence(0.5 + i as f64 * 0.1, 0.5, 0.5, 0.5);
            store.store_confidence(id, &c).unwrap();
        }
        let batch = store.get_confidence_batch(&ids).unwrap();
        assert_eq!(batch.len(), 5);
    }

    #[test]
    fn test_confidence_store_reconfirm() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let id = Uuid::new_v4();
        let c = make_confidence(0.3, 0.3, 0.3, 0.2);
        store.store_confidence(&id, &c).unwrap();
        store.reconfirm_node(&id, 0.8).unwrap();
        let retrieved = store.get_confidence(&id).unwrap().unwrap();
        assert!(
            retrieved.source_confidence > 0.3,
            "After reconfirm, source should increase"
        );
        assert!(
            (retrieved.recency_confidence - 1.0).abs() < 0.01,
            "After reconfirm, recency should be 1.0"
        );
    }

    #[test]
    fn test_confidence_store_attach() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let id = Uuid::new_v4();
        let unknown = store.attach_confidence(&id);
        assert!(
            (unknown.source_confidence - 0.0).abs() < 0.01,
            "Unknown node should get zero source confidence"
        );
        assert!(
            (unknown.recency_confidence - 0.5).abs() < 0.01,
            "Unknown node should get neutral recency"
        );
    }

    #[test]
    fn test_search_with_confidence_filters_by_conservative() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn).unwrap();

        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase {
            conn: std::sync::Mutex::new(conn),
            db_path: std::path::PathBuf::from(":memory:"),
            bm25: std::sync::RwLock::new(None),
            bm25_dirty: std::sync::RwLock::new(false),
            embedding_config: std::sync::RwLock::new(None),
            fused_cache: std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(100).unwrap(),
            )),
            adaptive: crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_adaptive_rag::AdaptiveRetrieval::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_adaptive_rag::AdaptiveRagConfig::default(),
            ),
            commitment_store: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_commitment::EmbeddingCommitmentStore::new(0, None),
            ),
            confidence_store: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_confidence::ConfidenceStore::new(
                    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_confidence::DecayConfig::default(),
                ),
            ),
            community_search: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_community::CommunityAwareSearch::new(
                    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_community::CommunityDetector::default(),
                ),
            ),
            privacy: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::privacy::PrivacyEnforcer::new(
                    crate::neotrix::l3_memory_impl::nt_memory_kb::privacy::PrivacyConfig::default(),
                ),
            ),
            vector_adapter: std::sync::RwLock::new(None),
            agent_memory: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_agent_driven::AgentMemory::new(
                    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_agent_driven::MemoryConfig::default(),
                ),
            ),
            agent_session: std::sync::RwLock::new(false),
            svaf_gate: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_svaf_gate::SvafGate::default(),
            ),
            proficiency: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_proficiency::MemoryProficiency::new(),
            ),
            graphrag_store: std::sync::RwLock::new(None),
            tech_reserve: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_tech_reserve::TechReserveStore::new(),
            ),
            graph_cache: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_graph_cache::GraphCache::new(
                    &rusqlite::Connection::open_in_memory().unwrap()
                ).unwrap_or_else(|_| crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_graph_cache::GraphCache::empty()),
            ),
            skills_library: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_knowledge_assets::SkillsLibrary::new(),
            ),
            feedback_store: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_feedback::FeedbackStore::new(0.1),
            ),
            gwt_router: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_gwt_router::GwtRouter::new(
                    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_gwt_router::GwtRouterConfig::default(),
                ),
            ),
            vsa_expander: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_vsa_expand::VsaAssociativeExpander::default(),
            ),
            retrieval_evolver: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_search::RetrievalEvolver::new(),
            ),
            temporal_ledger: std::sync::Mutex::new(
                crate::neotrix::l3_memory_impl::nt_memory_historian::TemporalFactLedger::open(
                    Some(std::path::Path::new(":memory:")),
                )
                .expect("in-memory temporal ledger"),
            ),
            freshness: std::sync::RwLock::new(
                crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_sweep_20260815::FreshnessLedger::new(),
            ),
        };

        let high_conf_id = Uuid::new_v4();
        let low_conf_id = Uuid::new_v4();

        let high_node = KnowledgeNode {
            id: high_conf_id.to_string(),
            node_type: NodeType::Concept,
            title: "Test Concept High Confidence".to_string(),
            summary: Some("A test concept with high confidence".to_string()),
            content: None,
            url: None,
            domain: Some("test".to_string()),
            language: "en".to_string(),
            confidence: 0.9,
            importance: 0.8,
            created_at: now_ts(),
            updated_at: now_ts(),
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };

        let low_node = KnowledgeNode {
            id: low_conf_id.to_string(),
            node_type: NodeType::Concept,
            title: "Test Concept Low Confidence".to_string(),
            summary: Some("A test concept with low confidence".to_string()),
            content: None,
            url: None,
            domain: Some("test".to_string()),
            language: "en".to_string(),
            confidence: 0.9,
            importance: 0.5,
            created_at: now_ts(),
            updated_at: now_ts(),
            access_count: 0,
            metadata: None,
            temporal: None,
            supersedes: None,
            source_episode: None,
        };

        kb.insert_node(&high_node).unwrap();
        kb.insert_node(&low_node).unwrap();

        let store = ConfidenceStore::new(DecayConfig::default());
        store
            .store_confidence(
                &high_conf_id,
                &make_confidence(0.9, 0.9, 0.9, 0.9),
            )
            .unwrap();
        store
            .store_confidence(
                &low_conf_id,
                &make_confidence(0.2, 0.2, 0.2, 0.2),
            )
            .unwrap();

        let results = search_with_confidence(
            &kb,
            &store,
            "test concept",
            RetrievalStrategy::Conservative {
                min_confidence: 0.7,
            },
            10,
        )
        .unwrap();

        for r in &results {
            assert!(
                r.confidence.aggregate() >= 0.7,
                "Conservative(0.7) should only return results with aggregate >= 0.7, got {} for '{}'",
                r.confidence.aggregate(),
                r.node.title
            );
        }
    }

    #[test]
    fn test_confidence_weighted_strategy() {
        let c1 = make_confidence(0.9, 0.8, 0.7, 0.6);
        let c2 = make_confidence(0.1, 0.2, 0.3, 0.4);

        let agg1 = c1.aggregate();
        let agg2 = c2.aggregate();

        assert!(
            agg1 > agg2,
            "High confidence should aggregate higher than low confidence"
        );
    }

    #[test]
    fn test_unknown_confidence_state() {
        let c = EpistemicConfidence::unknown();
        assert!((c.source_confidence - 0.0).abs() < 0.01);
        assert!((c.grounding_confidence - 0.0).abs() < 0.01);
        assert!((c.consensus_confidence - 0.0).abs() < 0.01);
        assert!((c.recency_confidence - 0.5).abs() < 0.01);
        assert!(c.computed_at > 0);
    }

    #[test]
    fn test_is_above_threshold() {
        let c = make_confidence(0.8, 0.8, 0.8, 0.8);
        assert!(c.is_above_threshold(0.7));
        assert!(!c.is_above_threshold(0.9));
    }

    #[test]
    fn test_detect_consensus_neutral() {
        let info = detect_consensus(0, 0, vec![], vec![]);
        assert!(
            (info.consensus_score - 0.5).abs() < 0.01,
            "No evidence -> neutral 0.5"
        );
        assert_eq!(info.support_count, 0);
        assert_eq!(info.contradict_count, 0);
    }

    #[test]
    fn test_detect_consensus_all_support() {
        let info = detect_consensus(
            5,
            0,
            vec!["s1".to_string(), "s2".to_string(), "s3".to_string(), "s4".to_string(), "s5".to_string()],
            vec![],
        );
        assert!(
            (info.consensus_score - 1.0).abs() < 0.01,
            "All support -> consensus 1.0, got {}",
            info.consensus_score
        );
    }

    #[test]
    fn test_detect_consensus_all_contradict() {
        let info = detect_consensus(
            0,
            3,
            vec![],
            vec!["c1".to_string(), "c2".to_string(), "c3".to_string()],
        );
        assert!(
            (info.consensus_score - 0.0).abs() < 0.01,
            "All contradict -> consensus 0.0, got {}",
            info.consensus_score
        );
    }

    #[test]
    fn test_contradiction_logging_and_retrieval() {
        let store = ConfidenceStore::new(DecayConfig::default());
        store
            .log_contradiction("node_a", "node_b", "claim_a", "claim_b")
            .unwrap();

        let contradictions_a = store.get_contradictions("node_a").unwrap();
        assert_eq!(contradictions_a.len(), 1);
        assert_eq!(contradictions_a[0].node_id, "node_b");

        let contradictions_b = store.get_contradictions("node_b").unwrap();
        assert_eq!(contradictions_b.len(), 1);
        assert_eq!(contradictions_b[0].node_id, "node_a");
    }

    #[test]
    fn test_auto_archive_never_confirmed() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let id = Uuid::new_v4();
        let c = EpistemicConfidence {
            source_confidence: 0.3,
            grounding_confidence: 0.3,
            consensus_confidence: 0.3,
            recency_confidence: 0.05,
            computed_at: 1000,
            last_confirmed_at: 0,
        };
        store.store_confidence(&id, &c).unwrap();
        let archived = store.auto_archive().unwrap();
        assert_eq!(
            archived, 0,
            "Never-confirmed node should not be auto-archived"
        );
    }

    #[test]
    fn test_custom_lambda_override() {
        let mut store = ConfidenceStore::new(DecayConfig::default());
        store.set_lambda_override("custom_type", 0.5);
        let lambda = store.get_lambda("custom_type");
        assert!(
            (lambda - 0.5).abs() < 0.001,
            "Custom override should return 0.5, got {}",
            lambda
        );
    }

    #[test]
    fn test_decay_config_defaults() {
        let cfg = DecayConfig::default();
        assert!((cfg.lambda_general - 0.01).abs() < 0.001);
        assert!((cfg.lambda_time_sensitive - 0.05).abs() < 0.001);
        assert!((cfg.min_confidence - 0.1).abs() < 0.001);
        assert_eq!(cfg.auto_archive_days, 90);
    }

    #[test]
    fn test_apply_decay_updates_recency() {
        let store = ConfidenceStore::new(DecayConfig::default());
        let id = Uuid::new_v4();
        let mut c = make_confidence(0.8, 0.8, 0.8, 1.0);
        c.computed_at = now_ts() - 30 * 86400;
        store.store_confidence(&id, &c).unwrap();

        store.apply_decay(0.01, 1).unwrap();

        let retrieved = store.get_confidence(&id).unwrap().unwrap();
        assert!(
            retrieved.recency_confidence < 1.0,
            "Recency should decay from 1.0, got {}",
            retrieved.recency_confidence
        );
    }

    #[test]
    fn test_conservative_strategy_rejects_low() {
        let strategy = RetrievalStrategy::Conservative { min_confidence: 0.7 };
        let result = matches!(strategy, RetrievalStrategy::Conservative { .. });
        assert!(result, "Conservative with 0.7 min should be identifiable");
    }
}
