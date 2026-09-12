//! SEAL 进化维度补 instincts / security（吸收 affaan-m/ECC harness perf 维度:
//! skills / instincts / memory / security / research）。
//!
//! R-P42 强化既有 `CapabilityVector`：固定 23 维之外，用其扩展维度机制承载 ECC 维度，
//! 避免平行重造。base 维的 `index_from_name` 不覆盖扩展维，故另提供扩展维专用的
//! 读写/演化 API，使 SEAL 自进化循环可定向调优这些维度而不产生死代码。

use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_knowledge::TaskType;
use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// affaan-m/ECC 吸收的 SEAL 进化维度。
pub const SEAL_ECC_DIMENSIONS: &[&str] = &[
    "skills", "instincts", "memory", "security", "research",
];

/// 将 ECC 维度注册为 `CapabilityVector` 扩展维度（upsert，安全可重复调用）。
pub fn register_ecc_dimensions(cv: &mut CapabilityVector) {
    for &d in SEAL_ECC_DIMENSIONS {
        cv.add_extension_dim(d, 0.5);
    }
}

/// 读取某 ECC 维度当前值（扩展维度，不参与 base `index_from_name`）。
pub(crate) fn _ecc_dimension_value(cv: &CapabilityVector, name: &str) -> Option<f64> {
    let names = cv.extension_names();
    let values = cv.extension_values();
    for (n, v) in names.iter().zip(values.iter()) {
        if *n == name {
            return Some(*v);
        }
    }
    None
}

/// 演化某 ECC 维度（SEAL 自进化调用点，避免依赖 base `index_from_name`）。
pub fn evolve_ecc_dimension(cv: &mut CapabilityVector, name: &str, delta: f64) {
    let cur = _ecc_dimension_value(cv, name).unwrap_or(0.5);
    let next = (cur + delta).clamp(0.0, 1.0);
    cv.add_extension_dim(name, next);
}

/// 针对任务类型选取 ECC 相关维度，供 SEAL 定向进化。
pub fn select_ecc_dimensions(task_type: &TaskType) -> Vec<String> {
    match task_type {
        TaskType::CodeAnalysis | TaskType::CodeGeneration => {
            SEAL_ECC_DIMENSIONS.iter().map(|s| s.to_string()).collect()
        }
        TaskType::Design | TaskType::UIDesign => {
            vec!["skills".into(), "instincts".into(), "research".into()]
        }
        _ => vec!["skills".into(), "instincts".into(), "security".into()],
    }
}

pub(crate) struct _SealEccSelfTest;

impl SelfTest for _SealEccSelfTest {
    fn name(&self) -> &str {
        "seal_ecc_dimensions"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut cv = CapabilityVector::default();
        register_ecc_dimensions(&mut cv);
        if cv.total_dim() != 23 + SEAL_ECC_DIMENSIONS.len() {
            return Err(vec![format!("total_dim mismatch: {}", cv.total_dim())]);
        }
        for d in SEAL_ECC_DIMENSIONS {
            if _ecc_dimension_value(&cv, d).is_none() {
                return Err(vec![format!("missing ecc dim {d}")]);
            }
        }
        let selected = select_ecc_dimensions(&TaskType::General);
        if selected.is_empty() {
            return Err(vec!["select_ecc_dimensions returned empty".into()]);
        }
        evolve_ecc_dimension(&mut cv, "security", 0.2);
        let after = _ecc_dimension_value(&cv, "security")
            .ok_or_else(|| vec!["security lost after evolve".to_string()])?;
        if (after - 0.7).abs() > 1e-9 {
            return Err(vec![format!("evolve wrong: {after}")]);
        }
        Ok(())
    }
}

/// 注册 SEAL ECC 维度 SelfTest 到全局注册表 (T2)。
pub fn register_seal_ecc_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(_SealEccSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_ecc_self_test_passes() {
        assert!(
            _SealEccSelfTest.self_test().is_ok(),
            "seal ecc self_test failed: {:?}",
            _SealEccSelfTest.self_test().err()
        );
    }
}
