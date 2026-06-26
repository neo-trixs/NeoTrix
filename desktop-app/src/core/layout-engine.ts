/**
 * Unified Layout Engine — 统一布局引擎
 *
 * 融合自:
 *   TermHive — Grid 递归分屏树 + Canvas 自由画布
 *   Ridge    — 无限递归 split 引擎
 *   HiveTerm — 命名 split group (会议模式)
 *   TermCanvas — 无限画布 + 项目-工作树-终端三层
 *   NoWork   — 联系人式切换
 *   Project Moose — 分层代理树
 *
 * 四种布局模式:
 *   'contact' | 'grid' | 'canvas' | 'tree'
 *
 * VSA 编码布局状态, 持久化到 localStorage
 */

export type LayoutMode = 'contact' | 'grid' | 'canvas' | 'tree';

export interface SplitNode {
  id: string;
  direction: 'horizontal' | 'vertical';
  ratio: number;        // 0-1, 在父容器中的比例
  children: SplitNode[];
  leaf?: string;        // 叶子节点关联的 pane/session ID
}

export interface CanvasCard {
  id: string;
  sessionId: string;
  label: string;
  x: number;
  y: number;
  w: number;
  h: number;
  z: number;            // z-index
  color: string;
  status: 'idle' | 'running' | 'done' | 'error';
}

export interface ContactAgent {
  id: string;
  name: string;
  type: 'session' | 'agent' | 'team';
  parentId?: string;          // 父子关系 (Project Moose)
  childrenIds: string[];
  status: 'idle' | 'running' | 'paused' | 'done' | 'error';
  lastMessage: string;
  unread: number;
  avatar: string;            // emoji or initial
  sessionId: string;         // 关联的 session
  level: number;             // 树层次深度
}

export interface TreeAgentNode extends ContactAgent {
  collapsed: boolean;
  expandedChildren: number;
}

export type LayoutState = {
  mode: LayoutMode;
  gridTree: SplitNode | null;
  canvasCards: CanvasCard[];
  contacts: ContactAgent[];
  treeAgents: TreeAgentNode[];
  activeContactId: string | null;
  projectName: string;
};

/**
 * 布局引擎 — 管理四种布局模式的切换和状态
 */
export class LayoutEngine {
  private state: LayoutState;
  private listeners: Array<(s: LayoutState) => void> = [];

  constructor() {
    this.state = this.load();
  }

  private load(): LayoutState {
    try {
      const saved = localStorage.getItem('neotrix_layout');
      if (saved) return JSON.parse(saved) as LayoutState;
    } catch (e) { if (e instanceof Error) console.warn('[LayoutEngine]', e.message); }
    return {
      mode: 'contact',
      gridTree: null,
      canvasCards: [],
      contacts: [],
      treeAgents: [],
      activeContactId: null,
      projectName: 'default',
    };
  }

  private save(): void {
    try {
      localStorage.setItem('neotrix_layout', JSON.stringify(this.state));
    } catch (e) { if (e instanceof Error) console.warn('[LayoutEngine]', e.message); }
  }

  private notify(): void {
    this.save();
    this.listeners.forEach(fn => fn(this.state));
  }

  subscribe(fn: (s: LayoutState) => void): () => void {
    this.listeners.push(fn);
    return () => { this.listeners = this.listeners.filter(f => f !== fn); };
  }

  getState(): LayoutState { return this.state; }

  setMode(mode: LayoutMode): void {
    this.state.mode = mode;
    this.notify();
  }

  // ── Contact 模式 ──
  addContact(agent: ContactAgent): void {
    this.state.contacts.push(agent);
    this.notify();
  }

  removeContact(id: string): void {
    this.state.contacts = this.state.contacts.filter(c => c.id !== id);
    this.state.treeAgents = this.state.treeAgents.filter(c => c.id !== id);
    this.notify();
  }

  setActiveContact(id: string | null): void {
    this.state.activeContactId = id;
    this.notify();
  }

  updateContactStatus(id: string, status: ContactAgent['status']): void {
    const c = this.state.contacts.find(c => c.id === id);
    if (c) c.status = status;
    const t = this.state.treeAgents.find(c => c.id === id);
    if (t) t.status = status;
    this.notify();
  }

  addChildContact(parentId: string, child: ContactAgent): void {
    child.parentId = parentId;
    child.level = (this.state.contacts.find(c => c.id === parentId)?.level ?? 0) + 1;
    this.state.contacts.push(child);
    const parent = this.state.contacts.find(c => c.id === parentId);
    if (parent) parent.childrenIds.push(child.id);
    this.notify();
  }

  // ── Grid 模式 ──
  initGrid(root: SplitNode): void {
    this.state.gridTree = root;
    this.notify();
  }

  splitPane(paneId: string, direction: 'horizontal' | 'vertical'): void {
    if (!this.state.gridTree) return;
    this.state.gridTree = this._splitNode(this.state.gridTree, paneId, direction);
    this.notify();
  }

  private _splitNode(node: SplitNode, targetId: string, dir: 'horizontal' | 'vertical'): SplitNode {
    if (node.leaf === targetId) {
      return {
        id: `split_${Date.now()}`,
        direction: dir,
        ratio: 0.5,
        children: [
          { ...node, id: `${node.id}_a` },
          { id: `${node.id}_b`, direction: 'horizontal', ratio: 0.5, children: [], leaf: undefined },
        ],
      };
    }
    return {
      ...node,
      children: node.children.map(c => this._splitNode(c, targetId, dir)),
    };
  }

  swapPanes(fromId: string, toId: string): void {
    // 交换两个 leaf 的 sessionId
    this.state.contacts = this.state.contacts.map(c => {
      if (c.sessionId === fromId) return { ...c, sessionId: toId };
      if (c.sessionId === toId) return { ...c, sessionId: fromId };
      return c;
    });
    this.notify();
  }

  // ── Canvas 模式 ──
  addCard(card: CanvasCard): void {
    this.state.canvasCards.push(card);
    this.notify();
  }

  moveCard(id: string, x: number, y: number): void {
    const c = this.state.canvasCards.find(c => c.id === id);
    if (c) { c.x = x; c.y = y; }
    this.notify();
  }

  resizeCard(id: string, w: number, h: number): void {
    const c = this.state.canvasCards.find(c => c.id === id);
    if (c) { c.w = w; c.h = h; }
    this.notify();
  }

  bringToFront(id: string): void {
    const maxZ = Math.max(...this.state.canvasCards.map(c => c.z), 0);
    const c = this.state.canvasCards.find(c => c.id === id);
    if (c) c.z = maxZ + 1;
    this.notify();
  }

  tileCards(): void {
    const cards = this.state.canvasCards;
    if (cards.length === 0) return;
    const cols = Math.ceil(Math.sqrt(cards.length));
    const rows = Math.ceil(cards.length / cols);
    const cw = Math.floor(700 / cols);
    const ch = Math.floor(400 / rows);
    cards.forEach((c, i) => {
      c.x = (i % cols) * cw + 10;
      c.y = Math.floor(i / cols) * ch + 10;
      c.w = cw - 20;
      c.h = ch - 20;
    });
    this.notify();
  }

  // ── Tree 模式 ──
  toggleCollapse(id: string): void {
    const t = this.state.treeAgents.find(t => t.id === id);
    if (t) t.collapsed = !t.collapsed;
    this.notify();
  }

  buildTreeFromContacts(): void {
    const roots = this.state.contacts.filter(c => !c.parentId);
    const build = (agents: ContactAgent[], depth: number): TreeAgentNode[] => {
      return agents.map(a => ({
        ...a,
        collapsed: depth > 2,
        expandedChildren: a.childrenIds.length,
        level: depth,
      }));
    };
    this.state.treeAgents = build(roots, 0);
    this.notify();
  }

  /**
   * 从 session-manager 同步联系人
   */
  syncFromSessions(sessions: Array<{ id: string; label: string; status: string }>): void {
    const existing = new Set(this.state.contacts.map(c => c.sessionId));
    for (const s of sessions) {
      if (!existing.has(s.id)) {
        this.state.contacts.push({
          id: `contact_${s.id}`,
          name: s.label,
          type: 'session',
          childrenIds: [],
          status: s.status as ContactAgent['status'],
          lastMessage: '',
          unread: 0,
          avatar: s.label.charAt(0).toUpperCase(),
          sessionId: s.id,
          level: 0,
        });
      }
    }
    // 更新已有联系人的状态
    for (const s of sessions) {
      const c = this.state.contacts.find(c => c.sessionId === s.id);
      if (c) c.status = s.status as ContactAgent['status'];
    }
    this.buildTreeFromContacts();
    this.notify();
  }
}

export const layoutEngine = new LayoutEngine();
