# agent-browser 吸收报告（2026-09-22）

> 象限: Explanation | 对象: `vercel-labs/agent-browser`（~39k★, Apache-2.0, Rust CLI + TS）
> 本地快照: 隔离区（未进仓）| 吸收落点: `nt_io_browser_engine.rs`（+6 动词）

## 1. 架构（抄大思路）

- 瘦 CLI → JSON 协议 → **常驻 Rust daemon** → CDP 直驱 Chrome（Chrome for Testing 渠道，`install` 自举，无 Playwright 依赖）。
- 快照 = **a11y 树 + `@eN` 紧凑引用**，点击/填充按引用不按选择器（抗改版）。
- CLI 自带 skill 内容服务（`skills get core`，版本自洽）+ 4848 observability dashboard + 会话/金库/录像。
- 对照结论：我们的 Http/CDP 双后端拆分与之同构；差的是**常驻 daemon**（我们是进程内引擎，CLI 跨调用无会话）和 **a11y 快照**。

## 2. 动词映射（`cli/src/commands.rs` 6721 行 → `BrowserAction`）

| agent-browser | 我们 | 落点 |
|---|---|---|
| open/navigate, snapshot, get text/links | Navigate/GetContent | 已有 |
| click/dblclick/hover/focus/check/uncheck/select/fill/type/press | Click/Type/FillField | 已有；**+SelectOption/+Check/+Uncheck/+Press**（本轮） |
| get text @ref（定点提取） | — | **+GetText**（本轮，快照子集/innerText，省 token） |
| pdf | — | **+PrintPdf**（本轮，`--print-to-pdf`/CDP save_pdf） |
| upload/download, dialog accept/dismiss, frame, window, clipboard, console/errors, trace, state save/load, auth vault, launch/stream | — | 缺口：upload 需 multipart/file-input；console 需 CDP 事件订阅；daemon 化后做 state/auth（P2） |
| evaluate | ExecuteJs | 已有（CDP 真求值，dump 管道诚实报错） |

## 3. 本轮 patch（R-P42 吸收加强，既有节点）

- `BrowserAction` +6：SelectOption/Check/Uncheck/Press/GetText/PrintPdf。
- Http：下拉/勾选走待填值（name 键），GetText 走快照，PrintPdf 诚实报错。
- Chrome：PrintPdf 真打印（`--print-to-pdf --no-pdf-header-footer`），余者回放 Http 并标注。
- Cdp：真设置+change 事件（JSON 转义防注入，纯函数可测）、条件 click 勾选、press_key、innerText、save_pdf。
- 单测 +6（JS 构造器转义、勾选待填、GetText 空页、PrintPdf 拒识），隔离 crate 21/21。
- 未抄：AGPL 无关（对方 Apache-2.0，可放心）；daemon/a11y/密码库列 P2。

## 4. 验证状态

- 隔离 crate（同版本依赖）：`cargo test` 21/21；`example.com` 端到端沿用此前结论。
- 入库 `cargo check`：等构建锁（后台循环重试，`CHECK-PASSED` 即合龙）。
- 入库 `cargo test`：仍被他窗 `skill_loader.rs:660` 挡住（未碰）。

## 5. 第二批：治理与验证融合（2026-09-23，多源情报）

情报：browser-use 100k★（Rust core + DOM 蒸馏 + SoM + O-P-A-V + 看门狗）、tsaagan（verify-first + API-first + 按域记忆 + OS 钥匙串 + isTrusted）、chrome-bridge（fail-closed 策略 + 审计脱敏 + waitForHandoff）、CapSolver 五层（准入/运行时/状态/挑战/证据，403≠429≠captcha）、pi-agent-browser（commit 确认门）、Naïve（Vault 隔离 + allowlist 默认拒绝 + TTL/credit 上限）。

落地（`nt_io_browser_engine.rs`，隔离 39/39）：
- 治理：`allowed_domains`（默认拒绝）+ SSRF 常闭（单 choke 点 `polite_wait`，四后端全覆盖）+ `max_actions_per_session`/`session_ttl_secs` + `AuditEvent`（无正文）+ `HistoryRetention::MetadataOnly`。
- 验证：`VerifyBlock{url_changed,http_status,text_len,links,forms}` 随动作同行返回（免二次快照）；`PageSnapshot.http_status`。
- 信号分类：429 → `RateLimited(secs)` + 域名冷却（`CoolingDown` 期内零请求），401/403 走认证通道，互不混淆。
- 未做（诚实缺口）：确认门 ConfirmRequired、a11y 树快照、vision  grounding、按域记忆——列 P2。
