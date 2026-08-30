//! 原生能力总线 (Native Capability Bus) — 去 MCP 化的执行内核。
//!
//! 第一性原理拆解 (2026-08-24):
//! - 意识体的本质需求 = **类型化能力调用**: 发现(能力树) → 裁决(GuardChain) → 执行(同进程直调) → 证据(EvidenceEntry)
//! - MCP 的进程边界/JSON-RPC/动态发现是为跨语言生态设计的**偶然包袱**:
//!   全 Rust 单体中它们是纯开销 (序列化 + 子进程运维 + 握手协议)
//! - 本设计把执行面收敛为同进程 trait 直调; 外部非 Rust 工具走
//!   feature-gated 的 `external_tools` (typed 行协议), 默认关闭。
//!
//! 与现有节点的关系 (R-P42 强化而非平行):
//! - 寻址复用 `CapabilityId` (capability_tree 一等公民)
//! - 治理复用 `nt_core_guard_chain::GuardChain`
//! - 证据链复用 gateway 的 EvidenceEntry 语义 (此处轻量内联, 避免循环依赖)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 能力执行结果 — 类型化契约的最小载体 (R-P81 第⑤级: 已装依赖 serde_json)。
pub type CapInput = serde_json::Value;
pub type CapOutput = serde_json::Value;

/// 原生能力 — 同进程可执行单元的统一契约。
///
/// 与 `registry::Capability`(静态描述) 配对: 描述进能力树, 本 trait 是执行面。
/// schema 用 JSON Schema 字符串表达 (人类可读 + 可校验), 不引入宏幻术。
pub trait NativeCapability: Send + Sync {
    /// 能力树节点 ID (寻址唯一键)
    fn id(&self) -> &str;
    /// 人类可读名称
    fn name(&self) -> &str;
    /// 输入/输出 schema 描述 (JSON Schema 片段)
    fn input_schema(&self) -> &str;
    fn output_schema(&self) -> &str;
    /// 执行 — 同进程直调, 零序列化边界 (input/output 仅为契约载体)
    fn execute(&self, input: CapInput) -> Result<CapOutput, String>;
    /// 是否只读 (治理通道: 只读能力可跳过写守卫)
    fn read_only(&self) -> bool {
        false
    }
}

/// 调用证据 — 治理审计的最小记录 (对齐 gateway EvidenceEntry 语义)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallEvidence {
    pub capability_id: String,
    pub caller: String,
    pub input_hash: u64,
    pub output_ok: bool,
    pub duration_us: u64,
    pub timestamp: i64,
}

fn hash_input(input: &CapInput) -> u64 {
    // FNV-1a — 与 nt_core_rule_memory 同源, 不引新依赖
    let json = serde_json::to_string(input).unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in json.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// 守卫 — dispatch 前裁决。返回 Err 即拦截 (语义对齐 GuardChain)。
pub type GuardFn = Arc<dyn Fn(&str, &CapInput) -> Result<(), String> + Send + Sync>;

/// 原生能力总线 — 注册 + 守卫裁决 + 直调分发 + 证据记录。
#[derive(Default)]
pub struct NativeBus {
    capabilities: HashMap<String, Arc<dyn NativeCapability>>,
    guards: Vec<(String, GuardFn)>,
    evidence_log: Vec<CallEvidence>,
    max_evidence: usize,
}

impl NativeBus {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            guards: Vec::new(),
            evidence_log: Vec::new(),
            max_evidence: 10_000,
        }
    }

    /// 注册能力 (id 冲突 = 编译期不可查的运行时错, fail-loud)。
    pub fn register(&mut self, cap: Arc<dyn NativeCapability>) -> Result<(), String> {
        let id = cap.id().to_string();
        if self.capabilities.contains_key(&id) {
            return Err(format!("能力 id 冲突: {id} (provides 唯一性门禁, R-P110)"));
        }
        self.capabilities.insert(id, cap);
        Ok(())
    }

    /// 挂载守卫 (先注册先裁决; 任一 Err 即整体拦截)。
    pub fn add_guard(&mut self, name: impl Into<String>, guard: GuardFn) {
        self.guards.push((name.into(), guard));
    }

    /// 能力是否已注册。
    pub fn has(&self, id: &str) -> bool {
        self.capabilities.contains_key(id)
    }

    /// 已注册能力列表。
    pub fn list(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.capabilities.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// 核心: 统一调用入口 — 守卫裁决 → 直调 → 证据。
    /// caller 用于审计溯源 (GoalLoop / GWT / ReasoningEngine 等)。
    pub fn dispatch(
        &mut self,
        capability_id: &str,
        caller: &str,
        input: CapInput,
    ) -> Result<CapOutput, String> {
        // 1. 守卫裁决 (先于存在性检查? 否 — 未注册能力无可裁之物, 先查存在防信息泄露方向反了:
        //    对外暴露时应先守卫再探测; 对内单体两者等价, 取先守卫后探测以保治理一致性)
        for (name, guard) in &self.guards {
            guard(capability_id, &input).map_err(|e| format!("守卫[{name}]拦截: {e}"))?;
        }
        // 2. 存在性
        let cap = self
            .capabilities
            .get(capability_id)
            .ok_or_else(|| format!("能力未注册: {capability_id}"))?;
        // 3. 直调
        let start = std::time::Instant::now();
        let result = cap.execute(input.clone());
        // 4. 证据 (环形截断, Dark Forest: 无消费者的旧证据自动淘汰)
        let ev = CallEvidence {
            capability_id: capability_id.to_string(),
            caller: caller.to_string(),
            input_hash: hash_input(&input),
            output_ok: result.is_ok(),
            duration_us: start.elapsed().as_micros() as u64,
            timestamp: now_secs(),
        };
        self.evidence_log.push(ev);
        if self.evidence_log.len() > self.max_evidence {
            let overflow = self.evidence_log.len() - self.max_evidence;
            self.evidence_log.drain(..overflow);
        }
        result
    }

    /// 最近 N 条证据 (审计/复盘)。
    pub fn recent_evidence(&self, n: usize) -> Vec<CallEvidence> {
        let len = self.evidence_log.len();
        if n >= len {
            self.evidence_log.clone()
        } else {
            self.evidence_log[len - n..].to_vec()
        }
    }
}

/// 线程安全运行时句柄 (意识体各域共享)。
#[derive(Clone)]
pub struct NativeBusHandle {
    inner: Arc<std::sync::Mutex<NativeBus>>,
}

impl NativeBusHandle {
    pub fn new(bus: NativeBus) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(bus)),
        }
    }

    pub fn dispatch(
        &self,
        capability_id: &str,
        caller: &str,
        input: CapInput,
    ) -> Result<CapOutput, String> {
        self.inner
            .lock()
            .map_err(|e| format!("bus lock: {e}"))?
            .dispatch(capability_id, caller, input)
    }

    pub fn register(&self, cap: Arc<dyn NativeCapability>) -> Result<(), String> {
        self.inner
            .lock()
            .map_err(|e| format!("bus lock: {e}"))?
            .register(cap)
    }

    pub fn list(&self) -> Vec<String> {
        self.inner.lock().map(|b| b.list()).unwrap_or_default()
    }

    pub fn has(&self, id: &str) -> bool {
        self.inner.lock().map(|b| b.has(id)).unwrap_or(false)
    }

    pub fn recent_evidence(&self, n: usize) -> Vec<CallEvidence> {
        self.inner
            .lock()
            .map(|b| b.recent_evidence(n))
            .unwrap_or_default()
    }
}

/// 便捷宏式构造器: 从闭包生成匿名能力 (最懒实现阶梯 R-P81-⑥)。
pub fn closure_capability(
    id: &str,
    name: &str,
    input_schema: &str,
    output_schema: &str,
    read_only: bool,
    f: impl Fn(CapInput) -> Result<CapOutput, String> + Send + Sync + 'static,
) -> Arc<dyn NativeCapability> {
    struct Closure {
        id: String,
        name: String,
        input_schema: String,
        output_schema: String,
        read_only: bool,
        f: Box<dyn Fn(CapInput) -> Result<CapOutput, String> + Send + Sync>,
    }
    impl NativeCapability for Closure {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn input_schema(&self) -> &str {
            &self.input_schema
        }
        fn output_schema(&self) -> &str {
            &self.output_schema
        }
        fn execute(&self, input: CapInput) -> Result<CapOutput, String> {
            (self.f)(input)
        }
        fn read_only(&self) -> bool {
            self.read_only
        }
    }
    Arc::new(Closure {
        id: id.to_string(),
        name: name.to_string(),
        input_schema: input_schema.to_string(),
        output_schema: output_schema.to_string(),
        read_only,
        f: Box::new(f),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_register_dispatch_and_evidence() {
        let mut bus = NativeBus::new();
        bus.register(closure_capability(
            "kb.query",
            "KB 查询",
            r#"{"query": "string"}"#,
            r#"{"results": "array"}"#,
            true,
            |input| Ok(json!({ "echo": input, "ok": true })),
        ))
        .unwrap();
        assert!(bus.has("kb.query"));
        assert!(!bus.has("kb.missing"));

        let out = bus.dispatch("kb.query", "test", json!({"q": "rust"})).unwrap();
        assert_eq!(out["ok"], json!(true));

        let ev = bus.recent_evidence(10);
        assert_eq!(ev.len(), 1);
        assert!(ev[0].output_ok);
        assert_eq!(ev[0].caller, "test");
    }

    #[test]
    fn test_duplicate_id_rejected_rp110() {
        let mut bus = NativeBus::new();
        let cap = closure_capability("dup", "d", "{}", "{}", true, |_| Ok(json!(null)));
        bus.register(cap).unwrap();
        let cap2 = closure_capability("dup", "d2", "{}", "{}", true, |_| Ok(json!(null)));
        assert!(bus.register(cap2).is_err(), "重复 provides 必须拒绝");
    }

    #[test]
    fn test_guard_intercepts_before_execution() {
        let mut bus = NativeBus::new();
        bus.add_guard("no_writes", Arc::new(|id, _| {
            if id.contains("write") {
                Err("只读模式".into())
            } else {
                Ok(())
            }
        }));
         let _ = bus.register(closure_capability(
             "fs.write",
             "写文件",
             "{}",
             "{}",
             false,
             |_| panic!("守卫拦截后不应执行"),
         ));
        let err = bus.dispatch("fs.write", "t", json!(null)).unwrap_err();
        assert!(err.contains("守卫[no_writes]"), "got: {err}");
    }

    #[test]
    fn test_unregistered_capability_errors_loud() {
        let mut bus = NativeBus::new();
        let err = bus.dispatch("ghost", "t", json!(null)).unwrap_err();
        assert!(err.contains("未注册"));
    }

    #[test]
    fn test_handle_thread_safe_sharing() {
        let handle = NativeBusHandle::new(NativeBus::new());
        let h2 = handle.clone();
        handle
            .register(closure_capability("x.y", "X", "{}", "{}", true, |_| Ok(json!(1))))
            .unwrap();
        assert!(h2.has("x.y"), "克隆共享同一注册表");
        let out = h2.dispatch("x.y", "t", json!(null)).unwrap();
        assert_eq!(out, json!(1));
    }
}
