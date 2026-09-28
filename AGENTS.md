# NeoTrix — Agent Guide (slim)

> 全量 codebase 索引已归档：`docs/architecture/CODEBASE-WIKI-2026-09-21.md`
> （2426 `.rs` / 798707 行快照，含文件树 / Domain 表 / Key Types / CLI 表）。
> 查文件位置用 `glob`/`grep` 现查，不要把全量表塞回本文件。本文件保持 < 100 行。

## Build & Test

```bash
cargo xl                              # 最轻检查（= check -p neotrix --lib），日常只用这个
cargo check --all-targets -p neotrix    # 快速检查
cargo test -p neotrix --lib             # 单元测试
cargo build -p neotrix                  # 完整构建
```

结构性改动后：`cargo clean && cargo build` 跑两遍，以拿到真实错误数。

## 并行公约（2026-09-21 事故复盘）

- 同一工作区只留 1 个 watcher，多任务用单窗口 Task 子代理；真并行走 `.worktrees/` 隔离。
- 禁多窗口同时跑 `--all-targets` / `--test` 全量构建（16G 机必爆 swap）。
- 关窗口前写 `sessions/handoff-<窗口>.md`（模板见 `sessions/HANDOFF-TEMPLATE.md`），收齐 + stash 兜底后再关。
- 写文件前重读（R-P16），禁整文件覆写他人内容；`stash pop / checkout -- <path>` 前先喊一声（2026-09-22 三次覆盖事故）。

## 模块前缀规范

- 所有模块名用 `nt_` 前缀（如 `nt_core_cache`、`nt_mind`、`nt_shield`）。
- 分层：`l0_substrate` / `l1_action` / `l2_perception` / `l3_embodiment` / `l4_emotion` / `l5_cognition` / `l6_meta`，详见 `docs/architecture/ARCHITECTURE.md`。
- 编码标准见 `RUST-STANDARDS.md`（生产代码禁 `unwrap`/`expect`/`panic!`，错误用 `?` 传播）。

## 硬规则

- `#![forbid(unsafe_code)]` —— 永不加 `unsafe`。
- 编辑后必须重读文件验证落盘（R-P16）。
- 外部技术必须同会话接到生产可用（R-P79）。

## 微操作公约（Agentation 思想吸收，skill: nt-locate）

- 改码前先定点：`python3 scripts/ops/nt_locate.py --component X --source-file Y`（选择器→文件:行），读上下文再下刀，无定点不改。
- 闭环：点选/标注 → 定点 → 最小改动 → 单测验证。
- **下刀前查并发**：`git status --porcelain <file>` 看他窗改动 + `stat -f "%Sm"` 看
  mtime 是否在数秒内变过。本轮撞见另一窗口正在用 python 改同一个文件 ——
  `mtime` 比检查时间只早 12 秒。**mtime 近期变动 = 他方在写，换文件或先通报。**
  （`nt_locate.py` 的索引可能陈旧：`--audit` 显示 3 天前、476 缺失时改用 grep 定点）

## 正典索引

- **唯一图纸入口**：`docs/architecture/NEOTRIX-MASTER-BLUEPRINT.md`（D-00~D-15，按图施工）
- 架构现状：`docs/architecture/ARCHITECTURE.md`（⚠️ §1-§12 的 C4/分层设计已被 §13 neobot 融合推翻，读 §13 起的实测部分）
- 模块拓扑实测：`docs/architecture/DIR-AUDIT-2026-09-27.md`（16 包依赖图 + 8 类重复类型）
- **目录架构解法**：`docs/architecture/DIR-REMEDY-2026-09-28.md` —— `neotrix-core/src/neotrix/`（129 文件/43,834 行）是**不参与 L0–L6 的第二棵树且**完全逃过 `check-layer-deps.sh`；解法是**层归属显式化**（`.neotrix/layer-map.json`）而非搬目录。**其 §2.5 记录：`nt_jev` + `nt_crystal_core` 是活路径（L1 有 6 个消费者），勿当死代码删** —— 「导出 ≠ 调用」已错过 3 次
- 外部吸收与进化路线：`docs/architecture/ABSORPTION-EXTERNAL-2026-09-27.md`、**`EVOLUTION-ROADMAP-CODE-NODES-2026-09-28.md`（正典）**；09-27 版已归档至 `_superseded/`
- 文档规范：`DOCUMENTATION-MAP.md`（目录导航以 `docs/architecture/README.md` 为准）
- 模块台账：`ARCHITECTURE-MAP-ROADMAP-V2.md` —— **其 §1-§7 数字自述永久陈旧，只取 §11 起的可再生实测值**。台账更新规则是 **R-P199**（`docs/standards/archive/dev-rules-legacy-R-P161-257.md:259`，**非规范副本**；"R-P161-257" 只是该归档文件的编号区间，不是规则号），口径是 **`neotrix-core` 的 L1–L6**；`crates/neotrix-neobot` 是独立 crate、不占 L 层 ⇒ 本轮**不进**此台账，正典记录见 `docs/architecture/ABSORPTION-DSH-SIDEBAR-IM.md`
- 已废止：`FUSION-ARCHITECTURE.md`（其"下一步"含已被证伪的"解决预存编译错误"）、`ARCHITECTURE-EVOLUTION-ROADMAP.md`（零引用）
- 待办：`TODO.md`（顶部为人工摘要区）；事故与分诊：`sessions/handoff-disease-list-20260927.md`（模板 `sessions/HANDOFF-TEMPLATE.md`，两者均已入库）
- **新窗口统一修复的**开头提示词**：`sessions/handoff-20260928-new-window-opening.md`（2026-09-28，接手前先读它 —— 含必读文档顺序、三条硬约束、以及「主工作树不是可信地面真相」这一最容易浪费数小时的前提）
- **本轮目录/算法改造的**交接**：`sessions/handoff-20260928-algo-extraction.md`（8 项算法萃取进 L0–L6 + 6 个目录归档 + 2 笔债裁决 + 删 4,640 行死引擎；含 5 个踩坑坑位与「反查消费者须一并 grep `.github/workflows/`」的方法论教训）
- **剩余任务汇总交接**：`sessions/handoff-20260928-consolidated.md`（2026-09-28，**动手前必读其 §2 勘误表** —— 4.1/2.1/2.2/5.2/4.4 的台账前提均已被实测证伪，照原文做会重造已存在的东西或"修"已正确工作的机制）
- **本轮方法论教训**：`docs/architecture/LESSONS-20260928-fresh-checkout.md` —— 「本地全绿但仓库不可交付」的完整解剖。**元教训：任何「X 是好的/坏的」断言都要问「我是在哪个环境里验证的」；答「我的工作树」就等于还没有证据**

## 三道闸（2026-09-27 事故后置入，违反即阻塞）

- 重型 cargo 前：`sh scripts/ops/nt_mem_gate.sh; echo $?` — 非 0 禁止起构建
- 死锁静态扫描：`python3 scripts/ops/nt_lock_audit.py neotrix-core/src`
  — 门记录（**每次改代码后必须刷新，禁止沿用旧值**）：
  - 2026-09-27 **22:52** 实测 3 条（1 真死锁 `kb_search.rs:549` / 2 误报）
  - 2026-09-27 **23:4x** 复测 **0 条**（真死锁已修 + 扫描器已补 `drop()`/临时锁识别）

  - 2026-09-28 **14:45** 复测 **0 条**（`neotrix-core/src` 2554 `.rs` + `crates/neotrix-neobot/src` 48 `.rs`，两次退出码均 0；**本轮仅改文档未改码，故沿用同值**）
  - 2026-09-28 **15:5x** 复测 **0 条**（`neotrix-core/src` 2554 `.rs` + `crates/neotrix-neobot/src` 48 `.rs` 同为 0，退出码 0；`apps/neobot-desktop/src` 亦 0；本批 /stop 接线后重跑）
  - 2026-09-28 **15:2x** 复测 **0 条**（同上两处）。改的是门不是产品码（2 个脚本：`.sh`/`.json`/`.txt`，非 `.rs`），故沿用同值。**`bdf1e9f1` 已把 `scripts/check-naming.sh`（新建）+ `check-layer-deps.sh`（纳入 `.neotrix/layer-map.json`）+ 重算的 102 基线入库** —— 此前它们只在主树未入库，导致新 clone 的分层门**完全看不见第二棵树却报 PASS**
  - 2026-09-28 **17:16** 复测 **0 条**（`neotrix-core/src` + `crates/neotrix-neobot/src` 两处均 0，退出码 0）。**本轮真的改了产品码**（`llama_process.rs` / `model_pool.rs`），故为重跑非沿用。同轮 `cargo check --all-targets -p neotrix` = **0 error**（HEAD `ecd10d3e` 原本有 3 个 error，在 `examples/v2_quick_start.rs`）
  - 2026-09-28 **18:1x** 复测 **0 条**（`neotrix-core/src`，退出码 0；本轮 8 个算法萃取进主代码 + 删 4,640 行死引擎）。同轮 `pre-commit` 的 `cargo check --tests` 全仓仅 1 error，**属并行窗口未提交改动**。详见 `sessions/handoff-20260928-algo-extraction.md`
  - 历史教训：22:52 之前本文长期写着"当前 0 命中"，而实际是 12 条 —— **陈旧门记录会让下一个 agent 去"修"正确代码，比没有门更危险**（见 R-SCAN-3）
- sidecar 按需：`sh scripts/ops/nt_sidecar.sh {start|stop|status}` — 用完即停
- 目录/命名门（2026-09-28 新增，均 bash，无需 cargo）：
  - `bash scripts/check-layer-deps.sh --strict` → **exit 0**，**102 known**
    （= 92 条 `l*_` 层 + 10 条 `neotrix/` 第二棵树）。**测量台：`git worktree add
    --detach HEAD` 的干净检出**，`bdf1e9f1` 15:2x 实测。⛔ 此前记录的「94」是
    **主工作树（脏）**测量 —— 那 8 条 `l*_` 之所以消失只因主树有未提交的 `.rs`
    修复；照抄 94 会让 CI 以 `FAIL: 8 new` 红。可交付态的真值是 **102**。
  - `bash scripts/check-naming.sh` → advisory，打印 **1,646** 个无 `nt_` 前缀文件
    （**规约 vs 现实差 1,646 ⇒ 规约无约束力**）。干净检出实测；此前的 1,644 同为脏树值
  - 层归属真源：`.neotrix/layer-map.json`（**`bdf1e9f1` 已入库**）；裁决表：`docs/architecture/OWNERSHIP.md`（随车道并入本提交入库）
  - ⛔ **`truth-surface` 本地红不是 CI 红**：他窗 WIP 造成 UNCOMMITTED_DEP；干净检出实测 exit=0
- 硬规则细则见 `RUST-STANDARDS.md` §17（锁/构建/卡死判别/Git/修 bug 判据/字节安全）

## 本地模型 — `docs/architecture/LOCAL-LLAMA-2026-09-28.md`

权重 `<repo>/models/` ＋ 归档区兜底（均 gitignored，git 保护不到）。
启动 llama.cpp **必须**带 `--jinja` `--reasoning off` `--ctx-size <N>` 显式值，
否则 Qwen3.5 系「装完开不了话」。原因、KV 推导、实测数据见该文档。

## 扫描器告警 ≠ 缺陷（2026-09-27 差点把 bug 修进正确代码）

- **R-SCAN-1 静态扫描告警必须先读现场证实或证伪，再动代码。** 本轮
  `nt_lock_audit.py` 报 12 条，逐个读代码后 **2/3 是误报**：`tor_client.rs:324`
  作者已显式 `drop(proc);`；`llama_process.rs:329` 的 `*self.x.lock().await = v;`
  是赋值型临时锁（`;` 处即释放）。若照单全修，会把 bug 引进**正确**代码。
  这是 `nt-locate` "无定点不改"对扫描器同样成立。
- **R-SCAN-2 手推 ≠ 实证。** 判断扫描器行为必须把**真实代码形态**喂进去跑，
  不要在脑子里模拟 —— 本轮手推 `audit_indirect` 不会误报，实测它确实误报。
- **R-SCAN-3 门记录必须带核实时间戳。** 本文与 `RUST-STANDARDS.md §17.1`
  都曾长期写着"0 命中"，而实际是 12 条 —— 陈旧的门记录会让下一个 agent
  **去"修"正确代码**，比没有门更危险。
