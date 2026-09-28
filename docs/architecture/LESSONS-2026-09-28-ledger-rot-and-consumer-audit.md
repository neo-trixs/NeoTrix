# 经验沉淀 — 2026-09-28 单窗口收口轮（第三篇）

> 承接 `LESSONS-2026-09-27-scanner-trust.md`（扫描器告警 ≠ 缺陷）与
> `LESSONS-2026-09-28-measurement-and-dedup.md`（脏树比干净树「更干净」）。
> 本文记录本轮**我自己犯的 4 个错** + **1 次被测试抓出的回归** + **1 条量化结论**。
> 全部数据 2026-09-28 实测，测量台见每节标注。

---

## L1 ⭐⭐ 台账不是任务，是**过期快照**——本轮量化：查 40 条，0 条「可修」

**现象**：接手 9 份交接 + `TODO.md` + `BATCH-FIX-CHECKLIST` 后逐条读现场核验，
「30 条一行级生产 bug」里**确认缺陷 0 条**。

| 台账条目 | 实测结论 |
|---|---|
| §6.2-A 的 24 条 + §8.2 的 6 条 | **已由 `f4a4eecd` 修**（该 commit 改动 25 文件，正对应这批）；`git merge-base --is-ancestor` 确认已在 HEAD |
| 其中 8 条（A-4/6/8/9/12/21/22 + A-3） | **前提证伪**：读的是**邻近构造**。例：A-4「`total_cost` 滤掉非 `CostUSD` 报 $0」—— `total_cost` 是裸 `.map(\|u\| u.cost).sum()` **无 filter**，被误读的是隔壁 `total_tokens` 的 filter，且测试**反钉**「跨类型求和是设计」 |
| B-2「`CapabilityRegistry` 零消费者」 | **活路径**（见 L2） |
| B-3「`neotrix-types` 包内 `SkillRegistry` 2→1」 | **字段集不同**（见 L3） |
| 「3 处同名双定义定正典」 | **全部正交**（见 L3） |
| 「`nt-lang` 无 `[lib]`」/「`src-tauri`」 | 已被外部 agent 删除/归档 |
| 「6 脚本硬编码 `M-477231~479942`」 | 那 6 个脚本**在树与 stash 中均已不存在**；活库 max 已从 `M-065651` 涨到 **`M-070817`** |
| 「Yootta gated 401」 | 用户指令移除 |
| 「3 处 `Severity` 派生反向」 | 已由 `f4a4eecd` 对齐；`health_signal.rs` 的倒序是**自陈契约**（"least to most severe"）且消费者用 `match` 不用 `Ord` |

> **可复用判据**：**动手前先花 5 分钟证伪**。本轮 40 条核验省下的工时，
> 远超核验本身；而如果照单全修，会去「修」27 处**正确代码**。

**量化后的话**：一个持续产出的任务台账，如果没有「每条都带核验时间戳 + 核验台」，
**它的期望值是负的** —— 不是「没信息」，而是**主动误导**。

---

## L2 ⭐⭐⭐ 「同名/零消费者」的判据本身会骗你：限定路径搜不到相对路径

**现象**：`handoff-disease-list` 与源码注释都称
`l5_cognition/nt_core/capability/registry.rs:462` 的 `CapabilityRegistry`
「全仓零消费者，仅自测」，依据是注释里写的
「精确搜索 `capability::CapabilityRegistry` 除本文件外无命中」。

**真因**：那条搜索用的是**限定路径**。而唯一的生产消费者写的是**相对路径**：

```rust
// neotrix-core/src/l5_cognition/nt_core/capability/cluster_self_test.rs:24
let empty = super::registry::CapabilityRegistry::new();
```

`cluster_self_test` 是 `mod.rs:31` 的 **`pub mod`**（`:38` 再导出
`CapabilityClusterSelfTest`），供 SEAL 自迭代 pipeline 以 `Box<dyn SelfTest>` 消费
⇒ **生产活路径**，删了打断自测体系。

**为什么危险**：这条判据在台账里被复述了 3 次（B 组 / `OWNERSHIP` / 交接），
**每一次复述都让它更可信**，于是差点删掉一个正在被用的类型。

> **可复用判据**：判活/判死**只能查构造点**（`Type::new(` / `Type {` 的调用方）
> 且**必须覆盖相对路径写法**（`super::` / `self::` / `crate::` / 裸名）。
> 「搜限定路径 0 命中」**不是**「零消费者」的证据。

---

## L3 ⭐⭐ 「N 份同名 → 合并到 1」是**指标**，不是判据（第 6、7 次实例）

本轮实测 4 组「台账要求合并」的同名类型，**全部字段集不同**：

| 类型 | A | B | 结论 |
|---|---|---|---|
| `ExtractConfig`×2 | `page_size:Option`+`max_records`+`filters` | `page_size:u32`+`max_pages`+`customer_ids` | 正交 |
| `EmailConfig`×2 | 时间窗 `since/until`+`max_count` | 分页+`box_id` | 正交 |
| `PlatformRegistry`×3 | `Arc<dyn PlatformAdapter>`+configs | `Arc<dyn ExternalPlatformExtractor>` / 裸 `PlatformConfig` | 三者互异 |
| `SemanticRouter`×2 | `rules:Vec<RouteRule>`+`provider_scores` | `confidence_threshold`+`route_table`+`fallback_model` | 两个不同轴 |

连同历史：`CapabilityRegistry`×4 · `SearchResult`×9 · 三个「决策引擎」 · JEV 反向 ·
`SkillRegistry`×2 ⇒ **9 组同名，全部不同型**。

> **可复用做法**：任何「N 处重复 / N 处违规」的计数，**先逐对比字段集与消费方**
> 再决定合并。机械「9→1」的合并指标会毁掉正在被使用的东西。

---

## L4 ⭐⭐⭐ 我自己犯的错：改共享 API 前**没审消费方** ⇒ 被统一测试当场抓出

**经过**（用户要求「先修复代码，最后统一测试」，这个做法救了这次回归）：

我按台账 **R-4**「`get_all_status` 只遍历 registry，与 `get_status` 不一致」
把返回值改成「registry ∪ 有指标的未注册能力」。理由听起来无懈可击。

**结果**：立刻打破 2 条**既有**测试 ——

```
auto_repair.rs:103  for status in self.monitor.get_all_status() {
```

合成报告的 `success_rate: 0.0` / `call_count: 0` 被 auto_repair 读成
「不健康 ⇒ 需修复」⇒ **凭空造出 3 条修复**（`results.is_empty()` 失败、
`len` 得 3 而非 2）。**已回滚**。

**错在哪**：我把台账里的「不一致」当成了「缺陷」。它只是**一个假设**。
`get_status` 有兜底而 `get_all_status` 没有，**完全可能是刻意的**：
前者是「查任意能力」，后者是「列出**已注册**能力」。

> **可复用判据**：改**共享 API 的返回集合**前，先 `grep` 全部消费方并逐个判断
> 「多出来的元素会不会被解读成异常」。**集合变大比集合变小危险得多** ——
> 变小时元素缺失是显式错误，变大时可能触发下游的隐式假设。
>
> 附带：这条 R-4 后来以另一种形式再次出现 —— 我为它加的锁在
> `--test-threads=4` 下暴露成 `auto_repair` 的 flake（见 L6），
> 两次都指向同一处 `get_all_status` 消费方。

---

## L5 ⭐⭐ 我自己犯的错：**重复造轮子**（`test_home_lock` vs 已有的 `TEST_ENV_LOCK`）

为修「全仓 6+ 处 `set_var("HOME")` 改进程全局环境变量」的下根因，我新建了
`neotrix-core/src/test_home_lock.rs` 并在 7 个函数里挂锁。

提交前才发现：**共享锁早就存在** ——
`l0_substrate/nt_core_self_test.rs:17` 的 `TEST_ENV_LOCK`，
且 `nt_act_crypto/cipher.rs:199` **已在用**，`bin/experience/tests.rs` 还有
自己的进程内锁（**那是对的**：bin 测试与 lib 测试是**不同进程**，跨进程无法共享）。

**已废弃**新模块，只给**真正没保护**的 5 处补既有锁。

> **可复用判据**：「导出不等于调用」同样适用于**基础设施**。
> 新造一个共享设施前，先搜「有没有已经存在的那个」。
> 另注：**先例的适用范围要核实** —— bin 与 lib 是不同测试进程，
> 照抄 lib 的锁到 bin 里是无效的（该文件已有注释写明这点，我仍然先搞错了方向）。

---

## L6 ⭐⭐ flaky 的根因不是环境噪声，是**宿主状态**（本轮 4/4 同类）

统一测试跑到 `--test-threads=4`（CI 历史上因此降级为 2）时暴露 4 条抖动 + SIGSEGV。
逐条读现场，**4 条全是同一类**：

| 测试 | 表面症状 | 真因 |
|---|---|---|
| `auto_repair` ×2 | 时绿时红 | `ComponentMonitor::new()` 的 `memory`/`cpu`/`disk` 走**真实 sys-info**（`>0.90`/`>0.95` 阈值）。宿主一忙就翻 Critical。**本机内存本就常在 75%~90%**，不是低概率而是常态边缘 |
| `heartbeat::test_heartbeat_fresh` | `left:1 right:0` | 断言 `tick_age_ms() == 0`（**精确 0**）。该值是「现在 − last_tick」，`new()` 后只要**经过一次调度**就非 0 |
| `ip_privacy::test_public_ip_range` | 偶发红 | 20 次抽样找 1 个 `Public`（概率 1/5）⇒ 失败率 (4/5)²⁰ ≈ **1.15%** |
| `streaming::test_pipeline_http_streaming` | 偶发红 | 打 `httpbin.org`，实测往返 2.2s，而测试每条消息只给 **5s** |

**修法**（按 LESSONS §6「消掉不确定性才解决」）：
`ComponentMonitor` 加 `with_components()` 让组件集可注入（硬编码 Healthy 的
`network`/`agent_pool` 与恒 Critical 的未知名，让测试不依赖宿主）；
`heartbeat` 改为 `age < STALE_AFTER_MS`；`ip_privacy` 抽样 20→500 且把断言从
「有没有抽到」改成「**每一个** Public 产物都不得含 RFC1918 前缀」。

**结果**：全量 `--test-threads=4` **4 连跑全绿**（107s vs 2 线程 216s），
SIGSEGV 未复现。**但仍不能断言已根除** —— 修前 3 跑 2 崩，P(4 连绿纯属侥幸) = 1.2%。

> **可复用判据**：flaky 归因顺序 —— ① 宿主状态（内存/CPU/时钟/网络）
> ② 随机采样 ③ **进程全局可变状态**。本轮 4 条里 1 条属 ①、1 条属 ①、
> 1 条属 ②、1 条属 ①。
> 「加重试、放宽阈值」都不解决；**注入固定状态**才解决。

---

## L7 ⭐⭐⭐ 共享 index 会竞争 —— 我**知道这条规则仍然踩了两次**（后续又 3 次，合计 5 次；**已解**）

`sessions/handoff-20260928-consolidated.md` §5 与 §4.5 都写明了
「共享暂存区风险：他窗 `git add` 后我 `git diff --cached` 会出现不是我暂存的文件」。

**第一次**：`d5edd461` 只想提 14 个文件，commit 出来 **14 个里有 2 个不是我暂存的**
—— `llama_process.rs` **+349 行**（另一窗的 llama.cpp 本地模型工作）被卷了进来。
原因：preflight 之后 index 仍有**预暂存项**，而 `git commit`（无 `-a`）
提交的是**整个 index**。

**第二次**：我要提 4 个**纯文档**改动，`.githooks/pre-commit` 的 P0 门跑
`cargo check --tests` ⇒ 他窗半成品 `model_pool.rs` 的
`unused imports: llamacpp_port, port_from_url` 让构建红 ⇒ **连文档都提不了**。
此时 index 里堆着 **177 个**他窗在制品，硬闯就会全卷进来。

**处置**：前两次都**停手并上报**，没有 `--no-verify`（那会同时破坏 P0 并吞掉 177 个文件）。

### L7-b ⭐⭐⭐ 后续又发生 3 次，且**这次找到了真解**

19:5x–20:0x 期间连续 3 次提交被 P0 门拒绝，形态与「第二次」不同：

| 次 | 我要提的 | 拦截者 | 是否我窗 |
|---|---|---|---|
| 3 | B-1 双时间 + 分层棘轮 | `nt_finalize_broadcast.rs:11` `E0432` | ❌ 他窗未提交 |
| 4 | 同上 | `nt_core_capability_tree::CapabilityRegistry` 未解析（4+ 处） | ❌ 他窗未提交 |
| 5 | 同上 | 同上（他窗 `d5413335` 删 `apps/neobot-desktop` 后） | ❌ 他窗未提交 |

> ⛔ **这里有一个真实的陷阱，我差点踩进去**：P0 门红时，第一反应是
> 「那 4 处 `CapabilityRegistry` 导入坏了，我顺手修一下」。**那是替别人
> 修他正在做的重构** —— 他随后自己改回来，就会变成我制造的幽灵 diff。
> 正确顺序是**先判定红因归属**，见 L9。

**真解：`.worktrees/` 独立 worktree（本轮首次实际使用，验证有效）**

```bash
git worktree add --detach .worktrees/ratchet HEAD
cd .worktrees/ratchet && git checkout -b fix/xxx
git diff -- <我的文件> > /tmp/mine.patch   # 精确搬移，不整树复制
git apply /tmp/mine.patch
```

- worktree 有**自己的 index 与 HEAD** ⇒ ① 不再卷走他窗暂存项；
  ② 他窗把主树构建搞红，**不影响**我这条车道的 P0 门。
- 实测：主树 P0 红 ×3 拒；同一份改动搬到 worktree 后
  `cargo check --tests` **exit=0**、P0 门 **✅ Build gate passed**、
  一次提交成功（`71e1c412`）。
- 代价：独立 `CARGO_TARGET_DIR` 首编较慢（可导出到 `/tmp/...` 与主树解耦，
  避免与对方抢 cargo 锁）。

> **纪律订正（对上面「可复用纪律」的补充）**：
> 第 3 条（"别人构建红时你也提不了"）**只对共享主树成立**。
> 发现被 P0 拦且红因在他窗 WIP 时，**不要干等** —— 立刻开 worktree 继续，
> 这是唯一能并行推进又不互相污染的姿势。
> `AGENTS.md`「真并行走 `.worktrees/` 隔离」一直写着，只是我直到
> 被拒 3 次才真正执行。**规则早就有了，缺的是被拒时立刻用的反应。**

> **可复用纪律**：
> 1. 提交前先 `git diff --cached --name-only` **数一遍文件数**，与 `git add` 的清单比对；
>    对不上就别提交。
> 2. index 里若已有**非本会话**的暂存项，**不要 `git commit`** —— 先让对方提交或清空。
> 3. ⚠️ **本条原文写错过，现订正**：原写「纯文档改动也会被 `.rs` 的 pre-commit
>    门拦住」。**实测 `.githooks/pre-commit:45` 的条件是**
>    `git diff --cached --name-only | grep -qE '\.rs$'` ⇒ **只要暂存区里有任一
>    `.rs`，纯文档改动才跟着被拦；全 `.md` 的提交根本不会触发 P0 门**
>    （本轮 `c9c1da9d` 三文件全 `.md`，只跑了 doc-drift advisory，秒过）。
>    所以上面「第二次」能被 P0 拦，说明**那次暂存区里确实有 `.rs`**，
>    我原文「4 个纯文档改动」的记述不准确。⇒ **P0 门拦你 ⇒ 暂存区里有 `.rs`**，
>    这本身就是一条可用的诊断信号。

---

## L8 ⭐ git 的 "Auto-merging" 会**静默**改坏东西

并入隔离车道时，`scripts/layer-deps-baseline.txt` **不在冲突列表**里，
git 打印 `Auto-merging` 并**无声产出 94 条**（车道那版的脏树值），
把我刚校正过的 102 悄悄退回去。照默认合并提交，CI 立刻 `FAIL: 8 new`，
**且零冲突提示**。

> **可复用判据**：合并后**必须核对门禁数字**，不能只看「有没有冲突」。
> 棘轮类门（baseline/ledger）的自动合并是**静默降级**的高发区。

## L9 ⭐⭐⭐ **P0 门红 ≠ 我的代码红** —— 判定红因归属的固定手法

第 4 次提交被拒时报 `unresolved import nt_core_capability_tree::CapabilityRegistry`。
第一反应是"去修那 4 处导入"，**但那是他窗正在删的 crate**。
判定只需两个**只读、零成本**的 git 查询：

```bash
git ls-tree -r HEAD --name-only | grep -i capability.tree   # HEAD 侧文件在不在
git grep -ln "pub struct CapabilityRegistry|pub use.*CapabilityRegistry" HEAD -- '*capability*tree*'
```

结果：HEAD 侧 `registry.rs`/`lib.rs` **仍有定义** ⇒ 断链**只存在于他的未提交
工作树**。再用干净 worktree 跑一次 `cargo check --tests` 得 **exit=0**，
双向确认。

> **可复用判据（三问，顺序不能乱）**
> 1. 这个错误是**我的改动引入的**吗？⇒ `git stash` 不用，用上面两条查 HEAD 侧。
> 2. HEAD 侧完好吗？⇒ 完好的话，红因必然是**未提交 WIP**，不是仓库。
> 3. 那我该怎么办？⇒ **不要碰那个文件**（见 L7-b），开 worktree 继续自己的车道。
>
> ⛔ 对应 `LESSONS-20260928-fresh-checkout.md` 的元教训：
> 「我是在哪个环境里验证的」—— 这次具体化为「**我在哪个 HEAD 上验证的**」。

## L10 ⭐⭐⭐ 复现门脚本时，**单文件 ≠ 目录**会让过滤器静默失效

改 `evolution_daemon.rs` 后自查"注释里写了层名会不会污染基线"，
我手工复现 `check-layer-deps.sh` 的过滤器：

```bash
rg -n "(crate::)?l6_meta" <单个文件> | rg -v ':[0-9]+:\s*//'   # ← 注释没被滤掉！
```

**根因**：ripgrep 传**单个显式文件**时**省略文件名字段**，输出是
`20://…`（**一个**冒号），而该模式 `:[0-9]+:\s*//` 需要**两个**冒号
⇒ 永不匹配。门实际是 `rg ... "$SRC/$tree"`（**目录**）⇒ 输出带文件名
⇒ 过滤正常。

**实测确认**（目录模式）我的两行注释确实被滤掉，门只报 198/221/267/847。

> **可复用纪律**：复现任何门/扫描器的行为，**必须用门自己用的调用形态**
> （同一入参类型：目录 vs 单文件、in-process vs 子进程）。
> 这是 `R-SCAN-2 手推 ≠ 实证` 的**第 2 次**实例化 —— 我先在脑子里推，
> 推错，再靠跑真实形态纠正。**门脚本里每个 `rg` 的入参类型都是语义的一部分。**

## L11 ⭐⭐ 「先查 schema 再设计」省掉一次 2 小时高风险迁移

B-1 原计划改 `nodes` 主键为 `(id, transaction_time)` 复合主键。动手前先读
建表语句，发现**两列早已存在**：

- `supersedes TEXT`（`:205`）
- `transaction_time ... NOT NULL DEFAULT 0`（`:212`，
  注释写明是当初为了让「bitemporal 探针能自建样本」才加的）

⇒ **零 schema 变更**。而复合主键方案的实际代价（本轮逐项核实）：

- `nodes` 有 **69 处生产写入**，其中
  `bin/experience/exp_absorb.rs:898,945,1039` 是
  `UPDATE nodes SET metadata=? WHERE id=?` ⇒ 复合主键下**同时改写所有版本**
- **5 处外键** `REFERENCES nodes(id)`（`:237,238,258,277,347`）
  ⇒ 复合主键下报 `foreign key mismatch`
- 既有库还需真实迁移（`CREATE TABLE IF NOT EXISTS` **不改已存在的表**）
- 本会话早前**半迁移过一次并回退**

> **两句话**：
> 1. **半迁移比不迁移更糟** —— 回退成本高于一开始就不做。
> 2. 别人为**探针**顺手加的列，可能是你正经功能的现成地基。写 schema 前
>    必须先读 schema，**不要按想象设计**。

## L12 ⭐⭐ 「门不需要 cargo」≠「修违规不需要编译」

用户指令是"先做**不需要 cargo** 的分层违规棘轮"。我照做并当场见效
（`102 → 101`，`PASS 0 new`，纯 bash 秒级），但必须如实标注代价：

- **门**（`check-layer-deps.sh`）确实零 cargo 依赖 ✔
- 但**修违规本身是改 Rust import** ⇒ 需要编译器来证明没写错 ✘
- 84 个单违规叶子文件里，**只有 1 个**是纯 import 重写
  （facade 早已 `pub use` 出**精确路径**）；其余 83 个要先给 facade
  **加** `pub use`，而加导出极易**同名冲突**（实测 6+ 文件同引
  `CapabilityVector`）—— 无编译器时做，等于批量制造看不见的破坏。

> **纪律**：接受"不跑 cargo"的指令时，**要区分「验证手段」与「修改对象」**。
> 可以只做能被静态门验证的子集（此处 1 处），但**必须显式说明剩下的为什么
> 不能盲做**，而不是假装 101 条都能顺手清掉。

---

**台账、门记录、交接文档 —— 都是「当时的快照」，不是「当前的事实」。**
本轮 9 个 commit 里，价值最高的几个**都不是修 bug**，而是：

1. 把分层门基线从脏树值 **94** 改回干净检出实测 **102**（否则 CI 一直红）
2. 把台账 §3 的 **27 条**伪缺陷标注证伪（省下 27 次错误修复）
3. 修 truth-surface 门「只计数不列清单」（最可交付性的一类看不见）
4. 4 条 flaky 归零，让 `--test-threads=4` 恢复（CI 快一倍且不再抖）

> **一句话**：**先量，再改；先证伪，再修。**
> 本仓的文档体系完备到具有**主动误导性**的程度 ——
> 一个不知道该怀疑台账的 agent，会照着 27 条正确代码去"修"，
> 那是比没有清单更坏的结果。

### 补记（L9–L12 加入后，这条主线更完整了）

本轮后半段（19:5x–20:0x）又补上 4 条，它们的共同形状是
**"我以为在修 A，其实在修 B"**：

| 教训 | 我以为 | 实际 |
|---|---|---|
| L9 | P0 门红 = 我代码坏了 | 断链只在他窗**未提交 WIP**；HEAD 侧完好 |
| L10 | 我的注释污染了基线 | ripgrep **单文件省略文件名** ⇒ 复现方式本身错了 |
| L11 | 需要改主键 + 数据迁移 | 两列**早已存在** ⇒ 零 schema 变更 |
| L12 | 101 条都能顺手清 | 84 个叶子里**只有 1 个**是纯 import 重写 |

> **元教训（升级版）**：**"顺手修一下"是本仓最大的隐性成本来源。**
> 四次里有三次，如果按第一反应动手，我都会在**别人正在改的地方**
> 或**别人早已解决的地方**制造改动。判别它们只需要几个只读命令 ——
> 而这几个命令的成本，远低于"改完发现是幽灵 diff"的返工。
>
> 与 R-SCAN-1 同源：**静态信号（门红 / 扫描器告警 / 台账条目 / 复现失败）
> 一律先问"这是谁的、哪个环境里的"，再动手。**
