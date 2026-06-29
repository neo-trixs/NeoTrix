import React from "react";
import type { Session } from "../types";
import { useStore } from "../store";

interface Props {
  sessions: Session[];
  activeSession: number;
  onSelect: (index: number) => void;
  onNew: () => void;
}

const SessionList: React.FC<Props> = ({ sessions, activeSession, onSelect, onNew }) => {
  const forkSession = useStore((s) => s.forkSession);
  const exportSession = useStore((s) => s.exportSession);
  const importSession = useStore((s) => s.importSession);

  return (
    <div className="session-sidebar glass-panel">
      <div className="session-header">
        <h2>NeoTrix</h2>
        <div className="session-header-actions">
          <button className="btn-icon" onClick={importSession} title="导入会话">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 12V3M4 7l4 5 4-5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
              <path d="M2 12v2h12v-2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
            </svg>
          </button>
          <button className="btn-icon" onClick={onNew} title="新建会话 (Ctrl+N)">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>
      <div className="session-list">
        {sessions.map((s, i) => (
          <div
            key={s.id}
            className={`session-item ${i === activeSession ? "active" : ""}`}
            onClick={() => onSelect(i)}
          >
            <div className="session-icon">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <circle cx="7" cy="7" r="3" fill={i === activeSession ? "#007aff" : "#aeaeb2"} />
              </svg>
            </div>
            <div className="session-info">
              <div className="session-name">{s.name}</div>
              <div className="session-meta">{s.messages.length} 条消息</div>
            </div>
            <div className="session-item-actions">
              <button
                className="btn-icon btn-icon-sm"
                onClick={(e) => { e.stopPropagation(); forkSession(s.id); }}
                title="复制会话"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                  <path d="M5 3V1h9v9h-2M2 5h9v9H2V5z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round"/>
                </svg>
              </button>
              <button
                className="btn-icon btn-icon-sm"
                onClick={(e) => { e.stopPropagation(); exportSession(s.id); }}
                title="导出会话"
              >
                <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                  <path d="M8 12V3M4 7l4 5 4-5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default SessionList;
