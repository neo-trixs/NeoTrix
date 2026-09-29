# 施工计划 · IM `/stop` 兑现（worker 池）

> 依据：`docs/architecture/DESIGN-CHANNEL-DISPATCH.md` §11（该设计 2026-09-29
> 已入库并**重新锚定行号**，见该文件顶部表格）。
> 定位方式：`python3 scripts/ops/nt_locate.py --component <符号>`
>
> **本文件只做计划，不施工。** 理由见 §0。

## 0. 为什么本会话不施工

| 因素 | 实测 |
|---|---|
| 规模 | 4 文件 / `nt_channel_dispatch.rs` **2123 行**、`nt_channel.rs` 433、`nt_channel_serve.rs` 640、`nt_channel_telegram.rs` 2353 |
| 并发 | 共享工作树**已有他窗在改 Rust**（`audio_decode.rs` / `thumbnail.rs`）。`AGENTS.md` 并行公约：禁多窗口边改边跑全量构建 |
| 性质 | 架构改造（引入第二执行流），非局部修补 |

⇒ 需**独立会话 + 独立 worktree**（`git worktree add`），一次只跑一个。

## 1. 当前锚点（2026-09-29 实测，行号已验证）

| 符号 | 位置 | 说明 |
|---|---|---|
| `on_inbound` | `crates/neotrix-neobot/src/nt_channel_dispatch.rs:367` | 要切成两半的函数 |
| `run_local_turn_cancellable` | 同文件 `:235`（定义） | 同步跑轮，**阻塞 poller** |
| 跑轮调用点 | 同文件 `:483` | `on_inbound` 内的同步调用 |
| `enqueue_outbound_with_attachments` | 同文件 `:791` | 已有 outbox，但 payload **缺 `edit_of`** |
| `deliver_result` | 同文件 `:859` | 签名仍 `adapter: &dyn ChannelAdapter` |
| `sweep_pending` | 同文件 `:904` | |

**⚠️ 与设计稿的差异（施工者必读）**：设计 §11.1 说「**两处**直连
`adapter.send` 必须改走出站」，**实测是 4 处**：

```
:416   drop(adapter.send(...))   edit_of: None
:502   drop(adapter.send(...))   edit_of: None
:745   match adapter.send(...)
:872   match adapter.send(...)   ← deliver_result 内
```

⇒ 设计低估了 1 倍。**先按 4 处核，不要按 2 处写。**

## 2. 核心架构判据（2026-09-29 复测，仍成立）

`ChannelAdapter`（`nt_channel.rs:151`）三方法：

| 方法 | 签名 | 行 |
|---|---|---|
| `poll` | `&mut self` | `:167` |
| `send` | `&self` | `:170` |
| `fetch_attachment` | `&self` | `:181` |

**三者均无 `Send`/`Sync`，trait 无 supertrait**；`ChannelRegistry` 持
`BTreeMap<String, Box<dyn ChannelAdapter>>`（`:207`）。

⇒ **切分判据唯一且不可协商：凡是碰适配器的代码必须留在 T0（poller 线程）。**
worker 半拿不到适配器 ⇒ 出站**必须**走 outbox。

## 3. 分阶段施工（每阶段独立可提交、可回退）

### 阶段 0 · 基线固化（必做前置）
1. `cargo check -p neotrix-neobot`（当前 **Finished 0 error**，作基准）
2. 建 worktree：`git worktree add .worktrees/nt-stop -b feat/im-stop HEAD`
3. 记录 `on_inbound` 的现有行为测试点

**判据**：干净检出能构建（`bash scripts/check-fresh-build.sh --full` 当前 PASS）。

### 阶段 1 · 补 `edit_of` 进 outbox payload（**先做这个**）
> 为什么先做：设计 §11.1 明确指出，若不把 `edit_of` 放进 payload，
> **「编辑原消息」会静默退化成重复两条**，而这类退化**没有任何门会报**。

- 现状：`payload_edit_of`（`:681`）**已经在读** `edit_of` 键
- 待做：写入侧 `:791` 的 payload 构造里补上该键
- 顺带修：`:416` / `:502` 现在显式传 `edit_of: None`，而 `:420` / `:506` 附近
  的 `deliver_result` / `sweep_pending` 在**填** `Some` ⇒ 口径不一致

**验证**：发一条消息 → 收到回复 → **原地编辑**，不是新增一条。
**判据**：编辑生效且消息数不变。任一不满足即回退。

### 阶段 2 · 4 处直连 `adapter.send` 改走出站
- `:416` `:502`（`on_inbound` 内，指令回执 + 跑轮结果）
- `:745` `:872`（`deliver_result` / `sweep_pending`）
- 改后 `deliver_result` 签名应去掉 `adapter: &dyn ChannelAdapter`

**验证**：`grep -c 'adapter\.send' nt_channel_dispatch.rs` == 0
**判据**：0 直连；功能不回退（阶段 1 的编辑测试仍过）。

### 阶段 3 · 切 `on_inbound` 为 T0 + worker 两半
- 分界线：`user_text` 拼好之后、跑轮之前（现 `:483` 之前）
- T0 半：去重 / 访问闸门 / 斜杠指令 / 投递 worker
- worker 半：`run_local_turn_cancellable` + 结果入 outbox

### 阶段 4 · 有界 worker 池 + 按 `convo_id` 串行
- in-flight 键 = `conversation_id`（设计 §11.2 决断，**不是** channel、**不是** bot）
- 有界队列 + 背压
- 正确性判据：同一 `convo_id` 不得并发跑轮
  （现有 `list_convo_tasks` + `claim_turn_task_id` 是它的依据）

### 阶段 5 · `/stop` 真正可兑现
**这是本改造的验收标准**：跑轮中发 `/stop` ⇒ 该轮真停。

**验证**：
1. 发起一个长跑轮
2. 跑轮进行中发 `/stop`
3. 断言：该轮中止（查 `turn_stop_state`，**不只看令牌** —— 设计 §11 记了
   「信号送达与轮次」要回库核对）

## 4. 每阶段必过的门

```sh
cargo check -p neotrix-neobot -j4        # 禁默认 -j10（16G 机 OOM，见 NTS-D01）
bash scripts/check-layer-deps.sh --strict
bash scripts/check-unwrap.sh --strict     # 阶段 1-4 会碰 send 路径
bash scripts/check-fresh-build.sh --full  # 干净检出仍能构建
python3 scripts/ops/nt_locate.py --audit  # 新增文件要进索引
```

## 5. 已知风险

| 风险 | 判据 / 缓解 |
|---|---|
| `edit_of` 静默退化 | 阶段 1 单独提交 + 专门测试；**无门会报这个** |
| 跨层依赖违规 | `CrtTimeScale` 类问题：改 `nt_channel.rs`（在 `crates/`）可能碰层门 |
| 改坏 `edit_of` 口径 | 现存「`None` 传入 / `Some` 填入」不一致，阶段 1 统一 |
| 行号再次腐化 | 施工时**重新 `nt_locate` 查**，勿信本文件行号（本文件也会腐化） |
| 与他窗冲突 | 必须独立 worktree，勿在共享工作树原地改 |

## 6. 不要做的事

- ⛔ 勿信本文件的行号 —— 索引 1 秒可重建，**用 `nt_locate` 现查**
- ⛔ 勿按设计稿的「2 处 `adapter.send`」写 —— 实测是 **4 处**
- ⛔ 勿把 in-flight 粒度定成 channel 或 bot —— 设计已决断 `conversation_id`
- ⛔ 勿在本会话/共享工作树原地改这 4 个文件
