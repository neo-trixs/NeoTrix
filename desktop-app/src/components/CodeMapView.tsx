import { useState, useMemo } from 'react';
import { codeGraph, type CodeEntityType, type AbsorbSourceType } from '../core/code-graph-integrator';

const TYPE_GROUP: Record<string, { label: string; color: string; types: CodeEntityType[] }> = {
  structure: { label: 'Structure', color: '#6c8cff', types: ['file', 'module'] },
  logic: { label: 'Logic', color: '#ff6cb4', types: ['function', 'class', 'type'] },
  data: { label: 'Data', color: '#4caf7d', types: ['import', 'config'] },
  comm: { label: 'Communication', color: '#c86cff', types: ['call', 'route'] },
};

const KNOWLEDGE_COLORS: Record<AbsorbSourceType, string> = {
  paper: '#ff6cb4',
  repo: '#6c8cff',
  architecture: '#c86cff',
  pattern: '#48b87a',
  defect: '#e04e4e',
  evolution: '#e8a030',
  conversation: '#6ccfff',
};

export default function CodeMapView() {
  const [focusType, setFocusType] = useState<CodeEntityType | null>(null);
  const [minConf, setMinConf] = useState(0);
  const [showKnowledge, setShowKnowledge] = useState(false);

  const stats = useMemo(() => codeGraph.getStats(), []);
  const phases = useMemo(() => codeGraph.getPhases(), []);

  const entityTypes = useMemo(() => {
    const counts: Record<string, { count: number; color: string; label: string }> = {};
    for (const [grp, cfg] of Object.entries(TYPE_GROUP)) {
      let total = 0;
      for (const t of cfg.types) total += codeGraph.queryByConfidence(t, minConf).length;
      counts[grp] = { count: total, color: cfg.color, label: cfg.label };
    }
    return counts;
  }, [minConf]);

  const knowledgeItems = useMemo(() => {
    if (!showKnowledge) return [];
    return codeGraph.getKnowledgeByConfidence(minConf);
  }, [showKnowledge, minConf]);

  const filteredEntities = useMemo(() => {
    if (!focusType) return [];
    return codeGraph.queryByConfidence(focusType, minConf);
  }, [focusType, minConf]);

  return (
    <div className="codemap-view">
      <div className="codemap-header">
        <h3>Code Map</h3>
        <span className="codemap-stats">{stats.entityCount}e · {stats.edgeCount}eg · {stats.knowledgeCount}k</span>
      </div>

      {/* Controls */}
      <div className="codemap-controls">
        <div className="codemap-control-row">
          <span className="codemap-label">Min confidence</span>
          {[0, 0.5, 0.8].map(v => (
            <button
              key={v}
              className={`codemap-chip ${minConf === v ? 'active' : ''}`}
              onClick={() => setMinConf(v)}
            >{v === 0 ? 'All' : `≥${v}`}</button>
          ))}
        </div>
        <div className="codemap-control-row">
          <span className="codemap-label">Knowledge</span>
          <button
            className={`codemap-chip ${showKnowledge ? 'active' : ''}`}
            onClick={() => setShowKnowledge(v => !v)}
          >{showKnowledge ? 'Hide' : 'Show'}</button>
        </div>
      </div>

      {/* Entity type overview */}
      <div className="codemap-types">
        {Object.entries(entityTypes).map(([key, val]) => (
          <div
            key={key}
            className={`codemap-type-row ${focusType === null || TYPE_GROUP[key]?.types.includes(focusType) ? '' : 'dim'}`}
            onClick={() => {
              const types = TYPE_GROUP[key]?.types || [];
              setFocusType(prev => prev && types.includes(prev) ? null : types[0]);
            }}
          >
            <div className="codemap-type-header">
              <span className="codemap-type-dot" style={{ background: val.color }} />
              <span className="codemap-type-label">{val.label}</span>
              <span className="codemap-type-count">{val.count}</span>
            </div>
            <div className="codemap-type-bar">
              <div className="codemap-type-fill" style={{ width: `${Math.min(100, (val.count / Math.max(1, stats.entityCount)) * 100)}%`, background: val.color }} />
            </div>
          </div>
        ))}
      </div>

      {/* Knowledge nodes */}
      {showKnowledge && knowledgeItems.length > 0 && (
        <div className="codemap-knowledge">
          <div className="codemap-section-label">Knowledge</div>
          {knowledgeItems.slice(0, 8).map(k => (
            <div key={k.id} className="codemap-knowledge-item">
              <span className="codemap-knowledge-dot" style={{ background: KNOWLEDGE_COLORS[k.sourceType] || '#888' }} />
              <span className="codemap-knowledge-title">{k.title}</span>
              <span className="codemap-knowledge-conf">{k.confidence.toFixed(2)}</span>
            </div>
          ))}
        </div>
      )}

      {/* Focused entity list */}
      {filteredEntities.length > 0 && (
        <div className="codemap-entities">
          <div className="codemap-section-label">{focusType} · {filteredEntities.length}</div>
          {filteredEntities.slice(0, 12).map(e => (
            <div key={e.id} className="codemap-entity-item">
              <span className="codemap-entity-dot" style={{ background: codeGraph.getEntityColor(e.type) }} />
              <span className="codemap-entity-name">{e.name}</span>
              <span className="codemap-entity-conf">{e.confidence.toFixed(2)}</span>
            </div>
          ))}
        </div>
      )}

      {/* Absorption phases */}
      {phases.length > 0 && (
        <div className="codemap-phases">
          <div className="codemap-section-label">Pipeline</div>
          {phases.map((p, i) => (
            <div key={i} className="codemap-phase-item">
              <span className={`codemap-phase-status phase-${p.phase}`} />
              <span className="codemap-phase-name">{p.phase}</span>
              <span className="codemap-phase-detail">{p.detail}</span>
            </div>
          ))}
        </div>
      )}

      {stats.entityCount === 0 && (
        <div className="codemap-empty">
          <div className="codemap-empty-text">No entities indexed yet.</div>
          <div className="codemap-empty-hint">Run an absorption pipeline to build the code map.</div>
        </div>
      )}
    </div>
  );
}
