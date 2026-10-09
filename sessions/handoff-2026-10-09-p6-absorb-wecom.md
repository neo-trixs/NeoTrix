# Handoff — 2026-10-09 上午：P6 迁移 / absorb-node --url / wecom 死码 + 任务2 移交

## 1. 会话标识

- 窗口：NeoTrix primary agent（opencode 主会话，非 tty s000）
- 分支：`feat/multi-agent-absorb-2026-10-08`（PR #11 base=main）
- 交接时间：2026-10-09 08:35 CST

## 2. 目标（一句话）

并行执行 4 项：① P6 `messages.compaction_head_seq` 持久化迁移；② `nt_gateway_http` 拉成
6 端点矩阵 + CostGate 接线；③ `neotrix-experience absorb-node` 接 `nt_self_forge`；
④ `nt_channel_wecom` feature gate 红档修复。

## 3. 已完成（本会话 4 笔提交，均已推送 PR #11）

- [x] **① P6 迁移** — `93f779d2`（3 文件 +71/-9）
  - `crates/neotrix-neobot/src/nt_store/mod.rs:461-471` ALTER 末尾追加列 + `:477-485`
    CREATE TABLE 同步带列；三列都在末尾 ⇒ 位置读 `r.get(0..N)` 不移位；老库补列由
    `.ok()` 吞 duplicate、新库由 CREATE 带上 ⇒ 两条路幂等。
  - `nt_store_messages.rs` `ChatMessage.compaction_head_seq: Option<i64>` +
    `stamp_compaction_head()`（写）+ `latest_compaction_head()`（读最近摘要行）。
  - **旧库迁移夹具补强** `nt_store_tool_calls.rs` 的
    `legacy_db_without_table_migrates_and_keeps_rows`：现在断言存量行读回 `None` ——
    `list_messages` 已 SELECT 该列，这句能过即证明 ALTER 真在存量库上补了列。
  - 修腐化引证：模块头原写「`rg 'DELETE FROM messages|UPDATE messages'` 零命中」，
    **实测 UPDATE 有 2 处**（`mark_latest_delivery` + 本次 stamp），真零的只有 DELETE。
    结论不变（UPDATE 不改 rowid ⇒ 无重号来源），证据改成实测口径。
  - 证据：`cargo test -p neotrix-neobot --lib -- nt_store` = **68 passed / 0 failed**；
    `nt_lock_audit` 两处路径均 **0**。
  - ⚠️ **诚实缺口**：`stamp_compaction_head` 目前**只有单测调用**，生产侧尚无
    compaction 生产者（`grep compact` 生产路径零命中）。本笔只交付 schema/读写 API 层。

- [x] **③ absorb-node 接 self_forge** — `b51dbedc`（3 文件 +56/-17）
  - `nt_self_forge::absorb_node_candidate(url)` 抽成独立函数（原先内联在
    `forge_from_url` 体内 ⇒ 两边同源，杜绝口径漂移）。
  - `neotrix-experience absorb-node --url <github url>` → `cmd_absorb_node_from_url`
    → 同一条 `absorb_node_value`（归一化/去重/`--apply-capability` 全复用）。
  - 证据：`cargo run -p neotrix --bin neotrix-experience -- absorb-node
    --url https://github.com/storytold/cadcraft --dry-run`
    ⇒ `would_insert #0` / `1 inserted, 0 duplicated, 0 mapped`，**RC=0**；
    `cargo test -p neotrix --bin neotrix-experience` = **21 passed / 0 failed**。

- [x] **④ wecom 死码** — `2e7767ca`（-3 行）
  - 删 `WecomChannel.failures`：HEAD 版本里只出现 2 处（声明 + 初始化），**零读点**，
    `probe()` 的健康位来自实时 HTTP 结果 ⇒ 写-only 死字段。
  - ⚠️ **原任务前提「feature gate 红」是误报**：`check-feature-gates.sh --quick`
    2026-10-09 08:06 实测 **RC=0 / 6/6 PASS**，红档早在 `99f3878b` 修掉。
    08:30 复测（他方已加 `wecom-channel`）= **8/8 PASS，RC=0**。

- [x] **P0 附带修复** — `14ca6f1c`（+172）
  - `86ddcb57`（前窗）在 `lib.rs` 声明了 `pub mod nt_mcp_bridge;` 却**从未 `git add` 文件**：
    `git cat-file -e HEAD:...nt_mcp_bridge.rs` ⇒ `exists on disk, but not in 'HEAD'`。
    裸 `pub mod` 无 cfg 可逃 ⇒ **任何人 clone 后 `cargo check -p neotrix-neobot --lib`
    必报 file not found for module**（不经编译即已确定）。本笔只补文件，不动 `lib.rs`。
  - 证据：`cargo test -p neotrix-neobot --lib -- mcp_bridge` = **4 passed**（含 mock
    stdio MCP e2e）。

## 4. 正在改的文件 —— **全部不是本会话的**（关键！）

本机存在**第二个 opencode 会话（PID 73901，约 08:03 起）**，它正在做任务②和部分④。

| 文件完整路径 | 改到什么程度 | 归属 | 是否可独立提交 |
|---|---|---|---|
| `crates/neotrix-neobot/src/nt_gateway/mod.rs` | 6 路由 + 4 个 POST 全接 CostGate + 真 embeddings 核，编译绿 | **他方** | 否，依赖下面 2 行 |
| `crates/neotrix-types/src/core/nt_core_embedder.rs` | 新文件，**未跟踪** | **他方** | 必须与上一行**同一批**提交 |
| `crates/neotrix-types/src/core/mod.rs` | 声明 `pub mod nt_core_embedder;`，未提交 | **他方** | 同上 |
| `crates/neotrix-neobot/Cargo.toml` | +`neotrix-gateway` optional dep、+`wecom-channel = []` | **他方**（我方只加了前者） | 与 lib.rs 同批 |
| `crates/neotrix-neobot/src/lib.rs` | +7 行 `#[cfg(feature="wecom-channel")]` 门控 | **他方** | 同上 |
| `.github/workflows/ci.yml` | feature 矩阵改 `<feature>:<crate>`，补 2 条 neobot 条目 | **他方** | 可独立 |
| `crates/neotrix-neobot/src/bin/neobot.rs`、`nt_channel_serve.rs` | 在途 | **他方** | — |
| `neotrix-core/.../kb_search.rs`、`nt_judge.rs`、`capability_registry.rs`、`l*_mod.rs`、`nt-core-capability-tree/`、`scripts/ops/*`、`.neotrix/*.json` | 在途 | **他方**（共享 index 老名单） | ⛔ 本会话一律未碰 |

**本会话只改了**：`nt_store/{mod,messages,tool_calls}.rs`、`nt_self_forge.rs`、
`neotrix-core/src/bin/experience{,.rs /exp_absorb.rs}`、`nt_channel_wecom.rs`、
（新文件）`sessions/handoff-2026-10-09-p6-absorb-wecom.md`。

## 5. 下一步（按优先级）

1. **任务②遗留缺口（他方文件，只记录不代改）**：
   - **G1 缺「6 端点矩阵测试」** — `grep -rn 'oneshot|reqwest|TcpStream'` 在
     `nt_gateway/` 与 `crates/neotrix-neobot/tests/` **零命中**；现有 7 个单测全是
     解析函数/CostGate/嵌入核的**纯函数**级，**没有任何测试真的打到 6 条路由**。
     ⇒ 路由被删/改名时无测试会红（L8「报 PASS 却结构上不可能失败」同型）。
   - **G2 提交时的 P0 陷阱** — `nt_gateway/mod.rs` 的 `embeddings_handler` 调
     `neotrix_types::core::nt_core_embedder::embed_texts`，而该文件**未跟踪**、
     `core/mod.rs` **未提交**。若他方只 `--only nt_gateway/mod.rs` 提交，
     会**重演 `nt_mcp_bridge` 那种「声明在、文件不在」的断检出**。
     ⇒ 提交批次必须含 `crates/neotrix-types/src/core/{mod.rs,nt_core_embedder.rs}`。
   - **G3 注释与行为不符** — `nt_gateway/mod.rs` embeddings 处注释写
     「估算口径与生成端点一致」，实际乘数 `1e-7`，生成端点是 `1e-6`（差 10 倍）。
   - **G4 已实测为绿**（供他方放心提交）：
     `cargo check -p neotrix-neobot --all-targets --features gateway-http` **RC=0**、
     `--features wecom-channel` **0 errors** ⇒ ci.yml 新增的 2 条矩阵条目会过。
2. ① 的生产侧：等 compaction 生产者落地时接 `stamp_compaction_head`（**导出 ≠ 调用**，
   本仓已错过 3 次）。
3. 主树 44 处未提交改动需在**他方会话**收工时逐个落账（本会话不动他们的文件）。

## 6. 阻塞点

- 任务②被活会话占用 ⇒ 已按用户裁决「让给对方，我只复核记录」。
- PR #11 检查状态：`GET /commits/2e7767ca/status` ⇒ `state: pending`，
  `check-runs total=0`（**尚无 check 落地**，不能据此说 CI 绿）。

## 7. 给接手会话的话

- 恢复命令：先读本文件，再 `git status --short` / `git log --oneline -6` 核对。
- **并发探测（下刀前必做）**：
  `stat -f "%Sm %N" <文件>` + `find . -newermt '-180 seconds' -name '*.rs'`。
  本会话实测：`nt_gateway/mod.rs` mtime 在 08:09:45→08:09:58→08:11:24 三次跳动、
  diff 从 295 行涨到 433 行 ⇒ **有他方在实时写**，据此判定移交而非抢文件。
- ⛔ **不要**替他方提交 §4 表里标「他方」的任何文件（共享 index + 对方仍在写）。
- ⚠️ **L11 复现**：`sh .../nt_worktree_gate.sh check | tail -30; echo $?` 读到的 `0`
  是 **`tail` 的退出码**；真值不接管道重测 = **4**。门脚本退出码一律⛔别接管道。
- 构建前先 `sh scripts/ops/nt_mem_gate.sh`（本会话多次实测 OPEN，RC=0）。

## 8. 收工自查

### 8.1 worktree 去向

`sh scripts/ops/nt_worktree_gate.sh check` 输出（**不接管道，真值 RC=4**）：

```
路径 | HEAD | 分支 | 脏 | 体积 | target | 近3h活动
/private/var/folders/.../opencode/nt_p22 | 39a71379 | HEAD | 0 | 1823M | 1753M | no
/Users/neo/Downloads/neotrix/.worktrees/evo | d524e278 | HEAD | 0 | 3562M | 3480M | no
/Users/neo/Downloads/neotrix/.worktrees/merge-b | 1a48ecd3 | HEAD | 3 | 66M | 0M | no
/Users/neo/Downloads/neotrix/.worktrees/nt-v2 | 6b57fe08 | HEAD | 0 | 81M | 0M | no
[worktree-gate] ℹ️  主树：44 处未提交 | target 220591M（只报告，不影响退出码）
[worktree-gate] worktree=4 个 | 合计 5532M | target 占 5233M
[worktree-gate] 带未提交改动: 1 个 | 近3h有改动: 0 个
[worktree-gate] ⛔ .worktrees/merge-b 有未提交改动不在任何提交里
[worktree-gate] ♻️ target 累计 5233M ≥ 1024M ⇒ 零风险可回收
```

- 本会话**新建** worktree：**无**（全程在主工作树，未起隔离树）。
- 既有 4 个 worktree 均非本会话创建，**未删任何一个**（`merge-b` 带脏文件 ⇒
  按 R-DISK-5 必须走 `prune` 补丁兜底，不是本会话的活）。

### 8.2 未提交改动的去向

本会话改动**全部已提交并推送**：

| 文件 | 改动内容 | 去向 |
|---|---|---|
| `crates/neotrix-neobot/src/nt_store/{mod,messages,tool_calls}.rs` | P6 列 + 读写 API + 夹具补强 | ☑ 已提交 `93f779d2`（已推） |
| `crates/neotrix-neobot/src/nt_self_forge.rs`、`neotrix-core/src/bin/experience.rs`、`.../exp_absorb.rs` | `absorb-node --url` 接线 | ☑ 已提交 `b51dbedc`（已推） |
| `crates/neotrix-neobot/src/nt_channel_wecom.rs` | 删死字段 | ☑ 已提交 `2e7767ca`（已推） |
| `crates/neotrix-neobot/src/nt_mcp_bridge.rs` | 补入库修 P0 断检出 | ☑ 已提交 `14ca6f1c`（已推） |
| `sessions/handoff-2026-10-09-p6-absorb-wecom.md` | 本文件 | ☑ 本笔提交 |

`git status` 里剩下的 44 处**逐条归他方在途**（见 §4 表），本会话：
☐ 未 `git add` 任何一个 → **明确声明不代提交**（理由：对方会话 PID 73901 仍在写，
代提交会把他的半成品扫进我的 commit，且他可能继续写同一批文件）。

### 8.3 门状态

- `nt_worktree_gate.sh check` exit code = **4**（主树 44 处未提交 + `merge-b` 带脏；
  **本会话自己的改动已全部入库**，红色来自他方在途）。
- 提交前是否跑过：☑ `cargo check -p neotrix --lib`（RC=0）、
  ☑ `cargo test -p neotrix-neobot --lib`（**650 passed / 0 failed**）、
  ☑ `cargo test -p neotrix --bin neotrix-experience`（**21 passed / 0 failed**）、
  ☑ `bash scripts/check-feature-gates.sh --quick`（**8/8 PASS，RC=0**）、
  ☑ `nt_lock_audit`（`crates/neotrix-neobot/src` 与 `neotrix-core/src/bin` 均 **0**）。
- 门红归因：`nt_worktree_gate` RC=4 = **他窗 WIP**（非本会话引入）。
- L9 归属自查：4 笔提交逐个 `git show --stat` ⇒ **只含本会话文件**，无他方文件混入。
