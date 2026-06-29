import React, { useState } from "react";
import { useStore } from "../store";

interface SessionSidebarProps {
  isOpen: boolean;
  onToggle: () => void;
}

const SessionSidebar: React.FC<SessionSidebarProps> = ({ isOpen, onToggle }) => {
  const sessions = useStore((s) => s.sessions);
  const activeSessionIndex = useStore((s) => s.activeSessionIndex);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);
  const addSession = useStore((s) => s.addSession);
  const removeSession = useStore((s) => s.removeSession);
  const renameSession = useStore((s) => s.renameSession);
  const setShowSettings = useStore((s) => s.setShowSettings);
  const showTerminal = useStore((s) => s.showTerminal);
  const setShowTerminal = useStore((s) => s.setShowTerminal);

  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editName, setEditName] = useState("");
  const [projectsExpanded, setProjectsExpanded] = useState(true);

  const firstUserMessage = (s: typeof sessions[0]): string => {
    const msg = s.messages.find((m) => m.role === "user");
    if (!msg) return "";
    return msg.content.length > 40
      ? msg.content.slice(0, 40) + "…"
      : msg.content;
  };

  const handleDoubleClick = (index: number, currentName: string) => {
    if (!renameSession) return;
    setEditingIndex(index);
    setEditName(currentName);
  };

  const handleRenameCommit = (index: number) => {
    if (editingIndex === index && editName.trim() && renameSession) {
      renameSession(index, editName.trim());
    }
    setEditingIndex(null);
    setEditName("");
  };

  if (!isOpen) return null;

  return (
    <div className="app-sidebar glass-strong">
      <div className="sidebar-header">
        <div className="sidebar-header-left">
          <div className="sidebar-logo">N</div>
          <span className="sidebar-title">NeoTrix</span>
        </div>
        <div className="sidebar-header-actions">
          <button className="sidebar-action-icon-btn" onClick={addSession} title="New session">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M7 2v10M2 7h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
          <button className="sidebar-action-icon-btn" onClick={onToggle} title="Hide sidebar (Cmd+B)">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M9 3L5 7l4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </div>
      </div>

      <div className="sidebar-tree">
        <div className="project-group">
          <div
            className="project-group-header active"
            onClick={() => setProjectsExpanded(!projectsExpanded)}
          >
            <span className={`project-chevron ${projectsExpanded ? "open" : ""}`}>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M3 2l4 3-4 3" />
              </svg>
            </span>
            <span className="project-icon">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3">
                <path d="M2 4h4l1 1h5v6H2V4z" />
              </svg>
            </span>
            <span className="project-name">Sessions</span>
            <span className="project-badge">{sessions.length}</span>
          </div>

          {projectsExpanded && (
            <div className="session-list">
              {sessions.map((s, i) => (
                <div
                  key={s.id}
                  className={`session-item ${i === activeSessionIndex ? "active" : ""}`}
                  onClick={() => setActiveSessionIndex(i)}
                  onDoubleClick={() => handleDoubleClick(i, s.name)}
                >
                  <span className="session-item-indicator" />
                  {editingIndex === i ? (
                    <input
                      className="session-rename-input"
                      value={editName}
                      onChange={(e) => setEditName(e.target.value)}
                      onBlur={() => handleRenameCommit(i)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") handleRenameCommit(i);
                        if (e.key === "Escape") setEditingIndex(null);
                      }}
                      autoFocus
                      onClick={(e) => e.stopPropagation()}
                    />
                  ) : (
                    <span className="session-item-name">{s.name}</span>
                  )}
                  {sessions.length > 1 && (
                    <button
                      className="session-item-close"
                      onClick={(e) => { e.stopPropagation(); removeSession(i); }}
                      title="Delete session"
                    >
                      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.2">
                        <path d="M2 2l6 6M8 2l-6 6" />
                      </svg>
                    </button>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <div className="sidebar-actions">
        <button className="sidebar-action-btn" onClick={() => setShowTerminal(!showTerminal)}>
          <span className="sidebar-action-icon">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round">
              <path d="M3 4l3 3-3 3M8 10h3" />
            </svg>
          </span>
          Terminal
        </button>
        <button className="sidebar-action-btn" onClick={() => setShowSettings(true)}>
          <span className="sidebar-action-icon">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3">
              <circle cx="7" cy="7" r="2" />
              <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" strokeLinecap="round" />
            </svg>
          </span>
          Settings
        </button>
      </div>
    </div>
  );
};

export default SessionSidebar;
