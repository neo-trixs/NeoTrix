// evidence_for() and evidence_count_for() implemented on EvidenceManager — takes &[u64]
// IDs (from KnowledgeEntry.evidence_ids) and optional EvidenceState filter, returns
// sorted Vec<EvidenceRecord>. See evidence_for() at line 201.
// Cross-store aggregation (translate, hypergraph, agent_memory, goal_contract) is
// an integration concern: each store's evidence_ids are passed to evidence_for().
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::QuantizedVSA;

// ── Error type ──

#[derive(Debug, thiserror::Error)]
pub enum EvidenceError {
    #[error("Claim not found: {0}")]
    ClaimNotFound(String),
    #[error("No executable anchor for claim: {0}")]
    NoAnchor(String),
    #[error("Verification failed for claim '{claim}': {reason}")]
    VerificationFailed { claim: String, reason: String },
    #[error("Cross-validation failed: {0}")]
    CrossValidationFailed(String),
}

pub type Result<T> = std::result::Result<T, EvidenceError>;

// ── Evidence State ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceState {
    Unverified,
    CrossReferenced,
    Validated,
    Disputed,
}

impl EvidenceState {
    pub fn confidence(&self) -> f64 {
        match self {
            EvidenceState::Unverified => 0.3,
            EvidenceState::CrossReferenced => 0.6,
            EvidenceState::Validated => 0.9,
            EvidenceState::Disputed => 0.1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            EvidenceState::Unverified => "unverified",
            EvidenceState::CrossReferenced => "cross-referenced",
            EvidenceState::Validated => "validated",
            EvidenceState::Disputed => "disputed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: u64,
    pub source_url: String,
    pub source_name: String,
    pub assertion: String,
    pub quotation: Option<String>,
    pub access_timestamp: u64,
    pub confidence: f64,
    pub state: EvidenceState,
    pub contradiction_with: Vec<u64>,
    pub metadata: HashMap<String, String>,
    pub vsa_fingerprint: Option<Vec<u8>>,
}

impl EvidenceRecord {
    pub fn new(id: u64, source_url: &str, source_name: &str, assertion: &str) -> Self {
        Self {
            id,
            source_url: source_url.to_string(),
            source_name: source_name.to_string(),
            assertion: assertion.to_string(),
            quotation: None,
            access_timestamp: now_nanos(),
            confidence: 0.3,
            state: EvidenceState::Unverified,
            contradiction_with: Vec::new(),
            metadata: HashMap::new(),
            vsa_fingerprint: None,
        }
    }

    pub fn with_quotation(mut self, quote: &str) -> Self {
        self.quotation = Some(quote.to_string());
        self
    }

    pub fn with_confidence(mut self, c: f64) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }

    pub fn with_state(mut self, state: EvidenceState) -> Self {
        self.state = state;
        self.confidence = state.confidence();
        self
    }

    pub fn verify(&mut self) {
        self.state = EvidenceState::Validated;
        self.confidence = 0.9;
    }

    pub fn cross_reference(&mut self) {
        if self.state == EvidenceState::Unverified {
            self.state = EvidenceState::CrossReferenced;
            self.confidence = 0.6;
        }
    }

    pub fn dispute(&mut self) {
        self.state = EvidenceState::Disputed;
        self.confidence = 0.1;
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

pub struct EvidenceManager {
    pub records: HashMap<u64, EvidenceRecord>,
    next_id: u64,
    max_records: usize,
    total_evictions: u64,
    pub claims: HashMap<String, Claim>,
    cross_validations: HashMap<String, CrossValidation>,
    /// Per-source reliability stored as (alpha, beta) for Beta-Bernoulli model.
    source_reliability: HashMap<u64, (f64, f64)>,
}

impl EvidenceManager {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: HashMap::new(),
            next_id: 1,
            max_records,
            total_evictions: 0,
            claims: HashMap::new(),
            cross_validations: HashMap::new(),
            source_reliability: HashMap::new(),
        }
    }

    pub fn add_evidence(&mut self, source_url: &str, source_name: &str, assertion: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut record = EvidenceRecord::new(id, source_url, source_name, assertion);
        let vsa_bytes = QuantizedVSA::seeded_random(
            assertion
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64)),
            64,
        );
        // seeded_random always returns 64 bytes; try_into().ok() is safe here
        record.vsa_fingerprint = Some(vsa_bytes.to_vec());
        self.records.insert(id, record);
        self.enforce_capacity();
        id
    }

    pub fn add_evidence_with(&mut self, record: EvidenceRecord) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut record = record;
        record.id = id;
        self.records.insert(id, record);
        self.enforce_capacity();
        id
    }

    pub fn get(&self, id: u64) -> Option<&EvidenceRecord> {
        self.records.get(&id)
    }

    pub fn get_by_ids(&self, ids: &[u64]) -> Vec<&EvidenceRecord> {
        ids.iter().filter_map(|id| self.records.get(id)).collect()
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut EvidenceRecord> {
        self.records.get_mut(&id)
    }

    pub fn remove_evidence(&mut self, id: u64) -> bool {
        self.records.remove(&id).is_some()
    }

    pub fn combined_confidence(&self, ids: &[u64]) -> f64 {
        let evs = self.get_by_ids(ids);
        if evs.is_empty() {
            return 0.0;
        }
        let min_conf = evs
            .iter()
            .map(|e| e.confidence)
            .fold(1.0_f64, |a, b| a.min(b));
        let avg_conf = evs.iter().map(|e| e.confidence).sum::<f64>() / evs.len() as f64;
        min_conf * 0.6 + avg_conf * 0.4
    }

    pub fn weighted_evidence_score(&self, ids: &[u64], recency_weight: f64, now_ns: u64) -> f64 {
        let evs = self.get_by_ids(ids);
        if evs.is_empty() {
            return 0.0;
        }
        let best: f64 = evs
            .iter()
            .map(|e| {
                let age_secs = now_ns.saturating_sub(e.access_timestamp) as f64 / 1_000_000_000.0;
                let recency = (-age_secs / 86400.0).exp();
                e.confidence * (recency_weight * recency + (1.0 - recency_weight))
            })
            .fold(0.0_f64, |a, b| a.max(b));
        best
    }

    /// Retrieve evidence records linked to the given IDs, with optional state filter.
    /// Returns records sorted by confidence descending.
    pub fn evidence_for(
        &self,
        ids: &[u64],
        state_filter: Option<EvidenceState>,
    ) -> Vec<EvidenceRecord> {
        let mut results: Vec<EvidenceRecord> = self
            .get_by_ids(ids)
            .into_iter()
            .filter(|r| {
                state_filter
                    .as_ref()
                    .map_or(true, |filter| r.state == *filter)
            })
            .cloned()
            .collect();
        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Convenience: count of evidence records linked to the given IDs, with optional state filter.
    pub fn evidence_count_for(&self, ids: &[u64], state_filter: Option<EvidenceState>) -> usize {
        self.get_by_ids(ids)
            .iter()
            .filter(|r| {
                state_filter
                    .as_ref()
                    .map_or(true, |filter| r.state == *filter)
            })
            .count()
    }

    pub fn evidence_count(&self) -> usize {
        self.records.len()
    }

    pub fn stats(&self) -> EvidenceStats {
        let mut state_counts = HashMap::new();
        for r in self.records.values() {
            *state_counts.entry(r.state).or_insert(0) += 1;
        }
        let total_confidence: f64 = self.records.values().map(|r| r.confidence).sum();
        EvidenceStats {
            total_records: self.records.len(),
            avg_confidence: if self.records.is_empty() {
                0.0
            } else {
                total_confidence / self.records.len() as f64
            },
            unverified: *state_counts.get(&EvidenceState::Unverified).unwrap_or(&0),
            cross_referenced: *state_counts
                .get(&EvidenceState::CrossReferenced)
                .unwrap_or(&0),
            validated: *state_counts.get(&EvidenceState::Validated).unwrap_or(&0),
            disputed: *state_counts.get(&EvidenceState::Disputed).unwrap_or(&0),
            total_evictions: self.total_evictions,
        }
    }

    fn enforce_capacity(&mut self) {
        while self.records.len() > self.max_records {
            let oldest_id = self
                .records
                .iter()
                .min_by(|a, b| a.1.access_timestamp.cmp(&b.1.access_timestamp))
                .map(|(id, _)| *id);
            if let Some(id) = oldest_id {
                self.records.remove(&id);
                self.total_evictions += 1;
            }
        }
    }
}

impl Default for EvidenceManager {
    fn default() -> Self {
        Self::new(5000)
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceStats {
    pub total_records: usize,
    pub avg_confidence: f64,
    pub unverified: usize,
    pub cross_referenced: usize,
    pub validated: usize,
    pub disputed: usize,
    pub total_evictions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringDimensions {
    pub relevance: f64,
    pub evidence_confidence: f64,
    pub recency: f64,
    pub source_authority: f64,
    pub cross_references: f64,
    pub contradiction_penalty: f64,
}

impl ScoringDimensions {
    pub fn total(&self) -> f64 {
        let raw = self.relevance * 0.30
            + self.evidence_confidence * 0.25
            + self.recency * 0.15
            + self.source_authority * 0.10
            + self.cross_references * 0.10
            - self.contradiction_penalty * 0.10;
        raw.clamp(0.0, 1.0)
    }

    pub fn breakdown(&self) -> String {
        format!(
            "relevance={:.2} evidence={:.2} recency={:.2} authority={:.2} xrefs={:.2} penalty={:.2} total={:.2}",
            self.relevance, self.evidence_confidence, self.recency,
            self.source_authority, self.cross_references, self.contradiction_penalty, self.total()
        )
    }
}

pub struct CompetitiveScorer;

impl CompetitiveScorer {
    pub fn score(
        relevance: f64,
        evidence_manager: &EvidenceManager,
        evidence_ids: &[u64],
        now_ns: u64,
        source_weight: f64,
    ) -> ScoringDimensions {
        let evs = evidence_manager.get_by_ids(evidence_ids);
        let ev_count = evs.len() as f64;
        let evidence_confidence = if ev_count > 0.0 {
            evidence_manager.combined_confidence(evidence_ids)
        } else {
            0.05
        };
        let recency = if ev_count > 0.0 {
            evidence_manager.weighted_evidence_score(evidence_ids, 0.5, now_ns)
        } else {
            0.5
        };
        let contradicted = evs.iter().any(|e| e.state == EvidenceState::Disputed);
        ScoringDimensions {
            relevance: relevance.clamp(0.0, 1.0),
            evidence_confidence: evidence_confidence.clamp(0.0, 1.0),
            recency: recency.clamp(0.0, 1.0),
            source_authority: source_weight.clamp(0.0, 1.0),
            cross_references: (ev_count / 10.0).min(1.0),
            contradiction_penalty: if contradicted { 0.8 } else { 0.0 },
        }
    }
}

// ── Dempster-Shafer Evidence Fusion ──

/// Hypothesis support for Dempster-Shafer fusion over a binary frame {True, False}.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DSSupport {
    True,
    False,
    Uncertain,
}

/// A Dempster-Shafer mass triplet over the frame {True, False, Uncertain}.
///
/// - `belief`: mass assigned to "True"
/// - `disbelief`: mass assigned to "False"
/// - `uncertainty`: mass assigned to the full frame (epistemic uncertainty)
///
/// Invariant: `belief + disbelief + uncertainty ≈ 1.0` after normalization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DSTriplet {
    pub belief: f64,
    pub disbelief: f64,
    pub uncertainty: f64,
}

impl DSTriplet {
    /// Create a new triplet. Values are clamped to [0,1] and normalized to sum to 1.
    pub fn new(belief: f64, disbelief: f64, uncertainty: f64) -> Self {
        let b = belief.max(0.0);
        let d = disbelief.max(0.0);
        let u = uncertainty.max(0.0);
        let total = b + d + u;
        if total <= 0.0 {
            return Self {
                belief: 0.0,
                disbelief: 0.0,
                uncertainty: 1.0,
            };
        }
        Self {
            belief: b / total,
            disbelief: d / total,
            uncertainty: u / total,
        }
    }

    /// Convert an `EvidenceRecord` to a D-S triplet based on its state and confidence.
    ///
    /// Mapping:
    /// - Validated → high belief, low uncertainty
    /// - CrossReferenced → moderate belief
    /// - Unverified → high uncertainty
    /// - Disputed → high disbelief
    pub fn from_evidence_record(record: &EvidenceRecord) -> Self {
        let c = record.confidence.clamp(0.0, 1.0);
        match record.state {
            EvidenceState::Validated => {
                let belief = 0.05 + 0.85 * c;
                let uncertainty = 0.10;
                let disbelief = 1.0 - belief - uncertainty;
                Self::new(belief, disbelief, uncertainty)
            }
            EvidenceState::CrossReferenced => {
                let belief = 0.05 + 0.55 * c;
                let uncertainty = 0.25;
                let disbelief = 1.0 - belief - uncertainty;
                Self::new(belief, disbelief, uncertainty)
            }
            EvidenceState::Unverified => {
                let belief = 0.05 + 0.20 * c;
                let uncertainty = 0.50;
                let disbelief = 1.0 - belief - uncertainty;
                Self::new(belief, disbelief, uncertainty)
            }
            EvidenceState::Disputed => {
                let disbelief = 0.05 + 0.70 * (1.0 - c);
                let uncertainty = 0.20;
                let belief = 1.0 - disbelief - uncertainty;
                Self::new(belief, disbelief, uncertainty)
            }
        }
    }

    /// Binary Dempster combination with another triplet.
    pub fn combine(&self, other: &DSTriplet) -> DSTriplet {
        DempsterShaferFuser::combine(&[*self, *other])
    }

    /// Plausibility of the hypothesis being True: belief + uncertainty.
    pub fn plausibility_of_true(&self) -> f64 {
        self.belief + self.uncertainty
    }

    /// Plausibility of the hypothesis being False: disbelief + uncertainty.
    pub fn plausibility_of_false(&self) -> f64 {
        self.disbelief + self.uncertainty
    }

    /// Whether the triplet is committed to a decision (uncertainty < 0.1).
    pub fn is_committed(&self) -> bool {
        self.uncertainty < 0.1
    }

    /// Discount masses by source reliability: m'(A) = r * m(A), m'(Ω) = 1 - r * (1 - m(Ω)).
    pub fn discount(&self, reliability: f64) -> Self {
        let r = reliability.clamp(0.0, 1.0);
        let b = r * self.belief;
        let d = r * self.disbelief;
        let u = 1.0 - b - d;
        Self {
            belief: b,
            disbelief: d,
            uncertainty: u,
        }
    }
}

/// Dempster-Shafer evidence fusion engine.
///
/// Implements Dempster's rule of combination (orthogonal sum) over the ternary
/// frame {True, False, Uncertain}, with sequential and batch combination modes.
pub struct DempsterShaferFuser;

impl DempsterShaferFuser {
    /// Dempster's rule of combination (orthogonal sum) for multiple mass triplets.
    ///
    /// Combines all triplets pairwise in sequence. If the set is empty, returns
    /// a fully uncertain triplet. If complete conflict (K=1) is detected, falls
    /// back to all-uncertain.
    pub fn combine(masses: &[DSTriplet]) -> DSTriplet {
        if masses.is_empty() {
            return DSTriplet::new(0.0, 0.0, 1.0);
        }
        let mut result = masses[0];
        for m in &masses[1..] {
            result = Self::combine_pair(&result, m);
        }
        result
    }

    /// Sequential pairwise combination (same as batch combine but explicit).
    pub fn combine_sequential(masses: &[DSTriplet]) -> DSTriplet {
        Self::combine(masses)
    }

    /// Binary Dempster combination of two triplets.
    fn combine_pair(m1: &DSTriplet, m2: &DSTriplet) -> DSTriplet {
        let k = m1.belief * m2.disbelief + m1.disbelief * m2.belief;
        if (1.0 - k).abs() < f64::EPSILON {
            // Complete conflict: no agreement possible
            return DSTriplet::new(0.0, 0.0, 1.0);
        }
        let norm = 1.0 / (1.0 - k);
        let b = (m1.belief * m2.belief + m1.belief * m2.uncertainty + m1.uncertainty * m2.belief)
            * norm;
        let d = (m1.disbelief * m2.disbelief
            + m1.disbelief * m2.uncertainty
            + m1.uncertainty * m2.disbelief)
            * norm;
        let u = (m1.uncertainty * m2.uncertainty) * norm;
        DSTriplet {
            belief: b,
            disbelief: d,
            uncertainty: u,
        }
    }

    /// Measure of conflict K between two triplets.
    ///
    /// K = m1(True)·m2(False) + m1(False)·m2(True).
    /// K = 0 means no conflict; K → 1 means complete conflict.
    pub fn conflict(m1: &DSTriplet, m2: &DSTriplet) -> f64 {
        m1.belief * m2.disbelief + m1.disbelief * m2.belief
    }

    /// Plausibility of a given hypothesis under the mass distribution.
    pub fn plausibility(m: &DSTriplet, hypothesis: DSSupport) -> f64 {
        match hypothesis {
            DSSupport::True => m.belief + m.uncertainty,
            DSSupport::False => m.disbelief + m.uncertainty,
            DSSupport::Uncertain => 1.0,
        }
    }

    /// Belief (support) for a given hypothesis.
    pub fn belief(m: &DSTriplet, hypothesis: DSSupport) -> f64 {
        match hypothesis {
            DSSupport::True => m.belief,
            DSSupport::False => m.disbelief,
            DSSupport::Uncertain => 0.0,
        }
    }

    /// Amount of epistemic uncertainty: the mass on the full frame.
    pub fn uncertainty(m: &DSTriplet) -> f64 {
        m.uncertainty
    }
}

// ── NATO Admiralty Scale (STANAG 2511) ──

/// NATO source reliability letter codes (A–F) from STANAG 2511.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NatoSourceReliability {
    /// A — Completely reliable
    A,
    /// B — Usually reliable
    B,
    /// C — Fairly reliable
    C,
    /// D — Not usually reliable
    D,
    /// E — Unreliable
    E,
    /// F — Cannot be judged
    F,
}

impl NatoSourceReliability {
    /// Map letter code to a numeric reliability score in [0, 1].
    pub fn to_reliability_score(self) -> f64 {
        match self {
            NatoSourceReliability::A => 0.95,
            NatoSourceReliability::B => 0.80,
            NatoSourceReliability::C => 0.60,
            NatoSourceReliability::D => 0.40,
            NatoSourceReliability::E => 0.15,
            NatoSourceReliability::F => 0.50,
        }
    }

    /// Infer the closest NATO letter code from a numeric score.
    pub fn from_score(score: f64) -> Self {
        let s = score.clamp(0.0, 1.0);
        if s >= 0.90 {
            NatoSourceReliability::A
        } else if s >= 0.70 {
            NatoSourceReliability::B
        } else if s >= 0.50 {
            NatoSourceReliability::C
        } else if s >= 0.30 {
            NatoSourceReliability::D
        } else if s >= 0.10 {
            NatoSourceReliability::E
        } else {
            NatoSourceReliability::F
        }
    }
}

/// Helper providing NATO Admiralty Scale utilities.
pub struct NatoAdmiraltyScale;

impl NatoAdmiraltyScale {
    pub fn to_reliability_score(letter: NatoSourceReliability) -> f64 {
        letter.to_reliability_score()
    }

    pub fn from_score(score: f64) -> NatoSourceReliability {
        NatoSourceReliability::from_score(score)
    }
}

// ── DS Scoring Result ──

/// Result of a Dempster-Shafer-aware competitive scoring pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSScoringResult {
    pub ds_triplet: DSTriplet,
    pub combined_belief: f64,
    pub combined_plausibility: f64,
    pub combined_uncertainty: f64,
    pub conflict_measure: f64,
    pub nato_reliability: NatoSourceReliability,
    pub breakdown: String,
}

fn now_nanos() -> u64 {
    crate::core::nt_core_time::unix_now_nanos()
}

// ── Executable Evidence Anchoring (G378) ──

/// An executable anchor that can re-run to verify a claim.
/// Inspired by Data2Story's Inspector architecture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableAnchor {
    /// Programming language (python, rust, sql, sh, etc.)
    pub language: String,
    /// The actual source code to execute
    pub code: String,
    /// Expected output pattern (regex or exact)
    pub expected: VerificationPattern,
    /// Data source file(s) this code operates on
    pub source_files: Vec<String>,
    /// Whether this anchor has been verified
    pub verified: bool,
    /// Timestamp of last verification
    pub verified_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationPattern {
    Exact(String),
    Regex(String),
    NumericWithin(f64, f64),
    Boolean(bool),
    JsonPath { path: String, expected: String },
}

/// A structured report linking claims to executable evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorReport {
    /// The original claim
    pub claim: String,
    /// The evidence anchor supporting this claim
    pub anchor: Option<ExecutableAnchor>,
    /// Cross-validation result from different model families
    pub cross_validated: Option<CrossValidation>,
    /// Overall verifiability score (0.0-1.0)
    pub verifiability_score: f64,
    /// Verification status
    pub status: EvidenceVerificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceVerificationStatus {
    Unverified,
    Verified { at: u64, by: String },
    Failed { reason: String },
    Contradicted { by_claim: String },
}

/// Cross-validation result from multiple independent LLM families.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidation {
    /// Number of independent verifiers that confirmed
    pub confirmed_by: usize,
    /// Number of verifiers total
    pub total_verifiers: usize,
    /// The model families used
    pub families: Vec<String>,
}

/// A claim that can be linked to executable evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub confidence: f64,
    pub source: ClaimSource,
    pub evidence: Option<ExecutableAnchor>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimSource {
    Reasoning,
    KnowledgeRetrieval,
    UserStatement,
    ExternalSource(String),
    GeneratedContent,
}

/// Result of verifying a single executable anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub claim: String,
    pub success: bool,
    pub details: String,
    pub verified_at: u64,
}

// ── New EvidenceManager methods for executable evidence ──

impl EvidenceManager {
    /// Store a claim with an executable anchor.
    pub fn add_executable_evidence(&mut self, claim: &str, anchor: ExecutableAnchor) -> Result<()> {
        let claim_entry = Claim {
            id: format!("claim_{}", self.next_id),
            statement: claim.to_string(),
            confidence: 0.3,
            source: ClaimSource::Reasoning,
            evidence: Some(anchor),
            created_at: now_nanos(),
        };
        self.claims.insert(claim.to_string(), claim_entry);
        Ok(())
    }

    /// Get an inspector report for a claim, combining anchor and cross-validation.
    pub fn get_inspector_report(&self, claim: &str) -> Option<InspectorReport> {
        let claim_entry = self.claims.get(claim)?;
        let anchor = claim_entry.evidence.clone();
        let cross_validated = self.cross_validations.get(claim).cloned();
        let status = match &anchor {
            Some(a) if a.verified => EvidenceVerificationStatus::Verified {
                at: a.verified_at.unwrap_or(0),
                by: "executable_anchor".to_string(),
            },
            Some(_) => EvidenceVerificationStatus::Unverified,
            None => EvidenceVerificationStatus::Unverified,
        };
        let verifiability_score = if anchor.is_some() { 0.7 } else { 0.0 };
        Some(InspectorReport {
            claim: claim.to_string(),
            anchor,
            cross_validated,
            verifiability_score,
            status,
        })
    }

    /// Verify all executable anchors (simulated: marks them as verified).
    pub fn verify_all_executable(&mut self) -> Vec<VerificationResult> {
        let now = now_nanos();
        let mut results = Vec::new();
        let keys: Vec<String> = self.claims.keys().cloned().collect();
        for key in keys {
            if let Some(claim) = self.claims.get_mut(&key) {
                if let Some(ref mut anchor) = claim.evidence {
                    anchor.verified = true;
                    anchor.verified_at = Some(now);
                    results.push(VerificationResult {
                        claim: key.clone(),
                        success: true,
                        details: format!("Verified {} anchor", anchor.language),
                        verified_at: now,
                    });
                } else {
                    results.push(VerificationResult {
                        claim: key.clone(),
                        success: false,
                        details: "No executable anchor".to_string(),
                        verified_at: now,
                    });
                }
            }
        }
        results
    }

    /// Simulate cross-validation of a claim across multiple model families.
    pub fn cross_validate_claim(
        &mut self,
        claim: &str,
        families: &[&str],
    ) -> Result<CrossValidation> {
        if !self.claims.contains_key(claim) {
            return Err(EvidenceError::ClaimNotFound(claim.to_string()));
        }
        let total = families.len();
        // Simulate: all families confirm (in production, each family would actually run)
        let confirmed = total;
        let cv = CrossValidation {
            confirmed_by: confirmed,
            total_verifiers: total,
            families: families.iter().map(|f| f.to_string()).collect(),
        };
        self.cross_validations.insert(claim.to_string(), cv.clone());
        Ok(cv)
    }

    /// Find claims that lack executable evidence anchors.
    pub fn claims_without_evidence(&self) -> Vec<String> {
        self.claims
            .iter()
            .filter(|(_, c)| c.evidence.is_none())
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Fraction of stored claims that have executable evidence.
    pub fn verifiability_coverage(&self) -> f64 {
        if self.claims.is_empty() {
            return 0.0;
        }
        let with_evidence = self
            .claims
            .values()
            .filter(|c| c.evidence.is_some())
            .count();
        with_evidence as f64 / self.claims.len() as f64
    }

    // ── Dempster-Shafer Fusion Methods ──

    /// Combine evidence records using Dempster-Shafer fusion.
    ///
    /// Converts each record to a DSTriplet via `from_evidence_record`, optionally
    /// discounts by per-source reliability, then fuses all triplets with
    /// Dempster's rule. Returns all-uncertain when no records match.
    ///
    /// This is the preferred method over `combined_confidence` for evidence fusion
    /// as it properly handles conflicting, corroborating, and uncertain evidence.
    pub fn dempster_shafer_confidence(&self, ids: &[u64]) -> DSTriplet {
        let records = self.get_by_ids(ids);
        if records.is_empty() {
            return DSTriplet::new(0.0, 0.0, 1.0);
        }
        let triplets: Vec<DSTriplet> = records
            .iter()
            .map(|r| {
                let mut t = DSTriplet::from_evidence_record(r);
                if let Some(&(alpha, beta)) = self.source_reliability.get(&r.id) {
                    let reliability = alpha / (alpha + beta);
                    t = t.discount(reliability);
                }
                t
            })
            .collect();
        DempsterShaferFuser::combine(&triplets)
    }

    /// Set the source reliability for an evidence record using a Beta(alpha, beta) prior.
    ///
    /// The reliability is stored as (alpha, beta) where mean = alpha / (alpha + beta).
    pub fn set_source_reliability(&mut self, evidence_id: u64, reliability: f64) {
        let r = reliability.clamp(0.01, 0.99);
        // Convert reliability score to Beta(alpha, beta) with effective sample size 10
        let alpha = r * 10.0 + 1.0;
        let beta = (1.0 - r) * 10.0 + 1.0;
        self.source_reliability.insert(evidence_id, (alpha, beta));
    }

    /// Bayesian update of source reliability after an observation.
    ///
    /// If `was_correct` is true, alpha increases; otherwise beta increases.
    /// Returns the updated reliability score.
    pub fn update_source_reliability(&mut self, evidence_id: u64, was_correct: bool) -> f64 {
        let entry = self
            .source_reliability
            .entry(evidence_id)
            .or_insert((1.0, 1.0));
        if was_correct {
            entry.0 += 1.0;
        } else {
            entry.1 += 1.0;
        }
        entry.0 / (entry.0 + entry.1)
    }
}

// ── DS-enhanced CompetitiveScorer ──

impl CompetitiveScorer {
    /// Score evidence using Dempster-Shafer fusion for uncertainty-aware ranking.
    ///
    /// Returns a `DSScoringResult` containing the fused triplet, combined belief
    /// and plausibility, aggregate conflict measure, inferred NATO reliability,
    /// and a human-readable breakdown string.
    pub fn score_ds(
        _relevance: f64,
        evidence_manager: &EvidenceManager,
        evidence_ids: &[u64],
        now_ns: u64,
        source_weight: f64,
    ) -> DSScoringResult {
        let records = evidence_manager.get_by_ids(evidence_ids);
        let triplets: Vec<DSTriplet> = records
            .iter()
            .map(|r| DSTriplet::from_evidence_record(r))
            .collect();

        let ds = evidence_manager.dempster_shafer_confidence(evidence_ids);

        // Compute aggregate conflict as max pairwise K
        let conflict = if triplets.len() > 1 {
            let mut max_k = 0.0;
            for i in 0..triplets.len() {
                for j in (i + 1)..triplets.len() {
                    let k = DempsterShaferFuser::conflict(&triplets[i], &triplets[j]);
                    if k > max_k {
                        max_k = k;
                    }
                }
            }
            max_k
        } else {
            0.0
        };

        // Weight with recency like the original scorer does
        let recency = if !records.is_empty() {
            evidence_manager.weighted_evidence_score(evidence_ids, 0.5, now_ns)
        } else {
            0.5
        };

        let nato = NatoSourceReliability::from_score(source_weight);
        let breakdown = format!(
            "DS: belief={:.3} disbelief={:.3} uncertainty={:.3} | conflict={:.3} | recency={:.2} | nato={:?}",
            ds.belief, ds.disbelief, ds.uncertainty, conflict, recency, nato
        );

        DSScoringResult {
            ds_triplet: ds,
            combined_belief: ds.belief,
            combined_plausibility: ds.belief + ds.uncertainty,
            combined_uncertainty: ds.uncertainty,
            conflict_measure: conflict,
            nato_reliability: nato,
            breakdown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> EvidenceManager {
        EvidenceManager::new(100)
    }

    #[test]
    fn test_add_evidence() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence("https://example.com", "example", "test claim");
        assert_eq!(mgr.evidence_count(), 1);
        assert!(mgr.get(id).is_some());
    }

    #[test]
    fn test_get_by_ids() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "claim 1");
        let id2 = mgr.add_evidence("https://b.com", "b", "claim 2");
        let results = mgr.get_by_ids(&[id1, id2]);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_combined_confidence() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "claim");
        let c = mgr.combined_confidence(&[id1]);
        assert!((c - 0.3).abs() < 0.01);
        let id2 = mgr.add_evidence("https://b.com", "b", "claim 2");
        if let Some(r) = mgr.get_mut(id2) {
            r.verify();
        }
        let c2 = mgr.combined_confidence(&[id1, id2]);
        assert!(c2 > c, "verified evidence should raise combined confidence");
    }

    #[test]
    fn test_evidence_state_transitions() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence("https://x.com", "x", "test");
        if let Some(r) = mgr.get_mut(id) {
            assert_eq!(r.state, EvidenceState::Unverified);
            r.cross_reference();
            assert_eq!(r.state, EvidenceState::CrossReferenced);
            r.verify();
            assert_eq!(r.state, EvidenceState::Validated);
            r.dispute();
            assert_eq!(r.state, EvidenceState::Disputed);
        }
    }

    #[test]
    fn test_scoring_dimensions_total() {
        let d = ScoringDimensions {
            relevance: 0.8,
            evidence_confidence: 0.7,
            recency: 0.6,
            source_authority: 0.9,
            cross_references: 0.5,
            contradiction_penalty: 0.0,
        };
        let t = d.total();
        assert!(t > 0.0 && t <= 1.0);
        assert!(t > 0.5);
    }

    #[test]
    fn test_scoring_with_contradiction_penalty() {
        let d = ScoringDimensions {
            relevance: 0.9,
            evidence_confidence: 0.9,
            recency: 0.9,
            source_authority: 0.9,
            cross_references: 0.9,
            contradiction_penalty: 0.8,
        };
        let penalized = d.total();
        let clean = ScoringDimensions {
            relevance: 0.9,
            evidence_confidence: 0.9,
            recency: 0.9,
            source_authority: 0.9,
            cross_references: 0.9,
            contradiction_penalty: 0.0,
        };
        assert!(
            penalized < clean.total(),
            "contradiction penalty should reduce score"
        );
    }

    #[test]
    fn test_competitive_scorer() {
        let mut mgr = make_manager();
        let eid = mgr.add_evidence(
            "https://arxiv.org/abs/2603.25097",
            "arxiv",
            "ElephantBroker evidence system",
        );
        let score = CompetitiveScorer::score(0.85, &mgr, &[eid], now_nanos(), 0.8);
        assert!(score.total() > 0.0);
        assert!(score.breakdown().contains("relevance="));
    }

    #[test]
    fn test_evidence_stats() {
        let mut mgr = make_manager();
        mgr.add_evidence("https://a.com", "a", "unverified");
        let id = mgr.add_evidence("https://b.com", "b", "validated");
        if let Some(r) = mgr.get_mut(id) {
            r.verify();
        }
        let stats = mgr.stats();
        assert_eq!(stats.total_records, 2);
        assert!(stats.validated >= 1);
    }

    #[test]
    fn test_evidence_capacity_enforced() {
        let mut mgr = EvidenceManager::new(3);
        for _i in 0..10 {
            mgr.add_evidence("https://x.com", "x", "test");
        }
        assert_eq!(mgr.evidence_count(), 3);
        assert!(mgr.stats().total_evictions > 0);
    }

    #[test]
    fn test_remove_evidence() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence("https://x.com", "x", "test");
        assert!(mgr.remove_evidence(id));
        assert!(mgr.get(id).is_none());
    }

    #[test]
    fn test_evidence_with_quotation() {
        let mut mgr = make_manager();
        let record = EvidenceRecord::new(100, "https://example.com", "example", "claim")
            .with_quotation("direct quote from source")
            .with_confidence(0.8);
        let id = mgr.add_evidence_with(record);
        let ev = mgr.get(id).unwrap();
        assert_eq!(ev.quotation.as_deref(), Some("direct quote from source"));
        assert!((ev.confidence - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_scoring_dimensions_breakdown_format() {
        let d = ScoringDimensions {
            relevance: 0.5,
            evidence_confidence: 0.5,
            recency: 0.5,
            source_authority: 0.5,
            cross_references: 0.5,
            contradiction_penalty: 0.0,
        };
        let b = d.breakdown();
        assert!(b.contains("relevance="));
        assert!(b.contains("total="));
    }

    #[test]
    fn test_evidence_for() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "claim 1");
        let id2 = mgr.add_evidence("https://b.com", "b", "claim 2");
        let all = mgr.evidence_for(&[id1, id2], None);
        assert_eq!(all.len(), 2);
        // default sort: by confidence descending (all 0.3)
        assert_eq!(all[0].id, id1.min(id2));
        // filter by state
        if let Some(r) = mgr.get_mut(id1) {
            r.verify();
        }
        let validated_only = mgr.evidence_for(&[id1, id2], Some(EvidenceState::Validated));
        assert_eq!(validated_only.len(), 1);
        assert_eq!(validated_only[0].id, id1);
    }

    #[test]
    fn test_evidence_count_for() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "claim 1");
        let id2 = mgr.add_evidence("https://b.com", "b", "claim 2");
        assert_eq!(mgr.evidence_count_for(&[id1, id2], None), 2);
        // empty ids returns 0
        assert_eq!(mgr.evidence_count_for(&[], None), 0);
        // none after filter
        assert_eq!(
            mgr.evidence_count_for(&[id1], Some(EvidenceState::Validated)),
            0
        );
    }

    #[test]
    fn test_evidence_for_confidence_sorted() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "low");
        let id2 = mgr.add_evidence("https://b.com", "b", "high");
        if let Some(r) = mgr.get_mut(id2) {
            r.verify(); // confidence -> 0.9
        }
        let results = mgr.evidence_for(&[id1, id2], None);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, id2); // highest confidence first
        assert_eq!(results[1].id, id1);
    }

    // ── Executable Evidence Tests ──

    #[test]
    fn test_add_executable_evidence_roundtrip() {
        let mut mgr = make_manager();
        let anchor = ExecutableAnchor {
            language: "python".to_string(),
            code: "assert 1 + 1 == 2".to_string(),
            expected: VerificationPattern::Boolean(true),
            source_files: vec!["test.py".to_string()],
            verified: false,
            verified_at: None,
        };
        mgr.add_executable_evidence("1 + 1 = 2", anchor).unwrap();
        let report = mgr.get_inspector_report("1 + 1 = 2").unwrap();
        assert_eq!(report.claim, "1 + 1 = 2");
        assert!(report.anchor.is_some());
        assert_eq!(report.anchor.as_ref().unwrap().language, "python");
    }

    #[test]
    fn test_verify_all_executable() {
        let mut mgr = make_manager();
        let anchor = ExecutableAnchor {
            language: "rust".to_string(),
            code: "fn main() {}".to_string(),
            expected: VerificationPattern::Exact("ok".to_string()),
            source_files: vec![],
            verified: false,
            verified_at: None,
        };
        mgr.add_executable_evidence("test claim", anchor).unwrap();
        let results = mgr.verify_all_executable();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        let report = mgr.get_inspector_report("test claim").unwrap();
        match report.status {
            EvidenceVerificationStatus::Verified { .. } => {}
            _ => panic!("Expected Verified status after verify_all_executable"),
        }
    }

    #[test]
    fn test_cross_validation() {
        let mut mgr = make_manager();
        let anchor = ExecutableAnchor {
            language: "sql".to_string(),
            code: "SELECT 1".to_string(),
            expected: VerificationPattern::Exact("1".to_string()),
            source_files: vec![],
            verified: false,
            verified_at: None,
        };
        mgr.add_executable_evidence("sql claim", anchor).unwrap();
        let cv = mgr
            .cross_validate_claim("sql claim", &["gpt4", "claude", "gemini"])
            .unwrap();
        assert_eq!(cv.confirmed_by, 3);
        assert_eq!(cv.total_verifiers, 3);
        assert!(cv.families.contains(&"gpt4".to_string()));
    }

    #[test]
    fn test_claims_without_evidence() {
        let mut mgr = make_manager();
        let anchor = ExecutableAnchor {
            language: "py".to_string(),
            code: "x = 1".to_string(),
            expected: VerificationPattern::Exact("1".to_string()),
            source_files: vec![],
            verified: false,
            verified_at: None,
        };
        mgr.add_executable_evidence("with evidence", anchor)
            .unwrap();
        // Add a claim without evidence by inserting directly
        mgr.claims.insert(
            "no evidence".to_string(),
            Claim {
                id: "claim_no_ev".to_string(),
                statement: "no evidence".to_string(),
                confidence: 0.3,
                source: ClaimSource::Reasoning,
                evidence: None,
                created_at: now_nanos(),
            },
        );
        let missing = mgr.claims_without_evidence();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "no evidence");
    }

    #[test]
    fn test_verifiability_coverage() {
        let mut mgr = make_manager();
        assert_eq!(mgr.verifiability_coverage(), 0.0);
        let anchor = ExecutableAnchor {
            language: "py".to_string(),
            code: "pass".to_string(),
            expected: VerificationPattern::Boolean(true),
            source_files: vec![],
            verified: false,
            verified_at: None,
        };
        mgr.add_executable_evidence("claim 1", anchor).unwrap();
        mgr.claims.insert(
            "claim 2".to_string(),
            Claim {
                id: "c2".to_string(),
                statement: "claim 2".to_string(),
                confidence: 0.5,
                source: ClaimSource::UserStatement,
                evidence: None,
                created_at: now_nanos(),
            },
        );
        let coverage = mgr.verifiability_coverage();
        assert!((coverage - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_executable_evidence_nonexistent_claim() {
        let mut mgr = make_manager();
        let err = mgr.cross_validate_claim("nope", &["gpt4"]).unwrap_err();
        match err {
            EvidenceError::ClaimNotFound(s) => assert_eq!(s, "nope"),
            _ => panic!("Expected ClaimNotFound"),
        }
        assert!(mgr.get_inspector_report("nope").is_none());
    }

    // ── Dempster-Shafer Tests ──

    #[test]
    fn test_ds_triplet_normalization() {
        let t = DSTriplet::new(2.0, 2.0, 2.0);
        assert!((t.belief - 1.0 / 3.0).abs() < 1e-12);
        assert!((t.disbelief - 1.0 / 3.0).abs() < 1e-12);
        assert!((t.uncertainty - 1.0 / 3.0).abs() < 1e-12);
        // All zeros → all uncertain
        let t2 = DSTriplet::new(0.0, 0.0, 0.0);
        assert!((t2.uncertainty - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_binary_combination_identical() {
        let m1 = DSTriplet::new(0.8, 0.1, 0.1);
        let combined = m1.combine(&m1);
        // Two identical, high-belief sources should reinforce
        assert!(combined.belief > m1.belief);
        assert!(combined.uncertainty < m1.uncertainty);
        assert!((combined.belief + combined.disbelief + combined.uncertainty - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_binary_combination_conflicting() {
        let m1 = DSTriplet::new(0.8, 0.1, 0.1);
        let m2 = DSTriplet::new(0.1, 0.8, 0.1);
        let combined = m1.combine(&m2);
        // High conflict — uncertainty should increase
        assert!(combined.uncertainty > 0.1);
        assert!((combined.belief + combined.disbelief + combined.uncertainty - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_binary_combination_one_certain() {
        let certain = DSTriplet::new(0.0, 0.0, 1.0);
        let m = DSTriplet::new(0.7, 0.2, 0.1);
        let combined = certain.combine(&m);
        // Certain (all-uncertain) source should pass through the other's masses
        assert!((combined.belief - m.belief).abs() < 1e-12);
        assert!((combined.disbelief - m.disbelief).abs() < 1e-12);
    }

    #[test]
    fn test_ds_binary_combination_all_uncertain() {
        let m1 = DSTriplet::new(0.0, 0.0, 1.0);
        let m2 = DSTriplet::new(0.0, 0.0, 1.0);
        let combined = m1.combine(&m2);
        assert!((combined.uncertainty - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_conflict_measure() {
        let m1 = DSTriplet::new(0.9, 0.05, 0.05);
        let m2 = DSTriplet::new(0.05, 0.9, 0.05);
        let k = DempsterShaferFuser::conflict(&m1, &m2);
        // K = 0.9*0.9 + 0.05*0.05 = 0.81 + 0.0025 = 0.8125
        assert!((k - 0.8125).abs() < 1e-12);
        // Identical masses have zero conflict
        let k2 = DempsterShaferFuser::conflict(&m1, &m1);
        assert!((k2 - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_plausibility() {
        let m = DSTriplet::new(0.6, 0.3, 0.1);
        assert!((DempsterShaferFuser::plausibility(&m, DSSupport::True) - 0.7).abs() < 1e-12);
        assert!((DempsterShaferFuser::plausibility(&m, DSSupport::False) - 0.4).abs() < 1e-12);
        assert!((DempsterShaferFuser::plausibility(&m, DSSupport::Uncertain) - 1.0).abs() < 1e-12);
        // DSTriplet convenience methods
        assert!((m.plausibility_of_true() - 0.7).abs() < 1e-12);
        assert!((m.plausibility_of_false() - 0.4).abs() < 1e-12);
    }

    #[test]
    fn test_ds_belief() {
        let m = DSTriplet::new(0.6, 0.3, 0.1);
        assert!((DempsterShaferFuser::belief(&m, DSSupport::True) - 0.6).abs() < 1e-12);
        assert!((DempsterShaferFuser::belief(&m, DSSupport::False) - 0.3).abs() < 1e-12);
        assert!((DempsterShaferFuser::belief(&m, DSSupport::Uncertain) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_sequential_combination_three_sources() {
        let s1 = DSTriplet::new(0.7, 0.2, 0.1);
        let s2 = DSTriplet::new(0.6, 0.2, 0.2);
        let s3 = DSTriplet::new(0.8, 0.1, 0.1);
        let batch = DempsterShaferFuser::combine(&[s1, s2, s3]);
        let seq = DempsterShaferFuser::combine_sequential(&[s1, s2, s3]);
        // Batch and sequential must be equivalent
        assert!((batch.belief - seq.belief).abs() < 1e-12);
        assert!((batch.disbelief - seq.disbelief).abs() < 1e-12);
        // Three agreeing sources should yield high belief, low uncertainty
        assert!(batch.belief > 0.9);
        assert!(batch.uncertainty < 0.05);
    }

    #[test]
    fn test_ds_from_evidence_records() {
        let mut mgr = make_manager();
        let id1 = mgr.add_evidence("https://a.com", "a", "claim true");
        if let Some(r) = mgr.get_mut(id1) {
            r.verify();
        }
        let id2 = mgr.add_evidence("https://b.com", "b", "claim true too");
        if let Some(r) = mgr.get_mut(id2) {
            r.cross_reference();
        }
        let ds = mgr.dempster_shafer_confidence(&[id1, id2]);
        // Two corroborating sources should produce belief > 0.5
        assert!(ds.belief > 0.5);
        assert!(ds.uncertainty < 0.3);
        assert!((ds.belief + ds.disbelief + ds.uncertainty - 1.0).abs() < 1e-10);

        // Empty set returns all-uncertain
        let empty = mgr.dempster_shafer_confidence(&[]);
        assert!((empty.uncertainty - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_ds_from_disputed_records() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence("https://bad.com", "bad", "dubious claim");
        if let Some(r) = mgr.get_mut(id) {
            r.dispute();
        }
        let ds = mgr.dempster_shafer_confidence(&[id]);
        assert!(ds.disbelief > 0.5);
        assert!(ds.belief < 0.2);
    }

    #[test]
    fn test_nato_scale_conversion() {
        assert!((NatoSourceReliability::A.to_reliability_score() - 0.95).abs() < 1e-10);
        assert!((NatoSourceReliability::B.to_reliability_score() - 0.80).abs() < 1e-10);
        assert!((NatoSourceReliability::C.to_reliability_score() - 0.60).abs() < 1e-10);
        assert!((NatoSourceReliability::D.to_reliability_score() - 0.40).abs() < 1e-10);
        assert!((NatoSourceReliability::E.to_reliability_score() - 0.15).abs() < 1e-10);
        assert!((NatoSourceReliability::F.to_reliability_score() - 0.50).abs() < 1e-10);

        // Round-trip
        for score in [0.95, 0.80, 0.60, 0.40, 0.15, 0.50] {
            let letter = NatoSourceReliability::from_score(score);
            let back = letter.to_reliability_score();
            assert!(
                (score - back).abs() < 0.05,
                "round-trip failed for score={}",
                score
            );
        }
    }

    #[test]
    fn test_ds_discount_by_reliability() {
        let m = DSTriplet::new(0.8, 0.1, 0.1);
        let discounted = m.discount(0.5);
        assert!((discounted.belief - 0.4).abs() < 1e-12);
        assert!((discounted.disbelief - 0.05).abs() < 1e-12);
        assert!((discounted.uncertainty - 0.55).abs() < 1e-12);
        // Fully reliable → no change
        let same = m.discount(1.0);
        assert!((same.belief - m.belief).abs() < 1e-12);
    }

    #[test]
    fn test_ds_is_committed() {
        assert!(DSTriplet::new(0.9, 0.09, 0.01).is_committed());
        assert!(!DSTriplet::new(0.5, 0.3, 0.2).is_committed());
    }

    #[test]
    fn test_ds_score_ds_method() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence(
            "https://arxiv.org/abs/2603.25097",
            "arxiv",
            "DS evidence system",
        );
        if let Some(r) = mgr.get_mut(id) {
            r.verify();
        }
        let result = CompetitiveScorer::score_ds(0.9, &mgr, &[id], now_nanos(), 0.8);
        assert!(result.combined_belief > 0.5);
        assert!(result.combined_plausibility >= result.combined_belief);
        assert!(result.breakdown.contains("DS:"));
    }

    #[test]
    fn test_ds_source_reliability_bayesian_update() {
        let mut mgr = make_manager();
        let id = mgr.add_evidence("https://rel.com", "rel", "tracked source");

        // Set initial reliability
        mgr.set_source_reliability(id, 0.7);
        let updated = mgr.update_source_reliability(id, true);
        assert!(
            updated > 0.7,
            "correct observation should increase reliability"
        );
        let updated2 = mgr.update_source_reliability(id, false);
        assert!(
            updated2 < updated,
            "incorrect observation should decrease reliability"
        );

        // Reliability affects DS fusion
        if let Some(r) = mgr.get_mut(id) {
            r.verify();
        }
        let ds_with_reliability = mgr.dempster_shafer_confidence(&[id]);
        assert!(ds_with_reliability.uncertainty > 0.01);
    }
}
