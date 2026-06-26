import React from "react";
import type { Session } from "../types";
import { useStore } from "../store";

interface Props {
  sessions: Session[];
  activeSession: number;
  onSelect: (index: number) => void;
  onNew: () => void;
  activePanel: string;
  onPanelChange: (panel: string) => void;
}

const NAV_ITEMS = [
  { id: 'chat', label: 'Chat', icon: '💬' },
  { id: 'dashboard', label: 'Dashboard', icon: '🧠' },
  { id: 'evolution', label: 'Evolution', icon: '🧬' },
  { id: 'terminal', label: 'Terminal', icon: '🖥' },
  { id: 'settings', label: 'Settings', icon: '⚙' },
];

const Sidebar: React.FC<Props> = ({ sessions, activeSession, onSelect, onNew, activePanel, onPanelChange }) => {
  const forkSession = useStore((s) => s.forkSession);

  return (
    <div className="session-sidebar glass-panel">
      <div className="session-header">
        <h2>NeoTrix</h2>
        <button className="btn-icon" onClick={onNew} title="New session (Cmd+N)">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>

      <div className="sidebar-nav">
        {NAV_ITEMS.map((item) => (
          <div
            key={item.id}
            className={`sidebar-nav-item ${activePanel === item.id ? 'active' : ''}`}
            onClick={() => onPanelChange(item.id)}
          >
            <span className="sidebar-nav-icon">{item.icon}</span>
            <span className="sidebar-nav-label">{item.label}</span>
          </div>
        ))}
      </div>

      <div className="sidebar-section-label">Sessions</div>
      <div className="session-list">
        {sessions.map((s, i) => (
          <div
            key={s.id}
            className={`session-item ${i === activeSession ? 'active' : ''}`}
            onClick={() => onSelect(i)}
          >
            <div className="session-info">
              <div className="session-name">{s.name}</div>
              <div className="session-meta">{s.messages.length} msgs</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default Sidebar;
