import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../../stores";
import type { NeoCodexSession } from "../../types";
import styles from "./SessionSidebar.module.css";

export function SessionSidebar({ activeSessionId, onSessionSelect, onSessionDelete, onSessionArchive }: { activeSessionId?: string | null; onSessionSelect?: (session: NeoCodexSession) => void; onSessionDelete?: (sessionId: string) => void; onSessionArchive?: (sessionId: string) => void }) {
  const [sessions, setSessions] = useState<NeoCodexSession[]>([]);
  const [archived, setArchived] = useState<NeoCodexSession[]>([]);
  const [showArchived, setShowArchived] = useState(false);
  const [loading, setLoading] = useState(true);
  const [showNewDialog, setShowNewDialog] = useState(false);
  const [newSessionName, setNewSessionName] = useState("");
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | "active" | "idle">("all");
  const [groupBy, setGroupBy] = useState<"date" | "mode" | "project">("date");
  const [pinnedIds, setPinnedIds] = useState<string[]>(() => {
    try { return JSON.parse(localStorage.getItem("neotrix:pinned-sessions") || "[]"); } catch { return []; }
  });
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [exportSession, setExportSession] = useState<NeoCodexSession | null>(null);
  const addNotification = useStore((s) => s.addNotification);

  useEffect(() => {
    localStorage.setItem("neotrix:pinned-sessions", JSON.stringify(pinnedIds));
  }, [pinnedIds]);

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

  const refreshArchived = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const list = await invoke("neocodex_list_archived") as any[];
      setArchived(list.map((s) => ({
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
      console.error("Failed to load archived sessions:", e);
      setArchived([]);
    }
  }, []);

  useEffect(() => {
    refreshArchived();
  }, [refreshArchived]);

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
    setPinnedIds((prev) => prev.filter((id) => id !== sessionId));
    onSessionDelete?.(sessionId);
    addNotification({ type: "info", message: "会话已删除", duration: 2000 });
  };

  const handleArchive = async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("neocodex_archive_session", { sessionId });
    } catch (e) {
      console.error("Failed to archive session:", e);
      addNotification({ type: "error", message: `归档失败: ${e}`, duration: 3000 });
      return;
    }
    setSessions((prev) => prev.filter((s) => s.id !== sessionId));
    setPinnedIds((prev) => prev.filter((id) => id !== sessionId));
    onSessionArchive?.(sessionId);
    refreshArchived();
    addNotification({ type: "info", message: "会话已归档", duration: 2000 });
  };

  const handleRestore = async (sessionId: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("neocodex_restore_session", { sessionId });
    } catch (e) {
      console.error("Failed to restore session:", e);
      addNotification({ type: "error", message: `恢复失败: ${e}`, duration: 3000 });
      return;
    }
    setArchived((prev) => prev.filter((s) => s.id !== sessionId));
    refresh();
    addNotification({ type: "info", message: "会话已恢复", duration: 2000 });
  };

  const togglePin = (sessionId: string) => {
    const willPin = !pinnedIds.includes(sessionId);
    setPinnedIds((prev) => prev.includes(sessionId) ? prev.filter((id) => id !== sessionId) : [sessionId, ...prev]);
    addNotification({ type: willPin ? "success" : "info", message: willPin ? "会话已置顶" : "已取消置顶", duration: 2000 });
  };

  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const renamedNames = useRef<Record<string, string>>({});

  const handleRename = async (sessionId: string, newName: string) => {
    if (!newName.trim()) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("neocodex_rename_session", { sessionId, name: newName.trim() });
      renamedNames.current[sessionId] = newName.trim();
      setSessions((prev) => prev.map((s) => (s.id === sessionId ? { ...s, name: newName.trim() } : s)));
      setRenamingId(null);
      addNotification({ type: "success", message: "已重命名", duration: 2000 });
    } catch (e) {
      addNotification({ type: "error", message: `重命名失败: ${e}`, duration: 3000 });
      setRenamingId(null);
    }
  };

  const handleExport = async (session: NeoCodexSession) => {
    setExportSession(session);
    setShowExportDialog(true);
  };

  const performExport = async (format: "markdown" | "json") => {
    if (!exportSession) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const items = await invoke("neocodex_get_session_messages", { sessionId: exportSession.id }) as any[];
      let content: string;
      if (format === "markdown") {
        content = items.map((m: any) => `## ${m.role === "user" ? "用户" : m.role === "assistant" ? "助手" : m.role}\n${m.content}\n`).join("---\n");
      } else {
        content = JSON.stringify(items, null, 2);
      }
      const blob = new Blob([content], { type: format === "markdown" ? "text/markdown" : "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${exportSession.name.replace(/[^a-z0-9]/gi, "_")}.${format === "markdown" ? "md" : "json"}`;
      a.click();
      URL.revokeObjectURL(url);
      addNotification({ type: "success", message: "会话已导出", duration: 2500 });
    } catch (e) {
      console.error("Export failed:", e);
      addNotification({ type: "error", message: "导出失败", duration: 3000 });
    }
    setShowExportDialog(false);
  };

  const groups = useMemo(() => {
    const q = query.trim().toLowerCase();
    const filtered = sessions.filter((s) => {
      if (q && !s.name.toLowerCase().includes(q)) return false;
      if (statusFilter === "active" && !((s.message_count ?? 0) > 0)) return false;
      if (statusFilter === "idle" && !((s.message_count ?? 0) === 0)) return false;
      return true;
    });
    const pinned = filtered.filter((s) => pinnedIds.includes(s.id));
    const unpinned = filtered.filter((s) => !pinnedIds.includes(s.id));
    const buckets: Array<{ label: string; sessions: NeoCodexSession[] }> = [];
    if (pinned.length) buckets.push({ label: "📌 置顶", sessions: pinned });
    if (groupBy === "project") {
      const byProject = new Map<string, NeoCodexSession[]>();
      for (const s of unpinned) {
        const key = deriveProject(s);
        if (!byProject.has(key)) byProject.set(key, []);
        byProject.get(key)!.push(s);
      }
      for (const [project, list] of byProject) {
        buckets.push({ label: `📁 ${project} (${list.length})`, sessions: list });
      }
    } else if (groupBy === "mode") {
      const modeOrder = ["Agent", "Shell", "Plan"];
      const byMode = new Map<string, NeoCodexSession[]>();
      for (const s of unpinned) {
        const key = s.mode || "未指定";
        if (!byMode.has(key)) byMode.set(key, []);
        byMode.get(key)!.push(s);
      }
      for (const m of modeOrder) {
        if (byMode.has(m)) buckets.push({ label: `${m} 模式`, sessions: byMode.get(m)! });
      }
      if (byMode.has("未指定")) buckets.push({ label: "未指定", sessions: byMode.get("未指定")! });
    } else {
      const day = 86400000;
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const todayStart = today.getTime();
      const push = (label: string, list: NeoCodexSession[]) => { if (list.length) buckets.push({ label, sessions: list }); };
      push("今天", unpinned.filter((s) => s.updated_at >= todayStart));
      push("昨天", unpinned.filter((s) => s.updated_at >= todayStart - day && s.updated_at < todayStart));
      push("7天内", unpinned.filter((s) => s.updated_at >= todayStart - 7 * day && s.updated_at < todayStart - day));
      push("更早", unpinned.filter((s) => s.updated_at < todayStart - 7 * day));
    }
    return buckets;
  }, [sessions, query, statusFilter, groupBy, pinnedIds]);

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

      <div className={styles.filterRow}>
        <select
          className={styles.filterSelect}
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value as "all" | "active" | "idle")}
        >
          <option value="all">全部</option>
          <option value="active">有消息</option>
          <option value="idle">空闲</option>
        </select>
        <select
          className={styles.filterSelect}
          value={groupBy}
          onChange={(e) => setGroupBy(e.target.value as "date" | "mode" | "project")}
        >
          <option value="date">按日期分组</option>
          <option value="mode">按模式分组</option>
          <option value="project">按项目分组</option>
        </select>
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
            <div className={groupBy === "project" ? styles.groupHeaderProject : styles.groupHeader}>{group.label}</div>
            {group.sessions.map((session) => (
              <SessionItem
                key={session.id}
                session={session}
                pinned={pinnedIds.includes(session.id)}
                active={session.id === activeSessionId}
                onClick={() => onSessionSelect?.(session)}
                onDelete={() => handleDelete(session.id)}
                onPin={() => togglePin(session.id)}
                onExport={() => handleExport(session)}
                onArchive={() => handleArchive(session.id)}
                onRename={() => { setRenamingId(session.id); setRenameValue(session.name); }}
              />
            ))}
          </div>
        ))}
        {sessions.length === 0 && (
          <div className={styles.empty}>
            <div className={styles.emptyIcon}>
              <svg width="32" height="32" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M3 7h8M3 4h8M3 10h8" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
            </div>
            <p>暂无会话</p>
            <span className={styles.emptyHint}>点击上方 + 号创建第一个会话，或按 <kbd>⌘N</kbd></span>
          </div>
        )}
        {sessions.length > 0 && groups.length === 0 && (
          <div className={styles.emptyFilter}>无匹配会话</div>
        )}

        {archived.length > 0 && (
          <div className={styles.archivedSection}>
            <button
              className={styles.archivedToggle}
              onClick={() => setShowArchived((v) => !v)}
            >
              <span className={styles.archivedCaret}>{showArchived ? "▾" : "▸"}</span>
              已归档 ({archived.length})
            </button>
            {showArchived && (
              <div className={styles.archivedList}>
                {archived.map((session) => (
                  <div key={session.id} className={styles.archivedItem}>
                    <span className={styles.archivedName} title={session.name}>{session.name}</span>
                    <button
                      className={styles.actionBtn}
                      onClick={() => handleRestore(session.id)}
                      title="恢复会话"
                    >
                      <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                        <path d="M7 11V3M3 7l4-4 4 4" strokeLinecap="round" strokeLinejoin="round"/>
                      </svg>
                    </button>
                    <button
                      className={styles.deleteBtn}
                      onClick={() => handleDelete(session.id)}
                      title="永久删除"
                    >
                      <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                        <path d="M3 4h8M6 4V3h2v1M4.5 4l.5 7h4l.5-7" strokeLinecap="round" strokeLinejoin="round"/>
                      </svg>
                    </button>
                  </div>
                ))}
              </div>
            )}
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

      {showExportDialog && exportSession && (
        <div className={styles.dialogOverlay} onClick={() => setShowExportDialog(false)}>
          <div className={styles.dialog} onClick={(e) => e.stopPropagation()}>
            <h3>导出会话：{exportSession.name}</h3>
            <p className={styles.dialogHint}>选择格式，内容将按消息顺序导出。</p>
            <div className={styles.dialogActions}>
              <button className={styles.btnSecondary} onClick={() => setShowExportDialog(false)}>取消</button>
              <button className={styles.btnPrimary} onClick={() => performExport("markdown")}>导出 Markdown</button>
              <button className={styles.btnPrimary} onClick={() => performExport("json")}>导出 JSON</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function SessionItem({ session, pinned, active, onClick, onDelete, onPin, onExport, onRename, onArchive }: { session: { id: string; name: string; mode: string; message_count?: number; updated_at: number }; pinned: boolean; active: boolean; onClick: () => void; onDelete?: () => void; onPin: () => void; onExport: () => void; onRename: () => void; onArchive?: () => void }) {
  const [showRename, setShowRename] = useState(false);
  const [renameVal, setRenameVal] = useState(session.name);
  const handleRenameSubmit = () => {
    if (renameVal.trim()) {
      onRename();
      // Update via parent's rename handler (stored in renamedNames ref)
    }
    setShowRename(false);
  };
  const handleRename = (e?: React.MouseEvent) => {
    e?.stopPropagation();
    setShowRename(true);
    setRenameVal(session.name);
  };
  const handleRenameKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") { e.preventDefault(); handleRenameSubmit(); }
    if (e.key === "Escape") { e.preventDefault(); setShowRename(false); }
  };
  return (
    <div className={`${styles.item} ${active ? styles.itemActive : ""} ${pinned ? styles.itemPinned : ""}`} onClick={onClick}>
      <div className={styles.itemRow}>
        {pinned && <span className={styles.pinIcon} title="已置顶">📌</span>}
        <div className={styles.itemMain}>
          {showRename ? (
            <input
              className={styles.renameInput}
              value={renameVal}
              onChange={(e) => setRenameVal(e.target.value)}
              onKeyDown={handleRenameKeyDown}
              onBlur={handleRenameSubmit}
              autoFocus
            />
          ) : (
            <span className={styles.itemName} onDoubleClick={handleRename} title="双击重命名">{session.name}</span>
          )}
          <span className={styles.itemMode}>{session.mode}</span>
        </div>
        <span className={styles.itemTime}>{session.message_count ? `${session.message_count}条 · ` : ""}{formatTime(session.updated_at)}</span>
        {onDelete && (
          <button className={styles.deleteBtn} onClick={(e) => { e.stopPropagation(); onDelete?.(); }} title="删除会话">
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M3 4h8M6 4V3h2v1M4.5 4l.5 7h4l.5-7" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        )}
        <button className={styles.actionBtn} onClick={(e) => { e.stopPropagation(); onPin(); }} title={pinned ? "取消置顶" : "置顶"}>
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M7 3v4M5 5l2-2 2 2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
        <button className={styles.actionBtn} onClick={(e) => { e.stopPropagation(); onExport(); }} title="导出">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M3 11v-6a2 2 0 012-2h6a2 2 0 012 2v6M7 3v8M3 7h8" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
        <button className={styles.actionBtn} onClick={(e) => { e.stopPropagation(); handleRename(); }} title="重命名">
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <path d="M10 2l2 2-8 8H4v-2l6-6z" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
        {onArchive && (
          <button className={styles.actionBtn} onClick={(e) => { e.stopPropagation(); onArchive(); }} title="归档会话">
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M2 4h10M3.5 4l.7 7h5.6l.7-7M5.5 7.5h3" strokeLinecap="round" strokeLinejoin="round"/>
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

function deriveProject(session: NeoCodexSession): string {
  const wire = session.wire_path;
  if (wire && typeof wire === "string" && wire.trim() !== "") {
    const parts = wire.replace(/\\/g, "/").split("/");
    const base = parts[parts.length - 1]?.trim() || "";
    if (base) return base;
    return wire;
  }
  return "未知项目";
}

export default SessionSidebar;