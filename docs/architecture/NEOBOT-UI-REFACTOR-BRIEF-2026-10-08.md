# neobot-ui 侧栏 + 对话栏重构简报（对齐吸收项目 · 2026-10-08）

> 依据：`ABSORPTION-NEOBOT-ECOSYSTEM-2026-10-08.md`（D 组）+ 本窗实测结论。
> 范围：`apps/neobot-desktop/neobot-ui/src/neobot-root.tsx`（侧栏 + 转录区 + 输入区）。
> 纪律：改前先 `stat -f '%Sm'` 查并发；每步 `tsc --noEmit` + `scripts/ui-draft-isolation-shots.cjs` 截图实测；
> ⛔ 不照搬外部代码（吸收的是设计，代码自研；OpenGhost 渲染器已 MIT vendor 有 PROVENANCE 护栏）。

## 0. 实测基线（2026-10-08，本窗证据）

- `tsc --noEmit` rc=0 / `vite build` 502ms / mock-IPC 草稿分区实测 **rc=0**
  （未读角标可见、A 草稿不串台到 B、切回 A 恢复）。
- 已落：草稿按会话分区、unread 角标、行折叠「还有 N 行」、openghost 渲染（MIT 合规 vendor）。
- 可复跑脚本：`neobot-ui/scripts/ui-demo-shots.cjs`、`ui-draft-isolation-shots.cjs`。

## 1. 侧栏（sidebar）重构对齐

### 1.1 注册制骨架 ← `omdsh-dev/DSH-better-sidebar`（MIT，代码可熔炼）
- 把侧栏从「硬编码页签列表」改为 **tab 注册表**：`registerTab({ id, title, order, render })`；
  内置「会话/记忆/对话/轨迹」四页签改为按同一注册表挂载（自举验证注册制可用）。
- **服务开放给未来插件**：暴露 `ctx.sidebar.registerTab / registerFileViewer` 等价物
  （本期仅内部 API，不接插件系统 —— 插件协议属 D5 未定项，⛔ 不得替他业务组发明）。
- 右栏 + 底部工作台**双工作台**结构：转录区为底栏语义，页签为右栏语义，布局不变，
  仅把「谁有权挂进来」收口为注册点。

### 1.2 分组与本地状态 ← `dsh-market`（MIT）
- 收藏 / 备注 / 分组三件套是**纯组织性本地状态**（不碰 enable/选中语义）：
  侧栏会话组支持重命名、组内搜索、拖拽归组；数据落 `nt_store`（Rust 侧新表
  `market_state` 同构的 `sidebar_state`），前端不自存。
- **优雅自禁并明说**：后端能力缺失时页签显示「不可用 + 原因 + 重试」，
  不渲染残缺面板（本窗实测已具备此行为，重构中保持为回归断言）。

### 1.3 行语义 ← `thinkany-ai/douchat`（GPL 族：⛔ 只抄语义不抄代码）
- 联系人/会话行三态：`unread / busy / muted` 已有 → 补 **busy（运行中）行内进度**与
  `@` 点名路由提示（群聊语义待 channel 侧就绪，本期只留展示位）。
- 行副标题显示「私聊 · 分钟前」已有 → 保留；组头可折叠已有 → 保留。

## 2. 对话栏（转录区 + 输入区）重构对齐

### 2.1 澄清卡 ← `CopilotKit/openmuse`（MIT）
- 新消息类型 `kind:'clarify'`：标题 + 结构化选项按钮 + 「继续输入」兜底。
- 渲染走 openghost shim 同层；数据契约先在 TUI 侧落地（TUI 先行，桌面跟进）——
  依赖 agent loop 产出 clarify 事件，**前端先做可渲染空壳 + mock 实测**
  （`ui-draft-isolation-shots.cjs` 增加 clarify 用例）。

### 2.2 Take control 同会话语义 ← `openmuse`
- 现有 `control`（take-the-wheel）CLI 已存在 → UI 补「接管中」横幅：
  明示「你正在控制同一会话，agent 暂停」，恢复 = 同一会话继续（不新开）。

### 2.3 细节交互 ← `ANDRETRIPOL/OpenGhost`（代码 MIT，⛔ 视觉设计不搬）
- 已落：长行折叠、草稿分区、unread。待补：
  - 工作中来信 **蓝点**（当前 unread 只在会话行；转录区头部补未读分隔线）；
  - 发送中消息**虚线待定态**（可撤回）—— 收发状态机在 `thread.ts`/root 侧扩展；
  - compaction 后**保留最后 N 步逐字原文**（依赖 P1-6 账本接线，前端只做展示位）。

## 3. 顺序与验收

| Step | 内容 | 验收 |
|---|---|---|
| S1 | 侧栏 tab 注册表化（内置四页签改注册挂载） | `tsc --noEmit` rc=0 + 截图对照基线无回归 |
| S2 | 会话组本地状态（收藏/备注/分组） | `ui-draft-isolation-shots.cjs` rc=0 + 新增分组用例 |
| S3 | clarify 空壳渲染 + mock 用例 | 截图：卡片可见、选项可点（mock 回显） |
| S4 | take-control 横幅 + 蓝点分隔线 | 截图 + 现有断言不回退 |

每步独立提交；任何一步 `mtime` 显示他窗在写 `neobot-root.tsx` 则暂停并通报。
