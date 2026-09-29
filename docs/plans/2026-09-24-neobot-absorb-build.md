# ntos + src-tauri 吸收 → neobot 完整构建单（2026-09-24）

> 范围：`ntos/` 全目录（SolidJS 壳，6 源文件）+ `src-tauri/` 全目录（Rust 后端 60+ 文件 +
> React 前端）。读法：核心文件全文，余部扫结构+注释。R-P79：本报告即接线索引，
> 代码改动排内存窗（`tauri build` 需前端工具链+全量编译）。

## 1. ntos 吸收（UX 参考实现，`ntos-shell@0.8.0` SolidJS）

`store.ts`（486 行已全读）是**唯一状态源模式**的教科书，可直接移植到 neobot 前端：

| 模式 | 内容 | 移植点 |
|---|---|---|
| 会话/消息/草稿三键 | `ntos_sessions/msgs/drafts_v1` + 读写容错（parse 失败回默认，绝不崩） | 前端 store 照抄 |
| AI 队友 | `NtTeammate{job/model/temp/joinAll}` + `@全体` + 8 字名钳制 + 非法条目过滤 | 多人格同聊（OpenExecutive 思想的本地版） |
| 提示词库 | builder 式 starter（规划/报告/周报/总结/翻译/生图），可增删替硬编码 chips | 前端 chips 换掉 |
| **任务中心** | `NtTask{status/targetId/mateNames快照/tokens}` + 重试（原输入+目标回退）+ 上限 50 + **重启 running→cancelled（防卡死）** | ActivityLog 直接用 |
| 长期记忆 | 单串注入 system prompt（OpenMuse personal context 同构） | 开机 personal 上下文 |
| 备份 | `exportAllData`（version:2）+ `parseBackup`（app 校验+逐段容错）+ `applyBackup`（confirm 后写） | 设置页备份/迁移 |

## 2. src-tauri 后端吸收（`neotrix-tauri`，Tauri 2 + forbid(unsafe)）

- **命令五面**（`commands/`）：domain_cmd（程序化 API）/ chat（NL 入口）/ pty（硬件级）/
  neobot（CLI 薄封装）/ ntos（独立窗）。**架构定论：neobot 保持薄封装，不含业务逻辑；
  长任务 `spawn_blocking`，DTO 脱敏。**
- **模型路由**（`service/`）：circuit_breaker（CLOSED/OPEN/HALF-OPEN+指数退避+按 provider 隔离）
  + provider_manager（注册/健康/按 task_type failover 链）+ cost_tracker。
  → HttpEngine 下一步就接 failover 链（见 §4 单）。
- **市场引擎**（`market/`）：dsh/github 双 discoverer（search/detail/releases/download/check_updates
  全套）+ schema + atomic_io 落盘。→ **skill intake 的生产形态**（master-schedule skill 批量引的家）。
- **桌面件**：session_manager / model_manager / universal_ui / capabilities；
  chat/router+response；pty manager（spawn/write/resize/close）；anthropic client；
  db_pool（rusqlite bundled）；recovery；masking；agent_identity；notifications；
  ntcode 默认对话模型；vault/validated（私有）；原生 macOS 菜单（Cmd+K 面板/Cmd+N/Cmd+,，
  ⌘W 特意让路给会话删除——细节有考据）。
- **插件面**：tray/global-shortcut/single-instance/autostart/updater（含 releases 端点+公钥，
  发布基建现成）/shell/dialog/http/fs/deep-link/notification + keyring（密钥！）+
  portable-pty + arboard（剪贴板）+ sentry。结论：**常驻 agent 的 OS 位全齐**。
- **安全注记**：capabilities fs 读写放 `$HOME/**`（宽！跟进收紧，shield 单）；
  CSP 白名单含 jsdelivr/fonts/localhost（dev 残留，build 前复核）。

## 3. 前端吸收（React + tailwind + playwright）

- `api/harness.ts`：统一网关 SDK，**三阶段流式事件**（allocated/step/done，step 带 internal/external
  kind）——D1 路由可视化的现成协议。
- `api/`：domain / im / proxy-pool / index；`canvas/SmartCanvas+evolution`；
  组件：ActivityBar/Log、ApprovalPanel、AutonomyMeter（审批+自治仪表的 UI 对应物）。
- `verify-glass.mjs`：**mock IPC 的视觉门**（playwright + `__TAURI_INTERNALS__` 桩）——
  无需后端即可验 UI，前端独立迭代的钥匙。
- e2e（playwright.config）+ CHANGELOG（前端独立版本纪要，学例）。

## 4. P0 缺陷（构建必红项，实测）

`tauri.conf.json` bundle.icon 引用的 **8 个图标缺 5 个**：
`icon.png`、`128x128.png`、`128x128@2x.png`、`256x256.png`、`32x32.png` 全 MISS
（在盘的只有 Square*Logo 系 + icns/ico/svg）。`tauri build` 打包必红——这就是"Tauri 红"的
同类项。修：`npx tauri icon icons/icon.svg`（或从 256 Logo 派生），需前端工具链 + 内存窗。
（另：frontend/icons 删掉的 2 个 png 无 manifest 引用，属 PWA 残留，无害。）

## 5. neobot 完整构建单（按序，内存窗执行）

| 序 | 步骤 | 验证 |
|---|---|---|
| B1 | 图标重生（`tauri icon`，8 件补齐，icns/ico 一并重出） | bundle 段无 MISS |
| B2 | 前端 `npm ci && npm run build`（tsc+vite） | dist 产出零错 |
| B3 | `cargo check -p neotrix-tauri`（lib forbid/pedantic 全量 lint） | 0 error |
| B4 | `tauri build`（dmg） | 产物落盘，可安装启动 |
| B5 | e2e + verify-glass（mock IPC 先行，真后端随后） | 截图门过 |
| B6 | neobot CLI parity：`run/doctor/task/audit/models` 逐个过 | 7 命令 EC:0 |
| B7 | ntos 模式并入前端（任务中心/队友/提示词库/备份四件，§1 表） | 功能对等清单 |
| B8 | market 接 skills intake（dsh/github discoverer 复用） | 下沉 skill 可装 |
| B9 | HttpEngine 接 provider failover 链 + 菜单/快捷键复核 | 断网切换演练 |
| B10 | capabilities 收紧（`$HOME/**` → 最小集）+ CSP 复核 | shield 单验收 |

## 6. 榨出的价值经验（跨项目复用）

1. 薄封装命令层 + DTO 脱敏 + 长任务 spawn_blocking（neobot.rs 范式，凡 IPC 照抄）。
2. 重启 running→cancelled（任何持久化状态机照抄，防卡死第一律）。
3. 备份三段式（export version 化 + parse 容错 + apply 前 confirm）。
4. mock IPC 视觉门（前端不被后端卡住的钥匙）。
5. 发布基建已存在（updater 端点+公钥+多目标 bundle）——只差可装的包。
6. 金手指纪律：ntos 证明"小而全的本地壳"跑得通，neobot 不要贪大（Dark Forest）。
