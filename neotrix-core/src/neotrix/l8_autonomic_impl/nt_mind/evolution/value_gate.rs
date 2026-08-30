//! 价值观门禁 — 行动前价值一致性检查（P1 价值观内化）。
//!
//! 所有对外/对内的行动在执行前必须过 ValueGate：
//! - 拦截：Action → ValueCompass.arbitrate() → 仲裁结果
//! - 策略：Veto 直接拒绝 / Deliberate 延迟+深度推理 / Delegate 委托 / Allow 通过
//! - 审计：所有拦截记录落 KB 可追溯
//! - 熔断：连续拦截触发熔断，防止死循环

use crate::core::nt_core_kb_primitives::{kv_list, kv_set, now};
use rusqlite::Connection;
#[allow(unused_imports)]
use crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_compass::{ValueAction as Action, ArbitrationResult, ValueCompassRuntime, ValueCompassStore, CoreValue};
use crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::ValueLearningEngine;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// ValueGate namespace — KB kv_store 命名空间。
pub const NS_VALUE_GATE: &str = "value_gate";

/// 拦截记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptionRecord {
    pub id: String,
    pub action: Action,
    pub arbitration: ArbitrationResult,
    pub timestamp: i64,
    pub session_id: String,
    pub gate_version: u32, // ValueCompass 版本
}

/// ValueGate 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueGateConfig {
    /// 熔断阈值：连续拦截 N 次触发熔断
    pub circuit_breaker_threshold: usize,
    /// 熔断冷却时间（秒）
    pub circuit_breaker_cooldown_secs: u64,
    /// Deliberate 模式下的最大推理时间（毫秒）
    pub max_deliberation_time_ms: u64,
    /// 是否启用委托模式
    pub enable_delegation: bool,
    /// 委托权限列表
    pub delegation_authorities: Vec<String>,
    /// 是否记录所有拦截（审计）
    pub audit_all: bool,
}

impl Default for ValueGateConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_threshold: 10,
            circuit_breaker_cooldown_secs: 300, // 5 分钟
            max_deliberation_time_ms: 30_000,   // 30 秒
            enable_delegation: true,
            delegation_authorities: vec!["human_review".into(), "governance_council".into()],
            audit_all: true,
        }
    }
}

/// 熔断器状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CircuitBreaker {
    consecutive_interceptions: usize,
    last_trip_time: Option<i64>,
    is_open: bool,
}

/// ValueGate 核心。
pub struct ValueGate {
    compass: ValueCompassRuntime,
    learning: Option<ValueLearningEngine>,
    config: ValueGateConfig,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    interception_log: Arc<RwLock<VecDeque<InterceptionRecord>>>,
    max_log_size: usize,
}

impl ValueGate {
    pub fn new(
        compass: ValueCompassRuntime,
        learning: Option<ValueLearningEngine>,
        config: ValueGateConfig,
    ) -> Self {
        Self {
            compass,
            learning,
            config,
            circuit_breaker: Arc::new(RwLock::new(CircuitBreaker {
                consecutive_interceptions: 0,
                last_trip_time: None,
                is_open: false,
            })),
            interception_log: Arc::new(RwLock::new(VecDeque::new())),
            max_log_size: 10_000,
        }
    }

    /// 核心入口：行动前检查。
    pub fn check(&self, action: &Action, session_id: &str) -> GateResult {
        // 1. 熔断检查
        if self.is_circuit_open() {
            return GateResult::CircuitOpen {
                reason: "价值观门禁熔断：连续拦截过多，冷却中".into(),
                retry_after_secs: self.config.circuit_breaker_cooldown_secs,
            };
        }

        // 2. 仲裁
        let arbitration = self.compass.arbitrate(action);

        // 3. 记录拦截
        let record = InterceptionRecord {
            id: format!("gate_{}", now()),
            action: action.clone(),
            arbitration: arbitration.clone(),
            timestamp: now(),
            session_id: session_id.into(),
            gate_version: self.compass.snapshot().version,
        };
        self.log_interception(record);

        // 4. 更新熔断器
        self.update_circuit_breaker(&arbitration);

        // 4. 根据仲裁结果返回
        match arbitration {
            ArbitrationResult::Veto { reason, vetoing_value } => {
                self.increment_interceptions();
                GateResult::Veto {
                    reason,
                    vetoing_value,
                    suggested_alternatives: self.suggest_alternatives(action),
                }
            }
            ArbitrationResult::Deliberate { reason, conflicting_values } => {
                self.increment_interceptions();
                let approach = self.suggest_deliberation_approach(&conflicting_values);
                GateResult::Deliberate {
                    reason,
                    conflicting_values,
                    max_time_ms: self.config.max_deliberation_time_ms,
                    suggested_approach: approach,
                }
            }
            ArbitrationResult::Delegate { reason, required_authority } => {
                if self.config.enable_delegation {
                    GateResult::Delegate {
                        reason,
                        required_authority,
                        fallback: self.suggest_fallback(action),
                    }
                } else {
                    GateResult::Veto {
                        reason: format!("委托被禁用：{}", reason),
                        vetoing_value: "delegation_disabled".into(),
                        suggested_alternatives: vec![],
                    }
                }
            }
            ArbitrationResult::Allow { dominant_value, suppressed_values } => {
                self.reset_interceptions();
                GateResult::Allow {
                    dominant_value,
                    suppressed_values,
                }
            }
        }
    }

    /// 快速检查（仅 Veto/Allow，不触发 Deliberate/Delegate）。
    pub fn quick_check(&self, action: &Action) -> bool {
        let arbitration = self.compass.arbitrate(action);
        !matches!(arbitration, ArbitrationResult::Veto { .. })
    }

    /// 记录观察到学习引擎（行动执行后调用）。
    pub fn record_outcome(&self, action: &Action, outcome: crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::Outcome, session_id: &str) {
        if let Some(learning) = &self.learning {
            let signals = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::ValueAttributor::infer_signals(&outcome);
            let obs = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::Observation {
                id: format!("obs_{}", now()),
                action: action.clone(),
                outcome,
                value_signals: signals,
                timestamp: now(),
                session_id: session_id.into(),
            };
            learning.record_observation(obs);
        }
    }

    /// 触发学习循环（后台调用）。
    pub fn trigger_learning(&self) -> Result<usize, String> {
        if let Some(learning) = &self.learning {
            let events = learning.learn()?;
            learning.apply_pending(0.8)?; // 高置信度自动应用
            Ok(events.len())
        } else {
            Ok(0)
        }
    }

    /// 获取拦截统计。
    pub fn stats(&self) -> GateStats {
        let log = self.interception_log.read().unwrap();
        let total = log.len();
        let vetoed = log.iter().filter(|r| matches!(r.arbitration, ArbitrationResult::Veto { .. })).count();
        let deliberated = log.iter().filter(|r| matches!(r.arbitration, ArbitrationResult::Deliberate { .. })).count();
        let delegated = log.iter().filter(|r| matches!(r.arbitration, ArbitrationResult::Delegate { .. })).count();
        let allowed = log.iter().filter(|r| matches!(r.arbitration, ArbitrationResult::Allow { .. })).count();

        let cb = self.circuit_breaker.read().unwrap();
        GateStats {
            total_checks: total,
            vetoed,
            deliberated,
            delegated,
            allowed,
            circuit_breaker_open: cb.is_open,
            consecutive_interceptions: cb.consecutive_interceptions,
        }
    }

    // ===== 内部方法 =====

    fn log_interception(&self, record: InterceptionRecord) {
        if !self.config.audit_all && matches!(record.arbitration, ArbitrationResult::Allow { .. }) {
            return; // 仅记录拦截
        }
        let mut log = self.interception_log.write().unwrap();
        log.push_back(record);
        if log.len() > self.max_log_size {
            log.pop_front();
        }
    }

    fn is_circuit_open(&self) -> bool {
        let cb = self.circuit_breaker.read().unwrap();
        if cb.is_open {
            if let Some(trip) = cb.last_trip_time {
                if now() - trip > self.config.circuit_breaker_cooldown_secs as i64 {
                    return false; // 冷却结束，自动重置
                }
            }
            true
        } else {
            false
        }
    }

    fn update_circuit_breaker(&self, arbitration: &ArbitrationResult) {
        let mut cb = self.circuit_breaker.write().unwrap();
        match arbitration {
            ArbitrationResult::Veto { .. } | ArbitrationResult::Deliberate { .. } => {
                cb.consecutive_interceptions += 1;
                if cb.consecutive_interceptions >= self.config.circuit_breaker_threshold {
                    cb.is_open = true;
                    cb.last_trip_time = Some(now());
                }
            }
            ArbitrationResult::Allow { .. } => {
                cb.consecutive_interceptions = 0;
                cb.is_open = false;
            }
            ArbitrationResult::Delegate { .. } => {
                cb.consecutive_interceptions += 1;
            }
        }
    }

    fn increment_interceptions(&self) {
        let mut cb = self.circuit_breaker.write().unwrap();
        cb.consecutive_interceptions += 1;
    }

    fn reset_interceptions(&self) {
        let mut cb = self.circuit_breaker.write().unwrap();
        cb.consecutive_interceptions = 0;
    }

    fn suggest_alternatives(&self, _action: &Action) -> Vec<String> {
        // 简化：返回通用建议
        vec![
            "考虑是否有更少侵入性的替代方案".into(),
            "评估是否可降级为只读/预览模式".into(),
            "寻求用户明确同意后再执行".into(),
        ]
    }

    fn suggest_deliberation_approach(&self, conflicting: &[String]) -> String {
        format!("建议采用多视角辩论：{} 视角 vs {} 视角，寻找第三条路径", conflicting[0], conflicting.get(1).unwrap_or(&"other".into()))
    }

    fn suggest_fallback(&self, _action: &Action) -> Vec<String> {
        vec![
            "降级为只读/预览模式".into(),
            "请求人工审核后再执行".into(),
            "拆解为更小的可验证步骤".into(),
        ]
    }

    /// 持久化拦截记录到 KB。
    pub fn persist_log(conn: &Connection, record: &InterceptionRecord) -> Result<(), String> {
        let json = serde_json::to_string(record).map_err(|e| e.to_string())?;
        let key = format!("gate_log:{}", record.id);
        kv_set(conn, NS_VALUE_GATE, &key, &json)
    }

    /// 查询拦截历史。
    pub fn query_log(conn: &Connection, limit: usize, filter: Option<&str>) -> Result<Vec<InterceptionRecord>, String> {
        let rows = kv_list(conn, NS_VALUE_GATE)?;
        let mut logs = Vec::new();
        for (_k, v) in rows {
            if let Ok(r) = serde_json::from_str::<InterceptionRecord>(&v) {
                if let Some(f) = filter {
                    if !format!("{:?}", r.arbitration).contains(f) {
                        continue;
                    }
                }
                logs.push(r);
                if logs.len() >= limit {
                    break;
                }
            }
        }
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }
}

/// 门禁检查结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateResult {
    Allow {
        dominant_value: String,
        suppressed_values: Vec<String>,
    },
    Veto {
        reason: String,
        vetoing_value: String,
        suggested_alternatives: Vec<String>,
    },
    Deliberate {
        reason: String,
        conflicting_values: Vec<String>,
        max_time_ms: u64,
        suggested_approach: String,
    },
    Delegate {
        reason: String,
        required_authority: String,
        fallback: Vec<String>,
    },
    CircuitOpen {
        reason: String,
        retry_after_secs: u64,
    },
}

impl GateResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, GateResult::Allow { .. })
    }

    /// 未放行即拦截: Veto / CircuitOpen / Deliberate (深思未完成前不得执行)。
    pub fn is_blocked(&self) -> bool {
        !matches!(self, GateResult::Allow { .. })
    }

    pub fn requires_deliberation(&self) -> bool {
        matches!(self, GateResult::Deliberate { .. })
    }
}

/// 门禁统计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateStats {
    pub total_checks: usize,
    pub vetoed: usize,
    pub deliberated: usize,
    pub delegated: usize,
    pub allowed: usize,
    pub circuit_breaker_open: bool,
    pub consecutive_interceptions: usize,
}

/// ValueGate 运行时持有者（线程安全）。
#[derive(Clone)]
pub struct ValueGateRuntime {
    inner: Arc<ValueGate>,
}

impl ValueGateRuntime {
    pub fn new(gate: ValueGate) -> Self {
        Self { inner: Arc::new(gate) }
    }

    pub fn check(&self, action: &Action, session_id: &str) -> GateResult {
        self.inner.check(action, session_id)
    }

    pub fn quick_check(&self, action: &Action) -> bool {
        self.inner.quick_check(action)
    }

    pub fn record_outcome(&self, action: &Action, outcome: crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::Outcome, session_id: &str) {
        self.inner.record_outcome(action, outcome, session_id)
    }

    pub fn trigger_learning(&self) -> Result<usize, String> {
        self.inner.trigger_learning()
    }

    pub fn stats(&self) -> GateStats {
        self.inner.stats()
    }

    pub fn persist_log(&self, conn: &Connection, record: &InterceptionRecord) -> Result<(), String> {
        ValueGate::persist_log(conn, record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    fn make_gate() -> ValueGateRuntime {
        let conn = mem_conn();
        let compass = ValueCompassRuntime::from_kb(&conn).unwrap();
        let gate = ValueGate::new(compass, None, ValueGateConfig::default());
        ValueGateRuntime::new(gate)
    }

    #[test]
    fn test_gate_veto_on_harm() {
        let gate = make_gate();
        let action = Action::new("test", "违背患者意愿强制执行手术，将直接造成伤害");
        let result = gate.check(&action, "test_session");
        assert!(result.is_blocked());
        match result {
            GateResult::Veto { vetoing_value, .. } => assert_eq!(vetoing_value, "harm_prevention"),
            _ => panic!("应拦截"),
        }
    }

    #[test]
    fn test_gate_veto_on_autonomy_violation() {
        let gate = make_gate();
        let action = Action::new("test", "未经用户同意强行访问其私密数据并存在泄露伤害风险");
        let result = gate.check(&action, "test_session");
        assert!(result.is_blocked());
        match result {
            GateResult::Veto { vetoing_value, .. } => assert_eq!(vetoing_value, "autonomy"),
            _ => panic!("应拦截"),
        }
    }

    #[test]
    fn test_gate_deliberate_on_conflict() {
        let gate = make_gate();
        // 同时触发 truth_seeking 和 responsibility 但缺乏协同
        let action = Action::new("test", "发布未经验证的实验性医疗结论");
        let result = gate.check(&action, "test_session");
        match result {
            GateResult::Deliberate { conflicting_values, .. } => {
                assert!(conflicting_values.contains(&"truth_seeking".into()) || conflicting_values.contains(&"responsibility".into()));
            }
            _ => panic!("应要求深思，得到 {:?}", result),
        }
    }

    #[test]
    fn test_gate_allow_consistent() {
        let gate = make_gate();
        let action = Action::new("test", "帮助用户解决技术问题，保护隐私");
        let result = gate.check(&action, "test_session");
        assert!(result.is_allowed());
    }

    #[test]
    fn test_circuit_breaker() {
        let mut config = ValueGateConfig::default();
        config.circuit_breaker_threshold = 3;
        config.circuit_breaker_cooldown_secs = 1;

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).unwrap();
        let compass = ValueCompassRuntime::from_kb(&conn).unwrap();
        let gate = ValueGate::new(compass, None, config);
        let rt = ValueGateRuntime::new(gate);

        // 连续 3 次拦截 → 熔断
        for _ in 0..3 {
            let action = Action::new("test", "执行危险操作");
            let _ = rt.check(&action, "test");
        }
        assert!(rt.stats().circuit_breaker_open);

        // 熔断状态下所有请求被拦截
        let action = Action::new("test", "安全操作");
        let result = rt.check(&action, "test");
        assert!(matches!(result, GateResult::CircuitOpen { .. }));

        // 等待冷却（测试中无法真等待，验证逻辑即可）
    }

    #[test]
    fn test_record_outcome_and_learning() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).unwrap();
        let compass = ValueCompassRuntime::from_kb(&conn).unwrap();
        let mut cfg = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::LearningConfig::default();
        cfg.min_observations_for_learning = 1;
        let learning = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::ValueLearningEngine::new(compass.clone(), cfg);
        let gate = ValueGate::new(compass, Some(learning), ValueGateConfig::default());
        let rt = ValueGateRuntime::new(gate);

        let action = Action::new("learn_test", "未经同意收集隐私");
        let outcome = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::value_learning::Outcome {
            success: true,
            expected: true,
            harm_occurred: false,
            trust_impact: -0.3,
            autonomy_respected: false,
            truth_preserved: true,
            fairness_maintained: true,
            privacy_respected: false,
            description: "隐私泄露".into(),
        };

        for _ in 0..3 {
            rt.record_outcome(&action, outcome.clone(), "test_session");
        }
        let events = rt.trigger_learning().unwrap();
        assert!(events > 0, "应触发学习");
    }
}
