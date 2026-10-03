# NeoTrix — Agent Guide

> **指针守恒**：本文件只存「判据 + 去哪读」，不存快照正文。全量索引 `docs/architecture/CODEBASE-WIKI-2026-09-21.md`；查文件用 `glob`/`grep` 现查。
> **引用规则**：按**章节名**引用（如「三道闸」），⛔ 不用行号 —— 行号锚点必腐化（`TODO.md` 曾写 `AGENTS.md:53` 指 R-P199，实际漂到 65 行）。

## 0. 决策树：任务 → 动作（先查这里，别全文读）

| 我要做的事 | 走哪条 | 判据在哪 |
|---|---|---|
| 日常改码 | 定点→最小改→`cargo xl`（`= cargo test -p neotrix --lib`）；全量档 `cargo check --all-targets -p neotrix` · `cargo build -p neotrix` | 硬规则 §1 · 微操作公约 §3 |
| 结构性改动 | `cargo clean && cargo build` **跑两遍** | 硬规则 §1 |
| 跑重型构建前 | `nt_mem_gate.sh` 非 0 则**禁止起构建** | 三道闸 §4 |
| 改完 `.rs` | `nt_lock_audit.py` 重跑，**禁止沿用旧值** | 三道闸 §4 |
| 找「该跑哪个脚本」 | `nt_find.py <意图>`（75 条索引，每条带「何时别用」） | 三道闸 §4 |
| 删 worktree / 收工 | `nt_worktree_gate.sh prune`，**禁手删目录** | 硬规则 §1 · 三道闸 §4 |
| 接外部技术 | 同会话接到生产可用，否则不算做完 | 硬规则 §1 |
| 扫出告警 | **先读现场证实/证伪**，再决定动不动 | 扫描器告警 §5 |
| 关窗口前 | 收工义务 3 步 | 硬规则 §1 |

## 1. 硬规则

- **`#![forbid(unsafe_code)]`** —— 永不加 `unsafe`。生产代码禁 `unwrap`/`expect`/`panic!`，错误用 `?` 传播。模块名一律 `nt_` 前缀；分层 `l0_substrate`→`l6_meta`。
- **R-P16** 编辑后**必须重读文件**验证落盘；禁整文件覆写他人内容。**R-P79** 外部技术**必须同会话接到生产可用**（导出 ≠ 接入）。
- **提交用 `git commit --only <我的文件...>`** —— 共享 index 下「暂存区核对」与「提交」**不原子**，2026-09-29 两次实测事故。⛔ pre-commit 门防不了（三种 commit 方式都只给 hook 传 0 个参数）。补救见 `sessions/handoff-commit-only-2026-09-29.md`。
- **收工义务**（违反即阻塞）—— ① `nt_worktree_gate.sh check`；② 自己开的 worktree 走 `prune` 收掉；③ 写 `sessions/handoff-<窗口>.md`，模板 §8「收工自查」**必填**（worktree 去向 + 未提交改动去哪：`git add` 提交 / patch 兜底 / 明确声明弃用）。依据：2026-09-28 实测 22 worktree 占 28G、**850 处未提交改动不在任何提交里**，事后大扫除误删 `ratchet`（4 处）靠 patch 找回。**收工是自己的义务。**
- 细则：锁/构建/卡死判别/Git/修 bug 判据/字节安全 → `RUST-STANDARDS.md` §17。

## 2. 并行公约（2026-09-21 事故复盘）

- 同一工作区只留 **1 个 watcher**；多任务用单窗口 Task 子代理；真并行走 `.worktrees/`。⛔ 禁多窗口同时跑 `--all-targets` / `--test` 全量构建（16G 机必爆 swap）。
- `stash pop` / `checkout -- <path>` 前**先喊一声**（2026-09-22 三次覆盖事故）。
- 关窗口前写 `sessions/handoff-<窗口>.md`（模板 `sessions/HANDOFF-TEMPLATE.md`），收齐 + stash 兜底后再关。

## 3. 微操作公约（skill: nt-locate）

**闭环**：点选/标注 → 定点 → 最小改动 → 单测验证。**无定点不改。** `python3 scripts/ops/nt_locate.py --component X --source-file Y`（选择器 → 文件:行）。

- **下刀前查并发**：`git status --porcelain <file>` 看他窗改动 + `stat -f "%Sm"` 看 mtime。**mtime 数秒内变过 = 他方在写，换文件或先通报**（实测撞见另一窗口 12 秒前正在改同一文件）。
- ⚠️ `nt_locate.py` 索引可能陈旧（`--audit` 曾显示 3 天前、476 缺失）⇒ 改用 grep 定点。

## 4. 三道闸（2026-09-27 事故后置入，违反即阻塞）

### 4.1 内存 / 死锁 / sidecar

| 闸 | 命令 | 判据 |
|---|---|---|
| 内存 | `sh scripts/ops/nt_mem_gate.sh; echo $?` | 非 0 禁起构建 |
| 死锁 | `python3 scripts/ops/nt_lock_audit.py neotrix-core/src` | **2026-09-29 实测 0 条**（RC=0） |
| sidecar | `sh scripts/ops/nt_sidecar.sh {start\|stop\|status}` | 用完即停 |
| **feature 门控** | `bash scripts/check-feature-gates.sh [--list\|--quick]` | **2026-09-30 核实 rc=0（6 个非默认 feature）**。改了 feature 门控的 `mod` / `Cargo.toml` features / 提交前跑（6 次 cargo check）。`--quick` = `--lib --tests`；`--all-targets` 另覆盖 examples/benches |

- **门记录纪律（R-SCAN-3）**：改 `.rs` 必须重跑；只改非 `.rs` 可沿用。历史：22:52 之前本文长期写「0 命中」而实际 12 条 —— **陈旧门记录会让下一个 agent 去「修」正确代码，比没有门更危险**。唯一一次非 0：2026-09-27 22:52 实测 3 条（1 真死锁 `kb_search.rs:549` + 2 误报），23:4x 修真死锁后归 0，此后每次复测均 0。

### 4.2 目录 / 命名（纯 bash，无需 cargo）

| 门 | 当前值 | ⛔ 怎么读这个值 |
|---|---|---|
| `check-layer-deps.sh --strict` | exit 0，**8 known** | 棘轮 101→…→8，`PASS 0 new`。剩余 8 条全是**已记录不可改道项**，删会让 CI 红。**测量台必须是 `git worktree add --detach HEAD` 的干净检出** —— 脏树值会让违规变少，照抄会让 CI 以 `FAIL: N new` 红 |
| `check-naming.sh` | advisory，clean-HEAD **1,646** 无前缀文件 | **规约 vs 现实差 1,646 ⇒ 该规约无约束力，advisory PASS ≠ 合规** |
| `check-truth-surface.sh` | — | ⛔ **本地红 ≠ CI 红**（他窗 WIP 造成 UNCOMMITTED_DEP）；干净检出 exit=0 |

- 层归属真源 `.neotrix/layer-map.json`；裁决表 `docs/architecture/OWNERSHIP.md`。改 feature 门控的 `mod` 宿主（含 `neotrix/` 树的 `ios-bridge` 门控）后跑 `check-feature-gates.sh`，它按 `--all-targets` 覆盖 6 个非默认 feature。删 `pub use` 前用能匹配花括号的形式查消费方 —— `neotrix::{A,B}` 不匹配 `mod::Name` 搜索式（2026-09-30 据此删掉活代码，而默认 `--lib`/`--all-targets`/12,209 测试/全部门均绿）。⛔ **`rg -E` 在本机静默返回 0**（`error parsing flag -E: … unknown encoding`），`rg` 不带 `-E` 正常（蓝图 `D-[0-9]{2}` 实测 87 vs 0）⇒ 命令一律用 `rg -n '…'`，且**不得用 `2>/dev/null` 吞 stderr**，零命中先确认退出码 1 而非 2。缺口诊断见 `docs/architecture/CAPABILITY-GAP-2026-09-30.md`。
- ⚠️ **同名 ≠ 同一符号**（换错了 `cargo check` 不报错，L15）；门**分不清字符串字面量**（L14）；批量改道须自查有无改到注释行（L13/L16）。改跨层引用**唯一合法通道是「消费方自己那层」的 facade**；走目标层 facade 无效。

### 4.3 磁盘 / worktree

- **只删生成物，不删带脏文件的 worktree**。`.worktrees/*/target` 常占 90%+ 体积。删 `target/` 零风险；删 worktree 本体须过双闸：`status --porcelain` 为空 **且** `branch -a --contains HEAD` 非空。**有 cargo 在跑时不碰主 `target/`**。细则 R-DISK-1~7 → `RUST-STANDARDS.md` §17.7；worktree 门 → `scripts/ops/WORKTREE-GATE.md`。

### 4.4 任务 → 工具索引

`make find QUERY="死锁"` / `python3 scripts/ops/nt_find.py 死锁` —— 75 条意图索引在 `.neotrix/task-index.json`（2026-10-03 实测；原文写 19，已漂移），**每条必带「何时别用」**：只写「何时用」agent 会用错（`check-naming` PASS 不代表合规；`nt_lock_audit` 报 12 条里 2/3 是误报）。pre-commit 校验索引指向的工具存在。

## 5. 扫描器告警 ≠ 缺陷（2026-09-27 差点把 bug 修进正确代码）

- **R-SCAN-1** 扫描告警**先读现场证实/证伪再动代码**。本轮 `nt_lock_audit` 报 12 条，逐个读代码后 **2/3 是误报**：`tor_client.rs:324` 作者已显式 `drop(proc);`；`llama_process.rs:329` 的 `*self.x.lock().await = v;` 是赋值型临时锁。若照单全修，会把 bug 引进**正确**代码 —— 「无定点不改」对扫描器同样成立。
- **R-SCAN-1b（2026-09-29 立）** **裸 `grep` 的命中不构成证据** —— 必须读那一行本身。
  本仓把 `unsafe`/`forbid`/`unwrap` 等禁词**当数据持有**（`nt_meta/scanner.rs` 扫禁词、
  `nt_laws.rs` 夹具、文档字符串），**注释与字符串里的命中全是误报**。
  实测：`grep forbid(unsafe_code) crates/neotrix-sysctl/src/lib.rs` 命中 ⇒ 我断言
  「声明失效」并立成 TODO 的 **P0 裁决项**。**真相**：第 12 行是
  `#![allow(unsafe_code, reason=…)]`，命中的是第 10 行**注释里的文字**；5 处 unsafe
  **每处带 `// SAFETY:`**，上层 `neotrix-core` 保持 `forbid` ⇒ **设计本就正确，我制造了问题**。
  ⛔ 同会话我刚写完 `_strip_noncode` 并在 `CODE-TOPOLOGY.md` 写下「raw grep 会误报」，
  **然后自己用 raw grep 下 P0 结论**。⇒ 判据统一走 `nt_topology.py`（5.1，已剥离）
  或 `nt_locate --component`。**grep 只找候选行，不下结论。**
- **R-SCAN-2** **手推 ≠ 实证**。判断扫描器行为必须把**真实代码形态**喂进去跑 ——
  本轮手推 `audit_indirect` 不会误报，实测它确实误报。
- **R-SCAN-3** **门记录必须带核实时间戳**（见 §4.1）。
- **R-SCAN-4 门脚本的干跑本身可能有副作用 —— 先审再跑**（2026-09-29 实测事故）。
  新写的 `check-disk.sh` 在「建议」文案里写了**反引号包裹的示例命令**，bash 把它
  当命令替换**真的执行了** `cargo clean` ⇒ **删掉 115.2 GiB**，且当时有 4 个他窗
  cargo 在构建。⇒ ① 反引号在 bash 里不是排版；`bash -n` 抓不到，语法完全合法。
  ② 新门先审有无写操作（判别标准：除 `du`/`df`/`ps`/`git status` 等只读命令外
  不得有写操作），**再**干跑。R-SCAN-2 的「喂真实输入跑」前提是先读码确认无副作用。
## 6. 正典索引

> 裸文件名默认指**根目录**；其余全部位于 `docs/architecture/`。**先读 `docs/architecture/README.md`（阅读索引）。**

| 主题 | 正典 |
|---|---|
| **唯一图纸** | `NEOTRIX-MASTER-BLUEPRINT.md`（D-00~D-15，按图施工） |
| **唯一排期真源** | `FINAL-ROADMAP-2026-09-29.md`（45 仓四轮吸收定稿）· 特性级 `FEATURE-MAP-TASKS-2026-09-29.md` |
| 架构现状 | `ARCHITECTURE.md` ⚠️ **§1-§12 已被 §13 推翻，只读 §13 起** |
| 模块台账 | `ARCHITECTURE-MAP-ROADMAP-V2.md` —— **2026-09-29 已拆分，只留 §11 起的事实对账层**（可再生实测值）。§1–§7 的 308 行死数据已移入 `_superseded/ARCHITECTURE-MAP-ROADMAP-V2-deathsnap-2026-09-19.md`。更新规则 **R-P199**，口径限 `neotrix-core` L1–L6；`neobot` 独立 crate 不占 L 层故不进 ⇒ 见 `ABSORPTION-DSH-SIDEBAR-IM.md` |
| 模块拓扑实测 | `DIR-AUDIT-2026-09-27.md`（16 包依赖图 + 8 类重复类型）· **目录解法** `DIR-REMEDY-2026-09-28.md` |
| 外部吸收 | `ABSORPTION-AGENT-ARCH-2026-09-28.md`（8 源）+ `…ARCH2-2026-09-29.md`（30 源，含 5 个被证伪前提，3 仓无 LICENSE ⇒ 只取设计）+ `BATCH-FIX-2026-09-29.md` · `ABSORPTION-EXTERNAL-2026-09-27.md` |
| 方法论教训 | `LESSONS-*.md` **8 档，按主题挑读，勿只读最新**。纪律类见 `…2026-09-24.md` §五（R36–R46：反引号当命令执行 / 门干跑有副作用 / `--only` 按路径取 diff / 全角标点吃字节 / 门记录声称已做而实现从未入库） |
| 文档规范 | `DOCUMENTATION-MAP.md` |
| 本地模型 | `LOCAL-LLAMA-2026-09-28.md` |
| 待办 | `TODO.md`（顶部人工摘要区）· 事故分诊 `sessions/handoff-disease-list-20260927.md` |

### 6.1 承接前必读（省数小时）

- `sessions/handoff-20260928-new-window-opening.md` —— 必读顺序 + 三条硬约束 + **「主工作树不是可信地面真相」**。
- `sessions/handoff-20260928-consolidated.md` —— ⛔ **动手前必读其 §2 勘误表**：4.1/2.1/2.2/5.2/4.4 的台账前提均已被实测证伪，照原文做会重造已存在的东西或「修」已正确工作的机制。
- `sessions/handoff-20260928-algo-extraction.md` —— 目录/算法改造交接（8 项算法萃取进 L0–L6 + 6 目录归档 + 删 4,640 行死引擎；含「反查消费者须一并 grep `.github/workflows/`」的方法论教训）。

### 6.2 三条血泪教训（照抄结论，不照抄数字）

- `DIR-REMEDY-2026-09-28.md` §2.5：`neotrix-core/src/neotrix/`（129 文件/43,834 行）是**不参与 L0–L6 的第二棵树**且完全逃过 `check-layer-deps.sh`；解法是**层归属显式化**（`layer-map.json`）而非搬目录。其 §2.5 记录 `nt_jev` + `nt_crystal_core` 是**活路径**（L1 有 6 个消费者），**勿当死代码删** ——「导出 ≠ 调用」已错过 3 次。
- `LESSONS-20260929-checked-is-not-verified.md`：`nt_judge.rs` 标注「EVO-02 mu 式」，但 `qybaihe/mu` 自己的回测显示 **admission/chunk 准入是它成本最高（54% token）、收益为零（2412 块 drop 0 个）**。⇒ `handoff-evo-20260926.md:68` 把 admission 列 P0 的表述需改判。**我们是无 I/O 的纯规则实现，故那些数字不适用，但方法论要抄。**
- 8 档 `LESSONS-*` 的元教训统一是：**任何「X 是好的/坏的」断言都要问「我是在哪个环境里验证的」；答「我的工作树」就等于还没有证据。**

### 6.3 已废止（⛔ 勿读、勿实现）

`ARCHITECTURE.md` §1-§12（已被 §13 推翻）· `EVOLUTION-ROADMAP-CODE-NODES-*.md`（行号/计数已部分失效，留作取证）· 根 `dev-rules.md`（已于 2026-09-29 删除。代码 `nt_core_self_constitution.rs` 的 legacy companion 候选对「文件不存在」分支本就是 stub-safe ⇒ 正典 `docs/standards/NEOTRIX-STD-1.0.md`。**删除等价性已实证**：`cargo test -p neotrix --lib self_constitution` 11 绿（含两个加载真实 AGENTS.md 的测试），且「桩存在」与「桩删除」两态结果完全相同；`--lib self_test` 60 绿。勿重跑）

## 7. 本地模型

权重 `<repo>/models/` ＋ 归档区兜底（**均 gitignored，git 保护不到，删了只能重下**）。启动 llama.cpp **必须**带 `--jinja` `--reasoning off` `--ctx-size <N>` 显式值，否则 Qwen3.5 系「装完开不了话」。原因、KV 推导、实测数据 → `LOCAL-LLAMA-2026-09-28.md`。
