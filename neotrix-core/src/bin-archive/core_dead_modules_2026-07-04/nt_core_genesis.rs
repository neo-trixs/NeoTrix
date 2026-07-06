use std::sync::{Arc, RwLock};
use crate::core::l7_capability::nt_cap_tree::{CapabilityNode, CapabilityTree, InterfaceType, Provenance};

#[derive(Debug, Clone)]
pub struct CapabilityPattern {
    pub name: String,
    pub vector: Vec<f64>,
    pub composed_from: Vec<String>,
    pub source_url: Option<String>,
    pub confidence: f64,
}

pub struct CapabilityGenesis {
    tree: Arc<RwLock<CapabilityTree>>,
    absorption_history: Vec<CapabilityPattern>,
    composition_threshold: f64,
    new_branch_threshold: f64,
}

impl CapabilityGenesis {
    pub fn new(tree: Arc<RwLock<CapabilityTree>>) -> Self {
        Self {
            tree,
            absorption_history: Vec::new(),
            composition_threshold: 0.70,
            new_branch_threshold: 0.30,
        }
    }

    pub fn set_thresholds(&mut self, composition: f64, new_branch: f64) {
        self.composition_threshold = composition;
        self.new_branch_threshold = new_branch;
    }

    pub fn tick(&mut self) -> Vec<CapabilityPattern> {
        let mut proposals = Vec::new();
        let internal_patterns = self.discover_composition_patterns();
        for pattern in internal_patterns {
            if let Some(proposal) = self.evaluate_pattern(&pattern) {
                proposals.push(proposal);
            }
        }
        proposals
    }

    pub fn absorb_external(&mut self, pattern: CapabilityPattern) -> Option<CapabilityPattern> {
        self.evaluate_pattern(&pattern)
    }

    fn evaluate_pattern(&self, pattern: &CapabilityPattern) -> Option<CapabilityPattern> {
        let tree = self.tree.read().unwrap_or_else(|e| {
            log::warn!("[genesis] tree read lock poisoned: {}", e);
            e.into_inner()
        });
        let matches = tree.find_closest(&pattern.vector, 3);
        if matches.is_empty() {
            if pattern.confidence > 0.6 {
                return Some(pattern.clone());
            }
            return None;
        }
        let (best_id, best_sim) = &matches[0];
        if *best_sim > 0.92 {
            return None;
        }
        if *best_sim > self.composition_threshold {
            let composed = CapabilityPattern {
                name: format!("{}_composed", pattern.name),
                vector: pattern.vector.clone(),
                composed_from: vec![best_id.clone()],
                source_url: pattern.source_url.clone(),
                confidence: *best_sim,
            };
            return Some(composed);
        }
        if *best_sim < self.new_branch_threshold && pattern.confidence > 0.7 {
            return Some(pattern.clone());
        }
        None
    }

    pub fn commit_proposal(&mut self, pattern: CapabilityPattern, domain: &str, description: &str) {
        let mut tree = self.tree.write().unwrap_or_else(|e| {
            log::warn!("[genesis] tree write lock poisoned: {}", e);
            e.into_inner()
        });
        let id = format!("cap_{}", pattern.name.replace(' ', "_").to_lowercase());
        let mut node = CapabilityNode::new(&id, &pattern.name, domain, pattern.vector.clone());
        node.description = description.to_string();
        node.provenance = if let Some(ref url) = pattern.source_url {
            Provenance::Absorbed { source_url: url.clone() }
        } else if !pattern.composed_from.is_empty() {
            Provenance::Composed
        } else {
            Provenance::Manual
        };
        node.interface_type = if !pattern.composed_from.is_empty() {
            InterfaceType::Composed {
                components: pattern.composed_from.clone(),
                strategy: "composition".into(),
            }
        } else {
            InterfaceType::External { confidence: pattern.confidence }
        };
        tree.register(node);
        self.absorption_history.push(pattern);
    }

    fn discover_composition_patterns(&self) -> Vec<CapabilityPattern> {
        let tree = self.tree.read().unwrap_or_else(|e| {
            log::warn!("[genesis] tree read lock poisoned: {}", e);
            e.into_inner()
        });
        let nodes: Vec<&CapabilityNode> = tree.all_nodes().collect();
        let mut patterns = Vec::new();
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let sim = nodes[i].similarity(&nodes[j].vector);
                if sim > 0.3 && sim < 0.9 {
                    let mut combined = nodes[i].vector.clone();
                    for (k, v) in nodes[j].vector.iter().enumerate() {
                        if k < combined.len() {
                            combined[k] = (combined[k] + v) / 2.0;
                        }
                    }
                    patterns.push(CapabilityPattern {
                        name: format!("{}+{}", nodes[i].name, nodes[j].name),
                        vector: combined,
                        composed_from: vec![nodes[i].id.clone(), nodes[j].id.clone()],
                        source_url: None,
                        confidence: sim,
                    });
                }
            }
        }
        patterns
    }

    pub fn absorb_from_github(&mut self, repo_name: &str, stars: u64, description: &str) -> Option<CapabilityPattern> {
        let relevance_keywords = ["agent", "ai", "mcp", "llm", "compression", "video", "image", "voice",
            "code", "memory", "skill", "orchestrat", "crawl", "search", "embedding", "rag", "graph"];
        let lower_desc = description.to_lowercase();
        let matched: Vec<&str> = relevance_keywords.iter().filter(|k| lower_desc.contains(*k)).copied().collect();
        if matched.is_empty() && stars < 100 {
            return None;
        }
        let mut vector = vec![0.0; 23];
        for kw in &matched {
            let idx = match *kw {
                "agent" | "orchestrat" => 10,
                "ai" | "llm" => 8,
                "code" | "mcp" => 1,
                "compression" => 14,
                "video" | "image" => 4,
                "voice" => 5,
                "memory" => 12,
                "skill" => 19,
                "crawl" | "search" => 11,
                "embedding" | "rag" => 9,
                "graph" => 3,
                _ => 0,
            };
            vector[idx] = 0.5 + (stars as f64 / 100000.0).min(0.5);
        }
        if vector.iter().all(|v| *v == 0.0) {
            vector[0] = 0.3;
        }
        let confidence = (matched.len() as f64 * 0.15 + (stars as f64 / 100000.0).min(0.3)).min(1.0);
        Some(CapabilityPattern {
            name: repo_name.to_string(),
            vector,
            composed_from: Vec::new(),
            source_url: Some(format!("https://github.com/{}", repo_name)),
            confidence,
        })
    }

    pub fn absorption_count(&self) -> usize {
        self.absorption_history.len()
    }

    pub fn history(&self) -> &[CapabilityPattern] {
        &self.absorption_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vector(v: &[f64]) -> Vec<f64> {
        let mut arr = vec![0.0; 23];
        for (i, &val) in v.iter().enumerate().take(arr.len()) {
            arr[i] = val;
        }
        arr
    }

    #[test]
    fn test_genesis_new() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let genesis = CapabilityGenesis::new(tree);
        assert_eq!(genesis.absorption_count(), 0);
    }

    #[test]
    fn test_absorb_external_new_capability() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let mut genesis = CapabilityGenesis::new(tree);
        let pattern = CapabilityPattern {
            name: "voice_synthesis".into(),
            vector: test_vector(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            composed_from: vec![],
            source_url: Some("https://github.com/jamiepine/voicebox".into()),
            confidence: 0.85,
        };
        let proposal = genesis.absorb_external(pattern);
        assert!(proposal.is_some());
    }

    #[test]
    fn test_absorb_external_high_similarity_no_new() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        {
            let mut t = tree.write().unwrap();
            t.register(CapabilityNode::new("existing", "E8 Reasoning", "Cognitive", test_vector(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.9, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
        }
        let mut genesis = CapabilityGenesis::new(tree);
        let pattern = CapabilityPattern {
            name: "similar_reasoning".into(),
            vector: test_vector(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.9, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            composed_from: vec![],
            source_url: None,
            confidence: 0.9,
        };
        let proposal = genesis.absorb_external(pattern);
        assert!(proposal.is_none());
    }

    #[test]
    fn test_commit_proposal_adds_to_tree() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let mut genesis = CapabilityGenesis::new(tree.clone());
        let pattern = CapabilityPattern {
            name: "image_generation".into(),
            vector: test_vector(&[0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            composed_from: vec![],
            source_url: None,
            confidence: 0.8,
        };
        genesis.commit_proposal(pattern, "Creative", "Text-to-image generation capability");
        assert_eq!(tree.read().unwrap().node_count(), 1);
        assert_eq!(genesis.absorption_count(), 1);
    }

    #[test]
    fn test_discover_composition_patterns() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        {
            let mut t = tree.write().unwrap();
            t.register(CapabilityNode::new("img", "Image Gen", "Creative", test_vector(&[0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
            t.register(CapabilityNode::new("voice", "Voice Synth", "Creative", test_vector(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
        }
        let genesis = CapabilityGenesis::new(tree);
        let patterns = genesis.discover_composition_patterns();
        assert!(patterns.len() >= 1);
        assert!(patterns[0].name.contains('+'));
    }

    #[test]
    fn test_absorb_from_github_high_stars() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let mut genesis = CapabilityGenesis::new(tree);
        let pattern = genesis.absorb_from_github("headroomlabs-ai/headroom", 54372, "LLM context compression engine with MCP support");
        assert!(pattern.is_some());
        let p = pattern.unwrap();
        assert!(p.confidence > 0.5);
        assert_eq!(p.source_url.unwrap(), "https://github.com/headroomlabs-ai/headroom");
    }

    #[test]
    fn test_absorb_from_github_low_stars_no_keywords() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let mut genesis = CapabilityGenesis::new(tree);
        let pattern = genesis.absorb_from_github("user/lowstar", 10, "A simple utility tool");
        assert!(pattern.is_none());
    }

    #[test]
    fn test_tick_with_existing_nodes() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        {
            let mut t = tree.write().unwrap();
            t.register(CapabilityNode::new("a", "A", "Test", test_vector(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
            t.register(CapabilityNode::new("b", "B", "Test", test_vector(&[0.5, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])));
        }
        let mut genesis = CapabilityGenesis::new(tree);
        let proposals = genesis.tick();
        assert!(proposals.len() >= 1, "tick should produce at least one composition proposal");
    }

    #[test]
    fn test_set_thresholds() {
        let tree = Arc::new(RwLock::new(CapabilityTree::new()));
        let mut genesis = CapabilityGenesis::new(tree);
        genesis.set_thresholds(0.8, 0.2);
        assert!((genesis.composition_threshold - 0.8).abs() < 1e-10);
        assert!((genesis.new_branch_threshold - 0.2).abs() < 1e-10);
    }
}
