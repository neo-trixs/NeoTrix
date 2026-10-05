# handoff — 2026-10-05 星号噪声清理 + 金丝雀改接真实派发路径

> 收工自查见末节。**本窗口最该被继承的经验在 §2，不要跳过。**

## 1. 本窗口做了什么

| commit | 内容 |
|---|---|
| `bd6b5cbe` | 能力市场清单自洽门（互斥/完备/反向一致）+ 注册与上架解耦 |
| `ea5b5424` | 意识自生缺口显式标 `CapabilityKind::Gap` + 3 条变异验证断言 |
| `94cbd34f` | 清除全仓装饰性符号噪声 + 金丝雀接线（**接线对象错误，见 §2.1**） |
| `11280660` | 金丝雀从死代码 `AgentLoop` 改接真实派发路径 `nt_agent.rs` |

### 关键结论（都已实测，不是推断）

- **市场**：`market_listable` / `market_unlisted` 由**市场自己报**，不由门推断。
  第一版拿 canary 当地图，canary 是**监视集合**不是**市场集合**，分母混了
  两个不同集合 ⇒ 报出 `5/10` 这种无意义的数。
- **注册 ≠ 上架**：一个能力缺 `market.license` 会让**其余 4 个从未注册**
  （`assert_market_ready` 返 Err ⇒ 中断 bootstrap）。已解耦：打标失败只 warn。
- **「缺口不是插件」是单点防线**：市场的唯一判据是 `is_listable()`，只看
  `kind.is_marketable() && license && version`。kind 一旦被标错，元数据救不了它，
  也拦不住它 —— 我原先以为有「标对 + 市场兜底」两层，测试当场证伪。
- **金丝雀**：`tick()` / `reset()` 此前**零生产调用者** ⇒ `window_ticks()` 恒 0
  ⇒ 判据 `fired>0 || ticks<3` 的第二项恒真 ⇒ **任何能力永远「健康」**。
  现在接在 `nt_agent.rs`：`reset` 在 `run_local_turn_inner` 入口，
  `tick` 在 `run_loop` 的 `tool_calls` 循环头（每 tool call 一次）。

## 2. 本窗口最该被继承的经验

### 2.1 ⛔ 「测试全绿」不等于「接线存在」——我自己犯的

我把金丝雀 tick 接在 `nt_io_agent_loop::AgentLoop` 上，写了 5 条测试、并行
连跑 3 次全绿，**并向用户报告"已接通"**。实际：

- `AgentLoop::new` 生产**零实例化**；`turn_stream` /
  `turn_stream_with_approval` 零调用者；唯一间接路径 `HiveAgentLoop` 自己
  也是死的。编译活着、测试全绿、**接线为零**。

⇒ **判据**：接线类改动必须回答「谁在生产调用这条路径」。
   `grep` 直接调用点**不够** —— 要追间接路径（`on_inbound` →
   `run_local_turn_cancellable`）。子代理独立复核，结论与本仓既有记录
   `crates/neotrix-neobot/src/nt_qwen_mm.rs:4-6` 一致（该文件 2026-09-29
   就写明了这件事，我没先读它）。

### 2.2 ⛔ 装饰性符号会撑爆工具调用

我曾用符号做注释强调，退化成纯重复：一次生成约 90KB 导致 JSON 解析失败，
另一次撑爆输出被截断。清理时又两次自己制造缺陷：

1. 「顺带清理」正则压坏内嵌 Python 缩进（`if fail:` 块 4 空格 → 1 空格）
   ⇒ **改带 heredoc 的脚本后必须单独 `compile()` 校验内嵌语言**。
2. 端点匹配错，把探针的 `json!` 段连同计算段整块吞掉（编译仍通过！）
   ⇒ **结构性替换后必须核对输出的全部字段**，不能只看编译绿。

两处最终都是靠「照 HEAD 逐字重建」修好的，不是靠推断。

### 2.3 ⛔ 进程级全局单例的并发测试，锁不是答案

断言要读金丝雀窗口，而窗口是 `OnceLock<Mutex<Canary>>` + `AtomicUsize`
的**进程全局**单例，且 `run_local_turn` 入口会 reset 它。本 crate 调用面很宽
（`nt_agent` 5 处 + `nt_http_engine` 1 处 + `on_inbound` 间接路径）。

依次试过三条路，**都不成立**：

1. 按直接调用点逐个测试加 `static Mutex` ⇒ 漏掉间接路径，并行下仍红。
2. 把锁挪进 `run_local_turn_inner`（`#[cfg(test)]` 门控）
   ⇒ **非重入 Mutex 直接自死锁**，测试进程挂死 45 分钟。
   生产编译虽不受影响，但那是「删门换绿」，本仓明令禁止。
3. 子进程隔离探针 ⇒ **方案可行**（已设计，见 §4），本窗口未实施。

⇒ **结论：只能靠进程隔离。** 根治方向是按 `convo_id` 分窗口（键化），
   既解决跨会话污染，又让断言可写。

### 2.4 ⛔ 扫描器/工具告警先读现场

本轮多次出现「工具输出与我的断言不符」的情况（预览行号误导、脚本删掉别人
原有的 `///` 分隔行）。**每一次的处置都是回到 HEAD 逐字重建，而不是继续
在错误状态上打补丁**。

## 3. 待办（按优先级）

| # | 任务 | 状态 |
|---|---|---|
| 1 | 金丝雀窗口按 `convo_id` 键化 | 未做。解锁并发测试，且根治跨会话污染 |
| 2 | 5 个 trade 能力调用数 0 → 正 | **阻塞于更根本的问题**，见下 |
| 3 | `AgentLoop` / `HiveAgentLoop` 死代码处置 | 需用户裁决：接线 or 删 |
| 4 | 能力市场用户可见入口（API/UI） | 未做 |
| 5 | KB namespace/sensitivity → ring 结果层门 | 未做 |
| 6 | UI 门已知失败（smoke 2 / markdown 11）+ 完整 CI | 未做 |

### ⚠️ 关于 #2 的重要发现

`TradeCapabilityRegistry` **同样零生产消费**（全仓只有 `error_conversions.rs`
拿它的错误类型）。所以「把调用数从 0 变正」不是接一根线的事 ——
**这 5 个贸易能力背后没有任何活的使用者**。需要先决定它们该被谁调用。

### 关于 #3

`AgentLoop` + `turn` / `turn_stream` / `turn_stream_with_approval` +
`HiveAgentLoop` 构成一条**自洽、25 条测试全绿、零生产接线**的对话循环。
真实对话由两条完全不同的路径承担：

- 桌面 App → HTTP → crystal `run_agent_loop`（**确定性手序列，不过 LLM**）
- CLI / IM / 定时任务 → `nt_agent.rs::run_loop`（**真 LLM + 真派发**）

## 4. 金丝雀并发测试的可行方案（已设计，未实施）

进程级全局单例 ⇒ 用子进程跑探针：

```rust
// nt_testutil.rs
pub fn run_isolated(tag: &str, body: &str) -> String {
    let exe = std::env::current_exe()?;   // 当前测试二进制
    cmd.arg("--exact").arg("nt_testutil::isolated_probe_body")
       .arg("--nocapture").arg("--ignored")   // 探针标 #[ignore]
       .env("NT_ISOLATED_TAG", tag)
       .env("NT_ISOLATED_BODY", body);
}
// 探针实体：#[test] #[ignore = "由 run_isolated 以子进程方式调起"]
fn isolated_probe_body() { /* 读环境变量，跑一段断言，打印结果 */ }
```

要点：探针必须标 `#[ignore]`（否则正常跑时自己撞自己）；
`NT_ISOLATED_BODY` 用受限的表达式枚举，不要接受任意代码字符串。

## 5. 验证口径（本窗口实测，干净检出）

- `neotrix --lib`：13262 绿（较 `94cbd34f` 少 5 条 = 撤掉的死代码 canary 测试）
- `neotrix-neobot`：531 绿
- `nt_lock_audit`：`neotrix-neobot/src` 与 `neotrix-core/src` 均 **0 处**
- `neobot-check-market.sh`：rc=0，市场清单自洽（5/5 上架）

⚠️ **未归因的偶发**：`neotrix --lib` 多次全量跑中，`test_ring_buffer_capacity`
与检索门 fail-open 两项 60 秒级测试偶发失败（7 次跑 5 绿 2 红）。与本窗口
改动无调用关系，判为全量并行下的资源竞争型超时，**未确证，不归因**。

## 6. 收工自查

| 项 | 状态 |
|---|---|
| 改动入库 | 全部已提交（`bd6b5cbe` / `ea5b5424` / `94cbd34f` / `11280660`） |
| 主树残留 | 我本轮的 5 个文件均已 clean |
| 我开的 worktree | `/tmp/nt-probe` 已回收（零独有内容，逐文件 diff 核实过） |
| 他窗 worktree | `merge-b` / `nt-v2` / `nt-v4` **未动**（`nt-v4` 近 3h 有改动，疑似他窗在用） |
| 锁审计 | 两 crate 均 0 处，改过 `.rs` 已重跑（未沿用旧值） |
| 星号 | 我碰过的 7 个文件均为 0 |
| 死代码残留 | 上一轮接在 `AgentLoop` 上的 tick / `begin_session_window` / 5 条测试**已全部撤回** |

### 未提交改动去哪了

- 主树我本轮的改动：**全部已入库，无遗留**。
- 他窗在共享树的 WIP（`nt_astar.rs` / `headless.rs` / `llm_judge.rs` /
  `sandbox.rs` / `nt_approval.rs` / `nt_action_facade.rs` 等）：**未碰，未提交**，
  归他窗所有。
