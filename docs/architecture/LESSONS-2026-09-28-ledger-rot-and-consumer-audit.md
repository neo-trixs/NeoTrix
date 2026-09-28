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

## L7 ⭐⭐⭐ 共享 index 会竞争 —— 我**知道这条规则仍然踩了两次**

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

**处置**：两次都**停手并上报**，没有 `--no-verify`（那会同时破坏 P0 并吞掉 177 个文件）。

> **可复用纪律**：
> 1. 提交前先 `git diff --cached --name-only` **数一遍文件数**，与 `git add` 的清单比对；
>    对不上就别提交。
> 2. index 里若已有**非本会话**的暂存项，**不要 `git commit`** —— 先让对方提交或清空。
> 3. 纯文档改动也会被 `.rs` 的 pre-commit 门拦住 ⇒ **别人构建红时你也提不了**，
>    这是设计使然，不是 bug，但要知道它会表现为「我的改动莫名其妙提不上去」。

---

## L8 ⭐ git 的 "Auto-merging" 会**静默**改坏东西

并入隔离车道时，`scripts/layer-deps-baseline.txt` **不在冲突列表**里，
git 打印 `Auto-merging` 并**无声产出 94 条**（车道那版的脏树值），
把我刚校正过的 102 悄悄退回去。照默认合并提交，CI 立刻 `FAIL: 8 new`，
**且零冲突提示**。

> **可复用判据**：合并后**必须核对门禁数字**，不能只看「有没有冲突」。
> 棘轮类门（baseline/ledger）的自动合并是**静默降级**的高发区。

---

## 元教训：这一轮真正的主线

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
