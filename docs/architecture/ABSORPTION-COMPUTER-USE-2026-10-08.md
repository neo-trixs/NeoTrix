# 吸收：`freeall12/computer-use` → neobot computer 能力 + bot app 界面（2026-10-08）

> **入参**：裸 URL + 用户说明「吸收完善 neobot app 对应的能力，对标 grok bot 等吸收的 bot app，完善界面」。
> **依据**：`NEOTRIX-STD-1.0.md` NTS-B10 · 操作面 `skills/external-absorption/SKILL.md` URL-only 熔炼模式。
> **取法**：零克隆；取 README 能力矩阵 + `reusable/patterns.md` 的 20 条模式标题/要点；**未取逐字代码**。

## 0. 信号初筛

| 字段 | 值 |
|---|---|
| stars / forks | **0 / 0**（2026-10-05 新建，三天） |
| license | **MIT**（raw 首行核实） |
| language / size | TypeScript / 2.4 MB |
| pushed_at | 2026-10-07（活跃） |
| topics | agent-tools, browser-use, computer-use, cua, mcp, xai(grok), ui-automation … |
| 性质 | **只读静态逆向图谱**：20 个 agent（12 coding + 4 浏览器 + 2 SDK + 2 GUI 框架）× CU/BU 实现，含 80 分册路径:行号证据 + 能力矩阵 + 20 可复用模式 |

⚠️ **记录真伪**：0★、个人逆向笔记；许可 MIT 允许取文本，但**证据链来自 macOS 静态分析**，我方采纳的是**模式层结论**，不是事实断言；任何数字（如工具数、安全层级数）都须回源核对后才能写进我方文档正文。

## 1. 熔炼成束（上下文工件）

| 束 | 取到的形状 |
|---|---|
| 能力矩阵 | 执行基线四型（本地 Helper / 全云执行+本地投影 / 执行层外包 / 无原生能力）；浏览器三载体（内嵌 IAB / 浏览器即载体 / 云端）；安全光谱 0→10 层 |
| 谱系 | **Grok Bot 桌面端 = Anysphere（Cursor 母公司）代工换牌**（TeamID 同 Cursor、`CUCursorService` 残留）；Claude/Cursor/MiMo/Codex 关系 |
| 20 模式（P1…P23） | P1 独立 Helper 进程 · P2 a11y 优先+视觉兜底 · P3 元素句柄/防漂移 · P4 后台定向输入不抢焦点 · P5 剪贴板 paste/setValue 分层 · P6 控制租约+generation fencing · P7 防重放 possibly_sent + kill switch · P8 浏览器架构六格 · P9 MCP 通用挂载（未启用=工具不存在）· P10 审批分级与域白名单 · P11 fail-closed 注入 |
| 共识四件套 | 审批分级 / 控制租约 / 防重放 / **verify-after 回读**（三态）；失败策略两端：Claude fail-open vs Synara 十层 |

## 2. 化为已有（四字段矩阵 · 人工 grounding）

| Source | Pattern | NeoTrix/neobot 映射节点 | 判定 | 消费者 |
|---|---|---|---|---|
| P2 + P3 | 观察-动作-再观察 + 句柄防漂移 | `nt_computer`（11 声明/8 执行）+ 网关 resolve→policy→audit→execute | **强化候选**（动作面与网关已齐；**没有 observe/句柄台账** ⇒ 漂移校验无处落） | `nt_computer.rs` `ComputerBackend` trait |
| P7 | 防重放三态（已派发/效果未知）+ kill switch | `nt_computer` **无**（错误只回 `NtBotError`） | **新增候选**：动作回执需 `action_sent` + `outcome_unknown`（我方 tasks 侧已有该语义，尚未接到动作面） | `nt_computer.rs` + `nt_agent` 工具回执 |
| P6 | 控制租约 + generation fencing | 无 | **新增候选**（多会话并行时必需） | `nt_store`（新表）或复用 `tasks.lease_id` |
| P10 | 审批分级 + 域白名单，拒绝带 `decisionSource` | `nt_policy`（fail-closed）+ 禁区概念 | **强化** | `nt_policy.rs` / `nt_workspace` jail |
| P9/P11 | MCP/工具注入 fail-closed（未启用=工具不存在） | `nt_capability_market` + 门控 | **强化** | capability 注册面 |
| P8 | 浏览器载体六格 | neobot 无内置浏览器；core 有 `nt_world_crawl::BrowserCircuit`（chromiumoxide） | **记录**：neobot 侧维持「外挂/不内嵌」定位，避免第二浏览器栈 | — |
| P1 | 独立 Helper 持 TCC | 我方零辅助进程（unix-only 且 crate 层禁 unsafe） | **不做**（架构不合） | — |

## 3. 对标 bot app 的界面差距（Grok Bot / Stagehand / Synara）

| 能力 | 对标做法 | 我方现状 | 差距判定 |
|---|---|---|---|
| 动作回执三态 | ZCode/Claude：`actionSent`+建议；Grok「错误即指令」16 码→四档建议 | `NtBotError` 一档 | **缺 UI 语义层**：需把错误映射为「可重试/需授权/已停止/效果未知」四档 |
| 授权确认卡 | Synara 十层 / Codex 四层确认；MiMo 1–20s 一次性租约 | `nt_policy` 有 approve/deny 决定，无**租约倒计时** | **缺**：租约倒计时条 + 一次性授权 |
| 全局急停 | 20 家共识：物理 Esc / kill switch（豁免自身与状态查询） | 无 | **缺**：Desktop 侧全局 kill switch（我们已有 `nt_cancel` stop token ⇒ 需一个「全停」聚合） |
| observe/回读 | verify-after 三态 + 语义树优先 | 无 observe 面 | **缺**：动作后回读结果的对比视图 |
| 后台不抢焦点 | P4 四路线 | 零（无浏览器控制） | 暂缺，随浏览器载体一起排期 |
| 权限域面板 | Grok per-origin cookie 审批、URL 禁区 | `nt_policy` 域规则 + workspace jail | **有基础**，缺 UI 呈现 |

## 4. 落地分期（可执行序列）

**A. 动作回执语义层（本周可做，小）**
1. `nt_computer` 新增 `ActionReceipt { sent: bool, outcome: Applied|Unknown|Failed, retry: Retry|ChangeAuth|Stop }`；
2. `NtBotError` 映射表：`denied`→`ChangeAuth`、超时→`Unknown`、网络→`Retry`；
3. 单测锁 4 档映射 + 「Unknown 不得自动重试」（P7 的核心纪律）。

**B. Desktop kill switch（小-中）**
- Desktop 顶栏加「全停」按钮 → 调 `nt_cancel` 扫全部 running turn（协议层豁免自身与状态查询）；
- 停后回执显示「已停止 / 效果未知」两态。

**C. 租约与回读（中）**
- 租约：`tasks.lease_id` 复用 + UI 倒计时；到期→`outcome_unknown`（复用 P1-2 已落账的语义）；
- 回读：动作后可选 `verify_after`（截图/DOM 文本比对），三态展示。

**D. 真后端（2026-10-08 裁决=「core 实现口、neobot 定义口」，neobot 半边已落地）**

依赖方向是 **core → neobot**，故口必须**由 neobot 先定义**：

```rust
// crates/neotrix-neobot/src/nt_computer.rs（已落地）
pub trait CdpTransport {
    fn evaluate(&self, script: &str) -> Result<String, NtBotError>;
    fn current_url(&self) -> Option<String> { None }
}
pub struct CdpBackend<T> { transport: T }   // 已落地：navigate/click/type/key/scroll/screenshot
```

- **已落地（neobot 半边）**：`compile_action()` 把动作编译成**确定性 JS**（选择器/文本经 `serde_json` 转义）；
  写动作回执非明确确认词（`clicked/typed/navigating/keyed/scrolled`）⇒ 返回 `Io` 错 ⇒ 上层映射 `Unknown` ⇒ **禁止自动重试**；
  文件类动作（`read_file/write_file/list_files`）在窄口**拒收**（属 `nt_workspace` jail，重复实现=第二条无门路径）；
  `truncate_for_detail()` 定长截断防上下文炸。
- **待落地（core 半边）**：在 `neotrix-core`（已有 chromiumoxide 依赖；现 `crawl::BrowserCircuit` 仅文本级 `fetch/login`）
  实现 `impl neotrix_neobot::nt_computer::CdpTransport for <chromiumoxide 会话>`，并暴露一个工厂；
  **接线位置**须由 owner 指定（CLI/desktop/core loop 谁注入）——**未注入前默认仍走 `NoopBackend` 诚实失败**，
  ⛔ 不允许「为了能跑」而把 Noop 换成假实现。
- **验收**：core 半边落地后，端到端一次真实 `click` + `verify_after` 回读三态（Match/Drift/Unavailable）+ 急停联动。

## 5. 记录纪律

- 本篇引用的是**模式与差距**，不是「某家做了什么」的事实断言；引用具体数字（如 16 码/十层）时必须回源 raw 原文。
- 0★ 新仓、无 issue、无社区共识 ⇒ 其结论按「单人研究笔记」权重处理，只作设计灵感源。

*版本：2026-10-08 初稿；段 0/1 取自 README 与 reusable/patterns.md；未 clone。*