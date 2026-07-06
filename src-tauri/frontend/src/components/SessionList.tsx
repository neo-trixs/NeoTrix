import React, { useMemo, useState } from "react";
import type { Session } from "../types";
import { useStore } from "../stores";
import styles from "./SessionList.module.css";

interface Props {
  sessions: Session[];
  activeSession: number;
  onSelect: (index: number) => void;
  onNew: () => void;
}

function getTimeGroup(ts: number): string {
  const now = Date.now();
  const diff = now - ts;
  if (diff < 86400000) return "Today";
  if (diff < 172800000) return "Yesterday";
  if (diff < 604800000) return "This week";
  return "Earlier";
}

const GROUP_ORDER = ["Pinned", "Today", "Yesterday", "This week", "Earlier"];

const SessionList: React.FC<Props> = ({ sessions, activeSession, onSelect, onNew }) => {
  const forkSession = useStore((s) => s.forkSession);
  const exportSession = useStore((s) => s.exportSession);
  const importSession = useStore((s) => s.importSession);
  const displayName = useStore((s) => s.userDisplayName);
  const setPopoverOpen = useStore((s) => s.setUserPopoverOpen);
  const pinSession = useStore((s) => s.pinSession);
  const renameSession = useStore((s) => s.renameSession);
  const removeSession = useStore((s) => s.removeSession);
  const sidebarCollapsed = useStore((s) => s.sidebarCollapsed);
  const setSidebarCollapsed = useStore((s) => s.setSidebarCollapsed);
  const sessionSearchQuery = useStore((s) => s.sessionSearchQuery);
  const setSessionSearchQuery = useStore((s) => s.setSessionSearchQuery);

  const [renamingIdx, setRenamingIdx] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const avatarInitial = (displayName || "Neo").charAt(0).toUpperCase();

  const groupedSessions = useMemo(() => {
    const q = sessionSearchQuery?.toLowerCase().trim();
    const filtered = q
      ? sessions.filter((s) =>
          s.name.toLowerCase().includes(q) ||
          s.messages.some((m) => m.content.toLowerCase().includes(q))
        )
      : sessions;

    const groups: Record<string, { session: Session; index: number }[]> = {
      Pinned: [], Today: [], Yesterday: [], "This week": [], Earlier: [],
    };

    filtered.forEach((s, i) => {
      const realIdx = sessions.indexOf(s);
      if (realIdx === -1) return;
      if (s.pinned) {
        groups.Pinned.push({ session: s, index: realIdx });
      } else {
        const group = getTimeGroup(s.lastActive || Date.now());
        if (!groups[group]) groups[group] = [];
        groups[group].push({ session: s, index: realIdx });
      }
    });

    return GROUP_ORDER.flatMap((g) => {
      const items = groups[g] || [];
      if (items.length === 0) return [];
      return [{ type: "header" as const, label: g }, ...items.map((item) => ({ type: "item" as const, ...item }))];
    });
  }, [sessions, sessionSearchQuery]);

  const handleRename = (idx: number) => {
    setRenamingIdx(idx);
    setRenameValue(sessions[idx]?.name || "");
  };

  const commitRename = (idx: number) => {
    if (renameValue.trim()) {
      renameSession(idx, renameValue.trim());
    }
    setRenamingIdx(null);
  };

  return (
    <div className={`${styles.sidebar} ${sidebarCollapsed ? styles.sidebarCollapsed : ""}`} role="region" aria-label="Session sidebar">
      <div className={styles.header}>
        {!sidebarCollapsed && <h2>NeoTrix</h2>}
        <div className="session-header-actions" style={{ display: "flex", gap: 2 }}>
          {!sidebarCollapsed && (
            <>
              <button className="btn-icon" onClick={importSession} aria-label="Import session" title="Import session">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                  <path d="M8 12V3M4 7l4 5 4-5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M2 12v2h12v-2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                </svg>
              </button>
              <button className="btn-icon" onClick={onNew} aria-label="New session" title="New session (Ctrl+N)">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                  <path d="M8 3v10M3 8h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
                </svg>
              </button>
            </>
          )}
          <button className="btn-icon" onClick={() => setSidebarCollapsed(!sidebarCollapsed)} aria-label="Toggle sidebar" title="Toggle sidebar">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d={sidebarCollapsed ? "M10 4l4 4-4 4M2 4l4 4-4 4" : "M6 4l-4 4 4 4M14 4l-4 4 4 4"} stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        </div>
      </div>

      {!sidebarCollapsed && (
        <div className={styles.searchBar}>
          <input
            type="text"
            placeholder="🔍 搜索会话..."
            value={sessionSearchQuery}
            onChange={(e) => setSessionSearchQuery(e.target.value)}
            className={styles.searchInput}
          />
        </div>
      )}

      <div className={styles.list} role="tablist" aria-label="Session list">
        {groupedSessions.length === 0 && !sidebarCollapsed && (
          <div className={styles.emptyHint}>No sessions found</div>
        )}
        {groupedSessions.map((entry) => {
          if (entry.type === "header") {
            return (
              <div key={entry.label} className={styles.groupLabel}>
                {entry.label}
              </div>
            );
          }
          const { session: s, index: i } = entry;
          const isActive = i === activeSession;
          return (
            <div
              key={s.id}
              role="tab"
              aria-selected={isActive}
              data-testid="session-item"
              data-active={isActive}
              data-session-index={i}
              className={`${styles.item} ${isActive ? styles.itemActive : ""} ${s.pinned ? styles.itemPinned : ""}`}
              onClick={() => onSelect(i)}
            >
              {sidebarCollapsed ? (
                <div className={styles.iconAvatar}>{avatarInitial}</div>
              ) : (
                <>
                  <div className={styles.itemDot}>
                    <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                      <circle cx="5" cy="5" r="3" fill={isActive ? "var(--nt-accent, #007aff)" : "var(--nt-text-muted, #aeaeb2)"} />
                    </svg>
                  </div>
                  <div className={styles.info}>
                    {renamingIdx === i ? (
                      <input
                        className={styles.renameInput}
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onBlur={() => commitRename(i)}
                        onKeyDown={(e) => { if (e.key === "Enter") commitRename(i); if (e.key === "Escape") setRenamingIdx(null); }}
                        autoFocus
                        onClick={(e) => e.stopPropagation()}
                      />
                    ) : (
                      <div className={styles.name}>{s.name}</div>
                    )}
                    <div className={styles.meta}>{s.messages.length} msgs</div>
                  </div>
                  <div className={styles.itemActions} onClick={(e) => e.stopPropagation()}>
                    <button className={styles.actionBtn} onClick={() => pinSession(i)} title={s.pinned ? "Unpin" : "Pin"}>
                      {s.pinned ? "★" : "☆"}
                    </button>
                    <button className={styles.actionBtn} onClick={() => handleRename(i)} title="Rename">✎</button>
                    <button className={styles.actionBtn} onClick={() => forkSession(s.id)} title="Fork">⑂</button>
                  </div>
                </>
              )}
            </div>
          );
        })}
      </div>

      <div className={styles.userBar} onClick={() => setPopoverOpen(true)}>
        <div className={styles.userAvatar}>{avatarInitial}</div>
        {!sidebarCollapsed && (
          <>
            <div className={styles.userName}>{displayName || "Neo"}</div>
            <svg className={styles.userChevron} width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M4 4l2 2 2-2" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </>
        )}
      </div>
    </div>
  );
};

export default SessionList;
