# DSH 侧边栏 + dsh-im 吸收 · 正典记录

> 吸收 `DSH-better-sidebar` + `dsh-im` 两个 TS 源，落地 `crates/neotrix-neobot`：
> **12 个新模块 / 5 张新表**、`apps/neobot-desktop` **97 个注册 IPC**（53→97）、
> 前端 4 个零依赖文件。以上数字为 2026-09-28 当时值；现况见 §6。

## 台账归属

12 个新模块**不进** `ARCHITECTURE-MAP-ROADMAP-V2.md`。理由：那条规则是 R-P199，
文本只在 `docs/standards/archive/` 的非规范副本里（`dev-rules-legacy-R-P161-257.md:259`，
"R-P161-257" 是归档文件编号区间不是规则号），口径是 `neotrix-core` 的 L1–L6；
`neotrix-neobot` 是独立 crate、不占 L 层。`crates/neotrix-neobot` 同理不进 L 层台账。

## 收口复核（2026-09-28 · 4 agent 并行产出 + 主 agent 独立核验）

验证（`CARGO_BUILD_JOBS=2` 串行，内存门 OPEN 时跑）：
`neotrix-neobot --lib` **308 绿 / 0 红**；`neobot-desktop` **4+4 注册 + 38 IPC 冒烟绿**；
`cargo check -p neobot-desktop --all-targets` 0 error 0 warning；
前端 `typecheck` / `selftest` / `build` 全过；`lock_audit` **0**；
clippy **新增/改动行 0**（27 条全在未触碰的旧代码）。

## 修真 bug（agent 报的问题，主 agent 逐条读现场证实后修 —— 不是照单全收）

1. **跨聊消息静默丢失**：`dedup_key` 只有 `{channel}:{message_id}`，而平台
   message_id **按 chat 各自编号** → A 群 5 号与 B 群 5 号撞键，后一条被当重复丢掉。
   改 `dedup_key(channel, chat, message_id)`。回归测试 `dedup_key_is_scoped_by_channel_and_chat`。
2. **入站附件永远读不到**：落在 `<data_dir>/attachments`，jail 只放行
   `<data_dir>/workspace` —— **兄弟目录**，note 里绝对路径必被网关拒 → 收了等于没收。
   改落进 `workspace/attachments`（不扩 jail）+ note 给工作区相对路径。
   回归测试 `inbound_attachment_lands_inside_the_jail_and_is_reachable`。
3. **第二个及以后的机器人永远收不到消息**：`run_once` 把 `poll()` 放进
   `for bot in &bots`，而适配器**按渠道**注册、offset 全渠道共享 → 第一个取走全部。
   改「每渠道只 poll 一次 + 按白名单 `route_bot` 路由」；桌面
   `neobot_channel_poll_once` 同一处也改了。
4. **`/stop` 回执推荐了不取消任何东西的键**：桌面停止键只置 `shell.stopRequested`
   让渲染跳过增量，`spawn_blocking` 照跑 —— 而测试还断言「桌面 App」必须在回执里，
   **把假建议钉成了契约**。回执改为只说事实 + 给真能生效的路径；测试改为**禁止**
   出现「按发送键」等动作短语。README / 吸收文档同步。
5. **2 条 clippy 落在新增行**（`nt_store_convos.rs` 10 元组 `type_complexity`）→
   提 `ConvoRow` 具名结构。

另改一批**在说谎的注释**（比没注释更坏）：`TurnStatus` 自称有 `failed/cancelled`
（实为 `done/continue/needs_clarification/blocked/waiting`）、`slice_sleep` 自称
让 Ctrl-C 200ms 生效（不检查任何标志）、`BotRow` 自称各机器人独立绑定模型、
`OutboundMessage.edit_of` 看着像已实现（实为半接）。

## 内存门 BLOCKED 期间的零编译批次

`rustc` 占 3.5GB 编 `neotrix-core` 时全程禁 cargo / npm，只做静态分析与文档：
- 关验证缺口「前端 invoke 键名 vs Rust 形参名没人守」→ 新增
  `scripts/ops/nt_ipc_keys.py`（纯标准库，`--self-test` 60/60），实测声明 97 /
  注册 97 / 零错配；接成 `nt_smoke.sh` 第 6 步。**仍不覆盖**：只到键名不到类型，
  只到静态不到运行时。
- 订正 R-P199 引用（见上「台账归属」）。
- 门记录刷新（R-SCAN-3）：`lock_audit` 两 scope 均 0，`AGENTS.md` 80 行。

## 现况（2026-09-29 独立核验，非复述旧文）

- 桌面端已迁独立仓 `~/Downloads/Neo/neobot`（`d5413335` 移除本仓 app）；
  12 模块随 `6ef1a96`（28→49 文件）完整带过去，无丢失。
- `dedup_key` 双约束 + 回归测试、`route_bot`、`poll_once`、/stop 禁断言
  全部在现仓源码中且测试通过（channel 相关 20+ 全绿，smoke 7 全绿）。
- 数字自然增长：lib 308→370+，IPC 97→111（后加 llamacpp 6 + web/vision 7）。
  本文件 §「收口复核」的数字是**当时值**，不随增长改写 —— 改写等于伪造当时记录。
