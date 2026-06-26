import { useState, useRef, useEffect, useCallback } from 'react';
import { memoryVisualizer, ViewMode, MemNode, MemEdge, MemoryGraphData } from '../core/memory-visualizer';
import { socialIdentity } from '../core/social-identity';

const TYPE_COLORS: Record<string, string> = {
  session: '#4d7cff', entity: '#7a4dff', concept: '#ff3d8c',
  tool: '#ff9a3d', file: '#3dff85', agent: '#3db8ff',
};

/** 力导向布局一次迭代 */
function tickForceDirected(nodes: MemNode[], edges: MemEdge[], width: number, height: number, iterations: number): void {
  const k = Math.sqrt(width * height / nodes.length) * 0.8;
  const disp: { x: number; y: number }[] = nodes.map(() => ({ x: 0, y: 0 }));

  for (let iter = 0; iter < iterations; iter++) {
    // Coulomb repulsion
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        let dx = (nodes[j].x ?? 0) - (nodes[i].x ?? 0);
        let dy = (nodes[j].y ?? 0) - (nodes[i].y ?? 0);
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        const force = k * k / dist;
        dx /= dist; dy /= dist;
        disp[i].x -= dx * force; disp[i].y -= dy * force;
        disp[j].x += dx * force; disp[j].y += dy * force;
      }
    }

    // Hooke attraction along edges
    for (const e of edges) {
      const si = nodes.findIndex(n => n.id === e.source);
      const ti = nodes.findIndex(n => n.id === e.target);
      if (si < 0 || ti < 0) continue;
      let dx = (nodes[ti].x ?? 0) - (nodes[si].x ?? 0);
      let dy = (nodes[ti].y ?? 0) - (nodes[si].y ?? 0);
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      const force = (dist * dist) / k;
      dx /= dist; dy /= dist;
      disp[si].x += dx * force; disp[si].y += dy * force;
      disp[ti].x -= dx * force; disp[ti].y -= dy * force;
    }

    // Apply with cooling
    const cooling = 1 - iter / iterations;
    for (let i = 0; i < nodes.length; i++) {
      const d = Math.sqrt(disp[i].x * disp[i].x + disp[i].y * disp[i].y) || 1;
      nodes[i].x = (nodes[i].x ?? 0) + (disp[i].x / d) * Math.min(d, 20) * cooling;
      nodes[i].y = (nodes[i].y ?? 0) + (disp[i].y / d) * Math.min(d, 20) * cooling;
      nodes[i].x = Math.max(10, Math.min(width - 10, nodes[i].x ?? 0));
      nodes[i].y = Math.max(10, Math.min(height - 10, nodes[i].y ?? 0));
    }
  }
}

/** D3 hive plot 布局 */
function layoutHive(nodes: MemNode[], radius: number): void {
  const groups = new Map<number, MemNode[]>();
  for (const n of nodes) {
    const g = n.group ?? 0;
    if (!groups.has(g)) groups.set(g, []);
    groups.get(g)!.push(n);
  }
  const groupKeys = Array.from(groups.keys()).sort();
  const angleStep = (Math.PI * 2) / groupKeys.length;
  for (let gi = 0; gi < groupKeys.length; gi++) {
    const members = groups.get(groupKeys[gi])!;
    const angle = angleStep * gi - Math.PI / 2;
    const cx = Math.cos(angle) * radius;
    const cy = Math.sin(angle) * radius;
    const r2 = 80 + Math.random() * 40;
    members.forEach((n, i) => {
      const a = angle + (i / members.length - 0.5) * 0.3;
      n.x = cx + Math.cos(a) * r2;
      n.y = cy + Math.sin(a) * r2;
    });
  }
}

/** Adjacency matrix 布局 — 按类型分组排列节点 */
function layoutMatrix(nodes: MemNode[]): void {
  const cols = Math.ceil(Math.sqrt(nodes.length));
  nodes.forEach((n, i) => {
    n.x = 20 + (i % cols) * 30;
    n.y = 20 + Math.floor(i / cols) * 30;
  });
}

export default function MemoryGraph() {
  const [viewMode, setViewMode] = useState<ViewMode>('graph');
  const [filterType, setFilterType] = useState<string>('all');
  const [minImp, setMinImp] = useState(0);
  const [searchQ, setSearchQ] = useState('');
  const [hideOrphans, setHideOrphans] = useState(false);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [data, setData] = useState<MemoryGraphData>({ nodes: [], edges: [] });

  // Apply filters and compute layout
  useEffect(() => {
    memoryVisualizer.setFilter({
      type: filterType === 'all' ? new Set() : new Set([filterType]),
      minImportance: minImp / 100,
      search: searchQ,
      hideOrphans,
      maxNodes: 60,
    });
    const filtered = memoryVisualizer.getFilteredData();

    // Initialize positions
    const { width, height } = canvasRef.current
      ? { width: canvasRef.current.width, height: canvasRef.current.height }
      : { width: 600, height: 400 };
    const nodes = filtered.nodes.map((n) => ({
      ...n,
      x: n.x ?? (Math.random() * width * 0.6 + width * 0.2),
      y: n.y ?? (Math.random() * height * 0.6 + height * 0.2),
      z: n.z ?? 0,
    }));

    if (viewMode === 'graph') tickForceDirected(nodes, filtered.edges, width, height, 80);
    else if (viewMode === 'hive') layoutHive(nodes, 120);
    else if (viewMode === 'matrix') layoutMatrix(nodes);

    setData({ nodes, edges: filtered.edges });
  }, [viewMode, filterType, minImp, searchQ, hideOrphans]);

  // Render canvas
  const renderCanvas = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const w = rect.width;
    const h = rect.height;

    ctx.clearRect(0, 0, w, h);

    // VSA identity fingerprint overlay
    socialIdentity.init();
    const vsaHex = socialIdentity.vsaId;
    ctx.save();
    ctx.font = '9px "SF Mono", "Fira Code", monospace';
    ctx.fillStyle = 'rgba(100,180,255,0.08)';
    ctx.textBaseline = 'top';
    // Render first 64 chars of VSA hex as a subtle watermark
    for (let i = 0; i < Math.min(vsaHex.length, 80); i += 40) {
      ctx.fillText(vsaHex.slice(i, i + 40), 16, 8 + i / 2);
    }
    ctx.restore();

    // Edges
    for (const e of data.edges) {
      const s = data.nodes.find(n => n.id === e.source);
      const t = data.nodes.find(n => n.id === e.target);
      if (!s || !t) continue;
      ctx.beginPath();
      ctx.moveTo(s.x ?? 0, s.y ?? 0);
      if (viewMode === 'hive') {
        // Curve along arc
        const cx = ((s.x ?? 0) + (t.x ?? 0)) / 2;
        const cy = ((s.y ?? 0) + (t.y ?? 0)) / 2 - 20;
        ctx.quadraticCurveTo(cx, cy, t.x ?? 0, t.y ?? 0);
      } else {
        ctx.lineTo(t.x ?? 0, t.y ?? 0);
      }
      ctx.strokeStyle = `rgba(255,255,255,${e.weight * 0.3})`;
      ctx.lineWidth = Math.max(0.5, e.weight * 2);
      ctx.stroke();
    }

    // Nodes
    for (const n of data.nodes) {
      const r = 3 + n.importance * 4;
      const color = TYPE_COLORS[n.type] || '#888';
      const alpha = n.importance * 0.5 + 0.5;

      // Glow
      const grad = ctx.createRadialGradient(n.x ?? 0, n.y ?? 0, 0, n.x ?? 0, n.y ?? 0, r * 2);
      grad.addColorStop(0, color + '66');
      grad.addColorStop(1, color + '00');
      ctx.beginPath();
      ctx.arc(n.x ?? 0, n.y ?? 0, r * 2, 0, Math.PI * 2);
      ctx.fillStyle = grad;
      ctx.fill();

      // Dot
      ctx.beginPath();
      ctx.arc(n.x ?? 0, n.y ?? 0, r, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.globalAlpha = alpha;
      ctx.fill();
      ctx.globalAlpha = 1;

      // Label (only for important nodes)
      if (n.importance > 0.6) {
        ctx.font = '8px system-ui, sans-serif';
        ctx.fillStyle = 'rgba(255,255,255,0.6)';
        ctx.fillText(n.label, (n.x ?? 0) + r + 3, (n.y ?? 0) + 3);
      }
    }
  }, [data, viewMode]);

  useEffect(() => { renderCanvas(); }, [renderCanvas]);

  const stats = memoryVisualizer.getStats();

  return (
    <div className="memgraph-container">
      <div className="memgraph-header">
        <h3>Memory</h3>
        <div className="memgraph-view-tabs">
          {(['graph', 'hive', 'matrix'] as ViewMode[]).map(m => (
            <button key={m} className={`memgraph-view-tab ${viewMode === m ? 'active' : ''}`}
              onClick={() => setViewMode(m)}>
              {m === 'graph' ? '◉' : m === 'hive' ? '◔' : '⊞'} {m}
            </button>
          ))}
        </div>
      </div>

      {/* Filters */}
      <div className="memgraph-filters">
        <div className="memgraph-filter-row">
          <select className="memgraph-select" value={filterType} onChange={e => setFilterType(e.target.value)}>
            <option value="all">All types</option>
            {Object.keys(TYPE_COLORS).map(t => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
          <input className="memgraph-search" placeholder="Search..." value={searchQ}
            onChange={e => setSearchQ(e.target.value)} />
        </div>
        <div className="memgraph-filter-row">
          <label className="memgraph-label">Min importance: {minImp}%</label>
          <input type="range" min={0} max={100} value={minImp}
            onChange={e => setMinImp(Number(e.target.value))}
            className="memgraph-slider" />
          <label className="memgraph-check" onClick={() => setHideOrphans(v => !v)}>
            <input type="checkbox" checked={hideOrphans} onChange={() => {}} />
            Hide orphans
          </label>
        </div>
      </div>

      {/* Canvas */}
      <div className="memgraph-canvas-wrap">
        <canvas ref={canvasRef} className="memgraph-canvas" />
        {data.nodes.length === 0 && (
          <div className="memgraph-empty">
            <div className="memgraph-empty-text">No nodes match filters</div>
          </div>
        )}
      </div>

      {/* Stats bar */}
      <div className="memgraph-stats">
        <span className="memgraph-stat">{data.nodes.length} nodes</span>
        <span className="memgraph-stat-sep" />
        <span className="memgraph-stat">{data.edges.length} edges</span>
        <span className="memgraph-stat-sep" />
        <span className="memgraph-stat">{stats.avgImportance.toFixed(2)} avg importance</span>
        <span className="memgraph-stat-sep" />
        <span className="memgraph-stat memgraph-stat-vsa">VSA 4096-bit</span>
      </div>
    </div>
  );
}
