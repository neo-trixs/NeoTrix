use crate::core::nt_core_cap::CapabilityVector;
use crate::core::nt_core_traits::SpecialistType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A serializable harness behavior profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessProfile {
    pub source_model: String,
    pub environment_contracts: Vec<String>,
    pub procedural_skills: Vec<String>,
    pub action_realizations: Vec<String>,
    pub trajectory_rules: Vec<String>,
    pub specialist_adaptations: HashMap<SpecialistType, Vec<String>>,
    pub performance_delta: f64,
}

impl HarnessProfile {
    pub fn new(source_model: &str) -> Self {
        Self {
            source_model: source_model.to_string(),
            environment_contracts: Vec::new(),
            procedural_skills: Vec::new(),
            action_realizations: Vec::new(),
            trajectory_rules: Vec::new(),
            specialist_adaptations: HashMap::new(),
            performance_delta: 0.0,
        }
    }

    pub fn add_contract(&mut self, contract: &str) {
        self.environment_contracts.push(contract.to_string());
    }

    pub fn add_skill(&mut self, skill: &str) {
        self.procedural_skills.push(skill.to_string());
    }

    pub fn add_realization(&mut self, realization: &str) {
        self.action_realizations.push(realization.to_string());
    }

    pub fn add_rule(&mut self, rule: &str) {
        self.trajectory_rules.push(rule.to_string());
    }

    pub fn add_specialist_adaptation(&mut self, specialist: SpecialistType, adaptation: &str) {
        self.specialist_adaptations
            .entry(specialist)
            .or_default()
            .push(adaptation.to_string());
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(format!("Source model: {}", self.source_model));
        parts.push(format!(
            "Contracts ({}): {}",
            self.environment_contracts.len(),
            self.environment_contracts.join(", ")
        ));
        parts.push(format!(
            "Skills ({}): {}",
            self.procedural_skills.len(),
            self.procedural_skills.join(", ")
        ));
        parts.push(format!(
            "Realizations ({}): {}",
            self.action_realizations.len(),
            self.action_realizations.join(", ")
        ));
        parts.push(format!(
            "Trajectory rules ({}): {}",
            self.trajectory_rules.len(),
            self.trajectory_rules.join(", ")
        ));
        parts.push(format!("Performance delta: {}", self.performance_delta));
        parts.join("\n\n")
    }
}

/// Life-Harness inspired runtime adapter for cross-model harness transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessAdapter {
    pub profiles: HashMap<String, HarnessProfile>,
    pub active: Option<String>,
    pub transfer_history: Vec<(String, String, String, f64)>,
}

impl HarnessAdapter {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            active: None,
            transfer_history: Vec::new(),
        }
    }

    pub fn register_profile(&mut self, env: &str, profile: HarnessProfile) {
        self.profiles.insert(env.to_string(), profile);
    }

    pub fn activate(&mut self, env: &str) -> Option<&HarnessProfile> {
        if self.profiles.contains_key(env) {
            self.active = Some(env.to_string());
            self.profiles.get(env)
        } else {
            None
        }
    }

    pub fn active_profile(&self) -> Option<&HarnessProfile> {
        self.active.as_ref().and_then(|env| self.profiles.get(env))
    }

    pub fn transfer_to(
        &mut self,
        source_env: &str,
        target_model: &str,
        capability: &mut CapabilityVector,
    ) -> bool {
        let profile = match self.profiles.get(source_env) {
            Some(p) => p,
            None => return false,
        };
        apply_harness_evolution(profile, capability);
        self.transfer_history.push((
            profile.source_model.clone(),
            target_model.to_string(),
            source_env.to_string(),
            profile.performance_delta,
        ));
        true
    }

    /// 经可替换后端执行一次 harness 进化（Agent-Evolver 分离接口）。
    ///
    /// 默认后端 `DefaultEvolverBackend` 复刻 `transfer_to` 的既有数学；未来可传入
    /// 独立小模型进程后端（论文发现 harness-updating 与能力无关，evolver 可独立）。
    pub fn evolve_via(
        &mut self,
        backend: &dyn EvolverBackend,
        source_env: &str,
        target_model: &str,
        capability: &mut CapabilityVector,
    ) -> bool {
        let profile = match self.profiles.get(source_env) {
            Some(p) => p,
            None => return false,
        };
        let ok = backend.evolve_harness(profile, capability);
        if ok {
            self.transfer_history.push((
                profile.source_model.clone(),
                target_model.to_string(),
                source_env.to_string(),
                profile.performance_delta,
            ));
        }
        ok
    }

    pub fn record_transfer_result(&mut self, _env: &str, delta: f64) {
        if let Some(profile) = self.active.as_ref().and_then(|e| self.profiles.get_mut(e)) {
            profile.performance_delta = delta;
        }
    }
}

impl Default for HarnessAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// EvolverBackend — 将 harness 进化逻辑抽象为可替换后端（Agent-Evolver 分离）。
///
/// 论文发现 harness-updating 与模型能力无关（最佳/最差 evolver 差距 ≤3.1pp），
/// 因此 SEAL 的 evolver 可用独立小模型进程承载。本 trait 隔离该边界：
/// 默认 `DefaultEvolverBackend` 就地使用既有数学，未来可接独立进程后端而不动主流程。
pub trait EvolverBackend: Send + Sync {
    /// 用 `source` 画像演化 `target` 能力向量，返回是否产生有效演化。
    fn evolve_harness(&self, source: &HarnessProfile, target: &mut CapabilityVector) -> bool;
}

/// 默认 in-process 后端：复刻 `HarnessAdapter::transfer_to` 的既有数学（单一事实源在
/// [`apply_harness_evolution`]，被 `transfer_to` 与默认后端共用，避免平行副本）。
#[derive(Debug, Clone, Default)]
pub struct DefaultEvolverBackend;

impl EvolverBackend for DefaultEvolverBackend {
    fn evolve_harness(&self, source: &HarnessProfile, target: &mut CapabilityVector) -> bool {
        apply_harness_evolution(source, target)
    }
}

/// 既有 harness→能力演化数学（单一事实源）。
///
/// - `synthesis` 维度按 `performance_delta * 0.1` 提升（封顶 1.0）。
/// - `domain_specificity` 维度按 `procedural_skills.len() * 0.02` 提升（封顶 1.0）。
/// - 末尾归一化，保持能力向量合法。
pub fn apply_harness_evolution(source: &HarnessProfile, capability: &mut CapabilityVector) -> bool {
    if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
        let boost = source.performance_delta * 0.1;
        capability.arr_mut()[idx] = (capability.arr()[idx] + boost).min(1.0);
    }
    if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
        let boost = source.procedural_skills.len() as f64 * 0.02;
        capability.arr_mut()[idx] = (capability.arr()[idx] + boost).min(1.0);
    }
    capability.normalize();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile() -> HarnessProfile {
        let mut p = HarnessProfile::new("model-a");
        p.performance_delta = 0.5;
        p.add_skill("s1");
        p.add_skill("s2");
        p.add_skill("s3");
        p
    }

    #[test]
    fn test_default_backend_matches_transfer_to() {
        let profile = sample_profile();
        let mut adapter = HarnessAdapter::new();
        adapter.register_profile("env", profile.clone());

        let mut cap_a = CapabilityVector::default();
        adapter.transfer_to("env", "m1", &mut cap_a);

        let mut cap_b = CapabilityVector::default();
        let backend = DefaultEvolverBackend;
        let ok = backend.evolve_harness(&profile, &mut cap_b);

        assert!(ok);
        assert_eq!(cap_a.arr(), cap_b.arr(), "默认后端必须与 transfer_to 数学一致");
    }

    #[test]
    fn test_evolve_via_default_backend_records_history() {
        let mut adapter = HarnessAdapter::new();
        adapter.register_profile("env", sample_profile());
        let mut cap = CapabilityVector::default();
        let ok = adapter.evolve_via(&DefaultEvolverBackend, "env", "target-model", &mut cap);
        assert!(ok);
        assert_eq!(adapter.transfer_history.len(), 1);
        assert_eq!(adapter.transfer_history[0].1, "target-model");
    }

    #[test]
    fn test_evolve_via_unknown_env_returns_false() {
        let mut adapter = HarnessAdapter::new();
        let mut cap = CapabilityVector::default();
        let ok = adapter.evolve_via(&DefaultEvolverBackend, "missing", "m", &mut cap);
        assert!(!ok);
        assert!(adapter.transfer_history.is_empty());
    }

    #[test]
    fn test_pluggable_stub_backend() {
        struct StubEvolver;
        impl EvolverBackend for StubEvolver {
            fn evolve_harness(&self, _source: &HarnessProfile, target: &mut CapabilityVector) -> bool {
                if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
                    target.arr_mut()[idx] = (target.arr()[idx] + 0.1).min(1.0);
                }
                target.normalize();
                true
            }
        }
        let mut adapter = HarnessAdapter::new();
        adapter.register_profile("env", sample_profile());
        let mut cap = CapabilityVector::default();
        let ok = adapter.evolve_via(&StubEvolver, "env", "m", &mut cap);
        assert!(ok);
        if let Some(idx) = CapabilityVector::index_from_name("synthesis") {
            assert!((cap.arr()[idx] - 0.1).abs() < 1e-9, "stub 后端应只加 0.1");
        }
    }

    #[test]
    fn test_apply_harness_evolution_boosts_specificity_by_skill_count() {
        let mut p = HarnessProfile::new("m");
        p.performance_delta = 0.0;
        p.add_skill("a");
        p.add_skill("b");
        let mut cap = CapabilityVector::default();
        apply_harness_evolution(&p, &mut cap);
        if let Some(idx) = CapabilityVector::index_from_name("domain_specificity") {
            assert!((cap.arr()[idx] - 0.04).abs() < 1e-9);
        }
    }
}
