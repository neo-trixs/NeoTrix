#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::nt_core_knowledge::evidence::{
    ClaimSource, EvidenceRecord, EvidenceState, ScoringDimensions,
};

// ── Error type ──

#[derive(Debug, thiserror::Error)]
pub enum ResearchPackageError {
    #[error("Claim not found: {0}")]
    ClaimNotFound(String),
    #[error("Trace node not found: {0}")]
    TraceNodeNotFound(String),
    #[error("Cross-layer binding broken: claim {claim} references missing {target}")]
    BrokenBinding { claim: String, target: String },
    #[error("VSA index mismatch: expected {expected} entries, found {found}")]
    VsaIndexMismatch { expected: usize, found: usize },
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serde(String),
}

pub type Result<T> = std::result::Result<T, ResearchPackageError>;

// ── Manifest ──

/// Metadata header for a .ne-research package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchManifest {
    /// Human-readable name.
    pub name: String,
    /// Semantic version of the research artifact.
    pub version: String,
    /// Free-form description.
    pub description: String,
    /// When the research was conducted.
    pub created_at: u64,
    /// Who/what produced this artifact.
    pub author: String,
    /// Entry claims — the top-level claims this package asserts.
    pub entry_claims: Vec<String>,
    /// Tags for VSA-based discovery.
    pub tags: Vec<String>,
    /// How many trace nodes, claims, experiments, evidence records are in this package.
    pub stats: PackageStats,
}

/// Structural statistics of the package.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageStats {
    pub trace_nodes: usize,
    pub claims: usize,
    pub experiments: usize,
    pub evidence_records: usize,
}

// ── Cross-Layer Binding ──

/// A cross-layer forensic binding: one entity references another by ID.
///
/// This is the key differentiator of .ne-research vs plain Ne packages:
/// every claim, experiment, trace node, and evidence table carries
/// typed references to its related counterparts, forming a fully
/// traversable provenance graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// What kind of entity this binding originates from.
    pub from_type: BindingEntityType,
    /// ID of the origin entity.
    pub from_id: String,
    /// What kind of entity this binding targets.
    pub to_type: BindingEntityType,
    /// ID of the target entity.
    pub to_id: String,
    /// Why the binding exists (e.g., "spawned", "tested_by", "supports", "implements").
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BindingEntityType {
    Claim,
    Experiment,
    TraceNode,
    EvidenceTable,
    CodePath,
}

// ── Claims (unified logic/ layer) ──

/// Falsifiability status of a research claim — ARA-compatible.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FalsifiabilityStatus {
    /// Claim is hypothesized, not yet tested.
    Hypothesized,
    /// Experiment is in progress.
    InProgress,
    /// Claim has been confirmed by evidence.
    Confirmed,
    /// Claim has been refuted by evidence.
    Refuted,
    /// Claim has been superseded by a newer claim.
    Superseded,
}

/// A unified research claim merging evidence::Claim (executable anchor),
/// evidence_inspector::Claim (verification), and falsifiability metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchClaim {
    /// Unique ID within the package (e.g., "claim-001").
    pub id: String,
    /// Human-readable statement of the claim.
    pub statement: String,
    /// Current falsifiability status.
    pub status: FalsifiabilityStatus,
    /// Overall confidence (0.0–1.0), aggregated from evidence.
    pub confidence: f64,
    /// Source of the claim.
    pub source: ClaimSource,
    /// Executable anchor: language + code + expected pattern (for automated verification).
    pub anchor: Option<ExecutableAnchor>,
    /// Which trace nodes spawned this claim.
    pub trace_bindings: Vec<String>,
    /// Which experiments tested this claim.
    pub experiment_bindings: Vec<String>,
    /// Which evidence tables support/refute this claim.
    pub evidence_bindings: Vec<String>,
    /// Which code paths implement this claim.
    pub code_bindings: Vec<String>,
}

/// An executable anchor that allows automated verification of a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableAnchor {
    /// Language of the executable code (e.g., "ne", "rust", "python").
    pub language: String,
    /// The executable code or pseudocode.
    pub code: String,
    /// Expected pattern or output that constitutes confirmation.
    pub expected_pattern: String,
    /// Whether the anchor has been verified.
    pub verified: bool,
}

// ── Trace DAG (trace/ layer) ──

/// Typed node in the research exploration DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceNode {
    pub id: String,
    /// What kind of research event this node represents.
    pub node_type: TraceNodeType,
    /// Human-readable label.
    pub label: String,
    /// Detailed description.
    pub description: String,
    /// When this node was created.
    pub timestamp: u64,
    /// Parent node ID (None for root).
    pub parent_id: Option<String>,
    /// Child node IDs.
    pub child_ids: Vec<String>,
    /// Cross-layer bindings.
    pub bindings: TraceBindings,
    /// Free-form metadata.
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TraceNodeType {
    /// An open research question.
    Question,
    /// A proposed hypothesis.
    Hypothesis,
    /// A decision to take a direction.
    Decision,
    /// An experiment that was run.
    Experiment,
    /// A dead end encountered.
    DeadEnd,
    /// A pivot to a new direction.
    Pivot,
    /// An obtained result.
    Result,
}

/// Cross-layer bindings carried by a trace node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceBindings {
    /// Claims spawned by this node.
    pub spawned_claims: Vec<String>,
    /// Experiments triggered by this node.
    pub triggered_experiments: Vec<String>,
    /// Evidence tables produced by this node.
    pub produced_evidence: Vec<String>,
    /// Why a dead end or pivot occurred.
    pub reason: Option<String>,
}

// ── Evidence Table (evidence/ layer) ──

/// An evidence table — a structured collection of evidence records
/// linked to a specific claim or experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTable {
    /// ID linking to bindings (e.g., "ev-table-001").
    pub id: String,
    /// Which claim this table supports/refutes.
    pub claim_id: String,
    /// Which experiment produced this table.
    pub experiment_id: Option<String>,
    /// The evidence records in this table.
    pub records: Vec<ResearchEvidenceRecord>,
    /// Aggregated confidence across all records.
    pub aggregated_confidence: f64,
}

/// A record in an evidence table — serializable from EvidenceManager records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEvidenceRecord {
    pub id: u64,
    pub source_url: String,
    pub source_name: String,
    pub assertion: String,
    pub quotation: Option<String>,
    pub confidence: f64,
    pub state: EvidenceState,
    pub scoring: Option<ScoringDimensions>,
}

// ── Experiment Plan (logic/experiments/) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub id: String,
    pub claim_id: String,
    pub name: String,
    pub hypothesis: String,
    pub r#type: ExperimentType,
    pub baseline: String,
    pub intervention: String,
    pub metrics: Vec<String>,
    pub status: ExperimentStatus,
    pub evidence_table_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentType {
    Ablation,
    Comparison,
    ParameterSweep,
    Replication,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentStatus {
    Planned,
    Running,
    Completed,
    Failed,
}

// ── VSA Index ──

/// A VSA index entry for a .ne-research package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaIndexEntry {
    /// 64-byte (512-bit) VSA fingerprint of the research package.
    pub fingerprint: Vec<u8>,
    /// Which claims are indexed by this fingerprint.
    pub claim_ids: Vec<String>,
    /// Tags contributing to the semantic fingerprint.
    pub tags: Vec<String>,
}

// ── Root Package ──

/// A .ne-research package — the portable, verifiable, AI-interpretable
/// research artifact format.
///
/// Mirrors ARA's 4-layer structure:
/// - logic/: claims + experiments (what was claimed and tested)
/// - trace/: exploration DAG (how the research unfolded)
/// - evidence/: evidence tables (what was observed)
/// - src/: executable kernel (reproducible implementation)
///
/// Plus cross-layer forensic bindings and a VSA index for discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPackage {
    pub manifest: ResearchManifest,
    pub claims: Vec<ResearchClaim>,
    pub trace_nodes: Vec<TraceNode>,
    pub evidence_tables: Vec<EvidenceTable>,
    pub experiments: Vec<ExperimentRecord>,
    /// Cross-layer bindings — the forensic graph.
    pub bindings: Vec<Binding>,
    /// VSA index for similarity-based discovery.
    pub vsa_index: Vec<VsaIndexEntry>,
}

impl ResearchPackage {
    /// Build a VSA fingerprint for this package from its manifest.
    /// Uses deterministic hashing for reproducibility.
    pub fn compute_fingerprint(&self) -> Vec<u8> {
        let seed: u64 = self
            .manifest
            .name
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let bytes = QuantizedVSA::seeded_random(seed, 64);
        bytes.to_vec()
    }

    /// Validate all cross-layer bindings: every from_id/to_id must exist.
    pub fn validate_bindings(&self) -> Result<()> {
        let claim_ids: std::collections::HashSet<&str> =
            self.claims.iter().map(|c| c.id.as_str()).collect();
        let trace_ids: std::collections::HashSet<&str> =
            self.trace_nodes.iter().map(|t| t.id.as_str()).collect();
        let exp_ids: std::collections::HashSet<&str> =
            self.experiments.iter().map(|e| e.id.as_str()).collect();
        let ev_ids: std::collections::HashSet<&str> =
            self.evidence_tables.iter().map(|e| e.id.as_str()).collect();

        for binding in &self.bindings {
            let exists = match binding.to_type {
                BindingEntityType::Claim => claim_ids.contains(binding.to_id.as_str()),
                BindingEntityType::TraceNode => trace_ids.contains(binding.to_id.as_str()),
                BindingEntityType::Experiment => exp_ids.contains(binding.to_id.as_str()),
                BindingEntityType::EvidenceTable => ev_ids.contains(binding.to_id.as_str()),
                BindingEntityType::CodePath => true, // code paths are not validated here
            };
            if !exists {
                return Err(ResearchPackageError::BrokenBinding {
                    claim: binding.from_id.clone(),
                    target: format!("{:?}:{}", binding.to_type, binding.to_id),
                });
            }
        }
        Ok(())
    }

    /// Verify the VSA index matches the claims in the package.
    pub fn validate_vsa_index(&self) -> Result<()> {
        let all_claim_ids: std::collections::HashSet<&str> =
            self.claims.iter().map(|c| c.id.as_str()).collect();
        let indexed_claim_ids: std::collections::HashSet<&str> = self
            .vsa_index
            .iter()
            .flat_map(|e| e.claim_ids.iter().map(|s| s.as_str()))
            .collect();
        // Every claim must appear in at least one VSA index entry
        for cid in &all_claim_ids {
            if !indexed_claim_ids.contains(cid) {
                return Err(ResearchPackageError::VsaIndexMismatch {
                    expected: all_claim_ids.len(),
                    found: indexed_claim_ids.len(),
                });
            }
        }
        Ok(())
    }

    /// Convert scoring dimensions to metadata kv pairs.
    pub fn scoring_to_metadata(scoring: &ScoringDimensions) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("relevance".into(), format!("{:.4}", scoring.relevance));
        m.insert(
            "evidence_confidence".into(),
            format!("{:.4}", scoring.evidence_confidence),
        );
        m.insert("recency".into(), format!("{:.4}", scoring.recency));
        m.insert(
            "source_authority".into(),
            format!("{:.4}", scoring.source_authority),
        );
        m.insert(
            "cross_references".into(),
            format!("{:.4}", scoring.cross_references),
        );
        m.insert(
            "contradiction_penalty".into(),
            format!("{:.4}", scoring.contradiction_penalty),
        );
        m
    }
}

// ── ResearchPackageManager ──

/// Manages the lifecycle of .ne-research packages: export from existing
/// NeoTrix infrastructure, import and validate, inject into MemoryLattice.
#[derive(Debug, Clone)]
pub struct ResearchPackageManager {
    /// Directory where .ne-research packages are stored.
    pub packages_dir: std::path::PathBuf,
    /// Maximum number of packages to keep.
    pub max_packages: usize,
    /// Number of successful exports (in-memory counter, no IO).
    pub export_count: u64,
}

impl Default for ResearchPackageManager {
    fn default() -> Self {
        Self {
            packages_dir: crate::core::nt_core_util::home_dir()
                .join(".neotrix")
                .join("research-packages"),
            max_packages: 50,
            export_count: 0,
        }
    }
}

impl ResearchPackageManager {
    pub fn new(packages_dir: std::path::PathBuf) -> Self {
        if let Err(e) = std::fs::create_dir_all(&packages_dir) {
            log::warn!("failed to create research packages dir {:?}: {}", packages_dir, e);
        }
        Self {
            packages_dir,
            max_packages: 50,
            export_count: 0,
        }
    }

    /// Export a skill crystallization result as a .ne-research package.
    ///
    /// Wraps the trace archive, evidence records, and Ne source into a
    /// self-describing research artifact with cross-layer bindings.
    pub fn export_skill_crystal(
        &self,
        skill_name: &str,
        skill_description: &str,
        ne_source: &str,
        invocation_count: u64,
        avg_score: f64,
        tags: &[String],
        trace_steps: &[crate::core::nt_core_experience::self_evolution_loop::SelfEvolutionStep],
        evidence_records: &[EvidenceRecord],
        scoring: Option<&ScoringDimensions>,
    ) -> Result<ResearchPackage> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // --- Claims layer ---
        let claim_id = "claim-001".to_string();
        let claim = ResearchClaim {
            id: claim_id.clone(),
            statement: skill_description.to_string(),
            status: FalsifiabilityStatus::Confirmed,
            confidence: avg_score,
            source: ClaimSource::GeneratedContent,
            anchor: Some(ExecutableAnchor {
                language: "ne".to_string(),
                code: ne_source.to_string(),
                expected_pattern: format!("score >= {:.3}", avg_score * 0.8),
                verified: true,
            }),
            trace_bindings: trace_steps.iter().map(|s| format!("trace-{}", s.id)).collect(),
            experiment_bindings: vec!["exp-001".to_string()],
            evidence_bindings: vec!["ev-table-001".to_string()],
            code_bindings: vec!["src/kernel.ne".to_string()],
        };

        // --- Trace DAG layer ---
        let mut trace_nodes = Vec::new();
        let mut bindings = Vec::new();

        for step in trace_steps {
            let node_id = format!("trace-{}", step.id);
            let node_type = if !step.compiles {
                TraceNodeType::DeadEnd
            } else if step.accepted {
                TraceNodeType::Result
            } else {
                TraceNodeType::Decision
            };

            let mut trace_bindings = TraceBindings::default();
            trace_bindings.spawned_claims.push(claim_id.clone());
            trace_bindings.triggered_experiments.push("exp-001".to_string());

            trace_nodes.push(TraceNode {
                id: node_id.clone(),
                node_type,
                label: format!("Mutation step {} (gen {})", step.id, step.generation),
                description: format!(
                    "score: {:.3} → {:?}, compiles: {}, accepted: {}",
                    step.score_before, step.score_after, step.compiles, step.accepted
                ),
                timestamp: step.timestamp,
                parent_id: step
                    .id
                    .checked_sub(1)
                    .map(|p| format!("trace-{}", p)),
                child_ids: vec![],
                bindings: trace_bindings,
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("score_before".into(), format!("{:.4}", step.score_before));
                    if let Some(after) = step.score_after {
                        m.insert("score_after".into(), format!("{:.4}", after));
                    }
                    m.insert("generation".into(), step.generation.to_string());
                    if let Some(cmp) = step.cmp_score {
                        m.insert("cmp_score".into(), format!("{:.4}", cmp));
                    }
                    m
                },
            });
        }

        // Link children (build adjacency from parent_id)
        let node_ids: Vec<String> = trace_nodes.iter().map(|n| n.id.clone()).collect();
        for (i, node) in trace_nodes.iter_mut().enumerate() {
            if let Some(ref parent_id) = node.parent_id {
                if let Some(child) = node_ids.get(i + 1) {
                    node.child_ids.push(child.clone());
                }
                // Add binding from trace node to evidence
                bindings.push(Binding {
                    from_type: BindingEntityType::TraceNode,
                    from_id: node.id.clone(),
                    to_type: BindingEntityType::EvidenceTable,
                    to_id: "ev-table-001".to_string(),
                    relation: "produced".to_string(),
                });
                // Add binding from trace node to parent
                bindings.push(Binding {
                    from_type: BindingEntityType::TraceNode,
                    from_id: node.id.clone(),
                    to_type: BindingEntityType::TraceNode,
                    to_id: parent_id.clone(),
                    relation: "child_of".to_string(),
                });
            }
        }

        // --- Evidence layer ---
        let ev_records: Vec<ResearchEvidenceRecord> = evidence_records
            .iter()
            .map(|r| ResearchEvidenceRecord {
                id: r.id,
                source_url: r.source_url.clone(),
                source_name: r.source_name.clone(),
                assertion: r.assertion.clone(),
                quotation: r.quotation.clone(),
                confidence: r.confidence,
                state: r.state,
                scoring: scoring.cloned(),
            })
            .collect();

        let aggregated_confidence = if ev_records.is_empty() {
            avg_score
        } else {
            ev_records.iter().map(|r| r.confidence).sum::<f64>() / ev_records.len() as f64
        };

        let evidence_table = EvidenceTable {
            id: "ev-table-001".to_string(),
            claim_id: claim_id.clone(),
            experiment_id: Some("exp-001".to_string()),
            records: ev_records,
            aggregated_confidence,
        };

        // --- Experiment layer ---
        let experiment = ExperimentRecord {
            id: "exp-001".to_string(),
            claim_id: claim_id.clone(),
            name: format!("Crystallization of {}", skill_name),
            hypothesis: skill_description.to_string(),
            r#type: ExperimentType::Replication,
            baseline: format!("invocations: {}", invocation_count),
            intervention: "crystallization".to_string(),
            metrics: vec![
                "score".to_string(),
                "invocation_count".to_string(),
            ],
            status: ExperimentStatus::Completed,
            evidence_table_id: Some("ev-table-001".to_string()),
        };

        // --- VSA index ---
        let fingerprint = {
            let seed: u64 = skill_name
                .bytes()
                .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
            let bytes = QuantizedVSA::seeded_random(seed, 64);
            bytes.to_vec()
        };

        let vsa_entry = VsaIndexEntry {
            fingerprint,
            claim_ids: vec![claim_id.clone()],
            tags: tags.to_vec(),
        };

        // --- Bindings ---
        bindings.push(Binding {
            from_type: BindingEntityType::Claim,
            from_id: claim_id.clone(),
            to_type: BindingEntityType::Experiment,
            to_id: "exp-001".to_string(),
            relation: "tested_by".to_string(),
        });
        bindings.push(Binding {
            from_type: BindingEntityType::Claim,
            from_id: claim_id.clone(),
            to_type: BindingEntityType::EvidenceTable,
            to_id: "ev-table-001".to_string(),
            relation: "supported_by".to_string(),
        });
        bindings.push(Binding {
            from_type: BindingEntityType::Claim,
            from_id: claim_id,
            to_type: BindingEntityType::CodePath,
            to_id: "src/kernel.ne".to_string(),
            relation: "implements".to_string(),
        });
        bindings.push(Binding {
            from_type: BindingEntityType::Experiment,
            from_id: "exp-001".to_string(),
            to_type: BindingEntityType::EvidenceTable,
            to_id: "ev-table-001".to_string(),
            relation: "produced".to_string(),
        });

        // --- Manifest ---
        let manifest = ResearchManifest {
            name: skill_name.to_string(),
            version: "0.1.0".to_string(),
            description: format!(
                "Crystallized skill: {} (score={:.3}, invocations={})",
                skill_description, avg_score, invocation_count
            ),
            created_at: now,
            author: "NeoTrix SelfEvolution".to_string(),
            entry_claims: vec!["claim-001".to_string()],
            tags: tags.to_vec(),
            stats: PackageStats {
                trace_nodes: trace_nodes.len(),
                claims: 1,
                experiments: 1,
                evidence_records: evidence_records.len(),
            },
        };

        Ok(ResearchPackage {
            manifest,
            claims: vec![claim],
            trace_nodes,
            evidence_tables: vec![evidence_table],
            experiments: vec![experiment],
            bindings,
            vsa_index: vec![vsa_entry],
        })
    }

    /// Serialize a ResearchPackage to JSON for on-disk storage.
    pub fn to_json(&self, pkg: &ResearchPackage) -> Result<String> {
        serde_json::to_string_pretty(pkg)
            .map_err(|e| ResearchPackageError::Serde(e.to_string()))
    }

    /// Deserialize a ResearchPackage from JSON.
    pub fn from_json(json: &str) -> Result<ResearchPackage> {
        serde_json::from_str(json).map_err(|e| ResearchPackageError::Serde(e.to_string()))
    }

    /// Save a package to disk as `{name}-{version}.ne-research.json`.
    pub fn save(&mut self, pkg: &ResearchPackage) -> Result<std::path::PathBuf> {
        let filename = format!(
            "{}-{}.ne-research.json",
            pkg.manifest.name, pkg.manifest.version
        );
        let path = self.packages_dir.join(&filename);
        let json = self.to_json(pkg)?;
        std::fs::write(&path, &json)
            .map_err(|e| ResearchPackageError::Io(format!("write {}: {}", filename, e)))?;
        self.export_count += 1;
        log::info!("saved .ne-research package: {:?}", path);
        Ok(path)
    }

    /// Load a package from disk.
    pub fn load(&self, name: &str) -> Result<ResearchPackage> {
        let path = self.packages_dir.join(format!("{}.ne-research.json", name));
        let json = std::fs::read_to_string(&path)
            .map_err(|e| ResearchPackageError::Io(format!("read {:?}: {}", path, e)))?;
        ResearchPackageManager::from_json(&json)
    }

    /// List all .ne-research packages on disk.
    pub fn list(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.packages_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".ne-research.json") {
                    names.push(name_str.replace(".ne-research.json", ""));
                }
            }
        }
        names.sort();
        Ok(names)
    }

    /// Validate a complete package: manifest, bindings, VSA index.
    pub fn validate(&self, pkg: &ResearchPackage) -> Result<()> {
        if pkg.manifest.name.is_empty() {
            return Err(ResearchPackageError::Serde(
                "manifest.name must not be empty".to_string(),
            ));
        }
        pkg.validate_bindings()?;
        pkg.validate_vsa_index()?;
        Ok(())
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::self_evolution_loop::{MutationOp, SelfEvolutionStep, MetaStrategy};

    fn sample_step(id: u64, generation: u32, compiles: bool, accepted: bool) -> SelfEvolutionStep {
        SelfEvolutionStep {
            id,
            mutation: MutationOp::TuneParam {
                target: "test.param".to_string(),
                delta: 0.1,
            },
            parent_id: id.saturating_sub(1),
            score_before: 0.5,
            score_after: Some(if accepted { 0.7 } else { 0.4 }),
            compiles,
            accepted,
            timestamp: 1_000_000 + id,
            generation,
            cmp_score: None,
        }
    }

    fn sample_evidence() -> EvidenceRecord {
        EvidenceRecord::new(1, "https://example.com/result", "test", "accuracy improved by 15%")
            .with_confidence(0.85)
            .with_state(EvidenceState::Validated)
    }

    #[test]
    fn test_export_skill_crystal() {
        let mgr = ResearchPackageManager::default();
        let steps = vec![
            sample_step(1, 0, true, true),
            sample_step(2, 0, true, true),
            sample_step(3, 1, false, false),
            sample_step(4, 1, true, true),
        ];
        let ev = sample_evidence();
        let scoring = ScoringDimensions {
            relevance: 0.9,
            evidence_confidence: 0.85,
            recency: 0.7,
            source_authority: 0.6,
            cross_references: 0.5,
            contradiction_penalty: 0.0,
        };
        let tags = vec!["vsa".to_string(), "kernel".to_string()];

        let pkg = mgr
            .export_skill_crystal(
                "test-skill",
                "VSA kernel mutation improves recall",
                "(define (test) (bundle [1 0 1 0] [0 1 0 1]))",
                10,
                0.85,
                &tags,
                &steps,
                &[ev],
                Some(&scoring),
            )
            .expect("export should succeed");

        assert_eq!(pkg.manifest.name, "test-skill");
        assert_eq!(pkg.claims.len(), 1);
        assert_eq!(pkg.trace_nodes.len(), 4);
        assert_eq!(pkg.evidence_tables.len(), 1);
        assert_eq!(pkg.experiments.len(), 1);
        assert_eq!(pkg.bindings.len(), 8);
        assert_eq!(pkg.vsa_index.len(), 1);

        // Trace node type detection
        assert_eq!(pkg.trace_nodes[2].node_type, TraceNodeType::DeadEnd);
        assert_eq!(pkg.trace_nodes[3].node_type, TraceNodeType::Result);

        // Binding validation
        assert!(pkg.validate_bindings().is_ok());

        // VSA index validation
        assert!(pkg.validate_vsa_index().is_ok());
    }

    #[test]
    fn test_validate_broken_binding() {
        let pkg = ResearchPackage {
            manifest: ResearchManifest {
                name: "broken".to_string(),
                version: "0.1.0".to_string(),
                description: "".to_string(),
                created_at: 0,
                author: "test".to_string(),
                entry_claims: vec![],
                tags: vec![],
                stats: PackageStats::default(),
            },
            claims: vec![],
            trace_nodes: vec![],
            evidence_tables: vec![],
            experiments: vec![],
            bindings: vec![Binding {
                from_type: BindingEntityType::Claim,
                from_id: "claim-001".to_string(),
                to_type: BindingEntityType::Claim,
                to_id: "claim-999".to_string(),
                relation: "references".to_string(),
            }],
            vsa_index: vec![],
        };
        assert!(pkg.validate_bindings().is_err());
    }

    #[test]
    fn test_json_roundtrip() {
        let mgr = ResearchPackageManager::default();
        let steps = vec![sample_step(1, 0, true, true)];
        let ev = sample_evidence();

        let pkg = mgr
            .export_skill_crystal(
                "roundtrip-test",
                "test",
                "(define (x) x)",
                5,
                0.9,
                &["test".to_string()],
                &steps,
                &[ev],
                None,
            )
            .expect("export");

        let json = mgr.to_json(&pkg).expect("serialize");
        let pkg2 = ResearchPackageManager::from_json(&json).expect("deserialize");
        assert_eq!(pkg2.manifest.name, "roundtrip-test");
        assert_eq!(pkg2.trace_nodes.len(), 1);
        assert_eq!(pkg2.claims[0].statement, "test");
    }

    #[test]
    fn test_vsa_fingerprint_determinism() {
        let mgr = ResearchPackageManager::default();
        let steps = vec![sample_step(1, 0, true, true)];
        let ev = sample_evidence();

        let pkg = mgr
            .export_skill_crystal(
                "det-test",
                "deterministic",
                "(define (d) d)",
                1,
                0.5,
                &[],
                &steps,
                &[ev],
                None,
            )
            .expect("export");

        let fp1 = pkg.compute_fingerprint();
        let fp2 = pkg.compute_fingerprint();
        assert_eq!(fp1, fp2);
    }
}
