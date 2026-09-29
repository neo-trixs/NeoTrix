# 设计：IM 渠道调度并发化

> 日期：2026-09-28 · 性质：**设计文档，零代码改动**（本文写作期间未改任何 `.rs`）
> 范围：`crates/neotrix-neobot` 的渠道入站链路（`nt_channel*`）与 `nt_agent::run_loop`
> 触发原因：`ABSORPTION-DSH-SIDEBAR-IM.md` §P0-3 记下「`/stop` 在当前架构下不可能实现」，
> 并把它归因为「要真做出来得把调度改成并发的（线程池或 async），那是架构级一步」。
> 本文就是把那一「架构级一步」拆到能直接施工的粒度。
>
> **本文的所有代码事实都带 `file:line`，且每一条都由实际读文件核对过**（不是从名字推的）。
> 末尾 §9 附复算命令 —— 数字会随代码推进漂移，用前先跑。
>
> ⚠️ **行号是 2026-09-28 11:25 的快照，且写作期间源文件正在被另一扇窗口改。**
> 实测：`nt_agent.rs` 在本文写作的 15 分钟内从 1239 行涨到 1551 行、
> `nt_channel_telegram.rs` 1005→1823、`nt_channel_dispatch.rs` 993→1191、
> `nt_types.rs` 264→336。本文所有 168 处 `file:line` 都在写完后**逐条重跑核对过一遍**
> （§9 的复算命令即那次核对），但**下刀前必须重读目标文件**：
> 按名字推断位置、或照抄本文的行号，都是在别人的活地上动手
> （`AGENTS.md` 记了 2026-09-22 的三次覆盖事故）。

---

## 0. 先纠三处前提（含两处「现有注释在说谎」）

派题里的框架我认为**一处偏小、两处偏大**。先把偏差摆出来，否则后面每个选项的
成本估算都会建立在错的基数上。

### 0.1 偏小：`NtBotError` **不是**并发障碍

派题说「`NtBotError` 没有被 `Send` 检查过（去核实，别断言）」。核实结果：**它不构成障碍**。

`nt_error.rs:7-27` 的六个变体载荷**全是 `String`**（`Store(String)` /
`Denied{rule:String, reason:String}` / `Engine{engine:String, reason:String}` /
`Invalid(String)` / `Io(String)` / `Codec(String)`）。auto trait 是结构性的，
所以 `NtBotError: Send + Sync + 'static` 成立。（已用一份镜像该枚举形状的独立
`.rs` 交给 `rustc` 验过 `assert_send` + `assert_sync`，通过。）

但**「没检查过」这半句是对的**：全 crate 搜不到任何 `assert_send` / `assert_sync`
（`rg ": Send|: Sync|assert_send" crates/neotrix-neobot/src` 只命中
`nt_channel_telegram.rs:1026` 的一句 `Arc<Mutex<...>>` 文档）。也就是说
**没有任何编译期证据钉住这个事实** —— 今天它成立只是因为载荷恰好都是 `String`，
将来谁往变体里塞一个 `Rc` 或 `std::io::Error` 的持有态，就会在离现场很远的地方炸。
§6 的 S0 就是补这三条断言。

### 0.2 偏大：真正的墙是 `NeobotStore`，而且它是**编译期**的

`nt_store/mod.rs:203-205`：

```rust
pub struct NeobotStore { conn: Connection }
```

`rusqlite::Connection` 的定义是 `{ db: RefCell<InnerConnection>, cache: StatementCache }`
（rusqlite-0.31.0 `src/lib.rs:377-380`），并且只写了
`unsafe impl Send for Connection {}`（`src/lib.rs:382`）—— **没有 `Sync`**。
`RefCell` 是 `Send` 但 `!Sync`，所以 `NeobotStore` 是 **`Send` 但 `!Sync`**。

后果是硬的：`on_inbound` 的第一个参数是 `store: &NeobotStore`
（`nt_channel_dispatch.rs:52`），`RunContext.store` 也是 `&'a NeobotStore`
（`nt_agent.rs:34`），而全 crate 约 100 个 store 方法**一律 `&self`**。
于是 `Arc<NeobotStore>` 根本编译不过，`&NeobotStore` 也无法借给工作线程。

**这一条把「并发化」的真实成本重新定了价**：不是「给几个类型加 `+ Send`」，而是
「决定 store 怎么跨线程」。§2 论证了**最便宜的答案是：每个工作线程自己
`NeobotStore::open` 一次**，不碰任何 store 方法签名。

### 0.3 偏大：还有一堵墙比 `!Sync` 更早撞上 —— 适配器

`nt_channel.rs:180` / `:189`：`BTreeMap<String, Box<dyn ChannelAdapter>>`，
`ChannelAdapter` trait（`nt_channel.rs:124`）**没有 `Send`/`Sync` 约束**。
`poll` 要 `&mut self`（`nt_channel.rs:140`，长轮询推进 offset），`send` 要 `&self`
（`nt_channel.rs:143`，但里面是阻塞 HTTP，`nt_channel_telegram.rs:153-155`）。
所以注册表既不能跨线程移动，也不能跨线程共享。

好消息是：**这条路已经被 outbox 铺好了一半**。`nt_channel_dispatch.rs:384-399`
的 `enqueue_outbound` 存在的意义就是「跑轮结束后不直接发、交给 outbox」，
而 `run_local_turn_inner` 已经在往 outbox 写 `CH_MESSAGE_NEW`（`nt_agent.rs:250-254`）。
只是 `on_inbound` 自己那两次发送（`nt_channel_dispatch.rs:88` 指令回执、
`:148-153` 轮结果）走的是**直连 `adapter.send`**，没走 outbox。
**把这三处改成入 outbox，适配器就再也不必跨线程了** —— 这是 §3 推荐方案的地基。

> **写作期间观察到的实时变化（施工前必须重读）**：另一扇窗口正在把出站这条路
> 往 outbox 上搬 —— `nt_channel_dispatch.rs:400` 新出现了
> `enqueue_outbound_with_attachments`，其文档（`:393-399`）自述
> 「附件路径会原样进 payload……这一段以前是断的（payload 里的附件被静默丢弃）」。
> 也就是说 **§3 推荐方案的地基正在被别人铺**。好消息是方向一致（都指向
> 「适配器不跨线程」）；坏消息是**`edit_of` 仍然不在 payload 里**
> （实测 `rg edit_of nt_channel_dispatch.rs` 只命中 `:92` / `:152` / `:358` / `:441` / `:484`
> 五处，全是 `OutboundMessage` 的构造点，没有一处进 payload），
> 所以 §6-S2 第 7 条那个「编辑原消息静默退化」的隐患**依然成立，且尚未被人堵上**。

### 0.5 本设计成文后，代码侧已修的问题（2026-09-28 复核）

本文的 §8 汇总列出过若干问题；**其中 5 条已在写成本文之后修掉**，
下面记下「当时是什么样、现在改成什么样」，免得下一个 agent 照着旧行号去「修」已经正确的代码
（`AGENTS.md` R-SCAN-1：静态告警先读现场证实或证伪）。

| 当时的问题 | 现在的代码 | 状态 |
|---|---|---|
| `dedup_key(channel, message_id)` 不含 chat —— 平台 message_id **按 chat 各自编号**，A 群 5 号与 B 群 5 号撞成同一键，**后一条被当重复静默丢弃**（多机器人/多群正是要支持的场景） | `dedup_key(channel, chat, message_id)`，调用点传 `msg.chat`；回归测试 `dedup_key_is_scoped_by_channel_and_chat` | **已修** |
| 入站附件落在 `<data_dir>/attachments`，而 jail 只放行 `<data_dir>/workspace` —— 两者是**兄弟目录**，于是给模型的那条「读它用 read_file，path=…」指向网关必拒的绝对路径，**附件收了但永远读不到** | 落进 `workspace_dir/attachments`（不扩 jail，只挪位置），note 给**工作区相对路径**，并按平台原名挑 `read_image`/`read_file`；回归测试 `inbound_attachment_lands_inside_the_jail_and_is_reachable` | **已修** |
| `run_once` 把 `poll()` 放在 `for bot in &bots` 里：适配器**按渠道**注册、offset 全渠道共享，第一个机器人取走全部 update，**第二个及以后的机器人永远收不到消息且一声不响** | 每渠道只 poll 一次，再按各自白名单用 `route_bot` 路由；多机器人时 `warn_shared_adapter_once` 说清共享适配器的限制。桌面 `neobot_channel_poll_once` 同一处也改了 | **已修** |
| `/stop` 回执让用户「到桌面 App 按发送键」—— 那个键只置 `shell.stopRequested` 让渲染跳过增量，**不终止运行**；且 `stop_admits_the_limitation_instead_of_pretending` 断言「桌面 App」必须在回执里，等于**把假建议钉成契约** | 回执只说事实并给**真能生效**的替代路径（退出 App 才会终止进程）；测试改为**禁止**出现「按发送键/它会变停止键」等动作短语 | **已修** |
| 4 处注释在说谎：`TurnStatus` 自称有 `failed/cancelled`（实为 `done/continue/needs_clarification/blocked/waiting`）、`slice_sleep` 自称让 Ctrl-C 200ms 生效（**不检查任何标志**）、`BotRow` 自称「各机器人独立绑定模型」、`nt_cmd_channels.rs:306` 自称「每个机器人各拉一次，各自独立」 | 4 处都改成只描述已实现的行为，并写明「为什么不能写得像实现了」 | **已修** |

**仍然成立**（本文 §8 的其余条目未被本轮触碰）：

- `/stop` **真的可用**仍需架构级改造：调度改并发（线程池 / async）+ `nt_agent` 侧
  的 stop hook。本轮只做到了**不说谎**。
- per-bot 的 `token_env` 与 `model` **存在库里但 serve 不用**：一个渠道只有一个
  共享适配器，跑轮只用全局 `config.engine`。
- `edit_of` **半接**：`deliver_result`/`sweep_pending` 填了 `Some`，但 `send()`
  不看它（不实现 `editMessage`），且 payload 里没有该字段 → 经 outbox 的消息连值都
  传不到适配器。**没有任何一条路径真的会编辑原消息**，用户只会看到重复两条。
- `slice_sleep` 的分片目前**没有用途**，要等「改成检查原子停止标志」那天才有意义。
- 桌面聊天**不解析斜杠指令**（走 `neobot_run_stream`，不经 `nt_channel_cmd::parse`），
  所以在桌面打 `/stop` 会被当**字面文本发给模型**。

### 0.4 纠正一处「现有注释在说谎」（重要，见 §8 汇总）

`nt_channel_cmd.rs:174` 的 `/stop` 回执让用户「**到桌面 App 按发送键（它会变停止键）**」。
这句话的两半都不成立：

1. 桌面那个键**不取消任何东西**。`apps/neobot-desktop/frontend/src/main.ts:573` 的
   `onSend` 在 `shell.running` 时只做 `shell.stopRequested = true; setRunning(false)`；
   `main.ts:634` 的 `channel.onmessage` 首行就是 `if (shell.stopRequested) return;`，
   `main.ts:673` / `:714` 只跳过收尾入库。**Rust 侧的 `spawn_blocking` 继续跑到结束**
   （`nt_cmd_run.rs:143` 起）—— 模型继续烧 token，工具继续执行，只是不画了。
2. 桌面聊天**根本没有 `/stop` 这个命令**。聊天发送走
   `main.ts:594 → invoke("neobot_run_stream")` → `nt_cmd_run.rs:133`，
   全程不经过 `nt_channel_cmd::parse`（`rg "'/new'|/stop" frontend/src/*.ts` 零命中）。
   在桌面聊天里打 `/stop`，它会作为**字面文本发给模型**。

所以现状是：桌面「停止」= 假停止，IM `/stop` = 诚实报错但**给出的替代路径也是假的**。
这比「停不了」本身更值得修 —— 用户会以为能停，于是**不会**去终端按 Ctrl-C。

### 0.5 顺带纠正：`/stop` 这件事比派题说的更小一点

派题 §2(d) 说「只做协作取消，不并发」能修好 `/stop`。**在 `channel serve` 这条路上不能。**
`/stop` 要被读到，前提是有人在跑轮的**同时**去 `poll()`。单线程顺序循环
（`nt_channel_serve.rs:87-167`）里，跑轮期间没有第二个执行流，所以协作取消钩子
装上了也**永远等不到置位的那一端**。§3 因此把 (d) 的定位改成：
**它修不了 `channel serve` 的 `/stop`，但它是 (a)/(b)/(c) 全部三个选项的公共前置件**，
而且它单独就能修好一件用户看得见的事（把 §0.4 的假停止变成真停止）。

---

## 1. 两个问题，精确陈述

### 1.1 `/stop` 在 `channel serve` 里**不可达**（不是因为没接线，是因为接线的那端是死的）

调用链，逐跳：

| # | 位置 | 事实 |
|---|---|---|
| 1 | `bin/neobot.rs:576-604` | `cmd_channel_serve` 是 `loop { run_once(...); slice_sleep(...) }`，**单线程、单执行流** |
| 2 | `nt_channel_serve.rs:87-91` | `run_once` 是普通同步函数，返回 `Result<RoundStats, String>` |
| 3 | `nt_channel_serve.rs:119-125` | 每条消息前先 `adapter.poll()`；**这是唯一一处读平台的地方** |
| 4 | `nt_channel_serve.rs:143-151` | `on_inbound(store, config, engine, adapter_ref, bot, &msg, None)` —— 第 7 个参数 `stop_flag` 传的是 **`None`** |
| 5 | `nt_channel_dispatch.rs:81-83` | `if outcome.stop_requested { if let Some(flag) = stop_flag { *flag = true } }` —— 传了 `None`，**这一支永远不进** |
| 6 | `nt_channel_cmd.rs:172-180` | `Command::Stop` 分支里 `stop_requested: true`（注释明说「将来调度改成并发时，这条命令已经在」） |
| 7 | `nt_channel_dispatch.rs:133-142` | `on_inbound` 调 `run_local_turn_as`，**同步阻塞到整轮多跳结束** |
| 8 | `nt_agent.rs:258-263` | `fn run_loop(ctx, task_id, on_delta, on_step) -> Result<TurnStatus, NtBotError>` —— **签名里没有任何取消入参** |
| 9 | `nt_agent.rs:282-453` | `for n in 0..steps { ... }`：**全循环零个取消检查点** |

**结论**：`/stop` 的管道有三段（命令解析 ✓ → `stop_flag` 置位 ✓ → `run_loop` 消费 ✗），
外加一段**根本不会被调度到**的读消息。第 4 步传 `None` 让第 5 步成为死代码；
即便去掉 `None`（把 `&mut bool` 提到 `run_once` 的栈上），第 9 步也没有地方去读它；
即便第 9 步读它，第 1 步也没有第二个执行流去置位它。**三处都得改，这就是「架构级」的准确含义。**

顺带证实派题那句「`run_loop` 里连一个 stop 钩子都没有」：
`nt_channel_cmd.rs:169-170` 的注释（"整条链路上没有任何停止机制（`run_loop` 里连一个
stop 钩子都没有）"）**与代码相符**，是准确的。

### 1.2 队头阻塞：主因**不是**跑轮，是**长轮询本身**

派题说「一个慢的轮次堵住所有渠道」。读完代码，这个说法**不够准**：有三个叠加的阻塞源，
其中最要命的那个跟模型无关。

| 阻塞源 | 位置 | 阻塞时长 | 触发条件 |
|---|---|---|---|
| **长轮询** | `nt_channel_telegram.rs:197-198`（`timeout=25`）+ `:153-155`（`ureq` 阻塞 `.call()`） | **最坏 25s，且是每条空闲轮询的常态** | **总是**（无消息时长轮询挂满才返回） |
| 跑轮 | `nt_channel_dispatch.rs:133` → `nt_agent.rs:297-299`（每跳一次引擎调用） | 跳数 × 每跳延迟；`nt_engine.rs:14` 的 CLI 引擎缺省超时 **300s** | 有消息时 |
| 工具 | `nt_agent.rs:798-867`（bash，60s 上限）/ 附件下载 `nt_channel_dispatch.rs:186`（`DOWNLOAD_TIMEOUT` = 120s，`nt_channel_telegram.rs:55`） | 最坏 60s / 120s | 模型真的要跑 |

串起来看：`run_once` 的结构是
`for 渠道 { for bot { poll(); for msg { on_inbound() } } }`（`nt_channel_serve.rs:96-158`），
**全在一个栈上**。所以：

- **空闲时**：`poll()` 挂 25s → `intervals()` 返回 `poll_secs`（`nt_channel_serve.rs:193`，缺省 5）
  → `slice_sleep(5s)`（`bin/neobot.rs:601`）。**一个「5 秒轮询」的渠道实际每 ~30s 才轮一次**，
  因为 sleep 是**加在**长轮询之上的，不是它的节拍。
- **忙时**：某条消息触发 300s 的引擎调用 → 这期间 `nt_channel_serve.rs:119` 的
  `poll()` 对**所有**渠道、所有 bot 都不执行；`nt_channel_telegram.rs:209-213` 的
  offset 在这期间原地不动 → Telegram 侧按 `offset` 语义重发那一段
  → 靠 `channel_seen` 去重表（`nt_store_channels.rs:201-208`，`INSERT OR IGNORE`）兜住。
  **所以「重投」今天不会造成重复处理**（去重是原子的），但会造成
  **一次 `getUpdates` 拉回 300s 的积压**，以及 Telegram 保留期（24h）内的重复拉取。
- **出站也被同一个队头堵住**：`drain_outbox_once` / `sweep_pending` 在
  `nt_channel_serve.rs:160-163`，也就是**所有消息都跑完之后**。一轮里只要有一条
  消息卡 300s，那一轮的**出站全部延迟 300s**。

派题里「适配器的 poll 游标没推进，有重投风险」这句**结论对、机制说反了**：
offset 恰恰是 `poll()` 返回时就推进的（`nt_channel_telegram.rs:209-213`），
真正没推进的是**下一次 `poll()` 的调用时机**。风险不是重投，是**积压 + 24h 保留期内的反复重拉**。

---

## 2. 四个选项：修什么 / 破什么 / 多贵

### (a) 每会话一个线程

**做法**：一个 poller 线程 `poll()`，收到消息后按 `convo_id` `thread::spawn` 一个线程跑
`on_inbound`（线程内自己 `NeobotStore::open` + 自己的 engine）。

| | |
|---|---|
| **修** | `/stop` 有了读取端（poller 一直在读）；跨会话的队头阻塞消失；同会话天然串行（一个会话一个线程） |
| **破** | 线程数 = 活跃会话数，**无上界**。群聊 + 公开 bot 很容易被几百个会话撑爆（每线程默认 2 MiB 栈 + 一条 SQLite 连接） |
| **贵** | 逻辑最简单，但**没有背压**：会话数涨上去之后 OOM，而不是变慢 |

### (b) 有界线程池，按会话串行

**做法**：固定 N 个 worker（`N = min(4, available_parallelism)`）从一个全局队列取
`WorkItem`；每个 `convo_id` 维护一个 **in-flight 集合**，已在飞的会话再收到消息就
**排队**（per-convo FIFO），绝不并发。

| | |
|---|---|
| **修** | 同 (a)，外加**有背压**（内存里最多 N 个在跑，其余排队） |
| **破** | 同会话串行这条不变式从「线程归属」隐含变成**必须显式保证**（§5-H2）；关停要 join（§5-H4） |
| **贵** | 比 (a) 多一个 in-flight 集合 + 一个队列 + 一份关停协议。是 (a) 的正确化版本，不是重写 |

### (c) 真上 async/await

**做法**：把 `ChannelAdapter` / `EngineAdapter` / `NeobotStore` 全改成 async
（或 `#[async_trait]` + `spawn_blocking` 包同步实现）。

| | |
|---|---|
| **修** | (b) 的全部，加上「N 个 IO 等待只占一个 OS 线程」 |
| **破** | ① **`NeobotStore` 的 `!Sync` 一点没被绕过** —— `rusqlite::Connection` 是阻塞的，
async 化它等于接一个 async 驱动（`tokio-rusqlite` / `deadpool`），这是**新依赖 + 新故障面**；
② `ureq`（`nt_channel_telegram.rs:29`、`nt_engine.rs` 链路）是同步 HTTP，async 化要换 `reqwest`，
而 `Cargo.toml:17` 的依赖面是刻意收敛的；③ `#[forbid(unsafe_code)]`（`lib.rs:20`）下
引入 async 生态，间接 unsafe 会顶到这条硬规则（`Cargo.toml:13-20` 全是纯 Rust 依赖）；
④ 引擎侧（`nt_http_engine.rs` 1183 行 / `nt_cli.rs` 子进程）全部要重写 async 包装 |
| **贵** | `tokio` 已在 `Cargo.toml:16` 声明（workspace 侧 `features = ["full"]`），但**确实一行没用**（`rg tokio crates/neotrix-neobot/src` 只命中 `nt_channel.rs:122` 那句自述的注释）。也就是说：**依赖是现成的，架构不是**。用它 = 承认「本 crate 无 async 运行时」这条刻意的设计前提（`nt_channel.rs:121-123`、`nt_channel_serve.rs:18`）作废。这不是补丁，是改地基 |

### (d) 只做协作取消，不并发

**做法**：只在 `nt_agent` 里加取消钩子（§4），dispatch 侧不动。

| | |
|---|---|
| **修** | 取消**机制**存在了 → (a)/(b)/(c) 三条路的后续都变成小改；`nt_channel_cmd.rs:179` 那句「将来调度改成并发时这条命令已经在」的承诺**兑现了一半** |
| **不修** | `channel serve` 的 `/stop` **仍然不可达**（§0.5：`poll()` 只有一个执行流，没人去读 `/stop`）；队头阻塞**一点没动** |
| **破** | 什么都不破。`run_loop` 加一个 `Option<&dyn Fn() -> bool>` 形状的入参，四个公开 wrapper 签名不变 |
| **贵** | **最小**。约 40 行，全在 `nt_agent.rs` 内，不动任何 store 方法、不动任何调用方 |

---

## 3. 推荐：**(d) 先行，然后直接落到 (b)，跳过 (a)，放弃 (c)**

一句话：**(d) 是公共前置件，先做；并发一次做到 (b) 的形态（poller 单线程 + 有界 worker 池 +
per-convo 串行），不要停在 (a)；(c) 现在不做。**

三条理由：

1. **(d) 单独就值得做，且能立刻兑现一个用户可见的修复。** §0.4 查明桌面「停止」是假的。
   有了 `run_loop` 的取消钩子，桌面那条 `spawn_blocking` 就能被真的中止
   （需要一个进程内的 cancel registry + 一次 IPC 触达，见 §6-S1），
   于是「桌面按停止键」从「只隐藏流」变成「真的停」。这修的是**一个正在骗人的按钮**，
   而不是新增一个功能 —— 按本仓自己的判据（「看起来能用但实际不能用比缺功能严重」，
   `ABSORPTION-DSH-SIDEBAR-IM.md` §第三轮），这类修复优先级高于新功能。
2. **(a) 是 (b) 的一个退化版本，不该作为终点停靠。** 两者的唯一区别是「worker 从哪来」。
   既然都要写 in-flight 集合和关停协议，就没理由省掉线程池那 20 行却背上无上界的线程数。
3. **(c) 的成本不在 async 本身，在它会顺手把 `NeobotStore` 拖进 async 驱动。**
   §0.2 说清了：store 是**单条阻塞连接 + `&self` 方法**。在 async 下唯一能真正并行的
   写法是每 worker 一条连接 —— 那正是 (b) 的做法，**用 `spawn_blocking` 包一下就够**，
   根本不需要 async 化 store，也就不需要 `tokio-rusqlite`、不需要换 `reqwest`、
   不需要推翻「本 crate 无 async 运行时」这条前提。**即 (b) 是 (c) 能拿到的全部收益，
   减去它的全部代价。**

**关于派题预期的「先 (d) 再并发」是否就是正解**：我同意分阶段，但**不同意把 (d) 说成
「修了 `/stop`」**（§0.5）。(d) 的真正价值是**为并发铺路 + 止住桌面那个假停止**。
如果只有一次改动的预算，(d) 是对的；如果要做完整的事，(b) 才是 `/stop` 在
`channel serve` 里能工作的**唯一**途径 —— (a) 只是 (b) 的易错版。

---

## 4. 协作取消设计（四个选项的公共前置件）

### 4.1 标志：放哪、什么类型

**类型**：`Arc<AtomicBool>`（`std::sync::atomic`）。**为什么不是 `&mut bool`**：
现有 `on_inbound` 的 `stop_flag: Option<&mut bool>`（`nt_channel_dispatch.rs:58`）在单线程下
够用，但 (b) 之后置位方（poller 线程）与消费方（worker 线程）是**两个线程**，
`&mut` 借不过去；`Arc<AtomicBool>` 两侧各拿一个 `Arc` clone，`Relaxed` 序即可
（这里不需要 `SeqCst`：置位与检查之间没有需要保护的其它数据，
唯一的跨线程一致性由 store 的事务保证）。

**放哪**：三层，按「谁的停止意图」分。

| 层 | 载体 | 覆盖 | 为什么需要它 |
|---|---|---|---|
| L1 进程内 | `Arc<Mutex<HashMap<String /*convo_id*/, Arc<AtomicBool>>>>`，由 dispatcher 持有并 `Arc` 共享给每个 `WorkItem` | `channel serve` 内的 `/stop` | §5 的 in-flight 集合已经保证「一个 convo 同一时刻只有一个 turn」，所以这张表天然是 **1:1**，不需要额外的生命周期管理 |
| L2 跨进程 | 新表 `turn_stops(convo_id TEXT PRIMARY KEY, at TEXT NOT NULL)`，写入用 `INSERT OR IGNORE`，**消费用 `DELETE … RETURNING`** | 桌面 App 与 `channel serve` 是两个进程共用同一个 db（`nt_store/mod.rs:211-213` 的注释明说这是有意的），任一进程处理到的 `/stop` 都要能停掉另一个进程正在跑的那轮 | 这是本文**唯一**的 schema 改动。`migrate()` 全是 `CREATE TABLE IF NOT EXISTS`（`nt_store/mod.rs:242-339`），所以是幂等追加 |
| L3 工具内 | `nt_agent.rs:840-853` bash 的 50ms 轮询循环 | 「已经起了的 `bash` 进程」 | 别处的检查粒度是「一跳」或「一个工具」，唯独这里能做到亚秒 |

**为什么要 L2 而不只是 L1**：桌面 App 的聊天（`neobot_run_stream`）与
`channel serve` 是两个进程、两条 db 连接。`nt_channel_cmd.rs:174` 那句
「到桌面 App 按停止键」之所以是假的（§0.4），根本原因就是**桌面根本没地方告诉
serve 进程「停下」**。只做 L1 的话，用户在桌面按停止仍然停不了 serve 那一轮 ——
那只是把一个假按钮换成另一个假按钮。

**L2 的消费语义必须「取即删」**：`INSERT OR IGNORE` + `DELETE … RETURNING` 组合，
让停止意图**恰好被消费一次**。若只写不清，下一轮会立刻被误杀。
（`nt_store/mod.rs:274-276` 那个 `control(id=1, holder, updated_at)` 表**不能复用**：
它是单行的、字段是 `holder`/`updated_at`，没有 convo 维度。用它会让 A 会话的
`/stop` 杀掉 B 会话正在跑的那轮 —— 跨会话串扰。）

**flag 何时清**：**在跑轮入口清一次**（不是消费时清）。理由：in-flight 集合保证同一
convo 同时只有一个 turn 在跑，所以「进入 turn 时的值」就是「这一轮专属的停止意图」，
不存在「上一个 turn 留下的 true 被这一轮读到」的窗口。
（反过来「消费时清」就有：`/stop` 置位 → turn 检查到 → 清 → 但下一个 turn 起点
与置位竞态 —— 极窄但真实存在，且这类竞态最难复现。）

### 4.2 检查点：`run_loop` 里的三个精确位置

`run_loop` 的 hop 循环是 `nt_agent.rs:282` 的 `for n in 0..steps`，其体跨越
`nt_agent.rs:283-453`。工具循环是 `nt_agent.rs:363` 的 `for call in &turn.tool_calls`，
其体跨越 `nt_agent.rs:364-439`。三个检查点：

| # | 位置 | 插在 | 取消延迟上界 | 作用 |
|---|---|---|---|---|
| **C1** | `nt_agent.rs:282` 之后、`nt_agent.rs:298` 的 `engine.run_turn_stream` / `run_turn_with_history` **之前** | hop 之间 | 一跳（最坏 300s，`nt_engine.rs:14`） | 「别再发起新的一次模型调用」—— **省钱的主要来源**，因为一次模型调用可能又是一整轮昂贵推理 |
| **C2** | `nt_agent.rs:363` 之后、`nt_agent.rs:380` 的 `store.record_audit(&pre_event)?` **之前** | 同一跳内的工具之间 | 一个工具（最坏 60s bash / 120s 附件下载） | **最重要的一处**，见下 |
| **C3** | `nt_agent.rs:840-853` bash 的 `try_wait` 轮询体内 | 子进程存活期 | 50ms + kill | 把已起的 bash 杀掉，唯一能做到亚秒的地方 |

**C2 为什么必须落在 `record_audit` 之前而不是之后 —— 律：审计的「allow」行一旦写下，
就必须真的执行。** `nt_agent.rs:366-367` 的注释把这条律写得很清楚：「网关律：先写审计行
（无论放行与否），再执行。崩溃也不丢『谁动了什么』的记录」。如果 C2 放在
`record_audit` 之后、`execute_tool`（`nt_agent.rs:383-405`）之前，就会造出一条
**声称放行、实际没执行**的审计行 —— 那是本项目明确禁止的「假账」。

**正确的取消分支**：在 C2 命中时**不静默 `break`**，而是往 audit 里写一条
`AuditDecision::Deny` + `rule: Some("cancelled")` 的行再 `break`。理由与
`nt_agent.rs:380` 那条律同源：审计是 append-only 的事实账，
「因为用户叫停所以没执行」也是一条**发生过的事实**，不记就等于这段工具链没发生过。
（`AuditEvent::new` 的签名见 `nt_agent.rs:369-378`，`rule` 字段允许 `Some(String)`。）

**C1 命中后不要自己 `return`**：`break` 出 `for n in 0..steps` 之后，
`nt_agent.rs:456-458` 那段 `if current == TurnStatus::Continue { current = Waiting }`
会自然把「没跑完」映射成 `Waiting`。这正是我们要的语义（人可接手），
不必另写一条早返回路径 —— 少一条路径就少一处未来要维护的分叉。

### 4.3 取消态怎么映射到 `TurnStatus`（**不发明谎言**）

事实：`TurnStatus` **没有** `Cancelled`。`nt_types.rs:9-15` 的五个变体是
`Done / Continue / NeedsClarification / Blocked / Waiting`。
有 `Cancelled` 的是 `TaskStatus`（`nt_types.rs:45-51`）。

（**注意 `nt_types.rs:6` 的文档注释是错的**，见 §8-1。它写「五态：done/continue/waiting/
failed/cancelled」—— 那是 `TaskStatus` 的口径。读注释会以为 `TurnStatus` 有
`Cancelled`，而实际没有。）

**不能做的三个映射**：

| 诱人做法 | 为什么是谎言 |
|---|---|
| 加一个 `TurnStatus::Cancelled` 变体 | 要改 `nt_types.rs:30-39` 的 `parse`、`:19-27` 的 `as_str`、serde 名、`nt_channel_dispatch.rs:269-277` 的 `status_text`、`nt_agent.rs:232-238` 的终态映射、桌面 `asRunResult`（`main.ts:669`）与其 `isTeamConvo`/mood 分支。按 `ABSORPTION-DSH-SIDEBAR-IM.md` §第四轮自己的教训（跨边界传参/契约改动正是历史上出事的地方），**为了一个已有等价表达的状态去动 6 个调用点的跨进程契约，收益为负** |
| 取消时 `return Err(NtBotError::Invalid("cancelled"))` | 走 `nt_agent.rs:215-229` 的 Err 分支 → 落库 `TaskStatus::Failed` + `error`。**「失败」≠「用户主动叫停」** —— 这会让 `/status` 和任务列表把用户的动作显示成故障 |
| 取消时返回 `Ok(TurnStatus::Continue)` | 走 `nt_agent.rs:456-458` → `Waiting` → 映射到 `TaskStatus::Pending`。而 `pending` 在本项目里意味着「可被认领/可重跑」（`nt_store_tasks.rs:51-75` 的 `claim_task` 只看 `claimed_by`，`nt_store_tasks.rs:139-150` 的 `retry_task` 只收 `failed`/`cancelled`）。**没有任何 worker 会来取一个 `pending` 的 IM 轮次**，所以 `pending` 是「看起来还能跑、实际没人跑」 |

**推荐映射（两层各说各的真话）**：

```
run_loop 命中 C1/C2  →  break（落到 nt_agent.rs:449-451 的 Continue→Waiting 兜底）
                     →  run_local_turn_inner 在 nt_agent.rs:213 拿到 status 之后、
                        在 nt_agent.rs:231 的 finished 映射**之前**插一段：

   if stop_token.is_cancelled() {
       // 律：停止是**任务行**的事，不是**这一轮谈成了什么**的事。
       store.save_task(&AgentTask {
           status: TaskStatus::Cancelled,       // nt_types.rs:50 已有的变体
           lease_id: None, lease_until: None,
           error: Some(format!("stopped by user at hop {n}/{steps}")),
           ..task
       })?;
       store.enqueue_outbox(..., "CH_MESSAGE_NEW",
           &json!({"task_id": task.id, "status": "cancelled"}))?;   // 与 nt_agent.rs:250-254 同形
       return Ok(TurnStatus::Waiting);          // 公开签名不变，且 Waiting 语义为「人可接手」
   }
```

**为什么这个映射不撒谎**：
- `TaskStatus::Cancelled` 是**既有的**变体，`cancel_task` 早就写它
  （`nt_store_tasks.rs:128`），`retry_task` 早就收它（`nt_store_tasks.rs:143`），
  桌面早就有徽章。**这条路的 80% 已经铺好了，只是从来没有一条活着的轮次产出过它。**
- 用户能重跑：`retry_task` 接受 `cancelled`（`nt_store_tasks.rs:143`），
  所以「按错了再按一次」有正规出口。
- 停掉的那一轮**没有声称完成**：`Waiting` 的既有中文回执是
  「已行动，等外部条件。」（`nt_channel_dispatch.rs:274`）—— 用户叫停确实就是
  「等外部条件（你重新说）」。
- `error` 列**记下了原因与位置**（第几跳），没有任何信息丢失。
- 给用户看的那句话由 **dispatch 侧**产出（它持有 stop_token，知道自己置过位），
  而不是由 `TurnStatus` 承载 —— 这条分工是本节最关键的一句。

**已知的不对称（如实记下）**：`run_local_turn*` 的公开签名
（`nt_agent.rs:45` / `:68` / `:92` / `:114`）返回 `TurnStatus`，**不携带「被取消」这一信息**；
只有 task 行知道。S1/S2 阶段接受这个不对称（换来 4 个公开 wrapper 零改动、
6 个调用点零改动）。**消除它的时机**：等 dispatcher 侧改成「跑完回读 task 行」
（它本来就要回读，见 §5-H2）而不是「读返回值」，那时 `TurnStatus` 的粗粒度
就不再有任何消费者了。**不要为此提前改公开签名。**

### 4.4 最终 task 行长什么样

```sql
-- tasks 表，由 nt_store_tasks.rs:19-46 的 save_task 写入：
id                = <uuid v4>            -- nt_agent.rs:198
title             = <入站正文摘要>        -- nt_agent.rs:199
status            = 'cancelled'          -- ← TaskStatus::Cancelled.as_str()，nt_types.rs:60
created_at        = <进轮时刻>           -- nt_agent.rs:166/201
updated_at        = <检查点命中时刻>
claimed_by        = NULL                 -- nt_agent.rs:203 起就是 None，不动
claimed_at        = NULL
visibility        = 'team'               -- nt_agent.rs:205
lease_id          = NULL                 -- ← 必须清。nt_store_tasks.rs:128 的 cancel_task 也清
lease_until       = NULL                 -- ← 同上；不清的话 recover_stale_running
                                            --   （nt_store_tasks.rs:155-163）10 分钟内
                                            --   不会碰它，但留着会让「有没有在跑」失真
attempts          = 1                    -- nt_agent.rs:208，不动
error             = 'stopped by user at hop 3/8'   -- ← 位置信息在这一行，不在别处
conversation_id   = <该会话 id>          -- nt_agent.rs:210
```

配套的 `steps` 表会停在最后一次 `add_step`（`nt_store_routines.rs:393`），
`outbox` 会多一条 `CH_MESSAGE_NEW`（`nt_agent.rs:250-254` 那条路）。

**`lease_id`/`lease_until` 清空这一条有个额外好处**：它把 §5-H4 里那个
「Ctrl-C 之后 task 卡在 `running` 十分钟」的洞**顺带缩小了** ——
正常结束和被叫停的轮次都会清租约，只有**进程被杀**才会留下 10 分钟的 `running`。

---

## 5. 正确性风险 × 缓解

### H1 SQLite 写竞争（真实存在，且失败模式是「整轮死掉」）

**事实**：store 是单 `Connection`（`nt_store/mod.rs:204`），`open()` 设了
5s busy timeout + WAL（`nt_store/mod.rs:213-216`，注释明说这是为「CLI 与桌面 App
双进程同库」加的）。**WAL 仍然只允许一个写者**。而一轮里的写是**频繁**的：
每跳一次 `record_ledger`（`nt_agent.rs:319`）、每个副作用一条 `add_step`（`nt_agent.rs:341`）、
每个工具先 `record_audit`（`nt_agent.rs:380`）再 `add_step`（`nt_agent.rs:407`）、
终态 `save_task`（`nt_agent.rs:249`）+ `enqueue_outbox`（`nt_agent.rs:250`）。

**放大器**：这些写全部用 `?` 直接传播 —— `nt_agent.rs:319` / `:341` / `:380` / `:407`
任何一处拿到 `database is locked` 都会**掀翻整轮**，且已经执行过的工具副作用留在这边。

**缓解**：
1. **每 worker 一条自己的连接**（§0.2），而不是共用一条 —— 这样 busy 竞争只在
   「写锁」这一层，不会把一个 worker 的 300s 长轮询期间的**读**也堵住别人。
2. **不要引入显式事务**。当前所有写都是裸 autocommit 语句
   （`nt_store_tasks.rs:20`、`nt_store_ledger.rs:74`），所以 SQLite **永远不需要把
   读事务升级成写事务** —— 这是多连接下最经典的 `SQLITE_BUSY` 死锁来源。
   加事务前必须先想清楚是不是在引入它。（代价：本来就没有多语句原子不变式，
   这不是本文要解决的问题，但要**记下来别顺手破坏**。）
3. **写路径加重试包装**：`busy_timeout` 是 5s，对几百毫秒的写来说过长了 ——
   与其卡 5s 不如「失败→退避 20ms→重试，最多 3 次」。落点：`nt_store` 里
   包一层 `write_retry`，只包写方法（`save_task` / `add_step` / `record_audit` /
   `record_ledger` / `enqueue_outbox` / `mark_seen`），**读方法不包**。
4. **N 收小到 4**。写竞争的严重度与并发写者数近似线性，而收益在
   「一个慢轮不堵别人」处就已经饱和了。单机本地工具，4 够了。

### H2 同会话串行：**必须显式保证，靠 in-flight 集合，不能靠「反正很快」**

**为什么它不只是整洁问题**：`on_inbound` 在跑轮结束后是这样认领「本轮任务」的
（`nt_channel_dispatch.rs:154-158`）：

```rust
let task_id = store.list_convo_tasks(&convo_id, 1)?.first().map(|t| t.id.clone())
```

`list_convo_tasks` 是 `ORDER BY created_at DESC LIMIT 1`
（`nt_store_routines.rs:334`）。**两个并发轮次落在同一会话时，A 会拿到 B 的 task**，
于是 A 把 **B 的回复**发回 A 的消息，B 自己再发一次 —— 用户看到**两条回复，
且内容与问题不对应**。`last_reply_of`（`nt_channel_dispatch.rs:258-267`）是同一个模式。
**这是并发化会引入的最严重的用户可见错误，而且它不会报错、只会「看起来有点怪」。**

**保证机制**：一个 `Mutex<HashSet<String>>`（in-flight 集合）+ RAII guard。

- 线性化点是「`HashSet::insert(convo_id)` 返回 true」这一行 —— 在**跑轮之前**。
- `insert` 返回 false（已在飞）→ 本条 `WorkItem` **压回该 convo 的 FIFO 尾**，不跑。
- guard 的 `Drop` 里 `remove(convo_id)` —— 用 RAII 是为了「`run_loop` 里任何一条
  `?` 早返回、任何一次 panic、任何一次 `break`」都不会漏删。
  （注意本 crate 生产代码禁 `panic!`/`unwrap`（`AGENTS.md` 硬规则），但 `Drop` 保险
  仍然值得，因为 `?` 早返回有十几处。）
- **入队序 = 轮次序**。这是 IM 语义：同一会话的两条消息**必须**按到达顺序跑。
  per-convo FIFO 是唯一能保证它的结构；全局队列 + 忙则重排会乱序。
  （`nt_store_routines.rs:334` 的 `created_at DESC` 决定了乱序会直接显示成乱序。）

**顺带暴露的一个既有设计缺口（不是并发引入的，但并发会放大它）**：
`on_inbound` 的会话绑定是**按 bot 而非按 chat**（`nt_channel_dispatch.rs:101-114`）——
`bot.conversation_id` 只有一个槽位，`msg.chat` 从不参与选会话。
后果是**同一个 bot 收到的所有聊天（DM 与群、不同群）全部塌进同一个会话**，
轮次互相穿插。这与 `nt_channel.rs:14-16` 宣称的「neobot 域模型本来就是 IM 形状
（`conversations(kind dm|group)`）」不符。并发化之前应该先决定这一条：
要么把 `convo_id` 改成按 `(bot, chat)` 查/建，要么在文档里如实写明
「IM 会话 ≠ 平台 chat，当前按 bot 聚合」。**不要在并发化里顺手改它** ——
那是独立的一次改动，独立的风险。

### H3 offset / 重投竞争：**只可能来自「两个线程 poll 同一个 adapter」**

**今天没有这个竞争**：`run_once` 里的 `reg` 是局部变量
（`nt_channel_serve.rs:92`），单线程独占，`reg.get_mut` 拿 `&mut` 后立刻
`adapter.poll()`（`nt_channel_serve.rs:118-125`，注释解释了为什么必须分两段借）。
offset 在 `poll()` 返回时就推进到 `max(update_id)+1`（`nt_channel_telegram.rs:209-213`）。

**并发化后的风险与缓解**：
1. **一个渠道只允许一个线程调 `poll()`**。架构上由「poller 线程独占注册表」
   保证（§3）。**不要为了「让两个渠道并行」而给每个渠道各建一份 adapter** ——
   那样每个 adapter 都从 `offset = 0` 起步（`nt_channel_telegram.rs:95`），
   各自重拉 24h 全量（Telegram 保留期，`nt_channel_telegram.rs:9`），
   N 个渠道 = N 倍拉取量。**单 poller 是特性不是限制。**
2. **去重是原子的，可以当兜底**：`mark_seen` 是
   `INSERT OR IGNORE` + `rows_affected > 0`（`nt_store_channels.rs:201-208`），
   单条 INSERT 本身在 SQLite 里就是原子的。所以即使发生重投，
   **不会重复跑轮**（`nt_channel_dispatch.rs:61-63`）。
3. **但 `dedup_key` 少了一段（这是个真 bug，见 §8-2）**：
   `dedup_key(channel, message_id)`（`nt_channel.rs:271-273`）不带 `chat`。
   而 Telegram 的 `message_id` 是**每 chat 独立计数**的
   （`nt_channel_telegram.rs:245-249` 直接取 `message["message_id"]`）。
   所以 chat A 的 `telegram:11` 和 chat B 的 `telegram:11` 会**互相把对方当重复丢掉**
   —— 一条真实消息被静默丢弃。**这在今天就已经存在，与并发无关。**
   修法：`dedup_key` 加 chat 段（`format!("{channel}:{chat}:{message_id}")`），
   并同步改 `nt_channel.rs:327-332` 的 `dedup_key_is_channel_scoped` 测试。
   **建议在 S0 就修**（一行 + 一个测试），因为它会让后续任何并发测试都不可信。

### H4 关停：在飞轮次会被直接杀掉，且留下 10 分钟的假 `running`

**事实**：本 crate **没有任何信号处理**（`rg "signal|ctrlc|SIGINT|ctrl_c"
crates/neotrix-neobot/src` 零命中）。Ctrl-C 走的是 OS 默认动作 —— 进程立刻消失。
`slice_sleep`（`nt_channel_serve.rs:170-176`）**没有任何检查**（见 §8-3：它的注释
声称的用途并不成立）。

**后果**：Ctrl-C 落在跑轮中间 → 进程死 → `tasks` 里那行还是 `running`，
`lease_until = 进轮时刻 + 600s`（`nt_agent.rs:24` `LEASE_SECS`、`:173-174`），
而 `recover_stale_running` 只回收**租约已过期**的行
（`nt_store_tasks.rs:155-163`：`WHERE status='running' AND (lease_until IS NULL OR lease_until < ?1)`），
下次启动在 `nt_agent.rs:169` 调它时租约还没到期 → **那行 `running` 要挂 10 分钟**，
期间任务列表显示「运行中」，而实际上没有进程在跑。

**缓解**：
1. **在飞轮次要排空**。并发化之后 T0 收到停止意图 → 置 `shutdown` → 不再 poll →
  worker 在 §4 的 C1/C2 检查点自然 bail → 主线程 join 全部 worker → 正常退出。
  **但这只在有信号处理器时成立**，否则 T0 还没读到 `shutdown` 就被 SIGINT 杀了。
2. **加信号处理器是新增依赖**：`lib.rs:20` 有 `#![forbid(unsafe_code)]`，
   所以不能自己写 `libc::signal`，得引 `ctrlc`（或 `signal-hook`）。
   这是并发化里唯一的新依赖，**必须在方案评审时明说**（本仓 `Cargo.toml:13-20`
   的依赖面是刻意收敛的）。
3. **不引依赖的退路**：接受 Ctrl-C 丢当前轮，但**把 `LEASE_SECS` 那个洞补上** ——
   worker 在每个 hop 之后续租（写 `lease_until = now + LEASE_SECS`）。
   这本来就是 `nt_agent.rs:23` 文档里承诺的（「运行租约**心跳**」）而**代码里没有**的
   （`rg "lease_until" nt_agent.rs` 只有 `:174` 写一次、`:240` 清一次，**没有续租**）。
   续租一上，崩溃恢复就从「最多 10 分钟」变成「最多 600s + 心跳间隔」，
   且不再需要信号处理器就能正确恢复。**这一条与并发无关，是独立的存量改进，
   建议独立提交。**

### H5 三个**真正**的 `Send` 障碍（替掉派题里那个已被证伪的 `NtBotError`）

| 障碍 | 证据 | 修法 |
|---|---|---|
| `NeobotStore: !Sync` | `nt_store/mod.rs:204` + rusqlite `src/lib.rs:377-382`（`RefCell` + 只有 `Send`） | **每 worker 一个 `NeobotStore::open`**（不动任何 store 方法签名） |
| `Box<dyn ChannelAdapter>` 既非 `Send` 也非 `Sync` | `nt_channel.rs:180` / `:189`，trait 无 `Send` 约束（`:124`） | **poller 单线程独占注册表**；dispatch 侧改走出站（§0.3） |
| `Box<dyn EngineAdapter>` 无 `Send` | `nt_engine.rs:32` trait 无约束；`nt_channel_serve.rs:135` 每条消息现建一个 `Box<dyn EngineAdapter>` | 改成 `Box<dyn EngineAdapter + Send>`（`engine_for` 的返回值类型，5 个分支全在本文件内，加 trait object bound 是一行） |

**不要用 `unsafe impl Send` 绕过任何一个** —— `lib.rs:20` 是
`#![forbid(unsafe_code)]`，而且这三个都有干净的正解。

### H6 关机/退出时正在跑的 `WorkItem` 队列内容丢失

T0 退出时队列里排着的 `WorkItem` 还没进 `mark_seen` 之后的任何持久化 ——
不对，`mark_seen` 在 T0 的 poll 阶段就写了（§3 的拓扑）。所以队列里丢的是
**已去重但未跑轮的消息** → 重启后 offset 从 0 重拉 → `mark_seen` 判定重复
（`nt_store_channels.rs:201-208`）→ **被静默丢弃，用户的消息永远不跑**。

**缓解**：T0 必须**先 `mark_seen` 再入队**这个顺序不能反；但关停时要把队列里
未跑的 `WorkItem` **重新 `mark_seen` 回滚**（`DELETE FROM channel_seen WHERE dedup_key=?`），
或者更简单：**入队前不 `mark_seen`，改成入队后由 worker 在跑轮前 `mark_seen`**
——但那样两个 worker 可能同时对同一条置位，得配合 per-convo 串行才安全。
**推荐后者**：`mark_seen` 移到 worker 端（per-convo 串行已经保证了同 convo 不会重复），
关停时未跑的消息自动重新可见，零回滚逻辑。

### H7 测试基础设施：`:memory:` 库**无法**跨线程

现有 store 测试清一色 `NeobotStore::open(":memory:")`
（`nt_channel_serve.rs:226`、`nt_channel_dispatch.rs:551`、`nt_channel_cmd.rs:190`、
`nt_store_ledger.rs:171`、`nt_store_tasks.rs:219` …）。**每次 `open(":memory:")`
都是一个独立数据库**，所以并发测试**必须**用临时文件 + 真 `Connection`。
且必须 `remove_dir_all`（现有测试已经这么做了，如 `nt_channel_serve.rs:231`）。

---

## 6. 分阶段实施计划

> 每阶段**独立可发布、独立可测**。「证明测试」是这一阶段**必须新增**的断言，
> 不是「已有的测试顺便还过」。阶段顺序不可换：S1 是 S2 的前置（没有取消钩子，
> 并发化会造出一个「按了没反应」的新 `/stop`）。

### S0 — 修三处已知缺陷（不改架构，独立提交）

| 改什么 | 位置 | 为什么放最前 |
|---|---|---|
| `dedup_key` 加 `chat` 段 | `nt_channel.rs:271-273` + 测试 `:327-332` | H3-3 的真 bug；不修的话后面所有并发测试都在测一个会静默丢消息的链路 |
| 补三条 `Send`/`Sync` 编译期断言 | `nt_error.rs` 测试段 | §0.1：现在「`NtBotError` 是 `Send`」这个事实没有任何编译期证据 |
| bash 轮询循环加租约续租 | `nt_agent.rs` 每 hop 之后 | H4-3：把崩溃恢复从「10 分钟」降下来；与并发无关，独立收益 |
| 修正 `slice_sleep` 的注释 | `nt_channel_serve.rs:15-16` | §8-3：注释在说一件不成立的事，后人会照着它设计 |

**证明测试**：
- `dedup_key_includes_chat` —— `dedup_key("telegram","5","11") != dedup_key("telegram","7","11")`。
- `nt_bot_error_is_send_and_sync` —— `fn assert_send<T:Send>(){}` + `assert_sync` 各调一次。
- 续租测试：跑一个 `LoopForever` 引擎（现成范本 `nt_agent.rs:1261-1305`），
  `max_steps` 设大、`LEASE_SECS` 缩短，断言中途 `lease_until` 已推后。

**独立可发布性**：是。三个改动都不动任何签名。

### S1 — 协作取消（`nt_agent` 内部，约 40 行）

**改什么**：
1. `run_loop` 加一个取消入参（建议 `Option<&dyn Fn() -> bool>` 或
   `Option<&StopToken>`，其中 `StopToken { flag: Arc<AtomicBool>, db: Option<StorePath> }`）。
   **四个公开 wrapper（`nt_agent.rs:45/68/92/114`）签名一律不动** —— 内部
   `run_local_turn_inner(&ctx, on_delta, on_step, stop)` 传 `None`。
2. 插 C1（`nt_agent.rs:282` 之后 / `:296` 之前）与 C2（`nt_agent.rs:363` 之后 /
   `:366` 之前，取消时写 `Deny`+`rule:"cancelled"` 审计行）。
3. 在 `nt_agent.rs:213` 之后 / `:231` 之前插 §4.3 的取消分支，落
   `TaskStatus::Cancelled` 行 + `CH_MESSAGE_NEW` 事件，返回 `Ok(Waiting)`。
4. flag 在跑轮入口清一次（§4.1「何时清」）。
5. 新表 `turn_stops`（L2）+ `mark_stop` / `take_stop` 两个 store 方法 +
   在 `nt_store_upkeep.rs`（该文件就是留存清扫的地方）里切掉过老的行。

**证明测试**：
- `cancel_stops_before_next_hop`：一个「hop 0 置位、hop 1 就不该被调用」的计数引擎，
  断言 hop 1 的调用次数 == 0。
- `cancel_writes_a_deny_audit_row_before_breaking`：断言 audit 表里存在
  `decision='deny' AND rule='cancelled'`，且**该工具的 step 行不存在** ——
  这一条钉住「审计的 allow 行一旦写下就必须执行」那条律不被破坏。
- `cancelled_task_row_is_truthful`：断言 `status='cancelled'`、
  `lease_id IS NULL`、`lease_until IS NULL`、`error` 含 `at hop 1/3`，
  并且 `retry_task` 之后变回 `pending`。
- `stop_is_consumed_exactly_once`：连续两次跑同一 convo，第一轮被停、第二轮
  **必须正常跑完**（钉住「flag 在入口清」而不是「用完才清」）。
- `turn_stop_does_not_cross_conversations`：A 会话置位，B 会话的一轮**不受影响**。

**独立可发布性**：是。这一阶段 `channel serve` 的行为**不变**（仍然不并发、
仍然读不到 `/stop`），但 `nt_channel_cmd.rs:174` 的文案**必须同时改** ——
在 S2 落地前它还得说「跑轮是同步的」，**不能提前说「已停」**（那会造出第二个谎言）。

### S2 — poller 单线程 + 有界 worker 池（= 选项 (b)）

**拓扑**：

```
T0 (poller，唯一碰注册表与适配器的线程)
  loop:
    读 channels/bots（自己的 NeobotStore）
    for 每个启用的渠道: adapter.poll()            ← 阻塞 25s，无所谓，只有它阻塞
      for msg: 解析/建会话/下载附件/拼正文        ← 都在 T0（要适配器 + store）
        入 per-convo FIFO 队列
    drain_outbox / sweep_pending / upkeep         ← 唯一的出站点
Worker × N=min(4, cpus)                            ← 每个自有 NeobotStore::open
  loop: 取 WorkItem；若 in-flight 集合里有它的 convo → 压回该 convo FIFO 尾
        否则 insert（RAII guard）→ 跑轮 → 出站结果入 outbox（不碰适配器）→ guard drop
```

**改什么**：
1. `on_inbound`（`nt_channel_dispatch.rs:51-163`）**按 T0 / worker 切成两半**：
   - T0 半：1) 去重（**移到这里** —— §5-H6）2) 访问闸门 3) 指令 4) 定/建会话
     4.5) 下载附件 5) 拼正文。
   - worker 半：6) `run_local_turn_as` + 出站入 outbox。
   这是**移动**，不是新写逻辑；两侧各自可单测。
2. `stop_flag` 从 `Option<&mut bool>`（`:50`）改成 `Option<&StopToken>`；
   `nt_channel_serve.rs:150` 的 `None` 换成 `Some(&token)`。
3. `drain_outbox_once` / `sweep_pending`（`:275` / `:413`）**只留在 T0**。
4. `engine_for` 返回类型加 `+ Send`（`:67`）。
5. `WorkItem` 里的 `user_text` 与已落盘附件**必须 owned**（`String` / `Vec<SavedAttachment>`），
   因为 `InboundMessage` 借的是 `poll()` 的返回值。
6. `BotRow` 也是 owned（`Clone` 派发，见 `nt_channel_dispatch.rs:109-110` 已有先例）。
7. **`edit_of` 要进 outbox payload**：`deliver_result` 用 `edit_of`
   （`nt_channel_dispatch.rs:441`）编辑原消息，但 `enqueue_outbound` 的 payload
   只有 `channel`/`chat`/`text`（`:353-357`）—— 改走 outbox 就必须补上，
   **否则「编辑原消息」这个特性静默退化**（`sweep_pending` 的 `:428` 也在用 `edit_of`）。

**证明测试**（全部用临时文件库，§5-H7）：
- `one_slow_turn_does_not_block_another_conversation`： convo A 用一个
  「睡 2s 再 done」的引擎（`LoopForever` 改一下即成），convo B 用 echo；
  断言 B 的任务行在 A 结束**之前**就已 `done`。**这一条是 S2 存在的全部理由。**
- `same_conversation_never_runs_two_turns_at_once`：用 `Barrier` 让两个
  同 convo 的 `WorkItem` 同时就位，断言 in-flight 集合在任一时刻只含 1 个 id，
  且 `steps` 行数 == 1 轮。
- `incoming_order_is_preserved_within_a_conversation`：同 convo 三条消息，
  断言 `list_convo_tasks` 的 `created_at` 序 == 入队序。
- `poll_happens_on_exactly_one_thread`：一个计数适配器，断言任意时刻
  `poll` 的并发度 == 1。
- `outbound_edit_of_survives_the_queue`：断言 `edit_of` 从 `WorkItem` 一直
  到 `outbox.payload` 再到 `send` 的 `OutboundMessage`，**三段都在**。
- `shutdown_drains_in_flight_turns`：置 `shutdown` 后，断言在飞轮次
  **落成了终态行**（不是 `running`），队列里未跑的**仍可重投**（§5-H6）。
- 回归：`nt_channel_dispatch.rs` 现有的 13 个测试**全部原样通过**
  （切分不许改语义）—— 这是切分正确性的主要证据。

**独立可发布性**：是，且是 `/stop` 在 `channel serve` 里第一次**真正工作**的那一步。
同阶段把 `nt_channel_cmd.rs:172-180` 的回执改成「已停（停在第 N 跳）」，
并**加一个测试钉住不再说「停不了」**（照 `stop_admits_the_limitation_instead_of_pretending`
`nt_channel_cmd.rs:292-304` 的现成范本改写 —— 那条测试的价值正在于它是**反向**的）。

### S3 — 顺带把桌面那个假停止变成真停止

复用 S1 的取消钩子：桌面 IPC 增一条 `neobot_cancel_turn(convo_id)`，
写 L2 的 `turn_stops` 行；`neobot_run_stream`（`nt_cmd_run.rs:133`）那条
`spawn_blocking` 在 hop 之间读库里的停止意图。`main.ts:573` 的
`shell.stopRequested = true` 之后**补一次 invoke**，而不再只是本地标志。

**证明测试**：`stop_key_actually_stops_the_turn` —— 发一轮长轮 → 调 cancel →
断言该轮在有限步内以 `cancelled` 落库，且**不再有新 hop**。

### S4 — 补齐 H2 暴露的会话归属缺口（**独立评估，不属于并发化**）

把 `convo_id` 从「按 bot 一个槽位」改成「按 `(channel, bot_id, chat)` 查/建」
（`nt_channel_dispatch.rs:101-114`），或如实改文档承认当前是按 bot 聚合。
**这一条应当独立成一次改动**：它改的是数据归属语义，不是调度形状。
混进 S2 会让 S2 的回归判据（「现有 13 个测试原样通过」）失效。

### 明确**不做**的

- **不引入 async/tokio**（§2(c)）。`tokio` 继续留在 `Cargo.toml:16` 当未使用的声明，
  或者在同一次提交里把它删掉以免继续误导（它让下一个人以为 runtime 已就位）。
- **不给任何类型加 `unsafe impl Send`**（`lib.rs:20`）。
- **不把 `dedup_key` 之外的幂等职责下放给模型**。
- **不做 per-chat 的 outbox 顺序保证**（`drain_outbox` 是
  `ORDER BY available_at LIMIT 20`，`nt_store_ledger.rs:117`；
  同 chat 两条出站**可能乱序**）。这是既有限制，不在本次范围，但要在
  S2 的文档里写明，否则用户会当成新引入的 bug。

---

## 7. 什么情况下该放弃这条路（如实列）

并发化不是无条件正确的。以下任一条成立，都应该**停手**而不是硬上：

1. **`NeobotStore` 将来要从「单连接」变成「连接池」**。那本身就是一次 `!Sync` 的
   正面解决，方案 (b) 的「每 worker 一条连接」会与之重复；届时应该先做池化，
   再看是否还需要 (b)。**先做池化会顺手把 H1 的写竞争解决掉一大半。**
2. **实测发现 SQLite 写锁等待已经占一轮耗时的可观比例**。那么真正的瓶颈是
   §5-H1 的写放大（每 hop 一次 ledger、每工具一次 audit + 一次 step），
   **该做的是减少写，不是加线程** —— 加线程只会让锁更热。
3. **产品形态变了**：`channel serve` 变成「一个用户一个 bot」的常驻服务，
   而每个 bot 只有一个会话。此时 (b) 的并发度天然是 1，**整条并发化只剩 `/stop`
   一个收益**，而 `/stop` 可以用 §4 的取消钩子 + L2 的跨进程表以**零并发**实现
   —— 那就是 S1 + S3，别做 S2。
4. **`/stop` 的真实使用率长期接近零**。则 S2 的复杂度（切分 `on_inbound`、
   新队列、in-flight 集合、关停协议）换不到任何可观测价值，而**它引入了
   H2 那个「回复串到别人的会话上」的失败模式** —— 那是**比队头阻塞更糟的**
   用户可见错误。判据：若 30 天内 `channel_stops` 表（无论怎么实现都该有）
   的行数是 0，撤。
5. **同时段活跃会话数实测 < 2**。那么「一个慢轮堵住所有渠道」在现实中从不发生，
   S2 解决的是一个不存在的问题。**先量再改**：
   在 `run_once` 里临时打一行「本轮收到 N 条、耗时 T」，跑一周。
6. **用户只有一个渠道一个 bot，且轮次普遍在 1s 内**（echo / 小模型）。
   同上，并发的收益被 SQLite 写竞争的成本吃掉。
7. **有人正在改 `nt_agent.rs` / `nt_store/mod.rs`**。这不是技术判据但它是硬约束：
   S1 就要动 `nt_agent.rs` 的 hop 循环，S2 要动 store 的连接所有权。
   撞车的代价是三天白干 + 一次覆盖事故（`AGENTS.md` 记录了 2026-09-22 的三次覆盖）。
   **先确认那扇门空着再动手。**

---

## 8. 现有注释/文档**不准确**清单（写作过程中实测）

这一节独立列出来，是因为「文档说有、代码没有」比缺功能更危险
（`ABSORPTION-DSH-SIDEBAR-IM.md` §第三轮的原话）。

| # | 位置 | 它说 | 实际 | 严重度 |
|---|---|---|---|---|
| **1** | `nt_types.rs:6` | 「回合终态协议（**五态：done/continue/waiting/failed/cancelled**）」 | `TurnStatus` 的五态是 `done / continue / **needs_clarification** / **blocked** / waiting`（`nt_types.rs:9-15`、`:19-27`）。注释里那两个是 **`TaskStatus`** 的值（`nt_types.rs:45-51`）。**照这条注释去实现取消会以为 `TurnStatus` 已有 `Cancelled`** | **高**（本文 §4.3 的整个设计就踩在这个错误注释上） |
| **2** | `nt_cmd_channels.rs:306` | 「收取：每个机器人各拉一次（同渠道可挂多个 bot，**各自独立**）」 | 同一个 `ChannelRegistry` 里**只有一个** Telegram adapter（`nt_channel_serve.rs:41`；桌面同构），`poll` 需要 `&mut` 且会推进**适配器内存里的 offset**（`nt_channel_telegram.rs:209-213`）。所以第一个 bot 拉走全部 update 并推进游标，**第二、第三个 bot 拿到空 vec**。而 `dedup_key` 又是 `channel:message_id` 不带 bot（`nt_channel.rs:271-273`），于是同渠道多 bot 时**只有第一个 bot 收得到消息**，其余静默饿死。`nt_channel_serve.rs:112-127` 是同一个形状（且**没有**这条注释，更容易误导） | **高**（多 bot 是 `channel_bots` 表的主键设计 `(channel, bot_id)`，`nt_store/mod.rs:324-331`，即这是**预期用法**） |
| **3** | `nt_channel_serve.rs:15-16` | 「Ctrl-C 要能立刻停 —— 所以 sleep 分片（`slice_sleep`），不然最坏要等满一个轮询周期」 | `slice_sleep`（`nt_channel_serve.rs:170-176`）**不检查任何东西**，只是把一次睡拆成 200ms 的片段。Ctrl-C 走 OS 默认 SIGINT 动作，**与是否分片无关**（进程立刻死）。分片只有在「循环体里每片检查一次退出标志」时才有意义，而那个标志不存在 | 中（会让人以为已有优雅退出机制，从而不去设计关停协议 —— 正是 §5-H4 的成因） |
| **4** | `nt_channel_cmd.rs:174` | 「想立刻打断，请到桌面 App 按发送键（它会变停止键）」 | ① 桌面那个键**不取消任何东西**（只置 `shell.stopRequested` 让渲染跳过，`main.ts:573`/`:634`/`:673`/`:714`；`spawn_blocking` 继续跑完，`nt_cmd_run.rs:143`）；② 桌面聊天**不解析斜杠指令**（走 `neobot_run_stream`，不经 `nt_channel_cmd::parse`），所以 `/stop` 在桌面会被当**字面文本发给模型** | **高**（这是一个**用户会被误导**的指引；且 `stop_admits_the_limitation_instead_of_pretending`（`nt_channel_cmd.rs:292-304`）这条测试**只**断言了「停不了/同步/桌面 App」三个词都在 —— 它把一条假建议**钉成了契约**） |
| **5** | `nt_channel.rs:122-123` | 「长轮询靠 `nt_channel::poll_once` 的阻塞 + 上层节拍驱动」 | `nt_channel` 里**没有** `poll_once`。同名的是桌面 Tauri 命令 `neobot_channel_poll_once`（`apps/…/nt_cmd_channels.rs:282`），在另一个 crate。`rg poll_once` 全仓只有这两处 | 低（但它是 §2/§3 里最容易被引用的一句，指错了地方） |
| **6** | `nt_channel_dispatch.rs:49-50` | 「`stop_flag` 是可选的外部停止信号（**`DaemonGate` 那类**）」 | `DaemonGate`（`nt_daemon.rs:20-25`）的字段是 `debounce` / `last_wake` / `last_text` / `steer`，**没有任何停止标志**。它只有 `push_steer` / `drain_steer`（`:63` / `:68`），即「同轮注入一条消息」，不是取消 | 中（会让人以为已有一套现成的停止设施可用） |
| **7** | `nt_channel_serve.rs:289-310`（测试） | 测试名 `run_once_survives_a_missing_token`，机器人行的 `token_env: "NEOBOT_TG_UNSET_XYZ"`，断言 `failed == 1` | 那条机器人行的 `token_env` **在 serve 路径上从未被读取**（`rg "bot.token_env"` 全仓：`bin/neobot.rs:519-524` 的 `channel list` 展示 + `nt_store_channels.rs:130` 的写库）。适配器用的是 `registry()` 里 `TelegramChannel::new("")` → `DEFAULT_TOKEN_ENV`（`nt_channel_serve.rs:41` + `nt_channel_telegram.rs:85-98`）。所以该测试**因为 `NEOBOT_TELEGRAM_TOKEN` 恰好未设置而通过**，与它声称的变量无关 —— 开发机上设了该变量就会**失败** | 中（一个靠环境巧合通过、且会误导「per-bot token 已接好」这个错误结论的测试） |

**由 #2 与 #7 合并推出的一条独立缺口**：`channel serve` 路径**完全不接 per-bot 绑定** ——
既不读 `bot.token_env`（token 永远是 `NEOBOT_TELEGRAM_TOKEN`），
也不读 `bot.model`（`engine_for(config)` 用全局 config，`nt_channel_serve.rs:135`；
`bot.model` 只出现在 `/status` 的回执文字里，`nt_channel_cmd.rs:120-124`）。
而 `nt_channel.rs:9` 宣称「**每机器人独立绑定**（工作区 / 模型 / 别名 / 白名单）」。
白名单确实接了（`nt_channel_dispatch.rs:66-77`），**模型与 token 没接**。
**建议**：要么接上（`registry()` 按 bot 的 `token_env` 建适配器，engine 按 `bot.model` 选），
要么把 `nt_channel.rs:9` 改成只声称已实现的那几项。**这与并发化无关，但同渠道多 bot 一旦真能收消息就会立刻显形**，所以最好在 S2 之前决定。

---

## 9. 复算命令

本文的每个数字都可以重跑（用前先跑，数字会随代码推进漂移）：

```sh
# 顺序执行 / 无取消钩子（§1.1 的 9 跳）
rg -n "for bot in &bots|adapter\.poll\(\)|on_inbound\(" crates/neotrix-neobot/src/nt_channel_serve.rs
rg -n "stop_requested|stop_flag" crates/neotrix-neobot/src/nt_channel_dispatch.rs
rg -n "fn run_loop" -A5 crates/neotrix-neobot/src/nt_agent.rs
rg -n "stop|Send|Sync|ctrl" crates/neotrix-neobot/src/nt_agent.rs crates/neotrix-neobot/src/nt_error.rs

# 25s 长轮询 + 阻塞 ureq（§1.2）
rg -n "POLL_TIMEOUT_SECS|HTTP_TIMEOUT|\.call\(\)" crates/neotrix-neobot/src/nt_channel_telegram.rs
rg -n "CLI_DEFAULT_TIMEOUT_SECS" crates/neotrix-neobot/src/nt_engine.rs
rg -n "BASH_TIMEOUT" crates/neotrix-neobot/src/nt_agent.rs

# store 单一连接 / Send 障碍（§0.2、§5-H1/H5）
rg -n "busy_timeout|journal_mode|conn: Connection" crates/neotrix-neobot/src/nt_store/mod.rs
rg -n "unsafe impl Send for Connection" ~/.cargo/registry/src/*/rusqlite-0.31.0/src/lib.rs
rg -n "Box<dyn (Channel|Engine)Adapter>" crates/neotrix-neobot/src/nt_channel.rs crates/neotrix-neobot/src/nt_engine.rs

# §8 的七条不准确
sed -n '6p'   crates/neotrix-neobot/src/nt_types.rs            # 1
sed -n '305,312p' apps/neobot-desktop/src/nt_commands/nt_cmd_channels.rs   # 2
sed -n '13,19p;169,177p' crates/neotrix-neobot/src/nt_channel_serve.rs      # 3
sed -n '165,181p' crates/neotrix-neobot/src/nt_channel_cmd.rs               # 4
rg -n "poll_once" --type rust                                          # 5
rg -n "steer|stop" crates/neotrix-neobot/src/nt_daemon.rs                  # 6
rg -n "bot\.token_env|token_env\(\)" --glob '!target' crates apps        # 7
```

---

## 未做（如实列出）

- **本文零代码改动**。所有设计都**未验证可编译**。S0–S4 的每一处 `file:line`
  都是现状坐标，实现时会漂移 —— 以实现时的实际读数为准。
- **本文与另一扇窗口在同一批文件上并行**（见文首的漂移警告）。因此
  ①本文的行号在写完后统一重核过一遍，但下一小时就可能再次失效；
  ②§0.3 引到的 `enqueue_outbound_with_attachments` 是**写作中途才出现的**，
  说明出站改造正在进行中 —— **S2 的第 1 项（切分 `on_inbound`）很可能与它撞车**，
  动手前先确认那扇门空着。
- **没有实测过任何一项性能数字**。「300s 一跳」「25s 长轮询」都是从常量与超时值
  推出的上界（`nt_engine.rs:14` / `nt_channel_telegram.rs:59`），
  **不是观测到的分布**。§7 的判据 4/5/6 都要求先量 —— 而量之前不该动手。
- **没有验证 `available_parallelism` 在目标机器上的取值**，也没考虑
  `neobot channel serve` 与桌面 App 同时开着时两进程抢同一个 db 的情形
  （H1 只覆盖了「同进程多 worker」，跨进程是同一个 busy timeout 机制，
  但 SQLite 层面还会多一层 WAL 写者竞争，**未测**）。
- **`run_loop` 之外的其他轮次入口没有纳入设计**：`nt_routine.rs:94`
  （routine firing）与 `bin/neobot.rs:801/810`（CLI `neobot run`）都调
  `run_local_turn_as`，它们会共用 §4 的取消钩子，但**各自的「谁来置位」未设计**
  （routine 没有用户可按停止键）。本文只覆盖 IM 链路。
- **没有设计 per-chat 的出站顺序保证**（`drain_outbox` 按 `available_at` 排序，
  `nt_store_ledger.rs:117`），这是既有限制，本文只要求**不把它变差**。

---
---

# 附：施工规格（追加于 2026-09-28 12:2x）

> 本节把 §1–§9 的**方向性设计**落成「照着敲键盘」的施工规格：
> 信号载体、检查点行号、落库决断、不可逆清单、可交付切片。
>
> **行号快照：2026-09-28 12:26–12:31 实读核对。** §0/§1/§4 的坐标是 11:25 的快照，
> 期间源文件又长了（实测 `nt_agent.rs` 1551→**1606**、`nt_channel_dispatch.rs` 1191→**1291**、
> `nt_channel_serve.rs` →**568**、`nt_types.rs` 336→**342**）。
> **本节的行号是重新实读的，不是从 §0–§9 抄的**；两者不一致时以本节为准，
> 但下刀前仍须重读目标文件（§10.7 附复算命令）。
>
> **本节零代码改动、零 `cargo` 命令。**

---

## §10 阶段一：cooperative cancellation 的完整管线

### 10.0 先把「阶段一到底交付什么」说死（否则会造出第二个谎言）

`/stop` 的管道三段：命令解析 ✓（`nt_channel_cmd.rs:172-189`，`stop_requested: true` 在 `:188`）
→ 外部信号置位 △（`nt_channel_dispatch.rs:86-91`，今天靠 `Option<&mut bool>`）
→ 回合循环消费 ✗（`nt_agent.rs` 全文件无任何 stop 钩子，实测 `rg "stop|cancel|AtomicBool|Notify" nt_agent.rs` 只命中 `SetTurnStatus` 工具名与测试名）。

**阶段一补的是第 3 段，消不掉第 2 段的那个「没有第二个执行流」**（§0.5 已论证）。
于是必须回答：**阶段一补上第 3 段，谁来置位？** 只有两个诚实的答案：

| 置位方 | 阶段一之后能工作吗 | 需要什么 |
|---|---|---|
| `channel serve` 的 poller 线程 | **不能**。`nt_channel_serve.rs:579-602` 是 `loop { run_once(); slice_sleep() }` 单执行流；`/stop` 那条消息要等 `run_once` 里的 `on_inbound`（`:213`）跑到它，而那必须等当前轮结束。**死结** | 必须先有第二个执行流 → 即阶段二 |
| **桌面 App 的 UI 线程 → IPC → `spawn_blocking`** | **能**。`nt_cmd_run.rs:143` 的 `spawn_blocking` 与 Tauri 主循环**本来就是两个执行流**，`main.ts:573` 的停止键今天缺的只是一次 `invoke` | 只要阶段一 + 一次 IPC（切片 C3） |

**结论（决断）**：**阶段一 + 切片 C3 是一对可独立上线的组合**，它把桌面那个正在骗人的
「停止」键（§0.4 / `main.ts:573` 只置 `shell.stopRequested`）变成真停止。
**阶段一单独上线时，`/stop` 的回执文案一个字都不能改**（`nt_channel_cmd.rs:181-185`
仍必须说「停不了」）—— 这条是硬门，验收测试见切片 C1。

### 10.1 信号载体：`Arc<AtomicBool>` 的具名封装 `StopToken`

**决断**：新增 `nt_cancel` 模块，导出

```rust
pub struct StopToken { flag: Arc<AtomicBool> }
impl StopToken {
    pub fn new() -> Self;                    // flag = false
    pub fn cancel(&self);                    // flag.store(true, Relaxed)
    pub fn is_cancelled(&self) -> bool;      // flag.load(Relaxed)
    pub fn reset(&self);                     // flag.store(false, Relaxed)  ← 只在入轮用一次
}
```

**为什么是 `AtomicBool` 而不是 `Arc<Notify>`**：`Notify` 只在「某个执行流正阻塞在
`.notified().await`」时才有价值，而**本 crate 刻意没有 async 运行时**
（`nt_channel_serve.rs:18` 「无 async：长轮询靠阻塞 `poll` + 分片 sleep 驱动，与本 crate 其余部分同律」；
`Cargo.toml:16` 声明的 `tokio` 在核心 crate 里**一行未用**）。
所有检查点都是**同步轮询**点，不是等待点。为一个「不存在等待方」的通知语义
去引入 `Notify` 会诱使后人在检查点写 `.notified().await`，那是把地基推翻。
唯一「能被唤醒」的地方是 `execute_bash` 的 50ms 睡（`nt_agent.rs:853`），
而那里 50ms 本来就是既有粒度，**不值得为它引入一个 runtime**。

**为什么 `Relaxed` 够**：置位与检查之间没有需要保护的其他数据。唯一需要跨线程一致性
的产物在 SQLite 里，由 store 自己的事务保证。`SeqCst` 在这里是白付屏障。

**落点**：新文件 `crates/neotrix-neobot/src/nt_cancel.rs` + `lib.rs:22` 后加
`pub mod nt_cancel;`（`lib.rs` 的 `pub mod` 块从 `:22` 起，**按字母序插在 `nt_channel` 之前**）。

### 10.2 管线全景：从哪来 → 传到哪 → 被谁读

| # | 环节 | 位置（实读） | 传什么 | 谁读 |
|---|---|---|---|---|
| 1 | 文本 → 指令 | `nt_channel_dispatch.rs:84` `nt_channel_cmd::parse(&msg.text)` | — | — |
| 2 | 指令 → 结果 | `nt_channel_cmd.rs:172-189` `Command::Stop` 分支；`:188` `stop_requested: true` | `CommandOutcome.stop_requested`（字段定义 `:91`） | 第 3 环 |
| 3 | 结果 → 信号 | `nt_channel_dispatch.rs:86-91`（`:86` 判、`:87-90` 置位） | **今天** `Option<&mut bool>`（`:58` 形参）；**改造后** `Option<&StopToken>` | 第 4 环 |
| 4 | 信号 → 调度层注册表 | **新增**：dispatcher 持 `Arc<Mutex<HashMap<String /*convo_id*/, Arc<StopToken>>>>` | `WorkItem { convo_id, stop: Arc<StopToken>, .. }` | 第 5 环 |
| 5 | 调度层 → 引擎回合 | `nt_agent.rs:138-147` `run_local_turn_as` 调用点 | `Option<&StopToken>` 形参（新增，见 §10.5） | `run_local_turn_inner` → `run_loop` |
| 6 | 回合 → 工具 | `nt_agent.rs:384` `execute_tool(...)` 调用点 | `&StopToken` | `execute_tool` → `execute_bash` |
| 7 | 工具 → 子进程 | `nt_agent.rs:844/846-852/853` `try_wait` 轮询臂 | `StopToken::is_cancelled()` | `execute_bash` 的 `None` 臂 |

**当前形态 vs 目标形态的唯一差别在第 3 环的载体**，第 4–7 环是纯新增。

**第 4 环的注册表与 §11.2 的 in-flight 集合是同一张表**（决断）：
`HashMap<convo_id, Arc<StopToken>>` 中 **「键存在」= 该会话正在跑**，
**「值」= 那一轮专属的停止令牌**。`/stop` 的处置因此只有三条，且都不需要额外的生命周期表：
- 键存在 → `token.cancel()`（真停住了）
- 键不存在 → 没有在跑的轮次，**如实回「这一轮已经结束了，没有可停的」**，不置任何旗
- 键存在但已跑完（竞态）→ `cancel()` 打在已结束的轮次上，它随后被 `reset()`（§10.3 C0）清掉，无害

### 10.3 `nt_agent` 的检查点清单（按代码结构，含精确插入点）

`run_loop` 是私有函数（`nt_agent.rs:258`，签名 `:258-263`），**唯一调用点**是
`run_local_turn_inner` 的 `:213`。hop 循环 `:282`，循环体 `:283-454`；
工具循环 `:363`，循环体 `:364-444`。四个检查点：

| # | 插在（行号锚点） | 命中时做什么 | 语义 | 取消延迟上界 |
|---|---|---|---|---|
| **C0** | `:166`（`let now = ...`）之后、`:169`（`recover_stale_running`）之前 | `stop.reset()` —— **入轮清一次** | 「进入 turn 时的值」就是「这一轮专属的停止意图」 | — |
| **C1** | `:295`（末步提醒块的 `}`）之后、`:296`（`let hop_started = Instant::now();`）之前 | `break` 出 hop 循环，**不写任何 steps 行** | 「别再发起新的一次模型调用」= **省钱的主要来源** | 一跳（`nt_engine.rs` CLI 引擎缺省 300s，**需复核**：`rg -n "TIMEOUT" crates/neotrix-neobot/src/nt_engine.rs`） |
| **C2** | `:363`（`for call in &turn.tool_calls {`）之后、`:364`（`gate(config, actor, call)?`）之前 | ①写一条 `Deny`/`rule="cancelled"` 审计行；②写**一条** `steps` 行 `tool="cancelled:tool_calls", ok=0`；③`break` 出工具循环 | 「同跳内不再执行下一个工具」= **最重要的一处** | 一个工具（bash 由 C3 压到 50ms；附件下载另见下） |
| **C3** | `:853`（`None => std::thread::sleep(Duration::from_millis(50))`）所在 match 臂内，**sleep 之前** | `child.kill()` + `child.wait()` + 收干两管道 + 返回 `ok=false`、output 追加 `[neobot] stopped by user` | 「已经起了的 bash 进程」= 唯一能到亚秒的地方 | **50ms + kill** |
| **C4** | `:213`（`let status = run_loop(...)`）之后、`:231`（`let finished = AgentTask {`）之前 | 见 §10.4 的取消落库分支 | 收尾落库后返回 | — |

**C2 三个子动作的顺序不可换，理由是 `nt_agent.rs:366-367` 那条律**：
「网关律：先写审计行（无论放行与否），再执行。崩溃也不丢『谁动了什么』的记录」。
若把 C2 放在 `:380`（`record_audit`）**之后**、`execute_tool`（`:384`）之前，
就会造出一条**声称网关已判定、实际什么也没做**的中间态；放在 `gate`（`:364`）之前
则连策略都不问，审计行如实写「用户叫停」而不是「网关拒绝」—— 这是两件不同的事实，
不能混。`AuditEvent::new` 的签名在 `:369-379`（`rule` 形参允许 `Some(String)`），
`AuditDecision` 已在 `:12` import。

**C1 命中后不要自己 `return`**：`break` 之后 `current` 仍是 `Continue`（`:273` 初值），
`:456-458` 的兜底会把它映射成 `Waiting`。**这正是我们要的语义**（人可接手），
不必另写早返回路径 —— 少一条路径就少一处未来要维护的分叉。

**一个检查点覆盖不到的长阻塞（如实记下，不是缺陷清单而是已知边界）**：
入站附件下载 `nt_channel_dispatch.rs:200` 的 `adapter.fetch_attachment`，
最坏 **120s**（`nt_channel_telegram.rs:55` `DOWNLOAD_TIMEOUT`，用在 `:759`）。
它在 **T0 半**（§11.1），不在 hop 循环里，因此 **C0–C3 一个都覆盖不到**。
阶段一的处置：**不假装覆盖**（附件下载无法中断，`ureq` 的 `.timeout()` 是硬超时）；
阶段二的处置：它占住的是 poller 线程，会拖慢收取 —— 这一条要么接受，要么把下载挪进
worker（**需复核**：`rg -n "download_attachments|fetch_attachment" crates/neotrix-neobot/src`）。

### 10.4 中止落库：写哪个 `TurnStatus` 变体（决断 + 理由）

**决断（三件事同时定）**：

1. **不新增 `TurnStatus` 变体。**
2. **task 行写 `TaskStatus::Cancelled`** —— `nt_types.rs:56` 既有变体，
   `as_str()` 在 `:66`（落库字符串 `"cancelled"`），`parse` 在 `:76`。
   既有产出者：`nt_store_tasks.rs:125-133` `cancel_task`（`:128` 写 `status='cancelled'`）。
   既有消费者：`nt_store_tasks.rs:143` `retry_task` 收 `('failed','cancelled')`。
3. **公开签名返回 `Ok(TurnStatus::Waiting)`** —— `nt_types.rs:20` 既有变体，
   `as_str()` = `"waiting"`（`:31`）。给用户看的那句中文由 **dispatch 侧**产出
   （它持有 `StopToken`，知道自己置过位），**不由 `TurnStatus` 承载**。

**为什么不选其它三个（逐一排除）**：

| 诱人做法 | 为什么否 |
|---|---|
| 加 `TurnStatus::Cancelled` | 要动 `nt_types.rs:25`(`as_str`)/`:36`(`parse`)/serde 名、`nt_channel_dispatch.rs:296-304`(`status_text`)、`nt_routine.rs:108` 的 error 串、`nt_channel_dispatch.rs:166` 的 `InboundOutcome::Turn.status`、`bin/neobot.rs:814/967/985` 的 CLI 输出、`nt_cmd_run.rs:175/178` 的 `Done` 事件与结果。**而 task 行已经如实记下了「被取消」，收益为负**（§14 更正了「落库字符串变化会让历史数据读不出来」这条前提 —— 实测 `TurnStatus` 从不被读回） |
| `return Err(NtBotError::Invalid("cancelled"))` | 走 `nt_agent.rs:215-229` 的 Err 分支 → 落 `TaskStatus::Failed` + `error`。**「失败」≠「用户主动叫停」**，且会让 `/status` 与任务列表把用户的动作显示成故障 |
| `return Ok(TurnStatus::Continue)` | 走 `:456-458` → `Waiting` → `:235-237` 映射到 `TaskStatus::Pending`。而 `pending` 在本项目里意味着「可被认领/可重跑」（`nt_store_tasks.rs` 的 `claim_task`/`retry_task`），**没有任何 worker 会来取一个 IM 的 pending 轮次** → 「看起来还能跑、实际没人跑」 |

**为什么 `Waiting` + `TaskStatus::Cancelled` 这个组合不撒谎**：
- `TaskStatus::Cancelled` 是**既有**变体，`cancel_task` 早就写它、`retry_task` 早就收它
  —— 这条路的 80% 已铺好，只是从来没有一条**活着的**轮次产出过它。
- 用户有正规重跑出口（`retry_task` 接受 `cancelled`），「按错了再按一次」不是死路。
- 停掉的那一轮**没有声称完成**：`Waiting` 既有中文回执是
  「已行动，等外部条件。」（`nt_channel_dispatch.rs:301`）—— 用户叫停确实就是
  「等外部条件（你重新说）」。
- `error` 列记下原因与位置（第几跳），信息不丢。

**C4 处的落库分支（照抄即可，注意字段与既有代码同形）**：

```rust
// nt_agent.rs:213 之后 / :231 之前
let status = run_loop(ctx, &task.id, on_delta, on_step);   // 改为 (TurnStatus, Option<usize>)
if let Some(hop) = stopped_at {
    let cancelled = AgentTask {
        status: TaskStatus::Cancelled,
        updated_at: Utc::now().to_rfc3339(),
        lease_id: None,          // ← 必须清，与 nt_store_tasks.rs:128 的 cancel_task 同款
        lease_until: None,       // ← 同上：不清则 recover_stale_running
                                //   (nt_store_tasks.rs:155-163) 十分钟内不碰它
        error: Some(format!("stopped by user at hop {hop}/{steps}")),
        ..task.clone()
    };
    store.save_task(&cancelled)?;
    store.enqueue_outbox(                                  // 与 :250-254 同形
        &Uuid::new_v4().to_string(), "CH_MESSAGE_NEW",
        &serde_json::json!({"task_id": cancelled.id, "status": cancelled.status.as_str()}).to_string(),
    )?;
    return Ok(TurnStatus::Waiting);
}
```

**「哪一跳」怎么从 `run_loop` 传出来（决断）**：`run_loop` 是**私有**函数
（`nt_agent.rs:258`）、**唯一调用点** `:213`，所以把返回类型改成
`(TurnStatus, Option<usize>)` 即可，**不新增共享可变状态**。
若改 `StopToken` 去装一个 `Cell<Option<usize>>`，就等于让一个跨线程共享的东西
带内部可变性 —— 那会逼出 `Mutex` 或 `unsafe`。**两个都不要**。

**`tool_calls` 半途状态怎么表示（决断）**：C2 命中时，本跳的
`turn.tool_calls`（已在 `:349-355` 压进 history）里**尚未执行的那些调用不会有
`steps` 行，也不会有 audit 行**。若就这么 break，库里就留下一批**悬空的
tool_call id**（history 里有、steps 里查无此行）—— 那是最难查的一类脏数据。
**决断：C2 的第二个子动作写一条汇总 steps 行**：
`tool = "cancelled:tool_calls"`、`ok = 0`（即 `false`）、`output` = 本跳剩余
未执行工具名清单（`call.name.as_str()` 逗号连接）。
于是**每个被请求过的工具都有归属**：要么有自己的 allow/deny 审计行 + steps 行
（正常路径），要么出现在这一条 `cancelled:tool_calls` 行里（被取消路径）。
**不留悬空。** 验收测试见切片 C1 的第 2 条。

**半途事务 / 账本怎么标记**：**不引入任何显式事务**（§5-H1-2 已论证：
当前全部是裸 autocommit，引入事务就引入「读事务升级写事务」这个经典死锁源）。
具体地：
- 已发生的**写就是发生了**：`record_ledger`（`:319`）、`add_step`（`:341`/`:358`/`:407`）、
  `record_audit`（`:380`）**一律不回滚**。账本里那跳的 token 用量照记 —— 因为模型**确实跑了**。
- **在途引擎调用（`:297-300`）被取消时不写 ledger**（`turn` 拿不到，无从写）。
  代价：这一跳的花费不落账。**如实接受**：这是「不发明数据」的代价，不是 bug。
  提醒：`nt_cost` 的成本汇总是按 ledger 求和，漏一跳 = 报表略偏低，**方向是保守的**。
- 附件/文件改动**不回滚**（`nt_changes::ChangeSink` 已在 `:277-281` 持有）。
  **这是对的**：一个跑了 30 秒的 bash 改了文件，然后被叫停 —— 说「什么都没发生”
  才是撒谎。「本轮文件」视角（`file_changes` 表）如实显示改了什么。

### 10.5 `stop_flag: Option<&mut bool>` 怎么接进新管线（决断 + 迁移路径）

**决断：原地改造形参为 `Option<&StopToken>`，不保留旧形参、不新增并行的第二个形参。**

理由（按权重）：
1. **旧形参在目标架构下永远不可能工作**：`&mut bool` 借不过线程边界。
   保留它 = 在同一个函数上留两套停止机制，其中一套**按构造就不可能**被用到。
   那不是向后兼容，那是留一个诱使人误以为「改个参数名就行」的坑。
2. **调用点只有 3 处**（实读）：
   - `nt_channel_serve.rs:213-221`（`run_once` 内，`:220` 传 `None`）
   - `apps/neobot-desktop/src/nt_commands/nt_cmd_channels.rs:328-336`（`:335` 传 `None`）
   - `nt_channel_dispatch.rs:729-745` 的测试 `stop_command_sets_the_stop_flag`（`:739` 传 `Some(&mut flag)`）
   加上文件内 8 处测试调用（`:640/643/660/679/694/710/756/775/1064`，全部 `None`）——
   **改一个形参类型的实际代价是 2 个生产调用点 + 1 个测试**。
3. `on_inbound` **没有在 `lib.rs` 的 crate root 被 re-export**
   （实测 `rg "pub use nt_channel" lib.rs` 只有 `:76-77` 的
   `AccessMode/ChannelAdapter/ChannelHealth/ChannelRegistry/InboundMessage/OutboundMessage`）。
   所以它对**工作区外**的消费者是「pub 但不可达」，破坏面=0；
   对**工作区内**只有桌面那一个 crate —— **但它必须跟着改，这是 §6-S2 漏掉的一处**（见 §14）。

**迁移路径（有序，每步单独可编译）**：
1. 新增 `nt_cancel.rs`（`StopToken` + 4 个方法 + 2 个单测）→ 独立提交。
2. `on_inbound` 形参改 `Option<&StopToken>`；`:86-91` 的 `*flag = true` 改为
   `if let Some(token) = stop_token { token.cancel(); }`；**同步改那 3 个调用点**
   （两个 `None` 不变，测试改成 `Some(&StopToken::new())` 并断言 `is_cancelled()`）。
   → 行为**完全不变**（今天没人传 `Some`），是纯签名迁移。
3. `run_local_turn_*` 四个 wrapper **签名不动**（`nt_agent.rs:45/68/92/114`），
   内部 `run_local_turn_inner(&ctx, None, None)` **多传一个 `None`**
   （`nt_agent.rs:158-162` 增第 4 参）。`nt_routine.rs:94` 与
   `bin/neobot.rs:801/810` **零改动** —— 这是「wrapper 不动」的直接收益。
4. 只有 dispatcher 与桌面需要「有 token 的那一版」→ 新增**一个**内部函数
   `run_local_turn_as_cancellable(..., stop: Option<&StopToken>)`，
   4 个 wrapper 全部转调它。**不新增第 5 个公开 wrapper 的理由**：桌面 IPC 与
   dispatcher 是**两个不同的调用方**，它们各自传自己的 token 即可，
   公开面不因实现手段而增长。
5. 插 C0–C4。此时 `channel serve` 行为**仍不变**（没人传 `Some`），
   **但取消机制已经真实存在且被测试覆盖**。

### 10.6 阶段一（`nt_agent` 内部）的实测改动面

| 文件 | 改什么 | 行数级 |
|---|---|---|
| `crates/neotrix-neobot/src/nt_cancel.rs` | **新文件**：`StopToken` + 4 方法 + 单测 | ~60 |
| `crates/neotrix-neobot/src/lib.rs` | `pub mod nt_cancel;`（按字母序插在 `:22` 起的 mod 块里） | 1 |
| `crates/neotrix-neobot/src/nt_agent.rs` | `run_local_turn_inner` 增 1 参（`:158`）；`run_loop` 返回类型改元组（`:258`）；调用点 `:213`；C0 `:166` 附近；C1 `:295/:296` 之间；C2 `:363/:364` 之间；C3 `:853` 臂内；C4 `:213`/`:231` 之间；`execute_bash` 签名 `:798`；`execute_tool` 签名 `:633`；其调用 `:384` 与 `:647` | ~70 |
| `crates/neotrix-neobot/src/nt_agent.rs`（测试） | `use super::execute_bash`（`:1078`）与 `execute_bash(...)`（`:1105`）；`super::execute_tool` 的 **6 处**（`:1415/1437/1472/1483/1496/1512`）—— 签名一改全部要跟 | ~14 |
| `crates/neotrix-neobot/src/nt_channel_dispatch.rs` | 形参 `:58`；置位 `:86-91`；测试 `:729-745` | ~8 |
| `crates/neotrix-neobot/src/nt_channel_serve.rs` | 调用点 `:220`（仍 `None`） | 0 |

**这张表纠正了 §6-S1「约 40 行」的说法**：真实量级是 **~150 行（含 7 处测试调用点跟改）**。
差的不是量级而是**触点**——「4 个公开 wrapper 零改动」成立，但
**两个私有执行器函数各有测试在直接调**，签名一改就波及 7 个测试点。
把这条写清楚，是为了让派活的人按 7 个点派，不是按 1 个点派。

### 10.7 复算命令（本节行号）

```sh
rg -n "pub fn run_local_turn" crates/neotrix-neobot/src/nt_agent.rs      # 4 wrapper，应为 45/68/92/114
rg -n "fn run_local_turn_inner|fn run_loop|for n in 0\.\.steps|for call in &turn\.tool_calls" crates/neotrix-neobot/src/nt_agent.rs
rg -n "let hop_started|engine\.run_turn_stream|engine\.run_turn_with_history|record_audit|fn execute_tool|fn execute_bash|try_wait|from_millis\(50\)" crates/neotrix-neobot/src/nt_agent.rs
rg -n "stop_flag|stop_requested" crates/neotrix-neobot/src/nt_channel_dispatch.rs crates/neotrix-neobot/src/nt_channel_cmd.rs
rg -n "on_inbound\(" crates/neotrix-neobot/src apps/neobot-desktop/src   # 应含桌面 nt_cmd_channels.rs 一处
rg -n "recover_stale_running|status='cancelled'" crates/neotrix-neobot/src/nt_store/nt_store_tasks.rs
rg -n "pub struct HttpEngine" -A8 crates/neotrix-neobot/src/nt_http_engine.rs   # 验 Send 字段
```

---

## §11 阶段二：poller 单线程 + 有界 worker 池

### 11.1 职责边界（切在哪一刀）

`on_inbound`（`nt_channel_dispatch.rs:51-168`）**切成两半**，分界线落在
`:133`（`user_text` 拼好）之后、`:135-138`（跑轮）之前：

| 半 | 步骤（现行行号） | 需要什么 | 归属 |
|---|---|---|---|
| **T0 半** | 1) 去重 `mark_seen`（`:62-68`）2) 访问闸门（`:70-80`）3) 斜杠指令 + 两次 `adapter.send`（`:84-103`，`:93` 与 `:153`）4) 定/建会话（`:106-119`）4.5) 附件下载（`:123`，最坏 120s）5) 拼正文（`:126-133`） | **适配器**（`send`/`fetch_attachment`） | poller 线程 |
| **worker 半** | 6) `run_local_turn_as`（`:138-147`）7) 结果回渠道（`:151`、`:159-163`） | **引擎** + store | worker 线程 |

**分界线的判据只有一条：谁拥有适配器。** `ChannelAdapter`（`nt_channel.rs:139`）的
`poll` 要 `&mut self`（`:155`，长轮询推进 offset）、`send`/`fetch_attachment` 要 `&self`，
而 trait **既无 `Send` 也无 `Sync` 约束**（`:139` 无 supertrait），
`ChannelRegistry` 持 `BTreeMap<String, Box<dyn ChannelAdapter>>`（`:194-196`）。
**所以：凡是碰适配器的代码必须在 T0。** 这条判据不需要协商，也不会被绕过。

**两处直连 `adapter.send` 必须改走出站**：`:93`（指令回执）与 `:153`（轮结果）
现在都是直连。worker 拿不到适配器，所以它们**必须**进 outbox
（`enqueue_outbound_with_attachments`，`nt_channel_dispatch.rs:427-444`）。
**顺带修一个既有半接线**：该函数的 payload（`:435-441`）只有
`channel/chat/text/attachments`，**没有 `edit_of`**；而 `:97` / `:155` 现在显式传
`edit_of: None`，`:441` 附近的 `deliver_result` 与 `sweep_pending`（`:496`）却在**填** `Some`。
改走出站时若不把 `edit_of` 放进 payload，**「编辑原消息」这个特性会静默退化成重复两条**
（§0.5「仍然成立」第 3 条）。

### 11.2 串行化粒度：**按 `conversation_id`**（决断）

**决断：in-flight 键 = `convo_id`，且这张表就是 §10.2 第 4 环那张表。**

| 候选粒度 | 判断 |
|---|---|
| **按渠道**（channel） | **否**。这正是要去掉的队头阻塞 —— 一个渠道挂 25s 长轮询 + 一个 300s 跑轮，粒度按渠道等于什么都没改 |
| **按 bot** | **否**。一个 bot 服务多个 chat（§5-H2 已记下 `bot.conversation_id` 只有一个槽位、`msg.chat` 从不参与选会话）；按 bot 串 = 按渠道串的加强版，更糟 |
| **按 `conversation_id`** | **是**。① 正确性风险就长在这个粒度上：`on_inbound:159-160` 用 `list_convo_tasks(&convo_id, 1)` 反查本轮 task（`ORDER BY created_at DESC LIMIT 1`，`nt_store_routines.rs:334` 需复核），两个并发轮落在同一会话会互相冒领 task，**A 把 B 的回复发回 A 的消息** —— 最好的情况是两条回复且内容与问题不对应，且**不报错**；② 用户可感知的单位就是「一个聊天」；③ 与停止信号同键 ⇒ §10.2 的注册表零额外成本；④ 同会话的入队序天然等于轮次序（IM 语义要求） |

**保证机制**：`Arc<Mutex<HashMap<String, Arc<StopToken>>>>`（同 §10.2 第 4 环）。
- 线性化点 = `HashMap::insert(convo_id, token)` 返回 `Some(旧值)`（即已在飞）。
- 已在飞 → **压回该 convo 的 per-convo FIFO 尾**，绝不并发。
- 释放用 **RAII guard 的 `Drop`**（`remove(convo_id)`）。理由：`run_loop` 里有十几处
  `?` 早返回（`:319/:341/:358/:380/:407` 等），靠人记得删必然漏一次；漏一次的后果是
  **该会话永久卡死**（再也不会有新轮次进去），而这种卡死**不报错、只表现为「机器人不回了」**。
- **per-convo FIFO，不是全局队列**。全局队列 + 忙则重排会乱序，而乱序会直接显示成乱序
  （`list_convo_tasks` 的 `created_at DESC`）。

### 11.3 worker 池大小：`min(4, available_parallelism())`，且**可配**

**决断**：`N = min(4, std::thread::available_parallelism())`，可用
`NEOBOT_CHANNEL_WORKERS` 环境变量覆盖（`0` 或 `1` = 单 worker 退化模式）。

理由：
- **写竞争随并发写者数近似线性**（§5-H1：一轮里的写是每跳一次 `record_ledger` `:319`、
  每工具一次 `record_audit` `:380` + 一次 `add_step` `:407`、终态两次 `:249/:250`；
  全部裸 autocommit + `?` 传播，任何一处 `database is locked` 都**掀翻整轮**）。
  而收益在「一个慢轮不堵别人」处就饱和了。
- 每个 worker 一条自己的 `NeobotStore::open`（**不动任何 store 方法签名** ——
  `NeobotStore{conn: Connection}`，`nt_store/mod.rs:207-209`；`rusqlite::Connection`
  是 `Send` 但 `!Sync`，所以「每 worker 一条」既是并发化的前提，也是绕开 `!Sync` 的正解）。
- **N 是旋钮不是开关**：`N=1` 时仍然有**两个**执行流（poller + 1 worker），
  于是 `/stop` 可达（§10.0 的死结在 N=1 时**已经解开**）、出站不再被 300s 的跑轮拖住。
  **这意味着 `/stop` 可以在 N=1 下先上线**，把并发度当后续调优 —— 风险分级的关键一刀。

### 11.4 队列满了怎么办：**不丢消息 —— 反压在 `poll()`，不投在内存**

**决断：per-convo FIFO 在内存里无界增长；背压由「poller 停止 `poll()`」实现。
既不丢消息，也不落库续做。**

理由（逐条排除另两种）：
- **丢** —— 直接违反派题硬要求，且 `mark_seen` 已经写过就等于**永久静默丢弃**（§5-H6）。
- **把队列落库**（新建 `work_queue` 表 + 一个消费者）—— 需要新表 + 新扫描 + 新崩溃恢复语义，
  换来的是一个**本来就不需要持久化**的东西：offset 是**内存态**
  （`nt_channel_telegram.rs` 每次 `new` 从 0 起步），所以进程一停，
  重启后**整段历史本来就会重拉**，靠 `channel_seen` 的 `INSERT OR IGNORE`
  （`nt_store_channels.rs:201-208`，`rows_affected > 0` 判定，**单条 INSERT 在 SQLite 里是原子的**）
  去重。**「落库续做」是在给一个已经由 offset 语义解决掉的问题建第二套机制。**
- **不 `poll()`**（采纳）—— 队列长度 ≥ 高水位（建议 64）时，**本轮跳过 `poll()`**、睡一个短间隔。
  因为 **offset 只在 `poll()` 返回时前进**（`nt_channel_telegram.rs:209-213` 需复核），
  **不调 `poll()` 就一条都不消费**，平台会重投 —— **零丢失，且不需要任何新机制**。
  这是真正的背压：上游被下游的消化速度钳住，而不是内存被撑爆。

**配套的一处顺序调整（必做，否则关机丢消息）**：
`mark_seen` 现在在 `on_inbound:62`（**跑轮之前**）。若它在 T0 半、消息入队后进程被 Ctrl-C，
那条消息已「见过」但永不跑 → 重启后重拉被判重复 → **静默丢弃**（§5-H6）。
**决断：把 `mark_seen` 移到 worker 半（跑轮之前）**。安全性：per-convo 串行 +
`INSERT OR IGNORE` 原子 ⇒ 两个 worker 同时对同一 key 置位时，后者拿到 `false`
→ `InboundOutcome::Duplicate` → 丢的是**重复件**，不是原件。
**代价**：`nt_channel_dispatch.rs` 现有 28 个测试里与去重时序有关的必须逐个复核
（`:640-643` 的「第二条被判 Duplicate」**应当照常通过**，因为它第二次调用仍会拿到 `false`）。

### 11.5 共享 offset 的竞态：消解 = **poller 独占注册表，且靠构造而非靠纪律**

**决断（两条，都不需要新机制）**：
1. **注册表永远是 poller 线程的局部变量** —— 与今天一样（`nt_channel_serve.rs:149`
   `let mut reg = registry();` 在 `run_once` 栈上）。**不 `Arc`、不 `Mutex`、不 `Box` 到别处。**
   于是「两个线程同时 `poll()` 同一个 adapter」在**类型层面**就不可能：
   没有任何跨线程的 `&mut ChannelRegistry`。
2. **不要为了「让两个渠道并行」给每渠道各建一份 adapter** —— 每份都从 `offset=0`
   起步（`nt_channel_telegram.rs:95` 需复核），各自重拉保留期全量，
   N 个渠道 = N 倍拉取量。**单 poller 是特性，不是限制**（§5-H3-1 的判断，本节复核后仍成立）。

**明确的反模式（写下来是因为它很诱人）**：把注册表包进
`Arc<Mutex<ChannelRegistry>>`「让它能跨线程」—— 那是**把 §11.5 要防的 bug 直接造出来**，
而且 `Mutex` 只提供互斥，不提供「只有一个人 poll」。

**验收**：`poll_happens_on_exactly_one_thread` —— 计数适配器断言任意时刻
`poll` 的并发度恰为 1（切片 C4）。

### 11.6 关机语义：**与 §10.4 同一套落库代码，不允许第二套**

**决断：关机 = 一次「范围更大的 stop」。** 具体：
1. T0 收到停止意图 → 置 `shutdown = true` → **不再 `poll()`**（§11.4 的同一条反压路径）。
2. T0 **遍历 in-flight 表，对每个 `StopToken` 调 `cancel()`**。
3. worker 在 **C1/C2/C3 原有的同四个检查点**观察到取消 → 走 **§10.4 的同一条落库分支**。
4. T0 `join` worker，**带 90s 上限**；超时则**照样退出**（残留由
   `recover_stale_running` 兜，`nt_store_tasks.rs:155-163`）。
5. 队列里未跑的消息**直接丢弃内存副本** —— 因为 §11.4 已把 `mark_seen` 移到 worker 半，
   它们**未被标记见过**，重启后重拉可见。**零回滚逻辑。**

**为什么必须是同一套**：两套停止行为必然产生两种 task 行 ——
一种 `cancelled`、一种 `failed` 或 `running`。而「这一轮到底怎么了」是**只有一个答案**的事实。
**验收判据（写死）**：在飞轮次在关机后，库里**不得存在** `status='running'` 且
`lease_until` 未过期的行；若存在，就是走了错误的分支。

**硬依赖（顺序约束，不是建议）**：第 4 步的兜底依赖
`recover_stale_running` 的回收条件 `lease_until < now`（`nt_store_tasks.rs:159`），
而**今天的代码从不续租**：`LEASE_SECS = 600`（`nt_agent.rs:24`）只在 `:173-174` 写一次、
`:240-241` 清一次，实测 `rg -n "lease_until" nt_agent.rs` 无任何续租写。
于是「90s 超时后残留的行」要挂满 **10 分钟**。
**决断：租约续租（每 hop 之后把 `lease_until` 推后）必须在 worker 池之前落地**（切片 C0）。
这与 §5-H4-3 是同一件事，本节把它从「独立改进」升级为「池子的前置件」。

**新增依赖的诚实申报**：第 1 步需要信号处理器。本 crate `#![forbid(unsafe_code)]`
（`lib.rs:20`），不能自己写 `libc::signal`，得引 `ctrlc` 或 `signal-hook`。
**这是并发化里唯一的新依赖，且它是行为破坏性的**（见 §12 第 5 项：Ctrl-C 从
「立刻杀进程」变成「最多 90s 才退」）。

---

## §12 不可逆 / 破坏兼容的改动与爆炸半径

按「爆炸半径」从大到小。**每项都给了回滚。**

| # | 改动 | 破坏性判定 | 迁移 / 回滚 |
|---|---|---|---|
| **1** | **引信号处理器依赖（`ctrlc`/`signal-hook`）** | **行为破坏（唯一一项真正不可逆的）**。今天 Ctrl-C = OS 默认 SIGINT = 进程立刻消失；引了之后 = 优雅退出，最长 90s。用户的 `Ctrl-C && neobot channel serve` 之类脚本会开始「按了没动」 | 缓解：①首次收到 SIGINT 时打印「正在收尾，第二次 Ctrl-C 立即退出」；②第二次 SIGINT 走 `std::process::exit(130)`（**不经 handler 路径**）；③**默认值做成可关**（`NEOBOT_GRACEFUL_SHUTDOWN=0` 时恢复「立刻死」，行为与今天逐字节相同）。**依赖本身可从 `Cargo.toml` 删除即回滚**，但用户已习惯的行为不会自己回去 |
| **2** | **`turn_stops` 新表（跨进程停止意图）** | **不破坏**。`migrate()` 全是 `CREATE TABLE IF NOT EXISTS`（`nt_store/mod.rs:245` 起，实测 17 条全为此形），**幂等追加**，旧库新开自动建 | 回滚 = `DROP TABLE turn_stops`。**但注意**：不建表时「跨进程停止」这个能力就没有，其余功能不受影响 |
| **3** | **`on_inbound` 形参 `Option<&mut bool>` → `Option<&StopToken>`** | **源码级破坏，语义零破坏**。`on_inbound` 未在 `lib.rs` crate root re-export（实测只有 `:76-77`），工作区外不可达；工作区内**只有桌面一处**（`nt_cmd_channels.rs:328`），且它今天传 `None` | 回滚 = 改回形参（该次提交内无其它耦合） |
| **4** | **`TaskStatus::Cancelled` 由「跑轮」新产出** | **不破坏**。`'cancelled'` 早就是库里的合法取值（`nt_types.rs:66/:76`；`cancel_task` 早已产出，`nt_store_tasks.rs:128`；`retry_task` 早已收，`:143`）。**唯一需查的**是「假设 `list_tasks` 只含 pending/running/done/failed」的读方 —— 实测 `nt_stale_guard.rs:59` 过滤 `== Running`（安全），前端 hero 取「第一条 running」（安全） | 若发现某读方对未知状态 `panic`/`unwrap`，那才是真破坏点 —— 落地前用切片 C1 的第 3 条测试扫一遍 |
| **5** | **`mark_seen` 从 poller 半移到 worker 半** | **不破坏 schema，破坏时序**。去重判据（`INSERT OR IGNORE` + `rows_affected`，`nt_store_channels.rs:201-208`）完全不变；变的只是「什么时候问」 | 回滚 = 移回去（但会把 §11.4 的关机丢消息洞重新打开）。**必须与 `nt_channel_dispatch.rs` 的 28 个测试一起复核** |
| **6** | **outbox payload 新增 `edit_of` 键** | **向后兼容**。`drain_outbox_once` 对缺键是宽容的（`payload_attachments` 在 `nt_channel_dispatch.rs:311-325` 明确「缺字段当空」）；只有 `channel` 是**必需**（`:345-348` 缺了就 `fail_outbox` 退避）。**加可选键安全，改必需键不安全** | 若误把 `edit_of` 设成必需 → 全部历史 payload 被退避重发，**队列雪崩**。加键时必须同步改 `:311` 附近的宽容解析 |
| **7** | **每 worker 一条 `NeobotStore::open`** | **不破坏**，但新增一个运行时事实：同进程 N+1 条连接同时开在一个 WAL 库上。`open` 已设 5s busy timeout + WAL（`nt_store/mod.rs:213-224`），**WAL 仍只允许一个写者** | 回滚 = 退回单连接（那就退回单执行流，等于放弃阶段二） |
| **8** | **`engine_for` 返回 `Box<dyn EngineAdapter + Send>`** | **不破坏**。实测 4 个引擎的字段全是 `String`/`Duration`/`bool`/`Option<String>`/无（`nt_engine.rs:78` `LocalEchoEngine` 无字段、`:107-110` `CliEngine`、`:297-300` `OpencodeEngine`、`:83-90` `HttpEngine`）⇒ 都是 `Send`。**更正 §5-H5 把这一项写成「可能撞上真正障碍」—— 实测它是四个结构体、零障碍** | 若将来某个引擎持有非 `Send` 字段，`+ Send` 会当场编译失败（**fail-fast，不是运行时事故**）—— 这正是想要的行为 |
| **9** | **`run_loop` 返回类型改元组、`execute_tool`/`execute_bash` 增参** | **零外部破坏**（三者皆私有），但**波及 7 个测试调用点**（`nt_agent.rs:1078/1105` + `:1415/1437/1472/1483/1496/1512`） | 无需回滚方案；派活时按 7 个点派（§10.6） |

**明确**不在此清单里的（因为已决定不做）：
**`TurnStatus` 新增变体** —— §14 更正了它的爆炸半径：实测 `TurnStatus::parse` 全仓
**只有两处调用**（`nt_agent.rs:611` 解析模型 `set_turn_status` 的参数、
`nt_types.rs:294-305` 自环往返测试），`TurnStatus` 字符串只**写进** `ledger.status`
（`nt_agent.rs:330`）而**从不被读回**（`ledger_sums` 只聚合成本）。所以加变体
**不会让任何历史数据读不出来**，它破坏的只是 3–4 个**展示**点。
这是个**比派题假设小一个数量级**的风险 —— 也正因如此，**新增变体仍然不值得**：
收益（task 行本来就说清了）远小于要动的跨进程展示契约。

---

## §13 分阶段可交付切片（给后续 agent 派活用）

> 每片**单独可编译、单独有测试、单独可上线**。
> 与 §6 的 S0–S4 的关系：C0≈S0 部分、C1≈S1、C2≈S1 续、C3≈S3、
> **C4 = §6-S2 拆出来的第一步（只切 `on_inbound`，不建池）**、C5 = S2 全量。
> **C0–C3 完全不碰并发**，可以在 `/stop` 仍不可达的情况下独立上线。

### C0 — 续租心跳 + `StopToken` 骨架（**池子的前置件**）
- **文件**：`nt_agent.rs`（每 hop 之后加一次 `lease_until` 推后）、**新** `nt_cancel.rs`、`lib.rs`（+1 行 mod）。
- **验收测试**：
  1. `stop_token_starts_false_and_flips_once` —— `is_cancelled()` 初值 false、`cancel()` 后 true、`reset()` 后 false。
  2. `lease_is_renewed_across_hops` —— 用现成范本 `LoopForever`（`nt_agent.rs:1261` 结构 / `:1265` impl / `:1324` 使用），断言 hop 1 之后 `lease_until` 已推后。
- **上线判据**：不改变任何对外行为；崩溃恢复窗口从「10 分钟」降到「600s + 心跳间隔」。

### C1 — 取消钩子进 `nt_agent`（C0–C4 全部检查点，**但仍无人置位**）
- **文件**：`nt_agent.rs`（§10.6 全表）、**新** `nt_cancel.rs` 的 `is_cancelled` 接线。
- **验收测试**（每条都是**新增断言**，不是「已有测试顺便还过」）：
  1. `cancel_stops_before_next_hop` —— hop 0 置位 ⇒ 引擎的 hop 1 调用次数 **== 0**。
  2. `cancel_writes_deny_audit_and_no_allow_row_for_unexecuted_tool` —— 断言 `audit` 里有 `decision='deny' AND rule='cancelled'`，**且**该工具的 `allow` 审计行**不存在**，**且**存在一条 `tool='cancelled:tool_calls'` 的 steps 行。**这三条合起来钉住「不留悬空 + 不造假账」。**
  3. `cancelled_task_row_is_truthful` —— `status='cancelled'`、`lease_id IS NULL`、`lease_until IS NULL`、`error` 含 `at hop 1/3`，且 `retry_task` 之后变回 `pending`。
  4. `stop_flag_is_cleared_at_turn_entry` —— 连续两轮同一 convo：第一轮被停、第二轮**必须正常跑完**（钉 C0 的「入口清」而不是「用完清」）。
  5. `cancel_does_not_cross_conversations` —— A 会话置位，B 会话的一轮不受影响。
  6. `bash_child_is_killed_within_one_poll_interval` —— 一个 `sleep 30` 的 bash，置位后断言在 **< 1s** 内返回且 `ok=false`（钉 C3）。
  7. **回归**：`nt_agent.rs` 现有 11 个测试**全部原样通过**（其中 7 个调用点随签名更新，但**断言内容一个字不改**）。
- **上线判据**：`channel serve` 行为**不变**；`nt_channel_cmd.rs:181-185` 的
  `/stop` 回执**一个字都不许改**。加一条反向测试
  `stop_receipt_still_admits_the_limitation`（照 `nt_channel_cmd.rs:292-304` 现成范本改写），
  **把「不能提前说已停」钉成契约** —— 这条是本切片最重要的护栏。

### C2 — 停止信号接线（L1：进程内）+ `on_inbound` 形参迁移
- **文件**：`nt_channel_dispatch.rs`（`:58`/`:86-91`/测试 `:729-745`）、`nt_channel_serve.rs:220`、
  **`apps/neobot-desktop/src/nt_commands/nt_cmd_channels.rs:328-336`（跨 crate，别漏）**。
- **验收测试**：
  1. `stop_command_cancels_the_inflight_token` —— 登记表里先放一个 token，`/stop` 后断言 `is_cancelled()`。
  2. `stop_with_no_inflight_turn_says_so_honestly` —— 登记表空时 `/stop`，断言回执**不含**「已停」类断言词（沿用 `nt_channel_cmd.rs:292-304` 的反向断言范式）。
  3. **回归**：`nt_channel_dispatch.rs` 现有 **28** 个测试全部原样通过。

### C3 — 桌面停止键变成真停止（L2 跨进程表 + 一次 IPC）
- **文件**：**新** store 表 `turn_stops` + `mark_stop`/`take_stop`（落 `nt_store_channels.rs`，
  留存清扫落 `nt_store_upkeep.rs`）、`nt_agent.rs` 的 L2 读点（入轮处，语义同 C0）、
  **`apps/neobot-desktop/src/nt_commands/nt_cmd_run.rs:143` 一带**、**新** IPC `neobot_cancel_turn`、
  **`apps/neobot-desktop/frontend/src/main.ts:573`** 补一次 `invoke`。
- **验收测试**：
  1. `stop_key_actually_stops_the_turn` —— 发一轮长轮 → 调 cancel → 断言该轮在有限步内以 `cancelled` 落库，**且之后不再有新 hop**。
  2. `stop_intent_is_consumed_exactly_once` —— 连续两次跑同一 convo，第二轮不被误杀（钉 `DELETE … RETURNING` 的取即删语义）。
  3. `stop_table_pruned_by_upkeep` —— 造一条 8 天前的行，断言 `upkeep_best_effort`（`nt_channel_dispatch.rs:530`）后消失。
  4. **C1 的 7 条 + C2 的 3 条 + `nt_agent` 11 + `nt_channel_dispatch` 28 全部仍绿。**
- **上线判据**：**这是 `/stop` 与桌面停止键第一次真正工作**。
  同片把 `nt_channel_cmd.rs:181-185` 的回执改成「已停（停在第 N 跳）」，
  并把 C1 的反向测试反向（改为**禁止**出现「停不了/同步/桌面 App」三词）。
  **两处文案必须同一个提交里改** —— 这就是本仓自己的教训（§8-4：假建议被测试钉成契约）。

### C4 — 切分 `on_inbound`（T0 半 / worker 半），**先不建池**
- **文件**：`nt_channel_dispatch.rs`（`:51-168` 切分；`:93`/`:153` 两处 `adapter.send` 改走出站；
  `:435-441` payload 补 `edit_of`；`:311-325` 宽容解析同步）、`nt_channel_serve.rs`（`:144-236` `run_once` 改双入口）。
- **验收测试**：
  1. `outbound_edit_of_survives_the_queue` —— 断言 `edit_of` 走完 `WorkItem → outbox.payload → OutboundMessage` **三段都在**（钉住「不静默退化成重复两条」）。
  2. `outbound_reply_goes_through_outbox` —— 轮结果与指令回执都**不在** `adapter.send` 里直发。
  3. **回归**：`nt_channel_dispatch.rs` 28 + `nt_channel_serve.rs` 16 全部原样通过。
     **这是切分正确性的主要证据**（切分不许改语义）。
- **上线判据**：功能与今天等价（仍是单执行流），但**出站不再被跑轮时长拖住**。

### C5 — poller + 有界 worker 池（= §6-S2 全量）
- **文件**：**新** `nt_channel_pool.rs`（或 `nt_channel_serve` 内）、`nt_channel_serve.rs`（`registry` 搬进 poller 闭包）、
  `bin/neobot.rs:572-603` `cmd_channel_serve` 起线程；`Cargo.toml`（+`ctrlc`，§12 第 1 项）。
- **验收测试**（全部用**临时文件库** —— `:memory:` 每次 `open` 都是独立库，跨线程测不了；§5-H7）：
  1. `one_slow_turn_does_not_block_another_conversation` —— A 睡 2s、B echo，断言 B 在 A 结束**之前**已 `done`。**这一条是 C5 存在的全部理由。**
  2. `same_conversation_never_runs_two_turns_at_once` —— `Barrier` 让同 convo 两个 `WorkItem` 同时就位，断言 in-flight 表任一时刻只含 1 个 id，`steps` 行数 == 1 轮。
  3. `incoming_order_is_preserved_within_a_conversation` —— 同 convo 三条，断言 `list_convo_tasks` 的 `created_at` 序 == 入队序。
  4. `poll_happens_on_exactly_one_thread` —— 计数适配器，断言 `poll` 并发度恰为 1。
  5. `high_water_stops_polling_instead_of_dropping` —— 灌满队列后断言 `poll` 不再被调用，**且没有一条消息被丢**（松压后全部跑掉）。
  6. `queued_but_unrun_survives_restart` —— 关机后重开，断言未跑的消息**重拉后仍可见**（钉 §11.4 的 `mark_seen` 后移）。
  7. `shutdown_leaves_no_running_row_with_live_lease` —— 关机后断言**不存在** `status='running' AND lease_until > now` 的行（钉 §11.6 的「只有一套落库」）。
  8. `second_ctrl_c_exits_immediately` —— 第二次停止立刻退出（§12 第 1 项的缓解）。
- **上线判据**：`/stop` 在 `channel serve` 里第一次**真正工作**；
  `nt_channel_cmd.rs:181-185` 的回执**第三次**改写，与 C3 一起保持「说得做得到」。

### C6（可选，独立评估）— 会话归属缺口
`convo_id` 从「按 bot 一个槽位」（`nt_channel_dispatch.rs:106-119`）改成
「按 `(channel, bot_id, chat)` 查/建」，或如实改文档承认当前是按 bot 聚合（§5-H2）。
**必须独立成片**：它改的是数据归属语义，混进 C5 会让 C5 的回归判据失效。

---

## §14 本节更正的事实前提（写作时实测，与 §0–§9 有出入）

1. **`nt_types.rs:12` 指向一个不存在的模块。** 该行说
   「终态要与 `nt_store/nt_store_turns.rs` 的落库字符串 1:1」，但
   `crates/neotrix-neobot/src/nt_store/` 下**没有 `nt_store_turns.rs`**
   （实测该目录 11 个文件，无此名），且 `migrate()`（`nt_store/mod.rs:245` 起）
   **没有 `turns` 表**（17 条 `CREATE TABLE IF NOT EXISTS` 全表列出，无 `turns`）。
   **实际落点是 `tasks.status`（`TaskStatus`，非 `TurnStatus`）**。
   这是 §8 那份「注释在说谎」清单里**漏掉的一条**（§8 收尾在 #7），
   建议追加为 **#8**：严重度中 — 读注释会以为存在一张按 turn 落的表，
   从而把取消态设计到一张不存在的表上。

2. **派题风险项「`TurnStatus` 落库字符串变化会让历史数据读不出来」不成立。**
   实测 `TurnStatus::parse` 全仓**只有两处**调用：`nt_agent.rs:611`
   （解析模型 `set_turn_status` 工具参数）与 `nt_types.rs:294-305`（自环往返测试）。
   `TurnStatus` 的字符串只**写进** `ledger.status`（`nt_agent.rs:330`），
   而 `ledger` 的读路径（`nt_store_ledger.rs`）只做成本求和，**从不解 `status` 回来比对**。
   ⇒ 新增变体**不会**让历史数据读不出来。**但结论不变：仍然不新增**（§12 已给理由）——
   只是拒绝的理由从「会破坏兼容」换成「要动 6 个展示点而收益为零」。

3. **§5-H5 把「`EngineAdapter` 无 `Send`」列为「三个真正的 Send 障碍」之一，
   实测它是零障碍。** 4 个引擎的字段全是 `String`/`Duration`/`bool`/`Option<String>`，
   `LocalEchoEngine`（`nt_engine.rs:78`）更是无字段单元结构
   ⇒ 加 `Box<dyn EngineAdapter + Send>` 不会撞任何东西。
   **这不改变 §2(c) 放弃 async 的结论**（理由是 store 与 HTTP 栈，不是这一项），
   但它把 C5 的一处工作量从「可能需要重构引擎」降为「一行 trait bound」。

4. **§6-S2 的文件清单漏了桌面 crate。** `on_inbound` 有一个工作区内的第二个调用方：
   `apps/neobot-desktop/src/nt_commands/nt_cmd_channels.rs:328`（`:335` 传 `None`）。
   §0.3 与 §6 都只提了 `nt_channel_serve.rs:143-151` 一处。
   ⇒ **改 `on_inbound` 签名 / 切分 `on_inbound` 都是跨 crate 改动**（切片 C2 / C4 的文件清单已补上）。

5. **§6-S1「约 40 行」低估了触点。** `execute_tool`（`nt_agent.rs:633`）与
   `execute_bash`（`:798`）虽私有，但**各有测试直接调用**：
   `super::execute_tool` **6 处**（`:1415/1437/1472/1483/1496/1512`）、
   `use super::execute_bash` + `execute_bash(...)`（`:1078`/`:1105`）。
   签名一改就是 **7 个测试点**要跟。真实量级 **~150 行**（§10.6 已列表）。

6. **`§5-H2` 的「并发化前应先决定会话归属」仍然成立，且被 C4 放大。**
   切分 `on_inbound` 时，T0 半的「定/建会话」（`nt_channel_dispatch.rs:106-119`）
   必须原样搬过去，**不得顺手改**；否则 C4 的回归判据（28 个测试原样过）就失去意义。
   这一条已写进 C6 的独立性要求。

---

## 附-未做（本文追加部分）

- **追加部分零代码改动、零 `cargo` 命令**（写作期间另一扇窗口在跑 `cargo check`）。
- **行号快照 12:26–12:31**；§10.7 给了复算命令。**下刀前必须重读目标文件**。
- **C3 的 L2 跨进程表只设计了 schema 与取即删语义，未设计**：`turn_stops` 的
  `convo_id` 键在桌面 App 与 `channel serve` **两侧的 convo id 是否同一套 id 空间**
  （桌面传 `convo_id: Option<String>`，`nt_cmd_run.rs:137`；IM 侧是 `bot.conversation_id`，
  `nt_channel_dispatch.rs:106`）—— **这两条 id 是否可比，未核**。若不可比，
  C3 的跨进程停止会**静默不生效**。落地前必须先核这一条：
  `rg -n "convo_id|conversation_id" apps/neobot-desktop/src/nt_commands/nt_cmd_run.rs crates/neotrix-neobot/src/nt_channel_dispatch.rs`。
- **C5 的 90s join 上限、`NEOBOT_CHANNEL_WORKERS`、队列高水位 64** 三个数字
  **都是拍的**，无实测依据（§7 判据 5/6 要求先量）。落地前应先按 §7-5 打一周的
  「本轮收到 N 条、耗时 T」日志。
- **未设计** routine 轮次（`nt_routine.rs:94`）的停止来源 —— 用户没有停止键可按。
  它们会共用 C0–C4 的检查点，但 `stop` 恒为 `None`，行为与今天一致。
- **未设计** per-chat 的出站顺序保证（既有限制，`nt_store_ledger.rs:117` 需复核），
  与 §6「明确不做」一致。
