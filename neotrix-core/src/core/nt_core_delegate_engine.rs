use std::collections::VecDeque;

use crate::core::nt_core_self_test::SelfTest;

/// 证据强度 — 反哺自 crm/loopx/PentAGI 吸收:
/// "nothing is guessed" 规则: 只有 Observed 证据才能记账, Inferred 是低成本估计。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceStrength {
    /// 真实观测 (工具返回、签名块、账号身份等) — 记账。
    Observed,
    /// 推断/估计 (模型置信度/启发式) — 不计强证据，只作参考。
    Inferred,
}

struct TaskRecord {
    id: String,
    task: String,
    source: String,
    domain: String,
    priority: u8,
    success: Option<bool>,
    evidence: EvidenceStrength,
}

/// 单域熔断 (circuit breaker) — PentAGI 域熔断 + loopx quota 精神:
/// 连续失败达到阈值后该域被熔断 (gate), 拒绝新委托直至 reset/rekey。
#[derive(Debug, Clone)]
struct DomainGate {
    consecutive_failures: u64,
    tripped_until: Option<u64>,
    max_ticks_failure: u64,
    failure_threshold: u64,
}

impl DomainGate {
    fn new(failure_threshold: u64, ticks: u64) -> Self {
        Self { consecutive_failures: 0, tripped_until: None, max_ticks_failure: ticks, failure_threshold }
    }
    fn is_tripped(&self, now: u64) -> bool {
        self.tripped_until.map(|t| now < t).unwrap_or(false)
    }
    fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.tripped_until = None;
    }
    fn record_failure(&mut self, now: u64) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.failure_threshold {
            self.tripped_until = Some(now + self.max_ticks_failure);
            self.consecutive_failures = 0;
        }
    }
}

pub struct DelegateEngine {
    tasks: VecDeque<TaskRecord>,
    total_completed: u64,
    total_succeeded: u64,
    next_id: u64,
    max_concurrent: usize,
    tick: u64,
    /// 域熔断表 — 允许被熔断的域列表 (quota/gate 控制面)。
    gates: std::collections::HashMap<String, DomainGate>,
}

impl DelegateEngine {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            total_completed: 0,
            total_succeeded: 0,
            next_id: 0,
            tick: 0,
            max_concurrent: 64,
            gates: std::collections::HashMap::new(),
        }
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// 注册一个带熔断控制的域 (默认阈值 3 连败 / 熔断 100 tick)。
    pub fn with_domain_gate(mut self, domain: &str) -> Self {
        self.gates.entry(domain.to_string()).or_insert(DomainGate::new(3, 100));
        self
    }

    /// 注册域熔断并指定失败阈值与熔断 tick 数。
    pub fn with_domain_gate_config(mut self, domain: &str, failure_threshold: u64, ticks: u64) -> Self {
        self.gates.insert(domain.to_string(), DomainGate::new(failure_threshold, ticks));
        self
    }

    /// 委托任务。若所属域已熔断则拒绝 (gate)。
    pub fn delegate(&mut self, task: &str, source: &str, priority: u8) -> Option<String> {
        self.delegate_to(task, source, "default", priority)
    }

    /// 委托到指定域，域熔断时拒绝。
    pub fn delegate_to(&mut self, task: &str, source: &str, domain: &str, priority: u8) -> Option<String> {
        if self.tasks.len() >= self.max_concurrent {
            return None;
        }
        if self.is_domain_tripped(domain) {
            return None;
        }
        let id = format!("del-{}", self.next_id);
        self.next_id += 1;
        self.tasks.push_back(TaskRecord {
            id: id.clone(),
            task: task.to_string(),
            source: source.to_string(),
            domain: domain.to_string(),
            priority,
            success: None,
            evidence: EvidenceStrength::Inferred,
        });
        Some(id)
    }

    /// 某域当前是否被熔断 (gate 拒绝)。
    pub fn is_domain_tripped(&self, domain: &str) -> bool {
        self.gates.get(domain).map(|g| g.is_tripped(self.tick)).unwrap_or(false)
    }

    /// 记录真实观测结果 (Observed 证据)。无结果不虚构成功。
    pub fn record_outcome(&mut self, id: &str, success: bool) -> bool {
        self.record_evidence(id, success, EvidenceStrength::Observed)
    }

    /// 记录结果并标证据强度; 强证据才计入熔断统计。
    pub fn record_evidence(&mut self, id: &str, success: bool, evidence: EvidenceStrength) -> bool {
        let domain = if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id && t.success.is_none()) {
            t.success = Some(success);
            t.evidence = evidence;
            Some(t.domain.clone())
        } else {
            return false;
        };
        // Observed 结果才驱动域熔断 — Inferred 不强判。
        if evidence == EvidenceStrength::Observed {
            if let Some(g) = self.gates.get_mut(&domain.clone().unwrap_or_default()) {
                if success { g.record_success(); } else { g.record_failure(self.tick); }
            }
        }
        true
    }

    /// 推进一个 tick; 被熔断的域在 max_ticks_failure 后自动 reset。
    pub fn tick_forward(&mut self) {
        self.tick += 1;
    }

    /// 结算已有真实结果的任务；返回仍未结算的任务数。
    pub fn synchronize(&mut self) -> u64 {
        let mut settled: Vec<String> = Vec::new();
        for task in self.tasks.iter_mut() {
            if let Some(outcome) = task.success.take() {
                self.total_completed += 1;
                if outcome {
                    self.total_succeeded += 1;
                }
                settled.push(task.id.clone());
            }
        }
        self.tasks.retain(|t| !settled.contains(&t.id));
        self.tasks.len() as u64
    }

    pub fn total_tasks(&self) -> u64 {
        self.total_completed + self.tasks.len() as u64
    }

    /// 无已完成任务时返回 0.0（不做假 1.0）。
    pub fn success_rate(&self) -> f64 {
        if self.total_completed == 0 {
            return 0.0;
        }
        self.total_succeeded as f64 / self.total_completed as f64
    }

    pub fn pending_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn completed_count(&self) -> u64 {
        self.total_completed
    }

    /// 读取待结算任务元数据 (task/source/priority) — 供日志与监控消费
    pub fn pending_tasks(&self) -> Vec<(String, String, u8)> {
        self.tasks.iter().map(|t| (t.task.clone(), t.source.clone(), t.priority)).collect()
    }
}

impl Default for DelegateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for DelegateEngine {
    fn name(&self) -> &'static str {
        "DelegateEngine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_returns_id() {
        let mut e = DelegateEngine::new();
        let id = e.delegate("task", "src", 0).expect("delegated");
        assert!(id.starts_with("del-"));
        assert_eq!(e.pending_count(), 1);
    }

    #[test]
    fn test_synchronize_without_outcome_keeps_pending() {
        let mut e = DelegateEngine::new();
        e.delegate("t", "s", 0);
        let pending = e.synchronize();
        assert_eq!(pending, 1);
        assert_eq!(e.completed_count(), 0);
        assert_eq!(e.success_rate(), 0.0);
    }

    #[test]
    fn test_record_outcome_success() {
        let mut e = DelegateEngine::new();
        let id = e.delegate("t", "s", 0).unwrap();
        assert!(e.record_outcome(&id, true));
        let pending = e.synchronize();
        assert_eq!(pending, 0);
        assert_eq!(e.completed_count(), 1);
        assert_eq!(e.success_rate(), 1.0);
    }

    #[test]
    fn test_record_outcome_failure_lowers_rate() {
        let mut e = DelegateEngine::new();
        let ok = e.delegate("ok", "s", 0).unwrap();
        let bad = e.delegate("bad", "s", 0).unwrap();
        e.record_outcome(&ok, true);
        e.record_outcome(&bad, false);
        e.synchronize();
        assert_eq!(e.completed_count(), 2);
        assert!((e.success_rate() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_max_concurrent_rejects() {
        let mut e = DelegateEngine::new().with_max_concurrent(2);
        assert!(e.delegate("a", "s", 0).is_some());
        assert!(e.delegate("b", "s", 0).is_some());
        assert!(e.delegate("c", "s", 0).is_none());
    }

    #[test]
    fn test_record_outcome_unknown_id_fails() {
        let mut e = DelegateEngine::new();
        assert!(!e.record_outcome("ghost", true));
    }

    #[test]
    fn test_domain_gate_trips_after_consecutive_observed_failures() {
        let mut e = DelegateEngine::new().with_domain_gate("danger");
        // 阈值默认 3 连败
        for i in 0..3 {
            let id = e.delegate_to("t", "s", "danger", 0).unwrap();
            assert!(!e.is_domain_tripped("danger"));
            e.record_outcome(&id, false);
        }
        // 再 delegate 到 danger 域应被 gate 拒绝
        e.tick_forward();
        assert!(e.is_domain_tripped("danger"));
        assert!(e.delegate_to("blocked", "s", "danger", 0).is_none());
    }

    #[test]
    fn test_domain_gate_resets_after_ticks() {
        let mut e = DelegateEngine::new().with_domain_gate_config("risk", 1, 5);
        let id = e.delegate_to("a", "s", "risk", 0).unwrap();
        e.record_outcome(&id, false);
        // 失败发生在 tick 0, 熔断持续到 tick<0+5
        assert!(e.is_domain_tripped("risk"));
        e.tick_forward();
        assert!(e.is_domain_tripped("risk")); // tick1
        e.tick_forward();
        e.tick_forward();
        assert!(e.is_domain_tripped("risk")); // tick3
        e.tick_forward();
        e.tick_forward(); // tick5 → 复位
        assert!(!e.is_domain_tripped("risk"));
        assert!(e.delegate_to("b", "s", "risk", 0).is_some());
    }

    #[test]
    fn test_observed_success_resets_gate() {
        let mut e = DelegateEngine::new().with_domain_gate("domain_s");
        let a = e.delegate_to("a", "s", "domain_s", 0).unwrap();
        let b = e.delegate_to("b", "s", "domain_s", 0).unwrap();
        let c = e.delegate_to("c", "s", "domain_s", 0).unwrap();
        e.record_outcome(&a, false);
        e.record_outcome(&b, false);
        e.record_outcome(&c, false);
        assert!(e.is_domain_tripped("domain_s"));
        // 即使熔断, 熔断期间的委托不被接受; 但 reset 后恢复
        e.tick_forward();
        assert!(e.is_domain_tripped("domain_s"));
    }

    #[test]
    fn test_default_domain_tracks_failures_for_gate_only() {
        // default 域未注册 gate → 永不熔断
        let mut e = DelegateEngine::new();
        for _ in 0..10 {
            let id = e.delegate("t", "s", 0).unwrap();
            e.record_outcome(&id, false);
        }
        assert!(!e.is_domain_tripped("default"));
    }
}
