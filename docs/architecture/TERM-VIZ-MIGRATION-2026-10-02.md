# 终端渲染迁移清单（2026-10-02）

## 背景

新建了 `crates/nt-term-viz`（共享原语：宽度感知对齐 / 方块密度条 / 迷你图 / 树连线），
并用它替换了 `nt-core-capability-tree/src/cli.rs` 的内联硬编码（见本会话提交 `refactor(capability-tree)` 那笔）。

## ⛔ 为什么**没有**做成门

我先写了一版门（`check-term-viz.py`），跑全仓得到 **336 处**，逐一判读后**删除**了它：

| 类别 | 实例 | 为什么不该拦 |
|---|---|---|
| 注释行 | 20+ 个 py 脚本的 `# ── 规则表 ──` | 我的注释剥离有 bug（`i > 0` 守卫导致行首 `#` 未剥） |
| 面板边框 | `╭─ Title ─╮`、`── 分节 ──` | **不是树连线**，`tree_connector` 不覆盖 |
| Python/JS 脚本 | `nt_topology.py` 的树打印 | **无法**使用 Rust crate |
| 第三方进度条 | `indicatif` 的 `█▉▊▋▌▍▎▏` | 库自带，正确用法 |

⇒ **一道 336 命中的门 = 会被立刻关掉的门。**

⇒ 这些是**迁移任务列表**，不是门违规。做成门只会阻断所有提交。

## 真实可迁移的站点（仅 `.rs`、仅树连线 `├`/`└`）

共 **28 个文件 / 138 处树连线**（该范围不含面板边框与注释）：

| 文件 | 树连线 | 框线合计 |
|---|---|---|
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/dao_engine.rs` | 14 | 28 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/dao_engine.rs` | 14 | 28 |
| `neotrix-core/src/l5_cognition/nt_mind/foundation/cleanup_engine/mod.rs` | 12 | 41 |
| `neotrix-core/src/l2_perception/nt_world/source/unified.rs` | 7 | 535 |
| `neotrix-core/src/l2_perception/nt_world/source/mod.rs` | 7 | 678 |
| `neotrix-core/src/l5_cognition/nt_mind/self_improvement/mod.rs` | 7 | 22 |
| `neotrix-core/src/l1_action/nt_act/nt_act_workspace_isolator.rs` | 5 | 278 |
| `neotrix-core/src/l1_action/nt_act/nt_act_scheduler.rs` | 5 | 278 |
| `neotrix-core/src/l1_action/nt_io/nt_io_provider/catalog/model_pool.rs` | 5 | 347 |
| `neotrix-core/src/l1_action/nt_file_ability/doc_parse.rs` | 5 | 10 |
| `neotrix-core/src/l5_cognition/nt_core_context_engine.rs` | 5 | 278 |
| `neotrix-core/src/entry/consciousness.rs` | 4 | 522 |
| `neotrix-core/src/l1_action/nt_act/agent_loop/mod.rs` | 4 | 175 |
| `neotrix-core/src/l1_action/nt_media/streaming/mod.rs` | 4 | 406 |
| `neotrix-core/src/l6_meta/lib.rs` | 4 | 706 |
| `neotrix-core/src/l6_meta/nt_meta/nt_evolution_eval.rs` | 4 | 376 |
| `neotrix-core/src/l6_meta/nt_core_self/seal/constitution_gate.rs` | 4 | 8 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/seal_core/self_iterating/pipeline/nt_assemble.rs` | 4 | 12 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_game/mod.rs` | 4 | 12 |
| `neotrix-core/src/l1_action/nt_io/universal_model/mod.rs` | 3 | 258 |
| `neotrix-core/src/l1_action/nt_io/nt_io_llm/mod.rs` | 3 | 256 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/evolution/goal_loop/loop_impl/execution.rs` | 3 | 7 |
| `neotrix-core/src/l0_substrate/nt_core_axiom_tree.rs` | 2 | 4 |
| `neotrix-core/src/l1_action/nt_io/nt_io_proxy_server.rs` | 2 | 161 |
| `neotrix-core/src/l1_action/nt_io/nt_io_multimodal_transform/nt_text_transform.rs` | 2 | 216 |
| `neotrix-core/src/l6_meta/nt_core_self/evolution_analysis.rs` | 2 | 344 |
| `neotrix-core/src/l5_cognition/nt_cognition_facade.rs` | 2 | 285 |
| `neotrix-core/src/l5_cognition/nt_mind/nt_mind/benchmark.rs` | 1 | 159 |

## 建议顺序

1. **先迁 `entry/` 下的面板渲染**（brain/browse/clean/consciousness/headless 等）：
   它们是 `╭─…─╮` 面板，**需要给 nt-term-viz 补一个 `panel()` 原语**（含左右边框与宽度计算），
   否则迁不动。

2. 再迁真正的**树连线**站点（`dao_engine.rs` ×2、`execution.rs`、`nt_assemble.rs`、
   `nt_core_axiom_tree.rs`）。

3. `indicatif` 进度条**不动**（库自带，正确用法）。

## 本轮的教训

我在写门之前**没有**先估命中规模 ⇒ 写了才发现 336 处 ⇒ 其中大部分不可修。

⇒ 纪律：**门的判据必须先在小样本上验证命中数**，
   一个命中上百处的「门」要先假设自己设计错了，而不是先合入再解释。
