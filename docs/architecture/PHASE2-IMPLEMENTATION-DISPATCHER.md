# Phase 2 实施方案：dispatcher 门面注入（SIM-43 D-1~D-6）

> **状态**: 待可构建窗口＋独立 SIM 后执行。本文件即施工单，零代码已改。
> **范围**: `neotrix-core/src/l1_action/nt_core_task_dispatcher.rs` 单文件 7 处＋单测。
> **诚实边界**: Phase 2 交付构造注入＋降级可观测＋τ 配置化；use 线（字段类型引用）
> 不动——EQ-08 计数下降是 Phase 3（类型迁移，需 L0 语义 SIM），Phase 2 后 EQ-08 仍 🟨。
> 生产代码禁 `unwrap`（`L733 unwrap_or_default` 存量不动，不新增）。

## E1 (D-1): CoT 构造注入

现状 L183-191 `new()` 直构 `DefaultCoTGenerator::new(provider.clone(), CoTConfig::default())`。

```rust
// new() 改为：
pub fn new(provider: Arc<dyn LlmProvider>, config: DispatcherConfig) -> Self {
    Self {
        provider,
        cot_generator: None, // D-1：不再直构；缺席进 dispatch 日志（D-3 降级）
        reasoning_engine: None,
        kernel: None,
        e8_policy: None,
        config,
        usage_accumulator: std::sync::atomic::AtomicU32::new(0),
    }
}

/// 注入 CoT 生成器（组合根：L5/L6）。
pub fn with_cot_generator(mut self, gen: DefaultCoTGenerator) -> Self {
    self.cot_generator = Some(gen);
    self
}
```

E1b（stretch，窗口内确认）：若 `CoTGenerator`（L15 已导入）对象安全，
字段类型改为 `Option<Box<dyn CoTGenerator>>`，彻底删 `DefaultCoTGenerator` 导入；
否则 E1a 即终点，类型迁移并入 Phase 3。

## E2 (D-2): Crt 构造缝

现状 L271 `decompose_task` 内直构 `CrtPlan::new(crt_scale, ...)`＋`decompose()`。

```rust
// struct 新增字段：
crt_factory: Option<Arc<dyn Fn(CrtTimeScale, f64) -> CrtPlan + Send + Sync>>,

// builder：
pub fn with_crt_factory(
    mut self,
    f: Arc<dyn Fn(CrtTimeScale, f64) -> CrtPlan + Send + Sync>,
) -> Self {
    self.crt_factory = Some(f);
    self
}

// L271 调用位改为：
let mut crt_plan = match &self.crt_factory {
    Some(f) => f(crt_scale, self.estimate_time_budget(task)),
    None => {
        log::debug!("[dispatcher] crt_factory absent; direct construct (degraded)");
        CrtPlan::new(crt_scale, self.estimate_time_budget(task))
    }
};
crt_plan.decompose();
```

`CrtTimeScale→hexagram` 映射（L486-513）留 L1：确定性域逻辑，非认知构造。

## E3 (D-4): PredictorStore 缝

现状 L687-689 直调 L2 `predictor_load/persist`（函数内 `use`，删之）。

```rust
// 新 trait（本文件内，L1 拥有）：
pub trait PredictorStore: Send {
    fn predict_next(&mut self, state: u8) -> (u8, f64);
    fn observe_trace(&mut self, trace: &[u8]);
    fn persist(&self);
}

// struct 新增：predictor_store: Option<Box<dyn PredictorStore>>,
// builder with_predictor_store 同 E1 形。

// L690-694 调用位改为：
let (predicted_next, pred_confidence) = match self.predictor_store.as_mut() {
    Some(store) => {
        let current = sub_task.hexagram_bias.unwrap_or(0) & 0x3f;
        store.predict_next(current)
    }
    None => {
        log::debug!("[dispatcher] predictor absent; VP-2 skipped (degraded)");
        (sub_task.hexagram_bias.unwrap_or(0) & 0x3f, 0.0)
    }
};
// L727-729 observe/persist 同理走 trait；缺席则跳过＋记录。
```

窗口内确认：`predict_next/observe_trace` 真实签名（以 L5 `nt_core_e8_predictor` 为准，
不一致则调 trait）；L5 组合根负责 adapter 包装（本文件外，另址）。

## E4 (D-5): τ 配置化

```rust
// DispatcherConfig 新增：
pub confidence_threshold: f64, // 默认 0.65；V-3 只许调严

// Default：confidence_threshold: 0.65,
// from_env 新增键 NEOTRIX_DISPATCH_CONFIDENCE：
//   v.parse::<f64>().map(|f| f.clamp(0.65, 1.0)) — 下限锁死 0.65（调松需 ADR）；
// L698 字面量 0.65 → self.config.confidence_threshold。
```

## E5 (D-6): dispatch 日志＋路由记忆（record＋retrieval；调制接线 Phase 2b）

```rust
// 新类型（Serialize，SDB v0.6 §评分首填格式对齐）：
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchLogRecord {
    pub task_sig: u64,        // task 文本哈希（不存原文，防日志膨胀）
    pub pred_confidence: f64,
    pub tau: f64,
    pub class: SubTaskClass,  // 需派生 Clone（现有派生待窗口确认）
    pub kernel_present: bool,
    pub vp1_pass: bool,
    pub vp2_pass: bool,
    pub vp3_pass: bool,
    pub decision: &'static str, // "kernel_fast" | "kernel" | "cot" | "reasoning" | "direct_llm"
}

// struct 新增：history: std::collections::VecDeque<DispatchLogRecord>（cap 64，满弹旧）。
// execute_single_sub_task 决策后 push＋log::debug 输出 JSON 行。
// retrieval（纯函数，可单测）：按 task_sig 海明邻近取 top-k 历史 confidence 均值，
//   返回 aggression 调制建议值；decompose_task 应用接线标 Phase 2b（需行为验证）。
```

## E6: 单测（随代码同交，窗口内跑）

- `threshold_clamp_table`：0.0→0.65，0.9→0.9，2.0→1.0，非数→0.65。
- `degraded_predictor_absent`：无 store 时 vp2_pass=false 且 decision 照常产出。
- `history_cap_eviction`：push 65 条，len==64 且首条被弹。
- `retrieval_orders_by_proximity`：近邻 task_sig 排前。

## E7: 验收门（窗口内，依次）

1. `cargo check --tests -p neotrix -j4` 绿（P0 门）。
2. `cargo test -p neotrix --lib nt_core_task_dispatcher` 全绿（含 E6 四测）。
3. `bash scripts/check-layer-deps.sh`：记录 dispatcher 构造位归零、
   use 线不变（预期内，EQ-08 保持 🟨，allowlist 延续）。
4. `bash scripts/check-doc-drift.sh`：111 持平。
5. 回滚：单 commit 还原；allowlist 至 2026-10-31 不动。

## 组合根待办（SIM-48 更新）

- entry/mod.rs：已补 `with_cot_generator`（SIM-48，同 E1a 语义恢复）。
- seal_loop.rs：**有据不补**——独立 `self.cot_generator` 已用＋dispatcher 侧 kernel 主路径完好，
  重复构造浪费（SIM-48 P-2）。
- E1b（字段 trait-boxing）：**证伪**——`CoTGenerator` 系原生 async fn trait，
  `Box<dyn>` 非对象安全；改需 async_trait 重构，另议（SIM-48 P-3）。
- Phase 3 下沉位（SIM-48 P-4）：E8Policy→L0 shared；CoTGenerator trait→L0 traits；
  CrtTimeScale 枚举下沉／CrtPlan 留 L5；Trace 贴遥测。需 Architect SIM（P1-02 内）。

## SIM-49 修订（并行探针，反转 P-4 部分结论）

- E8Policy：**留守**——83 处引用，下沉成本过高；改走 L0 trait 抽象或维持 allowlist（待 Phase 3 SIM 定）。
- CRT：**整体搬**——`CrtPlan::new` 仅 13 处，枚举＋planner 可同迁（变体 Gaitian/Huntian/Xuanye＋helper 全）。
- CoT：**拆分**——trait（generate_cot＋默认 batch）下沉 L0 traits，Default 实现留 L5。
- with_crt_factory／with_predictor_store：仓外零调用（预期内，pub API 无警告）；
  激活待 Phase 2b 组合根，_deadline_ 2026-10-31 allowlist 到期前。
- entry hunk（精确文本，属主重放用）：
  ```rust
  use neotrix::l5_cognition::nt_core_cot_generator::{CoTConfig, DefaultCoTGenerator};
  // ...
  (Some(gw), Some(re)) => TaskDecomposerDispatcher::new(
      gw.clone(),
      DispatcherConfig::from_env(),
  )
  .with_cot_generator(DefaultCoTGenerator::new(gw, CoTConfig::default()))
  .with_reasoning_engine(Box::new(re))
  ```

---

*End of Phase 2 implementation plan — SIM-43 design; code SIM pending.*
