#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::nt_core_shared_types::{
        default_edit_policy, default_subspace_topology, default_vsa_primitives, HandlerGraph,
        LanguageSpec,
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

    #[test]
    fn test_generate_test_module_contains_functions() {
        let code = CodegenBridge::generate_test_module("test_gen", "test module");
        assert!(code.contains("pub fn test_gen_bundle("));
        assert!(code.contains("QuantizedVSA::bundle("));
        assert!(code.contains("test_gen_generated"));
    }

    #[test]
    fn test_generate_skill_test_contains_tests() {
        let code = CodegenBridge::generate_skill_test("my_skill", "test skill", 3);
        assert!(code.contains("fn my_skill_skill_0()"));
        assert!(code.contains("fn my_skill_skill_1()"));
        assert!(code.contains("fn my_skill_skill_2()"));
        assert!(code.contains("QuantizedVSA::seeded_random"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_generate_compiler_contains_main() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("fn main()"),
            "generated compiler must contain fn main()"
        );
    }

    #[test]
    fn test_generate_compiler_contains_embedded_spec() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_compiler(&spec);
        assert!(
            code.contains("bind"),
            "generated compiler must embed spec with primitive 'bind'"
        );
        assert!(
            code.contains("bundle"),
            "generated compiler must embed spec with primitive 'bundle'"
        );
    }

    #[test]
    fn test_generate_ne_bootstrap_proof_contains_identity() {
        let spec = minimal_spec();
        let code = CodegenBridge::generate_ne_bootstrap_proof(&spec);
        assert!(
            code.contains("check identity"),
            "bootstrap proof must contain check identity"
        );
    }

    #[test]
    fn test_stage0_seed_contains_main_and_primitives() {
        let seed = CodegenBridge::generate_stage0_seed();
        assert!(
            seed.contains("fn main()"),
            "Stage 0 seed must contain fn main"
        );
        assert!(
            seed.contains("QuantizedVSA::bind"),
            "must reference QuantizedVSA::bind"
        );
        assert!(seed.contains("HashMap"), "must use HashMap variable store");
        assert!(seed.contains("permute"), "must support permute");
        assert!(seed.contains("rotate_left"), "must support rotate_left");
        assert!(seed.contains("rotate_right"), "must support rotate_right");
        assert!(seed.contains("negate"), "must support negate");
        assert!(seed.contains("similarity"), "must support similarity");
        assert!(seed.contains("seeded_random"), "must support random");
        assert!(seed.contains("DIM"), "must define DIM constant");
        assert!(seed.contains("stage0"), "must mention stage0 in usage");
    }

    #[test]
    fn test_stage0_seed_is_self_contained() {
        let seed = CodegenBridge::generate_stage0_seed();
        // The generated code uses only std + neotrix-core imports
        assert!(seed.contains("use std::"), "must use std");
        assert!(seed.contains("use neotrix::"), "must import neotrix-core");
        // No external crate imports
        assert!(!seed.contains("use serde"), "no serde dependency");
        assert!(!seed.contains("use tokio"), "no tokio dependency");
    }

    #[test]
    fn test_transpile_simple_bind() {
        let code = CodegenBridge::transpile_stage1("(bind [1 2 3] [4 5 6])").unwrap();
        assert!(code.contains("pub fn stage1_main() -> String"));
        assert!(code.contains("QuantizedVSA::bind"));
        assert!(code.contains("&vec![1, 2, 3]"));
        assert!(code.contains("&vec![4, 5, 6]"));
        assert!(code.contains("QuantizedVSA"));
    }

    #[test]
    fn test_transpile_nested() {
        let code = CodegenBridge::transpile_stage1("(bind (bundle [1] [2]) (negate [3]))").unwrap();
        assert!(code.contains("QuantizedVSA::bind"));
        assert!(code.contains("QuantizedVSA::bundle"));
        assert!(code.contains("QuantizedVSA::negate"));
        assert!(code.contains("&vec![1]"));
        assert!(code.contains("&vec![2]"));
        assert!(code.contains("&vec![3]"));
    }

    #[test]
    fn test_transpile_let() {
        let code = CodegenBridge::transpile_stage1("(let x [1 2 3] (negate x))").unwrap();
        assert!(code.contains("let x = vec![1, 2, 3];"));
        assert!(code.contains("QuantizedVSA::negate(&x)"));
    }

    #[test]
    fn test_transpile_add() {
        let code = CodegenBridge::transpile_stage1("(+ 1 2)").unwrap();
        assert!(code.contains("1i64 + 2i64"));
        assert!(code.contains("pub fn stage1_main"));
    }

    #[test]
    fn test_generate_stage2_compiler_is_valid_ne() {
        let ne_source = CodegenBridge::generate_stage2_compiler();
        assert!(
            ne_source.len() > 0,
            "Stage 2 compiler source must be non-empty"
        );
        assert!(ne_source.contains("seq"), "Must contain seq construct");
        assert!(ne_source.contains("tokenize"), "Must define tokenize");
        assert!(ne_source.contains("parse"), "Must define parse");
        assert!(ne_source.contains("transpile"), "Must define transpile");
        assert!(ne_source.contains("compile"), "Must define compile");
    }

    #[test]
    fn test_stage2_transpiles_to_valid_rust() {
        let ne_source = CodegenBridge::generate_stage2_compiler();
        let result = CodegenBridge::transpile_stage1(&ne_source);
        assert!(
            result.is_ok(),
            "Stage 2 compiler must transpile successfully: {:?}",
            result.err()
        );
        let rust = result.unwrap();
        assert!(
            rust.contains("pub fn stage1_main"),
            "Must contain stage1_main entry point"
        );
        assert!(rust.contains("QuantizedVSA"), "Must reference QuantizedVSA");
    }

    #[test]
    fn test_generate_bootstrap_identity_test() {
        let test_code = CodegenBridge::generate_bootstrap_identity_test();
        assert!(
            test_code.contains("test_stage2_bootstrap_identity"),
            "Test must contain the identity test function"
        );
        assert!(
            test_code.contains("generate_stage2_compiler"),
            "Test must generate Stage 2 compiler"
        );
        assert!(
            test_code.contains("transpile_stage1"),
            "Test must transpile via Stage 1"
        );
        assert!(
            test_code.contains("generate_self_source_v2"),
            "Test must verify roundtrip"
        );
        assert!(
            test_code.contains("QuantizedVSA"),
            "Test must reference QuantizedVSA"
        );
    }

    #[test]
    fn test_stage2_bootstrap_identity_inline() {
        // Inline version of the bootstrap identity test
        let stage2_ne = CodegenBridge::generate_stage2_compiler();
        let stage2_rust = CodegenBridge::transpile_stage1(&stage2_ne).unwrap();
        assert!(
            stage2_rust.contains("pub fn stage1_main"),
            "Stage 2 compiler must transpile to valid Rust with stage1_main entry point"
        );
        let spec = minimal_spec();
        let self_source = CodegenBridge::generate_self_source_v2(&spec);
        let roundtrip = CodegenBridge::transpile_stage1(&self_source).unwrap();
        assert!(
            roundtrip.contains("QuantizedVSA"),
            "Roundtrip must reference QuantizedVSA"
        );
        assert!(stage2_ne.len() > 0, "Stage 2 Ne source must be non-empty");
        assert!(
            stage2_rust.len() > 0,
            "Stage 2 transpiled Rust must be non-empty"
        );
    }

    #[test]
    fn test_bootstrap_identity_v2() {
        let spec = minimal_spec();
        let ne_code = CodegenBridge::generate_self_source_v2(&spec);
        let transpiled = CodegenBridge::transpile_stage1(&ne_code).unwrap();
        assert!(
            transpiled.contains("pub fn stage1_main"),
            "transpiled output must contain stage1_main entry point"
        );
        assert!(
            transpiled.contains("QuantizedVSA"),
            "transpiled output must reference QuantizedVSA"
        );
        assert!(
            transpiled.contains("QuantizedVSA::bind"),
            "must transpile 'bind' to QuantizedVSA::bind"
        );
        assert!(
            transpiled.contains("QuantizedVSA::bundle"),
            "must transpile 'bundle' to QuantizedVSA::bundle"
        );
        assert!(
            transpiled.contains("QuantizedVSA::negate"),
            "must transpile 'negate' to QuantizedVSA::negate"
        );
    }

    #[test]
    fn test_generate_bootstrap_proof_v2() {
        let spec = minimal_spec();
        let proof = CodegenBridge::generate_bootstrap_proof_v2(&spec);
        assert!(
            proof.contains("stage2_bootstrap_test"),
            "must contain test module"
        );
        assert!(
            proof.contains("test_stage2_compiler_is_valid_ne"),
            "must contain validity test"
        );
        assert!(
            proof.contains("test_stage2_transpiles_to_rust"),
            "must contain transpile test"
        );
        assert!(
            proof.contains("test_bootstrap_identity_chain"),
            "must contain chain test"
        );
        assert!(
            proof.contains("generate_stage2_compiler"),
            "must reference compiler"
        );
        assert!(
            proof.contains("transpile_stage1"),
            "must reference transpiler"
        );
    }

    #[test]
    fn test_self_source_v2_bootstrap_identity() {
        let spec = minimal_spec();
        let self_source = CodegenBridge::generate_self_source_v2(&spec);

        // 1. Non-empty
        assert!(
            self_source.len() > 500,
            "SELF_SOURCE v2 must be substantial ({} bytes)",
            self_source.len()
        );

        // 2. Starts with valid Ne
        assert!(
            self_source.starts_with("(seq"),
            "SELF_SOURCE v2 must start with (seq"
        );

        // 3. Balanced parentheses
        let opens = self_source.chars().filter(|&c| c == '(').count();
        let closes = self_source.chars().filter(|&c| c == ')').count();
        assert_eq!(
            opens, closes,
            "SELF_SOURCE v2 must have balanced parens: {} open vs {} close",
            opens, closes
        );

        // 4. VSA primitives from spec are referenced
        for prim in &spec.vsa_primitives {
            let pname: &str = &prim.name[..];
            if matches!(
                pname,
                "bind"
                    | "bundle"
                    | "negate"
                    | "permute"
                    | "similarity"
                    | "rotate_left"
                    | "rotate_right"
            ) {
                assert!(
                    self_source.contains(pname),
                    "SELF_SOURCE v2 must reference primitive '{}'",
                    prim.name
                );
            }
        }

        // 5. transpile_stage1 produces valid Rust
        let transpiled = CodegenBridge::transpile_stage1(&self_source)
            .expect("transpile_stage1 must succeed on SELF_SOURCE v2");
        assert!(
            transpiled.contains("pub fn stage1_main"),
            "transpiled output must contain stage1_main entry point"
        );
        assert!(
            transpiled.contains("QuantizedVSA::"),
            "transpiled output must reference QuantizedVSA"
        );

        // 6. Transpiled Rust has structural validity
        assert!(
            transpiled.contains("fn stage1_main"),
            "transpiled must contain function definition"
        );
        assert!(
            transpiled.contains("use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA"),
            "transpiled must contain import"
        );
        assert!(
            transpiled.contains("QuantizedVSA::bind"),
            "must transpile bind calls"
        );
        assert!(
            transpiled.contains("QuantizedVSA::bundle"),
            "must transpile bundle calls"
        );
        assert!(
            transpiled.contains("QuantizedVSA::negate"),
            "must transpile negate calls"
        );
    }

    #[test]
    fn test_generate_from_neir() {
        let parsed = ne_surface::parse("(bind [1 2 3] [4 5 6])").unwrap();
        let result = CodegenBridge::generate_from_neir(&parsed).unwrap();
        assert!(
            result.contains("QuantizedVSA::bind"),
            "result must contain QuantizedVSA::bind: {}",
            result
        );
        assert!(
            result.contains("stage1_main"),
            "result must contain stage1_main: {}",
            result
        );
    }

    // ── Sutra rotation binding tests ──

    #[test]
    fn test_transpile_rotate_left() {
        let code = CodegenBridge::transpile_stage1("(rotate_left [1 2 3] 2)").unwrap();
        assert!(code.contains("QuantizedVSA::permute"));
        assert!(code.contains("&vec![1, 2, 3]"));
        assert!(code.contains("2"));
        assert!(code.contains("pub fn stage1_main"));
    }

    #[test]
    fn test_transpile_rotate_right() {
        let code = CodegenBridge::transpile_stage1("(rotate_right [1 2 3] 1)").unwrap();
        assert!(code.contains("QuantizedVSA::permute"));
        assert!(code.contains("vec![1, 2, 3]"));
        assert!(code.contains("-((1))"));
        assert!(code.contains("pub fn stage1_main"));
    }

    #[test]
    fn test_rotate_left_nested() {
        let code = CodegenBridge::transpile_stage1("(rotate_left (bundle [1] [2]) 3)").unwrap();
        assert!(code.contains("QuantizedVSA::permute"));
        assert!(code.contains("QuantizedVSA::bundle"));
        assert!(code.contains("&vec![1]"));
        assert!(code.contains("&vec![2]"));
    }

    #[test]
    fn test_tensor_module_contains_rotate_ops() {
        let module = CodegenBridge::generate_tensor_module("test_sutra");
        assert!(
            module.contains("rotate_left"),
            "tensor module must contain rotate_left"
        );
        assert!(
            module.contains("rotate_right"),
            "tensor module must contain rotate_right"
        );
        assert!(
            module.contains("rotate_bind"),
            "tensor module must contain rotate_bind"
        );
        assert!(
            module.contains("rotate_unbind"),
            "tensor module must contain rotate_unbind"
        );
        assert!(
            module.contains("bind_tensor"),
            "tensor module must contain bind_tensor"
        );
        assert!(
            module.contains("bundle_tensor"),
            "tensor module must contain bundle_tensor"
        );
        assert!(
            module.contains("keyed_rotate"),
            "tensor module must contain keyed_rotate"
        );
        assert!(
            module.contains("complex_bind"),
            "tensor module must contain complex_bind"
        );
        assert!(
            module.contains("Tensor::roll"),
            "must use tch::Tensor::roll for rotation"
        );
        assert!(module.contains("tch::"), "must reference tch crate");
    }

    #[test]
    fn test_tensor_module_test_section() {
        let module = CodegenBridge::generate_tensor_module("test");
        assert!(
            module.contains("test_rotation_vs_hadamard_different"),
            "tensor module must contain rotation vs hadamard test"
        );
        assert!(
            module.contains("test_rotate_left_shift"),
            "must contain rotate_left test"
        );
        assert!(
            module.contains("test_rotate_bind_unbind_roundtrip"),
            "must contain bind/unbind roundtrip test"
        );
    }

    #[test]
    fn test_rotation_vs_hadamard_test_file() {
        let test_code = CodegenBridge::generate_rotation_vs_hadamard_test();
        assert!(
            test_code.contains("sutra_rotation_tests"),
            "must contain test module"
        );
        assert!(
            test_code.contains("test_rotation_binding_differs_from_hadamard"),
            "must contain rotation vs hadamard test"
        );
        assert!(
            test_code.contains("rotate_bind_vsa"),
            "must define rotate_bind_vsa helper"
        );
        assert!(
            test_code.contains("QuantizedVSA::permute"),
            "must use permute for rotation"
        );
        assert!(
            test_code.contains("QuantizedVSA::bind"),
            "must reference bind for hadamard"
        );
        assert!(
            test_code.contains("test_rotation_binding_deterministic"),
            "must test determinism"
        );
        assert!(
            test_code.contains("test_rotation_binding_differs_by_key"),
            "must test key sensitivity"
        );
        assert!(
            test_code.contains("test_rotation_unbind_roundtrip"),
            "must test unbind roundtrip"
        );
    }

    #[test]
    fn test_tensor_module_backward_compat() {
        // Existing Rust codegen must remain unchanged
        let old_code = CodegenBridge::generate_test_module("backward", "test");
        assert!(old_code.contains("QuantizedVSA::bundle"));
        assert!(!old_code.contains("tch::"));
        assert!(!old_code.contains("Tensor"));
    }
}
