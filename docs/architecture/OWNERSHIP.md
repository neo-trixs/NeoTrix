# 唯一裁决表 — 2026-09-28 建立

> **填写规约（不可协商）**：每条以**构造点**（`Type::new(` / `Type {` 的**调用点**）为证，
> **不以 `pub mod` 声明、`pub use` 导出、名字相同为证。**
>
> **「导出 ≠ 调用」已错过 4 次**：`CapabilityRegistry` ×4 份 · `SearchResult` ×9 份 ·
> 三个决策引擎 · **JEV（本次方向相反：它是活的）**。
>
> 配套：`.neotrix/layer-map.json`（层归属）· `BATCH-FIX-CHECKLIST-2026-09-28.md`（执行顺序）。

---

## 1. `ToolRegistry` —— 4 份，其中 2 份**同名正交**（不是冗余）

| 位置 | 字段集 / 语义 | 构造点证据 | 裁决 |
|---|---|---|---|
| `l1_action/nt_act/tool_registry.rs:85` | 运行期：inner + capability_tools 索引 + `ToolStats` 统计 | 770 行真实 dispatch + stats | ✅ **真典**，保留 |
| `l5_cognition/nt_core_gate/nt_tool_registry.rs:11` | **构建期**：`HashMap<String, ToolSpec>`，承载**可逆性**（`reversible` / `irreversible`） | 🔴 `l3_embodiment/nt_shield_enforcer.rs:390` `ToolRegistry::new().register(ToolSpec::reversible("write_file","undo_file"))`，`LazyLock` 静态 | ✅ **活，正交，保留** ⛔ **禁止删除** |
| `l2_perception/nt_world/crawl/agentic_browse.rs:34` | crawl 域局部：`Vec<_ToolAction>` | 同文件 `:65` `:302` `:318` `:331` `:410` | ✅ **活，自包含，保留** |
| `crates/neotrix-gateway/src/gate.rs:1273` | — | 已删（`7664ecd8`） | — |

> **⛔ 撤销 2026-09-27 路线图 0.2 的「删 `nt_core_gate/nt_tool_registry.rs`（45 行 stub）」。**
> 它不是 stub：45 行但**职责明确**，且 `nt_shield_enforcer` 依赖它做**写操作可逆性判定**。
> 「可逆性」与「运行期计数」是**正交轴**，字段集不同 ⇒ 不合并。
> 它的注释还自述 *"从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更"*，而 `mod.rs:28` 仍有
> `pub mod nt_tool_registry;` + `:35` `pub use nt_tool_registry::*;` —— 这是**搬移未收尾**，不是冗余。

---

## 2. `CapabilityRegistry` —— 4 份，其中 2 份**确认零消费者**（真冗余）

| 位置 | 形态 | 消费者 | 裁决 |
|---|---|---|---|
| `crates/nt-core-capability-tree/src/registry.rs:60` | `IndexMap` + `experience_targets` + `maturity_audit` | CLI 依赖 DAG + CI `capability-truth` | ✅ **真典** |
| `l0_substrate/nt_core_capability_types.rs:533` | `HashMap<Arc<dyn>>` + by_domain/by_layer | `l2_perception/nt_world/ocr/mod.rs:405` · `l1_action/nt_act/nt_act_trade/capability_registry.rs:171` | ✅ **活**，保留 |
| `neotrix/nt_file_ability/capability.rs:185` | `Vec<Arc<dyn>>` 精简版 | **零**（精确搜索 0 命中；`l5_cognition/.../registry.rs:454` 注释独立佐证） | ⛔ **真冗余 → B-1 删除** |
| `l5_cognition/nt_core/capability/registry.rs:462` | `Vec<Capability>` + tag_index | **仅自测** | ⛔ **可删 → B-2**（删前查 `mod.rs` 的 `pub use`） |

---

## 3. `SkillRegistry` —— 4 份

| 位置 | 角色 | 裁决 |
|---|---|---|
| `crates/neotrix-gateway/src/skill_registry.rs:157` | `Arc<RwLock>` + `search_paths`，被 `nt_crystal_serve` 用 | ✅ **真典** |
| `neotrix-core/src/skill_registry.rs:14` | L3 路由门面 | ✅ 保留（门面 ≠ 重复） |
| `crates/neotrix-types/src/core/skill.rs:54` | `Skill`（带 `reuse_count` / `total_reward`） | 🔵 归一对象 |
| `crates/neotrix-types/src/core/skills/mod.rs:25` | `SkillDefinition`（带 `tier` / `entry_point`） | 🔵 包内自重复 → **B-3 收敛** |

> **注意**：`Skill`（已演化实例）与 `SkillDefinition`（声明清单）**字段集不同 ⇒ 概念不同**。
> DIR-AUDIT §二「同名 ≠ 同类型」第 2 次实例。B-3 只做**注册表**归一，**不动这两个结构体**。

---

## 4. 决策引擎 —— 3 死 1 活 ⛔（**本表最重要的一条**）

| 位置 | 状态 | 构造点证据 |
|---|---|---|
| `crates/neotrix-decision-engine/` | ⛔ **死** | `gateway/Cargo.toml:13` 该 dep 为 `optional`，feature `decision-engine`(:29) 仅 `full`(:28) 包含，**全仓无人启用**；唯一消费者是自身 113 测试 + `examples/` |
| `l5_cognition/nt_decision_engine.rs:351` | ⛔ **死** | `DecisionEngine::new()` 全仓仅 `:611` `:620` `:644`，**全在 `#[test]` 内** |
| `nt_mind/nt_mind/decision_engine/`（scorer/recommender） | ⛔ **死** | `WeightedScorer`(scorer.rs:26) / `DecisionRecommender`(recommender.rs:28) 外部引用 = 0 |
| **`neotrix/nt_jev/` + `neotrix/nt_crystal_core/`** | ✅ **活（生产）** | L1 **6 个消费者**（`nt_dialogue_tui.rs` `nt_tui_app.rs` `nt_stdin_human.rs` `nt_crystal_llm_bridge.rs` `nt_free_pool.rs` `nt_dispatcher_core.rs`）+ `nt_crystal_core` 内部 5 处 |

> ⛔ **DIR-AUDIT §6「三个决策引擎全未接线」仅对那 3 个成立。**
> **禁止把 `neotrix/` 当死代码清理** —— 会误删 52 文件 / 21,134 行 + L1 的真实依赖。
> 完整依据见 `.neotrix/layer-map.json`。

---

## 5. 记忆树 —— 3 棵，2 真典 1 待裁

| 位置 | 规模 | 裁决 |
|---|---|---|
| `l4_emotion/nt_memory/` | 236 文件 / 82,071 行 | ✅ 存储真典 |
| `l5_cognition/nt_mind/nt_mind/experience_tree/` | 5 段协议 `:215` `:241` `:299` `:355` `:452` 全在此一处 | ✅ 经验树真典 |
| `l6_meta/memory/` | 7 文件 / 2,048 行 | 🔴 **待裁**（C-3） |

---

## 6. `Orchestrator` 家族 —— 20 处定义，C-2 处理

`l0_substrate/nt_core_platform/orchestrator.rs` · `neotrix/nt_crystal_core/{memory_orchestrator,nt_orchestrator}.rs`
· `l1_action/nt_act/nt_act_trade/orchestrator{,_v2}.rs` · `nt_act_orchestrator/mod.rs`（4 个定义）
· `actions/orchestration/production_orchestrator.rs` · `l6_meta/nt_auto_orchestrator.rs`
· `nt_mind_background_loop/consciousness_orchestrator.rs` · `nt_core_gwt/module_def.rs`
· `nt_core/capability/nt_act_orch_patterns.rs` · `l4_emotion/nt_memory/nt_memory_kb/memory_orchestrator.rs`

真典候选：`neotrix-core/src/pipeline/`（7 文件 / 1,130 行，D/E/B/A/R/X 阶段命名，对齐 BLUEPRINT D-06）。

---

## 7. `nt_` 前缀规约 —— 规约 vs 现实差 **1,644** 个文件

`bash scripts/check-naming.sh` 实测 **1,644**（口径：排除 `mod.rs`/`lib.rs`/`main.rs`）。

> ⚠️ **前缀合规 ≠ 架构正确**：`nt_` 命名的文件照样可能错层
> （例：`l0_substrate/nt_core_platform/orchestrator.rs` 是第 3 个 orchestrator）。
> 两条轴正交 —— 命名管「叫什么」，层归属管「属于哪一层」（`.neotrix/layer-map.json`）。

---

## 8. 门记录（每次改码后必须刷新，R-SCAN-3）

| 门 | 2026-09-28 实测 |
|---|---|
| `nt_lock_audit.py neotrix-core/src` | **0** exit=0 @ 14:45 |
| `nt_lock_audit.py crates/neotrix-neobot/src` | **0** exit=0 @ 14:45 |
| `check-layer-deps.sh --strict` | **PASS 0 new**（84 sites / 92 baseline，已解 8） |
| `check-naming.sh` | **1,644**（advisory，A-2 新建） |
| `check-truth-surface.sh --strict` | 本地红 = 他窗 WIP，**干净检出恒 exit=0**（DIR-AUDIT §6.1） |
| `nt_mem_gate.sh` | ⛔ **exit 2 BLOCKED**（free_pages 45,503→8,250） |
