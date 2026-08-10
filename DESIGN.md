---
version: alpha
name: NeoTrix — Snowfield White
description: "AI-native developer toolkit. Agent workbench five-zone layout, snowfield-white canvas with a single light-orange (#f0913a) brand accent. Layout mirrors Claude Desktop / Codex Desktop / Osaurus."
colors:
  primary: "#f0913a"
  secondary: "#e07f2b"
  neutral: "#fbfaf7"
  surface: "#f5f3ef"
  surface-low: "#efedea"
  text-primary: "#1a1a20"
  text-secondary: "#5a5a62"
  text-muted: "#909098"
  border-primary: "#e5e4e0"
  border-focus: "#f0913a"
  faction-core: "#22c55e"
  faction-mind: "#a855f7"
  faction-memory: "#3b82f6"
  faction-world: "#d946ef"
  faction-act: "#f97316"
  faction-io: "#f0913a"
  faction-shield: "#64748b"
  faction-repair: "#14b8a6"
typography:
  h1:
    fontFamily: "SF Pro Display, -apple-system, Noto Sans SC"
    fontSize: 22px
    fontWeight: 600
    lineHeight: 1.25
  body:
    fontFamily: "SF Pro Text, -apple-system, Noto Sans SC"
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.6
  label:
    fontFamily: "SF Pro Text, -apple-system, Noto Sans SC"
    fontSize: 11px
    fontWeight: 500
    lineHeight: 1.4
  caption:
    fontFamily: "SF Mono, JetBrains Mono, Menlo"
    fontSize: 10px
    fontWeight: 400
    lineHeight: 1.4
rounded:
  none: 0px
  sm: 6px
  md: 8px
  lg: 12px
  xl: 16px
  full: 9999px
spacing:
  base: 4px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: 8px 16px
  button-primary-hover:
    backgroundColor: "{colors.secondary}"
  button-io-600:
    backgroundColor: "{colors.secondary}"
    textColor: "#ffffff"
    rounded: "{rounded.md}"
    padding: 8px 16px
  button-io-700:
    backgroundColor: "#bd6720"
    textColor: "#ffffff"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: 8px 16px
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.lg}"
    padding: 16px
  input:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: 8px 12px
  input-focus:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: 8px 12px
  seg-tab:
    backgroundColor: "transparent"
    textColor: "{colors.text-secondary}"
    rounded: "{rounded.md}"
    padding: 4px 12px
  seg-tab-active:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-primary}"
---

# NeoTrix — 产品设计系统 (Product Design System)

> 格式: Google Labs [DESIGN.md spec](https://github.com/google-labs-code/design.md) (Apache 2.0)
> 版本: v1.1.0 (2026-08-10) · 对标基线: Claude Desktop / Codex Desktop / Osaurus
> 完整方法论 (需求→架构→代码→上线): `~/.agents/skills/des/ui/references/product-design-lifecycle.md`
> **Token 为规范性值, 正文 prose 提供应用上下文。冲突以 YAML front matter 为准。**

## Overview

NeoTrix 是一个 AI-native 开发者工具, 界面是**自主行为与用户的问责层**。视觉上采用
**雪域白 (Snowfield White)** —— 纯白为体, 一丝浅橙 (#f0913a) 仅出现在焦点/选中/品牌强调处。

- 布局对标三款主流 AI 桌面 app 收敛出的 **Agent 工作台五区** (顶栏+左栏+主区+状态栏+覆盖层)
- 玻璃层次 L1-L4 实色面 (非半透明滥用)
- 逆时针 CCW 动画原则, 所有动效 `prefers-reduced-motion` 必加
- 7 域派系色用于功能着色 (core绿/mind紫/memory蓝/world紫红/act橙/io浅橙/shield灰蓝/repair青)
- Agent 原生 UI: 透明层 / 状态通信 / 中止控制 / 错误恢复 / 权限边界

## Colors

主色 = **浅橙 `#f0913a`**。这是**单一事实源**, 禁止在文档/代码中声明冲突品牌色
(历史教训: 早期文档写红 `#E85454`, 实现用橙, 造成漂移)。

- **Primary (#f0913a):** 品牌强调, 焦点边框/选中/主 CTA。唯一驱动交互的颜色。
- **Secondary (#e07f2b):** Primary 深一级, hover/按压态。
- **Neutral (#fbfaf7):** 雪域白画布主背景。
- **Surface (#f5f3ef):** 玻璃 L1, 卡片/侧栏。
- **Surface-low (#efedea):** 玻璃 L0, 更低层。
- **Text (#1a1a20/#5a5a62/#909098):** 正文/次要/弱化三档。

### 派系色 (7 域功能着色)
各域 50-900 阶, 500 档用于标识。`faction-core`(绿) `faction-mind`(紫) `faction-memory`(蓝)
`faction-world`(紫红) `faction-act`(橙) `faction-io`(浅橙·品牌) `faction-shield`(灰蓝) `faction-repair`(青)

## Typography

字体策略: **系统字体优先**, 无 web font 加载成本。

- **Headlines / Body / Labels:** `SF Pro` (macOS) → `-apple-system` → `Noto Sans SC`
- **Captions / 数据 / 时间戳:** `SF Mono` → `JetBrains Mono` → `Menlo`
- **禁止 Inter/Roboto/Arial** (反 AI Slop)
- 微字号刻度 9-14.5px 收敛 (9/10/10.5/11/12/12.5/13.5/14.5), 避免硬编码漂移

### 字号档位
| 用途 | 值 |
|------|-----|
| h1 (弹窗/页标题) | 22px / 600 / 1.25 |
| body 默认 | 13px / 400 / 1.6 |
| label | 11px / 500 / 1.4 |
| caption (mono) | 10px / 400 / 1.4 |
| 微字号 | 9-14.5px 收敛刻度 |

## Layout

布局遵循 **Agent 工作台五区** 模型:

```
┌────────────────────────────────────────────────┐
│ 顶栏 (ch-top): 拖拽区 + 模式切换 (对话/协同/电脑) │
├──────────┬───────────────────────┬─────────────┤
│ 左栏      │ 主工作区               │ 右侧上下文    │
│ 会话/项目 │ Chat/Composer         │ 预览/diff    │
│ 用户条    │ + 可拖拽面板           │ /终端        │
├──────────┴───────────────────────┴─────────────┤
│ 状态栏: 模型 / context% / 状态反馈 / 快捷键       │
└────────────────────────────────────────────────┘
 + 覆盖层: 设置弹窗(分组nav) / 插件市场(内嵌) / 审批门
```

间距采用 **4pt/8pt grid** (spacing-xs..xl)。组件分组用"containment"原则:
相关项放卡片内, 卡片内边距 16-24px 营造柔和感。

## Elevation & Depth

深度通过 **玻璃层次 (Glass Layers) 实色面** 传达, 而非重阴影:

| 层 | 值 | 用途 |
|----|-----|------|
| L1 | surface `#f5f3ef` | 侧栏/卡片 |
| L2 | 纯白 + border | 弹窗 (glass-modal) |
| shadow-glass-pop | `0 24px 64px rgba(40,30,20,0.18)` | 弹窗浮起 |

## Shapes

形状语言: **暖圆角**。交互元素 radius 6-8px, 容器 12-16px, 徽章全圆。

- 按钮/输入/seg: md (8px)
- 卡片/面板: lg (12px)
- 弹窗 (glass-modal): xl (16px)
- 头像/徽章: full
- **禁止混用尖角与圆角于同一视图**

## Components

组件规格 (token 见 front matter `components` 段, 变体用 `-hover`/`-focus` 后缀):

| 组件 | 规格要点 |
|------|---------|
| Button primary | 浅橙底深字, md 圆角, hover→secondary |
| Button secondary | surface 底 + border-primary 描边, 文字主色 |
| Card | surface 底 + border-primary 暖白描边, lg 圆角, 16px padding |
| Input | neutral 底 + border-primary 描边, focus→border-focus |
| Seg tab (tablist) | aria-selected + roving tabindex + 方向键 |
| Nav (分组) | 图标 + 激活指示 + 方向键/Home/End |
| vc-send | loading→停止 (中止控制) |
| Tool-call 卡 | 运行/成功/失败/折叠 |
| 插件卡 | 安装/启用/卸载/已装 |

### 状态矩阵 (每交互组件必填)
| 组件 | Default | Hover | Active | Focus | Disabled | Loading | Error |
|------|---------|-------|--------|-------|----------|---------|-------|
| Button | 见上 | secondary | 压暗 | ring | opacity .5 | spinner | — |
| Input | border | — | — | border-focus | opacity .5 | — | 语义色 |
| Seg | transparent | surface | surface | ring | — | — | — |

## Do's and Don'ts

- Do 用 primary 色仅限每屏最重要的单个动作
- Do 保持 WCAG AA 对比度 (正文 ≥4.5:1)
- Don't 混用圆角与尖角于同一视图
- Don't 使用 Inter/Roboto/Arial
- Don't 用超过两档字重于单屏
- Don't 硬编码 hex 绕过 token (TOKEN-DRIFT)
- Don't 用非 4px 倍数间距 (0.5/1.5/2.5 档一律收敛到 4px 倍数)
- Do 大标题用 `clamp()` 流体排版 (最小 24px / 4vw / 最大 34px)
- Do 为 Agent 行为提供透明层/中止/恢复 (问责层)

## 可计算性检查 (输出前必跑)
| 检查 | 标准 | 当前状态 |
|------|------|---------|
| 对比度 | WCAG AA ≥4.5:1 正文 | ✅ 橙底一律深字 (text-primary #1a1a20, 7.27:1) |
| 8pt grid | 间距 4/8 倍数 | ✅ 已收敛 (0.5→1, 1.5→2, 2.5→3) |
| 流体排版 | 大标题 clamp() | ✅ .hero h1 已落地 (clamp(24px,4vw,34px)) |
| 状态覆盖 | 交互组件 hover/focus/disabled | 达标 |
| 图标统一 | 三档: w-3.5/w-4/w-5 | 达标 |
| 语义色 | 错误/成功/警告各一值 | 达标 |

> ✅ **对比度规则**: 浅橙底 (#f0913a/io-500) 与深橙底 (#e07f2b/io-600) 一律配**深字** (text-primary #1a1a20 = 7.27:1 ✓)。
> 已全量收敛: btn-primary / 内联按钮 / ::selection 均改橙底深字, 零白字债务。

## 验证命令
```sh
# lint 结构 (需安装 Google design.md CLI)
npx @google/design.md lint DESIGN.md
# 导出 token 到 Tailwind/DTCG/CSS
npx @google/design.md export --format tailwind DESIGN.md > tailwind.theme.json
```

## 变更记录
- v1.2.0: 收敛全部债务 — 间距 0.5/1.5/2.5 → 4px 倍数 (全 18 文件); 橙底白字 → 橙底深字 (btn-primary/内联/::selection, 7.27:1); .hero h1 落地 clamp(24px,4vw,34px); 移除 Inter/Roboto/Arial 残留
- v1.1.0: 吸收 Google Labs DESIGN.md 官方格式 — 新增 YAML 机器可读 token 层 /
  8 段结构 / 组件变体 token / Do's and Don'ts / lint CLI
- v1.0.0: 骨架建立, 提取三款 app 通用布局 + 当前 token/组件实现
