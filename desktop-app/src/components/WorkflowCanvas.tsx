import { useState, useRef, useCallback } from 'react';

interface WorkflowNode {
  id: string;
  label: string;
  color: string;
  type: string;
  x: number;
  y: number;
}

interface WorkflowEdge {
  from: string;
  to: string;
}

const NODE_W = 130;
const NODE_H = 44;
const GAP_X = 180;
const GAP_Y = 80;

const DEFAULT_NODES: WorkflowNode[] = [
  { id: 'gather', label: 'Gather', color: '#6c8cff', type: 'input', x: 40, y: 20 },
  { id: 'reason', label: 'Reason', color: '#ff6cb4', type: 'process', x: 40, y: 20 + GAP_Y },
  { id: 'plan', label: 'Plan', color: '#c86cff', type: 'process', x: 40 + GAP_X, y: 20 },
  { id: 'act', label: 'Act', color: '#48b87a', type: 'process', x: 40 + GAP_X, y: 20 + GAP_Y },
  { id: 'reflect', label: 'Reflect', color: '#e8a030', type: 'output', x: 40 + GAP_X * 2, y: 20 + GAP_Y / 2 },
];

const DEFAULT_EDGES: WorkflowEdge[] = [
  { from: 'gather', to: 'reason' },
  { from: 'gather', to: 'plan' },
  { from: 'reason', to: 'act' },
  { from: 'plan', to: 'act' },
  { from: 'act', to: 'reflect' },
];

const NODE_TYPES = [
  { type: 'input', label: 'Input', color: '#6c8cff' },
  { type: 'process', label: 'Process', color: '#ff6cb4' },
  { type: 'output', label: 'Output', color: '#48b87a' },
  { type: 'decision', label: 'Decision', color: '#e8a030' },
];

export default function WorkflowCanvas() {
  const [nodes, setNodes] = useState<WorkflowNode[]>(DEFAULT_NODES);
  const [edges, setEdges] = useState<WorkflowEdge[]>(DEFAULT_EDGES);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [connectingFrom, setConnectingFrom] = useState<string | null>(null);
  const [dragging, setDragging] = useState<{ id: string; ox: number; oy: number } | null>(null);
  const canvasRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent, nodeId: string) => {
    if (e.button !== 0) return;
    const node = nodes.find(n => n.id === nodeId);
    if (!node) return;
    setDragging({ id: nodeId, ox: e.clientX - node.x, oy: e.clientY - node.y });
    setSelectedNode(nodeId);
    e.preventDefault();
  }, [nodes]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!dragging) return;
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    setNodes(prev => prev.map(n =>
      n.id === dragging.id
        ? { ...n, x: Math.max(0, e.clientX - rect.left - dragging.ox), y: Math.max(0, e.clientY - rect.top - dragging.oy) }
        : n
    ));
  }, [dragging]);

  const handleMouseUp = useCallback(() => {
    setDragging(null);
  }, []);

  const handlePortClick = useCallback((nodeId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (connectingFrom === null) {
      setConnectingFrom(nodeId);
    } else if (connectingFrom !== nodeId) {
      setEdges(prev => {
        const exists = prev.some(e => e.from === connectingFrom && e.to === nodeId);
        if (exists) return prev;
        return [...prev, { from: connectingFrom, to: nodeId }];
      });
      setConnectingFrom(null);
    } else {
      setConnectingFrom(null);
    }
  }, [connectingFrom]);

  const handleDeleteEdge = useCallback((from: string, to: string) => {
    setEdges(prev => prev.filter(e => !(e.from === from && e.to === to)));
  }, []);

  const handleAddNode = useCallback((type: string) => {
    const color = NODE_TYPES.find(t => t.type === type)?.color || '#888';
    const id = `${type}_${nodes.length}`;
    const col = (nodes.length % 3);
    const row = Math.floor(nodes.length / 3);
    setNodes(prev => [...prev, {
      id, label: type.charAt(0).toUpperCase() + type.slice(1),
      color, type,
      x: 40 + col * GAP_X,
      y: 40 + row * GAP_Y + 100,
    }]);
  }, [nodes]);

  const getNodeAnchor = (node: WorkflowNode, side: 'right' | 'left'): { x: number; y: number } => ({
    x: node.x + (side === 'right' ? NODE_W : 0),
    y: node.y + NODE_H / 2,
  });

  return (
    <div className="workflow-canvas-wrapper">
      <div className="workflow-canvas-header">
        <h3>Consciousness Flow</h3>
        <div className="workflow-node-palette">
          {NODE_TYPES.map(t => (
            <button key={t.type} className="workflow-palette-item" onClick={() => handleAddNode(t.type)}>
              <span className="workflow-palette-dot" style={{ background: t.color }} />
              {t.label}
            </button>
          ))}
          {nodes.length > 5 && (
            <button className="workflow-palette-clear" onClick={() => { setNodes(DEFAULT_NODES.map(n => ({...n}))); setEdges([...DEFAULT_EDGES]); }}>
              Reset
            </button>
          )}
        </div>
        {connectingFrom && (
          <span className="workflow-connecting-hint">
            Connecting from {nodes.find(n => n.id === connectingFrom)?.label}... click another node
          </span>
        )}
        <span className="workflow-canvas-stats">{nodes.length} nodes · {edges.length} edges</span>
      </div>

      <div
        className="workflow-canvas"
        ref={canvasRef}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <svg className="workflow-svg" width="100%" height="100%">
          {edges.map(edge => {
            const from = nodes.find(n => n.id === edge.from);
            const to = nodes.find(n => n.id === edge.to);
            if (!from || !to) return null;
            const start = getNodeAnchor(from, 'right');
            const end = getNodeAnchor(to, 'left');
            const midX = (start.x + end.x) / 2;
            return (
              <g key={`${edge.from}→${edge.to}`}>
                <path
                  d={`M ${start.x} ${start.y} C ${midX} ${start.y}, ${midX} ${end.y}, ${end.x} ${end.y}`}
                  fill="none" stroke={to.color} strokeWidth={1.5}
                  opacity={0.5}
                  markerEnd="url(#arrowhead)"
                />
                {/* Invisible wider path for click target */}
                <path
                  d={`M ${start.x} ${start.y} C ${midX} ${start.y}, ${midX} ${end.y}, ${end.x} ${end.y}`}
                  fill="none" stroke="transparent" strokeWidth={12}
                  style={{ cursor: 'pointer' }}
                  onClick={() => handleDeleteEdge(edge.from, edge.to)}>
                  <title>Delete edge</title>
                </path>
              </g>
            );
          })}
          <defs>
            <marker id="arrowhead" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
              <polygon points="0 0, 8 3, 0 6" fill="#666" />
            </marker>
          </defs>
        </svg>

        {/* Nodes */}
        {nodes.map(node => {
          const isSelected = selectedNode === node.id;
          const isConnecting = connectingFrom === node.id;
          return (
            <div
              key={node.id}
              className={`workflow-node ${isSelected ? 'selected' : ''} ${isConnecting ? 'connecting' : ''}`}
              style={{
                left: node.x, top: node.y,
                width: NODE_W, height: NODE_H,
                borderColor: node.color,
                background: isSelected ? `${node.color}22` : undefined,
              }}
              onMouseDown={e => handleMouseDown(e, node.id)}
            >
              <span className="workflow-node-label">{node.label}</span>
              <span className="workflow-node-type">{node.type}</span>

              {/* Input port (left) */}
              <div
                className={`workflow-port workflow-port-left ${connectingFrom ? 'connectable' : ''}`}
                onClick={e => handlePortClick(node.id, e)}
                title="Connect from here"
              />

              {/* Output port (right) */}
              <div
                className={`workflow-port workflow-port-right ${connectingFrom === node.id ? 'active-port' : ''}`}
                onClick={e => handlePortClick(node.id, e)}
                title="Connect to here"
              />
            </div>
          );
        })}

        {nodes.length === 0 && (
          <div className="workflow-empty">
            <div className="workflow-empty-text">Empty canvas</div>
            <div className="workflow-empty-hint">Add nodes from the palette above</div>
          </div>
        )}
      </div>
    </div>
  );
}
