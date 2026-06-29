// G409: Self-understanding knowledge graph — Understand-Anything for NeoTrix's own codebase
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub description: String,
    pub tags: Vec<String>,
    pub layer: Option<CognitiveLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Module,
    Struct,
    Trait,
    Function,
    Impl,
    Subsystem,
    Interface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitiveLayer {
    Substrate,
    Perception,
    Cognition,
    MetaCognition,
    SelfEvolution,
    MetaArchitecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: DepType,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepType {
    Imports,
    Calls,
    Implements,
    Contains,
    Uses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfUnderstandingGraph {
    pub entities: HashMap<String, CodeEntity>,
    pub edges: Vec<DependencyEdge>,
    pub layers: HashMap<String, Vec<String>>,
    pub scan_timestamp: u64,
}

impl SelfUnderstandingGraph {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            edges: Vec::new(),
            layers: HashMap::new(),
            scan_timestamp: 0,
        }
    }

    pub fn add_entity(&mut self, entity: CodeEntity) {
        let id = entity.id.clone();
        if let Some(ref layer) = entity.layer {
            self.layers
                .entry(format!("{:?}", layer))
                .or_default()
                .push(id.clone());
        }
        self.entities.insert(id, entity);
    }

    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: DepType, weight: f64) {
        if self.entities.contains_key(from) && self.entities.contains_key(to) {
            self.edges.push(DependencyEdge {
                from: from.to_string(),
                to: to.to_string(),
                edge_type,
                weight,
            });
        }
    }

    pub fn get_dependents(&self, entity_id: &str) -> Vec<&DependencyEdge> {
        self.edges.iter().filter(|e| e.from == entity_id).collect()
    }

    pub fn get_dependencies(&self, entity_id: &str) -> Vec<&DependencyEdge> {
        self.edges.iter().filter(|e| e.to == entity_id).collect()
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&CodeEntity> {
        self.entities
            .values()
            .filter(|e| e.tags.iter().any(|t| t.contains(tag)))
            .collect()
    }

    pub fn find_by_layer(&self, layer: &CognitiveLayer) -> Vec<&CodeEntity> {
        let layer_name = format!("{:?}", layer);
        self.layers
            .get(&layer_name)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn critical_path(&self, start: &str, end: &str) -> Vec<String> {
        // Simple BFS to find dependency chain
        let mut visited = HashSet::new();
        let mut queue: Vec<(String, Vec<String>)> =
            vec![(start.to_string(), vec![start.to_string()])];
        while let Some((current, path)) = queue.pop() {
            if current == end {
                return path;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            for edge in &self.edges {
                if edge.from == current && !visited.contains(&edge.to) {
                    let mut new_path = path.clone();
                    new_path.push(edge.to.clone());
                    queue.push((edge.to.clone(), new_path));
                }
            }
        }
        Vec::new()
    }

    pub fn self_diagnose(&self) -> SelfDiagnosis {
        let missing_entities = Vec::new();
        let mut orphan_edges = Vec::new();
        let mut layer_gaps = Vec::new();

        // Find edges referencing missing entities
        for edge in &self.edges {
            if !self.entities.contains_key(&edge.from) {
                orphan_edges.push(edge.from.clone());
            }
            if !self.entities.contains_key(&edge.to) {
                orphan_edges.push(edge.to.clone());
            }
        }

        // Check layer completeness
        let expected_layers = [
            "Substrate",
            "Perception",
            "Cognition",
            "MetaCognition",
            "SelfEvolution",
            "MetaArchitecture",
        ];
        for layer in &expected_layers {
            let count = self.layers.get(*layer).map_or(0, |v| v.len());
            if count == 0 {
                layer_gaps.push(format!("Layer '{}' has no entities registered", layer));
            }
        }

        SelfDiagnosis {
            total_entities: self.entities.len(),
            total_edges: self.edges.len(),
            missing_entities,
            connected: !orphan_edges.is_empty(),
            orphan_edges,
            layer_gaps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfDiagnosis {
    pub total_entities: usize,
    pub total_edges: usize,
    pub missing_entities: Vec<String>,
    pub orphan_edges: Vec<String>,
    pub layer_gaps: Vec<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfUnderstandingEngine {
    pub graph: SelfUnderstandingGraph,
    pub scan_history: Vec<String>,
    pub max_history: usize,
}

impl SelfUnderstandingEngine {
    pub fn new() -> Self {
        Self {
            graph: SelfUnderstandingGraph::new(),
            scan_history: Vec::new(),
            max_history: 50,
        }
    }

    pub fn register_core_modules(&mut self) {
        // Register known NeoTrix cognitive architecture layers
        let modules = vec![
            (
                "hcube",
                "HyperCube VSA Engine",
                "Substrate",
                vec!["vsa", "vector", "hyperdimensional"],
                vec![],
            ),
            (
                "e8",
                "E8 64-State Reasoning Kernel",
                "Substrate",
                vec!["reasoning", "state-machine"],
                vec![],
            ),
            (
                "gwt",
                "Global Workspace Attention",
                "Cognition",
                vec!["attention", "consciousness"],
                vec!["e8", "hcube"],
            ),
            (
                "experience",
                "Experience Tree + SEAL Loop",
                "SelfEvolution",
                vec!["learning", "evolution"],
                vec!["gwt"],
            ),
            (
                "consciousness",
                "Consciousness Pipeline",
                "Perception",
                vec!["awareness", "stream"],
                vec!["hcube", "e8"],
            ),
            (
                "meta_cognition",
                "Meta-Cognition Loop",
                "MetaCognition",
                vec!["reflection", "meta"],
                vec!["consciousness", "experience"],
            ),
            (
                "calibration",
                "Calibration Engine",
                "Cognition",
                vec!["confidence", "calibration"],
                vec![],
            ),
            (
                "verification_gate",
                "Verification Gate",
                "MetaArchitecture",
                vec!["safety", "gate", "validation"],
                vec![],
            ),
        ];

        for (id, name, layer_str, tags, deps) in modules {
            let layer = match layer_str {
                "Substrate" => CognitiveLayer::Substrate,
                "Perception" => CognitiveLayer::Perception,
                "Cognition" => CognitiveLayer::Cognition,
                "MetaCognition" => CognitiveLayer::MetaCognition,
                "SelfEvolution" => CognitiveLayer::SelfEvolution,
                _ => CognitiveLayer::MetaArchitecture,
            };
            let entity = CodeEntity {
                id: id.to_string(),
                name: name.to_string(),
                entity_type: EntityType::Subsystem,
                file_path: format!("neotrix-core/src/core/nt_core_{}/", id),
                line_start: 0,
                line_end: 0,
                description: format!("{} — {}", name, layer_str),
                tags: tags.into_iter().map(String::from).collect(),
                layer: Some(layer),
            };
            self.graph.add_entity(entity);
            for dep in deps {
                self.graph.add_edge(id, dep, DepType::Imports, 1.0);
            }
        }
    }

    pub fn diagnose(&self) -> SelfDiagnosis {
        self.graph.self_diagnose()
    }

    pub fn explain(&self, entity_id: &str) -> String {
        let mut out = String::new();
        if let Some(entity) = self.graph.entities.get(entity_id) {
            out.push_str(&format!("=== {} ({}) ===\n", entity.name, entity.id));
            out.push_str(&format!("Type: {:?}\n", entity.entity_type));
            out.push_str(&format!("Layer: {:?}\n", entity.layer));
            out.push_str(&format!("Path: {}\n", entity.file_path));
            out.push_str(&format!("Description: {}\n", entity.description));
            out.push_str(&format!("Tags: {}\n", entity.tags.join(", ")));

            let deps = self.graph.get_dependencies(entity_id);
            if !deps.is_empty() {
                out.push_str("Dependencies:\n");
                for d in &deps {
                    out.push_str(&format!("  -> {} (weight: {:.2})\n", d.from, d.weight));
                }
            }
            let dependents = self.graph.get_dependents(entity_id);
            if !dependents.is_empty() {
                out.push_str("Dependents:\n");
                for d in &dependents {
                    out.push_str(&format!("  <- {} (weight: {:.2})\n", d.to, d.weight));
                }
            }
        } else {
            out.push_str(&format!("Entity '{}' not found in graph.\n", entity_id));
        }
        out
    }

    /// Export all entities as (name, layer_name, description) tuples for ArchitectureGovernor bridge.
    pub fn export_layer_map(&self) -> Vec<(String, String, String)> {
        self.graph
            .entities
            .values()
            .map(|e| {
                let layer = e
                    .layer
                    .as_ref()
                    .map(|l| format!("{:?}", l))
                    .unwrap_or_default();
                (e.name.clone(), layer, e.description.clone())
            })
            .collect()
    }
}
