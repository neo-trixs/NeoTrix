/**
 * Absorption Engine — 智能吸收引擎
 *
 * 扫描竞品项目 → 解构核心技术逻辑 → VSA 映射 → 融合 → 自动生成实现
 * 来源: TermHive, Project Moose, NoWork, Sinew, total-agent-memory,
 *       smelt, BB-Agent, Ridge, TermCanvas, HiveTerm, VoiceTree, Lattice,
 *       Claude Workflow Composer, AgnoLab, muxd, Heliox, AgentX, CodePilot
 */

export type CompetitorSource = {
  name: string;
  url: string;
  stars: number;
  category: 'layout' | 'memory' | 'context' | 'plugin' | 'session' | 'canvas' | 'contact' | 'swarm';
};

export type CoreLogic = {
  mechanism: string;        // 核心技术机制
  dataStructure: string;    // 数据结构描述
  algorithm: string;        // 算法/流程描述  
  tradeoffs: string;        // 取舍与边界
};

export type VsaMapping = {
  nativePrimitive: string;           // 映射到哪个 VSA 原语
  consciousnessStep?: number;         // 映射到意识循环第几步
  transformDescription: string;      // 如何从外部模式变换为 VSA
};

export type AbsorptionState = 'scanned' | 'deconstructed' | 'mapped' | 'fused' | 'implemented';

export type CompetitorPattern = {
  id: string;
  source: CompetitorSource;
  coreLogic: CoreLogic;
  vsaMapping: VsaMapping;
  state: AbsorptionState;
  absorbedAt?: number;
  confidence: number;  // 0-1
};

export type FusedFeature = {
  id: string;
  name: string;
  description: string;
  sourcePatterns: string[];        // 来自哪些竞品模式
  vsaPrimitives: string[];         // 用到的 VSA 原语
  implementationPath: string;      // 实现文件路径
  implementedAt?: number;
  active: boolean;
};

/**
 * 竞品模式目录 — 所有扫描结果的权威存储
 */
export class PatternCatalog {
  private patterns: Map<string, CompetitorPattern> = new Map();
  private fused: Map<string, FusedFeature> = new Map();

  register(pattern: CompetitorPattern): void {
    this.patterns.set(pattern.id, { ...pattern, absorbedAt: Date.now() });
  }

  get(id: string): CompetitorPattern | undefined {
    return this.patterns.get(id);
  }

  all(): CompetitorPattern[] {
    return Array.from(this.patterns.values());
  }

  byState(state: AbsorptionState): CompetitorPattern[] {
    return this.all().filter(p => p.state === state);
  }

  byCategory(category: CompetitorSource['category']): CompetitorPattern[] {
    return this.all().filter(p => p.source.category === category);
  }

  registerFused(feature: FusedFeature): void {
    this.fused.set(feature.id, { ...feature, implementedAt: Date.now() });
  }

  getFused(id: string): FusedFeature | undefined {
    return this.fused.get(id);
  }

  allFused(): FusedFeature[] {
    return Array.from(this.fused.values());
  }

  advanceState(id: string, next: AbsorptionState): void {
    const p = this.patterns.get(id);
    if (p) { p.state = next; }
  }

  /**
   * 内置模式 — 已从 100+ 竞品中蒸馏
   */
  static seed(): PatternCatalog {
    const c = new PatternCatalog();

    // ── Layout: Grid/Canvas 双布局 ──
    c.register({
      id: 'termhive_grid_canvas',
      source: { name: 'TermHive', url: 'github.com/0x0funky/TermHive', stars: 60, category: 'layout' },
      coreLogic: {
        mechanism: '递归分屏树 + 自由画布 — Grid 模式用树形 split 引擎, Canvas 模式用 8 方向自由拖放',
        dataStructure: 'SplitTree (Node={horizontal|vertical, children[], ratio}) + ZStack (Card={x,y,w,h,z})',
        algorithm: 'pointer movementX/Y delta → resize; drag header → swap agents in tree; persistent per-project JSON',
        tradeoffs: 'Grid 适合密集监控, Canvas 适合自由探索; Grid 保证无重叠, Canvas 可能重叠需 z-index',
      },
      vsaMapping: { nativePrimitive: 'SpatialArrangement', transformDescription: 'VSA 向量编码布局状态: {layoutMode, paneCount, activePane}' },
      state: 'deconstructed', confidence: 0.9,
    });

    c.register({
      id: 'ridge_recursive_splits',
      source: { name: 'Ridge', url: 'github.com/MySetsuna/ridge', stars: 0, category: 'layout' },
      coreLogic: {
        mechanism: '无限递归 split 引擎, 每个 pane 独立 PTY + WebGPU WASM 终端渲染',
        dataStructure: 'pane_tree.rs — RecursiveSplitTree<Node={Split|Leaf}, Direction, Ratio>',
        algorithm: 'workspace → named workspaces with independent pane trees; all processes kept alive across switches',
        tradeoffs: 'WebGPU 渲染开销 vs Canvas2D fallback; WASM 终端 ≈1.5MB',
      },
      vsaMapping: { nativePrimitive: 'SessionLayout', transformDescription: 'VSA 编码 workspace 拓扑' },
      state: 'deconstructed', confidence: 0.8,
    });

    c.register({
      id: 'hiveterm_meetings',
      source: { name: 'HiveTerm', url: 'hiveterm.com', stars: 0, category: 'layout' },
      coreLogic: {
        mechanism: '命名 split group — agent 协作会议模式: 一个 orchestrator 分配工作, 多个 worker 并行',
        dataStructure: 'MeetingConfig = {template, agents[], fileOwnership, crossReview, deliveryReport}',
        algorithm: '⌘⇧J → pick squad template → agents get exclusive file ownership → cross-review → delivery report',
        tradeoffs: '协作模式需要 agent 间的 MCP 通信, 增加了延迟但保证了文件一致性',
      },
      vsaMapping: { nativePrimitive: 'TeamOrchestration', transformDescription: 'VSA 编码协作拓扑: {orchestrator, workers[], ownership}' },
      state: 'deconstructed', confidence: 0.7,
    });

    // ── Contact: 联系人式 Agent ──
    c.register({
      id: 'nowork_contact_workers',
      source: { name: 'NoWork', url: 'github.com/yeyan00/nowork', stars: 3, category: 'contact' },
      coreLogic: {
        mechanism: '每个 Agent/Team 是聊天联系人, 点击即切换对话; Agent = 可对话的工作单元',
        dataStructure: 'Contact = {id, name, type, status, lastMessage, unread, avatar}',
        algorithm: '接触式交互: 点击联系人 → 加载/创建session → 聚焦该对话 → 可以随时切换',
        tradeoffs: '简单直观但缺少层级; 多团队可能需要文件夹/分组',
      },
      vsaMapping: { nativePrimitive: 'AgentIdentity', transformDescription: 'VSA 编码 agent 身份: {nameHash, role, capabilityVector}' },
      state: 'deconstructed', confidence: 0.9,
    });

    c.register({
      id: 'project_moose_hierarchy',
      source: { name: 'Project Moose', url: 'github.com/prob32/projectmoose', stars: 2, category: 'contact' },
      coreLogic: {
        mechanism: '分层代理树: Bull Moose → Junior Moose → Deer, 力导向布局实时展示',
        dataStructure: 'AgentTree = {parent, children[], role, status, tasks[]} — 纯节点图',
        algorithm: 'orchestrator spawns children → force-directed layout → real-time status badges → group chat via lasso select',
        tradeoffs: '层次清晰但大规模时力导向布局性能下降',
      },
      vsaMapping: { nativePrimitive: 'AgentHierarchy', transformDescription: 'VSA 编码父子关系: {parentHash ⊕ childHash}' },
      state: 'deconstructed', confidence: 0.85,
    });

    // ── Context: 自清理 ──
    c.register({
      id: 'sinew_clean_context',
      source: { name: 'Sinew', url: 'github.com/Paseru/sinew', stars: 65, category: 'context' },
      coreLogic: {
        mechanism: 'agent 自己标记无用 tool result → 替换为占位符, 只影响当前 turn (不破坏缓存)',
        dataStructure: 'tool_call_ids[] → 遍历 history → 替换 content 为 "[Tool result cleaned by you: irrelevant to future context.]"',
        algorithm: 'agent 调用 clean_context(tool_call_ids) → 对每个 id 在 history 中定位 → 内容替换为占位符 → 当前 turn 未缓存故无损',
        tradeoffs: '依赖 agent 的自我判断能力; 过于激进可能丢失有用信息; "when in doubt, keeps"',
      },
      vsaMapping: { nativePrimitive: 'ContextPrune', consciousnessStep: 5, transformDescription: '在意识步 4-5 之间插入上下文裁剪: {pruneTargets[], placeholder}' },
      state: 'deconstructed', confidence: 0.95,
    });

    c.register({
      id: 'opencode_acp_pruning',
      source: { name: 'OpenCode ACP Plugin', url: 'github.com/tuanhung303/opencode-agent-context-pruning', stars: 0, category: 'context' },
      coreLogic: {
        mechanism: '10 种 auto-supersede 策略: hash 去重 / file supersede / state query supersede / error truncation, 目标 ~50% 节省',
        dataStructure: 'SupersedeStrategy = {hash, file, todo, url, stateQuery, snapshot, retry, filePart, userCode, error}',
        algorithm: '每个策略是独立规则: 同 tool+同 params → 新覆盖旧; 写文件后清理旧读写; 同 URL fetch 保留最新',
        tradeoffs: '系统级强规则不依赖 agent 判断, 但可能过度删除; 每个策略有 risk rating (Low/Medium)',
      },
      vsaMapping: { nativePrimitive: 'AutoSupersede', consciousnessStep: 4, transformDescription: '在编码前运行去重: {strategyFlags[], hashCache}' },
      state: 'scanned', confidence: 0.85,
    });

    // ── Memory: 可视化 ──
    c.register({
      id: 'total_memory_3view',
      source: { name: 'total-agent-memory', url: 'github.com/vbcherepanov/total-agent-memory', stars: 0, category: 'memory' },
      coreLogic: {
        mechanism: '三视图: 3D WebGL 力导向 / D3 hive plot / Canvas adjacency matrix, 共享过滤控制',
        dataStructure: 'GraphData = {nodes[{id,type,importance}], edges[{source,target,weight}]}',
        algorithm: '6-stage hybrid retrieval (FTS5+BM25+dense+graph+CrossEncoder+MMR) → temporal knowledge graph → dashboard SSE push',
        tradeoffs: '3D 视图对 >3500 节点性能仍可接受; hive plot 适合类型分布; adjacency matrix 适合密集图',
      },
      vsaMapping: { nativePrimitive: 'KnowledgeGraph', transformDescription: 'VSA 向量编码节点语义, cosine 距离作为边权重' },
      state: 'deconstructed', confidence: 0.9,
    });

    c.register({
      id: 'agentmemory_graph',
      source: { name: 'agentmemory', url: 'github.com/rohitg00/agentmemory', stars: 24062, category: 'memory' },
      coreLogic: {
        mechanism: '知识图谱 + iii 控制台 + OpenTelemetry 追踪 + 置信度评分',
        dataStructure: 'KV store with knowledge graph edges; iii.dev console for tracing',
        algorithm: 'memory ops → OpenTelemetry spans → iii console visualization',
        tradeoffs: '24K 星但需要 iii 生态; OTel 追踪有额外开销',
      },
      vsaMapping: { nativePrimitive: 'MemoryTrace', transformDescription: 'VSA 标签携带追踪 ID' },
      state: 'scanned', confidence: 0.7,
    });

    // ── Plugin: Lua 插件系统 ──
    c.register({
      id: 'smelt_lua_plugins',
      source: { name: 'smelt', url: 'github.com/leonardcser/smelt', stars: 24, category: 'plugin' },
      coreLogic: {
        mechanism: 'Lua 嵌入式脚本引擎: keymaps, commands, autocmds, custom tools, custom modes',
        dataStructure: 'Plugin = {name, hooks: {keymap[], command[], autocmd[], tool[]}}; 从 ~/.config/smelt/init.lua 加载',
        algorithm: 'smelt init → load init.lua → register plugins → hook into agent loop: before_turn/after_turn/on_tool',
        tradeoffs: 'Lua VM 额外 ~500KB; 性能开销在 tool 调用边界; 需要沙箱安全',
      },
      vsaMapping: { nativePrimitive: 'VsaPlugin', transformDescription: 'VSA 原生插件 = {name, hooks[], vsaTriggers[]}' },
      state: 'deconstructed', confidence: 0.8,
    });

    // ── Session: 分支树 ──
    c.register({
      id: 'bbagent_session_forking',
      source: { name: 'BB-Agent', url: 'github.com/shuyhere/bb-agent', stars: 6, category: 'session' },
      coreLogic: {
        mechanism: 'SQLite-backed session with branching, forking, tree navigation',
        dataStructure: 'SessionTree = {root, nodes: {id, parentId, messages[], branchLabel}}; SQLite schema with parent_id FK',
        algorithm: '新消息 → 追加到当前节点; /fork → 创建子节点; /switch id → 切换上下文; tree view 显示全部分支',
        tradeoffs: 'SQLite 写放大; 分支深时 tree view 渲染慢; 需要手动清理 stale branches',
      },
      vsaMapping: { nativePrimitive: 'SessionBranch', transformDescription: 'VSA 编码会话路径: {root ⊕ parent ⊕ depth}' },
      state: 'deconstructed', confidence: 0.85,
    });

    c.register({
      id: 'muxd_git_like_sessions',
      source: { name: 'muxd', url: 'github.com/batalabs/muxd', stars: 0, category: 'session' },
      coreLogic: {
        mechanism: 'git 式 session 管理: commit-like persistence, checkout/reset/rebase 操作',
        dataStructure: 'Session = {messages[], checkpoint, branch}, Hub = {nodes{host, daemon}}',
        algorithm: 'save → commit to sqlite; checkout → restore state; hub architecture → connect from any device',
        tradeoffs: 'git 语义强大但学习曲线; hub 需要网络',
      },
      vsaMapping: { nativePrimitive: 'SessionVersion', transformDescription: 'VSA 哈希作为 session checkpoint ID' },
      state: 'scanned', confidence: 0.7,
    });

    // ── Canvas: 工作流构建器 ──
    c.register({
      id: 'claude_workflow_composer',
      source: { name: 'Claude Workflow Composer', url: 'github.com/fayzan123/claude-workflow-composer', stars: 10, category: 'canvas' },
      coreLogic: {
        mechanism: '拖拽 agent canvas → 导出 .claude/agents/ 文件; React Flow + 500ms 自动保存',
        dataStructure: 'Workflow = {nodes[{id,type,config}], edges[{from,to,trigger,context}]}',
        algorithm: 'drag agent from sidebar → canvas → connect → edit config → export to ~/.claude/agents/',
        tradeoffs: '导出依赖 Claude Code 文件格式; canvas 复杂时 React Flow 性能下降',
      },
      vsaMapping: { nativePrimitive: 'WorkflowGraph', transformDescription: 'VSA 编码工作流拓扑: 节点类型 hash ⊕ 边拓扑 hash' },
      state: 'scanned', confidence: 0.75,
    });

    // ── Swarm: 蜂群可视化 ──
    c.register({
      id: 'voicetree_spatial_graph',
      source: { name: 'Voicetree', url: 'github.com/voicetreelab/voicetree', stars: 0, category: 'swarm' },
      coreLogic: {
        mechanism: '空间图作为 IDE: markdown 节点 + agent 节点在同一个 Obsidian-like 图中, agent 可创建子图',
        dataStructure: 'Node = markdown | folder | agent; Edge = relation; 内存中 markdown hypergraph',
        algorithm: 'run(Agent, node) → 收集附近节点作为 context → spawn agent; agent 可 spawn sub-agent 到图',
        tradeoffs: '图复杂度随项目线性增长; 大量节点时力导向布局需要 GPU',
      },
      vsaMapping: { nativePrimitive: 'SpatialKnowledge', transformDescription: 'VSA 编码节点位置: {x,y} ⊕ semanticHash' },
      state: 'scanned', confidence: 0.7,
    });

    return c;
  }
}

/**
 * 融合引擎 — 合并多个竞品模式为 VSA 原生功能
 */
export class FusionEngine {
  private catalog: PatternCatalog;

  constructor(catalog: PatternCatalog) {
    this.catalog = catalog;
  }

  /**
   * 融合 TermHive Grid + Ridge RecursiveSplit + HiveTerm Meetings → 统一布局引擎
   */
  fuseLayoutEngine(): FusedFeature {
    const patterns = ['termhive_grid_canvas', 'ridge_recursive_splits', 'hiveterm_meetings'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'unified_layout_engine',
      name: 'Unified Layout Engine',
      description: '融合 Grid(递归分屏) + Canvas(自由画布) + Contact(联系人式) + Tree(分层树) 四种布局模式, VSA 编码布局状态',
      sourcePatterns: patterns,
      vsaPrimitives: ['SpatialArrangement', 'SessionLayout', 'TeamOrchestration', 'AgentIdentity', 'AgentHierarchy'],
      implementationPath: 'src/core/layout-engine.ts',
      active: true,
    };
  }

  /**
   * 融合 Sinew + OpenCode context pruning → 意识循环上下文裁剪步
   */
  fuseContextManager(): FusedFeature {
    const patterns = ['sinew_clean_context', 'opencode_acp_pruning'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'consciousness_context_prune',
      name: 'Consciousness Context Prune',
      description: '在意识循环步 4-5 之间插入上下文裁剪: agent 驱动 (Sinew clean_context) + 系统规则 (10 策略 auto-supersede)',
      sourcePatterns: patterns,
      vsaPrimitives: ['ContextPrune', 'AutoSupersede'],
      implementationPath: 'src/core/context-manager.ts',
      active: true,
    };
  }

  /**
   * 融合 total-agent-memory + agentmemory → 三视图记忆可视化
   */
  fuseMemoryVisualizer(): FusedFeature {
    const patterns = ['total_memory_3view', 'agentmemory_graph'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'memory_three_view',
      name: 'Memory Three-View',
      description: 'Graph(3D 力导向) / Hive(径向) / Matrix(邻接矩阵) 三视图, 共享过滤: type/importance/search/orphans',
      sourcePatterns: patterns,
      vsaPrimitives: ['KnowledgeGraph', 'MemoryTrace'],
      implementationPath: 'src/core/memory-visualizer.ts',
      active: true,
    };
  }

  /**
   * 融合 NoWork + Project Moose → 联系人式层次 Agent 列表
   */
  fuseContactHub(): FusedFeature {
    const patterns = ['nowork_contact_workers', 'project_moose_hierarchy'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'contact_hub',
      name: 'Contact Hub',
      description: '每个 Agent/Team 是聊天联系人 (NoWork), 支持分层父子关系 (Project Moose), 点击即切换',
      sourcePatterns: patterns,
      vsaPrimitives: ['AgentIdentity', 'AgentHierarchy'],
      implementationPath: 'src/components/ContactHub.tsx',
      active: true,
    };
  }

  /**
   * 融合 BB-Agent + muxd → 会话分支树
   */
  fuseSessionTree(): FusedFeature {
    const patterns = ['bbagent_session_forking', 'muxd_git_like_sessions'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'session_branch_tree',
      name: 'Session Branch Tree',
      description: '每个 session 可 fork 子分支, tree view 展示全部分支, VSA 编码路径做 checkpoint',
      sourcePatterns: patterns,
      vsaPrimitives: ['SessionBranch', 'SessionVersion'],
      implementationPath: 'src/core/session-tree.ts',
      active: true,
    };
  }

  /**
   * 融合 smelt Lua 插件 + Claude Workflow Composer + Voicetree → VSA 原生插件系统
   */
  fusePluginSystem(): FusedFeature {
    const patterns = ['smelt_lua_plugins', 'claude_workflow_composer', 'voicetree_spatial_graph'];
    patterns.forEach(id => this.catalog.advanceState(id, 'fused'));

    return {
      id: 'vsa_plugin_system',
      name: 'VSA Plugin System',
      description: 'VSA 原生插件: {name, hooks[], vsaTriggers[]}, 非 MCP/ACP 协议, 纯 VSA 向量通信',
      sourcePatterns: patterns,
      vsaPrimitives: ['VsaPlugin', 'WorkflowGraph', 'SpatialKnowledge'],
      implementationPath: 'src/core/plugin-system.ts',
      active: true,
    };
  }

  runAll(): FusedFeature[] {
    return [
      this.fuseLayoutEngine(),
      this.fuseContextManager(),
      this.fuseMemoryVisualizer(),
      this.fuseContactHub(),
      this.fuseSessionTree(),
      this.fusePluginSystem(),
    ];
  }
}

export const catalog = PatternCatalog.seed();
export const fusion = new FusionEngine(catalog);
export const fused = fusion.runAll();
