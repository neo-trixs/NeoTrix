# 吸收分析 — Kun → NeoTrix

> 来源: [KunAgent/Kun](https://github.com/KunAgent/Kun) (Electron + React 19 + Zustand 5, 6.2k★, 2134 commits)
> 定位: 本地优先 AI Agent 工作台 — Code/Work 双模式, 桌面 GUI 与终端 TUI 共享单一运行时
> 日期: 2026-08-25 · 遵循 external-absorption C1-C6 契约
> 关联: docs/absorption-dsh-minke.md (前批吸收)

## 一、Kun 核心架构洞察

### 1. 单运行时双客户端 (最重要架构决策)

```
kun serve (单一本地运行时)
  ├── HTTP/SSE surface ──── Electron 桌面 GUI
  └── 同一线程/目标/计划/审批 ─── 终端 TUI
```

线程、目标、计划、审批、后台任务在 GUI 与 TUI 之间**始终连续**——不是两套会话。
反模式清单明令禁止: "Use a second live agent runtime"。

**NeoTrix 现状对照**: CLI (`cli/commands` registry + `cli::tui`) 与桌面
(`src-tauri neocodex_cmds` + `neocodex-frontend`) 各有独立会话存储与命令分发，
是**两个割裂的运行时**。这是当前架构最大的结构性债务。

### 2. DESIGN.md = 机器可读设计令牌 (Design System as Code)

Kun 的 DESIGN.md frontmatter 是给设计 agent (Stitch/Figma 插件) 直接消费的
schema化 token 集:
- palette: 完整 `--ds-*` light/dark 双主题语义色 (bg/surface/border/text/accent/diff/skill...)
- typography: size_scale_px + **size_rhythm 语义映射** (caption/chip/body/title/hero → px)
- spacing/radius/elevation/motion/z-index 全部 scale 化
- **components**: 12 个复用组件的 base CSS class 定义 (card/button/input/chip/modal...)
- backgrounds: 渐变背景公式
- a11y: 焦点环/命中区/对比度/快捷键
- **dont 清单**: 10 条代码库强制执行的反模式

NeoTrix 的 DESIGN.md 已有 frontmatter (colors/typography/rounded) 但缺:
dark 主题 token、components 模式库、dont 反模式清单、a11y 规范、motion 时序。

### 3. 三支柱产品哲学 (可提炼为品牌承诺)

| 支柱 | Kun 实现 | NeoTrix 对应 |
|------|---------|-------------|
| Local-first | settings/sessions/logs 全磁盘; 自带 API key | ✅ knowledge.db 本地 |
| Observable | 每个 tool call/file change/reasoning step 在 UI 可见 | ⚠️ ToolCallCard 有, 但无统一证据流 |
| Controllable | approval policy + sandbox + interrupt + revert | ⚠️ permission_profiles + sandbox 有; **revert 弱** |

### 4. 目标→验收闭环 (任务证据链)

```
澄清目标 → 形成计划 → 执行与协作 → 检查证据 → 交付或继续
```
关键: 计划和需求**保存在项目内**(版本控制+代码审查+后续恢复), 而非应用私有存储。
每个任务的 tool calls / file changes / approvals 都关联到该任务(证据链)。

### 5. Cache-first Agent Loop

推理循环以 prompt cache 命中率为第一优化目标(省 token + 降低延迟)。
NeoTrix 的 ReasoningBrain 有记忆机制但未以 cache 命中为显式优化维度。

### 6. 其他值得注意的模式

- **AgentProvider 接口**: 渲染层可插拔 provider 抽象(NeoTrix LlmProvider trait 已对齐 ✅)
- **Workbench routes one store**: Zustand 单 store, 路由即 store 切片(SolidJS signals 可类比)
- **i18n 错误格式**: "human sentence ending in punctuation; never raw stack traces"
- **审批四件套**: approval policy + sandbox mode + interrupt + revert 缺一不可

## 二、特性吸收矩阵

| Kun 特性 | NeoTrix 现状 | 缺口 | 行动优先级 |
|----------|-------------|------|-----------|
| DESIGN.md 机器可读 frontmatter | 有基础版 | 🔴 缺 dark/components/dont/a11y | **本次补齐** |
| GUI+TUI 共享单运行时 | 两套割裂会话 | 🔴 架构级 | 路线图(重构方向) |
| Don't 反模式清单 | 分散于 dev-rules | 🟡 集中化 | **本次融合** |
| 任务证据链(tool call→diff→test 关联) | ToolCallCard 孤立 | 🟡 中 | 路线图 |
| Plan 存项目内(版本控制) | plan 内存态 | 🟡 中 | 路线图 |
| 审批四件套(policy/sandbox/interrupt/**revert**) | revert 缺失 | 🟡 中 | 路线图 |
| Cache-first loop | 未显式建模 | 🟢 低优 | 记录 |
| 多 Provider 预设(11 家) | 已支持主流+免费池 | 🟢 已对齐 | — |
| i18n 错误文案规范 | 无 | 🟢 文档级 | 本次记录 |

## 三、本次落地: DESIGN.md 升级

吸收 Kun 的 "Design System as Code" 模式, 对 NeoTrix DESIGN.md 补齐:
1. dark 主题完整 token (Snowfield White 只有 light)
2. components 模式库 (12 复用组件 base class)
3. dont 反模式清单 (从 dev-rules 提炼 UI 相关项)
4. motion 时序规范
5. a11y 基线

## 四、路线图(后续会话)

1. **GUI/TUI 共享运行时** — `neotrix serve` 单运行时, CLI 与桌面均为客户端;
   会话/计划/审批经 HTTP/SSE 连续。这是最大的架构演进方向。
2. **任务证据链** — tool call ↔ file diff ↔ test result 关联到任务实体,
   交付时可视化回放。
3. **Plan-in-repo** — 计划文件落项目 `.neotrix/plans/`, 进版本控制。
4. **Revert 能力** — 文件修改快照 + 一键回滚(对标 checkpoint 但细粒度到编辑)。
