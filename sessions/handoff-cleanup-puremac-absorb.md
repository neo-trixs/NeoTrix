# Handoff — cleanup-puremac-absorb

## 1. 会话标识

- 窗口：主窗口（opencode PID 5160）
- 分支：`master`（工作区有大量未提交改动，非本会话独有）
- 交接时间：2026-09-23 16:35

## 2. 目标（一句话）

> 吸收 PureMac 清理能力到 NeoTrix `nt_act_cleanup` 并接 CLI，同时完成系统/项目垃圾清理与对话慢优化。

## 3. 已完成

- [x] 对话慢 4 层原因分析（多窗口 / 单体 crate / 脏文件 / 免费模型）
- [x] `Cargo.toml` profile：dev `debug = "line-tables-only"` + deps `debug = false`（已重读核验 R-P16）
- [x] `.cargo/config.toml`：`jobs = 5`；`xl`/`xt`/`xc` 收窄为 `-p neotrix --lib`
- [x] `cargo clean` 释放 ~45.8 GiB；磁盘 86%→80% 区间；swap 后续再压
- [x] 项目垃圾：.DS_Store、空 node_modules、swap/backup、`repo-analyses/orca-20260920`(376M)、git gc
- [x] 系统垃圾第二遍：npm `~/.npm` 901M→40M、Chrome `code_sign_clone` 5.6G→0、sysdiagnose、DiagnosticReports 旧 ips、TMP>7d、oh-my-opencode>30d
- [x] PureMac 调研：`CleaningEngine` / `ScanEngine` / `CleanupExclusions` / `Locations` / CLI `Catalog`+`Clean`+`Safety`
- [x] **新吸收模块**（全部落盘）：
  - `nt_act_cleanup/locations.rs` — allow-list + provider/credential 拒绝根
  - `nt_act_cleanup/exclusions.rs` — 用户排除表持久化
  - `nt_act_cleanup/safety.rs` — symlink 分量 + 凭据 + cloud + TOCTOU revalidate
  - `nt_act_cleanup/catalog.rs` — dev/junk/ai/trash 目标表
  - `nt_act_cleanup/scan_engine.rs` — 分类扫描 → `CleanableItem` + 去重
  - `nt_act_cleanup/cleaning_engine.rs` — dry-run / allow-list / TOCTOU 删除
  - `nt_act_cleanup/mod.rs` — 模块注册与 re-export
- [x] CLI 接线：
  - `neotrix-core/src/entry/clean.rs` — `run_clean`（flags/exclude/json/force）
  - `neotrix-core/src/entry/mod.rs` — `mod clean; pub use clean::run_clean`
  - `neotrix-core/src/main.rs` — `Commands::Clean` + `is_ops_cmd` + dispatch

## 4. 正在改的文件（关键）

| 文件完整路径 | 改到什么程度 | 是否可独立提交 |
|---|---|---|
| `neotrix-core/src/l1_action/nt_act/nt_act_cleanup/{locations,exclusions,safety,catalog,scan_engine,cleaning_engine,mod}.rs` | 新写完，**未经 cargo 编译验证** | 否，需 `cargo xl` + 单测 |
| `neotrix-core/src/entry/clean.rs` | 新写完，未编译验证 | 否，依赖上面 + main |
| `neotrix-core/src/entry/mod.rs` | 已挂 clean | 否，需与 clean 一起过 |
| `neotrix-core/src/main.rs` | 加 Clean 命令三处 | 否 |
| `Cargo.toml` / `.cargo/config.toml` | 本会话早前优化，已核验 | 是（注意并行窗口回滚史） |

## 5. 下一步（按优先级）

1. **等其他窗口 `cargo test -p neotrix --lib`（PID 29051）与 tauri 构建结束后**，跑：
   ```sh
   CARGO_BUILD_JOBS=2 cargo check -p neotrix --lib
   CARGO_BUILD_JOBS=2 cargo test -p neotrix --lib -- nt_act_cleanup
   ```
2. 修编译错误（预期风险：clippy pedantic / `deny(warnings)` / `IsTerminal` 版本 / 未用 import）
3. 验证通过后 `target/debug/neotrix clean --dry-run --json`
4. 决定是否提交（仅本会话相关路径；工作区有大量他人脏文件勿误提交）

## 6. 阻塞点

- **swap 高压**：交接时 `vm.swapusage used≈7.9G/9G`；6 个 opencode 窗口仍在（1957/2240/2428/2594/2802 + 本窗 5160），用户应手动关多余窗口
- **cargo 锁竞争**：另一进程在跑 `cargo test -p neotrix --lib -- nt_approval…` 与 `src-tauri` 构建，导致本会话两次 `cargo xl` 超时/中止
- 编译验证未完成 → 新代码**不可宣称已通过**

## 7. 给接手会话的话

- 恢复命令：先读本文件 → `git status --short` → 只 `cargo check -p neotrix --lib`
- 禁止：`cargo check --all-targets` / 多窗口同时全量构建（AGENTS.md 并行公约）
- 风险：`main.rs`/`entry/mod.rs`/`Cargo.toml` 可能与其他窗口改动重叠；写前重读（R-P16）
- PureMac 源仍在：`/var/folders/jr/.../T/opencode/puremac`（MIT，只读参考）
