# 目录架构解法 — 2026-09-28

> 回答一个问题：**为什么目录架构与「主代码进化之路」不契合，怎么解。**
> 配套 `EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md`（给节点）。本文给**判据 + 搬砖顺序**。
> 全部数据 2026-09-28 实测（`find`/`wc`/`grep`/`bash scripts/check-layer-deps.sh`）。

---

## 一、结论先行

**不契合的根因不是「目录乱」，是「目录里藏着第四棵树，而且它比分层树更接近主代码进化之路」。**

三个可测的事实：

1. **`neotrix-core/src/neotrix/` 是一棵嵌在 core 内部的、**不参与 L0–L6 分层**的第二棵 NeoTrix。** 129 文件 / 43,834 行，占 core 的 **约 9%**。`scripts/check-layer-deps.sh` 的 `SRC="neotrix-core/src"` 扫得到路径，但**它的规则只对 `l*_` 目录生效** ⇒ 这棵树**完全逃过分层门**。
2. **它是活的。** `nt_crystal_core`（52 文件 / 21,134 行）被 L1 **6 个文件**消费，`nt_jev`（14 文件 / 4,312 行）被 `nt_crystal_core` **5 个文件**消费。
3. **新能力进来时被吸进这棵树，不是吸进分层树。** 因为它是**上一轮吸收的落点**（`nt_jev` / `nt_file_ability` / `nt_crystal_core` 都是吸收产物），而 L 层是**旧有**的。

> ⇒ **主代码进化之路 = 吸收 → 落到 `neotrix/`。目录架构 = L0–L6。两者不同源，于是「目录不契合」。**

---

## 二、实测证据

### 2.1 第二棵树的规模与构成

```
neotrix-core/src/neotrix/          129 files   43,834 lines
├── nt_crystal_core/                52 files   21,134   ← 活路径（L1 6 消费者）
├── nt_file_ability/                44 files   14,014
├── nt_jev/                         14 files    4,312   ← 活路径（crystal_core 5 消费者）
├── ffi/                            12 files    2,910
├── error_conversions.rs
├── nt_capability_bridge.rs
├── nt_core_error.rs
├── nt_core_event_bus.rs
├── proxy_daemon_wrapper.rs
```

### 2.2 命名空间的病理症状

`neotrix::neotrix::` — **同一个 crate 名出现两次**，实测 **20 处**：

| 文件 | 处数 |
|---|---|
| `neotrix-core/src/bin/ntcode.rs` | 多处 |
| `neotrix-core/src/bin/nt_train_export.rs` | 多处 |
| `neotrix-core/src/entry/headless.rs` `:292` | 1 |
| `neotrix-core/src/entry/interactive.rs` `:74` `:308` | 2 |
| `neotrix-core/src/bin/experience/exp_distill.rs` | — |
| `neotrix-core/src/entry/proxy_cmd.rs` | — |
| `src-tauri/src/ntcode/streaming.rs` | — |

> `use neotrix::neotrix::nt_crystal_core::…` 读起来像一个包依赖另一个包。**实际上 `neotrix` 是外层 crate，`neotrix` 是内层目录模块。**
> 这个命名冲突会**持续误导定位** —— 正好是 `nt_locate.py` 索引把它与别的混同的原因。

### 2.3 分层门测不到它

```bash
$ grep -n 'SRC=' scripts/check-layer-deps.sh
30:SRC="neotrix-core/src"
# 规则只解析 l0_substrate..l6_meta —— neotrix/ 不匹配任何分支
$ bash scripts/check-layer-deps.sh
violation sites: 84   baseline entries: 92   resolved: 8
PASS: 0 new violation(s); 84 known/recorded.
```

**⇒ 一个 43,834 行、活着的、不受任何架构门约束的子树。** 这是「目录不契合」最直接的物证。

### 2.4 与 2.3 并列的第二处失配：`nt_` 前缀规约

`AGENTS.md` 规定「所有模块名用 `nt_` 前缀」。实测 **`neotrix-core/src` 下 1,644 个 `.rs` 文件不以 `nt_` 开头**（排除 `mod.rs`/`lib.rs`/`main.rs`）：

```
neotrix-core/src/pipeline/route.rs
neotrix-core/src/pipeline/execute.rs
neotrix-core/src/pipeline/intake.rs
neotrix-core/src/pipeline/output.rs
neotrix-core/src/pipeline/main_pipeline.rs
neotrix-core/src/pipeline/validate.rs
neotrix-core/src/l0_substrate/nt_core_platform/pipeline_registry.rs
neotrix-core/src/l0_substrate/nt_core_platform/health.rs
...
```

> **规约与现实差 1,644 个文件。** 规约因此**不产生约束力** —— 没有门检查它。
> 这与 R-SCAN-3 是同一个病：**写下规范而不核实现状，规范就变成装饰。**

### 2.5 三棵记忆树（`同义不同型` 的第 4 次实例）

| 位置 | 规模 | 角色 |
|---|---|---|
| `l4_emotion/nt_memory/` | **236 文件 / 82,071 行** | 存储层真典 |
| `l5_cognition/nt_mind/` | **422 文件 / 132,990 行** | 认知/经验树 |
| `l6_meta/memory/` | 7 文件 / 2,048 行（含 `nt_memory_experience_tree.rs`） | **投影？重复？** 未裁决 |

### 2.6 1,644 个文件没前缀，但 `nt_` 文件名本身也不是分层信号

`neotrix-core/src/l0_substrate/nt_core_platform/pipeline_registry.rs` 有 `nt_` 前缀，
却和 `l0_substrate/nt_core_platform/orchestrator.rs` 一起构成 DIR-AUDIT 认定的**第 3 个 orchestrator**。

⇒ **前缀规约管不到「谁属于哪一层」，只管「叫什么」。**
前缀合规 **≠** 架构正确。这是两条正交的轴，混用会得出错误结论。

---

## 三、根因诊断

把上面 6 条压成一句：

> **目录结构编码的是「系统长什么样」（L0–L6 分层）。**
> **主代码进化之路编码的是「它怎么长大」（吸收 → 落点）。**
> **两者用了两套坐标，且没有一张映射表。**

历史上它们碰巧对齐过，因为吸收产物都落在 `neotrix/`。但：
- `nt_crystal_core` 已经是**全系统的决策/意识主干**（L1 的 TUI、stdin、dispatcher 都依赖它），
  却住在一个**不声明层归属**的目录里；
- 而它的「层归属」按依赖事实应该是 **L5 认知**（`CrystalConsciousness` + JEV 决策）。

**⇒ 它住在 `neotrix/`，但它在依赖图上是 L5。这就是不契合的精确位置。**

---

## 四、解法

### P0 — 让门看得见（1 天，零风险，不搬任何文件）

| # | 动作 | 验收 |
|---|---|---|
| 1 | **`check-layer-deps.sh` 扩到 `neotrix/`** —— 给第二棵树一个**显式的层归属声明**（见 P1 的 `layer_of` 映射），先入基线 | `bash scripts/check-layer-deps.sh --strict` 仍 **PASS 0 new** |
| 2 | **加 `scripts/check-naming.sh`** —— 检查 `nt_` 前缀规约，先**报告不阻塞**，打印当前 1,644 的真实数字 | 输出与 §2.4 实测一致 |
| 3 | **建 `docs/architecture/OWNERSHIP.md`（唯一裁决表）** —— 每个「重复类型/多棵树」一条：真典 / 投影 / 待裁 | 见 P2 |

**为什么先做这个**：门是**唯一**能让目录结构对文档说话的东西。
没有门，任何搬砖方案都会在下一个 agent 眼里退化成「他改了目录」。

### P1 — 声明层归属（2-3 天，纯元数据，不动代码）

给 `neotrix/` 下 4 个子树**显式声明**它属于哪一层，落成一张机器可读的表：

```jsonc
// .neotrix/layer-map.json
{
  "neotrix/nt_crystal_core":  { "layer": "l5_cognition", "role": "primary",
    "note": "52 files/21134 lines; L1 有 6 个活消费者 (nt_dialogue_tui, nt_tui_app,
             nt_stdin_human, nt_crystal_llm_bridge, nt_free_pool,
             nt_core_task_dispatcher/nt_dispatcher_core) —— 禁止当死代码删" },
  "neotrix/nt_jev":          { "layer": "l5_cognition", "role": "primary",
    "note": "决策三原语；被 nt_crystal_core 5 处消费" },
  "neotrix/nt_file_ability": { "layer": "l1_action",     "role": "primary" },
  "neotrix/ffi":             { "layer": "l0_substrate",  "role": "primary" }
}
```

> **这一步是本解法的核心**。它把「目录位置」与「层归属」**解耦**：
> 目录继续是 `neotrix/`（不动路径，零回归风险），
> 但**层归属变成显式数据**，分层门立刻能管它。

**为什么先解耦而不是直接搬目录**：直接搬 `nt_crystal_core` 到 `l5_cognition/` 要改
`lib.rs` 声明 + **52 个文件内的 `crate::neotrix::…` 路径** + 6 个 L1 消费者
+ `src-tauri` 2 处。**收益是「看起来整齐」，成本是一次大回归。**
而层归属一旦显式，**目录就不再承载层信息**，整齐度不再是架构问题。

### P2 — 唯一裁决表（1-2 天）

`docs/architecture/OWNERSHIP.md` 逐条裁决，每条带**构造点证据**（不是 `mod` 声明 —— 见下方教训）：

| 待裁决 | 已实测的裁决 | 证据 |
|---|---|---|
| 3 棵记忆树 | 存储真典 = `l4_emotion/nt_memory`；经验树真典 = `l5_cognition/nt_mind/nt_mind/experience_tree`；`l6_meta/memory` **待裁** | `experience_tree/mod.rs:215/241/299/355/452` 五段协议只此一处 |
| 3 个决策引擎（死） | `crates/neotrix-decision-engine` / `l5_cognition/nt_decision_engine.rs` / `nt_mind/decision_engine` —— **均无生产消费者**，但**不要现在删**（先接线或先标弃用） | 构造点全在 `#[test]` 内 |
| `nt_jev`（活） | **真典**，与 `nt_crystal_core` 同链 | `nt_eval_loop.rs:13` / `nt_jev_agentjev.rs:17` / `nt_jev_calibration.rs:18` / `nt_crystal_dialogue.rs:40` / `nt_crystal_task_fusion.rs:41` |
| 4 份 `CapabilityRegistry` | 真典 = `crates/nt-core-capability-tree/src/registry.rs:60` | 唯一有 `experience_targets` + CI `capability-truth` 在用 |
| 4 份 `SkillRegistry` | 真典 = `crates/neotrix-gateway/src/skill_registry.rs:157`（被 `nt_crystal_serve` 用）；门面 = `neotrix-core/src/skill_registry.rs:14` | — |
| 20 处 `Orchestrator*` | 待裁（路线图 0.3） | — |
| `nt_io_desktop/` 空壳（2 文件 340 行，`mod.rs` **8 行**） | 待补（路线图阶段 2 落点） | — |

> **⚠️ 本轮最贵的教训，必须写进表头**：
> `nt_jev` 曾被 DIR-AUDIT §六归入「三个全未接线的决策引擎」。
> **实测它是活路径。**
> **「导出 ≠ 调用」已错过 3 次（`CapabilityRegistry` 4 份 / `SearchResult` 9 份 / 三个决策引擎），
> 第 4 次是 JEV —— 只不过这次错的方向相反：它是活的。**
> ⇒ **裁决表每一条必须以「构造点」(`::new(`) 为准，不以 `mod` 声明为准。**

### P3 — 收敛（2-4 周，与路线图 0.1/0.2 合并做）

按路线图阶段 0 执行（`CapabilityRegistry` 4→1 · `ToolRegistry` 3→1 · `SkillRegistry` 4→1），
收敛完再跑 `check-layer-deps.sh --update-baseline` 把 84 → 更低。

### P4 — 目录名收敛（低优先，可选）

- `neotrix-core/src/neotrix/` → 重命名 `neotrix-core/src/kernel/` 或 `src/platform/`：
  **消除 `neotrix::neotrix::` 双命名**（20 处），**不改任何层归属**（P1 已解耦）。
- `neotrix-core/src/pipeline/` 与 `l0_substrate/nt_core_platform/pipeline_registry.rs`
  **合并去重**（路线图 0.3 第 ④ 项）。
- 根目录：`ARCHITECTURE-EVOLUTION-ROADMAP.md` / `FUSION-ARCHITECTURE.md` 归档到
  `docs/architecture/_superseded/`（承接 DIR-AUDIT P2-9，**本轮仍未做**）。

---

## 五、建议顺序与理由

```
P0 建门（1 天）
  └─→ P1 声明层归属（2-3 天，零代码改动）
        └─→ P2 唯一裁决表（1-2 天）──→ 防止下一轮误删 nt_crystal_core
              └─→ P3 收敛（2-4 周）──→ 刷新基线
                    └─→ P4 目录改名（可选，最后做）
```

**三条判据**：

1. **先建门，后搬砖。** 门是唯一让目录对文档说话的东西。
2. **先解耦层归属，后改路径。** 层归属变成显式数据后，「目录整齐」不再是架构问题 —— 于是**搬目录从「必须」降级为「可选」**，P4 可以永远不做而不损失架构正确性。
3. **P2 必须在 P3 之前。** 收敛 = 删重复实现 = **最容易误删活代码的时刻**。裁决表（按构造点取证）是唯一的护栏。

---

## 六、本解法**不做**什么，以及为什么

| 不做 | 理由 |
|---|---|
| 不把 `neotrix/` 强行拆进 L0–L6 | 129 文件 / 43,834 行、20 处 `crate::neotrix::` 路径、6 个 L1 消费者 + 2 个 `src-tauri` 消费者。**收益是观感，代价是一次大回归**，而 P1 之后观感不再是架构问题 |
| 不现在删那 3 个死决策引擎 | 删掉会造出「已收敛」的假象；**R-P79 要求外部技术接进生产**，死代码的正确处置是标弃用 + 接线，不是删除 |
| 不为 1,644 个文件批量补 `nt_` 前缀 | 纯改名 churn，零架构收益，且会掩盖 P1/P2 的真问题。**只建门（P0-2）让数字可见** |
| 不动 `models/` `sessions/` `thirdparty/` | 承接 DIR-AUDIT P2-11/12/13，与本文主线（层归属）无关，另案 |

---

## 七、验收（可机器检查）

```bash
# P0-1：第二棵树被分层门覆盖，且不新增违规
bash scripts/check-layer-deps.sh --strict            # 期望 PASS 0 new

# P0-2：命名规约的真实数字可见
bash scripts/check-naming.sh                          # 期望打印 ~1644

# P1：层归属可解析
python3 -c "import json;d=json.load(open('.neotrix/layer-map.json'));print(len(d),'trees declared')"

# P2：裁决表覆盖全部已知重复类型
grep -c '^##' docs/architecture/OWNERSHIP.md          # 期望 >= 7

# P3：收敛后重复计数
for t in CapabilityRegistry ToolRegistry SkillRegistry; do
  printf '%s=' "$t"
  grep -rn "pub struct $t" neotrix-core/src crates src-tauri/src 2>/dev/null | wc -l
done                                                  # 期望 1 1 1
```

---

## 八、与既有文档的关系

| 文档 | 处置 |
|---|---|
| `DIR-AUDIT-2026-09-27.md` | **保留**（依赖拓扑 + 8 类重复的实测仍有效）。但其 §六「三个决策引擎全未接线」**需加限定**：`nt_jev` 是活路径 |
| `EVOLUTION-ROADMAP-CODE-NODES-2026-09-27.md` | **归档** `_superseded/`，由 09-28 版取代（新增 0.2 证伪门 / 阶段 1 / 阶段 2 五项） |
| `ARCHITECTURE.md:97`（`nt_computer` 称「计算集群」） | ⛔ **实测为假** —— 是 filesystem/process trait，`screenshot()` 默认返回 `None`，neobot 侧 `NoopBackend` 是唯一后端。**本轮已二次确认仍未修** |
| `ARCHITECTURE-MAP-ROADMAP-V2.md` | §1–§7 数字永久陈旧（`AGENTS.md` 已注明）；**其能力声明应改用 OpenAdapt 的 claims-registry 形态** —— 每条声明指名一个存在的测试文件 |
