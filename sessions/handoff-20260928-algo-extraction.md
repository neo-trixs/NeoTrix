# handoff — 算法萃取 + 目录归档（2026-09-28）

> 本会话在共享工作区与另一窗口并行作业。**凡标「他窗」的都不是本会话的改动。**

## 1. 一句话

从 `crates/` 的 2 个孤立 crate 里萃取出 **8 个通用算法**进主代码 L0–L6，
把 4 个 crate + 2 个顶层目录归档到 `~/Downloads/Neo/neotrix-archive/`，
裁决 2 笔架构债。顶层目录 **17 → 9**（末轮清理后），`crates/` **13 → 9**。

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
| **`RagEngine` 同名 2 份** | ✅ **已消歧**：`l2/rag_pipeline/rag_engine.rs` → `DocumentRagEngine`（文档级 RAG），`l1/bank/rag_engine.rs` 保留 `RagEngine`（记忆层级管理）。**评估过「收敛为一个」，结论是不可合并** —— 22 个公开符号仅 3 名相同，`find_duplicates`/`should_promote`/`tier_counts`/`MemoryConsolidation` 仅 bank 侧有；两侧均重负载（A 35 处 / B 16 处调用）。若日后要功能增强（让 bank 侧复用 `nt_memory_search/nt_scoring.rs` 的 `SmartVectorScorer`/`fuse_signals`），那是产品决策而非收敛 |
| `bank::RagEngine` 打分是否升级为复用 `nt_scoring` | ✅ **已裁决：不换**（2026-09-28 评估两次）。理由：① **数据源不同** —— `nt_scoring` 的 4 维（semantic/temporal/confidence/relational）依赖 KB 节点的 `updated_at`/`node_confidence`/图边，而 bank 侧对象是 `ReasoningMemory`，**这三个信号无源**，硬接只能传 0 或造假值，比现有 3 信号线性加权更差；② 现状**无缺陷**，升级属产品调优非修 bug；③ 改打分会影响 16 处调用的检索排序并波及 `self_improvement` 闭环，风险不对称。若日后确需升级，**正确顺序是先给 `ReasoningMemory` 补数据源，再调 `fuse_signals`**。两侧文件顶部已立「能力边界」文档防止第三次来问 |
| `layer-map.json` 的 `trees` 数字过期 | ✅ **已更正**（2026-09-28 20:0x）。按 **HEAD 口径**（逐文件 `git show HEAD:<path>` 计数，非工作树）重测 4 条：nt_crystal_core 51/19371→**52/21247**、nt_file_ability 44/14016→**44/13993**、ffi 12/2907→**12/2919**、nt_jev 14/4312 复核一致。**只改数字，未动 role/note 裁决**。差异成因是首版数据量的 HEAD 早于其后的晶体核心等提交，**与本会话改动无关**（该目录 0 未提交改动，本会话 13 笔提交零命中）。教训：治理文件里的 files/lines 同样是**会过期的数据**，须与 `_rule` 同等对待 |
| `l6_meta/memory` 的层归属（C-3） | ✅ **已裁决**（2026-09-28 20:1x，从 `unresolved` 移入 `trees`）。该目录 8 文件**混装两种职责**：① **4 个超越层组件留 L6（正确）** —— `consonance_orchestrator`/`evolution_harness`/`meta_observer`/`transcendent_loop` 自述源自 `nt_mind::transcendent::` 但 `l5/nt_mind` 下无同名文件（**非重复**，只是历史搬到了 L6），且 L5 意识循环**主动消费 12 处**（`nt_transcendent.rs` 5 + `nt_audit.rs` 1 + `self_test_integration.rs` 2 + 其他），依赖方向 **L6→L5**，语义上元观察/共鸣/进化闭环本属元层 ⇒ 不搬；② **3 个记忆存储模块层归属错位但本轮不搬** —— `nt_memory_experience_tree.rs`(458,3测试) / `nt_memory_knowledge_pipeline.rs`(228,0测试) / `nt_memory_wikiskill.rs`(298,0测试) 库外引用**全为 0**，语义属 L4，但搬动牵涉各自数据落盘路径与序列化格式 ⇒ 单列 `dead_modules` 字段留证，待独立任务。同时立了**层归属裁决的证据门槛**（须有①构造点/消费点证据 ②物理位置与逻辑职责独立判断 ③零消费模块单列）并写进 `_rule`；代码内 `l6_meta/memory/mod.rs` 顶部也立了裁决文档（含 glob 导出的迁移陷阱提示） |
| 治理记录自身过期 | `_rule` 在两小时内过期两次（102→101、CapabilityRegistry 4→3→1）。**改完同名类型必须同步重写治理记录**，否则即制造下一个 R-SCAN-3 陷阱 |
| ~~`target/`~~ | ✅ **已清，释放 80 GB**。清理前实测本仓 target **0 句柄占用**（并行窗口当时在 `/Users/neo/Downloads/Neo/neobot` 跑测试，用的是另一个仓的 target）。清理后四门仍全绿 —— 证明门不依赖构建产物 |
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
6. **删文件必须确认 `D` 行随提交落地**（本会话踩中，`a8554a47` → `cca40a17` 补提交）。
   用 `git rm` + 逐个 `git add <mod.rs>` 时，**文件本体的删除会漏**，提交里只剩 `M`。
   后果：HEAD 仍跟踪该文件 → checkout/clone 复活成永不被编译的死文件，而 mod 声明已删。
   校验法：`git diff --cached --name-status` 应见 `D`；只见 `M` 即漏。正确做法是
   `git add -A <路径>` 或 `git commit -a`。
7. **Cargo 反查看不见 CI**：`neotrix-audit` 与 `nt-core-capability-tree` 的 Cargo
   反查显示零消费者，实为 **CI 门在用**（`nt-audit.yml:30`、`ci.yml:224-226`）。须一并 grep
   `.github/workflows/`。

## 8. 收工自查（2026-09-28 义务生效，本会话自补）

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` → **exit 4**（存在带未提交改动的 worktree）：

```
[worktree-gate] worktree=1 个 | 合计 50M | target 占 0M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 1 个
[worktree-gate] ⛔ 1 个 worktree 的未提交改动**不在任何提交里**
```

| worktree | 用途 | 去向 |
|---|---|---|
| `.worktrees/ratchet` | `fix/bitemporal-and-layer-ratchet`（5 处改动：4 个 .rs + `scripts/layer-deps-baseline.txt`） | **明确移交，不收** —— 近 3h 仍有 .rs 改动 ⇒ 判定为他窗活跃。本会话曾误删过一次并已用 patch 完整恢复（5 处全中），此后按 R-DISK-7 判据同向原则不再自动清理 |

本会话新建的 worktree：**0 个**。清理掉的 19 个均为历史遗留（详见各提交信息）。

### 8.2 未提交改动的去向

主仓 `git status` 现有 3 处未提交，**全部为他窗 WIP，非本会话产生**：

| 文件 | 归属 | 去向 |
|---|---|---|
| `neotrix-core/src/l0_substrate/nt_core_kb_primitives.rs` | 他窗 | ☐ 非本会话产物 → 移交 |
| `neotrix-core/src/l5_cognition/nt_mind/evolution/evolution_daemon.rs` | 他窗 | ☐ 非本会话产物 → 移交 |
| `scripts/layer-deps-baseline.txt` | 门脚本自动更新 | ☐ 非本会话产物 → 移交 |

本会话已移除的 15 个 worktree，其 850 处未提交改动**全部兜底**在
`/Users/neo/Downloads/Neo/neotrix-archive/dirty-salvage-20260928/`
（31 个文件 / 5.6M，15/15 通过 `git apply --check --reverse` 校验，附 README 回放说明）。
其中 `ratchet` 的 patch 已在实战中验证可回放（`apply --3way` 5 处全中）。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code：**4**（ratchet 脏，见上，已判定为他窗活跃）
- `cargo check --tests -p neotrix`：✅ 每笔提交的 pre-commit gate 均通过
- `check-layer-deps.sh --strict`：101 known / **0 new** ✅
- `check-truth-surface.sh`：EMPTY:0 / UNDECLARED:0 ✅
- `nt_lock_audit.py`：0 处 ✅
- 门红归属：无本会话引入的红

## 9. 补充：scripts/ + skills/ 架构吸收（2026-09-28 晚）

### 9.1 根因诊断
`scripts/` 的发现机制**散落在散文里**（AGENTS.md 10 处 / RUST-STANDARDS.md 3 处 /
Makefile 21 处），无单一权威；`skills/` 有 `index.json` 但它答的是
「有哪些技能」而非「我这任务该调什么」。⇒ 都不是"融入架构"的问题，
是**缺机器可查的意图→工具索引**。

### 9.2 交付
| 提交 | 内容 |
|---|---|
| `ef01b200` | skill_loader 路径式 key 解析（**13/58 → 58/58**）|
| `d183f04e` | check-skill-gate 改**双侧**校验（此前只看 index，看不见 67 个 SKILL.md）|
| `20b24595` | `SkillInvocationPolicy` 调用策略正交两维（官方规范吸收）|
| `73ad06e4` | run.rs 知识资产路径 bug（**166KB 资产从未被导入过一次**）+ 删撒谎注释 |
| `ca6aed8e` | **pre-push 收工门是死代码**（中途 exit 0），R-DISK-8 |
| `50ac02bd`+`176b074d` | task-index + `nt_find.py` + pre-commit 门 |
| `cc9032cd` | **Context Manifest**（`nt_manifest.py`）+ pre-commit 门 |
| `3f85b03a` | 索引 19→30 条，40 个脚本**孤儿归零** |
| `93a0898c` | dev-tools 6 技能补 frontmatter（按需，非批量）|
| `75fc7384` | 8 项目吸收正典文档 |

### 9.3 三次「验证后否决原计划」
1. **外部调研的核心结论被证伪** —— 「skills 文件名不是 SKILL.md 故不可发现」
   实测证伪（67 个 SKILL.md 齐备、59 条路径零缺失），真问题是 frontmatter
2. **`--serves` 反向声明被否决** —— 反向映射已能从 task-index 单向推导，
   再让每个脚本手写一份 = 制造第二份真源（本会话反复治的病）。改做孤儿检测
3. **批量生成 frontmatter 被否决** —— 失败模式是填假值（`description: TODO`），
   自己先造假值等于拆自己刚立的门。只补真引用执行体的 6 个

### 9.4 本会话的失败模式（已全部固化进门）
| 失败 | 固化成的机制 |
|---|---|
| 误删 ratchet（判据自相矛盾仍下结论）| R-DISK-7 判据同向原则 + `nt_worktree_gate.sh` 三判据 |
| pre-push 死代码（测逻辑 ≠ 测可达性）| R-DISK-8 + `nt_manifest.py --audit`（记了 file:line ≠ 还指在那里）|
| layer-map 数字过期 | 按 HEAD 口径重测 + 3 轮时效核查 |
| 采信外部 agent 结论未实测 | `nt_manifest.py add` 要求带 file:line 证据，`--audit` 校验行号有效 |
| grep 判据写死 `^exit 0` 匹配不到缩进 | R-DISK-6（追加前先 grep -c 计数）|

### 9.5 收工自查（§8 补充）
- `nt_worktree_gate.sh check` exit 4 —— `.worktrees/ratchet` 5 处脏，**他窗活跃**（近 3h 有改动），明确移交不收
- 本会话新建 worktree：**0 个**
- 主仓未提交 10 处：全为他窗 WIP
- 门：layer-deps 101known/0new · truth-surface 0/0/0 · find --audit 30/30 ·
  manifest --audit 5/5 file:line · lock-audit 0 · 全绿

## 10. 两个已知缺陷的根本修复（2026-09-28 夜）

### 10.1 缺陷一：`check-api-surface.sh` 死引用致「假 0」→ `f6c4929e`
`rg` 对不存在路径 exit 2 并写 stderr，但 `2>/dev/null` 吞掉它，`awk` 仍输出
`0` ⇒ 脚本打印 **`Tauri commands: 0`** —— 一个**看起来正常的假数字**。
读它的人会得出「本仓没有 Tauri 命令面」，而真因是「我压根没扫那个面」。

核心区分写进注释：
    **路径不存在 ≠ 该面为空**（前者是脚本缺陷，后者才是被扫描对象的事实）

修法三层：① 删死引用 ② 面缺失时打 `?` + exit 3 + stderr 说明 ③ 讲清
「tauri=0 是事实（桌面端已随 5c02e738 归档），非扫描失败」。

**实测**：把 `neotrix-core/src` 临时移走 ⇒ 输出 `?` + `SURFACE-MISSING` + exit 3；
移回 ⇒ exit 0。中间踩了两个坑并留档：
- 诊断混进 `$(...)` 返回值 ⇒ stderr 泄漏 + `[: : integer expression expected`
- `$(...)` 失败给空串，对空串 `[ -eq 2 ]` 报错 ⇒ 改用**退出码**判定

### 10.2 通用预防：`nt_scan_surface.py`（`f6c4929e`）
扫所有扫描器的扫描面是否存在。同型 4 处（`check-truth-surface.sh:43` 的
`apps`/`src-tauri/src`、`nt_mapgen.py:48-50` 三条 `src-tauri/*`），
前人已用不同方式修过（删引用+注释 / `[ -d ] || continue`）但**都不报「少扫了」**。

**工具自身调了两轮才可信**（守门工具噪声大 = 没守）：
① 首版 20 个 MISSING 里大半是假警报（注释里的 `foo/bar.rs`、文档里的
   `nested/test`）⇒ 加注释/heredoc 剥离 + 已知根过滤
② 剩 `tests/mod.rs` 误报（是 echo 里的提示文字）⇒ 加 PROSE_PATHS 豁免
③ 现存 11 面 / 缺失 5 面，**5 个全是真死引用**（4 个 KNOWN-GONE + 1 个未知）

### 10.3 缺陷二：`nt_jev_live_eval.py` 断链 import → `22a9c600`
`from nt_verify_sim import keywords`，该模块已随 `2bbed32c` 删除 ⇒
**脚本一 import 就 ImportError，完全跑不起来**（`__pycache__` 的 .pyc 掩盖了这点）。

修法遵守该文件自己的「**不重写**」纪律：新增 Rust 导出点
`neotrix-core/src/bin/nt_keywords.rs` 调 `CrystalConsciousness::keywords`
（`consciousness.rs:546`），而非在 Python 里重写分词。
**为何必须走 Rust**：① 那是权威口径（带 `strip_src_tag` + 停用词 + 单字符
过滤，且被觉醒循环复用），重写=第二份真源 ② crate 内另有 **3 个同名
`keywords` 且实现各不相同** ⇒「哪个权威」本身就是歧义源。

连带：`keywords` 由 `pub(crate)` 提为 `pub`（`src/bin/*.rs` 是独立 binary
target，`pub(crate)` 对它不可见，报 E0624）。doc 写明为何公开，且若降级则
`nt_keywords` 编译失败 —— 那正是想要的强制点。

**实测（干净检出）**：`cargo check --tests` 全绿；`Rust 的所有权很安全` →
`["Rust","的所有权很安全"]`；`a 的 了 and the Rust` → `["Rust"]`（停用词全过滤）。

### 10.4 R-SCAN-4：请示后仍须复核现场（`a972e03f`）
门红在他窗未提交文件，我取证确认归属后请示，用户选定「顺手删掉那个
unused import」。**动手前复核那一行，内容已变、mtime 是 42 秒前** ——
那个窗口还活着且已自己修好。若照决策下手，就往正在被编辑的文件里写入。

**规约**：请示与执行之间存在时间窗，那个窗口里世界可能已变。

### 10.5 当前门红（他窗 WIP，非本会话引入）
`check-truth-surface --strict` 报 `UNCOMMITTED_DEP:2`：
`neotrix-core/src/nt_mcp_stdio_session.rs` 与 `nt_qwen_mm_manifests.rs`
—— 两个都是他窗 `??` 未提交的新文件，而 `agent.rs`/`lib.rs` 正在引用它们。
该门的设计正是捕捉这个（"已提交代码引用了未入库文件"），属**如实报告**。

## 11. 收尾：死引用清零 + 经验吸收（2026-09-29 凌晨）

### 11.1 死引用 6 条清零（`0daee35d`）
`nt_mapgen.py` ROOT_AREAS：`src-tauri/{src,frontend,frontend/src}`（5c02e738
归档）、`games`/`fuzz`（2bbed32c 归档，现居 neotrix-archive/）、**`ntos/src`
（该目录从来不存在** —— 代码里 "ntos" 只是普通词）。
`check-truth-surface.sh:43`：`apps`（本会话移除）+ `src-tauri/src`。

**取证澄清（勿夸大）**：`nt_mapgen.py` 的 `ROOT_AREAS` 只是**分类标签表**，
真正遍历是 `os.walk(root)` + `.` 跳过 SKIP_DIRS，且已有 `"other"` fallback
⇒ 死引用影响的是**area 标注**，不是"生成不了图"。

### 11.2 工具自身的两处盲区（本轮最重要的发现）
用**注入法**实测发现——「用它防死引用」这件事本身有盲区：
① `KNOWN_ROOTS` 过滤器把**新死引用一起滤掉**（注入 `definitely-gone-dir` 不报）
② `PATH_RE` **要求路径含 `/`** ⇒ 单段扫描根（games/fuzz/ntos 都是单段！）连
候选都进不来 —— 而本仓死引用**恰好全是单段**。

修法：三级分档（现存 / 硬缺失 / 待确认，只有硬缺失进退出码）+ 二元组首段扫描。
**调三轮才既不漏又不吵**：裸扫二元组 24 噪声 → 限「≥4 字符无扩展名」9 噪声
→ 9 条全是固定项逐条登记豁免 ⇒ **基线 0 噪声，注入两种形态各抓到 1 条**。

取舍说明：「待确认」不进退出码是刻意的 —— 宁可多报一条 `?`，
也不让门因噪声被人 `--no-verify` 绕过。**门一旦被习惯性绕过就等于没有门。**

### 11.3 经验吸收（`71c55dac`）
`docs/architecture/LESSONS-20260928-verification-must-be-executable.md`：
9 次「结论与实测矛盾」的共同根因 —— **没有一次是因为想得不认真**，
全部因为验证只存在于叙述中。9 条按「症状→根因→机械化」排列。
元规则：**「把『我验证了』变成可检查的记录，而不是可信赖的声明。」**

### 11.4 收工自查（§8 更新）
- `nt_worktree_gate.sh check`：**worktree 1 个**（ratchet），脏文件 **0**
  （此前 5 处他窗已自行提交），近 3h 有活动 ⇒ 他窗活跃，**不碰**
- 本会话新建 worktree：**0 个**。临时验证台 `/tmp/nt-verify-` `/tmp/nt-v2-`
  均已 `worktree remove` + `prune`
- 主仓未提交 21 项：**全部为他窗 WIP**（multimodal 方向：QWEN-MM manifests
  / nt_mcp_stdio_session / skills/nt_multimodal / check-capability-manifests.sh
  / 9 个文档）。**本会话零产物留在工作区**
- 门：layer-deps 101known/0new · truth-surface EMPTY/UNDECLARED/TRACKED 全 0
  （UNCOMMITTED_DEP=2 是他窗新文件未入库，门如实报告）· find 31/31 ·
  manifest 5/5 · scan-surface 21/21 · lock-audit 0 · 全绿

## 12. 收尾二：占位 skill 治理 + 死链门（2026-09-29）

### 12.1 三个占位 skill：标注而非删除 `3e9e8783`
`code-expert`/`law-expert`/`mcp-gateway` —— 只有 SKILL.md 一个文件、零实现、
**仓内消费者 0 处**、84 天未动。

**取证澄清一处误判**：初查 `mcp-gateway` 有"1 处引用"，实为
`models/training/repo_meta.jsonl` 里某 GitHub repo 的 `topics` 字段恰好叫
`mcp-gateway` —— 不是消费。**index.json 的引用是注册关系，也不等于使用。**

**不删的四个理由**（删的证据齐了，但四条反证更硬）：
① 删 skill 要同步改 index.json + skill_loader + check-skill-gate（多点风险）
② `code-expert`/`law-expert` 是**真实能力领域**，只是内容待补
   ⇒「待建设」而非「错误资产」
③ 同类先例：`trending/` 5 个被 Rust 取代的 skill 本会话也没删
④ 删除不可逆（git 之外无副本）

⇒ 加 `disable-model-invocation: true`（模型不自动加载）+ 正文
「状态：PLACEHOLDER」写明不承担职责与真实承接者（`mcp-gateway` →
`crates/neotrix-gateway`）。**依据「导出 ≠ 调用」：0 消费者是删除的必要
条件，不是充分条件。**

### 12.2 `skills/SKILL.md`：死链存在一年无人发现
- 移出 `src-tauri/`（已随 5c02e738 删除）
- 解决自相矛盾：`crates/` 曾同时列在「skill 目录」与「非 skill 核心目录」
- 补 11 个漏列分类（external-absorption / research-absorption /
  self-iteration-agent / self-health / productivity / root + 3 占位 +
  trending 移入废弃段并标注已被 L1/L3/L5 Rust 取代）

### 12.3 根本预防：门加死链检查
`check-skill-gate.sh` 新增文档侧相对链接校验。判据与 `nt_scan_surface` 同源：
**路径不存在 ≠ 该面为空**（分类表里的死链是「导航腐烂」入口）。

**首版有覆盖面盲区，靠注入法自测抓到**：初版只在 `index.json` 覆盖的条目里查，
而 `skills/SKILL.md`（顶层导航，含全部链接表）**不在索引里** ⇒ 死链照样过。
改为扫整棵树。⇒ 这是本会话**第 10 次**翻车，同型于 nt_scan_surface 盲区 #1
（把「已注册的对象」当成了「全部对象」）。已补进 lessons 档第 10 条。

### 12.4 收工自查（§8 最终）
- worktree **2 个**（他窗新建 1 个 + ratchet），**本会话新建 0 个**
- 主仓未提交 22 项，其中 6 项是他窗 multimodal 方向（QWEN-MM manifests /
  nt_mcp_stdio_session / skills/nt_multimodal / check-capability-manifests.sh）
  ⇒ **本会话零产物留在工作区**
- 8 个门：layer-deps 101known/0new · truth-surface EMPTY/UNDECLARED/TRACKED
  全 0（UNCOMMITTED_DEP=2 是他窗新文件未入库，门如实报告）·
  skill-gate 0 broken doc links · find 32/32 · manifest 5/5 ·
  scan-surface 21/21 · lock-audit 0

## 13. 「哪些单文件该融合」—— 结论是**不该现在融合**，理由有实证（`20016818`）

### 13.1 三种朴素判据，三种答案，全部不成立
| 判据 | 指向 | 为什么错 |
|---|---|---|
| 文件行数排序 | `nt_channel_telegram.rs`(2353行) 等 15 个巨型文件 | 它内部结构完整（`TelegramChannel` 一个类型 + 9 个私有辅助函数），**不是"多件事塞一起"**。大 ≠ 该融合 |
| 同名动词聚类 | 34 个 `*Score`、17 个 `*parse*` | `MergeStrategy` 在 xlsx、`RenderMode` 在游戏引擎 —— **领域不同的合理同名** |
| 同类型名跨文件 | 871 个，看着全是问题 | **绝大多数字段集不同** |

⇒ **「同名」不等于「重复」**。硬判据是 **同名 + 字段集完全相同**。

### 13.2 实测规模
```
同名类型（分布多文件）        589 个
  ├─ 同名 + 字段集完全相同    122 组  ⬅ 融合候选
  └─ 合理同名（字段集不同）    467 个  ⬅ 不该动（改了会毁掉分层）
```

抽样：`AwarenessReport` 3 处（l0/l1/l5）**字段集完全相同** ⇒ 真重复；
同名的第 4 处（`l6_meta/healing/nt_mind_consciousness_monitor.rs`）有
**11 字段**而那 3 处各 6 字段 ⇒ 不同东西，工具正确排除。

### 13.3 为什么交付工具而非直接融合
三条实证理由：
① 字段集相同 ≠ 语义相同 —— 本例 3 处 derive 确实一致，但**分属 L0/L1/L5
   三层，各自 trait 实现与调用方都不同**
② 跨层移动类型会触发 `check-layer-deps.sh` 的层归属裁决
③ 本仓「导出 ≠ 调用」已误删 3~5 次；本会话刚因误删 `ratchet` 靠 patch 恢复

把 122 组候选一次性自动改，会同时踩这三条。
⇒ `scripts/ops/nt_dup_types.py` **只提供判据与证据**，改不改由人裁决。

### 13.4 工具可信度的一次交叉验证
与独立 `grep -rl` 核对时，我一度以为工具漏报（grep 4 处 / 工具 3 处）。
逐字段复核确认**工具对、grep 太粗**。
> 通则：**更精确的判据会把更粗判据的"错误"暴露成自己的"缺陷"**。
> 交叉验证时先问「谁的判据更精确」，而不是「谁报少了」。

已接 task-index（33 条），触发词「融合 / 精简 / 重复 / 去重」。
