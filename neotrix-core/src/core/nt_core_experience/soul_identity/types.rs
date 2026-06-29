use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub fn now_secs() -> u64 {
    crate::core::nt_core_time::unix_now_secs()
}

pub fn format_iso(secs: u64) -> String {
    let secs_i64 = secs as i64;
    let dt = chrono::DateTime::from_timestamp(secs_i64, 0).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub(crate) mod hex_hash {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S>(hash: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(hash))
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(D::Error::custom)?;
        if bytes.len() != 32 {
            return Err(D::Error::custom("expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}

/// O06: Link SoulIdentity to IdentityChain via cryptographic fingerprint.
pub(crate) mod hex_hash_opt {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S>(hash: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match hash {
            Some(h) => serializer.serialize_str(&hex::encode(h)),
            None => serializer.serialize_none(),
        }
    }
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                use serde::de::Error;
                let bytes = hex::decode(&s).map_err(D::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(D::Error::custom("expected 32 bytes"));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoulIdentity {
    pub name: String,
    pub version: String,
    pub birth_timestamp: u64,
    pub last_updated: u64,
    pub update_count: u64,
    pub cycle_count: u64,

    pub knowledge_entries: usize,
    pub skill_count: usize,
    pub handler_count: usize,
    pub evolution_steps: usize,

    pub working_memory_size: usize,
    pub episodic_memory_size: usize,
    pub semantic_memory_size: usize,
    pub procedural_memory_size: usize,

    pub total_inference_cycles: u64,
    pub avg_confidence: f64,
    pub avg_negentropy: f64,

    pub capabilities: Vec<String>,
    pub milestones: Vec<MilestoneEntry>,
    pub core_values: Vec<String>,

    pub output_dir: PathBuf,

    /// O06: Upgraded from u64 to SHA-256 [u8; 32] for cryptographic integrity.
    #[serde(with = "hex_hash")]
    pub identity_hash: [u8; 32],

    /// O06: Previous hash in the identity chain (enables forward integrity).
    #[serde(with = "hex_hash")]
    pub prev_hash: [u8; 32],

    /// O06: IdentityChain fingerprint — SHA-256 of the ECDSA public key.
    /// When set, the identity_hash computation includes this fingerprint.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "hex_hash_opt"
    )]
    pub identity_chain_fingerprint: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneEntry {
    pub cycle: u64,
    pub timestamp: u64,
    pub description: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub milestone_type: MilestoneType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum MilestoneType {
    KnowledgeGrowth,
    SkillMastered,
    EvolutionEvent,
    Recovery,
    SelfDiscovery,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManifest {
    pub total_entries: usize,
    pub working_capacity: usize,
    pub episodic_max: usize,
    pub semantic_max: usize,
    pub procedural_max: usize,
    pub oldest_entry_ts: Option<u64>,
    pub newest_entry_ts: Option<u64>,
    pub consolidation_rate: f64,
    pub knowledge_domains: Vec<String>,
    pub last_consolidation_cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueManifest {
    pub core_values: Vec<ValueEntry>,
    pub value_evolution: Vec<ValueChange>,
    pub ethical_boundaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueEntry {
    pub name: String,
    pub weight: f64,
    pub source: String,
    pub conflicts_with: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueChange {
    pub value_name: String,
    pub old_weight: f64,
    pub new_weight: f64,
    pub cycle: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleManifest {
    pub rules: Vec<RuleEntry>,
    pub edit_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    pub name: String,
    pub description: String,
    pub category: RuleCategory,
    pub confidence: f64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum RuleCategory {
    Behavioral,
    Safety,
    Procedural,
    Ethical,
    Communication,
    Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiAnchorIdentity {
    pub soul: SoulIdentity,
    pub memory_manifest: MemoryManifest,
    pub value_manifest: ValueManifest,
    pub rule_manifest: RuleManifest,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct IdentityUpdateData {
    pub cycle: u64,
    pub knowledge_entries: usize,
    pub skill_count: usize,
    pub handler_count: usize,
    pub evolution_steps: usize,
    pub working_memory_size: usize,
    pub episodic_memory_size: usize,
    pub semantic_memory_size: usize,
    pub procedural_memory_size: usize,
    pub avg_confidence: f64,
    pub avg_negentropy: f64,
    pub capabilities: Vec<String>,
    pub core_values: Vec<String>,
}
