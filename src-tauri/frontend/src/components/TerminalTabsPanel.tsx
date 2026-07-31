import React, { useState, useEffect, useCallback, useRef } from "react";

const TAB_COLORS = [
  "#4ade80", "#60a5fa", "#f472b6", "#fbbf24", "#fb923c",
  "#a78bfa", "#34d399", "#f87171", "#22d3ee", "#e879f9",
];

const LAYOUTS = [
  { id: "horizontal", label: "Horizontal" },
  { id: "vertical", label: "Vertical" },
  { id: "grid", label: "Grid" },
];

function fmtTime(iso: string | null): string {
  if (!iso) return "\u2014";
  const d = new Date(iso);
  const diff = Date.now() - d.getTime();
  if (diff < 60000) return "just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  return d.toLocaleDateString();
}

const TerminalTabsPanel: React.FC = () => {
  const [sessions, setSessions] = useState<string[]>([]);
  const [activeSession, setActiveSession] = useState("");
  const [tabs, setTabs] = useState<any[]>([]);
  const [groups, setGroups] = useState<any[]>([]);
  const [layout, setLayout] = useState("horizontal");
  const [config, setConfigState] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newCwd, setNewCwd] = useState("");
  const [newShell, setNewShell] = useState("");
  const [newColor, setNewColor] = useState(TAB_COLORS[0]);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const editRef = useRef<HTMLInputElement>(null);

  const [showGroupForm, setShowGroupForm] = useState(false);
  const [groupName, setGroupName] = useState("");
  const [groupTabIds, setGroupTabIds] = useState<string[]>([]);

  const fetchAll = useCallback(async () => {
    try {
      const mod = await import("../commands");
      let sessList: string[] = [];
      try {
        const raw = await mod.termTabsList("");
        const uniq = new Set(raw.map((t: any) => t.session_id));
        sessList = Array.from(uniq) as string[];
      } catch {
        const cfg = await mod.termTabsConfig().catch(() => null);
        if (cfg) {
          setConfigState(cfg);
          setLoading(false);
          return;
        }
      }
      setSessions(sessList);
      const sessionId = activeSession || sessList[0] || "";
      if (sessionId) {
        if (!activeSession) setActiveSession(sessionId);
        const [tabData, groupData, cfg] = await Promise.all([
          mod.termTabsList(sessionId).catch(() => []),
          mod.termTabsGroupList(sessionId).catch(() => []),
          mod.termTabsConfig().catch(() => null),
        ]);
        setTabs(tabData);
        setGroups(groupData);
        if (cfg) setConfigState(cfg);
      }
      setError(null);
    } catch {
      setError("Failed to fetch terminal tabs");
    }
    setLoading(false);
  }, [activeSession]);

  useEffect(() => {
    fetchAll();
    const timer = setInterval(fetchAll, 10000);
    return () => clearInterval(timer);
  }, [fetchAll]);

  const handleSessionChange = useCallback((sessionId: string) => {
    setActiveSession(sessionId);
  }, []);

  const handleCreateTab = useCallback(async () => {
    if (!newName.trim() || !activeSession) return;
    setActionLoading("create");
    try {
      const mod = await import("../commands");
      await mod.termTabsCreate(activeSession, newName.trim(), newCwd.trim() || undefined, newShell.trim() || undefined, newColor);
      setShowCreate(false);
      setNewName("");
      setNewCwd("");
      setNewShell("");
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [newName, newCwd, newShell, newColor, activeSession, fetchAll]);

  const handleRename = useCallback(async (tabId: string) => {
    if (!editName.trim()) { setEditingId(null); return; }
    setActionLoading(tabId);
    try {
      const mod = await import("../commands");
      await mod.termTabsRename(tabId, editName.trim());
      setEditingId(null);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [editName, fetchAll]);

  const handleClose = useCallback(async (tabId: string) => {
    setActionLoading(tabId);
    try {
      const mod = await import("../commands");
      await mod.termTabsClose(tabId);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleActivate = useCallback(async (tabId: string) => {
    try {
      const mod = await import("../commands");
      await mod.termTabsActivate(tabId);
      await fetchAll();
    } catch { }
  }, [fetchAll]);

  const handleSetColor = useCallback(async (tabId: string, color: string) => {
    try {
      const mod = await import("../commands");
      await mod.termTabsSetColor(tabId, color);
      await fetchAll();
    } catch { }
  }, [fetchAll]);

  const handleMoveTab = useCallback(async (index: number, direction: -1 | 1) => {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= tabs.length || !activeSession) return;
    const reordered = [...tabs];
    const [moved] = reordered.splice(index, 1);
    reordered.splice(newIndex, 0, moved);
    try {
      const mod = await import("../commands");
      await mod.termTabsReorder(activeSession, reordered.map((t) => t.id));
      await fetchAll();
    } catch { }
  }, [tabs, activeSession, fetchAll]);

  const handleSetLayout = useCallback(async (layoutId: string) => {
    setLayout(layoutId);
    if (!activeSession) return;
    try {
      const mod = await import("../commands");
      await mod.termTabsSetLayout(activeSession, layoutId);
    } catch { }
  }, [activeSession]);

  const handleCreateGroup = useCallback(async () => {
    if (!groupName.trim() || groupTabIds.length === 0 || !activeSession) return;
    setActionLoading("group");
    try {
      const mod = await import("../commands");
      await mod.termTabsGroupCreate(groupName.trim(), activeSession, groupTabIds);
      setShowGroupForm(false);
      setGroupName("");
      setGroupTabIds([]);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [groupName, groupTabIds, activeSession, fetchAll]);

  const handleDeleteGroup = useCallback(async (groupId: string) => {
    setActionLoading(groupId);
    try {
      const mod = await import("../commands");
      await mod.termTabsGroupDelete(groupId);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const toggleGroupTab = useCallback((tabId: string) => {
    setGroupTabIds((prev) =>
      prev.includes(tabId) ? prev.filter((id) => id !== tabId) : [...prev, tabId]
    );
  }, []);

  const startEdit = useCallback((tabId: string, name: string) => {
    setEditingId(tabId);
    setEditName(name);
    setTimeout(() => editRef.current?.focus(), 50);
  }, []);

  if (loading) {
    return (
      <div style={{ height: "100%", padding: "var(--nt-gap-sm)", display: "flex", flexDirection: "column", gap: "var(--nt-gap-md)" }}>
        <div className="lg-skeleton" style={{ height: 60 }} />
        <div className="lg-skeleton" style={{ flex: 1 }} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="lg-empty">
        <div className="lg-empty-icon">{'\u26A0\uFE0F'}</div>
        <div className="lg-empty-text">{error}</div>
        <button className="lg-btn" onClick={fetchAll}>Retry</button>
      </div>
    );
  }

  return (
    <div style={{ height: "100%", padding: "var(--nt-gap-sm)", display: "flex", flexDirection: "column", gap: "var(--nt-gap-md)", overflow: "hidden" }}>
      {/* Header: Session Selector + Layout */}
      <div className="lg-glass-strong" style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--nt-gap-sm)",
        padding: "var(--nt-gap-md)",
        borderRadius: "var(--nt-radius-md)",
        flexShrink: 0,
        flexWrap: "wrap",
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{ fontSize: 11, fontWeight: 600, color: "var(--nt-text-secondary)" }}>Session:</span>
          <select
            value={activeSession}
            onChange={(e) => handleSessionChange(e.target.value)}
            style={{
              padding: "4px 8px",
              borderRadius: "var(--nt-radius-sm)",
              border: "var(--nt-edge-width) solid var(--nt-glass-border)",
              background: "var(--nt-glass-bg)",
              color: "var(--nt-text)",
              fontSize: 12,
              outline: "none",
              minWidth: 160,
            }}
          >
            {sessions.length === 0 && <option value="">No sessions</option>}
            {sessions.map((sid) => (
              <option key={sid} value={sid}>{sid.length > 16 ? `${sid.slice(0, 16)}...` : sid}</option>
            ))}
          </select>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
          <span style={{ fontSize: 11, fontWeight: 600, color: "var(--nt-text-secondary)" }}>Layout:</span>
          {LAYOUTS.map((l) => (
            <button
              key={l.id}
              className="lg-btn"
              onClick={() => handleSetLayout(l.id)}
              style={{
                padding: "4px 8px",
                fontSize: 11,
                fontWeight: layout === l.id ? 600 : 400,
                color: layout === l.id ? "var(--nt-primary)" : "var(--nt-text-secondary)",
                background: layout === l.id ? "var(--nt-glass-bg)" : "transparent",
                border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                borderRadius: "var(--nt-radius-sm)",
                cursor: "pointer",
              }}
            >
              {l.label}
            </button>
          ))}
        </div>
        <div style={{ marginLeft: "auto", display: "flex", gap: 4 }}>
          <button
            className="lg-btn"
            onClick={() => setShowCreate(true)}
            style={{
              padding: "4px 12px",
              fontSize: 12,
              fontWeight: 600,
              background: "var(--nt-primary)",
              color: "#fff",
              border: "none",
              borderRadius: "var(--nt-radius-sm)",
              cursor: "pointer",
            }}
          >
            + Tab
          </button>
          <button
            className="lg-btn lg-btn-ghost"
            onClick={() => setShowGroupForm(!showGroupForm)}
            style={{ fontSize: 12, padding: "4px 8px" }}
          >
            {'\uD83D\uDCC1'} Group
          </button>
        </div>
      </div>

      <div className="lg-scrollbar" style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: "var(--nt-gap-sm)" }}>
        {/* Create Tab Form */}
        {showCreate && (
          <div className="lg-glass-strong" style={{
            padding: "var(--nt-gap-md)",
            borderRadius: "var(--nt-radius-md)",
            display: "flex",
            flexDirection: "column",
            gap: "var(--nt-gap-sm)",
            flexShrink: 0,
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>New Terminal Tab</div>
            <input
              className="lg-input"
              placeholder="Tab name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              style={{ padding: "6px 10px", fontSize: 12 }}
            />
            <input
              className="lg-input"
              placeholder="CWD (optional, defaults to project root)"
              value={newCwd}
              onChange={(e) => setNewCwd(e.target.value)}
              style={{ padding: "6px 10px", fontSize: 12 }}
            />
            <input
              className="lg-input"
              placeholder="Shell (optional, defaults to system shell)"
              value={newShell}
              onChange={(e) => setNewShell(e.target.value)}
              style={{ padding: "6px 10px", fontSize: 12 }}
            />
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ fontSize: 11, color: "var(--nt-text-secondary)" }}>Color:</span>
              {TAB_COLORS.map((c) => (
                <button
                  key={c}
                  onClick={() => setNewColor(c)}
                  style={{
                    width: 18, height: 18,
                    borderRadius: "50%",
                    background: c,
                    border: newColor === c ? "2px solid var(--nt-text)" : "2px solid transparent",
                    cursor: "pointer",
                    padding: 0,
                  }}
                />
              ))}
            </div>
            <div style={{ display: "flex", gap: "var(--nt-gap-sm)", justifyContent: "flex-end" }}>
              <button className="lg-btn" onClick={() => setShowCreate(false)} style={{ fontSize: 12 }}>Cancel</button>
              <button
                className="lg-btn"
                onClick={handleCreateTab}
                disabled={actionLoading === "create" || !newName.trim() || !activeSession}
                style={{ fontSize: 12, fontWeight: 600 }}
              >
                {actionLoading === "create" ? "Creating..." : "Create"}
              </button>
            </div>
          </div>
        )}

        {/* Create Group Form */}
        {showGroupForm && (
          <div className="lg-glass-strong" style={{
            padding: "var(--nt-gap-md)",
            borderRadius: "var(--nt-radius-md)",
            display: "flex",
            flexDirection: "column",
            gap: "var(--nt-gap-sm)",
            flexShrink: 0,
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>New Tab Group</div>
            <input
              className="lg-input"
              placeholder="Group name"
              value={groupName}
              onChange={(e) => setGroupName(e.target.value)}
              style={{ padding: "6px 10px", fontSize: 12 }}
            />
            <div style={{ fontSize: 11, color: "var(--nt-text-secondary)", marginBottom: 2 }}>Select tabs for this group:</div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => toggleGroupTab(tab.id)}
                  style={{
                    padding: "4px 8px",
                    fontSize: 11,
                    borderRadius: "var(--nt-radius-sm)",
                    border: groupTabIds.includes(tab.id) ? "2px solid var(--nt-primary)" : "1px solid var(--nt-glass-border)",
                    background: groupTabIds.includes(tab.id) ? "var(--nt-glass-bg)" : "transparent",
                    color: "var(--nt-text)",
                    cursor: "pointer",
                  }}
                >
                  {tab.name}
                </button>
              ))}
              {tabs.length === 0 && <span style={{ fontSize: 11, color: "var(--nt-text-muted)" }}>No tabs available</span>}
            </div>
            <div style={{ display: "flex", gap: "var(--nt-gap-sm)", justifyContent: "flex-end" }}>
              <button className="lg-btn" onClick={() => { setShowGroupForm(false); setGroupTabIds([]); }} style={{ fontSize: 12 }}>Cancel</button>
              <button
                className="lg-btn"
                onClick={handleCreateGroup}
                disabled={actionLoading === "group" || !groupName.trim() || groupTabIds.length === 0}
                style={{ fontSize: 12, fontWeight: 600 }}
              >
                {actionLoading === "group" ? "Creating..." : "Create Group"}
              </button>
            </div>
          </div>
        )}

        {/* Tab list */}
        {tabs.length === 0 && !showCreate && (
          <div className="lg-empty">
            <div className="lg-empty-icon">{'\uD83D\uDDA5\uFE0F'}</div>
            <div className="lg-empty-text">No terminal tabs</div>
            <div className="lg-empty-hint">Create a new tab to start working</div>
          </div>
        )}

        {tabs.map((tab, index) => (
          <div key={tab.id} className="lg-fade-in" style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            padding: "8px 10px",
            borderRadius: "var(--nt-radius-sm)",
            background: tab.is_active ? "var(--nt-glass-bg)" : "transparent",
            border: tab.is_active
              ? `1.5px solid ${tab.color || "var(--nt-primary)"}`
              : "1px solid var(--nt-glass-border)",
            transition: "all var(--nt-transition-fast)",
            cursor: "pointer",
          }} onClick={() => handleActivate(tab.id)}>
            {/* Color indicator */}
            <div style={{
              width: 10, height: 10,
              borderRadius: "50%",
              background: tab.color || "#888",
              flexShrink: 0,
            }} />

            {/* Name (with inline edit) */}
            <div style={{ flex: 1, minWidth: 0 }}>
              {editingId === tab.id ? (
                <input
                  ref={editRef}
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                  onBlur={() => handleRename(tab.id)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleRename(tab.id);
                    if (e.key === "Escape") setEditingId(null);
                  }}
                  onClick={(e) => e.stopPropagation()}
                  className="lg-input"
                  style={{ padding: "2px 6px", fontSize: 12, width: "100%" }}
                />
              ) : (
                <span
                  style={{ fontSize: 13, fontWeight: tab.is_active ? 600 : 400, color: "var(--nt-text)" }}
                  onDoubleClick={(e) => { e.stopPropagation(); startEdit(tab.id, tab.name); }}
                >
                  {tab.name}
                </span>
              )}
            </div>

            {/* Shell + CWD info */}
            <div style={{ fontSize: 10, color: "var(--nt-text-muted)", maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flexShrink: 0 }}>
              {tab.shell}{tab.cwd ? ` \u2022 ${tab.cwd}` : ""}
            </div>

            {/* Last used */}
            <span style={{ fontSize: 10, color: "var(--nt-text-muted)", flexShrink: 0 }}>{fmtTime(tab.last_used_at)}</span>

            {/* Color picker */}
            <div style={{ position: "relative", flexShrink: 0 }} onClick={(e) => e.stopPropagation()}>
              <select
                value={tab.color || ""}
                onChange={(e) => handleSetColor(tab.id, e.target.value || "")}
                style={{
                  padding: "2px 4px",
                  fontSize: 10,
                  borderRadius: "var(--nt-radius-sm)",
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  background: "var(--nt-glass-bg)",
                  color: "var(--nt-text)",
                  outline: "none",
                  cursor: "pointer",
                  maxWidth: 80,
                }}
                title="Change color"
              >
                <option value="">Default</option>
                {TAB_COLORS.map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>

            {/* Move up/down */}
            <div style={{ display: "flex", flexDirection: "column", gap: 1, flexShrink: 0 }} onClick={(e) => e.stopPropagation()}>
              <button
                className="lg-btn lg-btn-icon lg-btn-ghost"
                onClick={() => handleMoveTab(index, -1)}
                disabled={index === 0}
                title="Move up"
                style={{ fontSize: 8, padding: "1px 4px", lineHeight: 1 }}
              >
                {'\u25B2'}
              </button>
              <button
                className="lg-btn lg-btn-icon lg-btn-ghost"
                onClick={() => handleMoveTab(index, 1)}
                disabled={index === tabs.length - 1}
                title="Move down"
                style={{ fontSize: 8, padding: "1px 4px", lineHeight: 1 }}
              >
                {'\u25BC'}
              </button>
            </div>

            {/* Close */}
            <button
              className="lg-btn lg-btn-icon lg-btn-ghost"
              onClick={(e) => { e.stopPropagation(); handleClose(tab.id); }}
              disabled={actionLoading === tab.id}
              title="Close tab"
              style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-danger)", flexShrink: 0 }}
            >
              {'\u2716'}
            </button>
          </div>
        ))}

        {/* Group list */}
        {groups.length > 0 && (
          <>
            <div style={{ fontSize: 11, fontWeight: 600, color: "var(--nt-text-secondary)", textTransform: "uppercase", letterSpacing: 0.5, paddingTop: "var(--nt-gap-sm)" }}>
              Tab Groups
            </div>
            {groups.map((grp) => (
              <div key={grp.id} className="lg-fade-in" style={{
                display: "flex",
                alignItems: "center",
                gap: 6,
                padding: "8px 10px",
                borderRadius: "var(--nt-radius-sm)",
                background: "var(--nt-glass-bg)",
                border: "1px solid var(--nt-glass-border)",
              }}>
                <span style={{ fontSize: 14 }}>{grp.is_collapsed ? '\u25B6' : '\u25BC'}</span>
                <span style={{ fontSize: 13, fontWeight: 600, color: "var(--nt-text)", flex: 1 }}>{grp.name}</span>
                <span style={{ fontSize: 10, color: "var(--nt-text-muted)" }}>
                  {grp.tab_ids?.length ?? 0} tabs
                </span>
                <button
                  className="lg-btn lg-btn-icon lg-btn-ghost"
                  onClick={() => handleDeleteGroup(grp.id)}
                  disabled={actionLoading === grp.id}
                  title="Delete group"
                  style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-danger)" }}
                >
                  {'\u2716'}
                </button>
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
};

TerminalTabsPanel.displayName = "TerminalTabsPanel";

export default TerminalTabsPanel;
