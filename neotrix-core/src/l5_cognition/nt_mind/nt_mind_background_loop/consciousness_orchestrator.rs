//! 意识体协调器 — 连接 ConsciousnessCore 与 ReasoningEngine 的脑干。
//!
//! 解决 C1(意识核心不思考) + C2(推理引擎无意识)：
//! - 意识核心 tick 后 → 协调器驱动推理引擎思考
//! - 推理引擎行动前 → 协调器过 ValueGate 裁决
//! - 行动结果后 → 协调器反馈给 ValueLearning + NarrativeIntegrator

use crate::core::l7_capability::consciousness_bridge;
fn open_raw_conn() -> Option<rusqlite::Connection> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    rusqlite::Connection::open(
        std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db")
    ).ok()
}
use crate::core::nt_core_rule_memory::{crystallize_scan, gc_rules, ScanConfig};
use std::sync::{Arc, Mutex, OnceLock};

static ORCHESTRATOR: OnceLock<Option<ConsciousnessOrchestrator>> = OnceLock::new();

/// 意识体协调器 — 四系统融合的唯一枢纽。
pub struct ConsciousnessOrchestrator {
    bus: Arc<crate::core::l7_capability::native_bus::NativeBusHandle>,
    gate: Option<ValueGateRuntime>,
    stats: Mutex<OrchestratorStats>,
}

#[derive(Debug, Default, Clone)]
pub struct OrchestratorStats {
    pub ticks: u64,
    pub llm_calls: u64,
    pub tool_dispatches: u64,
    pub value_blocks: u64,
    pub rules_crystallized: u64,
    pub rules_gc_died: u64,
}

use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueCompassRuntime;
use crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::{ValueGate, ValueGateConfig, ValueGateRuntime};
use crate::l5_cognition::nt_mind::nt_mind::evolution::value_learning::ValueLearningEngine;

impl ConsciousnessOrchestrator {
    /// 初始化：一次性装配所有能力。
    pub fn init() -> Result<&'static ConsciousnessOrchestrator, String> {
        let conn = open_raw_conn()
            .unwrap_or_else(|| rusqlite::Connection::open_in_memory().expect("mem"));
        crate::core::nt_core_kb_primitives::schema_initialize(&conn)
            .map_err(|e| format!("schema: {e}"))?;

        let bus = crate::core::l7_capability::native_bus::NativeBus::new();
        let bus_handle = Arc::new(
            crate::core::l7_capability::native_bus::NativeBusHandle::new(bus),
        );

        let compass =
            ValueCompassRuntime::from_kb(&conn)?;
        let learning = ValueLearningEngine::new(compass.clone(), Default::default());
        let gate_impl = ValueGate::new(compass.clone(), Some(learning), ValueGateConfig::default());
        let gate_rt = ValueGateRuntime::new(gate_impl);

        // 同步到 consciousness_bridge
        let weights: std::collections::HashMap<String, f64> = compass
            .snapshot()
            .values
            .iter()
            .map(|(k, v)| (k.clone(), v.weight))
            .collect();
        consciousness_bridge::bridge().sync_value_weights(weights);
        consciousness_bridge::bridge().attach_bus(bus_handle.clone());

        let orchestrator = Self {
            bus: bus_handle,
            gate: Some(gate_rt),
            stats: Mutex::new(OrchestratorStats::default()),
        };

        ORCHESTRATOR.get_or_init(|| Some(orchestrator));
        Self::get().ok_or_else(|| "init failed".into())
    }

    /// 获取全局实例。
    pub fn get() -> Option<&'static ConsciousnessOrchestrator> {
        ORCHESTRATOR.get().and_then(|o| o.as_ref())
    }

    /// T1: tick 前价值裁决。
    pub fn pre_tick(&self, _cycle: u64) -> bool {
        if let Ok(mut s) = self.stats.lock() { s.ticks += 1; }
        true // 非阻断，bridge.pre_tick_check 已在 ConsciousnessCore 中调用
    }

    /// T2: LLM 调用前注入叙事上下文。
    pub fn pre_llm(&self, prompt: &str) -> String {
        if let Ok(mut s) = self.stats.lock() { s.llm_calls += 1; }
        consciousness_bridge::bridge().narrative_prefix().unwrap_or_default() + prompt
    }

    /// T4: 通过 NativeBus 执行工具 — 统一工具入口（含 ValueGate 裁决）。
    pub fn dispatch_tool(
        &self,
        capability_id: &str,
        caller: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // 先过 ValueGate
        if let Some(ref gate) = self.gate {
            use crate::l5_cognition::nt_mind::nt_mind::evolution::value_compass::ValueAction;
            use crate::l5_cognition::nt_mind::nt_mind::evolution::value_gate::GateResult;
            let action = ValueAction::new("tool_dispatch", &format!("dispatch {capability_id}"));
            if let GateResult::Veto { reason, .. } = gate.check(&action, caller) {
                if let Ok(mut s) = self.stats.lock() { s.value_blocks += 1; }
                return Err(format!("ValueGate vetoed: {reason}"));
            }
        }
        let result = self.bus.dispatch(capability_id, caller, input);
        if let Ok(mut s) = self.stats.lock() { s.tool_dispatches += 1; }
        result
    }

    /// E1c: 智慧周期。
    pub(crate) fn _wisdom_cycle(&self) -> Result<(usize, usize), String> {
        let conn = open_raw_conn()
            .unwrap_or_else(|| rusqlite::Connection::open_in_memory().expect("mem"));
        crate::core::nt_core_kb_primitives::schema_initialize(&conn)
            .map_err(|e| format!("schema: {e}"))?;
        let cfg = ScanConfig { min_group: 3, domain: None, limit: 100 };
        let cr = crystallize_scan(&conn, &cfg).map(|r| r.rules_created.len()).unwrap_or(0);
        let gc = gc_rules(&conn, 30).map(|r| r.deleted.len()).unwrap_or(0);
        if let Ok(mut s) = self.stats.lock() {
            s.rules_crystallized += cr as u64;
            s.rules_gc_died += gc as u64;
        }
        Ok((cr, gc))
    }

    pub fn stats(&self) -> OrchestratorStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
}
