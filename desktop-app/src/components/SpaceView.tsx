import { useState, useMemo } from 'react';
import { getSpaces, getSessions, createSpace, addSessionToSpace } from '../core/session-manager';

interface SpaceViewProps {
  activeSessionId: string;
  onSelect: (id: string) => void;
}

export default function SpaceView({ activeSessionId, onSelect }: SpaceViewProps) {
  const [expandedSpace, setExpandedSpace] = useState<string | null>(null);

  const allSpaces = useMemo(() => getSpaces(), []);
  const ungrouped = useMemo(() => getSessions().filter(s => !s.spaceId), []);

  function handleCreateSpace() {
    const label = `Space ${allSpaces.length + 1}`;
    const s = createSpace(label, activeSessionId ? [activeSessionId] : []);
    setExpandedSpace(s.id);
  }

  function handleDrop(e: React.DragEvent, spaceId: string) {
    e.preventDefault();
    const sid = e.dataTransfer.getData('sessionId');
    if (sid) addSessionToSpace(spaceId, sid);
  }

  function handleDragStart(e: React.DragEvent, sessionId: string) {
    e.dataTransfer.setData('sessionId', sessionId);
  }

  return (
    <div className="space-view">
      <div className="space-header">
        <h3>Spaces</h3>
        <button className="space-create-btn" onClick={handleCreateSpace} title="New space">
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>
      </div>

      <div className="space-list">
        {allSpaces.map(sp => {
          const isExpanded = expandedSpace === sp.id;
          const spaceSessions = getSessions().filter(s => s.spaceId === sp.id);
          return (
            <div
              key={sp.id}
              className="space-card"
              onDragOver={e => e.preventDefault()}
              onDrop={e => handleDrop(e, sp.id)}
            >
              <div className="space-card-header" onClick={() => setExpandedSpace(isExpanded ? null : sp.id)}>
                <span className="space-card-icon">{isExpanded ? '▾' : '▸'}</span>
                <span className="space-card-label">{sp.label}</span>
                <span className="space-card-count">{spaceSessions.length}</span>
              </div>

              {isExpanded && (
                <div className="space-card-body">
                  {spaceSessions.map(s => (
                    <div
                      key={s.id}
                      className={`space-session-item ${s.id === activeSessionId ? 'active' : ''}`}
                      onClick={() => onSelect(s.id)}
                      draggable
                      onDragStart={e => handleDragStart(e, s.id)}
                    >
                      <span className={`space-session-status status-${s.status}`} />
                      <span className="space-session-label">{s.label}</span>
                      <span className="space-session-step">{s.currentStep}</span>
                    </div>
                  ))}

                  {sp.prUrls.length > 0 && (
                    <div className="space-prs">
                      <span className="space-pr-label">PRs</span>
                      {sp.prUrls.map(pr => (
                        <span key={pr} className="space-pr-item">{pr}</span>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}

        {ungrouped.length > 0 && (
          <div className="space-card ungrouped">
            <div className="space-card-header" onClick={() => setExpandedSpace(expandedSpace === '_ungrouped' ? null : '_ungrouped')}>
              <span className="space-card-icon">{expandedSpace === '_ungrouped' ? '▾' : '▸'}</span>
              <span className="space-card-label">Ungrouped</span>
              <span className="space-card-count">{ungrouped.length}</span>
            </div>
            {expandedSpace === '_ungrouped' && (
              <div className="space-card-body">
                {ungrouped.map(s => (
                  <div
                    key={s.id}
                    className={`space-session-item ${s.id === activeSessionId ? 'active' : ''}`}
                    onClick={() => onSelect(s.id)}
                    draggable
                    onDragStart={e => handleDragStart(e, s.id)}
                  >
                    <span className={`space-session-status status-${s.status}`} />
                    <span className="space-session-label">{s.label}</span>
                    <span className="space-session-step">{s.currentStep}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
