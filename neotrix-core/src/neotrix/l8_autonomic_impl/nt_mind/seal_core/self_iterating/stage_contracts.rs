use super::pipeline::{BrainStage, StageDecision};
use crate::neotrix::nt_core_error::NeoTrixError;
use super::SelfIteratingBrain;

/// Severity of a contract violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractSeverity {
    /// Informational — logged but ignored
    Suggestion,
    /// Warning — logged, potentially blocks in strict mode
    Warning,
    /// Error — always blocks execution
    Error,
}

/// A typed contract violation with diagnostic context.
#[derive(Debug, Clone)]
pub struct ContractViolation {
    pub stage_name: String,
    pub severity: ContractSeverity,
    pub message: String,
}

/// How the enforcer handles violations at each severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnforcementPolicy {
    pub suggestion: ActionOnViolation,
    pub warning: ActionOnViolation,
    pub error: ActionOnViolation,
}

impl Default for EnforcementPolicy {
    fn default() -> Self {
        Self {
            suggestion: ActionOnViolation::Log,
            warning: ActionOnViolation::Log,
            error: ActionOnViolation::Block,
        }
    }
}

impl EnforcementPolicy {
    pub fn strict() -> Self {
        Self {
            suggestion: ActionOnViolation::Log,
            warning: ActionOnViolation::Block,
            error: ActionOnViolation::Block,
        }
    }

    pub fn permissive() -> Self {
        Self {
            suggestion: ActionOnViolation::Ignore,
            warning: ActionOnViolation::Log,
            error: ActionOnViolation::Log,
        }
    }

    pub fn action_for(&self, severity: ContractSeverity) -> ActionOnViolation {
        match severity {
            ContractSeverity::Suggestion => self.suggestion,
            ContractSeverity::Warning => self.warning,
            ContractSeverity::Error => self.error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionOnViolation {
    Ignore,
    Log,
    Block,
}

/// A contract that a pipeline stage may optionally satisfy.
///
/// Contracts are checked before stage execution (pre), after (post),
/// and continuously (invariant). A stage can implement one or more
/// contracts to make its behavior predictable and verifiable.
pub trait StageContract: Send + Sync {
    fn name(&self) -> &str;
    fn pre_check(&self, brain: &SelfIteratingBrain) -> Vec<ContractViolation>;
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation>;
    fn invariant_check(&self, brain: &SelfIteratingBrain) -> Vec<ContractViolation>;
}

/// Snapshot of brain state before a stage runs, used for post-check diffing.
#[derive(Debug, Clone)]
pub struct StageCheckpoint {
    pub iteration: u64,
    pub reward: f64,
    pub champion_score: Option<f64>,
    pub tool_call_count: usize,
    pub edit_count: usize,
    pub memory_count: usize,
}

impl StageCheckpoint {
    pub fn capture(brain: &SelfIteratingBrain) -> Self {
        Self {
            iteration: brain.iteration,
            reward: brain._reward,
            champion_score: brain.champion.as_ref().map(|c| c.score),
            tool_call_count: brain.tool_call_count,
            edit_count: brain._micro_edits.len(),
            memory_count: brain.reasoning_bank.memories().len(),
        }
    }
}

// ============================================================
// Built-in Contracts
// ============================================================

/// Ensures the stage does not modify iteration or reward (read-only check).
pub struct ReadOnlyContract;

impl StageContract for ReadOnlyContract {
    fn name(&self) -> &str { "read_only" }
    fn pre_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        let mut violations = Vec::new();
        if brain.iteration != before.iteration {
            violations.push(ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Error,
                message: format!("read_only contract violated: iteration changed from {} to {}", before.iteration, brain.iteration),
            });
        }
        if brain._reward != before.reward {
            violations.push(ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Warning,
                message: format!("read_only contract violated: reward changed from {} to {}", before.reward, brain._reward),
            });
        }
        violations
    }
    fn invariant_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
}

/// Ensures capability does not degrade significantly.
pub struct CapabilityStableContract {
    pub max_degradation: f64,
}

impl Default for CapabilityStableContract {
    fn default() -> Self { Self { max_degradation: 0.05 } }
}

impl StageContract for CapabilityStableContract {
    fn name(&self) -> &str { "capability_stable" }
    fn pre_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        let mut violations = Vec::new();
        if let (Some(before_score), Some(after_score)) = (before.champion_score, brain.champion.as_ref().map(|c| c.score)) {
            if before_score > 0.0 {
                let degradation = (before_score - after_score) / before_score;
                if degradation > self.max_degradation {
                    violations.push(ContractViolation {
                        stage_name: String::new(),
                        severity: ContractSeverity::Warning,
                        message: format!("capability degraded by {:.1}% (max allowed {:.1}%)", degradation * 100.0, self.max_degradation * 100.0),
                    });
                }
            }
        }
        violations
    }
    fn invariant_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
}

/// Ensures a stage stays within a resource budget.
pub struct ResourceBudgetContract {
    pub max_tool_calls_per_run: usize,
}

impl Default for ResourceBudgetContract {
    fn default() -> Self { Self { max_tool_calls_per_run: 5 } }
}

impl StageContract for ResourceBudgetContract {
    fn name(&self) -> &str { "resource_budget" }
    fn pre_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        let calls_used = brain.tool_call_count.saturating_sub(before.tool_call_count);
        if calls_used > self.max_tool_calls_per_run {
            vec![ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Warning,
                message: format!("used {} tool calls (budget {})", calls_used, self.max_tool_calls_per_run),
            }]
        } else {
            vec![]
        }
    }
    fn invariant_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
}

/// Ensures iteration number progresses monotonically (no rewinding).
pub struct MonotonicIterationContract;

impl StageContract for MonotonicIterationContract {
    fn name(&self) -> &str { "monotonic_iteration" }
    fn pre_check(&self, brain: &SelfIteratingBrain) -> Vec<ContractViolation> {
        if brain.iteration == 0 {
            return vec![];
        }
        let mut violations = Vec::new();
        if let Some(last) = brain._stage_results.last() {
            if last.efc.is_nan() || last.efficiency.is_nan() {
                violations.push(ContractViolation {
                    stage_name: String::new(),
                    severity: ContractSeverity::Warning,
                    message: format!("previous stage '{}' produced NaN values", last.stage_name),
                });
            }
        }
        violations
    }
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        if brain.iteration < before.iteration {
            vec![ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Error,
                message: format!("iteration regressed from {} to {}", before.iteration, brain.iteration),
            }]
        } else {
            vec![]
        }
    }
    fn invariant_check(&self, brain: &SelfIteratingBrain) -> Vec<ContractViolation> {
        if brain.iteration > 0 && brain._reward.is_nan() {
            vec![ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Error,
                message: "reward is NaN".to_string(),
            }]
        } else {
            vec![]
        }
    }
}

/// A stage that must not add micro-edits (no-self-modification).
pub struct NoSelfEditContract;

impl StageContract for NoSelfEditContract {
    fn name(&self) -> &str { "no_self_edit" }
    fn pre_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        let edits_before = before.edit_count;
        let edits_after = brain._micro_edits.len();
        if edits_after > edits_before {
            vec![ContractViolation {
                stage_name: String::new(),
                severity: ContractSeverity::Error,
                message: format!("stage generated {} micro-edits despite no_self_edit contract", edits_after.saturating_sub(edits_before)),
            }]
        } else {
            vec![]
        }
    }
    fn invariant_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
}

// ============================================================
// Delivery Promise Contract (P0-2 吸收自 OpenMontage delivery_promise.py)
// ============================================================

/// 交付承诺类型 — 对应 OpenMontage `PromiseType`。
/// 阶段执行前声明"承诺交付什么"，执行后验证是否兑现，禁止静默降级。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromiseType {
    /// 承诺真实能力提升：champion_score 必须提升（对应 OpenMontage MOTION_LED：
    /// 只有真实运动算数，动画幻灯片不算）。reward 提升但 champion 未动 = 表面提升。
    CapabilityLed,
    /// 承诺过程推进：reward 提升即可，能力可持平（对应 SOURCE_LED/DATA_EXPLAINER）。
    ProgressLed,
}

/// 交付承诺 — 在阶段执行前声明，执行后由 `DeliveryPromiseContract` 验证。
#[derive(Debug, Clone)]
pub struct DeliveryPromise {
    pub promise_type: PromiseType,
    /// 最低可接受质量（对应 OpenMontage `min_motion_ratio`）。
    /// CapabilityLed 下为 champion_score 最低提升比例；ProgressLed 下为 reward 最低提升比例。
    pub quality_floor: f64,
    /// 是否允许静默降级（对应 OpenMontage `still_fallback_allowed`）。
    /// false = 产出低于承诺必须 Block，不得静默替换。
    pub fallback_allowed: bool,
}

impl DeliveryPromise {
    pub fn capability_led(quality_floor: f64) -> Self {
        Self { promise_type: PromiseType::CapabilityLed, quality_floor, fallback_allowed: false }
    }

    pub fn progress_led(quality_floor: f64) -> Self {
        Self { promise_type: PromiseType::ProgressLed, quality_floor, fallback_allowed: true }
    }
}

/// 防降级契约 — 验证阶段产出是否兑现交付承诺。
/// 对应 OpenMontage `DeliveryPromise.validate_cuts()`：给"禁止静默降级"下可判定定义。
pub struct DeliveryPromiseContract {
    pub promise: DeliveryPromise,
}

impl StageContract for DeliveryPromiseContract {
    fn name(&self) -> &str { "delivery_promise" }
    fn pre_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }

    fn post_check(&self, brain: &SelfIteratingBrain, before: &StageCheckpoint) -> Vec<ContractViolation> {
        let mut violations = Vec::new();
        match self.promise.promise_type {
            PromiseType::CapabilityLed => {
                // 真实能力提升：champion_score 必须提升。
                // reward 提升但 champion 未动 = 表面提升（对应"动画幻灯片不算运动"）。
                match (before.champion_score, brain.champion.as_ref().map(|c| c.score)) {
                    (Some(before_score), Some(after_score)) => {
                        let improvement = if before_score > 0.0 {
                            (after_score - before_score) / before_score
                        } else {
                            after_score - before_score
                        };
                        if improvement < self.promise.quality_floor {
                            let msg = format!(
                                "delivery_promise (CapabilityLed): champion improvement {:.1}% below floor {:.1}% — reward may have risen but real capability did not (surface improvement, not real motion)",
                                improvement * 100.0, self.promise.quality_floor * 100.0
                            );
                            violations.push(ContractViolation {
                                stage_name: String::new(),
                                severity: if self.promise.fallback_allowed { ContractSeverity::Warning } else { ContractSeverity::Error },
                                message: msg,
                            });
                        }
                    }
                    (Some(_), None) => {
                        // champion 丢失 = 能力回退，永远不允许静默降级
                        violations.push(ContractViolation {
                            stage_name: String::new(),
                            severity: ContractSeverity::Error,
                            message: "delivery_promise (CapabilityLed): champion was lost — capability regression, silent fallback not allowed".to_string(),
                        });
                    }
                    (None, None) => {
                        // 无 champion 基线：无法验证能力提升，按承诺类型处理
                        if !self.promise.fallback_allowed {
                            violations.push(ContractViolation {
                                stage_name: String::new(),
                                severity: ContractSeverity::Warning,
                                message: "delivery_promise (CapabilityLed): no champion baseline to verify against — cannot confirm capability delivery".to_string(),
                            });
                        }
                    }
                    (None, Some(_)) => {}
                }
            }
            PromiseType::ProgressLed => {
                // 过程推进：reward 提升即可
                if brain._reward < before.reward {
                    violations.push(ContractViolation {
                        stage_name: String::new(),
                        severity: ContractSeverity::Warning,
                        message: format!(
                            "delivery_promise (ProgressLed): reward regressed from {:.3} to {:.3}",
                            before.reward, brain._reward
                        ),
                    });
                }
            }
        }
        violations
    }

    fn invariant_check(&self, _brain: &SelfIteratingBrain) -> Vec<ContractViolation> { vec![] }
}

// ============================================================
// Contract Registry
// ============================================================

/// Maps stage names to their contracts.
#[derive(Default)]
pub struct ContractRegistry {
    contracts: Vec<(String, Box<dyn StageContract>)>,
    pub policy: EnforcementPolicy,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: Vec::new(),
            policy: EnforcementPolicy::default(),
        }
    }

    pub fn register(&mut self, stage_name: &str, contract: Box<dyn StageContract>) {
        self.contracts.push((stage_name.to_string(), contract));
    }

    pub fn contracts_for(&self, stage_name: &str) -> Vec<&dyn StageContract> {
        self.contracts.iter()
            .filter(|(name, _)| name == stage_name)
            .map(|(_, contract)| contract.as_ref())
            .collect()
    }

    pub fn all_contracts(&self) -> &[(String, Box<dyn StageContract>)] {
        &self.contracts
    }

    pub fn check_stage(&self, stage_name: &str, brain: &SelfIteratingBrain, checkpoint: &Option<StageCheckpoint>) -> Vec<ContractViolation> {
        let mut all = Vec::new();
        for contract in self.contracts_for(stage_name) {
            if let Some(ref cp) = checkpoint {
                all.extend(contract.post_check(brain, cp));
            } else {
                all.extend(contract.pre_check(brain));
            }
            all.extend(contract.invariant_check(brain));
        }
        all
    }

    pub fn has_any_violations(&self, violations: &[ContractViolation]) -> bool {
        violations.iter().any(|v| {
            matches!(self.policy.action_for(v.severity), ActionOnViolation::Block)
        })
    }
}

// ============================================================
// Contract-Aware Stage Wrapper
// ============================================================

/// Wraps a `BrainStage` with contract enforcement.
pub struct ContractAwareStage {
    inner: Box<dyn BrainStage>,
    contracts: Vec<Box<dyn StageContract>>,
}

impl ContractAwareStage {
    pub fn new(inner: Box<dyn BrainStage>) -> Self {
        Self {
            inner,
            contracts: Vec::new(),
        }
    }

    pub fn with_contract(mut self, contract: Box<dyn StageContract>) -> Self {
        self.contracts.push(contract);
        self
    }
}

impl BrainStage for ContractAwareStage {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn frequency(&self) -> usize {
        self.inner.frequency()
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let checkpoint = StageCheckpoint::capture(brain);

        // Pre-checks
        let mut violations = Vec::new();
        for contract in &self.contracts {
            violations.extend(contract.pre_check(brain));
            violations.extend(contract.invariant_check(brain));
        }

        // Block if pre-checks fail
        let policy = ContractRegistry::new().policy;
        for v in &violations {
            match policy.action_for(v.severity) {
                ActionOnViolation::Ignore => {}
                ActionOnViolation::Log => {
                    log::warn!("[contract] pre-check FAIL: stage={} contract={} msg={:?}",
                        self.name(), contract_name_for(&self.contracts), v.message);
                }
                ActionOnViolation::Block => {
                    log::error!("[contract] pre-check BLOCK: stage={} contract={} msg={:?}",
                        self.name(), contract_name_for(&self.contracts), v.message);
                    return Err(NeoTrixError::Brain(format!(
                        "Contract violation in '{}': {}", self.name(), v.message
                    )));
                }
            }
        }

        // Execute the inner stage
        let result = self.inner.process(brain);

        // Post-checks
        let mut post_violations = Vec::new();
        for contract in &self.contracts {
            post_violations.extend(contract.post_check(brain, &checkpoint));
            post_violations.extend(contract.invariant_check(brain));
        }

        for v in &post_violations {
            match policy.action_for(v.severity) {
                ActionOnViolation::Ignore => {}
                ActionOnViolation::Log => {
                    log::warn!("[contract] post-check FAIL: stage={} msg={:?}", self.name(), v.message);
                }
                ActionOnViolation::Block => {
                    log::error!("[contract] post-check BLOCK: stage={} msg={:?}", self.name(), v.message);
                    return Err(NeoTrixError::Brain(format!(
                        "Contract violation (post) in '{}': {}", self.name(), v.message
                    )));
                }
            }
        }

        result
    }
}

fn contract_name_for(contracts: &[Box<dyn StageContract>]) -> String {
    contracts.first().map_or("unknown".to_string(), |c| c.name().to_string())
}

// ============================================================
// Built-in contract presets
// ============================================================

/// Pre-built contracts for common stage categories.
pub mod presets {
    use super::*;

    /// A stage that should not modify brain state (e.g. diagnostic, monitoring).
    pub fn read_only() -> Vec<Box<dyn StageContract>> {
        vec![
            Box::new(ReadOnlyContract),
            Box::new(NoSelfEditContract),
            Box::new(ResourceBudgetContract { max_tool_calls_per_run: 1 }),
        ]
    }

    /// A stage that may modify state but must keep capability stable.
    pub fn safe_mutation() -> Vec<Box<dyn StageContract>> {
        vec![
            Box::new(CapabilityStableContract::default()),
            Box::new(ResourceBudgetContract::default()),
            Box::new(MonotonicIterationContract),
        ]
    }

    /// A stage that performs self-modification (DGM-H etc) — minimal restrictions.
    pub fn self_modifying() -> Vec<Box<dyn StageContract>> {
        vec![
            Box::new(ResourceBudgetContract { max_tool_calls_per_run: 10 }),
            Box::new(MonotonicIterationContract),
        ]
    }

    /// A stage that promises real capability improvement — silent degradation blocked.
    /// (P0-2 吸收自 OpenMontage delivery_promise: 防静默降级)
    pub fn capability_led(quality_floor: f64) -> Vec<Box<dyn StageContract>> {
        vec![
            Box::new(DeliveryPromiseContract { promise: DeliveryPromise::capability_led(quality_floor) }),
            Box::new(MonotonicIterationContract),
        ]
    }
}

/// Convenience method to wrap a stage with a preset contract bundle.
pub fn with_contract_preset(stage: Box<dyn BrainStage>, preset: Vec<Box<dyn StageContract>>) -> ContractAwareStage {
    let mut wrapped = ContractAwareStage::new(stage);
    for c in preset {
        wrapped = wrapped.with_contract(c);
    }
    wrapped
}

#[cfg(test)]
#[allow(dead_code)] // test utilities shared across test functions
mod tests {
    use super::*;
    use crate::make_stage;

    fn dummy_brain() -> SelfIteratingBrain {
        SelfIteratingBrain::new()
    }

    #[test]
    fn test_read_only_contract_passes() {
        make_stage!(DummyStage);
        impl BrainStage for DummyStage {
            fn name(&self) -> &str { "dummy" }
            fn process(&self, _brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                Ok(StageDecision::Continue)
            }
        }

        let brain = dummy_brain();
        let before = StageCheckpoint::capture(&brain);
        let contract = ReadOnlyContract;
        let violations = contract.post_check(&brain, &before);
        assert!(violations.is_empty(), "read-only contract should pass when state unchanged");
    }

    #[test]
    fn test_read_only_contract_catches_iteration_change() {
        make_stage!(MutatingStage);
        impl BrainStage for MutatingStage {
            fn name(&self) -> &str { "mutator" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain.iteration += 1;
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        let before = StageCheckpoint::capture(&brain);
        let _ = MutatingStage.process(&mut brain).unwrap();
        let contract = ReadOnlyContract;
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "read-only contract should catch iteration change");
    }

    #[test]
    fn test_capability_stable_contract() {
        make_stage!(DegradingStage);
        impl BrainStage for DegradingStage {
            fn name(&self) -> &str { "degrader" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                if let Some(ref mut c) = brain.champion {
                    c.score *= 0.5;
                }
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        brain.champion = {
            let mut snap = crate::neotrix::nt_mind::self_iterating::BrainSnapshot::new(
                &brain.brain, &crate::neotrix::nt_world_model::TaskType::General
            );
            snap.score = 1.0;
            Some(snap)
        };
        let before = StageCheckpoint::capture(&brain);
        let _ = DegradingStage.process(&mut brain).unwrap();
        let contract = CapabilityStableContract::default();
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "should detect degradation > 5%");
    }

    #[test]
    fn test_resource_budget_contract() {
        make_stage!(TalkativeStage);
        impl BrainStage for TalkativeStage {
            fn name(&self) -> &str { "talkative" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain.tool_call_count += 10;
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        let before = StageCheckpoint::capture(&brain);
        let _ = TalkativeStage.process(&mut brain).unwrap();
        let contract = ResourceBudgetContract { max_tool_calls_per_run: 3 };
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "should detect budget overuse");
    }

    #[test]
    fn test_monotonic_iteration_contract() {
        make_stage!(RewinderStage);
        impl BrainStage for RewinderStage {
            fn name(&self) -> &str { "rewinder" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain.iteration = brain.iteration.saturating_sub(5);
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        brain.iteration = 10;
        let before = StageCheckpoint::capture(&brain);
        let _ = RewinderStage.process(&mut brain).unwrap();
        let contract = MonotonicIterationContract;
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "should detect iteration regression");
    }

    #[test]
    fn test_contract_registry_routing() {
        let mut registry = ContractRegistry::new();
        registry.register("read_only_stage", Box::new(ReadOnlyContract));
        registry.register("mutator_stage", Box::new(CapabilityStableContract::default()));

        let read_contracts = registry.contracts_for("read_only_stage");
        assert_eq!(read_contracts.len(), 1);
        assert_eq!(read_contracts[0].name(), "read_only");

        let no_contracts = registry.contracts_for("nonexistent");
        assert!(no_contracts.is_empty());
    }

    #[test]
    fn test_enforcement_policy_default() {
        let policy = EnforcementPolicy::default();
        assert_eq!(policy.action_for(ContractSeverity::Suggestion), ActionOnViolation::Log);
        assert_eq!(policy.action_for(ContractSeverity::Warning), ActionOnViolation::Log);
        assert_eq!(policy.action_for(ContractSeverity::Error), ActionOnViolation::Block);
    }

    #[test]
    fn test_enforcement_policy_strict() {
        let policy = EnforcementPolicy::strict();
        assert_eq!(policy.action_for(ContractSeverity::Warning), ActionOnViolation::Block);
        assert_eq!(policy.action_for(ContractSeverity::Error), ActionOnViolation::Block);
    }

    #[test]
    fn test_contract_aware_stage_proxies() {
        make_stage!(ProxyStage);
        impl BrainStage for ProxyStage {
            fn name(&self) -> &str { "proxy" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain._reward += 1.0;
                Ok(StageDecision::Continue)
            }
        }

        let wrapped = ContractAwareStage::new(Box::new(ProxyStage));
        assert_eq!(wrapped.name(), "proxy");
        assert_eq!(wrapped.frequency(), 1);

        let mut brain = dummy_brain();
        let result = wrapped.process(&mut brain);
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_self_edit_contract_detects_edits() {
        make_stage!(EditorStage);
        impl BrainStage for EditorStage {
            fn name(&self) -> &str { "editor" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain._micro_edits.push(crate::neotrix::nt_mind::self_edit::MicroEdit::AdjustDimension("test".into(), 0.5));
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        let before = StageCheckpoint::capture(&brain);
        let _ = EditorStage.process(&mut brain).unwrap();
        let contract = NoSelfEditContract;
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "should detect micro-edits");
    }

    #[test]
    fn test_capability_change_without_champion_no_violation() {
        make_stage!(NoChampionStage);
        impl BrainStage for NoChampionStage {
            fn name(&self) -> &str { "no_champion" }
            fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
                brain._reward = 100.0;
                Ok(StageDecision::Continue)
            }
        }

        let mut brain = dummy_brain();
        brain.champion = None;
        let before = StageCheckpoint::capture(&brain);
        let _ = NoChampionStage.process(&mut brain).unwrap();
        let contract = CapabilityStableContract::default();
        let violations = contract.post_check(&brain, &before);
        assert!(violations.is_empty(), "no champion = no violation possible");
    }

    #[test]
    fn test_presets_read_only_has_three_contracts() {
        let contracts = presets::read_only();
        assert_eq!(contracts.len(), 3);
    }

    #[test]
    fn test_presets_safe_mutation_has_three_contracts() {
        let contracts = presets::safe_mutation();
        assert_eq!(contracts.len(), 3);
    }

    // ── P0-2 防降级契约测试 (OpenMontage delivery_promise 吸收) ──

    #[test]
    fn test_delivery_promise_capability_led_blocks_surface_improvement() {
        // 表面提升：reward 上升但 champion 未动 → CapabilityLed 必须报 violation
        let mut brain = dummy_brain();
        brain.champion = {
            let mut snap = crate::neotrix::nt_mind::self_iterating::BrainSnapshot::new(
                &brain.brain, &crate::neotrix::nt_world_model::TaskType::General
            );
            snap.score = 1.0;
            Some(snap)
        };
        let before = StageCheckpoint::capture(&brain);
        brain._reward += 5.0; // reward 提升
        // champion 未动 = 表面提升（对应"动画幻灯片不算运动"）
        let contract = DeliveryPromiseContract { promise: DeliveryPromise::capability_led(0.05) };
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty(), "surface improvement must be flagged");
        assert_eq!(violations[0].severity, ContractSeverity::Error, "silent fallback must block");
    }

    #[test]
    fn test_delivery_promise_capability_led_passes_real_improvement() {
        let mut brain = dummy_brain();
        brain.champion = {
            let mut snap = crate::neotrix::nt_mind::self_iterating::BrainSnapshot::new(
                &brain.brain, &crate::neotrix::nt_world_model::TaskType::General
            );
            snap.score = 1.0;
            Some(snap)
        };
        let before = StageCheckpoint::capture(&brain);
        if let Some(ref mut c) = brain.champion {
            c.score = 1.1; // 真实能力提升 10%
        }
        let contract = DeliveryPromiseContract { promise: DeliveryPromise::capability_led(0.05) };
        let violations = contract.post_check(&brain, &before);
        assert!(violations.is_empty(), "real improvement must pass");
    }

    #[test]
    fn test_delivery_promise_champion_loss_always_blocks() {
        let mut brain = dummy_brain();
        brain.champion = {
            let mut snap = crate::neotrix::nt_mind::self_iterating::BrainSnapshot::new(
                &brain.brain, &crate::neotrix::nt_world_model::TaskType::General
            );
            snap.score = 1.0;
            Some(snap)
        };
        let before = StageCheckpoint::capture(&brain);
        brain.champion = None; // champion 丢失 = 能力回退
        let contract = DeliveryPromiseContract { promise: DeliveryPromise::capability_led(0.05) };
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, ContractSeverity::Error);
    }

    #[test]
    fn test_delivery_promise_progress_led_warns_on_regression() {
        let mut brain = dummy_brain();
        brain._reward = 10.0;
        let before = StageCheckpoint::capture(&brain);
        brain._reward = 5.0; // reward 回退
        let contract = DeliveryPromiseContract { promise: DeliveryPromise::progress_led(0.0) };
        let violations = contract.post_check(&brain, &before);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].severity, ContractSeverity::Warning);
    }

    #[test]
    fn test_delivery_promise_progress_led_passes_advance() {
        let mut brain = dummy_brain();
        brain._reward = 5.0;
        let before = StageCheckpoint::capture(&brain);
        brain._reward = 10.0; // reward 提升
        let contract = DeliveryPromiseContract { promise: DeliveryPromise::progress_led(0.0) };
        let violations = contract.post_check(&brain, &before);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_presets_capability_led_has_two_contracts() {
        let contracts = presets::capability_led(0.05);
        assert_eq!(contracts.len(), 2);
        assert_eq!(contracts[0].name(), "delivery_promise");
    }
}
