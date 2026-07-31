import React, { useCallback, useEffect, useState } from "react";
import {
  memoryList,
  memorySearch,
  memoryCreate,
  memoryUpdate,
  memoryDelete,
  memoryPin,
  memoryUnpin,
  memoryStats,
  memoryTimeline,
  memoryConsolidateNow,
  memoryClear,
  memoryExport,
  memoryImport,
  memoryConfig,
  memorySetConfig,
  type MemoryEntry,
  type MemoryStats,
  type MemoryConfig,
} from "../commands";

const KIND_COLORS: Record<string, string> = {
  preference: "#8b5cf6",
  fact: "#3b82f6",
  skill: "#10b981",
  knowledge: "#f59e0b",
};

const KIND_OPTIONS = ["preference", "fact", "skill", "knowledge"];

function fmt(s: string) {
  return s ? new Date(s).toLocaleString() : "-";
}

const MemoryManager: React.FC = () => {
  const [entries, setEntries] = useState<MemoryEntry[]>([]);
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [config, setConfig] = useState<MemoryConfig | null>(null);
  const [timeline, setTimeline] = useState<{ date: string; entries_created: number; entries_accessed: number; top_topic: string }[]>([]);
  const [filterKind, setFilterKind] = useState<string | undefined>(undefined);
  const [searchQuery, setSearchQuery] = useState("");
  const [selected, setSelected] = useState<MemoryEntry | null>(null);
  const [consolidateResult, setConsolidateResult] = useState<{ consolidated: number; deleted_duplicates: number; duration_ms: number } | null>(null);
  const [tab, setTab] = useState<"list" | "timeline" | "settings">("list");
  const [showCreate, setShowCreate] = useState(false);
  const [loading, setLoading] = useState(false);
  const [clearKind, setClearKind] = useState<string>("");

  const createForm = { kind: "fact", content: "", summary: "", tags: "", source: "" };
  const [form, setForm] = useState(createForm);

  const load = useCallback(async (kind?: string) => {
    setLoading(true);
    try {
      const [e, s, c, tl] = await Promise.all([
        memoryList(kind),
        memoryStats(),
        memoryConfig(),
        memoryTimeline(30),
      ]);
      setEntries(e);
      setStats(s);
      setConfig(c);
      setTimeline(tl);
    } catch { }
    setLoading(false);
  }, []);

  useEffect(() => { load(filterKind); }, [load, filterKind]);

  const handleSearch = useCallback(async (q: string) => {
    setSearchQuery(q);
    if (!q.trim()) { load(filterKind); return; }
    setLoading(true);
    try {
      const res = await memorySearch(q, filterKind);
      setEntries(res.results);
    } catch { }
    setLoading(false);
  }, [filterKind, load]);

  const handleCreate = useCallback(async () => {
    if (!form.content.trim()) return;
    setLoading(true);
    try {
      await memoryCreate(
        form.kind,
        form.content,
        form.summary || undefined,
        form.tags ? form.tags.split(",").map((t) => t.trim()).filter(Boolean) : undefined,
        form.source || undefined,
      );
      setForm(createForm);
      setShowCreate(false);
      await load(filterKind);
    } catch { }
    setLoading(false);
  }, [form, filterKind, load]);

  const handlePin = useCallback(async (id: string, pinned: boolean) => {
    try {
      if (pinned) await memoryUnpin(id);
      else await memoryPin(id);
      if (selected?.id === id) setSelected({ ...selected, is_pinned: !pinned });
      await load(filterKind);
    } catch { }
  }, [filterKind, load, selected]);

  const handleDelete = useCallback(async (id: string) => {
    try {
      await memoryDelete(id);
      if (selected?.id === id) setSelected(null);
      await load(filterKind);
    } catch { }
  }, [filterKind, load, selected]);

  const handleUpdate = useCallback(async () => {
    if (!selected) return;
    try {
      await memoryUpdate(
        selected.id,
        selected.content,
        selected.summary || undefined,
        selected.tags,
        selected.confidence,
        selected.is_pinned,
      );
      await load(filterKind);
    } catch { }
  }, [selected, filterKind, load]);

  const handleConsolidate = useCallback(async () => {
    setLoading(true);
    try {
      const res = await memoryConsolidateNow();
      setConsolidateResult(res);
      await load(filterKind);
    } catch { }
    setLoading(false);
  }, [load]);

  const handleClear = useCallback(async () => {
    try {
      const deleted = await memoryClear(clearKind || undefined);
      await load(filterKind);
      alert(`Deleted ${deleted} entries`);
    } catch { }
  }, [clearKind, filterKind, load]);

  const handleExport = useCallback(async () => {
    try {
      const data = await memoryExport("json");
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url; a.download = `neotrix-memory-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch { }
  }, []);

  const handleImport = useCallback(async () => {
    const input = document.createElement("input");
    input.type = "file"; input.accept = ".json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const count = await memoryImport(text);
        alert(`Imported ${count} entries`);
        await load(filterKind);
      } catch { }
    };
    input.click();
  }, [filterKind, load]);

  const handleConfigUpdate = useCallback(async (patch: Partial<MemoryConfig>) => {
    if (!config) return;
    const next = { ...config, ...patch };
    try {
      await memorySetConfig(next);
      setConfig(next);
    } catch { }
  }, [config]);

  const FILTER_TABS = [
    { label: "All", value: undefined },
    { label: "Preferences", value: "preference" },
    { label: "Facts", value: "fact" },
    { label: "Skills", value: "skill" },
    { label: "Knowledge", value: "knowledge" },
  ];

  return (
    <div style={{ display: "flex", height: "100%", background: "#0d1117", color: "#e6edf3", fontFamily: "system-ui, sans-serif" }}>
      {/* ── Left panel: entries list ── */}
      <div style={{ width: 380, minWidth: 380, borderRight: "1px solid #21262d", display: "flex", flexDirection: "column" }}>
        {/* Toolbar */}
        <div style={{ padding: "12px 16px", borderBottom: "1px solid #21262d" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
            <span style={{ fontSize: 18, fontWeight: 600 }}>Memory</span>
            <span style={{ fontSize: 11, color: "#8b949e", background: "#161b22", padding: "2px 8px", borderRadius: 10 }}>{stats?.total_entries ?? 0}</span>
            <div style={{ flex: 1 }} />
            <button onClick={() => setShowCreate(!showCreate)} style={{ background: "#238636", color: "#fff", border: "none", borderRadius: 6, padding: "4px 12px", fontSize: 12, cursor: "pointer" }}>+ New</button>
          </div>
          {/* Filter tabs */}
          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
            {FILTER_TABS.map((t) => (
              <button key={t.label} onClick={() => { setFilterKind(t.value); setSearchQuery(""); }}
                style={{
                  background: filterKind === t.value ? "#30363d" : "transparent", color: filterKind === t.value ? "#e6edf3" : "#8b949e",
                  border: "1px solid", borderColor: filterKind === t.value ? "#58a6ff" : "#21262d", borderRadius: 14, padding: "2px 10px", fontSize: 11, cursor: "pointer",
                }}>
                {t.label}
              </button>
            ))}
          </div>
          {/* Search */}
          <div style={{ marginTop: 8, position: "relative" }}>
            <input value={searchQuery} onChange={(e) => handleSearch(e.target.value)} placeholder="Search memory…"
              style={{ width: "100%", padding: "6px 10px 6px 28px", background: "#161b22", border: "1px solid #30363d", borderRadius: 8, color: "#e6edf3", fontSize: 12, outline: "none" }} />
            <span style={{ position: "absolute", left: 8, top: 6, color: "#8b949e", fontSize: 12 }}>🔍</span>
          </div>
        </div>

        {/* Entries list */}
        <div style={{ flex: 1, overflow: "auto" }}>
          {loading && entries.length === 0 && <div style={{ padding: 24, textAlign: "center", color: "#8b949e", fontSize: 12 }}>Loading…</div>}
          {!loading && entries.length === 0 && <div style={{ padding: 24, textAlign: "center", color: "#8b949e", fontSize: 12 }}>No entries</div>}
          {entries.map((e) => (
            <div key={e.id} onClick={() => setSelected(e)}
              style={{
                padding: "10px 16px", borderBottom: "1px solid #21262d", cursor: "pointer",
                background: selected?.id === e.id ? "#1c2128" : "transparent",
                transition: "background 0.1s",
              }}>
              <div style={{ display: "flex", alignItems: "center", gap: 6, marginBottom: 4 }}>
                <span style={{
                  background: KIND_COLORS[e.kind] || "#8b949e", color: "#fff", borderRadius: 4,
                  padding: "1px 6px", fontSize: 10, fontWeight: 600, textTransform: "uppercase",
                }}>{e.kind}</span>
                {e.is_pinned && <span style={{ fontSize: 11 }}>📌</span>}
                <span style={{ fontSize: 11, color: "#8b949e", marginLeft: "auto" }}>{fmt(e.last_accessed_at)}</span>
              </div>
              <div style={{ fontSize: 12, color: "#c9d1d9", marginBottom: 4, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {e.summary || e.content.slice(0, 80)}
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                <div style={{ flex: 1, height: 4, background: "#21262d", borderRadius: 2 }}>
                  <div style={{ width: `${(e.confidence * 100).toFixed(0)}%`, height: 4, background: KIND_COLORS[e.kind] || "#8b949e", borderRadius: 2 }} />
                </div>
                <span style={{ fontSize: 10, color: "#8b949e" }}>{(e.confidence * 100).toFixed(0)}%</span>
              </div>
              {e.tags.length > 0 && (
                <div style={{ display: "flex", gap: 4, marginTop: 4, flexWrap: "wrap" }}>
                  {e.tags.slice(0, 3).map((t) => (
                    <span key={t} style={{ background: "#161b22", color: "#8b949e", borderRadius: 4, padding: "0 4px", fontSize: 10 }}>{t}</span>
                  ))}
                  {e.tags.length > 3 && <span style={{ fontSize: 10, color: "#8b949e" }}>+{e.tags.length - 3}</span>}
                </div>
              )}
            </div>
          ))}
        </div>

        {/* Stats bar */}
        {stats && (
          <div style={{ padding: "8px 16px", borderTop: "1px solid #21262d", fontSize: 11, color: "#8b949e" }}>
            <div style={{ display: "flex", gap: 16 }}>
              <span>Avg {(stats.avg_confidence * 100).toFixed(0)}%</span>
              <span>{(stats.memory_usage_bytes / 1024).toFixed(0)} KB</span>
              <span style={{ flex: 1 }} />
              {stats.top_tags.slice(0, 3).map(([t]) => (
                <span key={t} style={{ background: "#161b22", borderRadius: 4, padding: "0 4px" }}>{t}</span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* ── Right panel: detail + actions ── */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column", overflow: "auto" }}>
        {/* Tab bar */}
        <div style={{ display: "flex", borderBottom: "1px solid #21262d" }}>
          {(["list", "timeline", "settings"] as const).map((t) => (
            <button key={t} onClick={() => setTab(t)} style={{
              flex: 1, padding: "10px", background: tab === t ? "#1c2128" : "transparent",
              border: "none", borderBottom: tab === t ? "2px solid #58a6ff" : "2px solid transparent",
              color: tab === t ? "#e6edf3" : "#8b949e", fontSize: 12, cursor: "pointer", textTransform: "capitalize",
            }}>
              {t === "timeline" ? "📈 Timeline" : t === "settings" ? "⚙️ Settings" : "📝 Detail"}
            </button>
          ))}
        </div>

        <div style={{ flex: 1, overflow: "auto", padding: 16 }}>
          {/* ── Detail View ── */}
          {tab === "list" && (
            selected ? (
              <div>
                <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
                  <span style={{
                    background: KIND_COLORS[selected.kind] || "#8b949e", color: "#fff",
                    borderRadius: 4, padding: "2px 8px", fontSize: 11, fontWeight: 600, textTransform: "uppercase",
                  }}>{selected.kind}</span>
                  <button onClick={() => handlePin(selected.id, selected.is_pinned)}
                    style={{ background: "transparent", border: "1px solid #30363d", borderRadius: 6, padding: "4px 8px", color: selected.is_pinned ? "#f59e0b" : "#8b949e", fontSize: 12, cursor: "pointer" }}>
                    {selected.is_pinned ? "Unpin" : "Pin"} {selected.is_pinned ? "📌" : "📍"}
                  </button>
                  <button onClick={() => handleDelete(selected.id)}
                    style={{ background: "transparent", border: "1px solid #30363d", borderRadius: 6, padding: "4px 8px", color: "#f85149", fontSize: 12, cursor: "pointer", marginLeft: "auto" }}>
                    🗑 Delete
                  </button>
                </div>

                <label style={{ fontSize: 11, color: "#8b949e", display: "block", marginBottom: 4 }}>Content</label>
                <textarea value={selected.content} onChange={(e) => setSelected({ ...selected, content: e.target.value })}
                  style={{ width: "100%", minHeight: 80, padding: 8, background: "#161b22", border: "1px solid #30363d", borderRadius: 8, color: "#e6edf3", fontSize: 12, resize: "vertical", marginBottom: 12 }} />

                <label style={{ fontSize: 11, color: "#8b949e", display: "block", marginBottom: 4 }}>Summary</label>
                <input value={selected.summary || ""} onChange={(e) => setSelected({ ...selected, summary: e.target.value })}
                  style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 8, color: "#e6edf3", fontSize: 12, marginBottom: 12 }} />

                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12, marginBottom: 12 }}>
                  <div>
                    <label style={{ fontSize: 11, color: "#8b949e", display: "block", marginBottom: 4 }}>Confidence</label>
                    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                      <input type="range" min={0} max={1} step={0.05} value={selected.confidence}
                        onChange={(e) => setSelected({ ...selected, confidence: parseFloat(e.target.value) })}
                        style={{ flex: 1 }} />
                      <span style={{ fontSize: 12, color: "#c9d1d9", minWidth: 36 }}>{(selected.confidence * 100).toFixed(0)}%</span>
                    </div>
                  </div>
                  <div>
                    <label style={{ fontSize: 11, color: "#8b949e", display: "block", marginBottom: 4 }}>Tags</label>
                    <input value={selected.tags.join(", ")} onChange={(e) => setSelected({ ...selected, tags: e.target.value.split(",").map((t) => t.trim()).filter(Boolean) })}
                      style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 8, color: "#e6edf3", fontSize: 12 }} />
                  </div>
                </div>

                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 12, marginBottom: 16, fontSize: 11, color: "#8b949e" }}>
                  <div>Created: {fmt(selected.created_at)}</div>
                  <div>Last accessed: {fmt(selected.last_accessed_at)}</div>
                  <div>Access count: {selected.access_count}</div>
                </div>

                <button onClick={handleUpdate}
                  style={{ background: "#238636", color: "#fff", border: "none", borderRadius: 6, padding: "8px 20px", fontSize: 12, cursor: "pointer" }}>
                  💾 Save Changes
                </button>
              </div>
            ) : (
              <div style={{ textAlign: "center", color: "#8b949e", paddingTop: 60 }}>
                <div style={{ fontSize: 32, marginBottom: 8 }}>🧠</div>
                <div style={{ fontSize: 13 }}>Select a memory entry to view details</div>
              </div>
            )
          )}

          {/* ── Timeline View ── */}
          {tab === "timeline" && (
            <div>
              <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>📈 Memory Activity (30 days)</h3>
              {timeline.length === 0 && <div style={{ color: "#8b949e", fontSize: 12 }}>No activity data</div>}
              {timeline.map((d) => {
                const max = Math.max(...timeline.map((x) => x.entries_created), 1);
                const h = (d.entries_created / max) * 100;
                return (
                  <div key={d.date} style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4, fontSize: 11 }}>
                    <span style={{ width: 60, color: "#8b949e", flexShrink: 0 }}>{d.date.slice(5)}</span>
                    <div style={{ flex: 1, height: 12, background: "#161b22", borderRadius: 4, position: "relative" }}>
                      <div style={{ width: `${h}%`, height: 12, background: "#3b82f6", borderRadius: 4, opacity: 0.8 }} />
                      <div style={{ position: "absolute", top: 0, right: 4, fontSize: 9, lineHeight: "12px", color: "#e6edf3" }}>
                        {d.entries_created > 0 ? d.entries_created : ""}
                      </div>
                    </div>
                    <span style={{ color: "#8b949e", width: 80, textAlign: "right", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{d.top_topic}</span>
                  </div>
                );
              })}
            </div>
          )}

          {/* ── Settings / Actions ── */}
          {tab === "settings" && (
            <div>
              <h3 style={{ fontSize: 14, fontWeight: 600, marginBottom: 12 }}>⚙️ Memory Settings</h3>

              {config && (
                <div style={{ marginBottom: 20 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                    <label style={{ fontSize: 12 }}>Enabled</label>
                    <input type="checkbox" checked={config.enabled} onChange={(e) => handleConfigUpdate({ enabled: e.target.checked })} />
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                    <label style={{ fontSize: 12, width: 140 }}>Auto-consolidate</label>
                    <input type="checkbox" checked={config.auto_consolidate} onChange={(e) => handleConfigUpdate({ auto_consolidate: e.target.checked })} />
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                    <label style={{ fontSize: 12, width: 140 }}>Consolidation interval (min)</label>
                    <input type="number" value={config.consolidation_interval_mins} onChange={(e) => handleConfigUpdate({ consolidation_interval_mins: parseInt(e.target.value) || 60 })}
                      style={{ width: 80, padding: "4px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12 }} />
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
                    <label style={{ fontSize: 12, width: 140 }}>Max entries</label>
                    <input type="number" value={config.max_entries} onChange={(e) => handleConfigUpdate({ max_entries: parseInt(e.target.value) || 1000 })}
                      style={{ width: 80, padding: "4px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12 }} />
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                    <label style={{ fontSize: 12, width: 140 }}>Enable search</label>
                    <input type="checkbox" checked={config.enable_search} onChange={(e) => handleConfigUpdate({ enable_search: e.target.checked })} />
                  </div>
                </div>
              )}

              {/* Actions */}
              <div style={{ borderTop: "1px solid #21262d", paddingTop: 16 }}>
                <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>Actions</h4>
                <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                  <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <button onClick={handleConsolidate} disabled={loading}
                      style={{ background: "#1f6feb", color: "#fff", border: "none", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: "pointer" }}>
                      🔄 Consolidate Now
                    </button>
                    {consolidateResult && (
                      <span style={{ fontSize: 11, color: "#8b949e" }}>
                        {consolidateResult.consolidated} consolidated, {consolidateResult.deleted_duplicates} dupes removed, {consolidateResult.duration_ms}ms
                      </span>
                    )}
                  </div>

                  <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <select value={clearKind} onChange={(e) => setClearKind(e.target.value)}
                      style={{ padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12 }}>
                      <option value="">All kinds</option>
                      {KIND_OPTIONS.map((k) => <option key={k} value={k}>{k}</option>)}
                    </select>
                    <button onClick={handleClear}
                      style={{ background: "#da3633", color: "#fff", border: "none", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: "pointer" }}>
                      🗑 Clear
                    </button>
                  </div>

                  <div style={{ display: "flex", gap: 8 }}>
                    <button onClick={handleExport}
                      style={{ background: "#21262d", color: "#e6edf3", border: "1px solid #30363d", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: "pointer" }}>
                      📤 Export JSON
                    </button>
                    <button onClick={handleImport}
                      style={{ background: "#21262d", color: "#e6edf3", border: "1px solid #30363d", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: "pointer" }}>
                      📥 Import JSON
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* ── Create Form ── */}
          {showCreate && (
            <div style={{ borderTop: "1px solid #21262d", padding: 16, marginTop: 16 }}>
              <h4 style={{ fontSize: 13, fontWeight: 600, marginBottom: 8 }}>New Memory Entry</h4>
              <div style={{ marginBottom: 8 }}>
                <select value={form.kind} onChange={(e) => setForm({ ...form, kind: e.target.value })}
                  style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12 }}>
                  {KIND_OPTIONS.map((k) => <option key={k} value={k}>{k}</option>)}
                </select>
              </div>
              <textarea value={form.content} onChange={(e) => setForm({ ...form, content: e.target.value })} placeholder="Content…"
                style={{ width: "100%", minHeight: 60, padding: 8, background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12, resize: "vertical", marginBottom: 8 }} />
              <input value={form.summary} onChange={(e) => setForm({ ...form, summary: e.target.value })} placeholder="Summary (optional)"
                style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12, marginBottom: 8 }} />
              <input value={form.tags} onChange={(e) => setForm({ ...form, tags: e.target.value })} placeholder="Tags (comma-separated)"
                style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12, marginBottom: 8 }} />
              <input value={form.source} onChange={(e) => setForm({ ...form, source: e.target.value })} placeholder="Source (optional)"
                style={{ width: "100%", padding: "6px 8px", background: "#161b22", border: "1px solid #30363d", borderRadius: 6, color: "#e6edf3", fontSize: 12, marginBottom: 12 }} />
              <div style={{ display: "flex", gap: 8 }}>
                <button onClick={handleCreate} disabled={loading || !form.content.trim()}
                  style={{ background: "#238636", color: "#fff", border: "none", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: loading ? "not-allowed" : "pointer", opacity: loading || !form.content.trim() ? 0.6 : 1 }}>
                  ➕ Create
                </button>
                <button onClick={() => { setShowCreate(false); setForm(createForm); }}
                  style={{ background: "transparent", color: "#8b949e", border: "1px solid #30363d", borderRadius: 6, padding: "8px 16px", fontSize: 12, cursor: "pointer" }}>
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default MemoryManager;
