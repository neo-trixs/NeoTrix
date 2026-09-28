# handoff — 算法萃取 + 目录归档（2026-09-28）

> 本会话在共享工作区与另一窗口并行作业。**凡标「他窗」的都不是本会话的改动。**

## 1. 一句话

从 `crates/` 的 2 个孤立 crate 里萃取出 **8 个通用算法**进主代码 L0–L6，
把 4 个 crate + 2 个顶层目录归档到 `~/Downloads/Neo/neotrix-archive/`，
裁决 2 笔架构债。顶层目录 **17 → 11**，`crates/` **13 → 9**。

## 2. 萃取清单（7 个文件被 git 识别为 rename，非增删）

| 萃取物 | 行 | 落点 | 主代码此前的空白 |
|---|---|---|---|
| `nt_fov.rs` | 284 | `l2_perception/` | 无任何 FOV 实现 |
| `nt_flow.rs` | 398 | `l3_embodiment/` | 只有单源 A*，无多源流场 |
| `nt_utility.rs` | 111 | `l5_cognition/` | 无通用效用选择器 |
| `dungeongen.rs` | 527 | `nt_game/world/` | — |
| `nt_net.rs` | 287 | `l6_meta/` | 权威房间协议 |
| `nt_commands.rs` | 141 | `nt_game/` | `persistence/replay.rs` 全文件 **0 个 sort** |
| `nt_clock.rs` | 86 | `nt_game/` | 无固定步长 |
| `nt_sampler.rs` | 244 | `l0_substrate/` | `top_k`/`top_p`/`repetition_penalty` **全是 config 透传，零实现** |

## 3. 两笔债的裁决

### 债1 — 度量冲突：统一到欧氏

`nt_flow` 原对角步不乘 √2（Chebyshev），但它自己的 A\* 启发式是 octile
（`a + 0.41421356b` = √2−1）——**Chebyshev 边代价配 octile 启发会高估，
返回次优路径**。这不是风格问题，是它出厂自带的 bug。

处置：`nt_flow` 两处边代价 ×`SQRT_2`；`nt_astar` 两处字面量 `1.414` 改用同一常量。
3 个断言具体距离值的测试改为**推导式 + 容差**。改动前实测两者**均 0 消费者**。

### 债2 — ECS「重复」：前提有误，划清作用域

`nt_game/ecs/`(291) vs L0 `nt_ecs.rs`(1,362) **不是重复**：
前者独有 tag 系统与 priority 调度，后者独有 SoA/ArchetypeId。
`render/scene.rs` 还靠它做场景图父子结构。**删掉会丢能力**，故不合并，
在该文件顶部立裁决文档。

## 4. 归档（`rsync -an -c` 逐字节 **0 差异**）

`crates/neotrix-abilities`(30) `neotrix-game`(37) `neotrix-decision-engine`(24)
`nt-lang`(6) `games/`(38) `fuzz/`(4) `thirdparty/`(7) `datasets/`(101M)
`models/{agent-jev,qwen3-06b,qwen35-4b-uncensored}`(7.2G) `sessions/` 未入库 232 件

## 5. ⚠️ 踩过的坑（都已有回归门或注释锁定）

1. **`models/training/jev_platts.json` 是雷** —— `nt_jev_calibration.rs:476` 用
   `include_str!` 编译期嵌入，且 `:590-595` 有回归门注释「曾被归档误搬后剩 `{}`
   空壳」。该文件**原地钉死**，勿动。
2. **归档顺序**：必须**先抽 `NtSampler` 再归档 decision-engine**，否则唯一真空算法
   一起丢。
3. **`nt_gen_model.rs` 973 行从未编译** —— 批量抢救提交 `e75272be` 加了文件漏了
   `mod` 声明。这是「入库 ≠ 编译」，与 layer-map `_rule` 的「导出 ≠ 调用」同源。
4. **`check-forbid-coverage.sh` 硬编码 crate 名单** —— 归档任何 crate 都必须同步
   改它，否则门报 `NO-FORBID`/漏检。本轮改了 2 次。
5. **反查消费者不能只扫 `Cargo.toml`** —— `neotrix-audit` 与
   `nt-core-capability-tree` 的 Cargo 反查显示「零消费者」，实为
   **CI 门在用**（`nt-audit.yml:30-31`、`ci.yml:224-226`）。须一并 grep
   `.github/workflows/`。

## 6. 同名类型消歧（第二轮，commit `25118be7` + 待提交）

主代码存在**多处同名类型**。逐目录 grep 确认它们**分处互斥作用域、无任何文件同时
引用 2 份** —— 不是功能冲突，是**检索歧义**（`grep ReasoningTrace` 得到 4 个语义
完全不同的结果）。故只改名，不动结构。

| 原名 | 处数 | 处置 |
|---|---|---|
| `ReasoningTrace` | 4 | 保留 crate 通用名作规范名；`crawl`→`CircuitTrace`、`seal_core`→`ProcessStageTrace`、`reason`→`ReasoningRecord` |
| `ReasoningMethod` | 3 | 同上；`crawl`→`CircuitMethod`、`reason`→`ReasoningTaxonomy` |
| `ReasoningStep` | 5 | 同上；`nt_core_ttc`→`TtcStep`、`control_distillation`→`ControlDistillStep`、`cross_domain`→`RefineStep` |
| `TraceSource` | 2 | `seal_core`→`ProcessStageTraceSource` |

**净结果：11 个同名类型消歧，每概念收敛到 1 处。零结构改动、零行为改变。**

### ⚠️ 更正一处不实自述（重要）

`crates/neotrix-reasoning/src/reasoning_core.rs:3` 原本写着
「消除 4 处 ReasoningTrace 重复定义」—— **事实是该统一类型从未被主代码采纳**，
4 处一直并存。逐字段核对后证明**根本不可统一**：

- `crawl` 那份的 `steps` 是 `usize`（**计数**），统一体里是 `Vec<ReasoningStep>`（**列表**）—— 语义不同，不是超集。
- `reason` 那份有 `prompt` / `perspective_lens` / `error_context` / `success` / `reasoning_type` 共 **5 个字段**，统一体**均无对应** —— 强行统一要往里塞 4 个使用方都用不上的字段。
- 三份 `ReasoningMethod` 的**变体集本质不同**（`reason` 那份有 `Direct`/`EdgeCaseFocus`/`ConstraintPropagation`，另两份没有）。

**教训：看到「已消除重复」的声明要重新验算，它可能只是意图没落地。**

### ⚠️ 改名过程中被 pre-commit 抓出的 3 个错

1. **撞上已有同名** —— 初版把 `reason` 的 `ReasoningMethod` 改成 `ReasoningApproach`，
   改完统计发现该名已被 `l0_substrate/nt_core_hex.rs:1017` 与
   `neotrix-types/core/nt_core_hex.rs:447` 占用（八卦工作流语义，恰好与推理策略无关）。
   → 立即改用 `ReasoningTaxonomy`（**改名后必须复统计**，否则用一个同名换一个同名）。
2. **漏了 re-export 路径的消费者** —— 首轮只搜定义所在子树，漏掉经
   `nt_mind/mod.rs:153 pub use reasoning_types::{...}` 暴露的 4 个消费者
   （`meta_panel/{types,engine}.rs`、`nt_mind/mod.rs`、`knowledge/memory.rs`），
   被 E0432 抓出。→ **找消费者必须顺 re-export 追，不能只看定义处。**
3. **整文件替换误伤全限定路径** —— `nt_reason_entry.rs` 里有 6 处用的是
   `crate::l5_cognition::reasoning_core::ReasoningTrace`（**crate 经 re-export 的**），
   被无差别 sed 一并改错（E0422）。→ **同一文件里同名符号可能指向不同作用域，
   全限定路径必须逐条甄别。**

### 方法论：先做三类测量，再动 sed

第二轮（`ReasoningStep`）先量后改，只动了 3 个文件就完成：
① `use`/`pub use` 导入 → 0 处 ② 全限定路径 → 0 处 ③ 文件内自用 → 25 处。
既无跨作用域消费者也无同文件多义，**不必像第一轮那样大范围追消费者**。
上轮三个错，全部源于没先做这三类测量。

## 7. crates/ 死依赖剔除（commit `00bc3223`）

`neotrix-sysctl`(150 行) 声明 5 个消费者，**只有 `neotrix-core` 真用**
（`nt_core_memory_budget.rs:43,47`）。其余 4 个（consciousness / gateway /
multi-agent / reasoning）源码内零引用，纯死依赖，已剔除。

**该 crate 本身绝对不可内联** —— 它是 `#![forbid(unsafe_code)]` 的 **FFI 专用逃生舱**
（crate 级 `allow(unsafe_code, reason=...)` + `libc::sysctl` + 裸指针转换）。
内联进 `neotrix-core` 就得给它加 `allow(unsafe_code)`，直接破坏项目最硬规则。

## 8. crates/ 整目录归档不可行（三条硬证据）

| 事实 | 实测 |
|---|---|
| `neotrix-core` 依赖其中 **8/9** 个 crate | 71,438 行。萃取 = 内联进已 797K 行的主代码 |
| `nt-core-capability-tree` 是 **CI 阻塞门** | `ci.yml:226` `neotrix-capability audit-maturity --strict`，带 318 节点/43 边落盘治理数据 |
| `neotrix-audit` 是 **CI 门** | `nt-audit.yml:26,30` 跑其 bin（0 库消费者但有 CI 门 = dev-tool crate，位置正确）|

另核实两处**不是**重复，是互补：
- `crates/neotrix-multi-agent`(2,571) vs 主代码 `l5_cognition/nt_core/multi_agent/`(4,655)
  —— 前者独有 35 个公开类型（hive/crew/blackboard/god_agent），后者独有 39 个
  （graph_orch/delegation/aggregation/load_balancer），仅 2 个同名；且前者有 3 处活调用
- `neotrix-reasoning` 有多处活调用（bank/kron、task_dispatcher/kernel_types、l5 re-export）

## 9. 遗留 / 待办

| 项 | 状态 |
|---|---|
| **`nt_core_capability` 14 个死引擎 4,940 行** | ✅ **已删**（`2bbed32c`）：14 引擎 + 2 死测试，5,708 → 155 行；其 91 个测试全是死代码互测；顺带合并逐字重复的 `inline_tests`/`inline_tests_2` |
| `Cargo.lock` 未入库 | ✅ **已修**（`2bbed32c`）：`.gitignore` 定点解禁（`*.lock` 仍生效于 node 侧锁文件） |
| **11 个同名类型** | ✅ **已消歧**：`25118be7` + 待提交那一笔。`ReasoningTrace` 4→1 / `ReasoningMethod` 3→1 / `ReasoningStep` 5→1 / `TraceSource` 2→1 |
| 4 条 `neotrix-sysctl` 死依赖 | ✅ **已剔**（`00bc3223`） |
| `layer-map.json` 的 `_rule` 写「CapabilityRegistry x4」 | ✅ **已更正并多次重测**（`99ec80b6`/`df80ccca`/本次）：消歧后 `CapabilityRegistry` **1 处**（唯一保留 L0 版）、`CapabilityTreeRegistry` 1、`CapabilityCatalog` 1；`LoadBalancer` **0 处**（两处已改名/删除）；`VersionManager` 1；`SearchResult` 7 处 |
| ~~剩余重名副本~~ | ✅ **已消歧**（`2afcbcc2`）：`CapabilityRegistry` 3 套 → L0 版保留原名（活，16 处 trait 实现），另两套改名 `CapabilityCatalog`（L5，仅自审）与 `CapabilityTreeRegistry`（crate，318 节点 CI 门）。`LoadBalancer` 2 处 → `MoELoadBalancer`（GWT，**已删** `f52238d2`）+ `AgentLoadBalancer`（multi-agent）|
| `nt_core/capability/registry.rs` | 该文件内 `CapabilityRegistry` 已改名 `CapabilityCatalog`（`registry.rs:473`），**不可删整文件** —— 同文件另 14 个类型中 11 个被 6 个兄弟文件生产引用。详见 `docs/architecture/OWNERSHIP.md` 的「B-2 撤销记录」 |
| `SearchResult` | ✅ **已定性，无需消歧**（`a8554a47` 后 7 处）。历史文档写 4 处低估；9 处中有 2 处是零消费者 hybrid 副本，删除后余 7 处**字段集两两不交**（doc_id/chunk_text、title/url、title/score/snippet、skill/score、id+similarity+metadata、id+score、id+title），非重复 |
| ~~hybrid 副本去重~~ | ✅ **已删**（`a8554a47`，−799 行）：`l5_cognition/nt_core_hybrid_search.rs`(401) 与 `crates/neotrix-gateway/src/hybrid_search.rs`(398) 零消费者，且被 `nt_memory_kb/nt_memory_search/`(2,423 行，全活) 完全覆盖。顺带 `HybridRetriever` 3→1、`NodeMeta` 2→0、`Bm25Document` 4→2 |
| 治理记录自身过期 | `_rule` 在两小时内过期两次（102→101、CapabilityRegistry 4→3→1）。**改完同名类型必须同步重写治理记录**，否则即制造下一个 R-SCAN-3 陷阱 |
| `target/` | 81G 构建产物（他窗持续跑测试，体积仍在涨），未清 —— 需 cargo 全空闲才能 `cargo clean` |
| `AGENTS.md` 门记录 | ✅ **已入库**（`3a0bf1c6`）：熔炼至 102 行，并更正两处过期数字（分层门 102→101、命名门补明 clean-HEAD 基线 1,646 vs 脏树 1,630） |

## 10. 门状态（本会话实测）

```
check-layer-deps.sh --strict   PASS: 0 new violation(s); 102 known/recorded
check-naming.sh                PASS (advisory)
nt_lock_audit.py               0 处
check-forbid-coverage.sh       OK      ← 曾因他窗删 src-tauri 而恒红，已修
cargo metadata --no-deps       exit 0，12 成员清单与 path 依赖全解析
pre-commit cargo check --tests ✅ Build gate passed（多轮；本轮门共抓出 4 个真实错误：
   2 个他窗的 model_pool.rs unused imports，1 个我漏 re-export 消费者，1 个我误伤全限定路径）
```

## 11. 本会话沉淀的方法论（可直接复用）

1. **找消费者必须三方查**：`use`/`pub use` 导入 + **全限定路径** + **顺 re-export 追**。只搜定义所在子树会漏。
2. **改名前必做**：① 语义定性（比字段，不比名字）② 候选名冲突预检 ③ 改完复统计。三步缺一就会「用一个同名换一个同名」。
3. **同名 ≠ 重复**：先查字段集是否相交、交集多大。4 处 `ReasoningTrace` 里 2 处字段
   语义冲突（`usize` vs `Vec`），根本不构成超集，只能改名不能合并。
4. **「已消除重复」的声明要重新验算** —— `reasoning_core.rs:3` 声称消除了 4 处重复，
   实际从未被采纳，且根本不可统一。
5. **共享 index 事故**：另一窗口把 64 个删除暂存进了共享 index，我 `git add` 时才发现。
   提交前必须 `git diff --cached --name-status` 核对归属；清理用 `git reset`
   （mixed，不动工作树）而非 `git checkout --`。
6. **Cargo 反查看不见 CI**：`neotrix-audit` 与 `nt-core-capability-tree` 的 Cargo
   反查显示零消费者，实为 **CI 门在用**（`nt-audit.yml:30`、`ci.yml:224-226`）。须一并 grep
   `.github/workflows/`。
