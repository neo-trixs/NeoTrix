use std::fmt;
use serde::{Deserialize, Serialize};

pub const VSA_DIM: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VsaOrigin {
    Self_(Thought),
    World(Sensory),
    Bridge(Domain),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Thought {
    Intention, Decision, Reflection, Plan, Memory
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Sensory {
    UserInput, NetworkEvent, PriceTick, SocialFeed, VisionFrame, PageContent
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Crypto, Earn, Network, Crawl, Social, Browse, Vision, System
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Crypto => "crypto",
            Domain::Earn => "earn",
            Domain::Network => "network",
            Domain::Crawl => "crawl",
            Domain::Social => "social",
            Domain::Browse => "browse",
            Domain::Vision => "vision",
            Domain::System => "system",
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsaTagged {
    pub vector: Vec<u8>,
    pub origin: VsaOrigin,
    pub timestamp_ms: i64,
    pub negentropy_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentionVsa {
    pub domain: Domain,
    pub action: String,
    pub parameters: serde_json::Value,
    pub confidence: f64,
    pub urgency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEffect {
    pub domain: Domain,
    pub description: String,
    pub success: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraceMode {
    FallbackDefault,
    SkipSilently,
    BlockAndWarn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriositySignal {
    pub domain: Domain,
    pub query: String,
    pub novelty_estimate: f64,
    pub potential_negentropy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHealth {
    pub domain: Domain,
    pub available: bool,
    pub last_seen_ms: i64,
    pub error_count: u64,
    pub total_actuations: u64,
}

pub trait ConsciousnessAbility: Send + Sync {
    fn domain(&self) -> Domain;

    fn sense(&mut self) -> Vec<VsaTagged>;

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String>;

    fn curiosity_signals(&self) -> Vec<CuriositySignal>;

    fn grace_mode(&self) -> GraceMode;

    fn health(&self) -> BridgeHealth;

    fn probe_available(&self) -> bool;

    fn negentropy_estimate(&self) -> f64;
}

#[derive(Debug)]
pub struct VsaLight {
    pub dim: usize,
}

impl VsaLight {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    pub fn seeded_vector(&self, seed: u64) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        seed.hash(&mut h);
        let base = h.finish();
        let mut v = vec![0u8; self.dim];
        for (i, byte) in v.iter_mut().enumerate() {
            let mut h2 = DefaultHasher::new();
            base.hash(&mut h2);
            (i as u64).hash(&mut h2);
            seed.hash(&mut h2);
            *byte = (h2.finish() & 0xFF) as u8;
        }
        v
    }

    pub fn cosine_similarity(a: &[u8], b: &[u8]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: u64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as u64) * (*y as u64)).sum();
        let na: f64 = (a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
        let nb: f64 = (b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
        if na < 1e-10 || nb < 1e-10 { 0.0 } else { dot as f64 / (na * nb) }
    }

    pub fn novelty(&self, known: &[Vec<u8>], candidate: &[u8], threshold: f64) -> f64 {
        if known.is_empty() {
            return 1.0;
        }
        let max_sim = known.iter()
            .map(|k| Self::cosine_similarity(k, candidate))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        if max_sim > threshold { 0.0 } else { 1.0 - max_sim }
    }
}
