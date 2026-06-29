use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::identity::IdentityCore;

const IDENTITY_DIR: &str = ".neotrix/identity";
const SOUL_VSA_FILE: &str = "soul.vsa";
const MEMORY_FILE: &str = "memory.json";
const RELATIONS_FILE: &str = "relations.json";

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
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
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
    signature: Option<[u8; 64]>,
    signature_valid: bool,
    anchor_statuses: HashMap<String, String>,
}

fn identity_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(IDENTITY_DIR))
        .unwrap_or_else(|| PathBuf::from(IDENTITY_DIR))
}

pub fn save_identity(core: &IdentityCore) {
    let path = identity_path();
    let _ = std::fs::create_dir_all(&path);

    let soul = SoulAnchor {
        self_vsa: core.self_vsa.clone(),
        anchor_self_vsa: core.anchor_self_vsa.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&soul) {
        let tmp = path.join("soul.vsa.tmp");
        let _ = std::fs::write(&tmp, &json);
        let _ = std::fs::rename(&tmp, path.join(SOUL_VSA_FILE));
    }

    let memory = MemoryAnchor {
        personality_traits: core.personality_traits.clone(),
        coherence_history: Vec::new(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&memory) {
        let tmp = path.join("memory.json.tmp");
        let _ = std::fs::write(&tmp, &json);
        let _ = std::fs::rename(&tmp, path.join(MEMORY_FILE));
    }

    let relations = RelationsAnchor {
        core_values: core.core_values.clone(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&relations) {
        let tmp = path.join("relations.json.tmp");
        let _ = std::fs::write(&tmp, &json);
        let _ = std::fs::rename(&tmp, path.join(RELATIONS_FILE));
    }
}

pub fn load_identity() -> Option<IdentityCore> {
    let path = identity_path();
    let mut core = IdentityCore::new();
    let mut any_loaded = false;

    let soul_path = path.join(SOUL_VSA_FILE);
    if let Ok(content) = std::fs::read_to_string(&soul_path) {
        if let Ok(soul) = serde_json::from_str::<SoulAnchor>(&content) {
            core.self_vsa = soul.self_vsa;
            core.anchor_self_vsa = soul.anchor_self_vsa;
            any_loaded = true;
        }
    }

    let mem_path = path.join(MEMORY_FILE);
    if let Ok(content) = std::fs::read_to_string(&mem_path) {
        if let Ok(mem) = serde_json::from_str::<MemoryAnchor>(&content) {
            core.personality_traits = mem.personality_traits;
            any_loaded = true;
        }
    }

    let rel_path = path.join(RELATIONS_FILE);
    if let Ok(content) = std::fs::read_to_string(&rel_path) {
        if let Ok(rel) = serde_json::from_str::<RelationsAnchor>(&content) {
            core.core_values = rel.core_values;
            any_loaded = true;
        }
    }

    if any_loaded { Some(core) } else { None }
}

pub fn identity_exists() -> bool {
    let path = identity_path();
    path.join(SOUL_VSA_FILE).exists()
}
