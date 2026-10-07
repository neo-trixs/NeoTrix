# handoff —— 2026-10-07「轨迹页 + 诚实增量 + 门复活」窗口

> 窗口目标：按用户给的 6 个外部源，补齐 neobot 界面与架构节点的缺陷。
> **收工状态**：代码全部落盘、测试与门全绿（除 4 道**既存**红门，见 §4）、
> ⛔ **未提交**（用户未要求提交；且主树有他窗在途工作，见 §7）。

---

## §1 我改了什么（21 个文件，其中 4 个新建）

### 库侧（`crates/neotrix-neobot`）— 新增轨迹读侧

| 文件 | 改动 |
|---|---|
| `src/nt_run_trace.rs` | **新建**。契约 DTO（`RunRow`/`RunListView`/`StepView`/`ChangeView`/`RunTraceView`）+ `run_list` / `run_trace` + 8 条单测 |
| `src/nt_store/nt_store_run_trace.rs` | **新建**。SQL 只放这里（`conn` 是 `nt_store` 私有字段）+ `list_run_rows` / `get_run_row` + 4 条单测 |
| `src/nt_store/mod.rs` | +2 行：注册子模块、重导出 `RunStoreRow` |
| `src/lib.rs` | +2 行：`pub mod` + `pub use` |

### 壳侧（`apps/neobot-desktop`）

| 文件 | 改动 |
|---|---|
| `src/commands.rs` | +`neobot_run_list` / `neobot_run_trace`；`neobot_send` 增 `AppHandle`（**泛型化**）并发 `neobot:run` 三段信号；+6 条单测 |
| `src/api.rs` | 契约登记 2 条新命令（并补记它们为什么曾是缺口） |
| `src/lib.rs` | 注册表 +2 条 |
| `tests/ipc_roundtrip.rs` | 修好**HEAD 上就红着**的 2 条（见 §4.4） |

### 界面（`neobot-ui/`）

| 文件 | 改动 |
|---|---|
| `src/neobot-root.tsx` | +`对话/轨迹` 页签、+`TraceView`、+`classifyFailure`/`failHint`/`stepLabel`/`changeLabel`、+看门狗、+`prefers-reduced-motion` 接点、空白回复计失败、7 处 `text-[11px]`→`[12px]` |
| `src/nb-reduced-motion.css` | **新建**（已 `git add`，否则 selfcontained 门报「不可从干净克隆构建」） |
| `src/i18n/locales/{zh,en}.json` | 89 → **129** 词条，两语严格对齐 |

### 门（`scripts/ops/`）— 本轮最大的一笔

| 门 | 改了什么 |
|---|---|
| `nt_check_api.mjs` | 修**崩溃**（`ReferenceError: f`）+ 注册表改读 `lib.rs` + 字符串感知剥注释 + 交付树/vendored 两种口径 |
| `nt_check_ui_calls.mjs` | 同上 + 改为**两棵树都扫**、aliasRoot 显式传入 |
| `nt_check_ship_ui.mjs` | 同上 |
| `nt_check_status.mjs` | 同上 |
| `nt_check_layout.mjs` | `DIST` 从 `frontend/dist` → `neobot-ui/dist`（**复活**）+ 桩跟上 `_page` + 加 2 条新命令 + 2 条断言按窗口化重写 |
| `nt_check_interact.mjs` | 同上（**复活**）+ 换掉四层后代链选择器 + 修 `convoId` 方向反了的断言 |
| `nt_check_trace.mjs` | **新建**，第 11 道门 |
| `nt_shot.mjs` / `nt_shot_state.mjs` | 仅改 `DIST` 路径（未逐条跑） |

### 文档

- `apps/neobot-desktop/STATUS.md`：§1.3 命令清单 / §1.5 门禁数 10→11 / 测试数 97→107 /
  §4 教训 **40–43** / 新增 **§11** / §6 许可边界 +6 行
- `docs/architecture/ABSORPTION-2026-10-07-TRACE-AND-HONEST-INCREMENT.md`：**新建**

---

## §2 验证结果（全部实跑，非推断）

```
cargo test -p neotrix-neobot --lib   581 passed  0 failed
cargo test -p neobot-desktop          81 + 3 + 5 passed  0 failed   （共 89）
cargo check -p neobot-desktop --all-targets   0 error
nt_lock_audit.py crates/neotrix-neobot/src   可疑 0 处
```

**19 道门全绿**：`nt_check_{api,visual,ship_ui,upstream_1to1,layout,interact,trace,status}` +
`neobot-check-{contrast,convo-groups,convo-keys,input-defects,ipc-keys,layout-scale,shortcuts,theme-persist,typography,selfcontained}` +
`nt_neobot_ui_wiring.py`。

⭐ **变异验证（新门必须做的）**：把「用量只在对应对话页」改成两页都显示 ⇒
门**第一版没抓到**（只量面板、数字在头栏）⇒ 改量整棵根后变红；
还原后复跑绿，且 `diff /tmp/nb-root.bak.tsx` 与变异前**逐字相同**。

---

## §3 一个前提要纠正：`egoist/mygo` 里没有 harness 界面

用户描述它「用 AI 对着 DeepSeek harness 抄了界面、复刻了流式对话页和轨迹页」。
**实测不成立**（curl API + 全树 + AI 分支 + 最近 100 commit）：
mygo 是 **Go 桌面框架**（WebView 或原生 UI），729 个 blob 里无任何 harness/chat/trajectory
界面文件，`examples/gallery/main.go` grep 全 0 命中。
⇒ 本轮**未从 mygo 取任何东西**。详见吸收记录 §0。
⚠️ 另：`titanwings/colleague-skill` **已改名** `distilly`，默认分支 `dot-skill`（`main` 404）。

---

## §4 遗留的 4 道红门 —— **全部既存，已用 `git stash` 逐一证实非本轮引入**

| 门 | 现象 | 证法 |
|---|---|---|
| `nt_check_bytes` | 11 处 U+FFFD，**全在我没碰的文件**（`ABSORPTION-*.md`、`kb_search.rs`、`nt_dead_flag.py`、`nt_worktree_gate.sh` …） | 逐条列名核对；我改的 21 个文件**零命中** |
| `neobot-check-markdown` | FAIL 11 项（代码块未渲染 ⇒ 复制按钮文案取不到） | stash 掉我的 `neobot-root.tsx` 并重建后**同样 11 项** |
| `neobot-check-msg-copy` | 等不到 `[data-testid=nb-msg-copy]` | 同上，HEAD 态同样失败 |
| `neobot-ui-smoke` | rc=1，4 条 ⛔ | 同上，HEAD 态 rc=1 且 ⛔ 完全相同 |

⛔ 这 4 道我**没有修**（超出本轮范围，且各有归属人）。
但注意它们的**共因**：三道 UI 门都指向/读着**冻结的 vendored 树**——
与我在 `nt_check_layout`/`nt_check_interact` 修掉的是同一个病。
⇒ **建议下一轮先审这 4 道门的树指向**，否则它们会继续报「别人的调用」。

---

## §5 本轮挖出、但**故意没做**的欠账（清单在吸收记录 §6）

最重要的一条：**真 SSE token 流仍不可达**。
库里**有**真流式（`nt_http_engine::run_turn_stream` 的 SSE 解析 +
`nt_agent::run_local_turn_stream_as` 的 `on_delta`/`on_step`），
但 GUI 的 `neobot_send` 走**配对核心的 `POST /v1/agents/run`** ——
那个端点是否支持 SSE **本仓无法验证** ⇒ 按「不可用就不渲染」，
只发了能证实的生命周期信号，**没有伪造 delta 流**。

⚠️ `neobot-ui/src/ipc.ts` 里早先否掉事件流协议的理由写的是「**后端没有可流的东西**」——
**那句话不完全准确**（库里确实有流式能力），只是当前这条路径够不着。
下个窗口别照抄那句结论。

---

## §6 收工自查（AGENTS.md §8「收工自查」必填）

| 项 | 状态 |
|---|---|
| **worktree 去向** | ⛔ **我没有开任何 worktree**。现存 4 个是**别人的**（`nt_worktree_gate.sh check` 报：2 个带未提交改动、1 个近 3h 有 `.rs` 改动）⇒ **我一个都没删**（R-DISK-5：删前须 patch 兜底，且门明确警告「可能他窗在用」） |
| **未提交改动去哪** | ⛔ **全部留在工作树，未提交**（用户未要求提交）。文件清单见 §1，共 21 个。⚠️ 用 `git add <显式路径> && git commit --only <同一批>`；⛔ **共享 index 下禁 `-A`**（AGENTS.md §1：2026-09-29 两次实测事故） |
| 我碰过的**未跟踪**新文件 | `crates/neotrix-neobot/src/nt_run_trace.rs`、`crates/neotrix-neobot/src/nt_store/nt_store_run_trace.rs`、`scripts/ops/nt_check_trace.mjs`、`docs/architecture/ABSORPTION-2026-10-07-TRACE-AND-HONEST-INCREMENT.md` —— 提交前需 `git add`（`selfcontained` 门已因 `nb-reduced-motion.css` 报过一次「不可从干净克隆构建」，那个我已 add） |
| **他窗在途**（⛔ 勿动） | 收工时发现 `neotrix-core/src/l0_substrate/nt_judge.rs`（13:13）与 `docs/architecture/DEBT-LEDGER-2026-10-07.md`（13:21）**近 15 分钟内被改** ⇒ **另一窗口正在工作**。我**没碰**这两个文件。另主树 `target` 83808M、`git status` 共 38 处未提交（含 `.neotrix/patches/*.patch` 8 个 10-05/10-06 的旧残留，**非我**） |

---

## §7 下一步建议（按依赖排序）

1. **审 §4 那 4 道红门的树指向**（同一病因，我在两道门上已修好可照抄）
2. `steps` 表加时间戳/耗时 ⇒ 轨迹页才能显示「何时/多久」（结构性，另立一轮）
3. `theme.css` 深浅两档改**结构性**对齐（hermes-workspace 用双向穷尽 `Record` 让「只发一个变体」编译不过）—— 本仓 `theme.css` 自己的注释记着「只补浅色 ⇒ 门也抓不到」
4. 抽 `scripts/ops/lib/strip-comments.mjs` 共用（5 道门各写了一份）
5. **需要产品判断而非 agent 判断**：taste-skill §4.11「一页一主题」与本仓「气泡固定内容色」是**设计分歧**，我**没有**单方面改