//! L0 Substrate shared types — foundational data types and traits
//! that are shared across layers without creating upward dependencies.
//!
//! Moved from L1/L2 to L0 to enforce the substrate invariant:
//! L0 MUST NOT import from L1 or L2.

use neotrix_types::core::nt_core_cap::CapabilityVector;
use serde::{Deserialize, Deserializer, Serialize};

// ─── ReasoningBankStats (moved from L1 nt_core_bank::stats) ──────────────────

/// Per-provider reasoning bank statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReasoningBankStats {
    pub total_memories: usize,
    pub success_count: usize,
    pub success_rate: f64,
}

// ─── KnowledgeSource (moved from L2 nt_core_knowledge::types) ────────────────

/// A known external knowledge source that can be absorbed into the ReasoningBrain.
///
/// Each variant maps to a real project/tool and provides a CapabilityVector
/// representing its strengths across 23 core dimensions plus extension axes.
/// Inherent methods (`name`, `capability_vector`, `source_weight`) live in
/// `l2_perception::nt_core_knowledge::sources` to avoid L0→L2 coupling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// ⛔⛔ **不要与 `neotrix-types` 的同名 enum 收敛**（2026-10-02 实测）
//
// 两者**同名，但不是同一个 enum**：
// · 交集 51 个变体
// · 本侧独有 **14** 个：全是**认知科学理论** —— ActiveInference /
//   GlobalWorkspaceTheory / IntegratedInformationTheory / PredictiveCoding /
//   JEPAWorldModel / OrchOR / HyperAgents / AttentionSchema …
// · 低层独有 **40** 个：全是**外部工具/项目** —— LangMem / LettaMemory /
//   HindsightMemory / OpenSwe / QwenCode / Maigret / Crush / ClawCode …
// ⇒ 两个集合**语义正交**，不是「一份漂移成两份」。
// ⇒ 收敛会造成：把 40 个工具名灌进认知理论枚举（语义污染），
//   或从低层删掉 14 个理论变体（丢能力）。
// 且低层那 40 个**被重度使用**（`core/nt_core_knowledge/sources.rs` 有 250 处
// `KnowledgeSource::` 引用）⇒ 不是死变体。
//
// ⛔ 正确处置是**改名**（如 `CognitiveTheorySource` vs `ToolSource`），
//   不是收敛。改名涉及 39 个文件，需独立批次裁决。
//
// ⚠️ 本条是实测结论，不是推断 —— 见
//    `docs/architecture/MIRROR-BANK-2026-10-02-NAMECOLLISION.md`。

pub enum KnowledgeSource {
    HeroUI,
    BaseUI,
    ArcUI,
    CortexUI,
    AgenticDS,
    DesignPhilosophy,
    Hyperframes,
    Betterleaks,
    YaoWebsecurity,
    Botasaurus,
    ReactDoctor,
    OpenPencil,
    AiTrader,
    SesameRobot,
    EverOS,
    MattPocockSkills,
    NestedLearning,
    AutonomousGoal,
    AwesomeDesignSkills,
    DeepSeekTui,
    Codebuff,
    OpenClaude,
    Cairn,
    Orca,
    RedRun,
    AutonomousSpeedrunning,
    Synesis,
    MemOS,
    Reflexio,
    Mem0,
    Mnemosyne,
    OriMnemos,
    OPSD,
    AttentionMechanism,
    PatchFile,
    KeyVault,
    SealLoop,
    HashCortxAgents,
    HashCortxSecurity,
    HashCortxSwarm,
    HashCortxFailover,
    HetuLuoshu,
    YijingBinary,
    FivePhasesGauge,
    ThreeCosmologies,
    HuainanziCalendar,
    ZhangHengSeismoscope,
    MawangduiAstronomy,
    ShaoYongCosmology,
    DayanNumber,
    AdamsLaw,
    IntegratedInformationTheory,
    GlobalWorkspaceTheory,
    ActiveInference,
    VSAHyperdim,
    JEPAWorldModel,
    PredictiveCoding,
    OrchOR,
    AttentionSchema,
    SiaHarnessUpdate,
    SiaWeightUpdate,
    SiaFeedbackLoop,
    HyperAgents,
    DialogueExperience,
    ResearchFindings,
}

// ─── AbsorptionRecord (moved from L2 nt_core_knowledge::tracker) ─────────────

/// Records a single absorption event: which source, when, and the applied weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorptionRecord {
    pub source: KnowledgeSource,
    pub timestamp: u64,
    pub weight: f64,
}

// ─── KnowledgeProvider trait (moved from L2 nt_core_knowledge::types) ─────────

/// Trait for objects that can provide domain-specific knowledge with capability vectors.
/// Implemented in L2 for `KnowledgeSource` (keeps real capability vectors in L2).
pub trait KnowledgeProvider {
    fn name(&self) -> &str;
    fn capability_vector(&self) -> CapabilityVector;
    fn source_weight(&self) -> f64;
}

// ─── RichMemoryProvider (simplified — no L1 ReasoningMemory dependency) ──────

/// RichMemoryProvider — simplified memory abstraction.
/// The full `ReasoningMemory` type lives in L1; this trait uses only L0 types.
pub trait RichMemoryProvider: Send + Sync {
    fn store_memory_json(&mut self, key: &str, value: &str) -> bool;
    fn recall_json(&self, query: &str, limit: usize) -> Vec<(String, String)>;
    fn stats(&self) -> ReasoningBankStats;
}

// ─── E8 Transition Matrix types (moved from L2 nt_core_e8) ───────────────────
// Serde compatibility: fixed arrays >32 elements need custom serialization.
// We use FlatCounts (Vec<u64>) and SerdeCompat64 (newtype) wrappers.

/// 64-element serde-compatible wrapper.
#[derive(Debug, Clone)]
pub struct SerdeCompat64(pub [u64; 64]);

impl serde::Serialize for SerdeCompat64 {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0[..].serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerdeCompat64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Vec::<u64>::deserialize(deserializer)?;
        if v.len() != 64 {
            return Err(serde::de::Error::custom("expected exactly 64 elements"));
        }
        let mut arr = [0u64; 64];
        arr.copy_from_slice(&v);
        Ok(SerdeCompat64(arr))
    }
}

/// Flat 64×64 matrix for serde compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatCounts(pub Vec<u64>);

impl FlatCounts {
    pub fn get(&self, i: usize, j: usize) -> u64 {
        self.0[i * 64 + j]
    }
    pub fn add(&mut self, i: usize, j: usize, d: u64) {
        self.0[i * 64 + j] = self.0[i * 64 + j].saturating_add(d);
    }
}

/// 64×64 transition probability matrix for E8 hexagram states.
/// cell[i][j] = empirical probability of transitioning from hexagram i to j.
/// Seeded from discovered Mythos trace patterns and updated continuously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E8TransitionMatrix {
    /// 64×64 transition count matrix (flat: index = i * 64 + j)
    pub counts: FlatCounts,
    /// Total transitions from each source state
    pub row_totals: SerdeCompat64,
    /// Number of times each state was visited
    pub visit_counts: SerdeCompat64,
    /// Sequence of recent transitions (for pattern detection)
    pub recent_transitions: Vec<(u8, u8)>,
    /// Maximum recent transitions to retain
    pub max_recent: usize,
}

impl Default for E8TransitionMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl E8TransitionMatrix {
    pub fn new() -> Self {
        Self {
            counts: FlatCounts(vec![0u64; 4096]),
            row_totals: SerdeCompat64([0u64; 64]),
            visit_counts: SerdeCompat64([0u64; 64]),
            recent_transitions: Vec::with_capacity(256),
            max_recent: 256,
        }
    }
}

// ─── SearchResultTrait (canonical interface for all SearchResult types) ────────

/// Common interface for all `SearchResult` types across the codebase.
///
/// Each module defines its own `SearchResult` struct with domain-specific fields,
/// but all implement this trait to provide a uniform access pattern for:
/// - relevance score (normalised to `f64`)
/// - primary content / identifier
/// - origin source
pub trait SearchResultTrait {
    /// Relevance score. Normalised to `[0.0, 1.0]` where higher = more relevant.
    /// For collection types (e.g. `MediaSearchResult`) returns `0.0`.
    fn search_score(&self) -> f64;

    /// Primary content or identifier of this result.
    /// Could be a title, chunk text, symbol name, or skill name.
    fn search_content(&self) -> &str;

    /// Origin source — URL, file path, engine name, or data source.
    fn search_source(&self) -> &str;
}

/// Canonical memory entry -- the single source of truth for memory representations.
/// Other modules should re-export or adapt this type rather than defining their own.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub importance: f64,
    pub embedding: Vec<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl MemoryEntry {
    pub fn new(id: &str, content: &str) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            importance: 0.5,
            embedding: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reasoning_bank_stats_default() {
        let s = ReasoningBankStats::default();
        assert_eq!(s.total_memories, 0);
        assert_eq!(s.success_rate, 0.0);
    }

    #[test]
    fn knowledge_source_serde_roundtrip() {
        let src = KnowledgeSource::HeroUI;
        let json = serde_json::to_string(&src).unwrap();
        let back: KnowledgeSource = serde_json::from_str(&json).unwrap();
        assert_eq!(src, back);
    }

    #[test]
    fn absorption_record_serde() {
        let rec = AbsorptionRecord {
            source: KnowledgeSource::MemOS,
            timestamp: 12345,
            weight: 0.8,
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: AbsorptionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source, KnowledgeSource::MemOS);
        assert_eq!(back.weight, 0.8);
    }

    #[test]
    fn flat_counts_get_and_add() {
        let mut fc = FlatCounts(vec![0u64; 4096]);
        assert_eq!(fc.get(0, 1), 0);
        fc.add(0, 1, 5);
        assert_eq!(fc.get(0, 1), 5);
        fc.add(0, 1, 3);
        assert_eq!(fc.get(0, 1), 8);
    }

    #[test]
    fn flat_counts_saturating_add() {
        let mut fc = FlatCounts(vec![u64::MAX; 4096]);
        fc.add(0, 0, 1); // should not panic
        assert_eq!(fc.get(0, 0), u64::MAX);
    }

    #[test]
    fn e8_transition_matrix_default() {
        let m = E8TransitionMatrix::default();
        assert_eq!(m.counts.0.len(), 4096);
        assert_eq!(m.row_totals.0.len(), 64);
        assert_eq!(m.visit_counts.0.len(), 64);
        assert!(m.recent_transitions.is_empty());
    }

    #[test]
    fn e8_transition_matrix_serde_roundtrip() {
        let m = E8TransitionMatrix::new();
        let json = serde_json::to_string(&m).unwrap();
        let back: E8TransitionMatrix = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_recent, 256);
    }

    #[test]
    fn serde_compat64_roundtrip() {
        let mut arr = [0u64; 64];
        arr[0] = 42;
        arr[63] = 99;
        let s = SerdeCompat64(arr);
        let json = serde_json::to_string(&s).unwrap();
        let back: SerdeCompat64 = serde_json::from_str(&json).unwrap();
        assert_eq!(back.0[0], 42);
        assert_eq!(back.0[63], 99);
    }

    #[test]
    fn memory_entry_new() {
        let me = MemoryEntry::new("id1", "content1");
        assert_eq!(me.id, "id1");
        assert_eq!(me.content, "content1");
        assert_eq!(me.importance, 0.5);
        assert!(me.embedding.is_empty());
    }

    #[test]
    fn memory_entry_serde_roundtrip() {
        let me = MemoryEntry::new("k", "v");
        let json = serde_json::to_string(&me).unwrap();
        let back: MemoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "k");
    }
}
