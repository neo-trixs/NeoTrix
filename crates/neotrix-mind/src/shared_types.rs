use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    General = 0,
    Design = 1,
    CodeAnalysis = 2,
    CodeGeneration = 3,
    CodeReview = 4,
    Security = 5,
    Planning = 6,
    Reflection = 7,
    UIDesign = 8,
    Research = 9,
    Learning = 10,
    Debugging = 11,
}

#[derive(Debug, Clone, Serialize)]
pub struct VsaPrimitive {
    pub name: &'static str,
    pub arity: isize,
    pub description: &'static str,
    pub subspace_requirements: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubspaceInfo {
    pub name: &'static str,
    pub tag: u8,
    pub field_count: usize,
    pub fields: Vec<&'static str>,
    pub operations: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubspaceMap {
    pub subspaces: Vec<SubspaceInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EditPolicy {
    pub max_gain: f64,
    pub max_edits_per_cycle: u32,
    pub lifetime_cap: u32,
    pub required_gates: Vec<&'static str>,
    pub allowed_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HandlerNode {
    pub name: &'static str,
    pub interval_secs: u64,
    pub call_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HandlerGraph {
    pub handlers: Vec<HandlerNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageSpec {
    pub vsa_primitives: Vec<VsaPrimitive>,
    pub subspace_topology: SubspaceMap,
    pub edit_policy: EditPolicy,
    pub handler_graph: HandlerGraph,
    pub confidence: f64,
    pub distilled_at: u64,
}

pub fn default_vsa_primitives() -> Vec<VsaPrimitive> {
    vec![
        VsaPrimitive { name: "bind", arity: 2, description: "XOR binding of two VSA vectors", subspace_requirements: vec![] },
        VsaPrimitive { name: "hlb_bind", arity: 2, description: "Hadamard linear binding (arXiv:2410.22669)", subspace_requirements: vec![] },
        VsaPrimitive { name: "bundle", arity: -1, description: "Majority-sum bundling of N VSA vectors", subspace_requirements: vec![] },
        VsaPrimitive { name: "permute", arity: 2, description: "Cyclic shift by k positions", subspace_requirements: vec![] },
        VsaPrimitive { name: "negate", arity: 1, description: "Bitwise NOT (0↔1 flip)", subspace_requirements: vec![] },
        VsaPrimitive { name: "similarity", arity: 2, description: "Normalized Hamming similarity [0,1]", subspace_requirements: vec![] },
        VsaPrimitive { name: "cosine", arity: 2, description: "Cosine similarity on raw byte vectors", subspace_requirements: vec![] },
        VsaPrimitive { name: "hamming_distance", arity: 2, description: "Raw Hamming distance count", subspace_requirements: vec![] },
        VsaPrimitive { name: "random_vector", arity: 0, description: "Generate random VSA vector", subspace_requirements: vec![] },
        VsaPrimitive { name: "seeded_random", arity: 2, description: "Deterministic PRNG VSA vector", subspace_requirements: vec![] },
        VsaPrimitive { name: "binarize", arity: 1, description: "Threshold at 128 → binary {0,1}", subspace_requirements: vec![] },
        VsaPrimitive { name: "majority_bundle", arity: -1, description: "Per-dimension majority vote bundling", subspace_requirements: vec![] },
        VsaPrimitive { name: "translate_direct", arity: 2, description: "Direct VSA bind/unbind translation lookup", subspace_requirements: vec!["@translate"] },
        VsaPrimitive { name: "translate_compositional", arity: 3, description: "Compositional word-by-word VSA translation", subspace_requirements: vec!["@translate"] },
        VsaPrimitive { name: "translate_refinement", arity: 2, description: "TEaR-style iterative translation refinement", subspace_requirements: vec!["@translate"] },
    ]
}

pub fn default_subspace_topology() -> SubspaceMap {
    SubspaceMap {
        subspaces: vec![
            SubspaceInfo { name: "@self", tag: 0x01, field_count: 3, fields: vec!["vsa_tag", "first_person_ref", "narrative_self"], operations: vec!["bind", "bundle"] },
            SubspaceInfo { name: "@world", tag: 0x02, field_count: 4, fields: vec!["vsa_tag", "user_input", "sensor_data", "web_content"], operations: vec!["bundle", "similarity"] },
            SubspaceInfo { name: "@spatial", tag: 0x03, field_count: 2, fields: vec!["position_vsa", "object_vsa"], operations: vec!["permute", "bind", "similarity"] },
            SubspaceInfo { name: "@episodic", tag: 0x04, field_count: 3, fields: vec!["time_anchor", "location_anchor", "content_vector"], operations: vec!["bind", "bundle"] },
            SubspaceInfo { name: "@goal", tag: 0x05, field_count: 2, fields: vec!["success_criteria", "execution_trace"], operations: vec!["similarity", "cosine"] },
            SubspaceInfo { name: "@physics", tag: 0x06, field_count: 7, fields: vec!["density", "elasticity", "friction", "gravity", "momentum", "collision", "contact"], operations: vec!["bind", "bundle", "similarity"] },
            SubspaceInfo { name: "@emotional", tag: 0x07, field_count: 8, fields: vec!["valence", "arousal", "drive_signal", "curiosity", "novelty", "certainty", "autonomy", "competence"], operations: vec!["bundle", "similarity", "cosine"] },
            SubspaceInfo { name: "@temporal", tag: 0x08, field_count: 3, fields: vec!["time_anchor", "context", "prediction"], operations: vec!["bind", "bundle", "permute"] },
        ],
    }
}

pub fn default_edit_policy() -> EditPolicy {
    EditPolicy {
        max_gain: 0.5,
        max_edits_per_cycle: 20,
        lifetime_cap: 1000,
        required_gates: vec!["safety_gate", "pcc_safety", "ball_verifier"],
        allowed_targets: vec![
            "handler", "prompt_template", "pipeline_step", "skill_crystal",
            "reasoning_strategy", "activation_threshold", "curiosity_temperature",
            "negentropy_weight", "consolidation_gamma",
            "emergent_reasoning.exploration_rate", "emergent_reasoning.learning_rate",
            "personality_matrix.plasticity", "valence_axis.valence",
            "valence_axis.arousal", "inner_critic.relevance_threshold",
            "inner_critic.consistency_threshold", "inner_critic.uncertainty_tolerance",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_variants() {
        assert_eq!(TaskType::General as isize, 0);
        assert_eq!(TaskType::CodeGeneration as isize, 3);
        assert_eq!(TaskType::Research as isize, 9);
    }

    #[test]
    fn test_default_primitives_cover_core_ops() {
        let prims = default_vsa_primitives();
        let names: Vec<&str> = prims.iter().map(|p| p.name).collect();
        for op in &["bind", "bundle", "permute", "negate", "similarity", "cosine", "random_vector"] {
            assert!(names.contains(op), "missing primitive: {}", op);
        }
    }

    #[test]
    fn test_default_subspace_topology_has_self_world() {
        let s = default_subspace_topology();
        assert!(s.subspaces.iter().any(|s| s.name == "@self"));
        assert!(s.subspaces.iter().any(|s| s.name == "@world"));
        assert_eq!(s.subspaces.len(), 8);
    }

    #[test]
    fn test_default_edit_policy_values() {
        let e = default_edit_policy();
        assert!((e.max_gain - 0.5).abs() < 1e-9);
        assert_eq!(e.max_edits_per_cycle, 20);
        assert_eq!(e.lifetime_cap, 1000);
    }

    #[test]
    fn test_language_spec_roundtrip() {
        let spec = LanguageSpec {
            vsa_primitives: default_vsa_primitives(),
            subspace_topology: default_subspace_topology(),
            edit_policy: default_edit_policy(),
            handler_graph: HandlerGraph { handlers: vec![] },
            confidence: 0.85,
            distilled_at: 1_000_000,
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("vsa_primitives"));
        assert!(json.contains("subspace_topology"));
    }
}
