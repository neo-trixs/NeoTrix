//! B-Rep Topology Validation SelfTest
//!
//! Validates that generated CAD programs produce valid water-tight B-rep models
//! using the native Rust geometry kernel (build_mesh + validate_water_tight).
//! Integrates with GenCAD's four-step framework: CSR→CCIP→CDP→Decoder→Geometry Kernel.

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};
use crate::neotrix::l2_world_impl::cad_generator::{
    build_mesh, validate_water_tight, CadGenerator, GenerationInput,
};

/// T3 检查: B-Rep 拓扑验证
///
/// 真实管线: `CadGenerator` 生成命令序列 → 几何内核 `build_mesh` 构建三角网格
/// → `validate_water_tight` 做 water-tight (流形) 校验. 不再是占位.
#[derive(Default)]
pub struct CadBRepTopologySelfTest;

impl SelfTest for CadBRepTopologySelfTest {
    fn name(&self) -> &str {
        "cad_brep_topology"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let gen = CadGenerator::new();
        let cmds = gen
            .generate(&GenerationInput::Features(vec![0.9, 0.2, 0.7, 0.1, 0.4, 0.8]))
            .map_err(|e| vec![format!("cad_brep_topology: generate: {e}")])?;

        let mesh = build_mesh(&cmds);
        let validation = validate_water_tight(&mesh);

        let mut failures = Vec::new();
        if validation.triangle_count == 0 {
            failures.push("cad_brep_topology: empty mesh (no geometry produced)".into());
        }
        if !validation.water_tight {
            failures.push(format!(
                "cad_brep_topology: mesh not water-tight (non-manifold edges = {})",
                validation.non_manifold_edges
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// Register B-Rep topology validation SelfTest
pub fn register_cad_brep_topology_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(CadBRepTopologySelfTest::default()));
}
