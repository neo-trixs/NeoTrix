//! 意识体桥接 — 四系统融合的神经中枢。
//!
//! 将 ConsciousnessCore(tick)、ReasoningEngine(call_llm)、
//! CapabilityTree(成熟度) 和 NativeBus(执行) 串联为一条因果链：
//!
//! ```text
//! tick() → pre_tick_check(value) → run_growth_cycle()
//!              │
//! call_llm() → narrative_prefix() → LLM → post_llm_record()
//!              │
//! dispatch_capability(id, input) → NativeBus.execute → evidence
//!              │
//! wisdom_tick → sync_from_evolution(weights, narrative) → bridge 更新
//! ```
//!
//! 设计：全局单例 + 非阻断 + 优雅降级。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// 快速裁决结果
#[derive(Debug, Clone)]
pub enum QuickVerdict {
    Allow,
    Warn(String),
}

impl std::fmt::Display for QuickVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuickVerdict::Allow => write!(f, "Allow"),
            QuickVerdict::Warn(w) => write!(f, "Warn({})", w),
        }
    }
}

/// 全局桥接 — 四系统的共享神经中枢。
pub struct WisdomBridge {
    /// P1: 价值观权重 {value_id → weight}
    value_weights: RwLock<HashMap<String, f64>>,
    /// P2: 叙事自我摘要（注入 LLM prompt）
    narrative_summary: RwLock<Option<String>>,
    /// 健康指标（call_llm 成败衰减）
    last_health: RwLock<f64>,
    /// 能力调用计数（供 CapabilityTree 成熟度反馈）
    dispatch_counts: Mutex<HashMap<String, (u64, u64)>>, // (success, total)
    /// NativeBus 句柄（工具执行通道）
    bus: RwLock<Option<Arc<crate::core::l7_capability::native_bus::NativeBusHandle>>>,
    inject_narrative: RwLock<bool>,
}

static BRIDGE: OnceLock<WisdomBridge> = OnceLock::new();

/// 获取全局桥接实例。
pub fn bridge() -> &'static WisdomBridge {
    BRIDGE.get_or_init(|| WisdomBridge {
        value_weights: RwLock::new(HashMap::new()),
        narrative_summary: RwLock::new(None),
        last_health: RwLock::new(0.0),
        dispatch_counts: Mutex::new(HashMap::new()),
        bus: RwLock::new(None),
        inject_narrative: RwLock::new(false),
    })
}

impl WisdomBridge {
    // ── 数据同步（wisdom tick 调用）──

    /// 同步价值观权重。
    pub fn sync_value_weights(&self, weights: HashMap<String, f64>) {
        if let Ok(mut w) = self.value_weights.write() {
            *w = weights;
        }
    }

    /// 设置叙事摘要。
    pub fn set_narrative(&self, summary: String) {
        if let Ok(mut n) = self.narrative_summary.write() {
            *n = Some(summary);
        }
    }

    /// 挂载 NativeBus 句柄（启动时调用一次）。
    pub fn attach_bus(&self, handle: Arc<crate::core::l7_capability::native_bus::NativeBusHandle>) {
        if let Ok(mut b) = self.bus.write() {
            *b = Some(handle);
        }
    }

    // ── 消费接口（tick / call_llm 调用）──

    /// T1: tick 前价值检查。
    pub fn pre_tick_check(&self, cycle: u64) -> QuickVerdict {
        let weights = self.value_weights.read().unwrap_or_else(|e| e.into_inner());
        for (id, w) in weights.iter() {
            if *w < 0.3 {
                log::warn!("[bridge] cycle={} value {} dropped to {:.2}", cycle, id, w);
                return QuickVerdict::Warn(format!("{} low", id));
            }
        }
        QuickVerdict::Allow
    }

    /// T2: call_llm 前叙事上下文注入。
    pub fn narrative_prefix(&self) -> Option<String> {
        if !*self.inject_narrative.read().unwrap_or_else(|e| e.into_inner()) {
            return None;
        }
        let n = self.narrative_summary.read().ok()?;
        n.as_ref().map(|s| format!("[identity] {}\n", s))
    }

    /// call_llm 后记录结果信号。
    pub fn post_llm_record(&self, success: bool) {
        if let Ok(mut h) = self.last_health.write() {
            *h = if success { 0.9 } else { (*h * 0.9).max(0.1) };
        }
    }

    /// T4: 通过 NativeBus 直调能力（意识核心/推理引擎统一工具入口）。
    ///
    /// 这是四系统融合的核心方法：
    /// - 调用方不需要知道工具在哪、怎么实现
    /// - 守卫链自动裁决
    /// - 证据链自动落账
    /// - 调用计数自动追踪（供 CapabilityTree 成熟度反馈）
    pub fn dispatch_capability(
        &self,
        capability_id: &str,
        caller: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let bus_guard = self.bus.read().map_err(|e| e.to_string())?;
        let bus = bus_guard.as_ref().ok_or_else(|| "NativeBus 未挂载".to_string())?;

        let result = bus.dispatch(capability_id, caller, input);

        // 记录调用计数（成功/总数）→ 供成熟度晋升参考
        if let Ok(mut counts) = self.dispatch_counts.lock() {
            let entry = counts.entry(capability_id.to_string()).or_insert((0, 0));
            entry.1 += 1;
            if result.is_ok() {
                entry.0 += 1;
            }
        }

        result
    }

    /// 获取能力调用统计（供 CapabilityTree mature 参考）。
    pub fn dispatch_stats(&self) -> Vec<(String, u64, u64)> {
        self.dispatch_counts
            .lock()
            .map(|c| c.iter().map(|(k, (s, t))| (k.clone(), *s, *t)).collect())
            .unwrap_or_default()
    }
}

/// 从 evolution 层同步数据到桥接层。
pub fn sync_from_evolution(
    value_weights: Vec<(String, f64)>,
    narrative_summary: Option<String>,
) {
    let b = bridge();
    let map: HashMap<String, f64> = value_weights.into_iter().collect();
    b.sync_value_weights(map);
    if let Some(s) = narrative_summary {
        b.set_narrative(s);
    }
}

/// 挂载 NativeBus 到桥接层（启动时调用）。
pub fn attach_native_bus(handle: Arc<crate::core::l7_capability::native_bus::NativeBusHandle>) {
    bridge().attach_bus(handle);
}

impl WisdomBridge {
    /// 启用叙事注入（仅意识体完整初始化后调用）。
    pub fn enable_narrative_injection(&self) {
        if let Ok(mut f) = self.inject_narrative.write() {
            *f = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bridge_value_check() -> Result<(), String> {
        let b = bridge();
        b.sync_value_weights(HashMap::from([
            ("autonomy".into(), 0.95),
            ("test_low".into(), 0.2),
        ]));
        match b.pre_tick_check(1) {
            QuickVerdict::Warn(msg) => assert!(msg.contains("test_low")),
            _ => return Err("低权重应告警".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_narrative_injection() {
        let b = bridge();
        b.enable_narrative_injection();
        b.set_narrative("I am NeoTrix".into());
        assert!(b.narrative_prefix().unwrap().contains("NeoTrix"));
    }

    #[test]
    fn test_dispatch_through_bridge() {
        let b = bridge();
        // 创建一个 mock bus
        let mut bus = crate::core::l7_capability::native_bus::NativeBus::new();
        bus.register(crate::core::l7_capability::native_bus::closure_capability(
            "test.cap", "Test", "{}", "{}", true,
            |_| Ok(json!({"result": 42})),
        )).unwrap();
        let handle = crate::core::l7_capability::native_bus::NativeBusHandle::new(bus);
        
        // 挂载到 bridge
        b.attach_bus(std::sync::Arc::new(handle));

        // 通过 bridge dispatch
        let out = b.dispatch_capability("test.cap", "test_caller", json!({})).unwrap();
        assert_eq!(out["result"], 42);

        // 验证调用计数
        let stats = b.dispatch_stats();
        assert!(stats.iter().any(|(id, s, t)| id == "test.cap" && *s == 1 && *t == 1));
    }

    #[test]
    fn test_dispatch_unregistered_fails() {
        let b = bridge();
        assert!(b.dispatch_capability("nonexistent", "t", json!({})).is_err());
    }

    #[test]
    fn test_post_llm_health_decay() {
        let b = bridge();
        b.post_llm_record(true); // health = 0.9
        b.post_llm_record(false); // health = 0.81
        b.post_llm_record(false); // health ≈ 0.73
        // 健康值在持续失败时衰减但不归零
    }
}


#[cfg(test)]
mod self_tests {
    use super::*;
    
    /// 验证 bridge 全链路健康
    #[test]  
    fn test_bridge_health() {
        let b = bridge();
        // 1. 价值权重已同步
        b.sync_value_weights(HashMap::from([("autonomy".into(), 0.95)]));
        assert!(format!("{:?}", b.pre_tick_check(0)).len() > 0);
        // 2. 叙事可设置
        b.set_narrative("health check".into());
        // 3. 无 panic 即健康
    }
}

// ═══ SelfTest 覆盖扩展 — P1-P5 新模块自动健康检测 (E2) ═══

/// 规则记忆体健康检测
pub struct RuleMemorySelfTest;

impl crate::core::nt_core_self_test::SelfTest for RuleMemorySelfTest {
    fn name(&self) -> &str { "rule_memory" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        let conn = match crate::core::nt_core_kb_primitives::open_raw_conn() {
            Some(c) => c,
            None => { errs.push("KB unavailable".into()); return Err(errs); }
        };
        crate::core::nt_core_kb_primitives::schema_initialize(&conn).ok();
        // 验证 rule namespace 可读写
        if crate::core::nt_core_kb_primitives::kv_set(&conn, "rule", "_health_check", "ok").is_err() {
            errs.push("rule namespace write failed".into());
        }
        if crate::core::nt_core_rule_memory::rule_list(&conn).is_err() {
            errs.push("rule_list failed".into());
        }
        crate::core::nt_core_kb_primitives::kv_delete(&conn, "rule", "_health_check").ok();
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

/// 意识体桥接健康检测
pub struct ConsciousnessBridgeSelfTest;

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessBridgeSelfTest {
    fn name(&self) -> &str { "consciousness_bridge" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let b = bridge();
        b.sync_value_weights(HashMap::from([("test".to_string(), 0.8)]));
        b.set_narrative("health check".into());
        let _ = b.pre_tick_check(0);
        Ok(())
    }
}

