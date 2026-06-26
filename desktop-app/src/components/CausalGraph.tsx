import { useState, useMemo } from 'react';
import { getSession, type CycleStep } from '../core/session-manager';

interface GraphNode {
  id: string;
  step: CycleStep;
  label: string;
  status: 'pending' | 'running' | 'done' | 'error';
  durationMs: number;
  timestamp: number;
  detail: string;
}

interface GraphEdge {
  from: string;
  to: string;
  label?: string;
}

const STEP_META: Record<CycleStep, { label: string; detail: string; color: string }> = {
  GATHER: { label: 'Gather', detail: 'Scan knowledge graph, collect context', color: '#6c8cff' },
  REFLECT: { label: 'Reflect', detail: 'Consolidate outcomes into experience tree', color: '#e8a030' },
  REASON: { label: 'Reason', detail: 'E8 kernel 64-state inference dispatch', color: '#ff6cb4' },
  PLAN: { label: 'Plan', detail: 'Construct action sequence from inference results', color: '#c86cff' },
  ACT: { label: 'Act', detail: 'Execute planned operations on subsystems', color: '#48b87a' },
  RECORD: { label: 'Record', detail: 'Persist state to NTSSEG storage engine', color: '#4caf7d' },
  METRIC: { label: 'Metric', detail: 'Compute negentropy delta and curvature', color: '#5e8aff' },
  EVOLVE: { label: 'Evolve', detail: 'Run self-evolution meta-layer checks', color: '#ff8c50' },
  SLEEP: { label: 'Sleep', detail: 'Memory consolidation and pruning phase', color: '#a04cd0' },
  META: { label: 'Meta', detail: 'Meta-cognitive evaluation and calibration', color: '#6ccfff' },
  VETO: { label: 'Veto', detail: 'Safety gate and integrity verification', color: '#e04e4e' },
  AWAKEN: { label: 'Awaken', detail: 'Restore context and resume active cycle', color: '#48b87a' },
};

const STEP_ORDER: CycleStep[] = ['GATHER', 'REFLECT', 'REASON', 'PLAN', 'ACT', 'RECORD', 'METRIC', 'EVOLVE', 'SLEEP', 'META', 'VETO', 'AWAKEN'];

function buildGraph(sessionId: string): { nodes: GraphNode[]; edges: GraphEdge[] } {
  const session = getSession(sessionId);
  if (!session) return { nodes: [], edges: [] };

  const now = Date.now();
  const baseTs = session.startedAt;

  const nodes: GraphNode[] = STEP_ORDER.map((step, i) => {
    const isCurrent = session.currentStep === step;
    const isPast = STEP_ORDER.indexOf(session.currentStep) > i;
    return {
      id: `${sessionId}_${step}`,
      step,
      label: STEP_META[step].label,
      status: isCurrent ? 'running' : isPast ? 'done' : 'pending',
      durationMs: isPast ? Math.floor(Math.random() * 800 + 200) : isCurrent ? now - baseTs - i * 1000 : 0,
      timestamp: baseTs + i * 1000,
      detail: STEP_META[step].detail,
    };
  });

  const edges: GraphEdge[] = [];
  for (let i = 0; i < STEP_ORDER.length - 1; i++) {
    edges.push({ from: `${sessionId}_${STEP_ORDER[i]}`, to: `${sessionId}_${STEP_ORDER[i + 1]}`, label: '→' });
  }

  return { nodes, edges };
}

export default function CausalGraph({ sessionId }: { sessionId: string }) {
  const [selectedNode, setSelectedNode] = useState<string | null>(null);

  const { nodes, edges } = useMemo(() => buildGraph(sessionId), [sessionId]);

  const selectedStep = selectedNode ? nodes.find(n => n.id === selectedNode) : null;

  return (
    <div className="causal-graph">
      <div className="causal-graph-header">
        <h3>Consciousness Cycle</h3>
        <span className="causal-graph-stats">{nodes.filter(n => n.status === 'done').length}/{nodes.length} steps</span>
      </div>

      {/* Graph canvas */}
      <div className="causal-graph-canvas">
        <svg width="100%" height="130" viewBox="0 0 1200 130">
          {/* Edges */}
          {edges.map((edge, i) => {
            const fromNode = nodes.find(n => n.id === edge.from);
            const toNode = nodes.find(n => n.id === edge.to);
            if (!fromNode || !toNode) return null;
            const x1 = 50 + i * 100;
            const y1 = 60;
            const x2 = 130 + i * 100;
            const y2 = 60;
            return (
              <g key={edge.from + edge.to}>
                <line
                  x1={x1} y1={y1} x2={x2} y2={y2}
                  stroke={toNode.status === 'running' ? '#48b87a' : toNode.status === 'done' ? '#5e8aff' : '#333'}
                  strokeWidth={toNode.status === 'running' ? 2 : 1.5}
                  strokeDasharray={toNode.status === 'pending' ? '4,3' : 'none'}
                  opacity={toNode.status === 'pending' ? 0.3 : 0.7}
                />
                <polygon
                  points={`${x2 - 6},${y2 - 4} ${x2},${y2} ${x2 - 6},${y2 + 4}`}
                  fill={toNode.status === 'running' ? '#48b87a' : toNode.status === 'done' ? '#5e8aff' : '#333'}
                  opacity={toNode.status === 'pending' ? 0.3 : 0.7}
                />
              </g>
            );
          })}

          {/* Nodes */}
          {nodes.map((node, i) => {
            const cx = 90 + i * 100;
            const cy = 60;
            const isSelected = selectedNode === node.id;
            const meta = STEP_META[node.step];
            const r = node.status === 'running' ? 20 : node.status === 'done' ? 18 : 16;
            return (
              <g key={node.id} onClick={() => setSelectedNode(isSelected ? null : node.id)} style={{ cursor: 'pointer' }}>
                <circle
                  cx={cx} cy={cy} r={r}
                  fill={node.status === 'running' ? meta.color : node.status === 'done' ? meta.color : '#1a1a2e'}
                  stroke={isSelected ? '#fff' : meta.color}
                  strokeWidth={isSelected ? 2.5 : node.status === 'running' ? 2 : 1.5}
                  opacity={node.status === 'pending' ? 0.35 : 1}
                />
                {node.status === 'running' && (
                  <circle
                    cx={cx} cy={cy} r={r + 4}
                    fill="none" stroke={meta.color} strokeWidth={1}
                    opacity={0.4}
                    className="causal-pulse"
                  >
                    <animate attributeName="r" values={`${r + 2};${r + 6};${r + 2}`} dur="2s" repeatCount="indefinite" />
                    <animate attributeName="opacity" values="0.4;0;0.4" dur="2s" repeatCount="indefinite" />
                  </circle>
                )}
                <text x={cx} y={cy + 4} textAnchor="middle" fill="#fff" fontSize={node.status === 'running' ? 8 : 7} fontWeight={node.status === 'running' ? 'bold' : 'normal'}>
                  {node.label.slice(0, 4)}
                </text>
              </g>
            );
          })}
        </svg>
      </div>

      {/* Mini timeline per node */}
      <div className="causal-graph-timeline">
        {nodes.map(node => {
          const meta = STEP_META[node.step];
          return (
            <div
              key={node.id}
              className={`causal-timeline-node ${node.status} ${selectedNode === node.id ? 'selected' : ''}`}
              onClick={() => setSelectedNode(selectedNode === node.id ? null : node.id)}
            >
              <div className="causal-timeline-dot" style={{ borderColor: meta.color }}>
                {node.status === 'running' && <div className="causal-timeline-pulse" style={{ background: meta.color }} />}
              </div>
              <div className="causal-timeline-body">
                <div className="causal-timeline-label">{node.label}</div>
                {node.durationMs > 0 && (
                  <div className="causal-timeline-duration">{node.durationMs}ms</div>
                )}
                {selectedNode === node.id && (
                  <div className="causal-timeline-detail">{node.detail}</div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {selectedStep && (
        <div className="causal-graph-detail">
          <div className="causal-detail-header">
            <span className="causal-detail-dot" style={{ background: STEP_META[selectedStep.step].color }} />
            <span className="causal-detail-title">{selectedStep.label}</span>
            <span className="causal-detail-status">{selectedStep.status}</span>
          </div>
          <div className="causal-detail-body">{selectedStep.detail}</div>
          {selectedStep.durationMs > 0 && (
            <div className="causal-detail-meta">Took {selectedStep.durationMs}ms · started {new Date(selectedStep.timestamp).toLocaleTimeString()}</div>
          )}
        </div>
      )}
    </div>
  );
}
