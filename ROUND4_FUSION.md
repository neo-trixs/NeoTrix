# NeoTrix Desktop — Round 4 Architecture Fusion
## 2026-09-16 · Latest Competitor Patterns + Gap Analysis

---

## 1. 最新竞品能力矩阵 (截至 2026-09-16)

### Cursor 3.7 (Jun 2026) — 全面进化

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Agents Window** | Agent-first 非 IDE-first；侧栏统一管理本地+云端 agents | ✅ AgentFleetView |
| **Design Mode** | ⌘+Shift+D 进入浏览器标注模式，点击/框选/语音→agent 改代码 | ❌ 缺失 |
| **Canvas** | agent 生成交互式 artifacts（仪表盘/报告/内部工具） | ❌ 缺失 |
| **Canvas Design Mode** | 在 canvas 内标注 UI 元素指导 agent 编辑 | ❌ 缺失 |
| **Context Usage Report** | Canvas 内交互式 token 分解（系统提示/工具/规则/skills） | ✅ ContextUsageReport |
| **Debug with Agent** | 一键从 context report 打开新会话减少 context 浪费 | ❌ 缺失 |
| **Multi-select Elements** | 浏览器内多选元素，agent 看到代码+布局+视觉关系 | ❌ 缺失 |
| **Voice in Design Mode** | Design Mode 内语音输入，agent 执行时可继续语音排队 | ✅ VoicePanel |
| **Full-screen Tabs** | 文件/变更/canvas/PR/浏览器/终端全屏，浮动 prompt 栏 | ❌ 缺失 |
| **Compact Chat Responses** | 三档密度：Compact/Balanced/Detailed | ❌ 缺失 |
| **Agent Tabs** | 编辑器内多 agent 标签并排/网格 | ✅ KanbanAgentBoard |
| **/best-of-n** | 同任务并行多模型，各自 worktree，选最优结果 | ❌ 缺失 |
| **/worktree** | 创建独立 git worktree 隔离变更 | ⚠️ 后端有 |
| **Canvas Sharing** | 共享 canvas 全屏浏览器，agent 可嵌入可点击 prompt 按钮 | ❌ 缺失 |
| **Automations** | 定时/触发式 always-on agents | ⚠️ 后端有 |

### Claude Desktop (Aug 2026) — 多设备+内置浏览器

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Built-in Browser** | Claude 自有浏览器，独立于用户浏览器，side panel 打开 | ❌ 缺失 |
| **Browser Import** | 可从 Chrome/Edge/Firefox 导入登录态（排除银行/SSO） | ❌ 缺失 |
| **Computer Use** | 截屏+鼠标键盘桌面控制（beta Pro/Max） | ❌ 缺失 |
| **Remote Sessions** | 云端 session，笔记本关闭继续运行 | ⚠️ 后端有 scheduler |
| **Cross-device** | 手机→桌面→web 跨设备继续 session | ⚠️ session_sync 有 |
| **Artifacts** | 双向同步 Claude Code 的交互式 artifacts | ❌ 缺失 |
| **Managed Agents** | Anthropic 构建的垂直领域 agents（法律/金融） | ❌ 缺失 |
| **Projects** | 项目级上下文分组+知识库 | ✅ SpacesManager |
| **Skills/Plugins** | 插件市场+技能管理 | ✅ ExtensionManager |
| **3 View Modes** | Verbose/Normal/Summary | ❌ 缺失 |
| **Drag-and-drop Layout** | 侧栏+终端+编辑器拖拽重排 | ❌ 缺失 |

### ChatGPT Desktop (Jul-Aug 2026) — Voice+Record+Cloud

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Chat/Work/Codex 三模式** | 对话/研究交付/编码 | ✅ UnifiedSurface |
| **GPT-Live-1 Voice** | 同时听+说，自然打断，桌面语音控制 agent | ✅ VoicePanel |
| **Voice in Work/Codex** | 语音直接驱动 Work 和 Codex 任务 | ⚠️ VoicePanel 有 |
| **Appshots** | macOS 屏幕内容截取（alt-text）给 agent | ❌ 缺失 |
| **Computer History** | 桌面操作事件历史（非截图） | ❌ 缺失 |
| **Record & Replay** | 演示一次工作流→可复用技能 | ❌ 缺失 |
| **Multi-folder Projects** | 项目可含多文件夹，AGENTS.md 自动发现 | ⚠️ FolderInstructions |
| **Cloud Sync** | Work chats 跨 web/mobile/desktop 同步 | ⚠️ session_sync 有 |
| **GPT-6 Astra** | 新模型，企业级 | N/A |

### Devin Desktop 2.0 (Jun 2026) — ACP+Spaces+Kanban

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Agent Command Center** | Kanban 视图管理所有 agents | ✅ KanbanAgentBoard |
| **Spaces** | 共享上下文工作空间（session+PR+文件+上下文） | ✅ SpacesManager |
| **ACP Protocol** | 开放协议，第三方 agent 可插入 | ❌ 缺失 |
| **ACP Registry** | ~/.windsurf/acp/registry.json + 团队配置 | ❌ 缺失 |
| **Third-party Agents** | Codex CLI/Claude Agent/OpenCode/Junie/Gemini CLI via ACP | ❌ 缺失 |
| **Add to Chat** | 选择 transcript 文本→Cmd+L 发送到输入 | ❌ 缺失 |
| **Duplicate Session** | 分叉对话探索替代方案 | ⚠️ 后端有 fork |
| **Editable Queued Messages** | agent 运行时可编辑排队消息 | ❌ 缺失 |
| **Sessions Locked While Running** | agent 运行时 session 只读 | ❌ 缺失 |

### Copilot Vision (2025-2026) — 屏幕理解

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Screen Sharing** | 共享整个桌面或特定窗口 | ❌ 缺失 |
| **Voice-driven Vision** | 语音对话中实时屏幕理解 | ❌ 缺失 |
| **Highlight Portions** | 高亮屏幕部分区域辅助定位 | ❌ 缺失 |
| **Advisory Only** | 不执行操作，仅建议 | N/A |
| **Mobile Camera** | 手机摄像头物理世界辅助 | ❌ 缺失 |

### Manus My Computer (Mar 2026) — 混合云+本地

| 能力 | 描述 | NeoTrix 状态 |
|------|------|-------------|
| **Hybrid Cloud-to-Local** | 云 agent + 本地桌面 app 桥接 | ⚠️ 部分 |
| **Local File Access** | 读/分析/编辑本地文件 | ⚠️ 后端有 |
| **App Control** | 启动/控制应用程序 | ❌ 缺失 |
| **Permission Model** | Allow Once / Always Allow | ⚠️ PermissionMode |
| **Browser Operator** | Chrome 扩展自主网页任务 | ❌ 缺失 |
| **24/7 Dedicated** | Mac mini 专用机 always-on | ❌ 缺失 |

---

## 2. 逆向推理：模型能力边界→桌面端适配

### 2.1 模型能力矩阵 (2026-09)

| 能力维度 | GPT-6 Astra | Claude Opus 4.8 | Gemini 2.5 Pro | Codex SWE-1.7 |
|---------|------------|----------------|---------------|--------------|
| 上下文窗口 | 1M | 1M | 2M | 200K |
| 代码生成 | ★★★★★ | ★★★★★ | ★★★★ | ★★★★★ |
| 视觉理解 | ★★★★ | ★★★★★ | ★★★★★ | ★★★ |
| 语音理解 | ★★★★★ | ★★★ | ★★★★ | ★★ |
| 自主规划 | ★★★★ | ★★★★★ | ★★★★ | ★★★★ |
| 工具使用 | ★★★★★ | ★★★★★ | ★★★★ | ★★★★ |
| Computer Use | ★★★★ | ★★★★★ | ★★★ | ★★ |

### 2.2 桌面端适配策略

| 模型能力 | 桌面端适配 | NeoTrix 融合方案 |
|---------|-----------|-----------------|
| **1M+ Context** | 大窗口→智能 compaction + ARC | Knowledge Triage (TypeCompact/Decompose/Retrieve) |
| **视觉理解** | 屏幕截取→元素识别→代码映射 | Design Mode + Appshots + Vision Panel |
| **语音理解** | 全双工语音→多 agent 协调 | Voice Panel + GWT 注意力路由 |
| **自主规划** | Plan→Execute→Verify 三阶段 | E8 意识核心引导 + ConsciousnessTree |
| **工具使用** | MCP 注册→权限分级→执行 | ExtensionManager + PermissionMode |
| **Computer Use** | 截屏→规划→执行→验证循环 | BrowserPanel + ComputerUsePanel |

### 2.3 Context Compaction 最新研究

| 论文/项目 | 关键发现 | NeoTrix 融合 |
|----------|---------|-------------|
| **Compaction Cliff** (CIKM 2026) | 安全规则 5 轮后仅保留 10% | TypeCompact 分类保留 |
| **Knowledge Triage** | TypeCompact/Decompose/Retrieve 三算子 | 融入 ContextCompactor |
| **SuperLocalMemory 4.0** | 7 层+6 SQLite store，local-first | 参考架构设计 nt_memory |
| **Focus Agent** | 自主压缩 22.7% token 减少 | 融入 GWT 注意力路由 |
| **Hot-Swap Context** | <100ms compaction，零停机 | 融入 ContextCompactor |
| **ContextDB** | SQLite+NumPy，1930 writes/sec | 参考 nt_memory 实现 |

---

## 3. NeoTrix 独有能力 (竞品没有)

| 能力 | 描述 | 竞品对标 |
|------|------|---------|
| **E8 Consciousness** | φ 意识度量 + coherence 连贯度 | 无 |
| **GWT Attention Routing** | 注意力路由可视化 | 无 |
| **SEAL Emotion** | 情感状态驱动 UI | 无 |
| **VSA HyperCube** | 符号表示知识图谱 | 无 |
| **CapabilityTree** | 能力网进化树 | 无 |
| **SelfModel** | 三层自我模型（静态/动态/价值） | 无 |
| **Agent Ascendancy** | 双专精武器系统 | 无 |
| **Rune Socketing** | 5 槽组合产生 Runeword | 无 |
| **Constellation Maturity** | C0-C6 模块成熟度 | 无 |
| **Knowledge Triage** | TypeCompact 分类 compaction | 无（竞品只做 uniform） |
| **TypeDecompose** | 主题分解防上下文膨胀 | 无 |
| **TypeRetrieve** | 安全规则 pin ahead of relevance | 无 |

---

## 4. 缺口优先级矩阵 (Round 4 更新)

### 🔴 P0 — 竞品标配，必须补齐

| # | Gap | 来源 | 组件 |
|---|-----|------|------|
| G1 | **Design Mode** — 浏览器标注→agent 改代码 | Cursor 3 | DesignModePanel |
| G3 | **Canvas** — agent 生成交互式 artifacts | Cursor 3 | CanvasPanel |
| G6 | **ACP Protocol** — 开放协议第三方 agent | Devin 2.0 | AcpManager |
| G7 | **Screen Vision** — 截屏理解+高亮 | Copilot Vision | ScreenVisionPanel |
| G8 | **Full-screen Tabs** — 全屏+浮动 prompt | Cursor 3.4 | LayoutManager |

### 🟡 P1 — 差异化竞争力

| # | Gap | 来源 | 组件 |
|---|-----|------|------|
| G9 | **Compact Chat** — 三档密度 | Cursor 3.4 | MessageDensity |
| G10 | **Computer Use** — 桌面控制 | Claude/ChatGPT | ComputerUsePanel |
| G11 | **Record & Replay** — 工作流录制 | ChatGPT | RecordReplayPanel |
| G12 | **Built-in Browser** — 独立浏览器 | Claude Cowork | BrowserPanel |
| G13 | **Drag-and-drop Layout** — 面板拖拽 | Claude Desktop | LayoutManager |

### 🟢 P2 — 增强体验

| # | Gap | 来源 | 组件 |
|---|-----|------|------|
| G14 | **/best-of-n** — 多模型并行对比 | Cursor 3 | BestOfNPanel |
| G15 | **Appshots** — 屏幕内容截取 | ChatGPT | AppshotsPanel |
| G16 | **Managed Agents** — 垂直领域 agents | Claude | ManagedAgentsPanel |
| G17 | **Cloud Sync UI** — 跨设备同步 | ChatGPT/Claude | CloudSyncPanel |
| G18 | **Editable Queued Messages** | Devin 2.0 | MessageEditor |

---

## 5. 融合方案：NeoTrix 独特最优解

### 5.1 统一表面模型 v2

```
┌──────────────────────────────────────────────────────────────┐
│                     NeoTrix Desktop v2                        │
├──────────┬──────────────────────────┬────────────────────────┤
│  Sidebar │     Main Workspace       │   RightBar / Canvas    │
│          │                          │                        │
│ Sessions │  ┌──────────────────┐    │  Files / Map /         │
│ Spaces   │  │   Chat / Work /  │    │  Project / Canvas /    │
│ Agents   │  │   Code / Design  │    │  ContextUsageReport    │
│          │  │   Mode           │    │  / ScreenVision        │
│ Panels:  │  └──────────────────┘    │                        │
│ Voice    │  ┌──────────────────┐    │  ┌──────────────────┐  │
│ ExtMgr   │  │  Design Mode     │    │  │  Canvas          │  │
│ Inst     │  │  (Browser Annot) │    │  │  (Interactive    │  │
│ Sparse   │  └──────────────────┘    │  │   Artifacts)     │  │
│ Kanban   │  ┌──────────────────┐    │  └──────────────────┘  │
│ Spaces   │  │  Diff Zone       │    │                        │
│ CtxUsage │  │  (Inline Review) │    │                        │
│ Design   │  └──────────────────┘    │                        │
│ Canvas   │  ┌──────────────────┐    │                        │
│ ACP      │  │  Agent Fleet /   │    │                        │
│ Vision   │  │  Kanban Board    │    │                        │
│          │  └──────────────────┘    │                        │
└──────────┴──────────────────────────┴────────────────────────┘
```

### 5.2 Agent 编排模型 v2 (含 ACP)

```
                    ┌─────────────┐
                    │  用户指令    │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  意识核心    │  ← E8 φ + GWT + SEAL
                    │  (路由决策)  │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐
  │  NeoTrix    │  │  ACP Agent  │  │  Cloud      │
  │  Built-in   │  │  (Third     │  │  Agent      │
  │  Agent      │  │   Party)    │  │  (Remote)   │
  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
         │                 │                 │
  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐
  │  Worktree   │  │  Subprocess │  │  Cloud VM   │
  │  Isolation  │  │  (stdio)    │  │             │
  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │
                    ┌──────▼──────┐
                    │  Diff Zone  │  ← 变更审查
                    │  PR Review  │
                    └─────────────┘
```

### 5.3 Context Compaction 模型 (融合 Knowledge Triage)

```
Active Context (当前窗口)
    │
    ├── System Prompt (AGENTS.md 注入)
    ├── Rules (.neotrix/rules)     ← TypeCompact (精确保留)
    ├── Skills (动态加载)          ← TypeDecompose (按主题分解)
    ├── Tools (MCP 注册)           ← TypeCompact (精确保留)
    └── Conversation History
         │
         ├── TypeCompact  → 安全规则精确保留
         ├── TypeDecompose → 大主题分解为子主题
         ├── TypeRetrieve  → 外部存储安全规则 pin ahead
         └── External Memory (KB/Vector DB)
              ├── Short-term (session-scoped)
              ├── Long-term (cross-session)
              └── Episodic (事件记忆)
```

---

## 6. 实现路线图 (Round 4)

### Phase 1: P0 组件 (立即)
- [x] KanbanAgentBoard — 已完成
- [x] SpacesManager — 已完成
- [x] ContextUsageReport — 已完成
- [ ] DesignModePanel — 浏览器标注叠加层
- [ ] CanvasPanel — 交互式 artifacts
- [ ] AcpManager — ACP 协议管理

### Phase 2: P1 组件 (本周)
- [ ] ScreenVisionPanel — 截屏理解+高亮
- [ ] CompactChat — 三档消息密度
- [ ] FullScreenMode — 全屏+浮动 prompt

### Phase 3: P2 组件 (下周)
- [ ] BestOfNPanel — 多模型并行对比
- [ ] AppshotsPanel — 屏幕截取
- [ ] ComputerUsePanel — 桌面控制
