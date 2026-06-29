/**
 * Memory Visualizer — 记忆三视图数据层
 *
 * 融合自:
 *   total-agent-memory — Graph(3D 力导向) / Hive(径向) / Matrix(邻接矩阵)
 *   agentmemory — 知识图谱 + 置信度评分 + 追踪
 *
 * 共享过滤: type / importance / search / orphans
 * 数据源: 模拟的知识图谱 (后续接入真实的 session 记忆)
 */

export type ViewMode = 'graph' | 'hive' | 'matrix';

export interface MemNode {
  id: string;
  label: string;
  type: 'session' | 'entity' | 'concept' | 'tool' | 'file' | 'agent';
  importance: number;  // 0-1
  group: number;       // 用于 hive plot 分组
  x?: number;
  y?: number;
  z?: number;
}

export interface MemEdge {
  source: string;
  target: string;
  weight: number;  // 0-1
  label?: string;
}

export interface FilterState {
  type: Set<string>;        // empty = all
  minImportance: number;    // 0-1
  search: string;
  hideOrphans: boolean;
  maxNodes: number;
}

export interface MemoryGraphData {
  nodes: MemNode[];
  edges: MemEdge[];
}

/**
 * 模拟知识图谱数据 (与 session-manager 同步)
 */
function generateMockData(): MemoryGraphData {
  const nodes: MemNode[] = [
    { id: 'ses_0001', label: 'Main Cycle', type: 'session', importance: 0.95, group: 0 },
    { id: 'ses_0002', label: 'Research', type: 'session', importance: 0.85, group: 0 },
    { id: 'ses_0003', label: 'Self-Evolution', type: 'session', importance: 0.90, group: 0 },
    { id: 'ses_0004', label: 'Memory Consolidation', type: 'session', importance: 0.80, group: 0 },
    { id: 'ent_001', label: 'HyperCube', type: 'entity', importance: 0.92, group: 1 },
    { id: 'ent_002', label: 'GWT', type: 'entity', importance: 0.88, group: 1 },
    { id: 'ent_003', label: 'SEAL', type: 'entity', importance: 0.85, group: 1 },
    { id: 'ent_004', label: 'E8 Engine', type: 'entity', importance: 0.90, group: 1 },
    { id: 'con_001', label: 'VSA 4096-bit', type: 'concept', importance: 0.95, group: 2 },
    { id: 'con_002', label: 'Negentropy', type: 'concept', importance: 0.93, group: 2 },
    { id: 'con_003', label: 'Consciousness Cycle', type: 'concept', importance: 0.91, group: 2 },
    { id: 'con_004', label: 'Self-World Boundary', type: 'concept', importance: 0.87, group: 2 },
    { id: 'tool_001', label: 'WebSearch', type: 'tool', importance: 0.78, group: 3 },
    { id: 'tool_002', label: 'CodeSearch', type: 'tool', importance: 0.75, group: 3 },
    { id: 'tool_003', label: 'Planner', type: 'tool', importance: 0.72, group: 3 },
    { id: 'file_001', label: 'layout-engine.ts', type: 'file', importance: 0.70, group: 4 },
    { id: 'file_002', label: 'context-manager.ts', type: 'file', importance: 0.68, group: 4 },
    { id: 'file_003', label: 'memory-visualizer.ts', type: 'file', importance: 0.65, group: 4 },
    { id: 'agent_001', label: 'Layout Agent', type: 'agent', importance: 0.75, group: 5 },
    { id: 'agent_002', label: 'Context Agent', type: 'agent', importance: 0.73, group: 5 },
    { id: 'agent_003', label: 'Memory Agent', type: 'agent', importance: 0.71, group: 5 },
  ];

  const edges: MemEdge[] = [
    { source: 'ses_0001', target: 'ent_001', weight: 0.9, label: 'uses' },
    { source: 'ses_0001', target: 'ent_002', weight: 0.85, label: 'uses' },
    { source: 'ses_0001', target: 'ent_003', weight: 0.8, label: 'uses' },
    { source: 'ses_0002', target: 'con_001', weight: 0.9, label: 'explores' },
    { source: 'ses_0002', target: 'con_002', weight: 0.88, label: 'explores' },
    { source: 'ses_0003', target: 'ent_003', weight: 0.95, label: 'evolves' },
    { source: 'ses_0003', target: 'agent_001', weight: 0.7, label: 'spawns' },
    { source: 'ses_0003', target: 'agent_002', weight: 0.7, label: 'spawns' },
    { source: 'ses_0004', target: 'con_003', weight: 0.85, label: 'records' },
    { source: 'ses_0004', target: 'con_004', weight: 0.8, label: 'records' },
    { source: 'ent_001', target: 'con_001', weight: 0.95, label: 'implements' },
    { source: 'ent_002', target: 'con_003', weight: 0.9, label: 'implements' },
    { source: 'ent_004', target: 'con_002', weight: 0.88, label: 'drives' },
    { source: 'con_001', target: 'file_001', weight: 0.7, label: 'encoded_in' },
    { source: 'con_001', target: 'file_002', weight: 0.7, label: 'encoded_in' },
    { source: 'con_001', target: 'file_003', weight: 0.7, label: 'encoded_in' },
    { source: 'tool_001', target: 'ses_0002', weight: 0.8, label: 'used_by' },
    { source: 'tool_002', target: 'ses_0003', weight: 0.75, label: 'used_by' },
    { source: 'tool_003', target: 'ses_0001', weight: 0.78, label: 'used_by' },
    { source: 'agent_001', target: 'file_001', weight: 0.8, label: 'manages' },
    { source: 'agent_002', target: 'file_002', weight: 0.8, label: 'manages' },
    { source: 'agent_003', target: 'file_003', weight: 0.8, label: 'manages' },
    { source: 'ent_001', target: 'ent_004', weight: 0.85, label: 'integrates' },
    { source: 'ent_002', target: 'ent_001', weight: 0.8, label: 'talks_to' },
    { source: 'ses_0001', target: 'ses_0002', weight: 0.6, label: 'related' },
    { source: 'ses_0003', target: 'ses_0004', weight: 0.55, label: 'related' },
  ];

  return { nodes, edges };
}

export class MemoryVisualizer {
  private data: MemoryGraphData;
  private filter: FilterState;

  constructor() {
    this.data = generateMockData();
    this.filter = {
      type: new Set(),
      minImportance: 0,
      search: '',
      hideOrphans: false,
      maxNodes: 100,
    };
  }

  setFilter(f: Partial<FilterState>): void {
    Object.assign(this.filter, f);
  }

  getFilter(): FilterState {
    return { ...this.filter, type: new Set(this.filter.type) };
  }

  /**
   * 获取经过过滤后的图数据 (用于 graph/hive/matrix 三视图)
   */
  getFilteredData(): MemoryGraphData {
    let nodes = [...this.data.nodes];

    // type filter
    if (this.filter.type.size > 0) {
      nodes = nodes.filter(n => this.filter.type.has(n.type));
    }

    // importance filter
    if (this.filter.minImportance > 0) {
      nodes = nodes.filter(n => n.importance >= this.filter.minImportance);
    }

    // search filter
    if (this.filter.search.trim()) {
      const q = this.filter.search.toLowerCase();
      nodes = nodes.filter(n => n.label.toLowerCase().includes(q));
    }

    // orphan filter
    if (this.filter.hideOrphans) {
      const connected = new Set<string>();
      for (const e of this.data.edges) {
        if (nodes.find(n => n.id === e.source) && nodes.find(n => n.id === e.target)) {
          connected.add(e.source);
          connected.add(e.target);
        }
      }
      nodes = nodes.filter(n => connected.has(n.id));
    }

    // maxNodes
    if (nodes.length > this.filter.maxNodes) {
      nodes = nodes.slice(0, this.filter.maxNodes);
    }

    const nodeIds = new Set(nodes.map(n => n.id));
    const edges = this.data.edges.filter(e =>
      nodeIds.has(e.source) && nodeIds.has(e.target)
    );

    return { nodes, edges };
  }

  /**
   * 获取统计摘要
   */
  getStats() {
    const all = this.data;
    const byType = new Map<string, number>();
    for (const n of all.nodes) {
      byType.set(n.type, (byType.get(n.type) || 0) + 1);
    }
    return {
      totalNodes: all.nodes.length,
      totalEdges: all.edges.length,
      avgImportance: all.nodes.reduce((s, n) => s + n.importance, 0) / all.nodes.length,
      byType: Object.fromEntries(byType),
    };
  }

  /**
   * 获取颜色映射
   */
  getColorForType(type: MemNode['type']): string {
    const colors: Record<MemNode['type'], string> = {
      session: '#6c8cff',
      entity: '#8a6cff',
      concept: '#ff6cb4',
      tool: '#ffb86c',
      file: '#6cffaa',
      agent: '#6ccfff',
    };
    return colors[type] || '#888';
  }
}

export const memoryVisualizer = new MemoryVisualizer();
