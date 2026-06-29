import { useState, useCallback } from 'react';
import { contextManager, SupersedeStrategy } from '../core/context-manager';

const STRATEGY_LABELS: Record<SupersedeStrategy, string> = {
  hash: 'Hash Dedup',
  file: 'File Supersede',
  todo: 'Todo Merge',
  url: 'URL Dedup',
  stateQuery: 'State Query Merge',
  snapshot: 'Snapshot Cleanup',
  retry: 'Retry Merge',
  filePart: 'File Part Merge',
  userCode: 'User Code Cleanup',
  error: 'Error Truncation',
};

const STRATEGY_COLORS: Record<SupersedeStrategy, string> = {
  hash: '#6c8cff',
  file: '#8a6cff',
  todo: '#ff6cb4',
  url: '#ffb86c',
  stateQuery: '#6cffaa',
  snapshot: '#6ccfff',
  retry: '#ff6c6c',
  filePart: '#c86cff',
  userCode: '#ff9a6c',
  error: '#ff6c8a',
};

export default function ContextInspector() {
  const [stats, setStats] = useState(contextManager.prune().stats);
  const [showStrategies, setShowStrategies] = useState(false);

  const handlePrune = useCallback(() => {
    const { stats } = contextManager.prune();
    setStats(stats);
  }, []);

  const handleToggle = (s: SupersedeStrategy) => {
    if (contextManager.isActive(s)) {
      contextManager.disableStrategy(s);
    } else {
      contextManager.enableStrategy(s);
    }
    handlePrune();
  };

  const saving = stats.originalTokens > 0
    ? Math.round((1 - stats.afterFinalTokens / stats.originalTokens) * 100)
    : 0;

  const totalHits = Object.values(stats.systemRuleHits).reduce((a, b) => a + b, 0);

  return (
    <div className="ctx-inspector">
      <div className="ctx-header">
        <h3>Context Inspector</h3>
        <button className="ctx-prune-btn" onClick={handlePrune}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5"/>
          </svg>
          Prune Now
        </button>
      </div>

      {/* Stats overview */}
      <div className="ctx-stats">
        <div className="ctx-stat-card">
          <span className="ctx-stat-label">Original</span>
          <span className="ctx-stat-value">{(stats.originalTokens / 1000).toFixed(1)}K</span>
        </div>
        <div className="ctx-stat-card">
          <span className="ctx-stat-label">After System</span>
          <span className="ctx-stat-value">{(stats.afterSystemTokens / 1000).toFixed(1)}K</span>
        </div>
        <div className="ctx-stat-card">
          <span className="ctx-stat-label">After Agent</span>
          <span className="ctx-stat-value">{(stats.afterAgentTokens / 1000).toFixed(1)}K</span>
        </div>
        <div className="ctx-stat-card accent">
          <span className="ctx-stat-label">Final</span>
          <span className="ctx-stat-value">{(stats.afterFinalTokens / 1000).toFixed(1)}K</span>
        </div>
      </div>

      {/* Saving bar */}
      <div className="ctx-saving-bar">
        <div className="ctx-saving-track">
          <div className="ctx-saving-fill" style={{ width: `${saving}%` }} />
        </div>
        <span className="ctx-saving-label">{saving}% saved</span>
      </div>

      {/* Detail stats */}
      <div className="ctx-detail">
        <div className="ctx-detail-row">
          <span className="ctx-detail-label">Agent pruned</span>
          <span className="ctx-detail-value">{stats.agentPruneCount} items</span>
        </div>
        <div className="ctx-detail-row">
          <span className="ctx-detail-label">Merged pairs</span>
          <span className="ctx-detail-value">{stats.mergedPairs} pairs</span>
        </div>
        <div className="ctx-detail-row">
          <span className="ctx-detail-label">Truncated long</span>
          <span className="ctx-detail-value">{stats.truncatedLongResults} results</span>
        </div>
        <div className="ctx-detail-row">
          <span className="ctx-detail-label">History length</span>
          <span className="ctx-detail-value">{contextManager.len} entries</span>
        </div>
      </div>

      {/* Strategy toggles */}
      <div className="ctx-strategies-toggle" onClick={() => setShowStrategies(v => !v)}>
        <span>{showStrategies ? '▾' : '▸'} Strategies ({totalHits} hits)</span>
      </div>

      {showStrategies && (
        <div className="ctx-strategies">
          {(Object.keys(STRATEGY_LABELS) as SupersedeStrategy[]).map(s => {
            const active = contextManager.isActive(s);
            const hits = stats.systemRuleHits[s] || 0;
            return (
              <div
                key={s}
                className={`ctx-strategy ${active ? 'active' : ''}`}
                onClick={() => handleToggle(s)}
              >
                <div className="ctx-strategy-left">
                  <span className="ctx-strategy-dot" style={{ background: STRATEGY_COLORS[s] }} />
                  <span className="ctx-strategy-name">{STRATEGY_LABELS[s]}</span>
                </div>
                <div className="ctx-strategy-right">
                  <span className="ctx-strategy-hits">{hits > 0 ? `${hits}x` : ''}</span>
                  <span className="ctx-strategy-check">{active ? '✓' : ''}</span>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
