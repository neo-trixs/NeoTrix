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

## `nt_term_viz::panel` 已就绪（39 测试全绿）

新增 `crates/nt-term-viz/src/panel.rs`：`panel_top` / `panel_bottom` /
`content_line` / `render_panel`，**全部按列宽计算**。

### 开发它时实测到的错位（`entry/consciousness.rs`）
Rust 的 `{:>N}` 按**字符数**补足，而中文占 2 列 ⇒ 各行错位量**不同**：

| 字面 | 字符数 | 视觉宽 | 错位 |
|---|---|---|---|
| `│ 周期      ` | 10 | 12 | **+2 列** |
| `│ 相位(Φ)   ` | 10 | 12 | **+2 列** |
| `│ 相干性    ` | 9 | 12 | **+3 列** |
| `│ 谐振周期  ` | 8 | 12 | **+4 列** |

⇒ **右边框参差不齐**。`panel` 的核心不变量就是
「面板所有行的**视觉宽度**一致」，并有测试逐项断言。

### `panel` 自身修掉的 2 个 bug（写下来备查）
① **`usize` 下溢**：预算需 `total >= 6`（`╭─ ` 3 列 + ` ─╮` 3 列），
   首版守卫写成 `total <= 3` ⇒ `total` 在 4~5 时 `total - 6` **panic**。
② **顶边未补齐**：标题短于预算时只截断不补齐 ⇒ 顶边比底边窄 ⇒
   「所有行宽度一致」不成立。

### 样板迁移：`entry/brain.rs`（**已写好，待验证条件具备时套用**）

⏸ **本轮未提交**：验证时 `neotrix-core` 处于**他窗在途状态**
（`l2_perception/nt_world/social_access/` 有 4~5 处未提交改动，
编译错误数在他窗编辑下波动 7→3→2→4）⇒ **拿不到干净编译**。
⇒ 按本会话纪律「**未验证改动不留共享树**」，已回退，方案记录在此。

迁移要点：
1. `neotrix-core/Cargo.toml` 加 `nt-term-viz = { path = "../crates/nt-term-viz" }`
2. 边框宽度**不再手写** —— 原代码里 `───…` 的长度是手数的，
   只要内容含中文就必然对不上；改由 `panel` 按最长内容的列宽计算
3. ⚠️ **颜色必须先剥离再算宽度**：`info()` 产生 ANSI 序列，
   `display_width` 会把转义算成列 ⇒ 用 `table::strip_ansi` 算宽度、
   `table::pad_to` 补齐，最后对**整行**染色
   （⛔ 不要用 `content_line(colored, w)`，那会让转义污染宽度）
4. 验证方式：`cargo check -p neotrix --lib` 0 error ⇒
   再跑该入口，确认右边框在同一列

### 迁移顺序（更新）
1. 先迁 `entry/brain.rs`（样板，4 处，最小）—— ⏸ 待编译条件具备
2. 再迁 `entry/consciousness.rs`（28 处，错位最明显的那个）
3. 然后 `headless` / `browse` / `clean` / `proxy_cmd` / `standalone`
4. `indicatif` 进度条**不动**（库自带，正确用法）

## 进度更新（第二批，2026-10-03）

已迁 **2 / 7**：`consciousness.rs`(15 行)、`headless.rs`(20 行，修了 emoji + 多行溢出)。

### ⛔ **筛掉了 5 个不该迁的**（先量后动，省掉纯 churn）

| 文件 | 面板行 | 宽字符 | 判定 |
|---|---|---|---|
| `standalone` | 14 | **0** | 纯 ASCII，手数边框**本就正确** ⇒ 不迁 |
| `proxy_cmd` | 8 | **0** | 同上 |
| `todo` | 6 | **0** | 同上 |
| `clean` | 5 | **0** | 同上 |
| `status` | 5 | **0** | 同上 |

⇒ **「有框线字符」不等于「对齐错位」。** 纯 ASCII 面板按字符手数宽度是**正确的**，
迁到 `panel()` 是**无证据的改动** —— 与我本会话反复反对的「为清理而清理」同源。
⇒ 纪律：**迁移前先量宽字符数**，`0` ⇒ 不动。

### `browse.rs`：⛔ 我上一条写错了，此处更正为地面真相

**上一条（错误）**：「三个子面板都**缺底边**、内容行**缺右边框** ⇒ 面板本身残缺，
迁移等于新增功能。」

**地面真相**（用完整字符类 `[╭╰│┌└─]` 重查）：
```
29:  info("╭─ NeoTrix Browser ────╮")
40:  info("╰───────────────╯")     ← 底边在
174: info("╭─ NeoTrix Browser Acts ────╮")
180: info("╰───────────────╯")     ← 底边在
290: info("╭─ NeoTrix Login ────╮")
296: info("╰───────────────╯")     ← 底边在
```
⇒ **三个面板都有底边，「缺底边」是我的误判。**

**误判成因**：上一条用的模式是 `rg '╭─|│ \{|└─'` ——
**字符类里没有 `╰`**，因此三处 `╰─` 全部落选。
⇒ 这是 AGENTS.md **§R-SCAN-1b** 的教科书案例（我在正文里贴过那条规则，又当场犯）：
**模式不完备的搜索只能找候选，不能下结论。**

**真缺陷只有一处**（且是**定位 bug**，不是宽度 bug）：
```
180:  info("╰───────────────╯")            ← 面板底边
200:  println!("│ {} {}", info("Actions:"), actions.len());
                                          ← 带 │ 前缀的内容行，印在底边之后 20 行
```
⇒ 该行掉到**面板外**。修复需理解 :182-199（读文件 + `match … return;`）
与 `:200` 的控制流依赖，**不是搬一行**。

⛔ 我已在同一文件上因 grep 模式不完备错了两次 ⇒ **不做第三次盲改**；
先读全 :182-210 的控制流再动。`sysops.rs` 同样待重查。

### 本轮我自己犯的两处（都当场修）
① 首版给 `headless` 面板加了 `info()` 染色 —— 但**原面板本就是纯文本**
   ⇒ 染色属**超范围行为变更**，已去掉，保持迁移「行为中性」。
② `info` 未导入 —— **又是 `--tests` 抓到的，`--lib` 漏掉**。
⇒ 与上一笔 `E0753` 同机制：**`--lib` 通过 ≠ 全目标通过**。
