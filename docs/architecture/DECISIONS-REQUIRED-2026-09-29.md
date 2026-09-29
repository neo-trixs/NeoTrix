# 裁决清单 — 2026-09-29（9 项，含本轮实测证据）

> **本文件只列「我不能替你决定」的事**，每项给：问题 / 实测证据 / 选项 / 建议 /
> **不决定的后果**。
> 配套：`FINAL-ROADMAP-2026-09-29.md`（路线）· `FEATURE-MAP-TASKS-2026-09-29.md`（特性）。
>
> **重要**：其中 **D-1 / D-2 本轮已做完实测**，可直接决策，不再需要调查。

---

## D-3 ⛔ 空账本？—— ✅ **已实证澄清：不是缺陷**（2026-09-29 更新）

### 我此前的判断（**已证伪**）

「`test-failures-baseline.txt` 0 字节 ⇒ `--strict` 下零容忍 ⇒ P0 缺陷」。

### 实测结论

```
隔离工作树（干净 HEAD）跑 check-test-baseline.sh：
  test result: ok. 12187 passed; 0 failed; 42 ignored
  PASS: the whole suite is green.
```

再看脚本逻辑（`check-test-baseline.sh:95-102`）：

```bash
if [ "$RC" -eq 0 ]; then
  echo "PASS: the whole suite is green."
  ...
  exit 0          # ← 全绿时**直接 exit 0**，压根不看账本
fi
```

⇒ **空账本 + 全绿是正确的空基线，不是「零容忍陷阱」。**
`handoff-20260928-consolidated §1` 记的那 57 条失败**早已被修好**。

### 非空门证明（该门**是有效的**，不是空门）

注入一条 `assert_eq!(1 + 1, 3)` 后：

```
failing now: 2   in ledger: 0
NEW failing tests (not in the ledger): 2
  l6_meta::...::deliberately_failing_probe
--strict exit = 1        ← 正确变红并指名
```

⇒ **A3 关闭**。它不是待做项，是**已完成项**。

### 教训

⛔ 我把「文件是 0 字节」当成了缺陷信号，**没有先问「它本该非空吗」**。
`check-truth-surface.sh` 的 baseline 有 265 条（真存量），
`test-failures-baseline.txt` 有 0 条（**真已清零**）——
**同样是 0 字节，含义完全相反**。

**⇒ 「数字异常」不等于「缺陷」。必须问：这个数字本该是什么。**
（同源于 09-28 那条：「先证伪提交的前提」—— 我对**自己的结论**也该这么做。）

---

## 决策速览（更新）

| # | 事项 | 谁能定 | 体量 | 建议 | 阻塞谁 |
|---|---|---|---|---|---|
| **D-1** | 265 条未编译 `.rs` 逐条处置 | **产品** | 68,603 行 | 分三批，见下 | 批次 D 全部 |
| **D-2** | `hybrid_retrieval` 等 4 个子树接线 | **技术**（我已实测） | 2,943 行 | ⭐ **可接** | D4 多因子打分 |
| **D-3** | `nt_meta/eval_engine` 接不接 | — | **已裁决** | ⛔ 不接（`llm_judge.rs:86` 恒满分）· 改为补第四层 `nt_evolution_eval` | — |
| **D-4** | `docs/` 要不要恢复 vitepress 站 | **产品** | 需重写 | 不恢复 | CI 路径（已修） |
| **D-5** | 桌面回路落点 | **架构** | — | 落 neobot | 批次 E 全部 |
| **D-6** | `qybaihe/mu` 署名链 | **法务/作者** | — | 澄清前不入正典 | 无（仅索引） |
| **D-7** | `Aegis` 22 个 skill 要不要吸收 | **产品** | — | ⛔ 只取方法论 | `skills/` 目录 |
| **D-8** | N-6/N-7 何时做 | **协调** | — | 等对方窗口 | 批次 C |
| **D-9** | `Provenance` 默认值 | **已澄清** | — | ⛔ 不动 | 无 |

---

## D-1 🔴 265 条未编译 `.rs` 的逐条处置

### 问题

`neotrix-core/` + `crates/` 下有 **265 个 `.rs` 文件（共 68,603 行）从未被任何 target 编译**。
它们不占编译时间、不会报错、也不会出现在任何测试覆盖里 —— 但**它们在 git 里，占着行数，
让人以为功能存在**。

### 实测证据（本轮）

```
baseline 条目:  265
总行数:      68,603
最��� 10 大文件:
  1,301  nt_mind/self_improvement/sleep_compute.rs
  1,077  nt_mind_background_loop/handlers_maintenance.rs
    819  nt_mind/cross_domain/entity_mapping.rs
    770  nt_act/tool_registry.rs
    747  nt_act/nt_act_scheduler.rs
    719  nt_act/nt_act_workspace_isolator.rs
    700  nt_mind/nt_mind/federation.rs
```

**分类实测**（按子树）：

| 类别 | 条数 | 特征 | 处置选项 |
|---|---|---|---|
| **(c) 声明被注释掉** | **6** | `nt_memory/mod.rs:51-54` 四棵子树被注释，注明「内部编译错误待修复」⇒ ⛔ **加回声明会让干净检出编不过**（实测见 D-2） | 修编译 / 删 |
| **(a) 明显已归档** | **3** | `nt_consciousness_core/archive/`(3)，目录名自带 `archive` | 直接删或归档 |
| **(b) 大子树** | **33** | `multi_agent/`(15) + `session_replay/`(12) + `otel_bridge/`(7) 等 | 逐个裁决 |
| **(d) 其余** | 223 | 单文件或小树，母目录 mod.rs 无声明 | 补 `mod` 或删 |

### 三个选项

| 选项 | 做法 | 代价 | 风险 |
|---|---|---|---|
| **A. 全删** | 265 条一次删净，baseline 归零 | 1–2 小时（机械） | 🔴 **高**。`DIR-REMEDY §2.5` 记「导出 ≠ 调用」**已错过 3 次**（`CapabilityRegistry` 4 份 / `SearchResult` 9 份 / 三个决策引擎）。第 4 次可能是「它们其实有用，只是没接线」 |
| **B. 全接线** | 每条补 `mod` 声明 | 未知（可能编不过） | 🔴 **高**。且**编不过就是净损失**（干净检出挂掉） |
| **C. 分类分批** ⭐ | (a) 删 → (b) 逐个查消费者 → (c) 见 D-2 → (d) 抽样补接线 | 3–5 天 | 🟢 低。**每批独立可回滚** |

### 建议

**选 C**，顺序：

1. **(a) 3 条直接删** —— 目录名自带 `archive`，零争议
2. **(c) 6 条交给 D-2** —— 本轮已实测，结论已出
3. **(b) 33 条**：逐个 `grep` 生产消费者。**有消费者的接，没消费者的问「这功能还想做吗」**
4. **(d) 223 条**：**先抽样 20 条**。若 20 条里 0 个有消费者 ⇒ 大概率整体可删；
   若有 ⇒ 说明这批是「设计好但没实现完」，那是产品决策不是清理决策

### ⛔ 不决定的后果

- baseline 棘轮**只挡新增**，不挡存量 ⇒ 265 条会一直躺在账上
- 每轮「未编译」类讨论都要重查这 265 条（本轮就查了 3 次）
- 新人读代码会以为这些功能存在 ⇒ **这是最贵的一种「说谎」**

---

## D-2 ⭐ 265 条里的 (c) 类 —— 本轮已实测，**结论已出**

### 问题

`neotrix-core/src/l4_emotion/nt_memory/mod.rs:50-54`：

```rust
// NOTE: 以下模块已声明但内部编译错误待修复，暂时注释
// pub mod add_only_writes;
// pub mod admission_control;
// pub mod decay_forgetting;
// pub mod hybrid_retrieval;
```

四棵子树被注释，理由是「内部编译错误」。

### 实测（本轮真的接上去编了）

```
动作：把 hybrid_retrieval 的注释去掉，跑 cargo check --lib -p neotrix
结果：hybrid_retrieval 相关错误 = 0
      编译出的 18 个错误全部在 crates/neotrix-neobot/src/nt_pdf_ground.rs
      （mtime 21 秒前 ⇒ 另一窗口在制品，与本项无关）
```

⇒ **注释里的「内部编译错误待修复」已经过期了。**

四棵子树的体量（**实测**）：

| 子树 | 行数 | 文件数 |
|---|---|---|
| `add_only_writes` | 789 | 4 |
| `admission_control` | 581 | **5** |
| `decay_forgetting` | 734 | **5** |
| `hybrid_retrieval` | **1,028** | 6 |
| **合计** | **3,132** | **20** |

### ⛔ 但 `hybrid_retrieval` 接上会引入一个**新的重复**

实测：`rrf_fuse` 在仓内**已有 5 份**

```
nt_core_bank/iteration.rs:140            ← 有生产消费者
nt_infra_unified_search.rs:343          ← 未编译（在 D-1 的 265 条里）
nt_memory_kb/bm25.rs:174                ← ⭐ 现役路径
nt_memory_kb/retrieval_fusion.rs:51     ← 零消费者
neotrix-types/.../nt_core_bank/iteration.rs:128
```

而 `hybrid_retrieval/fusion_engine.rs:14` 又定义 `const RRF_K: f64 = 60.0` ——
与 `bm25.rs:3` 的 `RRF_K=60.0` **完全同值但独立定义**。

### 选项

| 选项 | 做法 |
|---|---|
| **A** ⭐ | **只接 `hybrid_retrieval`**，但**不接线到生产**（先让它编译，验证 0 错），并在 `fusion_engine.rs` 顶部加注释指向 `bm25.rs` 的正典，标注「本文件是待裁决的重复实现」 |
| **B** | 接 4 棵全部（3,000 行），逐个查消费者 |
| **C** | 全删，`hybrid_retrieval` 与 `bm25.rs` 重复 |

### 建议

**选 A**，理由：

1. 本轮已实测 **0 编译错误** ⇒ 「待修复」这个前提不成立，应该先纠正
2. 但 `rrf_fuse` 已有 5 份 ⇒ **接上去会制造第 6 份**，必须同时标注
3. 这样 `check-truth-surface` 的 UNREACHABLE 从 212 降到 206，**且不引入任何行为变化**
   （因为没接生产消费者）

### ⚠️ 执行注意

`hybrid_retrieval` 被声明后**会进入编译**，增加编译时间。实测增量约 **15s**（15.75s 的一次
`Checking neotrix` 里有它）。若不接受，退路是选 C。

---

## ~~D-3~~ ✅ 已裁决（2026-09-29）→ 见 `DECISION-D3-EVOLUTION-EVAL.md`

**结论：⛔ 不接 `eval_engine`，改为补上缺失的第四层 `nt_evolution_eval`。**
理由：`llm_judge.rs:86` 是 `let normalized = 1.0;`（恒满分、从不调 LLM）。
以下为当时的原始分析，保留作取证。

---

## D-3 🔴 `nt_meta/eval_engine` 650 行接不接（已裁决，保留存档）

### 问题

`neotrix-core/src/l6_meta/nt_meta/eval_engine/` 是**全仓唯一**的
dataset / experiment / llm-judge 抽象，**650 行、4 个文件，从未被编译**
（`nt_meta/mod.rs` 无 `mod eval_engine`）。

### 实测

```
dataset_manager.rs      209 行
experiment_tracker.rs   244 行
llm_judge.rs            188 行
mod.rs                    9 行
                        ────
                        650 行
nt_meta/mod.rs 里 mod eval_engine：0 命中
```

### 为什么这很重要

**批次 B（测量面）的每一个节点都依赖它**：
N-11 门记录 env 指纹 · N-12 门可满足性元门 · 0.2 证伪门 · B5 影子→active

而四轮吸收的核心结论是：**「自进化机制不是护城河，能证明自进化是否有效才是」** ——
**验证能力就是这 650 行**。

### 选项

| 选项 | 做法 | 代价 |
|---|---|---|
| **A** ⭐ | **接上**（`nt_meta/mod.rs` 加一行 `pub mod eval_engine;`），先验证能编译 | 1 小时。但**它现在零消费者** ⇒ 接上只是「能编译」，不等于「在用」 |
| **B** | 接上 **且** 把 N-12 元门建在它上面 | 1 周。**这才是真正的价值** |
| **C** | 删 | 5 分钟。但**测量面就没有地基了** |

### 建议

**选 B**，但分两步：

1. **先 A**（1 小时）—— 让它编译，验证 0 错。**这一步纯粹是纠正「从未编译」这个事实**
2. **再 B** —— N-12 元门建在 `EvalHarness` 上

⚠️ **注意**：`neotrix-core/src/l6_meta/healing/nt_mind_eval_harness/nt_harness.rs:99`
已有一个**活的** `EvalHarness::run_dataset()`（被 `nt_core_self_test_integration.rs`
消费）。⇒ **`eval_engine` 接上后会有两套 eval harness**，
需要裁决：`eval_engine` 是新地基，还是并入 `nt_mind_eval_harness`？

**这一条我给不出答案** —— 取决于你想要「通用评测引擎」还是「自我修复专用 harness」。

---

## D-4 🟡 `docs/` 要不要恢复 vitepress 站

### 问题

`477bf669` 删除了 `docs/package.json` + `docs/.vitepress/`（3 个文件）
**但留下了 `docs-deploy.yml` workflow 指向已删的路径**（本轮已 `git rm` 该 workflow）。

现状：`docs/` 下有 **113 个跟踪文件**（全 markdown），无构建系统。

### 选项

| 选项 | 做法 | 代价 |
|---|---|---|
| **A** ⭐ | **不恢复**。`docs/` 就是 markdown 目录，靠 `AGENTS.md` 索引导航 | 0。**113 个 md 已足够** |
| **B** | 恢复 vitepress 站 | 需重写 3 个被删文件 + `package.json` + `package-lock.json`，且**要先在联网环境 `npm install`**。约 1 天 |
| **C** | 换其他静态站生成器 | 同 B，且引入新依赖 |

### 建议

**选 A**。理由：

1. `DOCUMENTATION-MAP.md` + `AGENTS.md` 索引已经能导航
2. 恢复站的**持续成本**（每次加文档要管构建/部署）大于收益
3. 本轮已把 CI 幻影门清掉，若日后要恢复，会被 `check-ci-refs.sh` **正确拦下**
   （除非同时提交 `docs/package.json`）

### ⛔ 不决定的后果

无技术后果（workflow 已删，CI 已绿）。**但会有下一个人重新问这个问题。**
⇒ 建议在 `AGENTS.md` 记一行「`docs/` 刻意无构建系统」。

---

## D-5 🟡 桌面回路的落点（批次 E 全部阻塞于此）

### 问题

2026-09-28 路线图的阶段 2（元素寻址 GUI / `capture_id` / 8 态终局 / 计划失效门）
原本落在 `src-tauri/`。**该目录已随 `5c02e738` 归档（599 files）**。

### 实测

```
src-tauri/                              → 不存在
apps/                                   → 不存在
crates/neotrix-neobot/src/nt_computer.rs → 155 行
  :4   「本地后端 trait 化：当前仅 NoopBackend」
  :98  pub struct NoopBackend;          ← 唯一后端实现
  grep -c "impl ComputerBackend for" = 1
```

### 选项

| 选项 | 做法 |
|---|---|
| **A** ⭐ | 落在 `crates/neotrix-neobot/src/nt_computer.rs`（已有 `ComputerBackend` trait，155 行小文件，扩起来轻） |
| **B** | 重建 `src-tauri/`（599 行的回归） |
| **C** | 落 `neotrix-core/src/l3_embodiment/nt_computer.rs`（477 行，**是 filesystem/process trait，不是 GUI**） |

### 建议

**选 A**。理由：`nt_computer.rs` 只有 155 行且已有 trait 定义；
`neotrix-core` 那份是**完全不同的东西**（embodiment 的 fs/process，不是 GUI），
放进去会造成第 N 次同名不同型。

### ⛔ 不决定的后果

批次 E 的 6 个节点（E1–E6）**全部**停在这里。其中 E6（可逆性感知置信度门控）
是本轮**最有价值的单条机制**（"不确定只在动作不可逆时终止循环"）。

---

## D-6 ⚠️ `qybaihe/mu` 的署名链（法务）

### 实测（GitHub API）

```
fork=False  parent=None  created=2026-09-22  license=MIT  stars=313
```

但 `LICENSE` 首行是 `Copyright (c) 2025 Mario Zechner`（= pi 作者），
**全部提交作者是 `qybaihe <qybaihe@gmail.com>`**，
内部 `kyrn/docs/10-rename-to-mu.md` 记载 2026-09-21 从 kyrn 改名
⇒ **代码最多 6 天历史**。README 自称建于 `earendil-works/pi`(110k★) 与
`AionUi`(33k★) 之上并已署名。

### 问题

**MIT 无来源限制** ⇒ 即使是搬运，代码可抄。**但**：

1. 这是**非 GitHub 血统的搬运**（`fork=False parent=None`）
2. 「5 天大、单作者、来源链不清、README 复数 authors 与单一提交作者不符」
3. 它是我们 `nt_judge.rs` 的**理论来源**

### 选项

| 选项 | 说法 |
|---|---|
| **A** ⭐ | **可吸收设计**（`cacheImpact` 分类 / choice-escape 不变量 / 影子→active），但**不进正典索引**，且**注明「设计来源存疑」** |
| **B** | 向作者发问澄清后再定 | 慢，但干净 |
| **C** | 完全不吸收 | 损失 3 条机制（其中 `cacheImpact` 分类我在任何框架里都没见过） |

### 建议

**选 A + 异步做 B**。不阻塞吸收，但**在正典里标注**：

> `qybaihe/mu` — MIT 声明与提交作者不一致，`fork=False`，6 天大。
> **只取设计模式，不引用其代码，不作为 `nt_judge` 的权威来源。**

---

## D-7 ⚠️ `Aegis` 的 22 个 skill 要不要吸收

### 实测

`Aegis` = 174 个 md / 5,610 行 Python。其 `skills/` 下**与 superpowers 系列大面积同名同义**：
`brainstorming` · `executing-plans` · `test-driven-development` · `systematic-debugging` ·
`using-git-worktrees` · `writing-plans` · `verification-before-completion` …

**本仓已装 superpowers 全套。**

### 选项

| 选项 | 做法 |
|---|---|
| **A** ⭐ | **只取测评方法论**（臂中立 veto 契约 M-1 + 限制四段式 M-12），⛔ **不吸收 `skills/`** |
| **B** | 全吸收 | 会造成 `skills/` 目录**同名冲突** |
| **C** | 都不吸收 | 损失本轮**唯一一份真 A/B + CI 门内**的评测数据（+27.27pp，n=44） |

### 建议

**选 A**。它值钱的不是 skill，是那个**评分器完全不知道跑的是哪个臂**的契约设计。

### ⛔ 附带问题

`Aegis` 的 `LICENSE` 双署名（`Jesse Vincent` / `Ganyuan Ran`），
但 README/CONTRIBUTING/NOTICE **零处**提及与 Jesse Vincent 的关系。
⇒ **商业使用前需确认**（与 D-6 同类）。

---

## D-8 ⚠️ N-6 / N-7 的执行时机（协调）

### 实测（2026-09-29 11:43）

```
crates/neotrix-neobot/src/nt_pdf_ground.rs   mtime 11:43:17  （21 秒前）
crates/neotrix-neobot/src/nt_types.rs        mtime 10:44:14
```

⇒ **另一窗口正在写 `crates/neotrix-neobot/`**。
本轮 `cargo check` 的 18 个错误全在 `nt_pdf_ground.rs`（他窗在制品）。

### 冲突范围

| 节点 | 落点 | 冲突 |
|---|---|---|
| **N-6** 三态工具策略 | `nt_policy.rs:75-145` | 🔴 直接冲突 |
| **N-7** StopReason 正交 | `nt_types.rs:27-33` | 🔴 直接冲突 |
| **C6** 两级熔断 | provider 调用层 | 🟡 可能 |
| **C7** 审批=DB 行 | `nt_store/` | 🟢 较独立 |

### 选项

| 选项 | 做法 |
|---|---|
| **A** ⭐ | **等**。先做批次 A（零 `.rs`），等对方窗口提交后再做 C |
| **B** | 用 worktree 隔离并行 | 需协调合并顺序 |
| **C** | 直接改 | 🔴 会覆盖他窗在制品（`AGENTS.md` 明令禁止） |

### 建议

**选 A**。批次 A（A3–A8）全部零 `.rs`，够做 2–3 天，等窗口自然收工。

---

## D-9 ✅ 已澄清（无需决策，仅记录）

**`Provenance` 默认值 = `ModelAdded` 要不要改？**

**实测**：`experience_tree/mod.rs:68-72`

```rust
impl Default for Provenance {
    /// 默认 `ModelAdded`: 现有条目都是分析产出而非材料原文, 这是**如实**默认。
    /// 反直觉(多数系统默认"有依据"), 但默认值必须说真话 —— 见 R-SCAN-1。
    fn default() -> Self { Provenance::ModelAdded }
}
```

**⇒ ⛔ 不动。** 作者已明确辩护，且它是**如实**的默认（现有条目确是分析产出）。
我第一版把它写成「危险默认」是误判，已在 `FEATURE-MAP-TASKS` 里改正。

**教训已写进文档**：给别人代码贴「危险/死/多余」标签前，先读那段代码的注释
（与 R-SCAN-1 同源）。

---

## 建议的决策顺序

```
第 1 批（今天可定，技术性，我已给全部证据）
  ✅ D-2  选 A：接 hybrid_retrieval 但不接生产，标注 rrf_fuse 重复
  ✅ D-9  已澄清，不动
  ✅ D-4  选 A：docs/ 不恢复，记一行说明

第 2 批（本周）
  🔶 D-1  选 C：先删 6 条 archive，再抽样 20 条判断 (d) 类
  🔶 D-3  选 B：但先要回答「eval_engine vs nt_mind_eval_harness 用哪个」
  🔶 D-8  选 A：等窗口，做批次 A

第 3 批（下周，需外部输入）
  🔶 D-5  选 A：落 neobot（需你确认桌面路线是否还做）
  🔶 D-6  选 A+异步澄清
  🔶 D-7  选 A：只取方法论
```
