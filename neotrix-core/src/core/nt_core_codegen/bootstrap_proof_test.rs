// G20 — Ne自举验证 (Ne Bootstrap Proof Test)
// Verifies that the generated Ne compiler:
//   1. Contains fn main() as entry point
//   2. Embeds all VSA primitives from the LanguageSpec
//   3. Embeds all cognitive subspaces from the LanguageSpec
//   4. Contains the identity check in the bootstrap proof
//   5. Round-trips the spec JSON through the embedded representation

use crate::core::nt_core_codegen::bridge::CodegenBridge;
use crate::core::nt_core_shared_types::{
    default_edit_policy, default_subspace_topology, default_vsa_primitives, EditPolicy,
    HandlerGraph, HandlerNode, LanguageSpec, SubspaceInfo, SubspaceMap, VsaPrimitive,
};

fn minimal_spec() -> LanguageSpec {
    LanguageSpec {
        vsa_primitives: default_vsa_primitives(),
        subspace_topology: default_subspace_topology(),
        edit_policy: default_edit_policy(),
        handler_graph: HandlerGraph { handlers: vec![] },
        confidence: 0.7,
        distilled_at: 0,
    }
}

fn spec_with_handlers_and_edit() -> LanguageSpec {
    LanguageSpec {
        vsa_primitives: default_vsa_primitives(),
        subspace_topology: default_subspace_topology(),
        edit_policy: default_edit_policy(),
        handler_graph: HandlerGraph {
            handlers: vec![
                HandlerNode {
                    name: "handle_attractor_dynamics",
                    interval_secs: 30,
                    call_count: 0,
                },
                HandlerNode {
                    name: "handle_curiosity",
                    interval_secs: 60,
                    call_count: 0,
                },
            ],
        },
        confidence: 0.85,
        distilled_at: 1718000000,
    }
}

fn minimal_spec_one_primitive_one_subspace() -> LanguageSpec {
    LanguageSpec {
        vsa_primitives: vec![VsaPrimitive {
            name: "bind",
            arity: 2,
            description: "XOR binding",
            subspace_requirements: vec![],
        }],
        subspace_topology: SubspaceMap {
            subspaces: vec![SubspaceInfo {
                name: "@self",
                tag: 0x01,
                field_count: 1,
                fields: vec!["vsa_tag"],
                operations: vec!["bind"],
            }],
        },
        edit_policy: EditPolicy {
            max_gain: 0.5,
            max_edits_per_cycle: 10,
            lifetime_cap: 100,
            required_gates: vec![],
            allowed_targets: vec!["inner_critic.relevance_threshold"],
        },
        handler_graph: HandlerGraph { handlers: vec![] },
        confidence: 0.7,
        distilled_at: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_has_main_function() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("fn main()"),
            "generated compiler must contain fn main()"
        );
    }

    #[test]
    fn test_bootstrap_embeds_all_primitives() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        for prim in &spec.vsa_primitives {
            assert!(
                code.contains(prim.name),
                "generated compiler must embed primitive '{}' from spec",
                prim.name
            );
        }
    }

    #[test]
    fn test_bootstrap_embeds_all_subspaces() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        for sub in &spec.subspace_topology.subspaces {
            assert!(
                code.contains(sub.name),
                "generated compiler must embed subspace '{}' from spec",
                sub.name
            );
        }
    }

    #[test]
    fn test_bootstrap_proof_identity_check() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_bootstrap_proof(&spec);
        assert!(
            code.contains("check identity"),
            "bootstrap proof must contain check identity"
        );
    }

    #[test]
    fn test_spec_json_round_trip_contains_top_level_keys() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("\"vsa_primitives\""),
            "embedded JSON must have vsa_primitives key"
        );
        assert!(
            code.contains("\"subspace_topology\""),
            "embedded JSON must have subspace_topology key"
        );
        assert!(
            code.contains("\"edit_policy\""),
            "embedded JSON must have edit_policy key"
        );
        assert!(
            code.contains("\"handler_graph\""),
            "embedded JSON must have handler_graph key"
        );
        assert!(
            code.contains("\"confidence\""),
            "embedded JSON must have confidence key"
        );
    }

    #[test]
    fn test_spec_json_round_trip_contains_edit_policy_fields() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("max_gain"),
            "embedded JSON must have max_gain"
        );
        assert!(
            code.contains("max_edits_per_cycle"),
            "embedded JSON must have max_edits_per_cycle"
        );
        assert!(
            code.contains("lifetime_cap"),
            "embedded JSON must have lifetime_cap"
        );
        assert!(
            code.contains("required_gates"),
            "embedded JSON must have required_gates"
        );
        assert!(
            code.contains("allowed_targets"),
            "embedded JSON must have allowed_targets"
        );
    }

    #[test]
    fn test_bootstrap_proof_contains_all_primitives() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_bootstrap_proof(&spec);
        for prim in &spec.vsa_primitives {
            assert!(
                code.contains(prim.name),
                "bootstrap proof must reference primitive '{}'",
                prim.name
            );
        }
    }

    #[test]
    fn test_bootstrap_proof_contains_all_subspaces() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_bootstrap_proof(&spec);
        for sub in &spec.subspace_topology.subspaces {
            assert!(
                code.contains(sub.name),
                "bootstrap proof must reference subspace '{}'",
                sub.name
            );
        }
    }

    #[test]
    fn test_bootstrap_proof_with_handlers() {
        let spec = spec_with_handlers_and_edit();
        let code = CodegenBridge::generate_ne_bootstrap_proof(&spec);
        assert!(
            code.contains("check identity"),
            "bootstrap proof must contain check identity"
        );
        for h in &spec.handler_graph.handlers {
            assert!(
                code.contains(h.name),
                "bootstrap proof must contain handler '{}'",
                h.name
            );
        }
        assert!(
            code.contains("max_gain"),
            "bootstrap proof must reference edit policy"
        );
        assert!(
            code.contains("max_edits"),
            "bootstrap proof must reference max edits"
        );
    }

    #[test]
    fn test_minimal_spec_one_primitive_compiles() {
        let spec = minimal_spec_one_primitive_one_subspace();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("fn main()"),
            "minimal spec compiler must have fn main()"
        );
        assert!(
            code.contains("bind"),
            "minimal spec must contain 'bind' primitive"
        );
        assert!(
            code.contains("@self"),
            "minimal spec must contain '@self' subspace"
        );
        assert!(
            code.contains("max_gain"),
            "minimal spec must contain edit policy"
        );
    }
}
