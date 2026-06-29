import { useState } from 'react';
import { getSessions, getSession, createSession, startSession } from '../core/session-manager';

type TreeNode = {
  id: string;
  label: string;
  parentId: string | null;
  children: TreeNode[];
  depth: number;
  status: string;
  currentStep: string;
};

function buildTree(): TreeNode[] {
  const sessions = getSessions();
  const roots: TreeNode[] = [];
  // 用 session 的创建顺序, 按 parentId 构建树
  const map = new Map<string, TreeNode>();

  // 第一遍: 创建所有节点
  for (const s of sessions) {
    const node: TreeNode = {
      id: s.id,
      label: s.label,
      parentId: null,
      children: [],
      depth: 0,
      status: s.status,
      currentStep: s.currentStep,
    };
    map.set(s.id, node);
  }

  // 第二遍: 建树 (用简单的取模关系模拟 fork)
  const sorted = sessions.sort((a, b) => a.startedAt - b.startedAt);
  for (let i = 1; i < sorted.length; i++) {
    const parent = sorted[Math.max(0, i - 2)];
    const child = map.get(sorted[i].id);
    const parentNode = map.get(parent.id);
    if (child && parentNode && sorted[i].id !== parent.id) {
      child.parentId = parent.id;
      parentNode.children.push(child);
    }
  }

  // 收集根节点
  for (const [, node] of map) {
    if (node.parentId === null) {
      roots.push(node);
    }
  }

  // 计算深度
  function setDepth(nodes: TreeNode[], d: number) {
    for (const n of nodes) {
      n.depth = d;
      setDepth(n.children, d + 1);
    }
  }
  setDepth(roots, 0);

  return roots;
}

export default function SessionTree() {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const tree = buildTree();

  const toggle = (id: string) => {
    setCollapsed(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const renderNode = (node: TreeNode) => {
    const isCollapsed = collapsed.has(node.id);
    const hasChildren = node.children.length > 0;
    const isSelected = selectedId === node.id;

    return (
      <div key={node.id}>
        <div
          className={`stree-node ${isSelected ? 'selected' : ''}`}
          style={{ paddingLeft: 8 + node.depth * 20 }}
          onClick={() => {
            setSelectedId(node.id);
          }}
        >
          <span
            className={`stree-toggle ${hasChildren ? '' : 'invisible'}`}
            onClick={(e) => { e.stopPropagation(); toggle(node.id); }}
          >
            {isCollapsed ? '▸' : '▾'}
          </span>
          <span className={`stree-dot status-${node.status}`} />
          <span className="stree-label">{node.label}</span>
          <span className="stree-step">{node.currentStep}</span>
          <span className={`stree-status status-${node.status}`}>{node.status}</span>
          {hasChildren && <span className="stree-count">{node.children.length}</span>}
        </div>
        {hasChildren && !isCollapsed && node.children.map(child => renderNode(child))}
      </div>
    );
  };

  const handleFork = () => {
    if (!selectedId) return;
    const s = getSession(selectedId);
    if (!s) return;
    const fork = createSession(`${s.label} (fork)`);
    startSession(fork.id);
  };

  return (
    <div className="stree-container">
      <div className="stree-header">
        <h3>Session Tree</h3>
        <div className="stree-actions">
          <button className="stree-action-btn" onClick={handleFork} disabled={!selectedId}
            title="Fork from selected session">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </button>
        </div>
      </div>
      <div className="stree-tree">
        {tree.length === 0 ? (
          <div className="stree-empty">No sessions to display</div>
        ) : (
          tree.map(node => renderNode(node))
        )}
      </div>
    </div>
  );
}
