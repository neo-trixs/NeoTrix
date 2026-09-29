# IM `/stop` 兑现（worker 池）Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 IM 渠道的 `/stop` 真正可兑现 —— 引入第二执行流（有界 worker 池 + poller 单线程），使长跑轮进行中能被 `/stop` 中止。

**Architecture:** `on_inbound` 切成两半。**T0 半**（碰适配器）留在 poller 线程；**worker 半**（跑轮）进有界池，按 `conversation_id` 串行。出站消息改走 outbox，worker 拿不到适配器。分界判据唯一：`ChannelAdapter` 三方法均无 `Send`/`Sync`。

**Tech Stack:** Rust 2021 · `crates/neotrix-neobot` · `nt_channel_dispatch.rs` 2123 行 / `nt_channel.rs` 433 行 / `nt_channel_serve.rs` 640 行 / `nt_channel_telegram.rs` 2353 行

---

## ⛔ 施工前必读

### 1. 切分判据（不可协商）

`ChannelAdapter`（`nt_channel.rs:151`）：

| 方法 | 签名 | 行 |
|---|---|---|
| `poll` | `&mut self` | `:167` |
| `send` | `&self` | `:170` |
| `fetch_attachment` | `&self` | `:181` |

**三者均无 `Send`/`Sync`，trait 无 supertrait**；`ChannelRegistry` 持
`BTreeMap<String, Box<dyn ChannelAdapter>>`（`:207`）。

⇒ **凡是碰适配器的代码必须在 T0。** 这条不需要协商，也无法绕过。

### 2. ⛔ 设计稿低估了一倍 —— 按 4 处写，不是 2 处

`DESIGN-CHANNEL-DISPATCH.md` §11.1 说「**两处**直连 `adapter.send` 必须改走出站」。
**实测是 4 处**：

```
:416   drop(adapter.send(...))   edit_of: None
:502   drop(adapter.send(...))   edit_of: None
:745   match adapter.send(...)
:872   match adapter.send(...)   ← deliver_result 内
```

### 3. ⛔ 阶段 1 必须先做 —— 那个退化没有任何门会报

设计 §11.1 明确指出：若不把 `edit_of` 放进 outbox payload，
**「编辑原消息」会静默退化成重复两条**，而**这类退化没有任何门会报**。
先修它是因为它**不可观测** —— 等阶段 3 切完线程才发现，定位成本高一个数量级。

### 4. 行号会腐化

本文件行号 2026-09-29 有效（索引已重锚）。**施工时现查**：
```sh
python3 scripts/ops/nt_locate.py --component on_inbound
```

---

# Task 1: 基线固化与 worktree

**Files:**
- Read: `docs/architecture/DESIGN-CHANNEL-DISPATCH.md` §11

**Step 1: 建独立 worktree**

```sh
git worktree add .worktrees/nt-stop -b feat/im-stop HEAD
```
⛔ 共享工作树已有他窗在改 Rust（`audio_decode.rs` / `thumbnail.rs`）。
`AGENTS.md` 并行公约：**禁多窗口边改边跑全量构建**。

**Step 2: 记录基线**

```sh
cargo check -p neotrix-neobot -j4 2>&1 | tail -2
bash scripts/check-layer-deps.sh --strict 2>&1 | tail -2
bash scripts/check-unwrap.sh --strict; echo "rc=$?"
```
Expected: `Finished` 0 error · `0 new` · `rc=0`

**Step 3: 记录现有行为测试点**

跑一遍与 IM 相关的测试，记下基线通过数（后续每一阶段都要对比）：
```sh
cargo test -p neotrix-neobot -j4 2>&1 | tail -15
```

**Step 4: Commit**（worktree 创建本身不需提交，跳过）

---

# Task 2: 补 `edit_of` 进 outbox payload ⭐ 最先做

**为什么排第一**：不可观测的退化必须最先消除。

**Files:**
- Modify: `crates/neotrix-neobot/src/nt_channel_dispatch.rs:781`（`enqueue_outbound_with_attachments`）
- Modify: 同文件 `:416` `:420` `:502` `:506`（两处显式 `edit_of: None`）
- Test: `crates/neotrix-neobot/src/nt_channel_dispatch.rs`（就近测试模块）

**现状**：`payload_edit_of`（`:681`）**已经在读** `edit_of` 键，
但**写入侧没放进 payload** ⇒ 读端永远拿到 `None` ⇒ 编辑退化成新增。

**Step 1: 写失败测试**

```rust
#[test]
fn outbox_payload_carries_edit_of() {
    // 构造一条带 edit_of 的出站消息
    let msg = OutboundMessage {
        channel: "test".into(), chat: "c1".into(),
        text: "edited".into(),
        edit_of: Some("msg-42".into()),
        attachments: vec![],
    };
    // 走 outbox
    enqueue_outbound_with_attachments(&store, &msg);
    // 读回来：payload 必须含 edit_of
    let back = sweep_pending(&store, &mut adapter);
    assert_eq!(back.edit_of.as_deref(), Some("msg-42"),
        "edit_of 未进 outbox payload ⇒ 编辑原消息会退化成重复两条");
}
```
Run: `cargo test -p neotrix-neobot --lib outbox_payload_carries 2>&1 | tail -5`
Expected: **FAIL** —— `edit_of` 为 `None`

**Step 2: 改写入侧**

`enqueue_outbound_with_attachments` 的 payload 构造里加 `edit_of`：
```rust
payload.insert("edit_of", match &msg.edit_of {
    Some(v) => json!(v),
    None => json!(null),
});
```
（键名口径必须与 `payload_edit_of`（`:681`）读的一致 ——
**两处都得看，别只改一边**）

**Step 3: 顺带修口径不一致**

现状：`:416`/`:502` 显式传 `edit_of: None`，而 `:420`/`:506` 附近的
`deliver_result` / `sweep_pending` 在**填** `Some`。
统一为「有就传 `Some`，没有就 `None`」，不要在两处各写一套。

**Step 4: 跑测试**

Run: `cargo test -p neotrix-neobot --lib outbox_payload_carries 2>&1 | tail -3`
Expected: PASS

**Step 5: 端到端验证（关键）**

手工/集成测试：发消息 → 收回复 → **原地编辑**。
**判据：消息数不变、编辑生效。** 任一不满足即回退。

**Step 6: 门禁 + Commit**

```sh
cargo check -p neotrix-neobot -j4 2>&1 | tail -2
bash scripts/check-unwrap.sh --strict; echo "rc=$?"
git add crates/neotrix-neobot/src/nt_channel_dispatch.rs
git commit -m "fix(neobot): edit_of 进 outbox payload（否则编辑原消息静默退化为重复两条）"
```

---

# Task 3: 4 处直连 `adapter.send` 改走出站

**Files:**
- Modify: `crates/neotrix-neobot/src/nt_channel_dispatch.rs:416 :502 :745 :872`

**Step 1: 写失败测试**

```rust
#[test]
fn no_direct_adapter_send_outside_t0() {
    // 扫描源码：on_inbound 之外的函数不应直连 adapter.send
    let src = include_str!("nt_channel_dispatch.rs");
    let direct = src.matches("adapter.send").count();
    // 允许 T0 内的调用；worker 半必须为 0
    assert!(direct <= T0_SEND_ALLOWANCE,
        "还有 {direct} 处直连 adapter.send 未改走出站");
}
```
Run: `cargo test -p neotrix-neobot --lib no_direct_adapter_send 2>&1 | tail -4`
Expected: **FAIL** —— 当前 4 处

**Step 2: 逐处改**

- `:416` `:502`（`on_inbound` 内，指令回执 + 跑轮结果）→ 改 `enqueue_outbound_with_attachments`
- `:745` `:872`（`deliver_result` / `sweep_pending`）→ 同上
- **`deliver_result`（`:859`）签名要去掉 `adapter: &dyn ChannelAdapter`** ——
  worker 拿不到适配器，这是切分的硬约束

**Step 3: 验证直连归零**

```sh
grep -c 'adapter\.send' crates/neotrix-neobot/src/nt_channel_dispatch.rs
```
Expected: `0`

**Step 4: 跑测试 + 门禁 + Commit**

```sh
cargo test -p neotrix-neobot --lib 2>&1 | tail -5
cargo check -p neotrix-neobot -j4 2>&1 | tail -2
git add crates/neotrix-neobot/src/nt_channel_dispatch.rs
git commit -m "refactor(neobot): 4 处直连 adapter.send 改走出站（设计稿说 2 处，实测 4 处）"
```

---

# Task 4: 切 `on_inbound` 为 T0 + worker 两半

**Files:**
- Modify: `crates/neotrix-neobot/src/nt_channel_dispatch.rs:367`（`on_inbound`）
- Modify: 同文件 `:483`（跑轮调用点）

**分界线**：`user_text` 拼好之后、`run_local_turn_cancellable`（`:483`）之前。

| 半 | 步骤 | 归属 |
|---|---|---|
| **T0** | 去重 `mark_seen` / 访问闸门 / 斜杠指令 / 投递 worker | poller 线程 |
| **worker** | `run_local_turn_cancellable` + 结果入 outbox | 池 |

**Step 1: 写失败测试**

```rust
#[test]
fn on_inbound_t0_half_does_not_block_poller() {
    // T0 半必须在有界时间内返回（不含跑轮）
    let start = Instant::now();
    let _ = on_inbound(&store, &config, &engine, &adapter, &bot, &msg);
    assert!(start.elapsed() < Duration::from_millis(200),
        "T0 半阻塞了 {:.1?}，说明跑轮还在里面同步跑", start.elapsed());
}
```
Run: `cargo test -p neotrix-neobot --lib on_inbound_t0_half 2>&1 | tail -4`
Expected: **FAIL** —— 当前 T0 含跑轮，会超时

**Step 2: 抽出 worker 半**

把 `:483` 起的跑轮 + 结果投递抽成独立函数：
```rust
async fn run_turn_worker(
    store: Arc<NeobotStore>, config: Arc<NeobotConfig>,
    engine: Arc<dyn EngineAdapter>, convo_id: String, user_text: String,
) -> Result<(), NtBotError> { /* 原 :483-:5xx 逻辑 */ }
```
⛔ **worker 里绝不能出现 `adapter`**（无 `Send`）。

**Step 3: `on_inbound` 只保留 T0**

T0 半末尾投递：
```rust
pool.submit(RunTurn{ store, config, engine, convo_id, user_text })?;
```

**Step 4: 跑测试 + 门禁 + Commit**

```sh
cargo test -p neotrix-neobot --lib 2>&1 | tail -5
cargo check -p neotrix-neobot -j4 2>&1 | tail -2
git add crates/neotrix-neobot/src/nt_channel_dispatch.rs
git commit -m "refactor(neobot): on_inbound 切 T0/worker 两半（poller 不再被跑轮阻塞）"
```

---

# Task 5: 有界 worker 池 + 按 `conversation_id` 串行

**Files:**
- Create: `crates/neotrix-neobot/src/nt_turn_pool.rs`
- Modify: `crates/neotrix-neobot/src/nt_channel_serve.rs:640`（接入池）
- Test: `crates/neotrix-neobot/src/nt_turn_pool.rs`（就近测试模块）

**Step 1: ⛔ 粒度不是可选项 —— 设计已决断**

| 候选 | 判断 |
|---|---|
| 按 channel | **否**。这正是要去的队头阻塞（一个渠道挂 25s 长轮询 + 300s 跑轮） |
| 按 bot | **否**。一个 bot 服务多个 chat |
| **按 `conversation_id`** | **是**。正确性风险就长在这个粒度上 |

**Step 2: 写失败测试**

```rust
#[tokio::test]
async fn same_convo_never_runs_concurrently() {
    let pool = TurnPool::new(4);
    let a = pool.submit(t("c1")).await.unwrap();
    let b = pool.submit(t("c1")).await.unwrap();
    assert!(a.completed_before(b.started), "同一 convo 并发了");
}
#[tokio::test]
async fn different_convos_do_run_concurrently() { /* 断言真并发 */ }
```
Run: `cargo test -p neotrix-neobot --lib convo_never 2>&1 | tail -4`
Expected: **FAIL**（池还不存在）

**Step 3: 实现有界池**

- 队列**有界** + 背压（满了要么拒要么等，**不要无界 spawn**）
- in-flight 键 = `convo_id`
- 现有 `list_convo_tasks` + `claim_turn_task_id` 是串行的依据，复用它们

**Step 4: 接入 serve**

`nt_channel_serve.rs` 起池并交给 `on_inbound`。

**Step 5: 门禁 + Commit**

```sh
cargo test -p neotrix-neobot --lib 2>&1 | tail -5
cargo check -p neotrix-neobot -j4 2>&1 | tail -2
bash scripts/check-layer-deps.sh --strict 2>&1 | tail -2
git add crates/neotrix-neobot/
git commit -m "feat(neobot): 有界 worker 池 + 按 conversation_id 串行"
```

---

# Task 6: `/stop` 验收 ⭐ 本改造的最终判据

**Step 1: 端到端测试**

```rust
#[tokio::test]
async fn stop_interrupts_running_turn() {
    let pool = /* 起真池 */;
    let turn = pool.submit(long_turn_task()).await.unwrap();
    sleep(200ms).await;                       // 跑轮进行中
    on_inbound(&store, &config, &engine, &adapter, &bot, &msg_stop).await?;
    // ⛔ 判据：回库核对，不只看令牌
    assert!(turn_stop_state(&store, &task_id), "/stop 未真正中止跑轮");
}
```
Run: `cargo test -p neotrix-neobot --lib stop_interrupts 2>&1 | tail -5`
Expected: 先 FAIL，修到 PASS

**Step 2: ⛔ 用 `turn_stop_state` 核对，不只看令牌**

设计 §11 明确记了：「**信号送达**」与「**轮次停止**」是两件事。
只断言「令牌已送达」会得到假绿。

**Step 3: 手工验证**

1. 发起一个真长跑轮（长上下文/多工具调用）
2. 跑轮进行中发 `/stop`
3. 断言：该轮中止（`turn_stop_state` 为真）、**且** 未产生半截输出

**Step 4: 全门禁 + Commit**

```sh
cargo test -p neotrix-neobot -j4 2>&1 | tail -8
cargo check -p neotrix --lib -j4 2>&1 | tail -2
bash scripts/check-layer-deps.sh --strict 2>&1 | tail -2
bash scripts/check-unwrap.sh --strict; echo "rc=$?"
bash scripts/check-fresh-build.sh --full 2>&1 | tail -3
git add crates/neotrix-neobot/ && git commit -m "test(neobot): /stop 端到端验收（turn_stop_state 回库核对）"
```
Expected: 全绿 · fresh-build `PASS (full tier)`

---

# 每 Task 必过的门

```sh
cargo check -p neotrix-neobot -j4        # 禁默认 -j10（16G 机 OOM，NTS-D01）
bash scripts/check-layer-deps.sh --strict
bash scripts/check-unwrap.sh --strict
bash scripts/check-fresh-build.sh --full
python3 scripts/ops/nt_mapgen.py && python3 scripts/ops/nt_topology.py
```

# 风险表

| 风险 | 判据 / 缓解 |
|---|---|
| `edit_of` 静默退化 | Task 2 单独提交 + 专门测试；**无门会报这个** |
| 跨层依赖违规 | 改 `nt_channel.rs`（在 `crates/`）可能碰层门 ⇒ 每步跑 `--strict` |
| `edit_of` 口径不一致 | 现状「`None` 传入 / `Some` 填入」并存 ⇒ Task 2 Step 3 统一 |
| 池无界导致 OOM | 有界队列 + 背压测试 |
| 同 convo 并发跑轮 | `convo_id` 串行 + 专门测试 |
| 行号再次腐化 | 现查 `nt_locate`，勿信本文件 |
| 与他窗冲突 | 独立 worktree，勿在共享工作树原地改 |

# 不要做的事

- ⛔ 勿按设计稿的「2 处 `adapter.send`」写 —— 实测 **4 处**
- ⛔ 勿跳过 Task 2 —— 那个退化**不可观测**，被跳过就会一路带到 Task 6
- ⛔ 勿把 in-flight 粒度定成 channel 或 bot —— 已决断 `conversation_id`
- ⛔ 勿只断言「令牌送达」当作 `/stop` 成功 —— 要回库核对 `turn_stop_state`
- ⛔ 勿在共享工作树原地改这 4 个文件
