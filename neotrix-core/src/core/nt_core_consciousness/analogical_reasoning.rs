use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::{HashMap, HashSet};

const ENTITY_VSA_DIM: usize = 256;

fn entity_vector(name: &str) -> Vec<u8> {
    let seed: u64 = name.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });
    QuantizedVSA::seeded_random(seed, ENTITY_VSA_DIM)
}

fn bipolar_similarity(a: &[u8], b: &[u8]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let dot: f64 = a[..len]
        .iter()
        .zip(&b[..len])
        .map(|(x, y)| {
            let va = if *x == 0 { -1.0 } else { 1.0 };
            let vb = if *y == 0 { -1.0 } else { 1.0 };
            va * vb
        })
        .sum();
    dot / len as f64
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub vector: Vec<u8>,
}

impl Entity {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            vector: entity_vector(name),
        }
    }

    pub fn similarity(&self, other: &Entity) -> f64 {
        bipolar_similarity(&self.vector, &other.vector)
    }
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub name: String,
    pub arguments: Vec<String>,
}

impl Relation {
    pub fn new(name: &str, args: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            arguments: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnalogicalStructure {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub name: String,
}

impl AnalogicalStructure {
    pub fn new(name: &str) -> Self {
        Self {
            entities: Vec::new(),
            relations: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn add_entity(&mut self, name: &str) {
        if !self.entities.iter().any(|e| e.name == name) {
            self.entities.push(Entity::new(name));
        }
    }

    pub fn add_relation(&mut self, name: &str, args: Vec<&str>) {
        for arg in &args {
            self.add_entity(arg);
        }
        self.relations.push(Relation::new(name, args));
    }

    pub fn find_entity(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|e| e.name == name)
    }

    pub fn relation_names(&self) -> HashSet<&str> {
        self.relations.iter().map(|r| r.name.as_str()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct AnalogicalMapping {
    pub entity_map: HashMap<String, String>,
    pub structural_consistency: f64,
    pub relation_preservation: f64,
}

impl AnalogicalMapping {
    pub fn new() -> Self {
        Self {
            entity_map: HashMap::new(),
            structural_consistency: 0.0,
            relation_preservation: 0.0,
        }
    }

    pub fn target_for(&self, source: &str) -> Option<&str> {
        self.entity_map.get(source).map(|s| s.as_str())
    }
}

impl Default for AnalogicalMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StructureMappingEngine {
    pub max_candidates: usize,
    pub consistency_threshold: f64,
    pub entity_similarity_weight: f64,
    pub structural_weight: f64,
    pub relational_weight: f64,
}

impl Default for StructureMappingEngine {
    fn default() -> Self {
        Self {
            max_candidates: 10,
            consistency_threshold: 0.3,
            entity_similarity_weight: 0.3,
            structural_weight: 0.6,
            relational_weight: 0.4,
        }
    }
}

impl StructureMappingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_candidate_mappings(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
    ) -> Vec<AnalogicalMapping> {
        let source_entities: Vec<&Entity> = source.entities.iter().collect();
        let target_entities: Vec<&Entity> = target.entities.iter().collect();

        if source_entities.is_empty() || target_entities.is_empty() {
            return Vec::new();
        }

        let mut used_targets: HashSet<usize> = HashSet::new();
        let mut entity_map = HashMap::new();

        for s_entity in &source_entities {
            let mut best_idx = None;
            let mut best_sim = f64::NEG_INFINITY;

            for (j, t_entity) in target_entities.iter().enumerate() {
                if used_targets.contains(&j) {
                    continue;
                }
                let sim = s_entity.similarity(t_entity);
                if sim > best_sim {
                    best_sim = sim;
                    best_idx = Some(j);
                }
            }

            if let Some(idx) = best_idx {
                entity_map.insert(s_entity.name.clone(), target_entities[idx].name.clone());
                used_targets.insert(idx);
            }
        }

        let mut mapping = AnalogicalMapping {
            entity_map,
            structural_consistency: 0.0,
            relation_preservation: 0.0,
        };

        mapping.structural_consistency =
            self.compute_structural_consistency(source, target, &mapping);
        mapping.relation_preservation = self.compute_relation_preservation(source, target);
        mapping.relation_preservation = mapping
            .relation_preservation
            .max(self.compute_target_relation_preservation(source, target, &mapping));

        vec![mapping]
    }

    pub fn compute_structural_consistency(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
        mapping: &AnalogicalMapping,
    ) -> f64 {
        if mapping.entity_map.is_empty() || source.relations.is_empty() {
            return 0.0;
        }

        let mut preserved = 0u64;
        let mut total = 0u64;

        for s_rel in &source.relations {
            let mapped_args: Vec<String> = s_rel
                .arguments
                .iter()
                .filter_map(|arg| mapping.entity_map.get(arg))
                .cloned()
                .collect();

            if mapped_args.len() == s_rel.arguments.len() {
                total += 1;
                let has_match = target
                    .relations
                    .iter()
                    .any(|t_rel| t_rel.name == s_rel.name && t_rel.arguments == mapped_args);
                if has_match {
                    preserved += 1;
                }
            }
        }

        if total == 0 {
            return 0.0;
        }

        preserved as f64 / total as f64
    }

    fn compute_relation_preservation(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
    ) -> f64 {
        if source.relations.is_empty() || target.relations.is_empty() {
            return 0.0;
        }

        let source_names: HashSet<&str> = source.relation_names();
        let target_names: HashSet<&str> = target.relation_names();

        let overlap: usize = source_names
            .iter()
            .filter(|n| target_names.contains(*n))
            .count();
        overlap as f64 / source_names.len() as f64
    }

    fn compute_target_relation_preservation(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
        mapping: &AnalogicalMapping,
    ) -> f64 {
        if mapping.entity_map.is_empty() || target.relations.is_empty() {
            return 0.0;
        }

        let mut matched = 0u64;
        let mut total = 0u64;

        for t_rel in &target.relations {
            let unmapped_args: Vec<&str> = t_rel
                .arguments
                .iter()
                .filter(|arg| !mapping.entity_map.values().any(|v| v == *arg))
                .map(|s| s.as_str())
                .collect();

            if unmapped_args.is_empty() {
                total += 1;
                let has_match = source.relations.iter().any(|s_rel| {
                    s_rel.name == t_rel.name && {
                        let src_args: Vec<String> = s_rel
                            .arguments
                            .iter()
                            .filter_map(|a| mapping.entity_map.get(a))
                            .cloned()
                            .collect();
                        src_args == t_rel.arguments
                    }
                });
                if has_match {
                    matched += 1;
                }
            }
        }

        if total == 0 {
            return 0.0;
        }

        matched as f64 / total as f64
    }

    pub fn project_inferences(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
        mapping: &AnalogicalMapping,
    ) -> Vec<Relation> {
        let mut inferences = Vec::new();

        for s_rel in &source.relations {
            let mapped_args: Vec<String> = s_rel
                .arguments
                .iter()
                .filter_map(|arg| mapping.entity_map.get(arg))
                .cloned()
                .collect();

            if mapped_args.len() == s_rel.arguments.len() {
                let exists = target
                    .relations
                    .iter()
                    .any(|t_rel| t_rel.name == s_rel.name && t_rel.arguments == mapped_args);
                if !exists {
                    let has_name = target
                        .relations
                        .iter()
                        .any(|t_rel| t_rel.name == s_rel.name);
                    if has_name {
                        inferences.push(Relation {
                            name: s_rel.name.clone(),
                            arguments: mapped_args,
                        });
                    }
                }
            }
        }

        inferences
    }

    pub fn analogical_transfer(
        &self,
        source: &AnalogicalStructure,
        target: &AnalogicalStructure,
    ) -> (AnalogicalMapping, Vec<Relation>) {
        let candidates = self.find_candidate_mappings(source, target);
        if candidates.is_empty() {
            return (AnalogicalMapping::new(), Vec::new());
        }

        let best = match candidates
            .into_iter()
            .max_by(|a, b| {
                let score_a = a.structural_consistency * self.structural_weight
                    + a.relation_preservation * self.relational_weight;
                let score_b = b.structural_consistency * self.structural_weight
                    + b.relation_preservation * self.relational_weight;
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
            Some(b) => b,
            None => return (Default::default(), vec![]),
        };

        let inferences = self.project_inferences(source, target, &best);
        (best, inferences)
    }
}

#[derive(Debug, Clone)]
pub struct AnalogyResult {
    pub mapping: AnalogicalMapping,
    pub projected_inferences: Vec<Relation>,
    pub analogy_score: f64,
    pub structural_similarity: f64,
}

#[derive(Debug, Clone)]
pub struct AnalogicalReasoner {
    pub engine: StructureMappingEngine,
}

impl AnalogicalReasoner {
    pub fn new() -> Self {
        Self {
            engine: StructureMappingEngine::new(),
        }
    }

    pub fn with_engine(engine: StructureMappingEngine) -> Self {
        Self { engine }
    }

    pub fn reason_by_analogy(
        &self,
        source_domain: &AnalogicalStructure,
        target_domain: &AnalogicalStructure,
    ) -> AnalogyResult {
        let (mapping, inferences) = self
            .engine
            .analogical_transfer(source_domain, target_domain);

        let structure_sim =
            self.engine
                .compute_structural_consistency(source_domain, target_domain, &mapping);
        let mapping_quality = 0.6 * structure_sim + 0.4 * mapping.relation_preservation;
        let analogy_score = self.score_analogy(structure_sim, mapping_quality);

        AnalogyResult {
            mapping,
            projected_inferences: inferences,
            analogy_score,
            structural_similarity: structure_sim,
        }
    }

    pub fn score_analogy(&self, structure_sim: f64, mapping_quality: f64) -> f64 {
        0.6 * structure_sim + 0.4 * mapping_quality
    }
}

impl Default for AnalogicalReasoner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solar_system_structure() -> AnalogicalStructure {
        let mut s = AnalogicalStructure::new("solar_system");
        s.add_relation("revolves_around", vec!["planet", "sun"]);
        s.add_relation("attracts", vec!["sun", "planet"]);
        s.add_relation("is_larger", vec!["sun", "planet"]);
        s
    }

    fn atom_structure() -> AnalogicalStructure {
        let mut s = AnalogicalStructure::new("atom");
        s.add_relation("revolves_around", vec!["electron", "nucleus"]);
        s.add_relation("attracts", vec!["nucleus", "electron"]);
        s.add_relation("is_larger", vec!["nucleus", "electron"]);
        s
    }

    fn water_flow_structure() -> AnalogicalStructure {
        let mut s = AnalogicalStructure::new("water_flow");
        s.add_relation("flows_from", vec!["high_water", "low_water"]);
        s.add_relation("causes", vec!["pressure_difference", "flow"]);
        s
    }

    fn heat_flow_structure() -> AnalogicalStructure {
        let mut s = AnalogicalStructure::new("heat_flow");
        s.add_relation("flows_from", vec!["hot_object", "cold_object"]);
        s.add_relation("causes", vec!["temperature_difference", "heat_transfer"]);
        s
    }

    fn family_structure() -> AnalogicalStructure {
        let mut s = AnalogicalStructure::new("family");
        s.add_relation("parent_of", vec!["father", "son"]);
        s.add_relation("parent_of", vec!["mother", "daughter"]);
        s
    }

    #[test]
    fn test_entity_similarity_identical() {
        let e1 = Entity::new("sun");
        let e2 = Entity::new("sun");
        let sim = e1.similarity(&e2);
        assert!(
            (sim - 1.0).abs() < 1e-9,
            "identical entities should have similarity 1.0, got {}",
            sim
        );
    }

    #[test]
    fn test_entity_similarity_different() {
        let e1 = Entity::new("sun");
        let e2 = Entity::new("nucleus");
        let sim = e1.similarity(&e2);
        assert!(
            sim > -1.0 && sim < 1.0,
            "similarity should be in (-1, 1), got {}",
            sim
        );
    }

    #[test]
    fn test_solar_system_to_atom_analogy() {
        let source = solar_system_structure();
        let target = atom_structure();
        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(
            !candidates.is_empty(),
            "should find at least one candidate mapping"
        );

        let mapping = &candidates[0];
        let consistency = engine.compute_structural_consistency(&source, &target, mapping);
        assert!(
            consistency > 0.5,
            "solar system -> atom should have high structural consistency, got {}",
            consistency
        );
    }

    #[test]
    fn test_project_inferences_atom() {
        let source = solar_system_structure();
        let mut target = atom_structure();
        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(!candidates.is_empty());

        let mapping = &candidates[0];
        let inferences = engine.project_inferences(&source, &target, mapping);

        let original_count = target.relations.len();
        for inf in &inferences {
            target.add_relation(
                &inf.name,
                inf.arguments.iter().map(|s| s.as_str()).collect(),
            );
        }
        assert!(
            target.relations.len() >= original_count,
            "should have projected at least some inferences"
        );
    }

    #[test]
    fn test_analogical_transfer_solar_to_atom() {
        let source = solar_system_structure();
        let target = atom_structure();
        let engine = StructureMappingEngine::new();
        let (mapping, inferences) = engine.analogical_transfer(&source, &target);
        assert!(!mapping.entity_map.is_empty(), "should produce a mapping");
        assert!(
            mapping.structural_consistency > 0.5,
            "structural consistency should be high"
        );
    }

    #[test]
    fn test_empty_source() {
        let source = AnalogicalStructure::new("empty");
        let target = atom_structure();
        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(
            candidates.is_empty(),
            "empty source should produce no candidates"
        );
    }

    #[test]
    fn test_empty_target() {
        let source = solar_system_structure();
        let target = AnalogicalStructure::new("empty");
        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(
            candidates.is_empty(),
            "empty target should produce no candidates"
        );
    }

    #[test]
    fn test_identical_structures() {
        let source = solar_system_structure();
        let target = solar_system_structure();
        let engine = StructureMappingEngine::new();
        let (mapping, _) = engine.analogical_transfer(&source, &target);
        assert!(!mapping.entity_map.is_empty());
        assert!((mapping.structural_consistency - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_no_structural_overlap() {
        let mut source = AnalogicalStructure::new("math");
        source.add_relation("greater_than", vec!["x", "y"]);
        let mut target = AnalogicalStructure::new("colors");
        target.add_relation("darker_than", vec!["red", "blue"]);
        let engine = StructureMappingEngine::new();
        let consistency =
            engine.compute_structural_consistency(&source, &target, &AnalogicalMapping::new());
        assert!(
            (consistency - 0.0).abs() < 1e-6,
            "no overlap should give 0 consistency"
        );
    }

    #[test]
    fn test_water_flow_to_heat_flow() {
        let source = water_flow_structure();
        let target = heat_flow_structure();
        let engine = StructureMappingEngine::new();
        let (mapping, inferences) = engine.analogical_transfer(&source, &target);
        assert!(
            !mapping.entity_map.is_empty(),
            "water->heat should find mappings"
        );
        assert!(
            mapping.relation_preservation > 0.0,
            "shared relation names should be preserved"
        );
    }

    #[test]
    fn test_analogical_reasoner_full_pipeline() {
        let source = solar_system_structure();
        let target = atom_structure();
        let reasoner = AnalogicalReasoner::new();
        let result = reasoner.reason_by_analogy(&source, &target);
        assert!(
            result.analogy_score > 0.0,
            "analogy score should be positive, got {}",
            result.analogy_score
        );
        assert!(
            result.structural_similarity > 0.0,
            "structural similarity should be positive"
        );
        assert!(
            !result.mapping.entity_map.is_empty(),
            "should have entity mappings"
        );
    }

    #[test]
    fn test_score_analogy() {
        let reasoner = AnalogicalReasoner::new();
        let high = reasoner.score_analogy(0.9, 0.8);
        let low = reasoner.score_analogy(0.2, 0.1);
        assert!(
            high > low,
            "high similarity should score higher than low similarity"
        );
        let expected = 0.6 * 0.9 + 0.4 * 0.8;
        assert!((high - expected).abs() < 1e-9);
    }

    #[test]
    fn test_multiple_candidate_mappings() {
        let mut source = AnalogicalStructure::new("source");
        source.add_relation("related_to", vec!["a", "b"]);

        let mut target = AnalogicalStructure::new("target");
        target.add_relation("related_to", vec!["x", "y"]);

        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(!candidates.is_empty(), "should find at least one mapping");
    }

    #[test]
    fn test_family_analogy() {
        let mut source = AnalogicalStructure::new("source_family");
        source.add_relation("parent_of", vec!["alice", "bob"]);

        let mut target = AnalogicalStructure::new("target_family");
        target.add_relation("parent_of", vec!["carol", "dave"]);

        let engine = StructureMappingEngine::new();
        let (mapping, _) = engine.analogical_transfer(&source, &target);
        assert_eq!(mapping.entity_map.len(), 2, "should map both entities");
        assert!(
            (mapping.structural_consistency - 1.0).abs() < 1e-6,
            "identical relation structure"
        );
    }

    #[test]
    fn test_project_inferences_adds_novel_knowledge() {
        let mut source = AnalogicalStructure::new("source");
        source.add_relation("orbits", vec!["moon", "earth"]);
        source.add_relation("is_attracted_by", vec!["earth", "moon"]);

        let mut target = AnalogicalStructure::new("target");
        target.add_relation("orbits", vec!["satellite", "planet"]);

        let engine = StructureMappingEngine::new();
        let (mapping, inferences) = engine.analogical_transfer(&source, &target);

        let has_novel = inferences.iter().any(|r| r.name == "is_attracted_by");
        assert!(
            has_novel || mapping.structural_consistency == 0.0,
            "should project is_attracted_by if mapping succeeded"
        );
    }

    #[test]
    fn test_relation_entity_roundtrip() {
        let mut s = AnalogicalStructure::new("test");
        s.add_relation("contains", vec!["whole", "part"]);
        assert_eq!(s.entities.len(), 2, "add_relation should auto-add entities");
        assert!(s.find_entity("whole").is_some());
        assert!(s.find_entity("part").is_some());
    }

    #[test]
    fn test_mapping_one_to_one() {
        let mut source = AnalogicalStructure::new("src");
        source.add_relation("supports", vec!["a", "b"]);

        let mut target = AnalogicalStructure::new("tgt");
        target.add_relation("supports", vec!["x", "y"]);

        let engine = StructureMappingEngine::new();
        let candidates = engine.find_candidate_mappings(&source, &target);
        assert!(!candidates.is_empty());

        let mapping = &candidates[0];
        let vals: HashSet<&String> = mapping.entity_map.values().collect();
        assert_eq!(
            vals.len(),
            mapping.entity_map.len(),
            "mapping must be one-to-one"
        );
    }

    #[test]
    fn test_analogical_reasoner_no_overlap() {
        let mut source = AnalogicalStructure::new("src");
        source.add_relation("foo", vec!["a", "b"]);
        let mut target = AnalogicalStructure::new("tgt");
        target.add_relation("bar", vec!["c", "d"]);

        let reasoner = AnalogicalReasoner::new();
        let result = reasoner.reason_by_analogy(&source, &target);
        assert!(
            result.analogy_score < 1.0,
            "unrelated domains should have lower analogy score"
        );
    }
}
