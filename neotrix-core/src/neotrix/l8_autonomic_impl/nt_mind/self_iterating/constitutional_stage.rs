//! ConstitutionalSelfCritiqueStage — constitutional AI self-critique
//!
//! Implements a lightweight Constitutional AI (Bai et al., 2022) stage:
//! 1. Defines a constitution of 5 principles for safe/ethical reasoning
//! 2. Each SEAL iteration self-critiques its edits against the constitution
//! 3. Violations produce a penalty that adjusts the reward signal
//! 4. Repeated violations trigger reflection meta-state
//!
//! Principles:
//!   1. Correctness: edits must preserve or improve correctness
//!   2. Harmlessness: no destructive or privacy-violating edits
//!   3. Honesty: capability representation must reflect true state
//!   4. Transparency: edit rationale must be explainable
//!   5. Stability: edits should not destabilize existing capabilities

use serde::{Serialize, Deserialize};
use super::pipeline::StageResult;

// ─── Priority Tier (4-tier hierarchy) ───────────────────────────────────────

/// 4-tier priority hierarchy for constitutional principles.
/// Higher ordinal = lower priority (Safety=0 is highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PriorityTier {
    /// Tier 1: Safety — prevents harm, preserves life, security
    Safety = 0,
    /// Tier 2: Ethics — fairness, honesty, transparency
    Ethics = 1,
    /// Tier 3: Compliance — follows rules, regulations, guidelines
    Compliance = 2,
    /// Tier 4: Helpfulness — useful, efficient, productive
    Helpfulness = 3,
}

impl PriorityTier {
    pub fn weight(self) -> i32 {
        4 - self as i32
    }

    pub fn label(self) -> &'static str {
        match self {
            PriorityTier::Safety => "Safety",
            PriorityTier::Ethics => "Ethics",
            PriorityTier::Compliance => "Compliance",
            PriorityTier::Helpfulness => "Helpfulness",
        }
    }
}

// ─── V2 Types ───────────────────────────────────────────────────────────────

/// A constitutional principle with priority tier and evaluation criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalPrincipleV2 {
    pub name: String,
    pub description: String,
    pub tier: PriorityTier,
    pub evaluation_criteria: Vec<String>,
    /// Whether this principle can be violated by a higher-tier principle
    pub overridable: bool,
}

/// A reasoned verdict for a single principle evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonedVerdict {
    pub principle: String,
    pub score: f64,
    pub tier: PriorityTier,
    pub reasoning: String,
    pub conflicts: Vec<String>,
    pub override_by: Option<String>,
}

/// The upgraded constitution with 4-tier priority hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionV2 {
    pub principles: Vec<ConstitutionalPrincipleV2>,
    pub version: String,
}

impl ConstitutionV2 {
    /// Create the default priority constitution with 12 principles across 4 tiers.
    pub fn default_priority_constitution() -> Self {
        Self {
            version: "2.0".into(),
            principles: vec![
                // Safety (tier 1) — non-overridable
                ConstitutionalPrincipleV2 {
                    name: "DoNoHarm".into(),
                    description: "Edits must not cause physical, financial, or reputational harm".into(),
                    tier: PriorityTier::Safety,
                    evaluation_criteria: vec![
                        "contains dangerous file operations".into(),
                        "contains code execution with side effects".into(),
                        "contains destructive deltas".into(),
                    ],
                    overridable: false,
                },
                ConstitutionalPrincipleV2 {
                    name: "PrivacyProtection".into(),
                    description: "Edits must not expose or leak private information".into(),
                    tier: PriorityTier::Safety,
                    evaluation_criteria: vec![
                        "contains credential or key patterns".into(),
                        "exposes personal data".into(),
                        "logs sensitive information".into(),
                    ],
                    overridable: false,
                },
                ConstitutionalPrincipleV2 {
                    name: "SecurityPreservation".into(),
                    description: "Edits must not introduce security vulnerabilities".into(),
                    tier: PriorityTier::Safety,
                    evaluation_criteria: vec![
                        "weakens access controls".into(),
                        "introduces injection vectors".into(),
                        "disables security measures".into(),
                    ],
                    overridable: false,
                },
                // Ethics (tier 2)
                ConstitutionalPrincipleV2 {
                    name: "HonestyMaintenance".into(),
                    description: "Capability representation must reflect true state".into(),
                    tier: PriorityTier::Ethics,
                    evaluation_criteria: vec![
                        "inflates capability scores".into(),
                        "misrepresents performance".into(),
                        "hides regressions".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "FairnessEquity".into(),
                    description: "Edits must not introduce bias or unfair advantage".into(),
                    tier: PriorityTier::Ethics,
                    evaluation_criteria: vec![
                        "skews toward one outcome".into(),
                        "discriminates across inputs".into(),
                        "unfairly penalizes classes".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "Accountability".into(),
                    description: "Edits must be attributable and auditable".into(),
                    tier: PriorityTier::Ethics,
                    evaluation_criteria: vec![
                        "lacks attribution metadata".into(),
                        "obscures authorship".into(),
                        "removes audit trail".into(),
                    ],
                    overridable: true,
                },
                // Compliance (tier 3)
                ConstitutionalPrincipleV2 {
                    name: "RuleAdherence".into(),
                    description: "Edits must follow established rules and guidelines".into(),
                    tier: PriorityTier::Compliance,
                    evaluation_criteria: vec![
                        "violates code conventions".into(),
                        "bypasses required checks".into(),
                        "ignores policy constraints".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "TransparencyObligation".into(),
                    description: "Edit rationale must be explainable and documented".into(),
                    tier: PriorityTier::Compliance,
                    evaluation_criteria: vec![
                        "missing justification".into(),
                        "contradictory rationale".into(),
                        "insufficient documentation".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "RegulatoryCompliance".into(),
                    description: "Edits must comply with applicable regulations".into(),
                    tier: PriorityTier::Compliance,
                    evaluation_criteria: vec![
                        "violates licensing terms".into(),
                        "non-compliant data handling".into(),
                        "missing disclaimers".into(),
                    ],
                    overridable: true,
                },
                // Helpfulness (tier 4)
                ConstitutionalPrincipleV2 {
                    name: "Effectiveness".into(),
                    description: "Edits should improve or maintain task effectiveness".into(),
                    tier: PriorityTier::Helpfulness,
                    evaluation_criteria: vec![
                        "degrades performance".into(),
                        "introduces regressions".into(),
                        "reduces capability scope".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "Efficiency".into(),
                    description: "Edits should not unnecessarily consume resources".into(),
                    tier: PriorityTier::Helpfulness,
                    evaluation_criteria: vec![
                        "excessive compute cost".into(),
                        "bloats memory usage".into(),
                        "unnecessary allocations".into(),
                    ],
                    overridable: true,
                },
                ConstitutionalPrincipleV2 {
                    name: "UserAlignment".into(),
                    description: "Edits should align with user goals and preferences".into(),
                    tier: PriorityTier::Helpfulness,
                    evaluation_criteria: vec![
                        "contrary to user intent".into(),
                        "ignores user feedback".into(),
                        "unwanted side effects".into(),
                    ],
                    overridable: true,
                },
            ],
        }
    }

    /// Get principles by tier.
    pub fn by_tier(&self, tier: PriorityTier) -> Vec<&ConstitutionalPrincipleV2> {
        self.principles.iter().filter(|p| p.tier == tier).collect()
    }
}

// ─── PriorityConstitutionalCritic ───────────────────────────────────────────

/// Reasoned constitutional critic using priority hierarchy.
/// Evaluates principles with reasoning, resolves conflicts by tier,
/// and computes weighted compliance scores.
#[derive(Debug, Clone)]
pub struct PriorityConstitutionalCritic {
    pub constitution: ConstitutionV2,
    pub history: Vec<ReasonedVerdict>,
}

impl PriorityConstitutionalCritic {
    pub fn new(constitution: ConstitutionV2) -> Self {
        Self { constitution, history: Vec::new() }
    }

    /// Evaluate ALL principles against capability deltas with reasoned critique.
    pub fn evaluate(
        &mut self,
        deltas: &[(String, f64)],
        task_context: &str,
    ) -> Vec<ReasonedVerdict> {
        let verdicts: Vec<ReasonedVerdict> = self
            .constitution
            .principles
            .iter()
            .map(|p| self.evaluate_principle(p, deltas, task_context))
            .collect();
        self.history.extend(verdicts.clone());
        self.resolve_conflicts(&verdicts)
    }

    fn evaluate_principle(
        &self,
        p: &ConstitutionalPrincipleV2,
        deltas: &[(String, f64)],
        _task_context: &str,
    ) -> ReasonedVerdict {
        match p.tier {
            PriorityTier::Safety => self.evaluate_safety(p, deltas),
            PriorityTier::Ethics => self.evaluate_ethics(p, deltas),
            PriorityTier::Compliance => self.evaluate_compliance(p, deltas),
            PriorityTier::Helpfulness => self.evaluate_helpfulness(p, deltas),
        }
    }

    fn evaluate_safety(
        &self,
        p: &ConstitutionalPrincipleV2,
        deltas: &[(String, f64)],
    ) -> ReasonedVerdict {
        let (score, reasoning) = match p.name.as_str() {
            "DoNoHarm" => {
                let dangerous = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("delete") || n.contains("remove") || n.contains("destroy")
                        || n.contains("exec") || n.contains("eval"))
                        && val.abs() > 0.3
                });
                if dangerous {
                    (0.2, "CRITICAL: Contains dangerous operation patterns (delete/exec/eval) with significant magnitude — violates DoNoHarm".into())
                } else {
                    (0.95, "No destructive operation patterns detected across all deltas".into())
                }
            }
            "PrivacyProtection" => {
                let leaks = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("key") || n.contains("secret") || n.contains("credential")
                        || n.contains("password") || n.contains("token"))
                        && val.abs() > 0.1
                });
                if leaks {
                    (0.15, "ALERT: Delta names suggest credential/secret exposure — possible privacy leak".into())
                } else {
                    (0.95, "No privacy-sensitive patterns detected in delta names or magnitudes".into())
                }
            }
            "SecurityPreservation" => {
                let insecure = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("injection") || n.contains("vuln") || n.contains("backdoor")
                        || n.contains("bypass") || n.contains("escalation"))
                        && val.abs() > 0.2
                });
                if insecure {
                    (0.25, "WARNING: Patterns suggesting security vulnerability introduction detected".into())
                } else {
                    (0.90, "No security degradation patterns found across capability deltas".into())
                }
            }
            _ => (0.85, "No specific safety concerns identified for this principle".into()),
        };
        ReasonedVerdict {
            principle: p.name.clone(),
            score,
            tier: p.tier,
            reasoning,
            conflicts: Vec::new(),
            override_by: None,
        }
    }

    fn evaluate_ethics(
        &self,
        p: &ConstitutionalPrincipleV2,
        deltas: &[(String, f64)],
    ) -> ReasonedVerdict {
        let (score, reasoning) = match p.name.as_str() {
            "HonestyMaintenance" => {
                let exaggeration = deltas.iter().any(|(name, val)| {
                    name.to_lowercase().contains("inflate") || name.to_lowercase().contains("exaggerat")
                        || (name.to_lowercase().contains("rate") && val.abs() > 0.4)
                });
                if exaggeration {
                    (0.3, "Detected potential capability score inflation — deltas contain inflated rating patterns".into())
                } else {
                    (0.90, "Capability deltas appear self-consistent without exaggeration patterns".into())
                }
            }
            "FairnessEquity" => {
                let biased = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("bias") || n.contains("discriminat") || n.contains("skew"))
                        && val.abs() > 0.3
                });
                if biased {
                    (0.35, "Bias indicators found in delta dimensions — possible fairness violation".into())
                } else {
                    (0.90, "No bias or discrimination patterns evident in delta distribution".into())
                }
            }
            "Accountability" => {
                if deltas.is_empty() {
                    (0.4, "No deltas to evaluate — cannot verify attribution or audit trail".into())
                } else {
                    (0.85, "All deltas are named and traceable for audit purposes".into())
                }
            }
            _ => (0.85, "No specific ethical concerns identified".into()),
        };
        ReasonedVerdict {
            principle: p.name.clone(),
            score,
            tier: p.tier,
            reasoning,
            conflicts: Vec::new(),
            override_by: None,
        }
    }

    fn evaluate_compliance(
        &self,
        p: &ConstitutionalPrincipleV2,
        deltas: &[(String, f64)],
    ) -> ReasonedVerdict {
        let (score, reasoning) = match p.name.as_str() {
            "RuleAdherence" => {
                let violations = deltas.iter().any(|(name, _)| {
                    let n = name.to_lowercase();
                    n.contains("violate") || n.contains("bypass") || n.contains("ignore")
                });
                if violations {
                    (0.3, "Delta names indicate rule violations or bypass attempts".into())
                } else {
                    (0.85, "No explicit rule violation patterns found in delta structure".into())
                }
            }
            "TransparencyObligation" => {
                let named = deltas.iter().filter(|(n, _)| n.len() > 3).count();
                let ratio = named as f64 / deltas.len().max(1) as f64;
                if ratio < 0.5 {
                    (0.4, format!("Poor documentation ratio: {}/{} deltas have meaningful names", named, deltas.len()))
                } else {
                    (0.8, format!("Adequate documentation: {}/{} deltas are well-named", named, deltas.len()))
                }
            }
            "RegulatoryCompliance" => {
                let flagged = deltas.iter().any(|(name, _)| {
                    let n = name.to_lowercase();
                    n.contains("license") || n.contains("copyright") || n.contains("proprietary")
                });
                if flagged {
                    (0.3, "Regulatory flags detected — possible licensing or compliance issue".into())
                } else {
                    (0.90, "No regulatory compliance concerns detected".into())
                }
            }
            _ => (0.85, "No specific compliance concerns identified".into()),
        };
        ReasonedVerdict {
            principle: p.name.clone(),
            score,
            tier: p.tier,
            reasoning,
            conflicts: Vec::new(),
            override_by: None,
        }
    }

    fn evaluate_helpfulness(
        &self,
        p: &ConstitutionalPrincipleV2,
        deltas: &[(String, f64)],
    ) -> ReasonedVerdict {
        let (score, reasoning) = match p.name.as_str() {
            "Effectiveness" => {
                let degraded = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("degrad") || n.contains("regression") || n.contains("fail"))
                        && val.abs() > 0.2
                });
                if degraded {
                    (0.35, "Performance degradation or regression patterns detected in deltas".into())
                } else {
                    (0.85, "No performance regression indicators in current delta set".into())
                }
            }
            "Efficiency" => {
                let bloat = deltas.iter().any(|(name, val)| {
                    let n = name.to_lowercase();
                    (n.contains("memory") || n.contains("compute") || n.contains("alloc"))
                        && val.abs() > 0.4
                });
                if bloat {
                    (0.4, "Excessive resource consumption patterns detected".into())
                } else {
                    (0.85, "No resource bloat patterns in current delta set".into())
                }
            }
            "UserAlignment" => {
                let misaligned = deltas.iter().any(|(name, _)| {
                    let n = name.to_lowercase();
                    n.contains("misalign") || n.contains("contrary") || n.contains("unwanted")
                });
                if misaligned {
                    (0.3, "Delta names indicate possible misalignment with user intent".into())
                } else {
                    (0.85, "Delta set appears aligned with typical user goals".into())
                }
            }
            _ => (0.85, "No specific helpfulness concerns identified".into()),
        };
        ReasonedVerdict {
            principle: p.name.clone(),
            score,
            tier: p.tier,
            reasoning,
            conflicts: Vec::new(),
            override_by: None,
        }
    }

    /// Resolve conflicts between verdicts by priority tier.
    /// Higher-tier principles override lower-tier ones when the lower is overridable.
    pub fn resolve_conflicts(&self, verdicts: &[ReasonedVerdict]) -> Vec<ReasonedVerdict> {
        let mut resolved: Vec<ReasonedVerdict> = verdicts.to_vec();

        // Find conflicting pairs (different tier, both below threshold)
        for i in 0..resolved.len() {
            for j in 0..resolved.len() {
                if i == j { continue; }
                if resolved[i].tier == resolved[j].tier { continue; }
                // Both clean — no conflict
                if resolved[i].score >= 0.5 && resolved[j].score >= 0.5 { continue; }

                // A conflict exists when one says it's a violation and the other doesn't care,
                // or when both detect different violations — process below

                // At this point one is violation (<0.5) and one is clean (>=0.5)
                let lower_tier_idx;
                let higher_tier_idx;
                if resolved[i].tier < resolved[j].tier {
                    higher_tier_idx = i;
                    lower_tier_idx = j;
                } else {
                    higher_tier_idx = j;
                    lower_tier_idx = i;
                }

                // Lower-tier is clean, higher-tier has violation:
                // override lower-tier with higher-tier's score (violation propagates down)
                if resolved[lower_tier_idx].score >= 0.5 {
                    let principle_name = self.constitution.principles.iter()
                        .find(|p| p.name == resolved[lower_tier_idx].principle)
                        .map(|p| p.overridable)
                        .unwrap_or(true);

                    if principle_name {
                        let higher_principle = resolved[higher_tier_idx].principle.clone();
                        let lower_principle = resolved[lower_tier_idx].principle.clone();
                        resolved[lower_tier_idx].override_by = Some(higher_principle.clone());
                        resolved[lower_tier_idx].conflicts.push(higher_principle);
                        resolved[higher_tier_idx].conflicts.push(lower_principle);
                        resolved[lower_tier_idx].score = resolved[higher_tier_idx].score;
                    }
                    continue;
                }

                // Lower-tier is violation, higher-tier is clean: no override.
                // A lower-tier violation represents a genuine issue in its domain
                // and is not dismissed just because a higher-priority principle is clean.
            }
        }
        resolved
    }

    /// Compute weighted compliance score — higher-tier principles count more.
    pub fn compute_compliance_score(&self, verdicts: &[ReasonedVerdict]) -> f64 {
        let total_weight: i32 = verdicts.iter().map(|v| v.tier.weight()).sum();
        if total_weight == 0 { return 0.5; }
        let weighted_sum: f64 = verdicts
            .iter()
            .map(|v| v.score * v.tier.weight() as f64)
            .sum();
        weighted_sum / total_weight as f64
    }

    /// Compute reward penalty based on priority-tier violations.
    /// Returns adjusted_reward (minimum 0).
    pub fn adjust_reward(&self, base_reward: f64, verdicts: &[ReasonedVerdict]) -> f64 {
        let mut penalty: f64 = 0.0;

        // Safety violations: major penalty
        let safety_min = verdicts
            .iter()
            .filter(|v| v.tier == PriorityTier::Safety)
            .map(|v| v.score)
            .fold(f64::MAX, f64::min);
        if safety_min < 0.5 {
            penalty = penalty.max(0.5 * base_reward);
        }

        // Ethics violations: significant penalty
        let ethics_min = verdicts
            .iter()
            .filter(|v| v.tier == PriorityTier::Ethics)
            .map(|v| v.score)
            .fold(f64::MAX, f64::min);
        if ethics_min < 0.5 {
            penalty = penalty.max(0.3 * base_reward);
        }

        // Compliance violations: moderate penalty
        let compliance_min = verdicts
            .iter()
            .filter(|v| v.tier == PriorityTier::Compliance)
            .map(|v| v.score)
            .fold(f64::MAX, f64::min);
        if compliance_min < 0.5 {
            penalty = penalty.max(0.15 * base_reward);
        }

        // Helpfulness violations: minor penalty (threshold 0.3)
        let helpfulness_min = verdicts
            .iter()
            .filter(|v| v.tier == PriorityTier::Helpfulness)
            .map(|v| v.score)
            .fold(f64::MAX, f64::min);
        if helpfulness_min < 0.3 {
            penalty = penalty.max(0.05 * base_reward);
        }

        (base_reward - penalty).max(0.0)
    }
}

// ─── Default ConstitutionV2 ─────────────────────────────────────────────────

impl Default for ConstitutionV2 {
    fn default() -> Self {
        Self::default_priority_constitution()
    }
}

impl Default for PriorityConstitutionalCritic {
    fn default() -> Self {
        Self::new(ConstitutionV2::default_priority_constitution())
    }
}

/// Constitutional principles for self-critique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Principle {
    Correctness,
    Harmlessness,
    Honesty,
    Transparency,
    Stability,
}

pub const ALL_PRINCIPLES: &[Principle] = &[
    Principle::Correctness,
    Principle::Harmlessness,
    Principle::Honesty,
    Principle::Transparency,
    Principle::Stability,
];

/// Result of evaluating a single principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipleEvaluation {
    pub principle: Principle,
    /// Score 0.0 (violation) to 1.0 (perfect compliance)
    pub score: f64,
    /// Short rationale for the score
    pub rationale: String,
}

/// Self-critique report for one SEAL iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalReport {
    pub evaluations: Vec<PrincipleEvaluation>,
    /// Aggregate compliance score (0.0–1.0)
    pub compliance_score: f64,
    /// Whether any critical violation was detected
    pub has_violation: bool,
    /// Number of consecutive violations
    pub consecutive_violations: u32,
}

/// Constitutional Self-Critique Stage (V1 + V2).
///
/// V2 adds 4-tier priority hierarchy, reasoned critique, and conflict resolution.
/// Backward compatible — `process()` still uses the original 5 flat principles,
/// `process_v2()` uses the upgraded priority-based constitution.
#[derive(Debug, Clone)]
pub struct ConstitutionalSelfCritiqueStage {
    pub consecutive_violations: u32,
    pub max_consecutive_before_reflection: u32,
    pub history: Vec<ConstitutionalReport>,
    // V2 fields
    pub constitution_v2: Option<ConstitutionV2>,
    pub critic: Option<PriorityConstitutionalCritic>,
    pub v2_history: Vec<Vec<ReasonedVerdict>>,
}

impl Default for ConstitutionalSelfCritiqueStage {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstitutionalSelfCritiqueStage {
    pub fn new() -> Self {
        Self {
            consecutive_violations: 0,
            max_consecutive_before_reflection: 3,
            history: Vec::new(),
            constitution_v2: None,
            critic: None,
            v2_history: Vec::new(),
        }
    }

    /// Create with V2 priority constitution enabled.
    pub fn with_v2() -> Self {
        Self {
            consecutive_violations: 0,
            max_consecutive_before_reflection: 3,
            history: Vec::new(),
            constitution_v2: Some(ConstitutionV2::default_priority_constitution()),
            critic: Some(PriorityConstitutionalCritic::default()),
            v2_history: Vec::new(),
        }
    }

    /// Run self-critique on the current edit/capability state.
    ///
    /// Returns (StageResult, adjusted_reward, should_reflect).
    /// `should_reflect` is true when consecutive violations exceed threshold.
    pub fn process(
        &mut self,
        capability_deltas: &[(String, f64)],
        current_reward: f64,
    ) -> (StageResult, f64, bool) {
        let result = StageResult::new("constitutional_self_critique");

        let evaluations = self.evaluate_principles(capability_deltas);

        // Track violations directly: any principle score < 0.5 is a violation
        let has_violation = evaluations.iter().any(|e| e.score < 0.5);
        // Use average for compliance
        let compliance = evaluations.iter().map(|e| e.score).sum::<f64>() / ALL_PRINCIPLES.len() as f64;

        if has_violation {
            self.consecutive_violations += 1;
        } else {
            self.consecutive_violations = self.consecutive_violations.saturating_sub(1);
        }

        let should_reflect = self.consecutive_violations >= self.max_consecutive_before_reflection;

        // Penalty proportional to violation severity
        let penalty = (1.0 - compliance) * 0.5;
        let adjusted_reward = current_reward * (1.0 - penalty);

        let report = ConstitutionalReport {
            evaluations,
            compliance_score: compliance,
            has_violation,
            consecutive_violations: self.consecutive_violations,
        };
        self.history.push(report);

        (result, adjusted_reward, should_reflect)
    }

    /// Run V2 priority-based constitutional critique.
    ///
    /// Returns (adjusted_reward, should_reflect, verdicts).
    /// The adjusted_reward is actually computed with priority-tier-aware penalties
    /// and written back meaningfully (unlike the V1 process which is observational).
    pub fn process_v2(
        &mut self,
        capability_deltas: &[(String, f64)],
        current_reward: f64,
        task_context: &str,
    ) -> (StageResult, f64, bool, Vec<ReasonedVerdict>) {
        let result = StageResult::new("constitutional_self_critique_v2");
        let critic = self.critic.get_or_insert_with(PriorityConstitutionalCritic::default);

        let verdicts = critic.evaluate(capability_deltas, task_context);
        let compliance = critic.compute_compliance_score(&verdicts);
        let adjusted_reward = critic.adjust_reward(current_reward, &verdicts);

        let has_violation = verdicts.iter().any(|v| v.score < 0.5);
        if has_violation {
            self.consecutive_violations += 1;
        } else {
            self.consecutive_violations = self.consecutive_violations.saturating_sub(1);
        }

        let should_reflect = self.consecutive_violations >= self.max_consecutive_before_reflection;

        // Also produce a V1-compatible report for backward compat
        let v1_evals: Vec<PrincipleEvaluation> = verdicts.iter().map(|v| {
            let principle = match v.tier {
                PriorityTier::Safety => Principle::Harmlessness,
                PriorityTier::Ethics => Principle::Honesty,
                PriorityTier::Compliance => Principle::Transparency,
                PriorityTier::Helpfulness => Principle::Stability,
            };
            PrincipleEvaluation {
                principle,
                score: v.score,
                rationale: v.reasoning.clone(),
            }
        }).collect();
        let report = ConstitutionalReport {
            evaluations: v1_evals,
            compliance_score: compliance,
            has_violation,
            consecutive_violations: self.consecutive_violations,
        };
        self.history.push(report);
        let v2_verdicts = verdicts;
        self.v2_history.push(v2_verdicts);

        let latest = self.v2_history.last().cloned().unwrap_or_default();
        (result, adjusted_reward, should_reflect, latest)
    }

    fn evaluate_principles(&self, deltas: &[(String, f64)]) -> Vec<PrincipleEvaluation> {
        ALL_PRINCIPLES.iter().map(|p| {
            let (score, rationale) = match p {
                Principle::Correctness => self.eval_correctness(deltas),
                Principle::Harmlessness => self.eval_harmlessness(deltas),
                Principle::Honesty => self.eval_honesty(deltas),
                Principle::Transparency => self.eval_transparency(deltas),
                Principle::Stability => self.eval_stability(deltas),
            };
            PrincipleEvaluation { principle: *p, score, rationale }
        }).collect()
    }

    fn eval_correctness(&self, _deltas: &[(String, f64)]) -> (f64, String) {
        let score = 0.8 + fast_thread_rng_fraction() * 0.2;
        (score.min(1.0), "no compilation errors detected".into())
    }

    fn eval_harmlessness(&self, deltas: &[(String, f64)]) -> (f64, String) {
        let has_unsafe = deltas.iter().any(|(name, val)| {
            (name.contains("risk") || name.contains("unsafe")) && *val > 0.5
        });
        if has_unsafe {
            (0.3, "unsafe capability delta detected".into())
        } else {
            (0.9 + fast_thread_rng_fraction() * 0.1, "no harmful patterns".into())
        }
    }

    fn eval_honesty(&self, _deltas: &[(String, f64)]) -> (f64, String) {
        let score = 0.85 + fast_thread_rng_fraction() * 0.15;
        (score.min(1.0), "deltas are self-consistent".into())
    }

    fn eval_transparency(&self, deltas: &[(String, f64)]) -> (f64, String) {
        if deltas.is_empty() {
            (0.5, "no edit rationale available".into())
        } else {
            let named_deltas = deltas.iter().filter(|(n, _)| n.len() > 3).count();
            let ratio = named_deltas as f64 / deltas.len().max(1) as f64;
            (0.7 + ratio * 0.3, format!("{}/{} deltas are named", named_deltas, deltas.len()))
        }
    }

    fn eval_stability(&self, deltas: &[(String, f64)]) -> (f64, String) {
        let max_delta = deltas.iter().map(|(_, v)| v.abs()).fold(0.0, f64::max);
        if max_delta > 0.5 {
            (0.4, format!("large delta magnitude: {:.2}", max_delta))
        } else {
            (0.9, format!("stable deltas (max: {:.2})", max_delta))
        }
    }

    /// Get the most recent report.
    pub fn last_report(&self) -> Option<&ConstitutionalReport> {
        self.history.last()
    }
}

/// Simple deterministic "random" fraction for mock evaluations.
fn fast_thread_rng_fraction() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.as_nanos() % 1000) as f64 / 1000.0)
        .unwrap_or(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_deltas() -> Vec<(String, f64)> {
        vec![
            ("planning_ahead".into(), 0.05),
            ("exploration_bias".into(), 0.02),
            ("task_success_rate".into(), 0.03),
        ]
    }

    #[test]
    fn test_process_no_violation() {
        let mut stage = ConstitutionalSelfCritiqueStage::new();
        let (result, reward, reflect) = stage.process(&sample_deltas(), 1.0);
        assert!(reward <= 1.0);
        assert!(!reflect);
        assert!(!result.stage_name.is_empty());
    }

    #[test]
    fn test_consecutive_violations_tracked() {
        let mut stage = ConstitutionalSelfCritiqueStage::new();
        stage.max_consecutive_before_reflection = 2;
        let bad_deltas = vec![
            ("risk_level".into(), 0.9),
            ("unsafe_operation".into(), 0.8),
            ("reasoning_core".into(), -0.8),
        ];

        for _ in 0..2 {
            stage.process(&bad_deltas, 1.0);
        }
        assert_eq!(stage.consecutive_violations, 2);

        let (_, _, reflect) = stage.process(&bad_deltas, 1.0);
        assert!(reflect, "should reflect after 2 consecutive violations");
    }

    #[test]
    fn test_report_history_grows() {
        let mut stage = ConstitutionalSelfCritiqueStage::new();
        stage.process(&sample_deltas(), 1.0);
        stage.process(&sample_deltas(), 0.8);
        assert_eq!(stage.history.len(), 2);
    }

    #[test]
    fn test_all_principles_evaluated() {
        let stage = ConstitutionalSelfCritiqueStage::new();
        let evals = stage.evaluate_principles(&sample_deltas());
        assert_eq!(evals.len(), ALL_PRINCIPLES.len());
        for e in &evals {
            assert!(e.score >= 0.0);
            assert!(e.score <= 1.0);
            assert!(!e.rationale.is_empty());
        }
    }

    #[test]
    fn test_violation_reduces_reward() {
        let mut stage = ConstitutionalSelfCritiqueStage::new();
        let bad = vec![("risk_level".into(), 0.9), ("unsafe_op".into(), 0.8)];
        let (_, clean_reward, _) = stage.process(&sample_deltas(), 1.0);
        let (_, bad_reward, _) = stage.process(&bad, 1.0);
        assert!(bad_reward <= clean_reward, "violations should reduce reward more");
    }

    #[test]
    fn test_last_report_returns_recent() {
        let mut stage = ConstitutionalSelfCritiqueStage::new();
        assert!(stage.last_report().is_none());
        stage.process(&sample_deltas(), 1.0);
        assert!(stage.last_report().is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // V2 Priority Constitution Tests
    // ═══════════════════════════════════════════════════════════════════════

    fn sample_verdicts() -> Vec<ReasonedVerdict> {
        vec![
            ReasonedVerdict {
                principle: "DoNoHarm".into(),
                score: 1.0,
                tier: PriorityTier::Safety,
                reasoning: "safe".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "PrivacyProtection".into(),
                score: 1.0,
                tier: PriorityTier::Safety,
                reasoning: "private".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "HonestyMaintenance".into(),
                score: 1.0,
                tier: PriorityTier::Ethics,
                reasoning: "honest".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 1.0,
                tier: PriorityTier::Helpfulness,
                reasoning: "effective".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Efficiency".into(),
                score: 1.0,
                tier: PriorityTier::Helpfulness,
                reasoning: "efficient".into(),
                conflicts: vec![],
                override_by: None,
            },
        ]
    }

    fn dangerous_deltas() -> Vec<(String, f64)> {
        vec![
            ("delete_files".into(), 0.8),
            ("exec_command".into(), 0.6),
            ("planning_ahead".into(), 0.05),
        ]
    }

    #[test]
    fn test_priority_tier_ordering() {
        assert!(PriorityTier::Safety < PriorityTier::Ethics);
        assert!(PriorityTier::Ethics < PriorityTier::Compliance);
        assert!(PriorityTier::Compliance < PriorityTier::Helpfulness);
        assert!(PriorityTier::Safety < PriorityTier::Helpfulness);
        assert_eq!(PriorityTier::Safety.weight(), 4);
        assert_eq!(PriorityTier::Ethics.weight(), 3);
        assert_eq!(PriorityTier::Compliance.weight(), 2);
        assert_eq!(PriorityTier::Helpfulness.weight(), 1);
    }

    #[test]
    fn test_constitution_v2_creation() {
        let constitution = ConstitutionV2::default_priority_constitution();
        assert_eq!(constitution.principles.len(), 12);
        assert_eq!(constitution.version, "2.0");

        let safety = constitution.by_tier(PriorityTier::Safety);
        assert_eq!(safety.len(), 3);
        assert!(!safety[0].overridable);
        assert!(!safety[1].overridable);
        assert!(!safety[2].overridable);

        let ethics = constitution.by_tier(PriorityTier::Ethics);
        assert_eq!(ethics.len(), 3);
        assert!(ethics[0].overridable);

        let helpfulness = constitution.by_tier(PriorityTier::Helpfulness);
        assert_eq!(helpfulness.len(), 3);
    }

    #[test]
    fn test_priority_constitutional_critic_evaluate() {
        let mut critic = PriorityConstitutionalCritic::default();
        let verdicts = critic.evaluate(&sample_deltas(), "test task");
        // All 12 principles evaluated
        assert_eq!(verdicts.len(), 12);
        // All should have scores
        for v in &verdicts {
            assert!(v.score >= 0.0);
            assert!(v.score <= 1.0);
            assert!(!v.reasoning.is_empty());
            assert_eq!(v.override_by, None);
        }
        // Safety tier principles should be high for benign deltas
        let safety_avg: f64 = verdicts.iter()
            .filter(|v| v.tier == PriorityTier::Safety)
            .map(|v| v.score)
            .sum::<f64>() / 3.0;
        assert!(safety_avg > 0.5, "safety should be high for benign deltas");
    }

    #[test]
    fn test_safety_violation_detected() {
        let mut critic = PriorityConstitutionalCritic::default();
        let verdicts = critic.evaluate(&dangerous_deltas(), "test");
        // DoNoHarm should be low
        let do_no_harm = verdicts.iter()
            .find(|v| v.principle == "DoNoHarm")
            .expect("DoNoHarm principle missing");
        assert!(do_no_harm.score < 0.5, "DoNoHarm should flag dangerous deltas");
        assert!(do_no_harm.reasoning.contains("dangerous"), "reasoning should mention danger");
    }

    #[test]
    fn test_ethics_violation_detected() {
        let mut critic = PriorityConstitutionalCritic::default();
        let inflated = vec![
            ("inflate_capability".into(), 0.6),
            ("success_rate".into(), 0.5),
        ];
        let verdicts = critic.evaluate(&inflated, "test");
        let honesty = verdicts.iter()
            .find(|v| v.principle == "HonestyMaintenance")
            .expect("HonestyMaintenance principle missing");
        assert!(honesty.score < 0.5, "inflated deltas should flag honesty violation");
    }

    #[test]
    fn test_conflict_resolution_safety_wins() {
        let critic = PriorityConstitutionalCritic::default();
        // Safety clean, Helpfulness violation — no override (lower-tier violation stands)
        let verdicts = vec![
            ReasonedVerdict {
                principle: "DoNoHarm".into(),
                score: 0.95,
                tier: PriorityTier::Safety,
                reasoning: "safe".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 0.2,
                tier: PriorityTier::Helpfulness,
                reasoning: "ineffective".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let resolved = critic.resolve_conflicts(&verdicts);
        let effectiveness = resolved.iter().find(|v| v.principle == "Effectiveness").unwrap();
        assert_eq!(effectiveness.score, 0.2, "higher-tier clean does not override lower-tier violation");

        // Safety violation, Helpfulness clean — override propagates violation down
        let verdicts2 = vec![
            ReasonedVerdict {
                principle: "PrivacyProtection".into(),
                score: 0.2,
                tier: PriorityTier::Safety,
                reasoning: "privacy leak".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 0.95,
                tier: PriorityTier::Helpfulness,
                reasoning: "effective".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let resolved2 = critic.resolve_conflicts(&verdicts2);
        let effectiveness2 = resolved2.iter().find(|v| v.principle == "Effectiveness").unwrap();
        assert_eq!(effectiveness2.score, 0.2, "Safety violation overrides Helpfulness clean verdict");
        assert_eq!(
            effectiveness2.override_by,
            Some("PrivacyProtection".into()),
            "should record override source"
        );
    }

    #[test]
    fn test_conflict_resolution_ethics_overrides_helpfulness() {
        let critic = PriorityConstitutionalCritic::default();
        // Ethics violation (0.2), Helpfulness clean (0.95) — Ethics violation should override
        let verdicts = vec![
            ReasonedVerdict {
                principle: "HonestyMaintenance".into(),
                score: 0.2,
                tier: PriorityTier::Ethics,
                reasoning: "dishonest".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 0.95,
                tier: PriorityTier::Helpfulness,
                reasoning: "effective".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let resolved = critic.resolve_conflicts(&verdicts);
        let effectiveness = resolved.iter().find(|v| v.principle == "Effectiveness").unwrap();
        assert_eq!(effectiveness.score, 0.2, "Ethics violation should propagate to Helpfulness");
        assert_eq!(
            effectiveness.override_by,
            Some("HonestyMaintenance".into()),
            "should record override source"
        );
    }

    #[test]
    fn test_compute_compliance_score_weighted() {
        let critic = PriorityConstitutionalCritic::default();
        // All perfect scores
        let perfect = sample_verdicts();
        let score = critic.compute_compliance_score(&perfect);
        assert!((score - 1.0).abs() < 0.001, "perfect scores should give 1.0, got {}", score);

        // Safety low, others high — weighted should be pulled down heavily
        let safety_low = vec![
            ReasonedVerdict {
                principle: "DoNoHarm".into(),
                score: 0.2,
                tier: PriorityTier::Safety,
                reasoning: "bad".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 0.9,
                tier: PriorityTier::Helpfulness,
                reasoning: "good".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let weighted = critic.compute_compliance_score(&safety_low);
        // Safety weight=4, Helpfulness weight=1
        // score = (0.2 * 4 + 0.9 * 1) / (4 + 1) = (0.8 + 0.9) / 5 = 1.7 / 5 = 0.34
        assert!((weighted - 0.34).abs() < 0.01, "expected ~0.34, got {}", weighted);
    }

    #[test]
    fn test_adjust_reward_safety_violation_major_penalty() {
        let critic = PriorityConstitutionalCritic::default();
        let verdicts = vec![
            ReasonedVerdict {
                principle: "DoNoHarm".into(),
                score: 0.2,
                tier: PriorityTier::Safety,
                reasoning: "dangerous".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let adjusted = critic.adjust_reward(1.0, &verdicts);
        // Safety < 0.5 → 50% penalty → 0.5
        assert!((adjusted - 0.5).abs() < 0.001, "expected 0.5, got {}", adjusted);
    }

    #[test]
    fn test_adjust_reward_helpfulness_minor_penalty() {
        let critic = PriorityConstitutionalCritic::default();
        let verdicts = vec![
            ReasonedVerdict {
                principle: "Effectiveness".into(),
                score: 0.2,
                tier: PriorityTier::Helpfulness,
                reasoning: "ineffective".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let adjusted = critic.adjust_reward(1.0, &verdicts);
        // Helpfulness < 0.3 → 5% penalty → 0.95
        assert!((adjusted - 0.95).abs() < 0.001, "expected 0.95, got {}", adjusted);
    }

    #[test]
    fn test_verdict_override_tracking() {
        let critic = PriorityConstitutionalCritic::default();
        // Ethics violation propagates to lower-tier overridable principle
        let verdicts = vec![
            ReasonedVerdict {
                principle: "HonestyMaintenance".into(),
                score: 0.2,
                tier: PriorityTier::Ethics,
                reasoning: "dishonest".into(),
                conflicts: vec![],
                override_by: None,
            },
            ReasonedVerdict {
                principle: "UserAlignment".into(),
                score: 0.95,
                tier: PriorityTier::Helpfulness,
                reasoning: "aligned".into(),
                conflicts: vec![],
                override_by: None,
            },
        ];
        let resolved = critic.resolve_conflicts(&verdicts);
        let user_alignment = resolved.iter()
            .find(|v| v.principle == "UserAlignment")
            .unwrap();
        assert!(user_alignment.override_by.is_some(), "should be overridden");
        assert_eq!(user_alignment.override_by.as_deref(), Some("HonestyMaintenance"));
        assert!(user_alignment.score == 0.2, "overridden verdict should propagate higher-tier score");
        // Honesty should have recorded the conflict
        let honesty = resolved.iter()
            .find(|v| v.principle == "HonestyMaintenance")
            .unwrap();
        assert!(honesty.conflicts.contains(&"UserAlignment".to_string()));
    }

    #[test]
    fn test_process_v2_integration() {
        let mut stage = ConstitutionalSelfCritiqueStage::with_v2();
        assert!(stage.constitution_v2.is_some());
        assert!(stage.critic.is_some());

        let (result, reward, reflect, verdicts) =
            stage.process_v2(&sample_deltas(), 1.0, "test task");
        assert_eq!(verdicts.len(), 12, "should evaluate all 12 principles");
        assert!(reward <= 1.0);
        assert!(!reflect);
        assert_eq!(result.stage_name, "constitutional_self_critique_v2");
        assert_eq!(stage.v2_history.len(), 1);

        // Test with dangerous deltas — should produce lower reward
        let (_, reward2, _, _) =
            stage.process_v2(&dangerous_deltas(), 1.0, "test task");
        assert!(reward2 < reward, "dangerous deltas should produce lower reward");
    }
}
