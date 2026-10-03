# 分层债务严重度定级（2026-10-03 实测）

> ⭐ 本文档**只做定级与排序**，⛔ 不含实现方案（结构改动须先 `cargo clean && cargo build` 跑两遍，
> 见 `AGENTS.md` §1）。
> ⭐ 全部数据来自 `bash scripts/check-layer-deps.sh` 与逐文件核实，可复现。

## 0. 当前真值

```
$ bash scripts/check-layer-deps.sh --strict
PASS: 0 new violation(s); 14 known/recorded
```

⓰ 注意口径变化：`AGENTS.md` §4.2 记录的是「**8 known**」，实测已长到 **14**。
⇒ 该文档的台账**已陈旧**，按 `R-SCAN-3`（门记录必须带核实时间戳）以本文为准。

## 1. ⭐ 严重度定级标准（按「倒置后能造成什么」排，不按数量排）

| 级 | 判据 | 后果 |
|---|---|---|
| **S1** | **L0 → L5/L6** | ⛔ **近乎循环**：L0 是基座，全仓都依赖它；基座反向依赖顶层意味着「顶层定义基座接口」 |
| **S2** | L1 → L5（跨 ≥4 层） | 高层语义泄漏进动作层，层名出现在业务代码里 |
| **S3** | L1 → L2/L3（跨 1–2 层） | 轻度，facade 可解 |

## 2. ⭐ S1 清单（L0 反向依赖，**最高优先**）

⚠️ 以下**都是真的 `use`**，⛔ 不是注释（已逐条核实，见「§4 注释误报」）。

| 文件 | 反向依赖 | 位置 |
|---|---|---|
| `l0_substrate/ffi/consciousness_tree.rs` | ⛔ **`l6_meta`**（`healing::nt_core_self_test_integration::register_absorbed_modules`） | `:208` |
| `l0_substrate/ffi/consciousness_tree.rs` | ⛔ `l5_cognition` | 基线在册 |
| `l0_substrate/ffi/seal_pipeline.rs` | ⛔ `l5_cognition` | 基线在册 |
| `l0_substrate/nt_core_event_bus.rs` | ⛔ **`l5_cognition`**（`nt_core_dispatch::Dispatcher`） | `:6` |
| `l0_substrate/nt_core_event_bus.rs` | ⛔ `l3_embodiment` | 基线在册 |

⇒ **`nt_core_event_bus.rs` 单独一个文件同时反向依赖 L3 与 L5**，是 S1 里最集中的一处。

### 2.1 为什么这比 L1→L5 严重（S1 vs S2 的实质差别）

- **L1 → L5**：L1 是**顶层消费者**，依赖下层语义尚可理解（虽然违规）。
- **L0 → L6**：L0 是**被所有人依赖的基座**。若基座引用 L6，则
  「编译 L0」 ⇒ 「需要 L6 的类型」⇒ **层的单向性在编译图上被打破**。
  ⭐ 这类倒置会让「先改 L6 再改 L0」成为不可能，**把本可独立的模块焊死在一起**。

## 3. S2 清单（L1 → L5 引擎簇，**16 处，集中在 2 文件**）

| 文件 | 处数 | 需要的符号簇 |
|---|---|---|
| `l1_action/nt_tui_app.rs` | 13 | `NtCrystalTaskLoop` / `NtTaskLoopConfig` / `NtProgressSink` / `NtDemandKind` / `NtLlmAsk` / `NtLlmReply` / `NtTaskFusionError` |
| `l1_action/nt_core_task_dispatcher/nt_dispatcher_core.rs` | 3 | `CrystalCore` / `NtCrystalTaskLoop` / `NtTaskLoopConfig` / `NtTaskLoopReport` |

⛔ **刻意未用 `pub use` 盖掉**（`f5cff5cc` 已把另两簇收敛掉，这簇刻意保留）。
理由：转出引擎簇等于宣告「**L1 驱动认知引擎**」，这是**架构立场**。
⭐ 而 `nt_crystal_core` 的消费者分布恰好支持这个疑问：

| 消费方 | 文件数 |
|---|---|
| L5 内部（同层，合法） | 11 |
| **L1** | **7** |
| `bin` 入口 | 3 |
| `entry` | 2 |
| L0 | 1（⓰ 注释误报，见 §4） |

⇒ **近一半消费者在 L1 之下**。⭐ 真问题不是「怎么去重」，
而是「**`nt_crystal_core` 是否被错放在 L5**」。若它本质是**任务编排引擎**，
那它更接近 L1/L3 而非 L5。⛔ 这需要一次分层裁决，不是 refactor 能顺手解决的。

## 4. ⭐ 注释误报（本轮实测踩到，必须记录）

`l0_substrate/nt_sampler.rs:17` 被「L0 依赖 L5」的朴素扫描命中，实测是**注释**：

```rust
//! `l5_cognition/nt_crystal_core/ctm.rs` 用 `rand::random()`，非确定、无 top-k、
```

⛔ 它只是**在注释里提到**那个路径。⇒ 该文件**不构成**依赖，**不得**改道。

⭐ 印证 `AGENTS.md` §4.2 的 **L13/L16**：「批量改道须自查有无改到注释行」。
⚠️ 更危险的是**反向**：本次差点把一个**合规**文件当成违规去「修」。

## 5. 建议处理次序（**未实施**）

1. ⭐⭐⭐ **S1 的 `nt_core_event_bus.rs`** —— 单点最集中，先查它是否可下沉为 L0 自己的接口
2. ⭐⭐ **S1 的 `ffi/consciousness_tree.rs`** —— `l6_meta` 那个引用在 `:208` 的**函数内** `use`，可能只是测试/自检路径 ⇒ 先定性再动
3. ⭐ **S2：先裁决 `nt_crystal_core` 的层归属**，再谈去重
4. ⛔ 全部属结构改动，**每批须 `cargo clean && cargo build` 跑两遍**

## 6. 已完成的部分（供对照）

| commit | 内容 | 违规数 |
|---|---|---|
| `4796c84e` | L1 facade 转出「LLM 问答契约」簇（3 符号 / 3 文件） | 19 → 16 |
| `f5cff5cc` | L1 facade 转出「人机通道」簇（4 符号 / 2 文件，含一处函数内 `use`） | 16 → ⭐ **14** |

⭐ 两批均按**能力簇**分组而非整包 re-export；每次收缩都是
`check-layer-deps.sh` **自己**检出的（`0 new violation(s)`），⛔ 不是靠改门。