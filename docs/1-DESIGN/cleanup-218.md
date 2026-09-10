# Cleanup #218 — unwrap 热点 / 跨域引用 / EventBus 连接率

**日期**: 2026-09-11
**状态**: 规划中

---

## 1. Top 3 unwrap 热点文件（非测试代码）

### 1.1 `l5_cognition/nt_mind/nt_mind/autobiographical_index.rs` — 17 处

| 行号 | 代码 | 修复方案 |
|------|------|----------|
| 153 | `self.entries.write().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 178 | `self.entries.write().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 189 | `self.chapter_index.write().unwrap()` | → `.expect("RwLock poisoned: chapter_index")` |
| 198 | `self.entries.read().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 225 | `partial_cmp(&a.0).unwrap()` | → `.unwrap_or(std::cmp::Ordering::Equal)` |
| 238 | `self.causal_graph.read().unwrap()` | → `.expect("RwLock poisoned: causal_graph")` |
| 251 | `self.entries.read().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 275 | `self.entries.read().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 296 | `self.entries.read().unwrap()` | → `.expect("RwLock poisoned: entries")` |
| 312 | `self.causal_graph.write().unwrap()` | → `.expect("RwLock poisoned: causal_graph")` |
| 317 | `self.causal_graph.write().unwrap()` | → `.expect("RwLock poisoned: causal_graph")` |
| 337 | `self.entries.read().unwrap()` | → `.expect("RwLock poisoned: entries")` |

**策略**: RwLock poisoning 在 NeoTrix 中不可恢复（无事务回滚），使用 `.expect()` 比裸 `.unwrap()` 多提供诊断信息，且不改变函数签名。`partial_cmp().unwrap()` → `unwrap_or(Ordering::Equal)` 防 NaN。

### 1.2 `core/nt_core_meta/knowledge_gap_detector.rs` — 14 处

| 行号 | 代码 | 修复方案 |
|------|------|----------|
| 753 | `kb.conn.lock().unwrap()` | → `.expect("Mutex poisoned: KB conn")` |
| 764 | `conn.prepare(...).unwrap()` | → `?` (函数返回 `Vec`，需改签名为 `Result<Vec, String>` 或用 `.unwrap_or_default()`) |
| 771 | `stmt.query_map(...).unwrap()` | → 同上 |
| 805 | `kb.conn.lock().unwrap()` | → `.expect(...)` |
| 815 | `conn.prepare(...).unwrap()` | → 同 764 |
| 822 | `stmt.query_map(...).unwrap()` | → 同上 |
| 833 | `DateTime::from_timestamp(updated, 0).unwrap()` | → `.unwrap_or_default()` |
| 856 | `kb.conn.lock().unwrap()` | → `.expect(...)` |
| 864 | `conn.prepare(...).unwrap()` | → 同上 |
| 870 | `stmt.query_map(...).unwrap()` | → 同上 |
| 881 | `kb.conn.lock().unwrap()` | → `.expect(...)` |
| 893 | `conn.prepare(...).unwrap()` | → 同上 |
| 909 | `kb.conn.lock().unwrap()` | → `.expect(...)` |
| 936 | `conn.execute(...).unwrap()` | → `.unwrap_or(0)` |

**策略**: Mutex/RwLock 用 `.expect()`；SQL prepare/query 在 scan_* 内部函数中（返回 `Vec<KnowledgeGap>`），最简方案是 `.unwrap_or_default()` 空 vec fallback，避免改签名传播。

### 1.3 `l5_cognition/nt_core/nt_governance/skill_validator/mod.rs` — 12 处

| 行号 | 代码 | 修复方案 |
|------|------|----------|
| 198 | `regex::Regex::new(...).unwrap()` | → `lazy_static!` 或 `std::sync::OnceLock` 缓存静态正则 |
| 325 | `regex::Regex::new(...).unwrap()` | → 同上（在 test 代码中，可保留） |
| 327 | `cap.get(1).unwrap()` | → test 代码，保留 |
| 353-401 | `std::fs::create_dir_all/write().unwrap()` | → test 代码，保留 |

**结论**: 非测试 unwrap 仅 1 处（L198 静态正则）。修复为 `OnceLock<Regex>`。

---

## 2. 跨域引用检测

### 2.1 L1→L2 引用（L1 不应依赖 L2）

| 文件 | 引用 | 严重性 |
|------|------|--------|
| `l1_action/nt_act/nt_act_orchestrator/critic.rs:33` | `use crate::l2_perception::nt_world::nt_world_model::TaskType` | **中** — L1 Act 引用 L2 感知类型 |
| `l1_action/mod.rs:10` | `pub use crate::l2_perception::nt_sense::nt_infra_semantic_router` | **低** — re-export，可接受 |
| `l1_action/nt_memory/mod.rs:15` | `pub use crate::l2_perception::nt_sense::nt_memory_spatial` | **低** — re-export |

### 2.2 L1→L3 引用（L1 不应依赖 L3）

| 文件 | 引用 | 严重性 |
|------|------|--------|
| `l1_action/nt_io/nt_io_agent_loop.rs:33` | `use crate::l3_embodiment::nt_shield::nt_shield_propagation_guard` | **高** — L1 IO 依赖 L3 Shield |
| `l1_action/nt_io/nt_io_agent_loop.rs:34` | `use crate::l3_embodiment::nt_shield::nt_shield::redaction::Redactor` | **高** — 同上 |
| `l1_action/nt_memory/nt_memory_kb/nt_memory_pipeline.rs:22` | `use crate::l3_embodiment::nt_shield::nt_shield::self_poison::scan_absorb_text` | **高** — L1 Memory 依赖 L3 Shield |
| `l1_action/nt_memory/nt_memory_kb/nt_memory_pipeline.rs:23` | `use crate::l3_embodiment::nt_shield::nt_shield::receipt::AgentReceipt` | **高** — 同上 |

### 2.3 L5→L1 引用（L5 不应依赖 L1）

| 文件 | 引用 | 严重性 |
|------|------|--------|
| `l5_cognition/nt_mind/nt_mind/mod.rs:20` | `pub use crate::l1_action::nt_memory::nt_memory_kb::bm25` | **中** — re-export |
| `l5_cognition/nt_mind/nt_mind/mod.rs:196-244` | 多处 `pub use crate::l1_action::nt_act::nt_act_trade::*` | **高** — L5 大量 re-export L1 类型 |
| `l5_cognition/nt_mind/nt_mind_background_loop/*.rs` | 多处 `use crate::l1_action::nt_memory/nt_act/nt_io` | **高** — L5 直接调用 L1 函数 |
| `l5_cognition/nt_mind/nt_mind/seal_core/*.rs` | 多处 `use crate::l1_action::nt_act/nt_io/nt_memory` | **高** — SEAL 循环依赖 L1 |

### 2.4 修复建议

| 问题 | 建议 |
|------|------|
| L1→L3 (Shield) | 将 `PropagationGuard`/`Redactor`/`scan_absorb_text` 抽为 L1 traits，L3 实现（依赖反转） |
| L5→L1 (Act trade re-exports) | 将 trade types 移至共享 `types` crate 或 `l4_emotion` 层 |
| L5→L1 (Background loop) | 引入 `ActionDispatch` trait，L5 通过 trait 调用 L1，不直接 import |

---

## 3. EventBus 连接率

| 指标 | 值 |
|------|-----|
| EventBus 总引用（非测试/非注释） | 73 |
| 实际连接/订阅/发射调用 | 8 |
| **连接率** | **11.0%** |
| EventBus struct 定义 | 2 处（`nt_core_event_bus.rs` + `nt_act_eventbus.rs`） |
| `subscribe_layer` / `subscribe_all_layers` | 3 个函数 |

**问题**: EventBus 存在两个定义（`nt_core_event_bus::EventBus` vs `nt_act_eventbus::EventBus`），可能造成混淆。连接率低说明大部分模块绕过 EventBus 直接调用。

---

## 4. 执行计划

| 步骤 | 任务 | 影响范围 |
|------|------|----------|
| 1 | `autobiographical_index.rs` — RwLock unwrap → expect | 12 处 |
| 2 | `knowledge_gap_detector.rs` — Mutex/SQL unwrap → expect/unwrap_or_default | 14 处 |
| 3 | `skill_validator/mod.rs` — 静态正则 → OnceLock | 1 处 |
| 4 | 跨域引用 — 标记 L1→L3 为 TODO（需 trait 抽象） | 4 处 |
| 5 | EventBus — 统一双定义，标记低连接率为架构债务 | 文档 |
| 6 | `cargo check -p neotrix --lib` 验证 | — |
