# handoff — neobot N1~N6 演进轮（2026-10-08）

## 收工自查（模板必填项）

- **worktree 去向**：本轮**未新建任何 worktree**，全程在主工作树 `/Users/neo/Downloads/neotrix` 上做定点改动；`nt_worktree_gate.sh` 未跑（无 worktree 产生/删除动作）。
- **未提交改动去哪**：⛔ **全部留在工作树，未 `git add`、未 `git commit`**（遵循共享 index 纪律 + `--only` 纪律）。若窗口被强杀，改动仍在磁盘上，`git status --porcelain` 即可见；兜底手段：
  ```sh
  git diff > /tmp/neobot-n1n6.patch   # 或按文件 git add 精确提交
  ```
- **他窗 WIP**：工作树里另有约 20 个**非本轮**文件（`capability_registry.json` 全量重写、`nt_io_plugin/*`、`nt_mind/*` 等），属别的窗口，**本轮一律未触碰**。

## 本轮改了什么（已验证）

| 项 | 落地 | 验证 |
|---|---|---|
| N1 IM `/stop` | `nt_channel_serve::run_once` 跑轮解耦到 worker 线程（每 worker 独立 `NeobotStore` 连接同库 + 独立 `TelegramChannel` + 独立 engine；`mpsc + recv_timeout(200ms)` 收成确定性统计；缺 token/非 telegram/引擎不可用走旧同步语义）| `nt_channel_serve` 16 侧；**真并发回归** `stop_from_another_thread_interrupts_a_running_turn`（300ms 慢引擎 + 主线程翻 `/stop` ⇒ 落库 `Cancelled`/`stopped by user`）|
| N2 edit_of | `messages` 加 `platform_msg_id`/`delivery_status`；`ChannelAdapter::can_edit()`（默认 false、Telegram true）；占位 `⏳ 思考中…` → `edit_of` 覆盖；**流式增量编辑** `StreamEdit`（`STREAM_EDIT_MIN_INTERVAL_MS=1200` 节流 + 末尾 `finish()` 强制发）；`mark_latest_delivery` 记账 | dispatch 53 侧（含 4 流式测 + 2 占位测）；Telegram e2e `placeholder_then_final_answer_hits_edit_message_text_end_to_end`（假服务器断 `sendMessage`→`editMessageText` 且 `message_id=42` 真被复用）|
| N3 成本/压缩 | `CostPolicy` 上移 `neotrix-types::nt_cost_policy`；core `distill_output` 死循环修（对齐 neobot `8196dd10` 范式）；压缩摘要记 `COST_TRACKER`；`COMPACTION_SUMMARY_INPUT_MAX_TOKENS=24k` 预算门 ⇒ 超限**降级为明确驱逐**并记 `degraded_count` | neobot 606 绿；core `nt_io_agent_loop` 21 / `cost` 109 绿 |
| N4 路由组/额度 | `RouteMode::Quota`（失败最久优先）；ledger `session_id`/`key_env` 两列 + `HttpEngine::key_env_name()` 接线；`quota_windows` **快照法**三窗口；`neobot quota [--snapshot] [--kind]` + `set`/`rm` + `quota_limits` 人工上限（`source=manual`）| quota 4 测（三态出表 / 幂等不漂移 / 空库）；实机 CLI 验证 |
| N6.2 degraded | `is_degraded()` 判据（只有塌成 `…` 才算降级）；`run_loop` 落 `status=degraded`、`cost_usd=0` 行；`neobot ledger` 独立降级段 | 1 测（降级行不改费用合计）|
| N6.3 P0 | `docs/architecture/neobot-gateway-design.md`（端点矩阵/形状差异表/错误映射表/接线口径/P1-P3/风险登记）| 只读设计，无代码 |

**收口基线**：`cargo test -p neotrix-neobot --lib` = 606 绿；`cargo check -p neotrix-neobot --all-targets` 0 error；`nt_lock_audit`（neobot/core）0 处；`nt_smoke.sh` 全绿。
⛔ 用户中止了本轮最后一次 `check-feature-gates.sh`，故「feature 门 rc=0」是**上一轮**的记录，本轮**未复核**。

## 关键教训（方法论，沿用）

1. **域/跨执行流共享是 async/thread 化的真正成本，不是 HTTP 本身**：`NeobotStore`（rusqlite `Connection` 非 Sync）与 `ChannelAdapter`（`poll(&mut self)`）都不可共享 ⇒ 真并行的唯一安全形态是「每 worker 自己打开同库连接 + 自建适配器」。这次省掉了一版 `Arc<Mutex<…>>` 方案，因为它会把跑轮整个串行化（store 锁跨网络调用被持有）。
2. **`recv_timeout` 兜底必须能诚实分类**：超时（长轮）算「放手」，但**不能**假装成失败计数 —— 否则 `run_once` 的 stats 会撒谎。本实现把超时归到 `failed`（保守但可见），**已知待改**：应单列 `deferred` 计数，避免长轮被误读为失败。
3. **降级 ≠ 花钱 ≠ 失败**：压缩超预算降级、工具输出塌成 `…`、编辑失败降级为新消息 —— 三者都必须在账本/出表上**分列**，否则「这轮为什么什么都没拿到」永远查不出来。
4. **人工声明的上限必须带 `source`**：`quota_limits` 把「人填的」与「供应商报的」分列，否则出表会把猜测讲成事实（与 B10.4「记录不撒谎」同律）。
5. **快照法优于增量法**（额度窗口）：历史被改/删后重算即真值；代价是全表扫描（十万行后需索引/物化，已登记为技术债）。
6. **Rust 内联格式化不支持字段访问**（`{w.field}` 不合法）；`chrono` 的 `weekday()` 需显式 `use chrono::Datelike`。
7. **测试夹具决定架构**：`:memory:` 库无法被第二个线程打开 ⇒ 想测「第二个执行流」必须用文件库（`create_dir_all` + `NeobotStore::open(path)`）。

## 下一步（按顺序，首选 N6.3 P1，但有硬前置）

1. **N6.3 P1 网关骨架**（设计已就绪）：
   - ⛔ **硬前置 1**：依赖口径由**主人拍板** —— `axum 0.8` 已在 **neotrix-core** 依赖着，但**不在 workspace 依赖表**、neobot 未引；引它 = 改 `crates/neotrix-neobot/Cargo.toml`；另一选项是手写极简 HTTP/1.1（零新依赖，keep-alive/chunked 自己扛）。本轮倾向 axum（长尾正确性不值自造），但**必须先过 `check-feature-gates.sh`**（本轮未跑成）。
   - ⛔ **硬前置 2**：只绑 `127.0.0.1`（保 local-first）；`model` 名解析口径「先路由组、后 provider」。
2. **N6.1 P1 自动额度探针**：**需主人给 provider + 口径**（哪个 provider、走哪个 API/响应头）；未给则停在人工上限（P0 已完成）。
3. **N6.4 OTLP 出口**：P0 先做**事实核查**（core 已有 `nt_io_telemetry::init_l` + `otel_bridge::cost_tracker`，禁止重复造 exporter），P1 只把 neobot 的 ledger/quota 口径接进已有 exporter；验收含「未配置时零网络」。
4. **N6.5 移动端 PWA/LAN**：**跨仓（`Neo/neobot`），需主人授权**；本仓只提供只读 HTTP 快照，**不把前端塞回本仓**（`apps/neobot-desktop` 迁出的教训）。
5. **技术债登记**（见 N6.6）：`run_once` 超时应单列 deferred；`quota_windows` 大表快照；worker 无池上限；`ChatMessage` 位置元组读行（**新增列只许追加末尾**）。

## 门记录（带核实时间）

- `nt_lock_audit`：2026-10-08 本轮实测 `crates/neotrix-neobot/src` 与 `neotrix-core/src` 均 **0 处**。
- `nt_mem_gate`：2026-10-08 本轮 OPEN（free≈8G，avail≈173k）。
- `nt_smoke.sh`：2026-10-08 本轮 **✅ 全绿**（core / shell / 前端委托 typecheck+selftest / check / IPC 键名）。
- `check-feature-gates.sh`：⛔ 本轮**未复核**（用户中止）。