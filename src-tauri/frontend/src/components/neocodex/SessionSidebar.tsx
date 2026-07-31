import React, { useCallback, useEffect, useState } from "react";
import { useStore } from "../../stores";
import type { NeoCodexSession } from "../../types";
import styles from "./SessionSidebar.module.css";

export function SessionSidebar({ onSessionSelect, onSessionDelete }: { onSessionSelect?: (session: NeoCodexSession) => void; onSessionDelete?: (sessionId: string) => void }) {
  const [sessions, setSessions] = useState<NeoCodexSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [newSessionName, setNewSessionName] = useState("");

  const refresh = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const list = await invoke("neocodex_list_sessions") as any[];
      setSessions(list.map((s) => ({
        id: s.id,
        name: s.name,
        mode: s.mode,
        messages: [],
        wire_path: s.wire_path,
        created_at: s.created_at || 0,
        updated_at: s.updated_at || 0,
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

  const handleCreateSession = () => {
    if (!newSessionName.trim()) return;
    const session: NeoCodexSession = {
      id: `session-${Date.now()}`,
      name: newSessionName.trim(),
      mode: "Agent",
      messages: [],
      wire_path: "",
      created_at: Date.now(),
      updated_at: Date.now(),
    };
    setSessions([session, ...sessions]);
    setShowNewDialog(false);
    setNewSessionName("");
    onSessionSelect?.(session);
  };

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

      <div className={styles.list}>
        {sessions.map((session) => (
          <SessionItem
            key={session.id}
            session={session}
            onClick={() => onSessionSelect?.(session)}
            onDelete={onSessionDelete ? () => onSessionDelete(session.id) : undefined}
          />
        ))}
        {sessions.length === 0 && (
          <div className={styles.empty}>
            暂无会话，点击上方按钮创建
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

function SessionItem({ session, onClick, onDelete }: { session: { id: string; name: string; mode: string; updated_at: number }; onClick: () => void; onDelete?: () => void }) {
  const handleDelete = (e: React.MouseEvent) => {
    e.stopPropagation();
    onDelete?.();
  };
  return (
    <div className={styles.item} onClick={onClick}>
      <div className={styles.itemRow}>
        <div className={styles.itemMain}>
          <span className={styles.itemName}>{session.name}</span>
          <span className={styles.itemMode}>{session.mode}</span>
        </div>
        <span className={styles.itemTime}>{formatTime(session.updated_at)}</span>
        {onDelete && (
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