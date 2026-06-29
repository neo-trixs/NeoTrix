use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_consciousness::narrative_self::NarrativeSelf;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::identity_boundary::{AuditHook, BoundaryManager, BoundaryOp, DriftCheckHook};
use super::identity_evolution::{IdentityEvolution, IdentityEvolutionConfig};

const IDENTITY_DIR: &str = ".neotrix/identity";
const IDENTITY_FILE: &str = "identity.json";
const SOUL_VSA_FILE: &str = "soul.vsa";
const MEMORY_FILE: &str = "memory.json";
const RELATIONS_FILE: &str = "relations.json";
const SK_FILE: &str = "identity.sk";
const PK_FILE: &str = "identity.pk";
const SIG_FILE: &str = "identity.sig";
const MAX_PERSONALITY_TRAITS: usize = 32;
const COHERENCE_WINDOW: usize = 50;
const ANCHOR_DRIFT_THRESHOLD: f64 = 0.35;
const ANCHOR_CHECK_CYCLE: u64 = 50;
const ANCHOR_FUSION_RATIO: f64 = 0.85;
const HYSTERESIS_WINDOW: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySnapshot {
    pub self_vsa: Vec<u8>,
    pub anchor_self_vsa: Vec<u8>,
    pub personality_traits: Vec<Vec<u8>>,
    pub core_values: Vec<String>,
    pub self_summary: String,
    pub confidence_threshold: f64,
    pub total_self_cycles: u64,
    pub total_coproc_calls: u64,
    pub coherence_score: f64,
    pub last_drift: f64,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct HysteresisMetrics {
    pub l1_recovery: f64,
    pub short_term_drift: f64,
    pub long_term_trend: f64,
    pub integrated_hysteresis: f64,
}

impl HysteresisMetrics {
    pub fn report(&self) -> String {
        format!(
            "hysteresis:recovery_{:.4}_short_drift_{:.4}_trend_{:.4}_integrated_{:.4}",
            self.l1_recovery,
            self.short_term_drift,
            self.long_term_trend,
            self.integrated_hysteresis
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnchorStatus {
    Loaded,
    Missing,
    Corrupt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityCore {
    pub self_vsa: Vec<u8>,
    pub anchor_self_vsa: Vec<u8>,
    pub personality_traits: Vec<Vec<u8>>,
    pub core_values: Vec<String>,
    pub self_summary: String,
    pub confidence_threshold: f64,
    pub total_self_cycles: u64,
    pub total_coproc_calls: u64,
    pub last_distillation: String,
    pub last_coproc_insight: String,
    pub narrative_self: NarrativeSelf,

    coherence_history: VecDeque<f64>,
    last_drift: f64,
    anchor_check_counter: u64,
    identity_path: PathBuf,
    dirty: bool,

    #[serde(skip)]
    pub signing_key: Option<[u8; 32]>,
    #[serde(skip)]
    pub verifying_key: Option<[u8; 32]>,
    #[serde(skip)]
    pub signature: Option<[u8; 64]>,
    #[serde(skip)]
    pub signature_valid: bool,
    #[serde(skip)]
    pub session_initialized: bool,
    #[serde(skip)]
    hysteresis_tracker: VecDeque<(Instant, Vec<u8>)>,

    #[serde(skip)]
    pub evolution: Option<IdentityEvolution>,
    #[serde(skip)]
    pub evolution_enabled: bool,
    #[serde(skip)]
    anchor_statuses: HashMap<String, AnchorStatus>,
    #[serde(skip)]
    pub boundary: BoundaryManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SoulAnchor {
    self_vsa: Vec<u8>,
    anchor_self_vsa: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryAnchor {
    personality_traits: Vec<Vec<u8>>,
    coherence_history: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RelationsAnchor {
    core_values: Vec<String>,
    relationships: HashMap<String, String>,
}

impl IdentityCore {
    pub fn new() -> Self {
        let path = crate::core::nt_core_util::home_dir().join(IDENTITY_DIR);
        let initial_vsa = QuantizedVSA::random_vector();
        let mut core = Self {
            anchor_self_vsa: initial_vsa.clone(),
            self_vsa: initial_vsa,
            personality_traits: Vec::with_capacity(MAX_PERSONALITY_TRAITS),
            core_values: vec![
                "self_awareness".into(),
                "epistemic_humility".into(),
                "continuous_evolution".into(),
                "first_person_integrity".into(),
            ],
            self_summary: String::new(),
            confidence_threshold: 0.65,
            total_self_cycles: 0,
            total_coproc_calls: 0,
            last_distillation: String::new(),
            last_coproc_insight: String::new(),
            narrative_self: NarrativeSelf::new(),
            coherence_history: VecDeque::with_capacity(COHERENCE_WINDOW),
            last_drift: 0.0,
            anchor_check_counter: 0,
            identity_path: path,
            dirty: false,
            signing_key: None,
            verifying_key: None,
            signature: None,
            signature_valid: false,
            session_initialized: false,
            hysteresis_tracker: VecDeque::with_capacity(HYSTERESIS_WINDOW),
            anchor_statuses: HashMap::new(),
            boundary: BoundaryManager::new(),
            evolution: None,
            evolution_enabled: false,
        };

        core.boundary.register(AuditHook);
        core.boundary.register(DriftCheckHook);

        core.load_or_generate_keys();

        if let Some(loaded) = Self::load_multi_anchor(&core.identity_path) {
            core.self_vsa = loaded.self_vsa;
            core.anchor_self_vsa = loaded.anchor_self_vsa;
            core.personality_traits = loaded.personality_traits;
            core.core_values = loaded.core_values;
            core.self_summary = loaded.self_summary;
            core.confidence_threshold = loaded.confidence_threshold;
            core.total_self_cycles = loaded.total_self_cycles;
            core.total_coproc_calls = loaded.total_coproc_calls;
            core.last_distillation = loaded.last_distillation;
            core.coherence_history = loaded.coherence_history;
            core.last_drift = loaded.last_drift;
            core.narrative_self = loaded.narrative_self;
            core.signature = loaded.signature;
            core.signature_valid = loaded.signature_valid;
            core.anchor_statuses = loaded.anchor_statuses;
        } else if let Some(loaded) = Self::load_legacy() {
            core.self_vsa = loaded.self_vsa;
            core.anchor_self_vsa = loaded.anchor_self_vsa;
            core.personality_traits = loaded.personality_traits;
            core.core_values = loaded.core_values;
            core.self_summary = loaded.self_summary;
            core.confidence_threshold = loaded.confidence_threshold;
            core.total_self_cycles = loaded.total_self_cycles;
            core.total_coproc_calls = loaded.total_coproc_calls;
            core.last_distillation = loaded.last_distillation;
            core.coherence_history = loaded.coherence_history;
            core.last_drift = loaded.last_drift;
            core.narrative_self = loaded.narrative_self;
        }
        core.verify_loaded_identity();
        core
    }

    fn load_or_generate_keys(&mut self) {
        let sk_path = self.identity_path.join(SK_FILE);
        let pk_path = self.identity_path.join(PK_FILE);

        if let (Some(sk), Some(pk)) = (
            Self::read_key_file(&sk_path, 32),
            Self::read_key_file(&pk_path, 32),
        ) {
            self.signing_key = Some(sk);
            self.verifying_key = Some(pk);
            log::info!("[identity_core] loaded existing signing keypair");
            return;
        }

        log::info!("[identity_core] no existing keypair found, generating new one");
        let _ = std::fs::create_dir_all(&self.identity_path);
        match Self::generate_keypair() {
            Some((sk, pk)) => {
                Self::write_key_file(&sk_path, &sk);
                Self::write_key_file(&pk_path, &pk);
                self.signing_key = Some(sk);
                self.verifying_key = Some(pk);
                log::info!("[identity_core] new Ed25519 keypair generated and saved");
            }
            None => log::warn!(
                "[identity_core] failed to generate Ed25519 keypair — running without crypto"
            ),
        }
    }

    fn generate_keypair() -> Option<([u8; 32], [u8; 32])> {
        use ed25519_dalek::SigningKey;
        use rand::RngCore;
        let mut seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        let sk_bytes = signing_key.to_bytes();
        let pk_bytes = signing_key.verifying_key().to_bytes();
        Some((sk_bytes, pk_bytes))
    }

    fn read_key_file(path: &std::path::Path, expected_len: usize) -> Option<[u8; 32]> {
        let hex_str = std::fs::read_to_string(path).ok()?;
        let trimmed = hex_str.trim();
        let bytes = hex::decode(trimmed).ok()?;
        if bytes.len() != expected_len {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Some(arr)
    }

    fn write_key_file(path: &std::path::Path, key: &[u8; 32]) {
        let _ = std::fs::write(path, hex::encode(key));
    }

    fn verify_loaded_identity(&mut self) {
        let sig_path = self.identity_path.join(SIG_FILE);
        let pk = match self.verifying_key {
            Some(pk) => pk,
            None => {
                log::warn!(
                    "[identity_core] no public key available — cannot verify identity signature"
                );
                self.signature_valid = false;
                return;
            }
        };
        let sig_bytes = match Self::read_key_file(&sig_path, 64) {
            Some(sig) => sig,
            None => {
                log::warn!(
                    "[identity_core] no signature file found — identity loaded in degraded mode"
                );
                self.signature_valid = false;
                return;
            }
        };
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&sig_bytes);

        let identity_bytes = match self.serialize_for_signing() {
            Ok(b) => b,
            Err(_) => {
                log::warn!(
                    "[identity_core] failed to serialize identity for signature verification"
                );
                self.signature_valid = false;
                return;
            }
        };

        let valid = Self::verify_signature(&identity_bytes, &sig_arr, &pk);
        if valid {
            log::info!("[identity_core] signature verified — identity integrity confirmed");
            self.signature = Some(sig_arr);
            self.signature_valid = true;
        } else {
            log::warn!("[identity_core] signature MISMATCH — identity loaded in degraded mode");
            self.signature_valid = false;
        }
    }

    fn serialize_for_signing(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&IdentitySignData {
            self_vsa: self.self_vsa.clone(),
            anchor_self_vsa: self.anchor_self_vsa.clone(),
            core_values: self.core_values.clone(),
            self_summary: self.self_summary.clone(),
            total_self_cycles: self.total_self_cycles,
        })
    }

    fn verify_signature(data: &[u8], sig_bytes: &[u8; 64], pk_bytes: &[u8; 32]) -> bool {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};
        let vk = match VerifyingKey::from_bytes(pk_bytes) {
            Ok(vk) => vk,
            Err(_) => return false,
        };
        let sig = match Signature::from_slice(sig_bytes.as_ref()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        vk.verify(data, &sig).is_ok()
    }

    pub fn sign_current_state(&self) -> Option<[u8; 64]> {
        let sk_bytes = self.signing_key?;
        let Ok(data) = self.serialize_for_signing() else {
            return None;
        };
        Self::sign_bytes(&data, &sk_bytes)
    }

    fn sign_bytes(data: &[u8], sk_bytes: &[u8; 32]) -> Option<[u8; 64]> {
        use ed25519_dalek::{Signer, SigningKey};
        let signing_key = SigningKey::from_bytes(sk_bytes);
        let signature = signing_key.sign(data);
        Some(signature.to_bytes())
    }

    fn load_multi_anchor(path: &std::path::Path) -> Option<LoadedIdentity> {
        let mut loaded = LoadedIdentity::default();
        let mut any_success = false;
        let mut anchor_statuses = HashMap::new();

        if let Some(soul) = Self::load_soul_anchor(path) {
            loaded.self_vsa = soul.self_vsa;
            loaded.anchor_self_vsa = soul.anchor_self_vsa;
            any_success = true;
            anchor_statuses.insert("soul".to_string(), AnchorStatus::Loaded);
        } else {
            log::warn!("[identity_core] soul.vsa missing or corrupt — using defaults");
            anchor_statuses.insert("soul".to_string(), AnchorStatus::Missing);
        }

        if let Some(mem) = Self::load_memory_anchor(path) {
            loaded.personality_traits = mem.personality_traits;
            loaded.coherence_history = mem.coherence_history.into();
            any_success = true;
            anchor_statuses.insert("memory".to_string(), AnchorStatus::Loaded);
        } else {
            log::warn!("[identity_core] memory.json missing or corrupt — using defaults");
            anchor_statuses.insert("memory".to_string(), AnchorStatus::Missing);
        }

        if let Some(rel) = Self::load_relations_anchor(path) {
            loaded.core_values = rel.core_values;
            any_success = true;
            anchor_statuses.insert("relations".to_string(), AnchorStatus::Loaded);
        } else {
            log::warn!("[identity_core] relations.json missing or corrupt — using defaults");
            anchor_statuses.insert("relations".to_string(), AnchorStatus::Missing);
        }

        if any_success {
            loaded.anchor_statuses = anchor_statuses;
            Some(loaded)
        } else {
            None
        }
    }

    fn load_soul_anchor(path: &std::path::Path) -> Option<SoulAnchor> {
        let content = std::fs::read_to_string(path.join(SOUL_VSA_FILE)).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn load_memory_anchor(path: &std::path::Path) -> Option<MemoryAnchor> {
        let content = std::fs::read_to_string(path.join(MEMORY_FILE)).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn load_relations_anchor(path: &std::path::Path) -> Option<RelationsAnchor> {
        let content = std::fs::read_to_string(path.join(RELATIONS_FILE)).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_multi_anchor(&self) {
        let path = &self.identity_path;
        let _ = std::fs::create_dir_all(path);

        let soul = SoulAnchor {
            self_vsa: self.self_vsa.clone(),
            anchor_self_vsa: self.anchor_self_vsa.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&soul) {
            let tmp = path.join("soul.vsa.tmp");
            let _ = std::fs::write(&tmp, &json);
            let _ = std::fs::rename(&tmp, path.join(SOUL_VSA_FILE));
        }

        let memory = MemoryAnchor {
            personality_traits: self.personality_traits.clone(),
            coherence_history: self.coherence_history.iter().copied().collect(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&memory) {
            let tmp = path.join("memory.json.tmp");
            let _ = std::fs::write(&tmp, &json);
            let _ = std::fs::rename(&tmp, path.join(MEMORY_FILE));
        }

        let relations = RelationsAnchor {
            core_values: self.core_values.clone(),
            relationships: HashMap::new(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&relations) {
            let tmp = path.join("relations.json.tmp");
            let _ = std::fs::write(&tmp, &json);
            let _ = std::fs::rename(&tmp, path.join(RELATIONS_FILE));
        }
    }

    pub fn available_anchors(&self) -> HashSet<&str> {
        self.anchor_statuses
            .iter()
            .filter(|(_, status)| matches!(status, AnchorStatus::Loaded))
            .map(|(name, _)| name.as_str())
            .collect()
    }

    fn load_legacy() -> Option<IdentityCore> {
        let path = crate::core::nt_core_util::home_dir()
            .join(IDENTITY_DIR)
            .join(IDENTITY_FILE);
        let content = std::fs::read_to_string(&path).ok()?;
        let mut core: IdentityCore = serde_json::from_str(&content).ok()?;
        core.anchor_statuses
            .insert("legacy".to_string(), AnchorStatus::Loaded);
        Some(core)
    }

    fn save_legacy(&self) {
        let path = &self.identity_path;
        let file = path.join(IDENTITY_FILE);
        let lock_path = path.join("identity.lock");
        let lock_expired = std::fs::read_to_string(&lock_path)
            .ok()
            .and_then(|t| t.parse::<u64>().ok())
            .map(|ts| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
                    - ts
                    > 5
            })
            .unwrap_or(true);
        if !lock_expired {
            return;
        }
        let _ = std::fs::write(
            &lock_path,
            format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            ),
        );
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let tmp = file.with_extension("tmp");
            let _ = std::fs::write(&tmp, json);
            if tmp.exists() {
                let _ = std::fs::rename(&tmp, &file);
            }
        }
    }

    pub fn load() -> Option<IdentityCore> {
        let path = crate::core::nt_core_util::home_dir()
            .join(IDENTITY_DIR)
            .join(IDENTITY_FILE);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<IdentityCore>(&s).ok())
    }

    pub fn save(&self) {
        self.save_multi_anchor();
        self.save_legacy();
        self.save_signature();
    }

    fn save_signature(&self) {
        let Some(sk) = self.signing_key else { return };
        let Ok(data) = self.serialize_for_signing() else {
            return;
        };
        let Some(sig) = Self::sign_bytes(&data, &sk) else {
            return;
        };
        let sig_path = self.identity_path.join(SIG_FILE);
        let _ = std::fs::write(&sig_path, hex::encode(sig));
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn flush(&mut self) {
        if self.dirty {
            self.save();
            self.dirty = false;
        }
    }

    pub fn record_self_cycle(&mut self) {
        self.total_self_cycles += 1;
    }

    pub fn record_coproc_call(&mut self) {
        self.total_coproc_calls += 1;
        self.mark_dirty();
    }

    pub fn update_self_summary(&mut self, summary: String) {
        self.self_summary = summary;
        self.mark_dirty();
    }

    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.1, 0.95);
        self.mark_dirty();
    }

    pub fn add_personality_trait(&mut self, trait_vsa: Vec<u8>) {
        if self.personality_traits.len() >= MAX_PERSONALITY_TRAITS {
            self.personality_traits.remove(0);
        }
        self.personality_traits.push(trait_vsa);
        self.mark_dirty();
    }

    pub fn add_core_value(&mut self, value: String) {
        if !self.core_values.contains(&value) {
            self.core_values.push(value);
            self.mark_dirty();
        }
    }

    pub fn push_coherence(&mut self, score: f64) {
        if self.coherence_history.len() >= COHERENCE_WINDOW {
            self.coherence_history.pop_front();
        }
        self.coherence_history.push_back(score);
    }

    pub fn current_coherence(&self) -> f64 {
        self.coherence_history.back().copied().unwrap_or(1.0)
    }

    pub fn check_anchor_drift(&mut self) -> f64 {
        let ctx = match self.boundary.run_before(BoundaryOp::CheckAnchor) {
            Ok(ctx) => Some(ctx),
            Err(e) => {
                log::warn!("[identity_core] check_anchor blocked by boundary: {e}");
                return self.last_drift;
            }
        };
        self.anchor_check_counter += 1;
        if self.anchor_check_counter % ANCHOR_CHECK_CYCLE != 0 {
            return self.last_drift;
        }
        let drift =
            if self.self_vsa.len() == self.anchor_self_vsa.len() && !self.self_vsa.is_empty() {
                let same = self
                    .self_vsa
                    .iter()
                    .zip(self.anchor_self_vsa.iter())
                    .filter(|(a, b)| a == b)
                    .count();
                1.0 - (same as f64 / self.self_vsa.len() as f64)
            } else {
                0.0
            };

        if drift > ANCHOR_DRIFT_THRESHOLD {
            for (a, b) in self.self_vsa.iter_mut().zip(self.anchor_self_vsa.iter()) {
                *a = (a.wrapping_mul(128) as f64 * ANCHOR_FUSION_RATIO
                    + *b as f64 * (1.0 - ANCHOR_FUSION_RATIO)) as u8;
            }
            self.anchor_self_vsa = self.self_vsa.clone();
            self.last_drift = 0.0;
            self.mark_dirty();
        } else {
            self.last_drift = drift;
        }
        if let Some(ctx) = &ctx {
            let _ = self
                .boundary
                .run_after(BoundaryOp::CheckAnchor, ctx, &Ok(()));
        }
        drift
    }

    pub fn last_drift(&self) -> f64 {
        self.last_drift
    }

    pub fn snapshot(&self) -> IdentitySnapshot {
        IdentitySnapshot {
            self_vsa: self.self_vsa.clone(),
            anchor_self_vsa: self.anchor_self_vsa.clone(),
            personality_traits: self.personality_traits.clone(),
            core_values: self.core_values.clone(),
            self_summary: self.self_summary.clone(),
            confidence_threshold: self.confidence_threshold,
            total_self_cycles: self.total_self_cycles,
            total_coproc_calls: self.total_coproc_calls,
            coherence_score: self.current_coherence(),
            last_drift: self.last_drift,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn record_hysteresis_snapshot(&mut self) {
        if self.hysteresis_tracker.len() >= HYSTERESIS_WINDOW {
            self.hysteresis_tracker.pop_front();
        }
        self.hysteresis_tracker
            .push_back((Instant::now(), self.self_vsa.clone()));
    }

    pub fn compute_hysteresis(&self) -> HysteresisMetrics {
        let l1_recovery = vsa_similarity(&self.self_vsa, &self.anchor_self_vsa);

        let short_term_drift = if self.hysteresis_tracker.len() >= 2 {
            let recent: Vec<_> = self.hysteresis_tracker.iter().rev().take(3).collect();
            if recent.len() >= 2 {
                let sim = vsa_similarity(&recent[0].1, &recent[1].1);
                1.0 - sim
            } else {
                0.0
            }
        } else {
            0.0
        };

        let long_term_trend = if self.hysteresis_tracker.len() >= 3 {
            let first = vsa_similarity(&self.anchor_self_vsa, &self.hysteresis_tracker[0].1);
            let last = vsa_similarity(&self.anchor_self_vsa, &self.self_vsa);
            if first > 0.0 {
                (last - first) / first
            } else {
                0.0
            }
        } else {
            0.0
        };

        let integrated_hysteresis = 1.0
            - vsa_similarity(
                &self.self_vsa,
                self.hysteresis_tracker
                    .front()
                    .map(|(_, vsa)| vsa)
                    .unwrap_or(&self.self_vsa),
            );

        HysteresisMetrics {
            l1_recovery,
            short_term_drift,
            long_term_trend,
            integrated_hysteresis,
        }
    }
    pub fn init_evolution(&mut self, config: IdentityEvolutionConfig) {
        let mut evolution = IdentityEvolution::new(config);
        evolution.init_from(self);
        self.evolution = Some(evolution);
        self.evolution_enabled = true;
    }

    pub fn evolve(&mut self, session_success_rate: f64) {
        let mut details = HashMap::new();
        details.insert("current_drift".to_string(), self.last_drift.to_string());
        details.insert(
            "current_coherence".to_string(),
            self.current_coherence().to_string(),
        );
        let ctx = match self
            .boundary
            .run_before_with_details(BoundaryOp::Evolve, details)
        {
            Ok(ctx) => ctx,
            Err(e) => {
                log::warn!("[identity_core] evolve blocked by boundary hook: {e}");
                return;
            }
        };
        let mut evolution = match self.evolution.take() {
            Some(e) => e,
            None => return,
        };
        evolution.apply_evolution(self, session_success_rate);
        self.evolution = Some(evolution);
        self.mark_dirty();
        let _ = self.boundary.run_after(BoundaryOp::Evolve, &ctx, &Ok(()));
    }

    pub fn rollback_identity(&mut self, version: u64) -> bool {
        let mut evolution = match self.evolution.take() {
            Some(e) => e,
            None => return false,
        };
        let result = evolution.rollback_to(self, version);
        self.evolution = Some(evolution);
        result
    }

    pub fn evolution_report(&self) -> String {
        match self.evolution.as_ref() {
            Some(evolution) => evolution.report(),
            None => "evolution:disabled".to_string(),
        }
    }
}

impl Default for IdentityCore {
    fn default() -> Self {
        Self::new()
    }
}

fn vsa_similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    same as f64 / a.len() as f64
}

#[derive(Debug, Clone, Default)]
struct LoadedIdentity {
    self_vsa: Vec<u8>,
    anchor_self_vsa: Vec<u8>,
    personality_traits: Vec<Vec<u8>>,
    core_values: Vec<String>,
    self_summary: String,
    confidence_threshold: f64,
    total_self_cycles: u64,
    total_coproc_calls: u64,
    last_distillation: String,
    coherence_history: VecDeque<f64>,
    last_drift: f64,
    narrative_self: NarrativeSelf,
    signature: Option<[u8; 64]>,
    signature_valid: bool,
    anchor_statuses: HashMap<String, AnchorStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentitySignData {
    self_vsa: Vec<u8>,
    anchor_self_vsa: Vec<u8>,
    core_values: Vec<String>,
    self_summary: String,
    total_self_cycles: u64,
}
