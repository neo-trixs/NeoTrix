# Handoff：自研浏览器内核 + Lingee 收割（2026-09-23 早）

## 1. 会话标识

- 分支：`feat/capability-absorb-20260828`
- 经验 cycle：`nt0923browse`（12 branches 已入库，可 `query` 检索）
- 交接时间：2026-09-23

## 2. 已完成（验证状态附后）

- Lingee 收割：`/work` 网关 + 11 档模型 + 89 agents + 266 skills；文档 `docs/plans/2026-09-22-lingee-*.md` 3 篇 + `repo-analyses/lingee-20260922/`（已脱敏）。
- 内核 `nt_io_browser_engine.rs`（~4400 行）：Mock/Http/ChromeHeadless/Cdp + 29 动作 + AuthManager + auth.toml 纳管 + 治理（allowlist/SSRF/预算/TTL/审计）+ VerifyBlock + 429 冷却；隔离 crate **39 单测全过**（`nt_verify2`，含线上 Lingee 实测）。
- `run_browse` 切 Http 内核（生产二进制实测过）；`session.rs` 去 profile + 全链路 `--use-mock-keychain`（钥匙串弹窗根因）。
- sudo 弹窗案结案：Battery 维持循环 + sudoers 缺 CH0B 读规则；用户已卸载，残留清理命令已给，蹲守脚本 `/tmp/nt_sudowatch.sh`。
- Token 已迁 `~/.config/neotrix/lingee.token` + `auth.toml`（均 600）；`/tmp` 敏感拷贝已 shred。

## 3. 未完成（按优先级）

1. **入库 `cargo check`**（含 CDP）：树一直被他窗破坏 + 构建锁排队；后台循环已死（8 轮耗尽），需重起。唯一我们文件的错（E0004）已修。
2. **重编二进制 + 活体验证**：排在 1 之后。
3. **browser 窗口 3 处缺参**：已喊话 `sessions/handoff-to-browser-window.md`，**不代修**。
4. **能力树 bud**：注册表老 schema，CLI 拒绝写入。
5. **P2**：确认门 / a11y 快照 / vision 兜底 / 按域记忆 / daemon 化 / 密码全自动续期（需 owner 拍板才存密码）。
6. **9/29 token 过期**：改文件即恢复。

## 4. 正在改的文件

| 文件 | 状态 |
|---|---|
| `neotrix-core/src/l1_action/nt_io/nt_io_browser_engine.rs` | 我的，可提交（待 check） |
| `neotrix-core/src/l1_action/nt_io/nt_io_auth_store.rs` | 我的新建，可提交 |
| `neotrix-core/src/l1_action/nt_io/mod.rs` | 我的 2 行，可提交 |
| `neotrix-core/src/l2_perception/nt_world/crawl/session.rs` | 我的 8+1 行，可提交 |
| `neotrix-core/src/entry/mod.rs::run_browse` | 我的，但他窗在重写此文件，提交前先 `grep BackendKind` 确认在位 |

## 5. 禁止事项

- 不跑 `--all-targets`（16G 爆 swap）；`cargo check/test` 全部后台 + 日志排队。
- 不碰 `skill_loader.rs` / `skill_evolution.rs` / `nt_tui_app.rs` / `mcp_protocol/`（他窗在改，挡 check/test）。
- 不代修 browser 窗口的签名（owner 令）。
- token/PII 永不进仓；`~/.neotrix/pending-absorb.json` 是别窗的（wsd0922b），别覆盖。
