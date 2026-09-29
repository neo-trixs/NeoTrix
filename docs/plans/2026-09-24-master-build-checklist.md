# neobot 完整实施任务清单（2026-09-24，总控）

> 三线合一：B 线（Tauri 构建）+ U 线（Team AI UI）+ D 线（进化核心）。
> 门：每项零绿不进下一项；内存窗内按序执行；邻窗文件（frontend/src 双改区）开工前喊一声。
> 工具链已验：node 25.9 / npm 11.12 / rustc 1.94 / tauri-cli 2.11.1 / frontend node_modules 在位。

## Phase 0（内存窗开，约 30min）

| # | 线 | 任务 | 命令 | 门 |
|---|---|---|---|---|
| 0-1 | B1 | 图标重生（8 件，icns/ico 并出） | `tauri icon src-tauri/icons/icon.svg`（缺 svg 则用 Square256x256Logo 派生） | bundle 段 8/8 在盘 |
| 0-2 | B2 | 前端 build（tsc+vite） | `npm ci && npm run build`（frontend/） | dist 零错 |
| 0-3 | B3 | `cargo check -p neotrix-tauri` | `cargo xl` 级别单包 | 0 error（pedantic 全量 lint 随带） |

## Phase 1（U1–U3.5，纯前端+薄命令，约 2h）

| # | 线 | 任务 | 验证 |
|---|---|---|---|
| 1-1 | U1 | 作曲区（同键+草稿+pills+私队锁） | 停后草稿在；锁默认队 |
| 1-2 | U2 | 转录体 + hero 三选一 + token 对比表 | 焦点≤3；AA；截图门 |
| 1-3 | U3 | NtTask 全域 ID（+claimed_by/visibility） | 三处同变 |
| 1-4 | U3.5 | RosterBar + 原子认领 + presence（5 IPC：roster_list/presence/task_claim/release/cost_ledger/member_set_visibility + 4 事件） | 双开抢认领被拒显 holder |
| 1-5 | — | verify-glass + e2e（mock IPC） | 全绿 |

## Phase 2（D 核心 + B 打包，约 3h + 训练窗）

| # | 线 | 任务 | 验证 |
|---|---|---|---|
| 2-1 | D | eval 验收 + 权重封存（ftv6 落盘触发） | eval_v2.done |
| 2-2 | D | bf16 smoke → D3 MLX LoRA 首版（mask-user 开） | eval_lora + D5 |
| 2-3 | D | D1 实现（深特征 + Graft 格式 + 10 组对照） | 对照 7/10 |
| 2-4 | D | D4 L1/L2（规则裁判 + surprise） | 有用占比口径 |
| 2-5 | B4–B6 | `tauri build`（dmg）+ e2e 真后端 + CLI 7 命令 parity | 可安装启动；EC:0×7 |

## Phase 3（本周：治理+市场+收紧）

| # | 线 | 任务 | 验证 |
|---|---|---|---|
| 3-1 | U4 | 网关 actor 上下文 + fail-closed 演练 + take-the-wheel | 坏规则拒行 |
| 3-2 | B7–B8 | ntos 四件并入 + market 接 skills intake | 功能对等；可装下沉 skill |
| 3-3 | D | D2 插桩（helper_use_log + registry 落盘 + banthis） | 首周报 |
| 3-4 | D | 检索升级 + NT-BROWSE 四件 + 流程件 | 各自门 |
| 3-5 | B9–B10 | failover 接线 + capabilities 收紧 + CSP 复核 | 断网演练；shield 验 |

## 回滚律

- 图标/dist/target 产物：git 可丢弃 + 可重生，不进备份。
- store 键变更：`ntos_*_v1` 只增不改（新键新版本，后缀 _v2），parse 容错保旧数据。
- Rust 改动：单包 check 先行，全量 build 随后；邻窗 cargo 活着时只读。
