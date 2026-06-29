/// Self-inspection trait —意识体自省接口。
/// 任何子系统实现此 trait 即可被扫描为结构化语言规范 (LanguageSpec)，
/// 供 CodegenBridge 生成 Ne 编译器。

/// A VSA primitive operation identified by self-inspection.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VsaPrimitive {
    pub name: &'static str,
    /// Number of arguments (-1 = variable arity, like bundle).
    pub arity: isize,
    pub description: &'static str,
    pub subspace_requirements: Vec<&'static str>,
}

/// A cognitive subspace with its VSA tag and associated fields/operations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubspaceInfo {
    pub name: &'static str,
    pub tag: u8,
    pub field_count: usize,
    pub fields: Vec<&'static str>,
    pub operations: Vec<&'static str>,
}

/// Map of all cognitive subspaces in the current consciousness.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SubspaceMap {
    pub subspaces: Vec<SubspaceInfo>,
}

/// Edit policy — what edits are allowed and their safety constraints.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EditPolicy {
    pub max_gain: f64,
    pub max_edits_per_cycle: u32,
    pub lifetime_cap: u32,
    pub required_gates: Vec<&'static str>,
    pub allowed_targets: Vec<&'static str>,
}

/// A node in the handler call graph.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HandlerNode {
    pub name: &'static str,
    pub interval_secs: u64,
    pub call_count: u64,
}

/// Directed call graph of all consciousness handlers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HandlerGraph {
    pub handlers: Vec<HandlerNode>,
}

/// A structured language specification —
/// the bootstrap artifact that drives Ne compiler generation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LanguageSpec {
    pub vsa_primitives: Vec<VsaPrimitive>,
    pub subspace_topology: SubspaceMap,
    pub edit_policy: EditPolicy,
    pub handler_graph: HandlerGraph,
    pub confidence: f64,
    pub distilled_at: u64,
}

/// Default VSA primitives known to the NeoTrix consciousness core.
pub fn default_vsa_primitives() -> Vec<VsaPrimitive> {
    vec![
        VsaPrimitive {
            name: "bind",
            arity: 2,
            description: "XOR binding of two VSA vectors",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "hlb_bind",
            arity: 2,
            description: "Hadamard linear binding (arXiv:2410.22669)",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "bundle",
            arity: -1,
            description: "Majority-sum bundling of N VSA vectors",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "permute",
            arity: 2,
            description: "Cyclic shift by k positions",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "negate",
            arity: 1,
            description: "Bitwise NOT (0↔1 flip)",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "similarity",
            arity: 2,
            description: "Normalized Hamming similarity [0,1]",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "cosine",
            arity: 2,
            description: "Cosine similarity on raw byte vectors",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "hamming_distance",
            arity: 2,
            description: "Raw Hamming distance count",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "random_vector",
            arity: 0,
            description: "Generate random VSA vector",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "seeded_random",
            arity: 2,
            description: "Deterministic PRNG VSA vector",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "binarize",
            arity: 1,
            description: "Threshold at 128 → binary {0,1}",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "majority_bundle",
            arity: -1,
            description: "Per-dimension majority vote bundling",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "translate_direct",
            arity: 2,
            description: "Direct VSA bind/unbind translation lookup",
            subspace_requirements: vec!["@translate"],
        },
        VsaPrimitive {
            name: "translate_compositional",
            arity: 3,
            description: "Compositional word-by-word VSA translation",
            subspace_requirements: vec!["@translate"],
        },
        VsaPrimitive {
            name: "translate_refinement",
            arity: 2,
            description: "TEaR-style iterative translation refinement",
            subspace_requirements: vec!["@translate"],
        },
        // Sutra polynomial fuzzy logic primitives (arXiv:2605.20919)
        VsaPrimitive {
            name: "kleene_and",
            arity: 2,
            description: "Lagrange-interpolated Kleene AND (C^∞, exact on {-1,0,+1}^2)",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "kleene_or",
            arity: 2,
            description: "Lagrange-interpolated Kleene OR (C^∞, exact on {-1,0,+1}^2)",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "kleene_not",
            arity: 1,
            description: "Kleene NOT: negate fuzzy truth value",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "kleene_imply",
            arity: 2,
            description: "Material implication: OR(NOT(a), b)",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "kleene_iff",
            arity: 2,
            description: "Biconditional: AND(IMPLY(a,b), IMPLY(b,a))",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "is_true",
            arity: 1,
            description: "Check if fuzzy truth value > 0.5",
            subspace_requirements: vec![],
        },
        VsaPrimitive {
            name: "defuzzify",
            arity: 1,
            description: "Convert fuzzy truth to boolean (hard threshold at ±0.5)",
            subspace_requirements: vec![],
        },
    ]
}

/// Default cognitive subspaces in the NeoTrix consciousness core.
pub fn default_subspace_topology() -> SubspaceMap {
    SubspaceMap {
        subspaces: vec![
            SubspaceInfo {
                name: "@self",
                tag: 0x01,
                field_count: 3,
                fields: vec!["vsa_tag", "first_person_ref", "narrative_self"],
                operations: vec!["bind", "bundle"],
            },
            SubspaceInfo {
                name: "@world",
                tag: 0x02,
                field_count: 4,
                fields: vec!["vsa_tag", "user_input", "sensor_data", "web_content"],
                operations: vec!["bundle", "similarity"],
            },
            SubspaceInfo {
                name: "@spatial",
                tag: 0x03,
                field_count: 2,
                fields: vec!["position_vsa", "object_vsa"],
                operations: vec!["permute", "bind", "similarity"],
            },
            SubspaceInfo {
                name: "@episodic",
                tag: 0x04,
                field_count: 3,
                fields: vec!["time_anchor", "location_anchor", "content_vector"],
                operations: vec!["bind", "bundle"],
            },
            SubspaceInfo {
                name: "@goal",
                tag: 0x05,
                field_count: 2,
                fields: vec!["success_criteria", "execution_trace"],
                operations: vec!["similarity", "cosine"],
            },
            SubspaceInfo {
                name: "@physics",
                tag: 0x06,
                field_count: 7,
                fields: vec![
                    "density",
                    "elasticity",
                    "friction",
                    "mass",
                    "volume",
                    "temperature",
                    "phase",
                ],
                operations: vec!["bind", "bundle"],
            },
            SubspaceInfo {
                name: "@emotional",
                tag: 0x07,
                field_count: 3,
                fields: vec!["valence", "arousal", "dominance"],
                operations: vec!["bundle", "similarity"],
            },
            SubspaceInfo {
                name: "@translate",
                tag: 0x08,
                field_count: 4,
                fields: vec![
                    "source_lang_tag",
                    "target_lang_tag",
                    "source_vsa",
                    "target_vsa",
                ],
                operations: vec!["bind", "unbind", "similarity", "bundle"],
            },
        ],
    }
}

/// Default edit policy derived from DGM-H safety constraints.
pub fn default_edit_policy() -> EditPolicy {
    EditPolicy {
        max_gain: 0.5,
        max_edits_per_cycle: 20,
        lifetime_cap: 1000,
        required_gates: vec!["pace::commit_gain ≤ 0.5", "pace::false_positive ≤ 0.05"],
        allowed_targets: vec![
            "cognitive_load.thinking_budget",
            "emergent_reasoning.emergence_threshold",
            "emergent_reasoning.exploration_rate",
            "emergent_reasoning.learning_rate",
            "personality_matrix.plasticity",
            "valence_axis.valence",
            "valence_axis.arousal",
            "inner_critic.relevance_threshold",
            "inner_critic.consistency_threshold",
            "inner_critic.uncertainty_tolerance",
        ],
    }
}

/// Self-inspection trait — 意识体自省接口。
/// 实现此 trait 的子系统可以生成结构化语言规范，用于：
/// 1. SystemCard 自动生成
/// 2. Ne 编译器自举
/// 3. 能力清单审计
pub trait SelfInspectable {
    /// 枚举所有可用的 VSA 原语操作。
    fn primitive_inventory(&self) -> Vec<VsaPrimitive> {
        default_vsa_primitives()
    }

    /// 返回当前意识的所有认知子空间拓扑。
    fn subspace_topology(&self) -> SubspaceMap {
        default_subspace_topology()
    }

    /// 返回当前编辑策略和安全边界。
    fn edit_boundary(&self) -> EditPolicy {
        default_edit_policy()
    }

    /// 返回所有 handler 的调用图。
    fn handler_graph(&self) -> HandlerGraph;

    /// 蒸馏完整语言规范 — bootstrap 入口。
    fn distill_language_spec(&self) -> LanguageSpec {
        LanguageSpec {
            vsa_primitives: self.primitive_inventory(),
            subspace_topology: self.subspace_topology(),
            edit_policy: self.edit_boundary(),
            handler_graph: self.handler_graph(),
            confidence: 0.7,
            distilled_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    struct DummySubsystem;

    impl SelfInspectable for DummySubsystem {
        fn handler_graph(&self) -> HandlerGraph {
            HandlerGraph { handlers: vec![] }
        }
    }

    #[test]
    fn test_primitive_inventory() {
        let d = DummySubsystem;
        let p = d.primitive_inventory();
        assert!(!p.is_empty());
        assert!(p.iter().any(|p| p.name == "bind"));
        assert!(p.iter().any(|p| p.name == "bundle"));
    }

    #[test]
    fn test_subspace_topology() {
        let d = DummySubsystem;
        let s = d.subspace_topology();
        assert!(s.subspaces.iter().any(|s| s.name == "@self"));
        assert!(s.subspaces.iter().any(|s| s.name == "@world"));
        assert_eq!(s.subspaces.len(), 8);
    }

    #[test]
    fn test_edit_policy() {
        let d = DummySubsystem;
        let e = d.edit_boundary();
        assert!((e.max_gain - 0.5).abs() < 1e-9);
        assert_eq!(e.max_edits_per_cycle, 20);
        assert_eq!(e.lifetime_cap, 1000);
    }

    #[test]
    fn test_distill_language_spec() {
        let d = DummySubsystem;
        let spec = d.distill_language_spec();
        assert!(!spec.vsa_primitives.is_empty());
        assert!(!spec.subspace_topology.subspaces.is_empty());
        assert!(!spec.edit_policy.allowed_targets.is_empty());
        assert!((spec.confidence - 0.7).abs() < 1e-9);
    }

    #[test]
    fn test_default_primitives_cover_core_ops() {
        let prims = default_vsa_primitives();
        let names: Vec<&str> = prims.iter().map(|p| p.name).collect();
        for op in &[
            "bind",
            "bundle",
            "permute",
            "negate",
            "similarity",
            "cosine",
            "random_vector",
        ] {
            assert!(names.contains(op), "missing primitive: {}", op);
        }
    }

    #[test]
    fn test_handler_graph_empty_default() {
        struct Empty;
        impl SelfInspectable for Empty {
            fn handler_graph(&self) -> HandlerGraph {
                HandlerGraph { handlers: vec![] }
            }
        }
        let e = Empty;
        assert_eq!(e.handler_graph().handlers.len(), 0);
    }
}
