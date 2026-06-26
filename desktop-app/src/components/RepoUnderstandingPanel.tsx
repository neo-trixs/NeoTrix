import { useState, useEffect } from 'react';
import { codeGraph, type CodeEntityType } from '../core/code-graph-integrator';

const ENTITY_TYPES: CodeEntityType[] = ['file', 'module', 'function', 'class', 'type', 'import', 'call', 'route', 'config'];
const ENTITY_COLORS: Record<CodeEntityType, string> = {
  file: '#6c8cff', module: '#8a6cff', function: '#ff6cb4',
  class: '#ffb86c', type: '#4caf7d', import: '#6ccfff',
  call: '#ff6c6c', route: '#c86cff', config: '#6cffd4',
};

export default function RepoUnderstandingPanel() {
  const [graph, setGraph] = useState(codeGraph.getStats());
  const [phases, setPhases] = useState(codeGraph.getPhases());
  const [entityList, setEntityList] = useState<Array<{ type: CodeEntityType; count: number }>>([]);
  const [showEntities, setShowEntities] = useState(false);

  useEffect(() => {
    const ti = setInterval(() => {
      setGraph(codeGraph.getStats());
      setPhases(codeGraph.getPhases());
      setEntityList(
        ENTITY_TYPES.map(t => ({ type: t, count: codeGraph.getEntitiesByType(t).length })),
      );
    }, 2000);
    return () => clearInterval(ti);
  }, []);

  return (
    <div className="dash-section">
      <h4>Repository Understanding</h4>

      {/* Code graph snapshot */}
      <div className="repo-snapshot">
        <div className="repo-stat-row">
          <span className="repo-stat-label">Entities</span>
          <span className="repo-stat-value">{graph.entityCount}</span>
        </div>
        <div className="repo-stat-row">
          <span className="repo-stat-label">Edges</span>
          <span className="repo-stat-value">{graph.edgeCount}</span>
        </div>
        <div className="repo-stat-row">
          <span className="repo-stat-label">Knowledge</span>
          <span className="repo-stat-value">{graph.knowledgeCount}</span>
        </div>
        <div className="repo-stat-row">
          <span className="repo-stat-label">Coverage</span>
          <span className="repo-stat-value" style={{ fontSize: 10 }}>
            {graph.coverage.files}f · {graph.coverage.modules}m · {graph.coverage.functions}c
          </span>
        </div>
        <div className="repo-stat-row">
          <span className="repo-stat-label">Defects</span>
          <span className="repo-stat-value" style={{ color: graph.defects.critical > 0 ? '#ff6c6c' : '#4caf7d' }}>
            {graph.defects.critical} critical / {graph.defects.total} total
          </span>
        </div>
      </div>

      {/* Phase pipeline status */}
      {phases.length > 0 && (
        <div className="repo-phases">
          <div className="repo-phase-header">Absorption Pipeline</div>
          <div className="repo-phase-track">
            {phases.map((p, i) => (
              <div key={p.phase} className="repo-phase-item">
                <span className="repo-phase-dot" style={{
                  background: ['#6c8cff', '#c86cff', '#ff6cb4', '#6cffd4'][i],
                }} />
                <span className="repo-phase-name">{p.phase}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Entity type distribution */}
      <button className="repo-toggle-btn" onClick={() => setShowEntities(v => !v)}>
        Entity Types {showEntities ? '▾' : '▸'}
      </button>
      {showEntities && (
        <div className="repo-entity-types">
          {entityList.filter(e => e.count > 0).map(e => (
            <div key={e.type} className="repo-entity-row">
              <span className="repo-entity-dot" style={{ background: ENTITY_COLORS[e.type] }} />
              <span className="repo-entity-label">{e.type}</span>
              <span className="repo-entity-count">{e.count}</span>
              <div className="repo-entity-bar">
                <div
                  className="repo-entity-fill"
                  style={{
                    width: `${(e.count / Math.max(1, graph.entityCount)) * 100}%`,
                    background: ENTITY_COLORS[e.type],
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      )}


    </div>
  );
}
