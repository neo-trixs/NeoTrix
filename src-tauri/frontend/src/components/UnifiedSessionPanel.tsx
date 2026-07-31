import React, { useState, useEffect, useCallback, useMemo } from "react";
import {
  unifiedSessionList,
  unifiedSessionSummary,
  unifiedSessionGroupBy,
  unifiedSessionSearch,
  unifiedSessionConnect,
  unifiedSessionDisconnect,
  unifiedSessionTag,
  unifiedSessionUntag,
  unifiedSessionRefresh,
} from "../commands";
import type { UnifiedSession } from "../commands";

const C = {
  bg: "#0b0b14",
  card: "#14141f",
  cardBorder: "#1e1e2f",
  accent: "#7c5cfc",
  accentDim: "#5a3fc8",
  text: "#e8e8ed",
  muted: "#6b6b80",
  dim: "#3a3a4a",
  green: "#34d399",
  red: "#f87171",
  orange: "#fb923c",
  blue: "#60a5fa",
  yellow: "#facc15",
  white: "#ffffff",
};

type TypeTab = "all" | "local" | "remote" | "teleport";
type GroupBy = "none" | "project" | "type_" | "status" | "surface";
const SURFACES = ["all", "cli", "desktop", "web", "mobile", "background"] as const;

const TYPE_ICONS: Record<string, string> = {
  local: "\uD83D\uDDA5\uFE0F",
  remote: "\u2601\uFE0F",
  teleport: "\uD83D\uDD17",
};

const STATUS_BADGE: Record<string, { icon: string; color: string }> = {
  active: { icon: "\uD83D\uDFE2", color: C.green },
  idle: { icon: "\u26AA", color: C.muted },
  paused: { icon: "\uD83D\uDFE1", color: C.yellow },
  error: { icon: "\uD83D\uDD34", color: C.red },
};

function fmtDuration(minutes: number): string {
  if (minutes < 1) return "<1m";
  if (minutes < 60) return `${Math.round(minutes)}m`;
  const h = Math.floor(minutes / 60);
  const m = Math.round(minutes % 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function fmtTime(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
  } catch {
    return iso;
  }
}

const btnBase: React.CSSProperties = {
  padding: "5px 12px",
  borderRadius: "6px",
  border: "none",
  fontSize: "12px",
  fontWeight: 600,
  cursor: "pointer",
  transition: "all 0.15s",
};

export default function UnifiedSessionPanel() {
  const [sessions, setSessions] = useState<UnifiedSession[]>([]);
  const [summary, setSummary] = useState<Record<string, number> | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<TypeTab>("all");
  const [surfaceFilter, setSurfaceFilter] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("none");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [newTagInput, setNewTagInput] = useState<Record<string, string>>({});

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const [s, sum] = await Promise.all([unifiedSessionList(), unifiedSessionSummary()]);
      setSessions(s ?? []);
      setSummary(sum as unknown as Record<string, number>);
    } catch {
      /* backend may not be ready */
    }
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleRefresh = useCallback(async () => {
    try {
      const r = await unifiedSessionRefresh();
      setSummary(r as unknown as Record<string, number>);
      const s = await unifiedSessionList();
      setSessions(s ?? []);
    } catch { /* ignore */ }
  }, []);

  const handleSearch = useCallback(async (q: string) => {
    setSearchQuery(q);
    if (!q.trim()) {
      const s = await unifiedSessionList();
      setSessions(s ?? []);
      return;
    }
    try {
      const r = await unifiedSessionSearch(q);
      setSessions(r ?? []);
    } catch { /* ignore */ }
  }, []);

  const handleConnect = useCallback(async (id: string) => {
    try {
      await unifiedSessionConnect(id);
      const s = await unifiedSessionList();
      setSessions(s ?? []);
    } catch { /* ignore */ }
  }, []);

  const handleDisconnect = useCallback(async (id: string) => {
    try {
      await unifiedSessionDisconnect(id);
      const s = await unifiedSessionList();
      setSessions(s ?? []);
    } catch { /* ignore */ }
  }, []);

  const handleAddTag = useCallback(async (id: string) => {
    const tag = newTagInput[id]?.trim();
    if (!tag) return;
    const session = sessions.find((s) => s.id === id);
    if (!session) return;
    try {
      await unifiedSessionTag(id, [...session.tags, tag]);
      setSessions((prev) => prev.map((s) => (s.id === id ? { ...s, tags: [...s.tags, tag] } : s)));
      setNewTagInput((prev) => ({ ...prev, [id]: "" }));
    } catch { /* ignore */ }
  }, [sessions, newTagInput]);

  const handleRemoveTag = useCallback(async (id: string, tag: string) => {
    const session = sessions.find((s) => s.id === id);
    if (!session) return;
    try {
      await unifiedSessionUntag(id, [tag]);
      setSessions((prev) => prev.map((s) => (s.id === id ? { ...s, tags: s.tags.filter((t) => t !== tag) } : s)));
    } catch { /* ignore */ }
  }, [sessions]);

  const handleGroupBy = useCallback(async (g: GroupBy) => {
    setGroupBy(g);
    if (g === "none") {
      const s = await unifiedSessionList();
      setSessions(s ?? []);
      return;
    }
    try {
      const r = await unifiedSessionGroupBy(g);
      const merged: UnifiedSession[] = [];
      for (const group of r.groups) merged.push(...group.sessions);
      setSessions(merged);
    } catch { /* ignore */ }
  }, []);

  const filtered = useMemo(() => {
    let list = sessions;
    if (activeTab !== "all") list = list.filter((s) => s.type_ === activeTab);
    if (surfaceFilter !== "all") list = list.filter((s) => s.surface === surfaceFilter);
    return list;
  }, [sessions, activeTab, surfaceFilter]);

  const grouped = useMemo(() => {
    if (groupBy === "none") return null;
    const map = new Map<string, UnifiedSession[]>();
    const keyFn = (s: UnifiedSession): string => {
      if (groupBy === "project") return s.project ?? "No Project";
      if (groupBy === "type_") return s.type_;
      if (groupBy === "status") return s.status;
      if (groupBy === "surface") return s.surface;
      return "Other";
    };
    for (const s of filtered) {
      const k = keyFn(s);
      if (!map.has(k)) map.set(k, []);
      map.get(k)!.push(s);
    }
    return Array.from(map.entries()).sort((a, b) => b[1].length - a[1].length);
  }, [filtered, groupBy]);

  const summaryBar = summary ?? {};

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: C.bg, color: C.text, fontSize: "13px", overflow: "hidden" }}>
      {/* ── Summary Bar ── */}
      <div style={{ display: "flex", gap: "8px", padding: "12px 16px", flexWrap: "wrap", borderBottom: `1px solid ${C.cardBorder}` }}>
        {[
          { label: "Total", key: "total_sessions", color: C.text },
          { label: "Active", key: "total_active", color: C.green },
          { label: "Local", key: "active_local", color: C.blue },
          { label: "Remote", key: "active_remote", color: C.orange },
          { label: "Teleport", key: "active_teleport", color: C.accent },
          { label: "Idle", key: "total_idle", color: C.muted },
          { label: "Paused", key: "total_paused", color: C.yellow },
          { label: "Errors", key: "total_errors", color: C.red },
        ].map((m) => (
          <div key={m.key} style={{ display: "flex", alignItems: "center", gap: "5px", background: C.card, padding: "5px 12px", borderRadius: "8px", border: `1px solid ${C.cardBorder}` }}>
            <span style={{ fontSize: "11px", color: C.muted }}>{m.label}</span>
            <span style={{ fontSize: "14px", fontWeight: 700, color: m.color }}>{(summaryBar as any)[m.key] ?? "-"}</span>
          </div>
        ))}
        <button onClick={handleRefresh} style={{ ...btnBase, background: C.card, color: C.muted, border: `1px solid ${C.cardBorder}`, marginLeft: "auto" }} title="Refresh">
          {"\u21BB"} Refresh
        </button>
      </div>

      {/* ── Toolbar: tabs + filters + search ── */}
      <div style={{ display: "flex", gap: "10px", padding: "10px 16px", alignItems: "center", flexWrap: "wrap", borderBottom: `1px solid ${C.cardBorder}` }}>
        {/* Type tabs */}
        <div style={{ display: "flex", gap: "3px", background: C.card, padding: "3px", borderRadius: "8px", border: `1px solid ${C.cardBorder}` }}>
          {(["all", "local", "remote", "teleport"] as TypeTab[]).map((t) => (
            <button key={t} onClick={() => setActiveTab(t)}
              style={{ padding: "5px 14px", borderRadius: "6px", border: "none", background: activeTab === t ? C.accent : "transparent", color: activeTab === t ? "#fff" : C.muted, fontSize: "12px", fontWeight: activeTab === t ? 600 : 400, cursor: "pointer", textTransform: "capitalize" }}>
              {t === "all" ? "All" : `${TYPE_ICONS[t] ?? ""} ${t}`}
            </button>
          ))}
        </div>

        {/* Surface filter */}
        <select value={surfaceFilter} onChange={(e) => setSurfaceFilter(e.target.value)}
          style={{ padding: "5px 10px", borderRadius: "6px", border: `1px solid ${C.cardBorder}`, background: C.card, color: C.text, fontSize: "12px", cursor: "pointer" }}>
          {SURFACES.map((s) => <option key={s} value={s}>{s === "all" ? "All surfaces" : s}</option>)}
        </select>

        {/* Search */}
        <input
          placeholder="Search name / project / tags..." value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
          style={{ flex: "1", minWidth: "180px", padding: "5px 10px", borderRadius: "6px", border: `1px solid ${C.cardBorder}`, background: C.card, color: C.text, fontSize: "12px", outline: "none" }}
        />

        {/* Group by */}
        <select value={groupBy} onChange={(e) => handleGroupBy(e.target.value as GroupBy)}
          style={{ padding: "5px 10px", borderRadius: "6px", border: `1px solid ${C.cardBorder}`, background: C.card, color: C.text, fontSize: "12px", cursor: "pointer" }}>
          <option value="none">No grouping</option>
          <option value="project">Group by Project</option>
          <option value="type_">Group by Type</option>
          <option value="status">Group by Status</option>
          <option value="surface">Group by Surface</option>
        </select>
      </div>

      {/* ── Sessions List ── */}
      <div style={{ flex: 1, overflowY: "auto", padding: "8px 16px 16px" }}>
        {loading ? (
          <div style={{ textAlign: "center", padding: "40px", color: C.muted }}>Loading sessions...</div>
        ) : filtered.length === 0 ? (
          <div style={{ textAlign: "center", padding: "40px", color: C.muted }}>No sessions found.</div>
        ) : grouped ? (
          grouped.map(([key, items]) => (
            <div key={key} style={{ marginBottom: "14px" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px", padding: "6px 8px 4px" }}>
                <span style={{ fontSize: "13px", fontWeight: 700, color: C.text, textTransform: "capitalize" }}>{key}</span>
                <span style={{ fontSize: "11px", color: C.muted, background: C.card, padding: "1px 8px", borderRadius: "10px" }}>{items.length}</span>
              </div>
              {items.map((s) => renderSessionCard(s))}
            </div>
          ))
        ) : (
          filtered.map((s) => renderSessionCard(s))
        )}
      </div>

      {/* ── Legend ── */}
      <div style={{ display: "flex", gap: "14px", padding: "6px 16px", borderTop: `1px solid ${C.cardBorder}`, fontSize: "11px", color: C.muted }}>
        <span>{"\uD83D\uDDA5\uFE0F"} Local</span>
        <span>{"\u2601\uFE0F"} Remote</span>
        <span>{"\uD83D\uDD17"} Teleport</span>
        <span>{"\uD83D\uDFE2"} Active</span>
        <span>{"\u26AA"} Idle</span>
        <span>{"\uD83D\uDFE1"} Paused</span>
        <span>{"\uD83D\uDD34"} Error</span>
      </div>
    </div>
  );

  function renderSessionCard(s: UnifiedSession) {
    const expanded = expandedId === s.id;
    const statusStyle = STATUS_BADGE[s.status] ?? { icon: "\u2753", color: C.muted };

    return (
      <div key={s.id} style={{
        background: C.card, border: `1px solid ${expanded ? C.accent : C.cardBorder}`,
        borderRadius: "8px", padding: expanded ? "12px" : "8px 12px", marginBottom: "6px",
        cursor: "pointer", transition: "all 0.12s",
      }}
        onClick={() => setExpandedId(expanded ? null : s.id)}
      >
        {/* Mini row */}
        <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
          <span style={{ fontSize: "15px", lineHeight: 1 }}>{TYPE_ICONS[s.type_] ?? "\uD83D\uDCCB"}</span>
          <span style={{ fontWeight: 600, fontSize: "13px", minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>{s.name}</span>
          <span style={{ fontSize: "11px", padding: "2px 8px", borderRadius: "4px", background: C.cardBorder, color: C.muted, textTransform: "capitalize" }}>{s.surface}</span>
          <span style={{ fontSize: "11px", display: "flex", alignItems: "center", gap: "3px", color: statusStyle.color, fontWeight: 600 }}>
            {statusStyle.icon} {s.status}
          </span>
          {s.project && <span style={{ fontSize: "11px", color: C.blue }}>{s.project}</span>}
          <span style={{ fontSize: "11px", color: C.muted }}>{s.command_count ?? 0} cmd</span>
          <span style={{ fontSize: "11px", color: C.muted }}>{s.file_changes ?? 0} files</span>
          <span style={{ fontSize: "11px", color: C.muted }}>{fmtDuration(s.active_duration_minutes)}</span>

          {/* Action buttons */}
          <div style={{ marginLeft: "auto", display: "flex", gap: "4px" }} onClick={(e) => e.stopPropagation()}>
            {s.status === "active" ? (
              <button onClick={() => handleDisconnect(s.id)} style={{ ...btnBase, background: "rgba(248,113,113,0.15)", color: C.red, border: `1px solid rgba(248,113,113,0.3)` }}>
                Disconnect
              </button>
            ) : (
              <button onClick={() => handleConnect(s.id)} style={{ ...btnBase, background: "rgba(124,92,252,0.15)", color: C.accent, border: `1px solid rgba(124,92,252,0.3)` }}>
                Connect
              </button>
            )}
          </div>
        </div>

        {/* Expanded detail */}
        {expanded && (
          <div style={{ marginTop: "12px", paddingTop: "10px", borderTop: `1px solid ${C.cardBorder}`, display: "flex", flexDirection: "column", gap: "8px" }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: "6px" }}>
              <DetailRow label="ID" value={s.id} />
              <DetailRow label="Type" value={s.type_} />
              <DetailRow label="Surface" value={s.surface} />
              <DetailRow label="Status" value={s.status} />
              <DetailRow label="Project" value={s.project ?? "-"} />
              <DetailRow label="Started" value={fmtTime(s.started_at)} />
              <DetailRow label="Last Active" value={fmtTime(s.last_active_at)} />
              <DetailRow label="Duration" value={fmtDuration(s.active_duration_minutes)} />
              <DetailRow label="Commands" value={String(s.command_count ?? 0)} />
              <DetailRow label="File Changes" value={String(s.file_changes ?? 0)} />
              <DetailRow label="Errors" value={String(s.error_count ?? 0)} />
              {s.remote_host && <DetailRow label="Remote Host" value={s.remote_host} />}
              {s.remote_location && <DetailRow label="Location" value={s.remote_location} />}
              <DetailRow label="Sync" value={s.sync_status ?? "-"} />
            </div>

            {/* Tags */}
            <div style={{ display: "flex", alignItems: "center", gap: "6px", flexWrap: "wrap" }}>
              <span style={{ fontSize: "11px", color: C.muted, fontWeight: 600 }}>Tags:</span>
              {s.tags.map((t) => (
                <span key={t} style={{ display: "inline-flex", alignItems: "center", gap: "3px", fontSize: "11px", padding: "1px 8px", borderRadius: "10px", background: "rgba(124,92,252,0.15)", color: C.accent, border: `1px solid rgba(124,92,252,0.25)` }}>
                  #{t}
                  <button onClick={() => handleRemoveTag(s.id, t)}
                    style={{ background: "none", border: "none", color: C.muted, cursor: "pointer", fontSize: "11px", padding: 0, lineHeight: 1 }}>
                    {"\u2715"}
                  </button>
                </span>
              ))}
              <div style={{ display: "inline-flex", gap: "4px", alignItems: "center" }}>
                <input
                  value={newTagInput[s.id] ?? ""}
                  onChange={(e) => setNewTagInput((prev) => ({ ...prev, [s.id]: e.target.value }))}
                  onKeyDown={(e) => { if (e.key === "Enter") handleAddTag(s.id); }}
                  placeholder="+ tag"
                  style={{ width: "70px", padding: "2px 6px", borderRadius: "4px", border: `1px solid ${C.cardBorder}`, background: C.bg, color: C.text, fontSize: "11px", outline: "none" }}
                />
                <button onClick={() => handleAddTag(s.id)} style={{ ...btnBase, padding: "2px 8px", fontSize: "11px", background: "rgba(124,92,252,0.15)", color: C.accent, border: `1px solid rgba(124,92,252,0.25)` }}>
                  Add
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  }
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: "flex", gap: "4px", fontSize: "11px", alignItems: "center" }}>
      <span style={{ color: C.muted, minWidth: "70px", flexShrink: 0 }}>{label}</span>
      <span style={{ color: C.text, fontWeight: 500, wordBreak: "break-all" }}>{value}</span>
    </div>
  );
}
