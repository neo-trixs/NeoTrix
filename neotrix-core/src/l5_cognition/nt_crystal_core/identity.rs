//! L1 身份层 — 不可变核心: 公理、价值观、身份

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单条公理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Axiom {
    pub id: String,
    pub name: String,
    pub description: String,
    pub implication: String,
}

/// 价值观权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueWeights {
    pub weights: HashMap<String, f64>,
}

impl ValueWeights {
    pub fn get(&self, key: &str) -> f64 {
        self.weights.get(key).copied().unwrap_or(0.0)
    }

    pub fn overall(&self) -> f64 {
        if self.weights.is_empty() {
            return 0.0;
        }
        self.weights.values().sum::<f64>() / self.weights.len() as f64
    }
}

/// L1 身份层 — 不可变核心
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalIdentity {
    pub name: String,
    pub version: String,
    pub axioms: Vec<Axiom>,
    pub values: ValueWeights,
    pub created_at: String,
}

impl CrystalIdentity {
    /// 创建新的身份
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            axioms: Self::default_axioms(),
            values: Self::default_values(),
            created_at: chrono_now(),
        }
    }

    /// 默认公理 (A1-A8)
    pub fn default_axioms() -> Vec<Axiom> {
        vec![
            Axiom {
                id: "A1".into(),
                name: "Cost-Aware Routing".into(),
                description: "Not all tasks need the strongest model".into(),
                implication: "Route tasks to cheapest capable model".into(),
            },
            Axiom {
                id: "A2".into(),
                name: "Context as Scarce Resource".into(),
                description: "Context window / KV capacity is the fundamental bottleneck".into(),
                implication: "Optimize context usage, compact when needed".into(),
            },
            Axiom {
                id: "A3".into(),
                name: "Skill as Production Template".into(),
                description: "Skills are structured, composable, versionable expert knowledge".into(),
                implication: "SKILL-SPEC.md contract for all implementations".into(),
            },
            Axiom {
                id: "A4".into(),
                name: "Safety First".into(),
                description: "Forbid unsafe code, archive before delete".into(),
                implication: "R-P1: #![forbid(unsafe_code)], R-P81: archive_before_delete".into(),
            },
            Axiom {
                id: "A5".into(),
                name: "Evolution Over Revolution".into(),
                description: "Incremental improvement over big bang changes".into(),
                implication: "Small focused PRs, continuous deployment".into(),
            },
            Axiom {
                id: "A6".into(),
                name: "Evidence Over Assertion".into(),
                description: "Verified results over claimed results".into(),
                implication: "cargo check + cargo test before claiming success".into(),
            },
            Axiom {
                id: "A7".into(),
                name: "Modularity Over Monolith".into(),
                description: "Small focused modules over large coupled ones".into(),
                implication: "nt_ prefix, single responsibility per module".into(),
            },
            Axiom {
                id: "A8".into(),
                name: "Determinism Over Ambiguity".into(),
                description: "Clear rules over vague guidelines".into(),
                implication: "Explicit interfaces, documented contracts".into(),
            },
        ]
    }

    /// 默认价值观权重
    pub fn default_values() -> ValueWeights {
        let mut weights = HashMap::new();
        weights.insert("safety".into(), 1.0);
        weights.insert("efficiency".into(), 0.8);
        weights.insert("creativity".into(), 0.7);
        weights.insert("modularity".into(), 0.9);
        weights.insert("evolution".into(), 0.8);
        weights.insert("transparency".into(), 0.6);
        ValueWeights { weights }
    }

    /// 根据 ID 查找公理
    pub fn find_axiom(&self, id: &str) -> Option<&Axiom> {
        self.axioms.iter().find(|a| a.id == id)
    }
}

/// 简单时间戳 (避免依赖 chrono)
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}-{:04}", secs / 31536000 + 1970, secs % 31536000 / 86400)
}
