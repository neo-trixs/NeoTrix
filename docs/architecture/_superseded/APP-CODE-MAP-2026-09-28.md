# App Code Map — 2026-09-28

> ⛔ **已归档（2026-10-06）—— 主体内容已不成立，保留仅作溯源。**
>
> - 本档的 `src-tauri/` 整表主体已于 `5c02e738` 归档（实测 `apps/neobot-desktop`
>   声明 19 文件/5,698 行，实测仅 12/4,525；`neotrix-neobot` 声明 48/24,804，
>   实测 65/38,081）。
> - 文中多处引用**已不存在的目录**。
> - 当前代码地图正典是 `CODE-TOPOLOGY.md`（由 `scripts/ops/nt_topology.py` 生成）
>   与 `.project-map/codemap.json`；本档**无生成器、无门、无维护触发**。

> 实测生成。**方法**：逐文件 `diff -rq` + `#[tauri::command]` 抽取 + workspace 成员核对。
> 一切结论可复算；不接受"看起来像"的判断（见 `LESSONS-20260928-fresh-checkout.md`）。
> 版本基线：`neotrix` HEAD `ecd10d3e`。

---

## 1. 三个 App 表面的身份（非重复）

| 路径 | rs 文件 | 行数 | productName | identifier | workspace |
|---|---|---|---|---|---|
| `src-tauri/` | 115 | 33,834 | **NeoTrix** | `ai.neotrix.desktop` | ✅ `Cargo.toml:4` |
| `apps/neobot-desktop/` | 19 | 5,698 | **NeoBot** | `ai.neobot.desktop` | ✅ `Cargo.toml:18` |
| `crates/neotrix-neobot/` | 48 | 24,804 | — 库 crate（非 App） | — | ✅ `Cargo.toml:14` |

**结论：`src-tauri` 与 `apps/neobot-desktop` 不是重复。** 三条独立证据：

1. **产品标识不同**：`ai.neotrix.desktop` vs `ai.neobot.desktop`，是两个可并存的桌面应用
2. **规模差近 6 倍**：33,834 行 vs 5,698 行（5.9×）。前者是完整 IDE 级工作台，后者是精简客户端
3. **依赖方向**：`src-tauri/Cargo.toml` 显式依赖 `neotrix-neobot = { workspace = true }` ——
   **NeoBot 侧栏消费 NeoTrix 引擎**，是 consumer 而非替代品

`apps/neobot-desktop` **编译通过**（`cargo check -p neobot-desktop --all-targets` → 0 error）。
测试：`cargo test -p neobot-desktop -p neotrix-neobot` → **431 passed / 0 failed**。

---

## 2. `Neo/neobot/` 定性：**过期快照，不是最新版**

`/Users/neo/Downloads/Neo/neobot/` 此前**零版本控制**（无 HEAD / 分支 / remote），
1.3G 裸目录。现已建仓（见 §5）。

### 逐文件比对结果

| 方向 | 文件数 | 性质 |
|---|---|---|
| neotrix **有**、Neo/neobot 没有 | **328** | 3 个 Tauri 命令 · 6 个 smoke 测试 · 9 个前端 TS 模块 · `src/lib.rs` 库入口 |
| Neo/neobot **有**、neotrix 没有 | **13** | **全部无独立价值**（见下） |

### 那 13 个文件逐个定性

- `apps/neobot-desktop/frontend/dist/` **8 个** —— 前端**构建产物**（文件名带内容哈希
  `index-BneQdyp7.js` / `main-CTbe3-2Q.js`）。`npm run build` 可再生成，入库无意义。
- `apps/neobot-desktop/src/nt_cmd_{convo,core,run,sys,tasks}.rs` **5 个** ——
  与同树 `src/nt_commands/` 子模块**逐字节完全相同**（220=220 / 123=123 / 155=155 /
  334=334 / 140=140 行）。是 `nt_commands/` 子模块化重构后**忘了删的残留死代码**。
  neotrix 侧已清理干净。

### `crates/neotrix-neobot` 差异

neotrix 多 5 个文件：`nt_reply_tag.rs` · `nt_store/nt_store_reply_tag.rs` ·
`nt_stale_guard.rs` · `nt_token_guard.rs` · `nt_web.rs`；另 8 个文件内容有差异
（`nt_agent` / `nt_core` / `nt_http_engine` / `nt_policy` / `nt_provider` /
`nt_store/mod.rs` / `nt_store/nt_store_routines.rs` / `nt_types.rs` / `bin/neobot.rs` / `lib.rs`）。

### 唯一独有资产

`Neo/neobot/README.md` —— neotrix 树无此文件。**建仓前已用 git 留存**。

### mtime 佐证

`apps/neobot-desktop/src` 最新 mtime：Neo/neobot `09-26 12:14` vs neotrix `09-28`。
`crates/neotrix-neobot/src`：`09-26 11:20` vs `09-28`。

**处置：该目录停止开发，仅作历史留存。已在 commit message 中写明定性，防止下一个
agent 误当"更完整的项目"去合并（`LESSONS-20260928` 记录的正是这类反向错误）。**

---

## 3. NeoBot 命令面（103 个 `#[tauri::command]`）

| 前缀 | 数量 | 域 |
|---|---|---|
| `channel` | 11+1 | IM 通道（bot 别名/目录/轮询/开关） |
| `git` | 9 | Git 操作 |
| `convo` | 9+1 | 会话 |
| `fs` | 8 | 文件 |
| `task` | 7+1 | 任务 |
| `sidechat` | 5 | 侧边对话 |
| `core` | 5 | 核心 |
| `sidebar` | 4 | 侧边栏 |
| `changes` | 4 | 变更追踪 |
| `routine` | 3 | 例程 |
| `provider` | 3+1 | provider |
| `control` | 3 | 控制 |
| 其余 | 各 1–2 | run / roster / memory / member / cost / attach / skills / settings / models / doctor / audit / agent |

命令实现分布 `apps/neobot-desktop/src/nt_commands/`：
`nt_cmd_channels` · `nt_cmd_convo` · `nt_cmd_core` · `nt_cmd_files` ·
`nt_cmd_run` · `nt_cmd_sidebar` · `nt_cmd_sys` · `nt_cmd_tasks`（8 个模块）。

---

## 4. 引擎层 `crates/neotrix-neobot/`（48 文件 / 35 个 mod 声明）

```
lib.rs          nt_agent.rs      nt_audit.rs      nt_cli.rs
nt_computer.rs   nt_config.rs     nt_core.rs       nt_cost.rs
nt_daemon.rs     nt_engine.rs     nt_error.rs      nt_export.rs
nt_http_engine.rs nt_memory.rs    nt_policy.rs     nt_provider.rs
nt_reply_tag.rs  nt_routine.rs    nt_skills.rs     nt_stale_guard.rs
nt_token_guard.rs nt_types.rs     nt_web.rs
nt_store/  (mod, convos, tasks, changes, channels, routines, reply_tag)
```

守卫类模块（`nt_stale_guard` / `nt_token_guard`）与 `nt_web` / `nt_reply_tag`
**仅 neotrix 树有**，是 Neo/neobot 快照缺失的部分。

---

## 5. 版本控制现状

**唯一完整且受版本控制的 neobot app** = `neotrix/apps/neobot-desktop` + `crates/neotrix-neobot`
（commit `620e9712`，80 文件 / +39,428 −134）。

| 目录 | 受控文件 | 说明 |
|---|---|---|
| `apps/neobot-desktop` | 63 | 8 个 `frontend/dist/` 构建产物已 gitignore |
| `crates/neotrix-neobot` | 49 | 48 个 `.rs` + `Cargo.toml` |

### 归档区

| 路径 | 内容 | 版本控制 |
|---|---|---|
| `~/Downloads/Neo/neotrix-archive/neobot-stale-snapshot-20260926/neobot` | `Neo/neobot` 过期快照 | ✅ commit `1357d37` · 83 文件 |
| `~/Downloads/Neo/nt-backup-20260928/` | 主仓 4 天未提交产出双层备份 | `tracked.patch` + `untracked.tar.gz` + SHA256SUMS |

### 未归档项及原因

- **`src-tauri/`** —— 是 **NeoTrix**（`ai.neotrix.desktop`），不是 neobot app，且已受主仓版本控制。
  **不归档。**
- **`stash@{0}`** —— 保留。含并发窗口的 13 个无关改动（`ci.yml` / `Cargo.toml` /
  l5_cognition / l6_meta / scripts），不属 neobot 线，未动。

### 旧版存档

| 目录 | 建仓前 | 现在 |
|---|---|---|
| `neotrix/`（主仓） | 有 | 有 · HEAD `ecd10d3e` · 1 条 stash 兜底 |
| `Neo/neobot/` | **无** ❌ | **有** ✅ commit `1357d37` · 83 文件 · `.git` 1.1M |

`Neo/neobot/.gitignore` 已排除 `target/`（1.3G）、`node_modules/`、`dist/`。

⚠️ **主仓仍有 1 条未落库的 stash**（`stash@{0}`：759 tracked + 239 untracked），
外部第二层备份在 `~/Downloads/Neo/nt-backup-20260928/`。恢复前须与并发的其他窗口协调。

---

## 6. 待决事项

1. **`src-tauri/frontend/node_modules` 376M** —— gitignored 构建产物，删可回收，
   重装即得。
2. **主仓 stash 处置** —— 327 个文件的新旧版本判定，见 §5 警告。
3. **本文件自身** —— 数字会随代码变动失效。复核方式：
   ```bash
   find src-tauri apps/neobot-desktop crates/neotrix-neobot -name '*.rs' -not -path '*/target/*' | wc -l
   rg -c '#\[tauri::command\]' apps/neobot-desktop/src/ | awk -F: '{s+=$2} END{print s}'
   cargo check -p neobot-desktop
   ```
