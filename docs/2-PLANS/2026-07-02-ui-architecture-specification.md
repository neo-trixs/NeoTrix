# NeoTrix UI 架构规范 v2 — "Consciousness Glass"

> **版本**: 2026-07-02  
> **前置**: `2026-07-01-ui-consciousness-design-system.md` (设计哲学/色板/排版)  
> **范围**: 本文件定义 CSS 架构、组件分类、布局系统、JS 模块结构和命名约定  
> **收敛对象**: `preview-ui-v2.html` (原型) + `src-tauri/frontend/src/styles/*.css` (现有实现)

---

## 目录

1. [架构总览](#1-架构总览)
2. [Token 三层体系](#2-token-三层体系)
3. [CSS 架构: Liquid Glass 层叠系统](#3-css-架构-liquid-glass-层叠系统)
4. [组件分类法](#4-组件分类法)
5. [布局系统](#5-布局系统)
6. [JavaScript/TypeScript 模块结构](#6-javascripttypescript-模块结构)
7. [命名约定](#7-命名约定)
8. [设计规则修正](#8-设计规则修正)
9. [实现路线图](#9-实现路线图)

---

## 1. 架构总览

### 1.1 同心布局 (Concentric Layout)

```
┌──────────────────────────────────────────────────────────────┐
│  Consciousness Bar (L5)  [E8状态][GWT谐振][SEAL进化]    40px │
├─────────┬─────────────────────────────────┬──────────────────┤
│         │                                 │                  │
│  Sidebar│        Main Chat (核心)         │  Right Panel     │
│  L6自我  │                                 │  L3记忆          │
│  (220px) │  输入 → 推理流 → 响应          │  (260-320px)     │
│         │                                 │  文件树/预览     │
│         │                                 │  智能体追踪      │
│ 可折叠  │                                 │  可折叠/自动隐藏 │
├─────────┴─────────────────────────────────┴──────────────────┤
│  Status Bar (L1身体)  [提供者][延迟][成本][E8状态]     28px  │
└──────────────────────────────────────────────────────────────┘
```

### 1.2 分层规则 (与 9 层意识架构对齐)

| UI 区 | 对应意识层 | 功能 | 视觉特征 |
|-------|----------|------|---------|
| Consciousness Bar | L5 GWT | 显示 E8/GWT/SEAL 状态 | 琥珀金发光 |
| Sidebar | L6 Self | 用户身份、导航、最近会话 | 玻璃 L3 |
| Main Chat | L1-L4 | 对话/推理/工具调用 | 玻璃 L2 (最透) |
| Right Panel | L3 Memory | 文件树/预览/检索 | 玻璃 L2 |
| Status Bar | L1 Body | 系统健康、延迟、模式 | 玻璃 L1 (最实) |
| Overlay/Popover | L7 Capability | 设置/注册表/能力视图 | 玻璃 L4 (最实) |

### 1.3 CSS 三层架构

```
Layer 0: Design Tokens  (--nt-*)         设计令牌
Layer 1: Glass System   (.lg-*)          玻璃组件基座
Layer 2: Component CSS  (.nt-*)          业务组件 (CSS Modules)
```

---

## 2. Token 三层体系

### 2.1 分层定义

```
Layer A: Base Tokens       (design-tokens.css)    原始值
Layer B: Semantic Tokens   (consciousness-tokens.css)  语义映射
Layer C: Component Tokens  (如 .nt-sidebar 内部)  组件级作用域
```

### 2.2 Token 分类 + 命名规范

| 类别 | 前缀 | 示例 | 位置 |
|------|------|------|------|
| 背景 | `--nt-bg` | `--nt-bg-canvas`, `--nt-bg-surface` | design-tokens |
| 玻璃深度 | `--nt-glass` | `--nt-glass-bg`, `--nt-glass-L2-bg` | design-tokens |
| 布局 | `--nt-layout` | `--nt-layout-sidebar-w`, `--nt-layout-cons-h` | design-tokens |
| 模糊 | `--nt-blur` | `--nt-blur-sm(8px)`, `--nt-blur-md(16px)` | design-tokens |
| 圆角 | `--nt-radius` | `--nt-radius-xs(4px)`, `--nt-radius-sm(7px)` | design-tokens |
| 间距 | `--nt-gap` | `--nt-gap-xs(4px)`, `--nt-gap-md(10px)` | design-tokens |
| 色彩 (语义) | `--nt-color` | `--nt-color-primary`, `--nt-color-danger` | design-tokens |
| 阴影 | `--nt-shadow` | `--nt-shadow-sm`, `--nt-shadow-md` | design-tokens |
| 动画 | `--nt-dur` | `--nt-dur-f(0.15s)`, `--nt-dur-n(0.25s)` | design-tokens |
| 意识 (覆盖) | `--nt-accent` | `--nt-accent(amber)`, `--nt-e8-line-yang` | consciousness-tokens |
| 模式 | `--nt-mode` | `--nt-canvas(warm)`, `--nt-surface` | consciousness-tokens |

### 2.3 Token 值规范

| Token | Light | Dark | 用途 |
|-------|-------|------|------|
| `--nt-bg-canvas` | `#FAF8F4` | `#1E1C19` | 主背景(暖) |
| `--nt-bg-surface` | `#F5F2EC` | `#2A2824` | 面板背景 |
| `--nt-bg-elevated` | `#FFFFFF` | `#33312C` | 弹窗/输入框 |
| `--nt-glass-L0-bg` | `rgba(255,255,255,0.30)` | `rgba(16,16,24,0.50)` | 最深玻璃 |
| `--nt-glass-L1-bg` | `rgba(255,255,255,0.40)` | `rgba(20,20,28,0.55)` | 次深玻璃 |
| `--nt-glass-L2-bg` | `rgba(255,255,255,0.50)` | `rgba(26,26,34,0.62)` | 标准玻璃 |
| `--nt-glass-L3-bg` | `rgba(255,255,255,0.62)` | `rgba(32,32,42,0.70)` | 次浅玻璃 |
| `--nt-glass-L4-bg` | `rgba(255,255,255,0.75)` | `rgba(40,40,52,0.80)` | 最浅玻璃(最实) |
| `--nt-accent` | `#C4944A` | `#D4A84E` | 琥珀金主色 |
| `--nt-text-primary` | `#1C1B19` | `#EDEDED` | 正文 |
| `--nt-text-secondary` | `#6E6A64` | `#A0A0A8` | 次要文字 |
| `--nt-text-muted` | `#9F9A92` | `#606068` | 弱化文字 |
| `--nt-color-success` | `#5A8F5A` | `#66BB6A` | 苔绿 |
| `--nt-color-danger` | `#BC4A3C` | `#FF6B6B` | 陶土红 |
| `--nt-color-warning` | `#D4A04A` | `#FFD54F` | 姜黄 |

### 2.4 Z-index 体系 (变量化)

```
--nt-z-traffic:   150    窗口控制点
--nt-z-sidebar:   100    侧栏浮动按钮
--nt-z-popover:   300    弹出菜单
--nt-z-overlay:   200    遮罩层/模态框
--nt-z-toast:     400    提示消息
--nt-z-tooltip:   500    工具提示
```

**规则**: 禁止在 CSS 中硬编码 `z-index` 数值。一律使用 `var(--nt-z-*)`.

---

## 3. CSS 架构: Liquid Glass 层叠系统

### 3.1 文件结构

```
styles/
├── design-tokens.css         ← Layer 0A: Base tokens (colors, spacing, radius, blur, shadow)
├── consciousness-tokens.css  ← Layer 0B: Semantic tokens (consciousness overrides, dark theme)
├── liquid-glass.css          ← Layer 1: Glass component system (.lg-*)
│   ├── Glass base (.lg-glass, .lg-glass-clear, .lg-glass-strong)
│   ├── Controls (.lg-btn, .lg-input, .lg-select, .lg-toggle)
│   ├── Composite (.lg-card, .lg-panel, .lg-tab-bar, .lg-badge)
│   ├── Feedback (.lg-skeleton, .lg-empty, .lg-toast)
│   ├── Motion (.lg-fade-in, .lg-slide-up, .lg-scale-in)
│   └── Utility (.lg-flex, .lg-scrollbar)
├── utilities.css              ← Layer 1.5: Utility classes (.u-*)
│── Component.module.css       ← Layer 2: Per-component scoped CSS
└── global.css                 ← Orchestrator: @import /* + rollup */
```

### 3.2 Glass 深度层映射

```css
/* 每层对应的布局区域 */
--nt-glass-L0:  body::before(背景光晕), 应用最深层
--nt-glass-L1:  .ma(主区域背景)
--nt-glass-L2:  .sb(侧栏), .rb(右栏), .cic(输入框), .seg(Tab)
--nt-glass-L3:  .sf(用户栏), .popover-card, .sb-float, .rb-float
--nt-glass-L4:  .app(外壳), .overlay-box(遮罩框), .st-modal(设置)
```

### 3.3 Component CSS 原则 (CSS Modules)

所有业务组件使用 CSS Modules, 遵循:

- 文件名: `ComponentName.module.css`
- 类名: `camelCase` (CSS Modules 自动处理)
- 布局相关使用 `--nt-layout-*` tokens
- 玻璃效果使用 `.lg-glass` / `.lg-glass-strong` 等基类
- 避免在 module 中重复定义颜色/间距 — 使用 token

```tsx
// ✅ Correct
import styles from './ChatPanel.module.css'
<div className={`${styles.container} lg-glass`}>

// ❌ Wrong
<div className="chat-panel" style={{ background: '#FAF8F4' }}>
```

### 3.4 `global.css` 中的全局类 (仅限布局)

`global.css` 只应包含:
1. CSS Reset
2. 布局结构类 (`.app-body`, `.main-panel`, `.right-panel`)
3. `@import` 所有其他 CSS 文件
4. 顶级媒体查询 (responsive breakpoints)

不再新增业务组件样式到 `global.css`。

### 3.5 媒体查询管理

```css
/* 断点系统 */
--nt-bp-sm: 640px;    /* 移动端 */
--nt-bp-md: 1024px;   /* 平板 */
--nt-bp-lg: 1280px;   /* 桌面 (默认) */
--nt-bp-xl: 1536px;   /* 大屏 */

/* 响应式策略:
   < 640px:  全屏单列, 侧栏/右栏隐藏, 弹出式导航
   640-1024: 侧栏可见, 右栏自动隐藏
   1024-1280: 全布局, 右栏自动隐藏
   > 1280px: 全布局 (默认) */
```

---

## 4. 组件分类法

### 4.1 完整组件树

```
App (外壳)
├── ConsciousnessBar (L5)
│   ├── E8Indicator          — 六爻显示器
│   ├── GWTResonanceMeter    — 专家谐振热图
│   ├── SEALBadge           — 进化进度标记
│   └── WindowControls      — 交通灯 (macOS)
├── Sidebar (L6)
│   ├── SidebarHeader       — 窗口按钮 + 搜索
│   ├── ModeSegmented       — 4 模式选择 (对话/团队/代码/代理)
│   ├── NavList             — 动态导航项
│   ├── RecentList          — 最近会话列表
│   └── UserBar             — 用户信息 + Popover
│       └──UserPopover      — 弹窗菜单
├── MainContent
│   ├── ChatView            — 对话模式
│   │   ├── ChatTopBar      — 升级/信息按钮
│   │   ├── HeroSection     — 欢迎状态
│   │   ├── ChatStream      — 消息流
│   │   │   ├── MessageBubble (user/assistant)
│   │   │   └── ArtifactLink
│   │   └── InputPanel      — 输入区
│   │       ├── Textarea
│   │       ├── AttachButton
│   │       └── SendButton  — 语音/发送切换
│   ├── CoworkView          — 团队模式
│   │   ├── SessionList     — 左栏会话列表
│   │   └── TaskBoard       — 右栏任务看板
│   │       ├── TaskItem
│   │       └── AgentChip
│   └── CodeView            — 代码模式
│       ├── FileTree        — 文件导航
│       ├── EditorTabs      — 标签页
│       ├── EditorToolbar   — 操作栏
│       └── CodeViewer      — 代码显示
├── AgentDashboard          — 代理模式
│   ├── HeroCard            — 状态环 + 链可视化
│   ├── TabNavigation       — 概览/地图/节点/订阅/设置
│   ├── OverviewPane        — 指标网格/IP池/LLM池/网络
│   ├── WorldMap            — SVG 节点地图
│   ├── NodeTable           — 节点表格
│   ├── SubscriptionList    — 订阅管理
│   └── SettingsForm        — 配置表单
├── RightPanel (L3)
│   ├── ArtifactPreview     — 文件预览
│   │   ├── ViewToggle      — 预览/代码
│   │   ├── FormatTabs      — Raw/Rendered/WeChat/Zhihu/Juejin/Web
│   │   ├── ContentArea     — 内容渲染
│   │   └── FooterActions   — 复制/刷新/展开/关闭
│   └── FileTree            — 文件树
├── StatusBar (L1)
│   ├── StatusProvider      — 提供者信息
│   ├── StatusLatency       — 延迟
│   ├── StatusE8            — E8 状态
│   └── StatusProxy         — 代理状态
├── Overlays
│   ├── SettingsModal       — 设置面板 (双栏: nav + content)
│   ├── CapabilityRegistry  — 能力注册表
│   ├── HypercubeViewer     — 知识超立方体
│   └── GenericPanel        — 通用弹窗 (项目/成果/计划)
└── Misc
    ├── Toast               — 提示消息
    ├── PopoverMenu         — 通用弹出菜单
    └── LoadingSkeleton     — 加载骨架
```

### 4.2 组件分类

| 类别 | 命名前缀 | 说明 | 示例 |
|------|---------|------|------|
| 玻璃组件 | `.lg-*` | 可复用组件 | `.lg-btn`, `.lg-card`, `.lg-panel` |
| 布局 | (global.css) | 页级布局 | `.app-body`, `.main-panel` |
| 业务组件 | CSS Module | 功能单元 | `ChatPanel.module.css` |
| 工具 | `.u-*` | 原子类 | `.u-flex-center`, `.u-text-muted` |

---

## 5. 布局系统

### 5.1 布局变量

```css
--nt-layout-sidebar-w:    220px;
--nt-layout-sidebar-collapsed: 0px;
--nt-layout-cons-h:       40px;
--nt-layout-rightpanel-w: 280px;
--nt-layout-status-h:     28px;
--nt-layout-app-w:        1280px;
--nt-layout-app-h:        760px;
--nt-layout-panel-gap:    0px;  /* 玻璃面板间无间距 */
```

### 5.2 响应式行为

| 视口宽度 | 侧栏 | 右栏 | 布局 |
|---------|------|------|------|
| ≥1280px | 可见 | 可见/自动隐藏 | 默认 |
| 1024-1279px | 可见 | 自动隐藏 | 中等 |
| 640-1023px | 可折叠 | 隐藏 | 紧凑 |
| <640px | 隐藏 (浮动按钮) | 隐藏 | 移动端 |

### 5.3 侧栏折叠

```css
/* 3 状态: 展开 / 折叠 / 自动隐藏 */
.sb--expanded   { width: var(--nt-layout-sidebar-w); }
.sb--collapsed  { width: 0; overflow: hidden; }
.sb--auto-hide  { /* 鼠标靠近左边缘触发 */ }
```

### 5.4 右栏折叠

```css
/* 4 状态: 展开 / 折叠 / 自动隐藏(默认) / hover */
.rb--expanded      { width: var(--nt-layout-rightpanel-w); }
.rb--collapsed     { width: 0; opacity: 0; }
.rb--auto-hide     { width: 0; } /* 默认状态 */
.rb--auto-hide:hover,
.rb--hover         { width: var(--nt-layout-rightpanel-w); }
```

---

## 6. JavaScript/TypeScript 模块结构

### 6.1 模块拆分

```
src/
├── components/
├── pages/
├── stores/
├── lib/
│   ├── api.ts              ← Tauri invoke/poll 统一封装
│   ├── invoke.ts           ← 后端命令函数
│   ├── events.ts           ← 事件监听/流处理
│   ├── state.ts            ← 本地状态工具
│   └── utils.ts            ← 通用函数 (escHtml, 日期格式化等)
└── types/
    ├── index.ts            ← 共享类型
    ├── chat.ts             ← 对话相关
    ├── proxy.ts            ← 代理仪表盘
    └── system.ts           ← 系统配置
```

### 6.2 API 层设计

```typescript
// lib/api.ts — 统一接口
interface BackendAPI {
  reason(prompt: string, sessionId?: string): Promise<StreamHandle>;
  readDir(path: string): Promise<FileEntry[]>;
  readFile(path: string): Promise<string>;
  proxyStatus(): Promise<ProxyStatus>;
  proxyPoolNodes(): Promise<ProxyNode[]>;
  // ...
}

// 具体实现: TauriCommandAPI implements BackendAPI
export const api = new TauriCommandAPI();

// 组件使用时:
import { api } from '../lib/api';
const nodes = await api.proxyPoolNodes();
```

### 6.3 状态管理 (Zustand)

```typescript
// stores/uiSlice.ts — UI 相关状态
interface UISlice {
  sidebarCollapsed: boolean;
  rightPanelMode: 'auto-hide' | 'expanded' | 'collapsed';
  currentView: 'chat' | 'cowork' | 'code' | 'agent';
  theme: 'light' | 'dark';
  consciousnessActive: boolean;
  // actions
  toggleSidebar(): void;
  setRightPanelMode(mode: UISlice['rightPanelMode']): void;
  switchView(view: UISlice['currentView']): void;
  toggleTheme(): void;
}
```

### 6.4 原型 → 正式代码迁移规则

1. 不再在 `.html` 中添加新 JS — 所有新逻辑写 TypeScript
2. 原型中的 `showToast`, `escHtml`, `render*Table` 等函数移到 `lib/utils.ts`
3. 静态数据 (`PROXY_DATA`, `FREE_LLM_DATA`, `PX_NODES` 等) 移到 `data/*.ts`
4. `sendMsg()`, `renderCowork()`, `switchView()` 等逻辑移到对应组件

---

## 7. 命名约定

### 7.1 CSS

| 类型 | 格式 | 示例 |
|------|------|------|
| Custom Properties | `--nt-{category}-{property}` | `--nt-glass-bg`, `--nt-layout-sidebar-w` |
| 玻璃组件类 | `.lg-{component}[-{variant}]` | `.lg-glass`, `.lg-btn`, `.lg-btn-primary` |
| 工具类 | `.u-{property}` | `.u-flex`, `.u-text-muted` |
| 业务组件 (CSS Module) | `.{camelCase}` | `.container`, `.header`, `.active` |
| 全局布局 | `.{kebab-case}` | `.app-body`, `.main-panel`, `.right-panel` |
| 状态修饰符 | `.{component}--{state}` | `.sb--collapsed`, `.rb--hover` |
| 动画 | `@keyframes lg-{name}` | `@keyframes lg-fadeIn` |
| 媒体查询 | `@include bp-{size}` | `@media (max-width: 640px)` |

### 7.2 TypeScript/JavaScript

| 类型 | 格式 | 示例 |
|------|------|------|
| 组件 | PascalCase | `ChatPanel`, `AgentDashboard` |
| 函数 | camelCase | `sendMessage()`, `formatDate()` |
| 类型/接口 | PascalCase | `ProxyNode`, `CoworkSession` |
| 常量 | UPPER_SNAKE | `PROXY_DATA`, `FREE_LLM_DATA` |
| CSS Module | `styles.{camelCase}` | `styles.container` |

---

## 8. 设计规则修正

基于 `preview-ui-v2.html` 审查, 以下设计规则需要修正:

### 8.1 必须修正 (P0)

| # | 规则 | 当前错误 | 修正 |
|---|------|---------|------|
| R1 | **主题一致性** | `body` 背景硬编码 `#0B0B12`，不随 theme 切换 | Light: `var(--nt-bg-canvas)`, Dark: `#1E1C19` |
| R2 | **响应式** | 固定 `1280×760`, <1270px 无行为 | 按断点表折叠侧栏/右栏 |
| R3 | **Z-index 变量化** | 硬编码 100/150/200/300 | 全部替换为 `var(--nt-z-*)` |
| R4 | **背景光源分离** | Light Mode body 背景色应为暖米白 | `var(--nt-bg-canvas)` + 星云光晕仅在 Dark 模式 |
| R5 | **Light 模式玻璃** | Light 模式玻璃 L0-L4 白色基座正确 | 确认 `prefers-reduced-transparency` 降级 |
| R6 | **CCW 旋转原则** | 所有旋转动画是逆时针 | 已正确执行 |

### 8.2 建议修正 (P1)

| # | 规则 | 说明 |
|---|------|------|
| R7 | **禁止 inline style** | 所有移入 CSS 类或 token 变量 |
| R8 | **Focus-visible 全覆盖** | 所有可交互元素加 `:focus-visible` 样式 |
| R9 | **XSS 防护** | 扫描所有 `innerHTML` 替换为安全渲染 (DOMPurify) |
| R10 | **Skeleton 状态** | 所有数据加载区域加 `.lg-skeleton` |
| R11 | **Error 边界** | 每个视图有 error 恢复状态 |
| R12 | **Loading 状态** | API 操作期间显示加载指示器 |
| R13 | **Empty 状态** | 空数据区域显示 `.lg-empty` |

### 8.3 前瞻改进 (P2)

| # | 规则 | 说明 |
|---|------|------|
| R14 | **i18n 准备** | 所有用户可见字符串通过 i18n key 引用 |
| R15 | **动画性能** | `will-change: transform` + `@media (prefers-reduced-motion)` |
| R16 | **暗色模式星云** | Light 模式移除纯黑背景光源 |

---

## 9. 实现路线图

### Phase 1: 架构落地 (当前)

| Task | 产出 |
|------|------|
| 本规范文档 | ✅ `2026-07-02-ui-architecture-specification.md` |
| 修复 Light 背景问题 | `preview-ui-v2.html` body 背景修正 |
| Z-index 变量化 | 添加 `--nt-z-*` tokens + 替换所有硬编码 |
| `preview-ui-v2.html` 按规范重构 | 分离为逻辑模块 |

### Phase 2: CSS 模块化

| Task | 产出 |
|------|------|
| 对齐 `liquid-glass.css` 与 preview 的 .px-* / .cw-* 组件 | 统一 .lg-* 命名 |
| 合并 `preview-ui-v2.html` 样式到 `styles/` | 消除重复 |
| 删除 `global.css` 中的死代码 ("MOVED TO CSS MODULE" 行) | 清理遗留 |

### Phase 3: JS 模块化

| Task | 产出 |
|------|------|
| `lib/api.ts` — 统一 API 层 | 所有 `invoke` 调用封装 |
| `stores/uiSlice.ts` — UI 状态 | Sidebar/RBPanel/View/Theme |
| 静态数据迁移 | PROXY_DATA 等移到 `data/` |
| `lib/utils.ts` — 工具函数 | showToast, escHtml, render* |

### Phase 4: 正式接入

| Task | 产出 |
|------|------|
| 替换桩函数为真实 Tauri invoke | 端到端连通 |
| Playwright E2E 测试 | 覆盖 4 视图切换 |
| Vitest 单元测试 | 覆盖 utils, store |

---

> **架构原则摘要**  
> 1. 一切是 token — 无硬编码值  
> 2. 一切是组件 — 无重复的 DOM 结构  
> 3. 一切有状态 — 无隐式 DOM 状态  
> 4. 一切走 API — 无直接 Tauri 调用  
> 5. 一切可响应 — 无固定尺寸布局
