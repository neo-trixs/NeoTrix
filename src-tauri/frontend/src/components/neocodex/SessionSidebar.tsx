import React, { useCallback, useEffect, useMemo, useState } from "react";
import { useStore } from "../../stores";
import type { NeoCodexSession } from "../../types";
import styles from "./SessionSidebar.module.css";

export function SessionSidebar({ activeSessionId, onSessionSelect, onSessionDelete }: { activeSessionId?: string | null; onSessionSelect?: (session: NeoCodexSession) => void; onSessionDelete?: (sessionId: string) => void }) {
  const [sessions, setSessions] = useState<NeoCodexSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [newSessionName, setNewSessionName] = useState("");
  const [query, setQuery] = useState("");

  const refresh = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const list = await invoke("neocodex_list_sessions") as any[];
      setSessions(list.map((s) => ({
        id: s.id,
        name: s.name,
        mode: s.mode,
        message_count: s.message_count || 0,
        messages: [],
        wire_path: s.wire_path,
        created_at: s.created_at || 0,
        updated_at: (s.updated_at || 0) * 1000,
      })));
    } catch (e) {
      console.error("Failed to load sessions:", e);
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const openNew = () => setShowNewDialog(true);
    window.addEventListener("neotrix:new-session", openNew);
    return () => window.removeEventListener("neotrix:new-session", openNew);
  }, []);

  const handleCreateSession = async () => {
    if (!newSessionName.trim()) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const info = await invoke("neocodex_create_session", { name: newSessionName.trim() }) as any;
      const session: NeoCodexSession = {
        id: info.id,
        name: info.name,
        mode: info.mode || "Agent",
        message_count: info.message_count || 0,
        messages: [],
        wire_path: info.wire_path || "",
        created_at: Date.now(),
        updated_at: Date.now(),
      };
      setSessions((prev) => [session, ...prev]);
      onSessionSelect?.(session);
    } catch (e) {
      console.error("Failed to create session:", e);
      const session: NeoCodexSession = {
        id: `session-${Date.now()}`,
        name: newSessionName.trim(),
        mode: "Agent",
        message_count: 0,
        messages: [],
        wire_path: "",
        created_at: Date.now(),
        updated_at: Date.now(),
      };
      setSessions((prev) => [session, ...prev]);
      onSessionSelect?.(session);
    }
    setShowNewDialog(false);
    setNewSessionName("");
  };

  const handleDelete = async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("neocodex_delete_session", { sessionId });
    } catch (e) {
      console.error("Failed to delete session:", e);
    }
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    onSessionDelete?.(sessionId);
  };

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase();
    const filtered = q ? sessions.filter((s) => s.name.toLowerCase().includes(q)) : sessions;
    const day = 86400000;
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const todayStart = today.getTime();
    const buckets: Array<{ label: string; sessions: NeoCodexSession[] }> = [];
    const push = (label: string, list: NeoCodexSession[]) => {
      if (list.length) buckets.push({ label, sessions: list });
    };
    push("今天", filtered.filter((s) => s.updated_at >= todayStart));
    push("昨天", filtered.filter((s) => s.updated_at >= todayStart - day && s.updated_at < todayStart));
    push("7天内", filtered.filter((s) => s.updated_at >= todayStart - 7 * day && s.updated_at < todayStart - day));
    push("更早", filtered.filter((s) => s.updated_at < todayStart - 7 * day));
    return buckets;
  }, [sessions, query]);

  const formatTime = (ts: number) => {
    const date = new Date(ts);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
    return `${Math.floor(diff / 86400000)}天前`;
  };

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.header}>
          <h3>会话</h3>
          <button className={styles.newBtn} disabled>新建会话</button>
        </div>
        <div className={styles.skeleton} />
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h3>会话 ({sessions.length})</h3>
        <button className={styles.newBtn} onClick={() => setShowNewDialog(true)}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M7 3v8M3 7h8" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </div>

      <div className={styles.search}>
        <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
          <circle cx="6" cy="6" r="4"/><path d="M9.5 9.5L12 12" strokeLinecap="round"/>
        </svg>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索会话…"
          className={styles.searchInput}
        />
      </div>

      <div className={styles.list}>
        {groups.map((group) => (
          <div key={group.label}>
            <div className={styles.groupHeader}>{group.label}</div>
            {group.sessions.map((session) => (
              <SessionItem
                key={session.id}
                session={session}
                active={session.id === activeSessionId}
                onClick={() => onSessionSelect?.(session)}
                onDelete={() => handleDelete(session.id)}
              />
            ))}
          </div>
        ))}
        {sessions.length === 0 && (
          <div className={styles.empty}>
            暂无会话，点击上方按钮创建
          </div>
        )}
        {sessions.length > 0 && groups.length === 0 && (
          <div className={styles.empty}>
            无匹配会话
          </div>
        )}
      </div>

      {showNewDialog && (
        <div className={styles.dialogOverlay} onClick={() => setShowNewDialog(false)}>
          <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
            <h3>新建会话</h3>
            <input
              type="text"
              value={newSessionName}
              onChange={(e) => setNewSessionName(e.target.value)}
              placeholder="会话名称"
              className={styles.dialogInput}
              autoFocus
              onKeyDown={(e) => e.key === "Enter" && handleCreateSession()}
            />
            <div className={styles.dialogActions}>
              <button className={styles.btnSecondary} onClick={() => setShowNewDialog(false)}>取消</button>
              <button className={styles.btnPrimary} onClick={handleCreateSession}>创建</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function SessionItem({ session, active, onClick, onDelete }: { session: { id: string; name: string; mode: string; message_count?: number; updated_at: number }; active: boolean; onClick: () => void; onDelete?: () => void }) {
  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete?.();
  };
  return (
    <div className={`${styles.item} ${active ? styles.itemActive : ""}`} onClick={onClick}>
      <div className={styles.itemRow}>
        <div className={styles.itemMain}>
          <span className={styles.itemName}>{session.name}</span>
          <span className={styles.itemMode}>{session.mode}</span>
        </div>
        <span className={styles.itemTime}>{session.message_count ? `${session.message_count}条 · ` : ""}{formatTime(session.updated_at)}</span>        {onDelete && (
          <button className={styles.deleteBtn} onClick={handleDelete} title="删除会话">
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M3 4h8M6 4V3h2v1M4.5 4l.5 7h4l.5-7" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}

function formatTime(ts: number): string {
  const date = new Date(ts);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
  return `${Math.floor(diff / 86400000)}天前`;
}

export default SessionSidebar;