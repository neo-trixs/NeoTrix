// SkillTree Implementation
// POE-inspired capability progression: Small Passive / Notable Passive / Keystone

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;

struct SkillTreeInner {
    nodes: Vec<SkillNode>,
    allocated: u32,
    available: u32,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct SkillTreeImpl {
    inner: Arc<RwLock<SkillTreeInner>>,
}

#[uniffi::export]
impl SkillTreeImpl {
    #[uniffi::constructor]
    pub fn init() -> Result<Self, NeoTrixError> {
        Ok(Self {
            inner: Arc::new(RwLock::new(SkillTreeInner {
                nodes: build_skill_nodes(),
                allocated: 0,
                available: 10,
            })),
        })
    }

    pub fn get_state(&self) -> SkillTreeState {
        let inner = self.inner.read().unwrap();
        SkillTreeState {
            nodes: inner.nodes.clone(),
            allocated_points: inner.allocated,
            available_points: inner.available,
            active_constellations: compute_constellations(&inner),
        }
    }

    pub fn allocate_point(&self, node_id: &str) -> Result<SkillNode, NeoTrixError> {
        let mut inner = self.inner.write().unwrap();
        if inner.available == 0 {
            return Err(NeoTrixError::OperationFailed);
        }
        let idx = inner.nodes.iter().position(|n| n.id == node_id).ok_or(NeoTrixError::NotFound)?;
        let prereqs_ok = inner.nodes[idx].prerequisites.iter().all(|p| {
            inner.nodes.iter().any(|n| n.id == *p && n.unlocked)
        });
        if !prereqs_ok {
            return Err(NeoTrixError::InvalidInput);
        }
        inner.nodes[idx].unlocked = true;
        inner.nodes[idx].progress = 1.0;
        inner.allocated += 1;
        inner.available -= 1;
        Ok(inner.nodes[idx].clone())
    }

    pub fn respec(&self) -> SkillTreeState {
        let mut inner = self.inner.write().unwrap();
        for node in inner.nodes.iter_mut() {
            node.unlocked = false;
            node.progress = 0.0;
        }
        inner.available = inner.allocated + 10;
        inner.allocated = 0;
        SkillTreeState {
            nodes: inner.nodes.clone(),
            allocated_points: inner.allocated,
            available_points: inner.available,
            active_constellations: compute_constellations(&inner),
        }
    }

    pub fn get_node(&self, node_id: &str) -> Result<SkillNode, NeoTrixError> {
        self.inner.read().unwrap().nodes.iter().find(|n| n.id == node_id).cloned().ok_or(NeoTrixError::NotFound)
    }

    pub fn is_constellation_active(&self, constellation: &str) -> bool {
        let inner = self.inner.read().unwrap();
        let constellation_level = constellation[1..].parse::<u8>().unwrap_or(0);
        let total = inner.nodes.iter().filter(|n| n.unlocked).count() as u8;
        total >= constellation_level
    }

    pub fn get_recommendations(&self, playstyle: &str) -> Vec<String> {
        match playstyle {
            "acquisition" => vec!["NT-WORLD-1".into(), "NT-WORLD-2".into(), "NT-MEMORY-1".into()],
            "evolution" => vec!["NT-MIND-1".into(), "NT-MIND-2".into(), "NT-CORE-1".into()],
            "balanced" => vec!["NT-CORE-1".into(), "NT-MIND-1".into(), "NT-WORLD-1".into()],
            _ => Vec::new(),
        }
    }
}

fn build_skill_nodes() -> Vec<SkillNode> {
    let mut nodes = Vec::new();
    let defs: Vec<(&str, &str, &str, &str, Vec<&str>, &str, &str, f32)> = vec![
        // NT-CORE
        ("NT-CORE-1", "E8 Clarity", "E8 reasoning confidence +15%", "Small Passive", vec![], "NT-CORE", "stat_boost", 0.15),
        ("NT-CORE-2", "GWT Resonance", "Attention routing efficiency +20%", "Small Passive", vec!["NT-CORE-1"], "NT-CORE", "stat_boost", 0.20),
        ("NT-CORE-3", "VSA Mastery", "HyperCube dimension 1024→2048", "Notable Passive", vec!["NT-CORE-2"], "NT-CORE", "new_ability", 0.0),
        ("NT-CORE-4", "Consciousness Core", "Phi integration score +0.1", "Keystone", vec!["NT-CORE-3"], "NT-CORE", "stat_boost", 0.1),
        // NT-MIND
        ("NT-MIND-1", "Pattern Extractor", "Distillation pattern extraction +30%", "Small Passive", vec![], "NT-MIND", "efficiency", 0.30),
        ("NT-MIND-2", "Skill Crystallizer", "New skills crystallize 25% faster", "Small Passive", vec!["NT-MIND-1"], "NT-MIND", "efficiency", 0.25),
        ("NT-MIND-3", "Evolution Accelerator", "SEAL cycle velocity +40%", "Notable Passive", vec!["NT-MIND-2"], "NT-MIND", "efficiency", 0.40),
        ("NT-MIND-4", "Meta-Crystallizer", "Auto-crystallize meta patterns", "Keystone", vec!["NT-MIND-3"], "NT-MIND", "new_ability", 0.0),
        // NT-MEMORY
        ("NT-MEMORY-1", "Spatial Memory", "Unlock spatial memory store", "Small Passive", vec![], "NT-MEMORY", "new_ability", 0.0),
        ("NT-MEMORY-2", "Semantic Indexer", "Search relevance +25%", "Small Passive", vec!["NT-MEMORY-1"], "NT-MEMORY", "stat_boost", 0.25),
        ("NT-MEMORY-3", "Knowledge Weaver", "Cross-namespace edge linking", "Notable Passive", vec!["NT-MEMORY-2"], "NT-MEMORY", "new_ability", 0.0),
        ("NT-MEMORY-4", "Infinite Archive", "Unlimited KB capacity", "Keystone", vec!["NT-MEMORY-3"], "NT-MEMORY", "new_ability", 0.0),
        // NT-WORLD
        ("NT-WORLD-1", "Sensor Fusion", "Multi-sensor data fusion", "Small Passive", vec![], "NT-WORLD", "new_ability", 0.0),
        ("NT-WORLD-2", "Pattern Radar", "Discovery confidence +20%", "Small Passive", vec!["NT-WORLD-1"], "NT-WORLD", "stat_boost", 0.20),
        ("NT-WORLD-3", "World Model", "Predictive world model", "Notable Passive", vec!["NT-WORLD-2"], "NT-WORLD", "new_ability", 0.0),
        ("NT-WORLD-4", "Omniscient View", "Full omniscient perception", "Keystone", vec!["NT-WORLD-3"], "NT-WORLD", "new_ability", 0.0),
    ];

    for (id, name, desc, tier, prereqs, domain, effect_type, value) in defs {
        nodes.push(SkillNode {
            id: id.into(),
            name: name.into(),
            description: desc.into(),
            tier: tier.into(),
            domain: domain.into(),
            prerequisites: prereqs.iter().map(|s| s.to_string()).collect(),
            unlocked: false,
            progress: 0.0,
            effects: vec![SkillEffect {
                effect_type: effect_type.into(),
                target: id.into(),
                value,
                description: desc.into(),
            }],
        });
    }
    nodes
}

fn compute_constellations(inner: &SkillTreeInner) -> Vec<String> {
    let mut active = Vec::new();
    let tiers: Vec<(&str, usize)> = vec![("NT-CORE", 0), ("NT-MIND", 1), ("NT-MEMORY", 2), ("NT-WORLD", 3)];
    for (domain, _base) in tiers {
        let domain_nodes: Vec<&SkillNode> = inner.nodes.iter().filter(|n| n.domain == domain && n.unlocked).collect();
        let c = (domain_nodes.len() as u8).min(6);
        if c > 0 {
            active.push(format!("C{}", c));
        }
    }
    active
}