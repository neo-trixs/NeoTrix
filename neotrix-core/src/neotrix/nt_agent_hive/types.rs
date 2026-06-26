use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unique identifier for a sub-hive.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HiveId(u64);

impl HiveId {
    pub fn new(id: u64) -> Self {
        HiveId(id)
    }

    pub fn random() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        HiveId((nanos & 0xFFFF_FFFF_FFFF_FFFF) as u64)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for HiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hive-{:016x}", self.0)
    }
}

/// Content-hash based packet identifier (SHA-256).
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PacketId([u8; 32]);

impl PacketId {
    pub fn compute(domain: &str, text: &str, negentropy: f64) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(domain.as_bytes());
        hasher.update(b"|");
        hasher.update(text.as_bytes());
        hasher.update(b"|");
        hasher.update(negentropy.to_le_bytes());
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result);
        PacketId(id)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut id = [0u8; 32];
        let len = bytes.len().min(32);
        id[..len].copy_from_slice(&bytes[..len]);
        PacketId(id)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// The smallest unit of knowledge exchanged between sub-hives.
/// Each packet represents a delta — what a sub-hive learned since last publish.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgePacket {
    pub packet_id: PacketId,
    pub sub_hive_id: HiveId,
    pub domain: String,
    pub timestamp_ns: u64,
    pub capability_delta: String,
    pub vsa_vectors: Vec<Vec<u8>>,
    pub text_summary: String,
    pub local_negentropy_gain: f64,
    pub local_validation_count: u32,
    pub local_confidence: f64,
    pub provenance: Vec<PacketId>,
    pub lineage_depth: u32,
}

impl KnowledgePacket {
    pub fn new(
        sub_hive_id: HiveId,
        domain: &str,
        capability_delta: &str,
        text_summary: &str,
        negentropy_gain: f64,
    ) -> Self {
        let timestamp_ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let packet_id = PacketId::compute(domain, text_summary, negentropy_gain);
        KnowledgePacket {
            packet_id,
            sub_hive_id,
            domain: domain.to_string(),
            timestamp_ns,
            capability_delta: capability_delta.to_string(),
            vsa_vectors: Vec::new(),
            text_summary: text_summary.to_string(),
            local_negentropy_gain: negentropy_gain,
            local_validation_count: 0,
            local_confidence: 0.5,
            provenance: Vec::new(),
            lineage_depth: 0,
        }
    }

    pub fn with_provenance(mut self, parents: Vec<PacketId>) -> Self {
        let max_depth = parents.iter().map(|_| 0u32).max().unwrap_or(0);
        self.provenance = parents;
        self.lineage_depth = max_depth + 1;
        self
    }

    pub fn with_vsa(mut self, vectors: Vec<Vec<u8>>) -> Self {
        self.vsa_vectors = vectors;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.local_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn age_ns(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        now.saturating_sub(self.timestamp_ns as u128) as u64
    }
}

/// Specification for spawning a new sub-hive.
#[derive(Debug, Clone)]
pub struct SubHiveSpec {
    pub instruction: String,
    pub context: Vec<String>,
    pub tools: Vec<String>,
    pub model: String,
    pub resource_budget: ResourceBudget,
    pub permissions: PermissionSet,
    pub domain: String,
}

impl SubHiveSpec {
    pub fn new(instruction: &str, domain: &str) -> Self {
        SubHiveSpec {
            instruction: instruction.to_string(),
            context: Vec::new(),
            tools: Vec::new(),
            model: "default".to_string(),
            resource_budget: ResourceBudget::default(),
            permissions: PermissionSet::default(),
            domain: domain.to_string(),
        }
    }

    pub fn with_context(mut self, ctx: Vec<String>) -> Self {
        self.context = ctx;
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_budget(mut self, max_ticks: u64) -> Self {
        self.resource_budget.max_ticks = max_ticks;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ResourceBudget {
    pub max_ticks: u64,
    pub max_memory_bytes: u64,
    pub max_publish_per_tick: usize,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        ResourceBudget {
            max_ticks: 1000,
            max_memory_bytes: 10 * 1024 * 1024,
            max_publish_per_tick: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub allowed_tools: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub auto_absorb: bool,
}

impl Default for PermissionSet {
    fn default() -> Self {
        PermissionSet {
            allowed_tools: Vec::new(),
            allowed_domains: vec!["*".to_string()],
            auto_absorb: true,
        }
    }
}

/// Subscription filter for subscribing to knowledge from the Pool.
#[derive(Debug, Clone)]
pub struct SubHiveSubscription {
    pub domain_filter: Vec<String>,
    pub min_score: f64,
    pub max_age_ns: u64,
    pub auto_absorb: bool,
}

impl Default for SubHiveSubscription {
    fn default() -> Self {
        SubHiveSubscription {
            domain_filter: vec!["*".to_string()],
            min_score: 0.4,
            max_age_ns: 3_600_000_000_000,
            auto_absorb: true,
        }
    }
}

impl SubHiveSubscription {
    pub fn for_domain(domain: &str) -> Self {
        SubHiveSubscription {
            domain_filter: vec![domain.to_string()],
            min_score: 0.4,
            max_age_ns: 3_600_000_000_000,
            auto_absorb: true,
        }
    }

    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = score;
        self
    }
}

/// Status of a sub-hive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubHiveStatus {
    Idle,
    Busy,
    Learning,
    Stalled,
    Dying,
    Dead,
}

/// Metadata about a packet stored in the pool.
#[derive(Debug, Clone)]
pub struct StoredPacket {
    pub packet: KnowledgePacket,
    pub score: f64,
    pub received_at_ns: u64,
    pub hit_count: u64,
    /// Whether SVAF gate accepted at least one field (content-driven convergence).
    pub sva_accepted: Option<bool>,
    /// Weighted SVAF score across all CAT7 fields.
    pub sva_weighted_score: Option<f64>,
}

/// Aggregate statistics for the Knowledge Pool.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_packets: usize,
    pub total_published: u64,
    pub total_evicted: u64,
    pub total_subscribes: u64,
    pub by_domain: std::collections::HashMap<String, usize>,
    pub avg_score: f64,
    pub avg_negentropy_gain: f64,
    /// Fraction of stored packets that passed SVAF acceptance.
    pub avg_svaf_accepted: f64,
}
