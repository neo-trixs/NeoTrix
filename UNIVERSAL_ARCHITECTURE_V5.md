# NeoTrix Desktop App — Universal Optimal Architecture v5
## 2026-09-16 · Post-Research Fusion · Gap Analysis + Implementation Roadmap

---

## 1. 竞品全景 (2026-09)

### Cursor 3 (April 2026)
| 能力 | 描述 | NeoTrix 对标 |
|------|------|-------------|
| **Agents Window** | Agent-first 非 IDE-first；侧栏统一管理本地+云端 agents | ✅ AgentFleetView 已有 |
| **Design Mode** | ⌘+Shift+D 进入浏览器标注模式，点击/框选/语音→agent 改代码 | ❌ 缺失 |
| **Canvas** | agent 生成交互式 artifacts（仪表盘/报告/内部工具） | ❌ 缺失 |
| **/best-of-n** | 同任务并行多模型，各自 worktree，选最优结果 | ❌ 缺失 |
| **Agent Tabs** | 编辑器内多 agent 标签并排/网格 | ⚠️ 部分（有 AgentFleetView） |
| **Context Usage Report** | Canvas 内交互式 token 分解（系统提示/工具/规则/skills） | ❌ 缺失 |
| **Automations** | 定时/触发式 always-on agents | ⚠️ 后端有 scheduler，前端缺 UI |
| **MCP Apps structured** | MCP 输出结构化内容（非纯文本） | ❌ 缺失 |
| **Voice in Design Mode** | Design Mode 内语音输入，agent 执行时可继续语音 | ✅ VoicePanel 已有 |

### Claude Desktop (2026)
| 能力 | 描述 | NeoTrix 对标 |
|------|------|-------------|
| **Chat/Work/Code 三标签** | 对话/研究交付/编码 三模式 | ✅ UnifiedSurface 已有 |
| **Cowork** | 后台自主 agent，VM 隔离，关闭笔记本继续运行 | ❌ 缺失（有 scheduler 无 VM） |
| **Built-in Browser** | Cowork 内置浏览器，导航/点击/填表 | ❌ 缺失 |
| **Sub-agents** | 主 agent 分解任务→并行子 agent | ⚠️ 后端 multi_agent 有，前端缺 UI |
| **Computer Use** | 截屏+鼠标键盘控制桌面 | ❌ 缺失 |
| **Persistent Thread** | 手机→桌面跨设备持续线程 | ⚠️ session_sync 有，缺跨设备 UI |
| **Scheduled Tasks** | 定时/按需任务调度 | ⚠️ 后端有，前端 ScheduledTasks 已有 |
| **Skills/Plugins** | 插件市场+技能管理 | ✅ ExtensionManager 已有 |
| **Memory** | 跨设备记忆，免费用户可用 | ⚠️ 后端 KB 有，前端 MemoryManager 有 |
| **Design** | Claude Design Canvas，双向同步 Claude Code | ❌ 缺失 |
| **3 View Modes** | Verbose/Normal/Summary | ❌ 缺失 |
| **Drag-and-drop Layout** | 侧栏+终端+编辑器拖拽重排 | ⚠️ 有侧栏拖拽，无面板拖拽 |

### ChatGPT Desktop (2026)
| 能力 | 描述 | NeoTrix 对标 |
|------|------|-------------|
| **Chat/Work/Codex 三模式** | 对话/研究交付/编码 | ✅ UnifiedSurface 已有 |
| **GPT-Live-1 Voice** | 同时听+说，自然打断，桌面语音控制 agent | ✅ VoicePanel 已有 |
| **Record & Replay** | 演示一次工作流→可复用技能 | ❌ 缺失 |
| **Computer History** | 桌面操作历史回放 | ❌ 缺失 |
| **Appshots** | macOS 屏幕内容截取（alt-text） | ❌ 缺失 |
| **Projects** | 项目级上下文分组 | ⚠️ 部分（有项目分组，缺 Projects UI） |
| **Cloud Sync** | 跨 web/mobile/desktop 同步 | ⚠️ session_sync 有，缺跨设备 UI |
| **Work Local + Cloud** | 本地/云端双模式 | ❌ 缺失 |

### Devin Desktop (June 2026)
| 能力 | 描述 | NeoTrix 对标 |
|------|------|-------------|
| **Agent Command Center** | Kanban 视图管理所有 agents | ⚠️ AgentFleetView 有，缺 Kanban |
| **Spaces** | 共享上下文的工作空间（session+PR+文件+上下文） | ❌ 缺失 |
| **ACP Protocol** | 开放协议，第三方 agent 可插入 | ❌ 缺失 |
| **Devin Local** | Rust 重写，30% token 高效 | N/A（NeoTrix 已是 Rust） |
| **Multi-Agent Orchestration** | 主 agent 管理并行子 agent VMs | ⚠️ 后端有，前端缺 UI |
| **Subagents** | 并行子 agent，各自隔离上下文 | ⚠️ 后端有，前端缺 UI |

---

## 2. 缺口矩阵 (Gap Analysis)

### 🔴 Critical Gaps (Must Have — 竞品标配)

| # | Gap | 竞品来源 | 优先级 |
|---|-----|---------|--------|
| G1 | **Design Mode** — 浏览器内点击/标注/语音→agent 改代码 | Cursor 3 | P0 |
| G2 | **Spaces** — 工作空间组织 agent sessions + PR + 文件 + 上下文 | Devin Desktop | P0 |
| G3 | **Canvas** — agent 生成交互式 artifacts（仪表盘/报告） | Cursor 3 | P0 |
| G4 | **Context Usage Report** — token 分解可视化 | Cursor 3 | P0 |
| G5 | **Kanban Agent View** — 按状态分列的 agent 管理 | Devin Desktop | P0 |

### 🟡 Important Gaps (Should Have — 差异化竞争力)

| # | Gap | 竞品来源 | 优先级 |
|---|-----|---------|--------|
| G6 | **Record & Replay** — 工作流录制→可复用技能 | ChatGPT | P1 |
| G7 | **Computer Use** — 截屏+鼠标键盘桌面控制 | Claude Cowork | P1 |
| G8 | **Built-in Browser** — 内置浏览器独立于用户浏览器 | Claude Cowork | P1 |
| G9 | **3 View Modes** — Verbose/Normal/Summary | Claude Desktop | P1 |
| G10 | **Drag-and-drop Panel Layout** — 面板自由拖拽重排 | Claude Desktop | P1 |

### 🟢 Nice-to-Have (Could Have — 增强体验)

| # | Gap | 竞品来源 | 优先级 |
|---|-----|---------|--------|
| G11 | **/best-of-n** — 多模型并行对比 | Cursor 3 | P2 |
| G12 | **Computer History** — 桌面操作历史 | ChatGPT | P2 |
| G13 | **Cloud Sync UI** — 跨设备同步管理 | ChatGPT/Claude | P2 |
| G14 | **Appshots** — 屏幕内容截取 | ChatGPT | P2 |
| G15 | **MCP Structured Output** — MCP 结构化内容渲染 | Cursor 3 | P2 |

---

## 3. NeoTrix 特有能力 (Already Have — 竞品没有)

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

---

## 4. 架构融合方案

### 4.1 统一表面模型 (Unified Surface Model)

```
┌─────────────────────────────────────────────────────┐
│                    NeoTrix Desktop                    │
├──────────┬──────────────────────────┬───────────────┤
│  Sidebar │     Main Workspace       │   RightBar    │
│          │                          │               │
│ Sessions │  ┌──────────────────┐    │  Files/Map/   │
│ Spaces   │  │   Chat / Work /  │    │  Project/     │
│ Agents   │  │   Code / Canvas  │    │  Canvas       │
│          │  └──────────────────┘    │               │
│ Panels:  │  ┌──────────────────┐    │  Context      │
│ Voice    │  │  Design Mode     │    │  Usage        │
│ ExtMgr   │  │  (Browser Annot) │    │  Report       │
│ Inst     │  └──────────────────┘    │               │
│ Sparse   │  ┌──────────────────┐    │               │
│ CIG      │  │  Diff Zone       │    │               │
│          │  │  (Inline Review) │    │               │
│          │  └──────────────────┘    │               │
└──────────┴──────────────────────────┴───────────────┘
```

### 4.2 Agent 编排模型 (Agent Orchestration Model)

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
              ┌────────────┼────────────┐
              │            │            │
       ┌──────▼──────┐ ┌──▼───┐ ┌──────▼──────┐
       │  Agent A    │ │ B    │ │  Agent C    │  ← 并行 agents
       │  (本地)     │ │(云端)│ │  (VM)       │
       └──────┬──────┘ └──┬───┘ └──────┬──────┘
              │            │            │
       ┌──────▼──────┐ ┌──▼───┐ ┌──────▼──────┐
       │  Worktree   │ │Cloud │ │  Sandbox    │  ← 隔离环境
       │  Isolation  │ │ VM   │ │  VM         │
       └──────┬──────┘ └──┬───┘ └──────┬──────┘
              │            │            │
              └────────────┼────────────┘
                           │
                    ┌──────▼──────┐
                    │  Diff Zone  │  ← 变更审查
                    │  PR Review  │
                    └─────────────┘
```

### 4.3 Context 管理模型 (Context Lifecycle)

```
Active Context (当前窗口)
    │
    ├── System Prompt (AGENTS.md 注入)
    ├── Rules (.neotrix/rules)
    ├── Skills (动态加载)
    ├── Tools (MCP 注册)
    └── Conversation History
         │
         ├── Sliding Window (最近 N 条)
         ├── Recursive Summarization (定期压缩)
         ├── ARC (Addressable Recall Compaction)
         │    ├── ID-addressable log
         │    └── Compact citations
         └── External Memory (KB/Vector DB)
              ├── Short-term (session-scoped)
              ├── Long-term (cross-session)
              └── Episodic (事件记忆)
```

---

## 5. 实现路线图

### Phase 1: Critical Gaps (本周)
- [ ] G5: Kanban Agent View — 将 AgentFleetView 升级为 Kanban 布局
- [ ] G2: Spaces — 工作空间组织
- [ ] G4: Context Usage Report — token 分解可视化

### Phase 2: Important Gaps (下周)
- [ ] G1: Design Mode — 浏览器标注叠加层
- [ ] G3: Canvas — 交互式 artifacts
- [ ] G9: 3 View Modes — Verbose/Normal/Summary

### Phase 3: Enhancement (两周内)
- [ ] G6: Record & Replay — 工作流录制
- [ ] G7: Computer Use — 桌面控制
- [ ] G8: Built-in Browser — 内置浏览器

---

## 6. NeoTrix 独特优势融合

将 E8 意识系统与竞品能力融合：

| 竞品能力 | NeoTrix 融合方案 |
|---------|-----------------|
| Cursor Design Mode | + E8 意识状态感知标注（φ 高时自动标注高优先级元素） |
| Devin Spaces | + Constellation 成熟度标记（C0-C6 标记空间内模块成熟度） |
| Claude Cowork | + SEAL 情感驱动任务优先级（confident→高信任自主执行） |
| ChatGPT Voice | + GWT 注意力路由（语音指令经注意力路由到最佳 agent） |
| Context Compaction | + ARC + VSA HyperCube 符号压缩（保留语义结构） |
| Agent Orchestration | + E8 引导者决策（意识核心决定路由策略） |
