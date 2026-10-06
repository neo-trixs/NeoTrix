# handoff — 2026-10-06 capability_invoke 接线 + 星号/编码清理

> 分支 `feat/capability-absorb-20260828` · 提交 `82824585` · 窗口：capability_invoke

## 1. 这一轮做成了什么

**接通能力市场的「模型可调用」通路**（`82824585`，5 文件 +573/-119）。

实测起点（不是推断）：市场门 5/5 上架、0 未上架，但探针里 5 个 trade
能力的 `invoked` 全为 0、`never_invoked` 列满。查证两条根因：

1. `tool_schemas()` 是**硬编码**列表 ⇒ 模型看不见市场里任何条目；
2. `nt_capability_bridge::route_experience` 只把解析结果写进 rationale
   ⇒ **从不执行**能力。

⇒ 「已上架」与「能被模型调用」之间没有通路。本轮建立它。

新增/改动：`ToolName::CapabilityInvoke`、`execute_capability_invoke`、
`nt_capability_registry::record_dispatch`、`tool_schemas` 动态挂载、
`enforced_timeout_ms` 30s 上限。

## 2. 判据与验证（都可复现）

三条新测试 + **三点变异验证**（判别标准是「拆接线必须精确报红」）：

| 变异 | 报红信息 |
|---|---|
| `record_dispatch` → `let after = before` | ★ 调用数未增加 |
| 市场门前插 `return ok:true` | ★ 未上架的 id 竟被放行 |
| `if !market_ids.is_empty()` → `if false` | ★ 没把 capability_invoke 摆上桌 |

门结果：

- `cargo test -p neotrix-neobot --lib` ⇒ **561 passed / 0 failed**（558 + 3）
- `cargo test -p neotrix --lib` ⇒ 13358 passed / 1 failed；唯一红项是**既有**
  `l0_substrate::nt_core_platform::mod_orphan::tests::scan_tree_sorted_by_lines_desc`
  （他窗 `64913227`，与本次无关）
- `nt_lock_audit` core + neobot ⇒ 可疑 **0** 处
- `neobot-check-market.sh` ⇒ PASS（5 上架 / 0 未上架 / 5 金丝雀）
- `check-layer-deps.sh --strict` ⇒ PASS: **0 new**（13 known）

测量台是 `/tmp/nt-x`（`git worktree add --detach HEAD` 的干净检出 + 覆盖
5 个目标文件），**不是**主树。

## 3. ⛔ 诚实边界（最重要的一节）

**本轮没有让能力本体真正执行。** `execute_capability_invoke` 能诚实做到的
是：市场校验 + 解析 + 计数 + 回执。做不到的是执行 trade 本体 ——
执行入口在 `neotrix-core` 的 L1（`nt_act_trade`），而 `neotrix-neobot`
**不依赖** core（core → neobot 是既有方向，反过来循环依赖）。

**刻意没有**在 executor 里编一个「看起来执行了」的假结果：那会让调用数变绿
而能力依然不可用，正是本仓一路在治的「建成未用却看着健康」。

### 3.1 接法建议（三选一，需先裁决）

| 方案 | 做法 | 代价 |
|---|---|---|
| A. 下沉独立 crate | 把 trade 能力本体抽到 `nt-core-trade` 之类**中立** crate，core 与 neobot 都依赖它 | 动目录，最重，但唯一无循环依赖且两侧都能执行 |
| B. 执行端口 + 注册 | neobot 定义 `trait CapabilityInvoker`，core 侧实现并在启动时注册进 neobot 的注册表 | 中等；core 侧启动路径要改；**core 不启动时能力仍不可执行**（须在回执里说清） |
| C. 上移调用 | 保持 core 依赖 neobot，把模型调用侧的「跑一次能力」整体放在 core，neobot 只提供工具协议 | 轻，但会把 agent 循环语义往 core 搬，与本轮方向相反 |

⛔ 选 A/B/C 之前**先看 `TradeCapabilityRegistry` 到底有多少实现、是否纯
函数**（若是纯函数，A 的成本远低于想象）。**不要**为省事让 neobot 直接
`use neotrix_core::...` —— 那是循环依赖，`cargo` 会直接拒绝。

## 4. ⚠️ 顺带查出的**既有**缺陷（本轮未修，避免超范围）

`nt_capability_registry::dispatch_by_capability` **只解析节点就 `counts += 1`**，
而 `nt_capability_bridge` 也走它 ⇒ **rationale 路径会污染**
`registered_never_invoked()`。

⇒ 「能力被路由过」被记成「能力被调用过」，且 `never_invoked` 清单因此不可信。
**修法**：拆成无副作用的 `resolve_by_capability()`（bridge 用）+ 仅在真实
执行成功后调用的 `record_invocation(id)`（executor 用）。
⚠️ 注意 `record_dispatch`（本轮新增）已经是「成功后记账」的语义，
但它与 `dispatch_by_capability` 内部的 `+= 1` **并存** ⇒ 修这个缺陷时
应把后者一并收敛，否则计数规则仍有两处。

## 5. 星号与编码（本轮附带）

- 清除这 5 个文件里**全部**装饰性 `⭐`：`nt_capability_registry` 345、
  `nt_agent` 37、`nt_policy` 22、`nt_types` 9、`nt_http_engine` 9。
  保留 `★`（那是论证标记，与装饰星号不同）。
- 修掉 3 处 **U+FFFD 编码损坏**：
  - `nt_agent.rs` `按最坏算（同 Unknown）` —— 本次引入
  - `nt_types.rs` `之的别名` —— 本次引入
  - `nt_capability_registry.rs` `三参考仓库` —— ⛔ **已在 HEAD 里**，
    非本次引入（`git show HEAD:...` 已核实）
- **全仓普查仍发现 2 处 U+FFFD，未修（他窗文件，只报告）**：
  - `neotrix-core/src/l0_substrate/nt_core_event_bus.rs`
  - `neotrix-core/src/l4_emotion/nt_memory/nt_memory_kb/kb_search.rs`
  ⇒ 建议下一个窗口认领这两处（纯文本修复，1 行内），并把
  `grep -rl $'\ufffd' --include='*.rs'` 加进某个门，否则编码损坏会静默传播。

## 6. 收工自查

- [x] `nt_worktree_gate.sh check` —— `/tmp/nt-x` 为本轮测量台，已 `git worktree remove`
- [x] 自己开的 worktree 已收掉（未手删目录）
- [x] 未提交改动去向：`82824585` 已提交，5 文件全部入库；工作树对这 5 文件干净
- [x] 本轮**未改** feature 门控的 `mod` 宿主，也未动 `Cargo.toml` features
      ⇒ 无需跑 `check-feature-gates.sh`（若下轮要动 `mod`，必须跑）
- [x] 锁审计已重跑（改了 `.rs` ⇒ R-SCAN-3），core + neobot 均 0

## 7. 仍未做（按优先级）

1. **§3 的执行通路**（A/B/C 裁决）—— 这是「能力市场真能用」的最后一块
2. **§4 的计数语义拆分** —— `never_invoked` 清单当前不可信
3. `maybe_compact_context` 未接：它会新增 LLM 调用，需先纳入 `nt_cost.rs` 预算
4. 金丝雀窗口仍**进程全局**，多会话互相 `reset()` ⇒ 需按 `convo_id` 键化
5. `neotrix-core/src/l1_action/nt_io/nt_io_agent_loop/nt_loop_step.rs:156-161`
   的旧 `distill_output` 仍有死循环（neobot 侧已修，core 属他窗）