import { useState, useMemo } from 'react';
import { getSessions, createSession, startSession } from '../core/session-manager';

interface SessionSidebarProps {
  activeId: string;
  onSelect: (id: string) => void;
}

const FILTERS = ['All', 'Running', 'Paused', 'Completed'] as const;

const STATUS_ICON: Record<string, string> = {
  idle: '○', running: '▶', paused: '⏸',
  completed: '✓', error: '✕',
};

const STATUS_COLOR: Record<string, string> = {
  idle: '#5e5e6e', running: '#48b87a', paused: '#e8a030',
  completed: '#5e8aff', error: '#e04e4e',
};

export default function SessionSidebar({ activeId, onSelect }: SessionSidebarProps) {
  const [filter, setFilter] = useState<string>('All');
  const [search, setSearch] = useState('');
  const [groupBy, setGroupBy] = useState<'none' | 'status'>('none');

  const sessions = useMemo(() => {
    const all = getSessions();
    const q = search.toLowerCase();
    return all.filter(s => {
      if (filter !== 'All' && s.status !== filter.toLowerCase()) return false;
      if (search && !s.label.toLowerCase().includes(q) && !s.id.toLowerCase().includes(q)) return false;
      return true;
    });
  }, [filter, search]);

  const grouped = useMemo(() => {
    if (groupBy === 'none') return { 'Sessions': sessions };
    const groups: Record<string, typeof sessions> = {};
    for (const s of sessions) {
      const key = s.status.charAt(0).toUpperCase() + s.status.slice(1);
      if (!groups[key]) groups[key] = [];
      groups[key].push(s);
    }
    return groups;
  }, [sessions, groupBy]);

  function handleNewSession() {
    const n = getSessions().length + 1;
    const s = createSession(`Session ${n}`);
    onSelect(s.id);
    startSession(s.id);
  }

  function handleContextMenu(e: React.MouseEvent, id: string) {
    e.preventDefault();
    onSelect(id);
  }

  return (
    <aside className="session-sidebar">
      {/* Header with new button */}
      <div className="session-sidebar-header">
        <h3>Sessions</h3>
        <div className="session-header-actions">
          <button className="session-new-btn" onClick={handleNewSession} title="New session">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
          <span className="session-count">{getSessions().length}</span>
        </div>
      </div>

      {/* Search */}
      <div className="session-search">
        <svg className="session-search-icon" width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          className="session-search-input"
          type="text"
          placeholder="Search sessions..."
          value={search}
          onChange={e => setSearch(e.target.value)}
        />
      </div>

      {/* Filters row */}
      <div className="session-filters">
        {FILTERS.map(f => (
          <button
            key={f}
            className={`session-filter-btn ${filter === f ? 'active' : ''}`}
            onClick={() => setFilter(f)}
          >
            {f}
          </button>
        ))}
        <div className="session-filter-spacer" />
        <button
          className={`session-group-btn ${groupBy === 'status' ? 'active' : ''}`}
          onClick={() => setGroupBy(g => g === 'none' ? 'status' : 'none')}
          title="Group by status"
        >
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="3" y="3" width="7" height="7" rx="1" />
            <rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" />
            <rect x="14" y="14" width="7" height="7" rx="1" />
          </svg>
        </button>
      </div>

      {/* Sessions list */}
      <div className="session-list">
        {Object.entries(grouped).map(([groupName, items]) => (
          <div key={groupName}>
            {groupBy !== 'none' && (
              <div className="session-group-label">{groupName}</div>
            )}
            {items.map(s => (
              <div
                key={s.id}
                className={`session-item ${s.id === activeId ? 'active' : ''}`}
                onClick={() => onSelect(s.id)}
                onContextMenu={(e) => handleContextMenu(e, s.id)}
              >
                <div className="session-item-header">
                  <span className="session-status" style={{ color: STATUS_COLOR[s.status] || '#5e5e6e' }}>
                    {STATUS_ICON[s.status] || '○'}
                  </span>
                  <span className="session-label">{s.label}</span>
                  <span className="session-step-badge">{s.currentStep}</span>
                </div>
                <div className="session-item-meta">
                  <span className="session-meta-progress">{s.progress.toFixed(0)}%</span>
                  {s.activeSubsystems > 0 && (
                    <span className="session-meta-subs">{s.activeSubsystems}/{s.subsystemCount}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        ))}
        {sessions.length === 0 && (
          <div className="session-empty">No matching sessions</div>
        )}
      </div>
    </aside>
  );
}
