use super::types::ConsciousnessTree;

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessTree {
    fn name(&self) -> &str {
        "consciousness_tree"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        self.self_test()
    }
}
