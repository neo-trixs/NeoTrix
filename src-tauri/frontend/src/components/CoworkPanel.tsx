import React, { useState, useEffect, useCallback } from "react";
import type { CoworkSession, CoworkFile, CoworkDeliverable } from "../commands";

/* ───────── helpers ───────── */

const STATUS_COLORS: Record<string, string> = {
  active: "#2ecc71",
  paused: "#f39c12",
  completed: "#3498db",
  failed: "#e74c3c",
};

const TEMPLATES = [
  { id: "doc_report", name: "Document Report", description: "Generate a structured document report from workspace files", category: "documentation", suggested_prompt: "Analyze the workspace and produce a comprehensive document report covering purpose, structure, key components, and usage." },
  { id: "code_review", name: "Code Review", description: "Run code review across the workspace with quality scoring", category: "development", suggested_prompt: "Review all source files in the workspace. Check for code quality, security issues, performance problems, and adherence to best practices. Score each file." },
  { id: "translation", name: "Translation", description: "Translate project strings or documentation to target languages", category: "content", suggested_prompt: "Find all locale or documentation files and translate them to the specified target language while preserving formatting and variables." },
  { id: "data_analysis", name: "Data Analysis", description: "Analyze structured data files and produce summary statistics", category: "analysis", suggested_prompt: "Scan the workspace for data files (CSV, JSON, etc.), load them, compute summary statistics, and produce an analysis report." },
  { id: "research_summary", name: "Research Summary", description: "Crawl and summarize research materials into a brief", category: "research", suggested_prompt: "Find all research documents, notes, and markdown files. Extract key findings, open questions, and methodology notes into a structured summary." },
  { id: "api_docs", name: "API Docs", description: "Generate API documentation from source code comments", category: "documentation", suggested_prompt: "Parse all source code files to extract function signatures, type definitions, and doc comments. Produce API documentation in Markdown format." },
];

function fmtTime(iso: string | null): string {
  if (!iso) return "—";
  const d = new Date(iso);
  const diff = Date.now() - d.getTime();
  if (diff < 60000) return "just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString();
}

function fmtBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

const tagInputStyle: React.CSSProperties = {
  background: "#1a1a2e", border: "1px solid #2d2d4a", borderRadius: 6,
  padding: "6px 10px", color: "#e0e0e0", fontSize: 12, outline: "none", width: "100%",
  fontFamily: "inherit", boxSizing: "border-box",
};

/* ───────── sub-components ───────── */

function StatusBadge({ status }: { status: string }) {
  const color = STATUS_COLORS[status] || "#888";
  return (
    <span style={{ display: "inline-flex", alignItems: "center", gap: 4, fontSize: 11, fontWeight: 600, color, textTransform: "capitalize" }}>
      <span style={{ width: 6, height: 6, borderRadius: "50%", background: color }} />
      {status}
    </span>
  );
}

const btnBase: React.CSSProperties = {
  padding: "6px 14px", borderRadius: 6, border: "none", cursor: "pointer",
  fontSize: 12, fontWeight: 600, transition: "all .15s",
};

function ActionBtn({ label, color, onClick, disabled, loading }: { label: string; color: string; onClick: () => void; disabled?: boolean; loading?: boolean }) {
  return (
    <button
      style={{ ...btnBase, background: color, color: "#fff", opacity: disabled ? 0.4 : 1, cursor: disabled ? "default" : "pointer" }}
      onClick={onClick}
      disabled={disabled}
    >
      {loading ? "..." : label}
    </button>
  );
}

/* ───────── main component ───────── */

const CoworkPanel: React.FC = () => {
  const [sessions, setSessions] = useState<CoworkSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [showNewForm, setShowNewForm] = useState(false);
  const [showConfig, setShowConfig] = useState(false);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  /* new session form */
  const [newWorkspace, setNewWorkspace] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [newName, setNewName] = useState("");
  const [newTagsStr, setNewTagsStr] = useState("");

  /* session detail tabs */
  const [detailTab, setDetailTab] = useState<"files" | "deliverables" | "actions" | "templates">("files");

  /* files state */
  const [files, setFiles] = useState<CoworkFile[]>([]);
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [fileLoading, setFileLoading] = useState(false);
  const [filePattern, setFilePattern] = useState("");

  /* deliverables state */
  const [deliverables, setDeliverables] = useState<CoworkDeliverable[]>([]);

  /* actions state */
  const [actions, setActions] = useState<Array<{ id: string; action_type: string; target_path: string; status: string; started_at: string; completed_at: string | null; details: string | null; result_summary: string | null }>>([]);

  /* config state */
  const [config, setConfig] = useState<{ max_files_per_scan: number; max_file_size_kb: number; auto_save: boolean; allow_file_create: boolean; allow_file_modify: boolean; allow_file_delete: boolean } | null>(null);

  const mod = React.useRef<typeof import("../commands") | null>(null);
  const fetchMod = useCallback(async () => {
    if (!mod.current) mod.current = await import("../commands");
    return mod.current;
  }, []);

  const fetchSessions = useCallback(async () => {
    try {
      const m = await fetchMod();
      const list = await m.coworkList();
      setSessions(list);
      setError(null);
    } catch {
      setError("Failed to load cowork sessions");
    }
  }, [fetchMod]);

  const fetchConfig = useCallback(async () => {
    try {
      const m = await fetchMod();
      const cfg = await m.coworkConfig();
      setConfig(cfg);
    } catch { /* ignore */ }
  }, [fetchMod]);

  useEffect(() => {
    Promise.all([fetchSessions(), fetchConfig()]).finally(() => setLoading(false));
  }, [fetchSessions, fetchConfig]);

  const activeSession = sessions.find((s) => s.id === activeSessionId) || null;

  const selectSession = useCallback(async (id: string) => {
    setActiveSessionId(id);
    setFileContent(null);
    setDetailTab("files");
    try {
      const m = await fetchMod();
      const [flist, dlist, alist] = await Promise.all([
        m.coworkScanFiles(id).catch(() => [] as CoworkFile[]),
        m.coworkListDeliverables(id).catch(() => [] as CoworkDeliverable[]),
        m.coworkActions(id).catch(() => []),
      ]);
      setFiles(flist);
      setDeliverables(dlist);
      setActions(alist);
    } catch { /* ignore */ }
  }, [fetchMod]);

  const handleStart = async () => {
    if (!newWorkspace.trim()) return;
    setActionLoading("start");
    try {
      const m = await fetchMod();
      const tags = newTagsStr.split(",").map((t) => t.trim()).filter(Boolean);
      const id = await m.coworkStart(newWorkspace.trim(), newDescription.trim(), newName.trim() || undefined, tags.length ? tags : undefined);
      await fetchSessions();
      setShowNewForm(false);
      setNewWorkspace("");
      setNewDescription("");
      setNewName("");
      setNewTagsStr("");
      setActiveSessionId(id);
    } catch {
      setError("Failed to start session");
    } finally {
      setActionLoading(null);
    }
  };

  const handleSessionAction = async (sessionId: string, action: "pause" | "resume" | "stop") => {
    setActionLoading(`${action}_${sessionId}`);
    try {
      const m = await fetchMod();
      if (action === "pause") await m.coworkPause(sessionId);
      else if (action === "resume") await m.coworkResume(sessionId);
      else await m.coworkStop(sessionId);
      await fetchSessions();
      if (action === "stop") setActiveSessionId(null);
    } catch {
      setError(`Failed to ${action} session`);
    } finally {
      setActionLoading(null);
    }
  };

  const handleScanFiles = async (pattern?: string) => {
    if (!activeSessionId) return;
    setFileLoading(true);
    try {
      const m = await fetchMod();
      const flist = await m.coworkScanFiles(activeSessionId, pattern || undefined);
      setFiles(flist);
    } catch {
      setError("Failed to scan files");
    } finally {
      setFileLoading(false);
    }
  };

  const handleReadFile = async (path: string) => {
    if (!activeSessionId) return;
    setFileLoading(true);
    try {
      const m = await fetchMod();
      const content = await m.coworkReadFile(activeSessionId, path);
      setFileContent(content);
    } catch {
      setError("Failed to read file");
    } finally {
      setFileLoading(false);
    }
  };

  const handleApplyTemplate = async (templateId: string) => {
    if (!activeSessionId) return;
    setActionLoading(`tmpl_${templateId}`);
    try {
      const m = await fetchMod();
      await m.coworkApplyTemplate(activeSessionId, templateId);
      const [flist, dlist, alist] = await Promise.all([
        m.coworkScanFiles(activeSessionId).catch(() => [] as CoworkFile[]),
        m.coworkListDeliverables(activeSessionId).catch(() => [] as CoworkDeliverable[]),
        m.coworkActions(activeSessionId).catch(() => []),
      ]);
      setFiles(flist);
      setDeliverables(dlist);
      setActions(alist);
    } catch {
      setError("Failed to apply template");
    } finally {
      setActionLoading(null);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    setActionLoading("config");
    try {
      const m = await fetchMod();
      await m.coworkSetConfig(config as unknown as Record<string, unknown>);
    } catch {
      setError("Failed to save config");
    } finally {
      setActionLoading(null);
    }
  };

  /* ───────── styles ───────── */

  const containerStyle: React.CSSProperties = {
    display: "flex", height: "100%", background: "#0d0d1a", color: "#e0e0e0",
    fontFamily: "'SF Mono', 'Fira Code', monospace", fontSize: 13, overflow: "hidden",
  };

  const panelBase: React.CSSProperties = {
    display: "flex", flexDirection: "column", height: "100%",
  };

  const sectionHeader: React.CSSProperties = {
    padding: "10px 14px", fontSize: 11, fontWeight: 700, textTransform: "uppercase",
    letterSpacing: "0.5px", color: "#666", borderBottom: "1px solid #1a1a30",
    display: "flex", justifyContent: "space-between", alignItems: "center", flexShrink: 0,
  };

  /* ───────── render ───────── */

  if (loading) {
    return (
      <div style={{ ...containerStyle, alignItems: "center", justifyContent: "center" }}>
        <span style={{ color: "#555", fontSize: 14 }}>Loading cowork sessions...</span>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      {/* left sidebar — session list */}
      <div style={{ ...panelBase, width: 260, borderRight: "1px solid #1a1a30", flexShrink: 0, background: "#0f0f20" }}>
        <div style={sectionHeader}>
          <span>Cowork Sessions</span>
          <div style={{ display: "flex", gap: 4 }}>
            <button
              style={{ ...btnBase, background: "transparent", color: "#888", padding: "4px 8px", fontSize: 11 }}
              onClick={() => setShowConfig(true)}
              title="Config"
            >
              ⚙
            </button>
            <button
              style={{ ...btnBase, background: "var(--nt-accent, #6c5ce7)", color: "#fff", padding: "4px 8px", fontSize: 11 }}
              onClick={() => setShowNewForm(true)}
            >
              + New
            </button>
          </div>
        </div>
        <div style={{ flex: 1, overflowY: "auto", padding: 4 }}>
          {sessions.length === 0 && (
            <div style={{ padding: "20px 14px", color: "#555", fontSize: 12, textAlign: "center" }}>
              No sessions yet. Start a new cowork session.
            </div>
          )}
          {sessions.map((s) => (
            <div
              key={s.id}
              style={{
                padding: "10px 12px", borderRadius: 8, cursor: "pointer", marginBottom: 2,
                background: activeSessionId === s.id ? "#1a1a35" : "transparent",
                transition: "background .15s",
              }}
              onClick={() => selectSession(s.id)}
              onMouseEnter={(e) => { if (activeSessionId !== s.id) e.currentTarget.style.background = "#151528"; }}
              onMouseLeave={(e) => { if (activeSessionId !== s.id) e.currentTarget.style.background = "transparent"; }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 4 }}>
                <span style={{ fontSize: 13, fontWeight: 600, color: "#e0e0e0", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 160 }}>
                  {s.name || s.workspace_path.split("/").pop() || s.id.slice(0, 8)}
                </span>
                <StatusBadge status={s.status} />
              </div>
              <div style={{ fontSize: 10, color: "#555", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {s.workspace_path}
              </div>
              <div style={{ display: "flex", gap: 8, marginTop: 4, fontSize: 10, color: "#666" }}>
                <span>📄 {s.files_read}</span>
                <span>✏️ {s.files_created}</span>
                <span>🔧 {s.files_modified}</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* main content */}
      <div style={{ ...panelBase, flex: 1 }}>
        {!activeSession && (
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", color: "#555", gap: 12 }}>
            <svg viewBox="0 0 48 48" fill="none" style={{ width: 48, height: 48, opacity: 0.3 }}>
              <circle cx="18" cy="15" r="6" stroke="currentColor" strokeWidth="2" />
              <circle cx="33" cy="15" r="6" stroke="currentColor" strokeWidth="2" />
              <path d="M18 27c-7 0-12 4.5-12 10.5v3h24v-3c0-6-5-10.5-12-10.5z" stroke="currentColor" strokeWidth="2" />
              <path d="M33 27c7 0 12 4.5 12 10.5v3H39" stroke="currentColor" strokeWidth="2" />
            </svg>
            <span style={{ fontSize: 14 }}>Select a session to start coworking</span>
          </div>
        )}

        {activeSession && (
          <>
            {/* session header */}
            <div style={{ ...sectionHeader, background: "#111125" }}>
              <div>
                <span style={{ color: "#e0e0e0", fontWeight: 600, fontSize: 13, textTransform: "none", letterSpacing: 0 }}>
                  {activeSession.name || activeSession.workspace_path.split("/").pop() || activeSession.id.slice(0, 8)}
                </span>
                <span style={{ marginLeft: 8, fontSize: 10, color: "#555" }}>· {fmtTime(activeSession.last_active_at)}</span>
              </div>
              <div style={{ display: "flex", gap: 6 }}>
                {activeSession.status === "active" && (
                  <ActionBtn label="Pause" color="#f39c12" onClick={() => handleSessionAction(activeSession.id, "pause")} loading={actionLoading === `pause_${activeSession.id}`} />
                )}
                {activeSession.status === "paused" && (
                  <ActionBtn label="Resume" color="#2ecc71" onClick={() => handleSessionAction(activeSession.id, "resume")} loading={actionLoading === `resume_${activeSession.id}`} />
                )}
                {(activeSession.status === "active" || activeSession.status === "paused") && (
                  <ActionBtn label="Stop" color="#e74c3c" onClick={() => handleSessionAction(activeSession.id, "stop")} loading={actionLoading === `stop_${activeSession.id}`} />
                )}
              </div>
            </div>

            {/* stats row */}
            <div style={{
              display: "flex", gap: 16, padding: "10px 14px", background: "#0f0f20",
              borderBottom: "1px solid #1a1a30", fontSize: 11, color: "#888", flexShrink: 0,
            }}>
              <div><span style={{ color: "#bbb", fontWeight: 600 }}>{activeSession.files_read}</span> read</div>
              <div><span style={{ color: "#bbb", fontWeight: 600 }}>{activeSession.files_created}</span> created</div>
              <div><span style={{ color: "#bbb", fontWeight: 600 }}>{activeSession.files_modified}</span> modified</div>
              <div><span style={{ color: "#bbb", fontWeight: 600 }}>{activeSession.deliverables.length}</span> deliverables</div>
            </div>

            {/* tabs */}
            <div style={{ display: "flex", borderBottom: "1px solid #1a1a30", flexShrink: 0, background: "#111125" }}>
              {(["files", "deliverables", "actions", "templates"] as const).map((tab) => (
                <button
                  key={tab}
                  style={{
                    padding: "8px 14px", fontSize: 11, fontWeight: 600, textTransform: "uppercase",
                    letterSpacing: "0.3px", border: "none", background: "transparent",
                    color: detailTab === tab ? "#e0e0e0" : "#555", cursor: "pointer",
                    borderBottom: detailTab === tab ? "2px solid var(--nt-accent, #6c5ce7)" : "2px solid transparent",
                    transition: "all .15s",
                  }}
                  onClick={() => setDetailTab(tab)}
                >
                  {tab}
                </button>
              ))}
            </div>

            {/* tab content */}
            <div style={{ flex: 1, overflowY: "auto", padding: 12, background: "#0d0d1a" }}>

              {detailTab === "files" && (
                <div>
                  <div style={{ display: "flex", gap: 8, marginBottom: 10 }}>
                    <input
                      style={tagInputStyle}
                      placeholder="Filter pattern (e.g. *.tsx)"
                      value={filePattern}
                      onChange={(e) => setFilePattern(e.target.value)}
                      onKeyDown={(e) => { if (e.key === "Enter") handleScanFiles(filePattern || undefined); }}
                    />
                    <button
                      style={{ ...btnBase, background: "#2d2d4a", color: "#e0e0e0", padding: "6px 12px", fontSize: 11 }}
                      onClick={() => handleScanFiles(filePattern || undefined)}
                    >
                      {fileLoading ? "..." : "Scan"}
                    </button>
                  </div>
                  <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                    {files.length === 0 && !fileLoading && (
                      <div style={{ color: "#555", fontSize: 12, textAlign: "center", padding: 20 }}>
                        No files found. Click Scan to browse the workspace.
                      </div>
                    )}
                    {files.map((f) => (
                      <div
                        key={f.path}
                        style={{
                          display: "flex", justifyContent: "space-between", alignItems: "center",
                          padding: "6px 10px", borderRadius: 6, background: "#111125",
                          cursor: "pointer", transition: "background .15s",
                        }}
                        onClick={() => handleReadFile(f.path)}
                        onMouseEnter={(e) => { e.currentTarget.style.background = "#1a1a30"; }}
                        onMouseLeave={(e) => { e.currentTarget.style.background = "#111125"; }}
                      >
                        <div style={{ display: "flex", alignItems: "center", gap: 8, overflow: "hidden", flex: 1 }}>
                          <span style={{ fontSize: 11, color: "#555", flexShrink: 0 }}>
                            {f.kind === "source" ? "📄" : f.kind === "doc" ? "📝" : f.kind === "data" ? "📊" : f.is_deliverable ? "✅" : "📄"}
                          </span>
                          <div style={{ overflow: "hidden" }}>
                            <div style={{ fontSize: 12, fontWeight: 500, color: "#ccc", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                              {f.relative_path}
                            </div>
                            <div style={{ fontSize: 10, color: "#555", marginTop: 1 }}>
                              {fmtBytes(f.size_bytes)} · {f.kind} · {f.is_deliverable ? "deliverable" : fmtTime(f.last_modified)}
                            </div>
                          </div>
                        </div>
                        {f.content_summary && (
                          <span style={{ fontSize: 10, color: "#666", flexShrink: 0, marginLeft: 8, maxWidth: 120, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {f.content_summary}
                          </span>
                        )}
                      </div>
                    ))}
                  </div>

                  {/* file content viewer */}
                  {fileContent !== null && (
                    <div style={{ marginTop: 12, border: "1px solid #1a1a30", borderRadius: 8, overflow: "hidden" }}>
                      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "6px 10px", background: "#111125", borderBottom: "1px solid #1a1a30" }}>
                        <span style={{ fontSize: 11, color: "#888" }}>File Preview</span>
                        <button
                          style={{ ...btnBase, background: "transparent", color: "#888", padding: "2px 6px", fontSize: 10 }}
                          onClick={() => setFileContent(null)}
                        >
                          Close
                        </button>
                      </div>
                      <pre style={{ margin: 0, padding: 10, fontSize: 11, color: "#aaa", maxHeight: 300, overflow: "auto", background: "#0a0a18", whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
                        {fileContent}
                      </pre>
                    </div>
                  )}
                </div>
              )}

              {detailTab === "deliverables" && (
                <div>
                  {deliverables.length === 0 && (
                    <div style={{ color: "#555", fontSize: 12, textAlign: "center", padding: 20 }}>
                      No deliverables yet. Apply a template or use Cowork to create files.
                    </div>
                  )}
                  <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
                    {deliverables.map((d) => (
                      <div key={d.id} style={{ padding: "10px 12px", borderRadius: 8, background: "#111125", border: "1px solid #1a1a30" }}>
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 4 }}>
                          <span style={{ fontSize: 13, fontWeight: 600, color: "#e0e0e0" }}>{d.name}</span>
                          {d.quality_score !== null && (
                            <span style={{
                              fontSize: 10, fontWeight: 700, padding: "2px 8px", borderRadius: 10,
                              background: d.quality_score >= 0.8 ? "rgba(46,204,113,0.15)" : d.quality_score >= 0.5 ? "rgba(243,156,18,0.15)" : "rgba(231,76,60,0.15)",
                              color: d.quality_score >= 0.8 ? "#2ecc71" : d.quality_score >= 0.5 ? "#f39c12" : "#e74c3c",
                            }}>
                              {Math.round(d.quality_score * 100)}%
                            </span>
                          )}
                        </div>
                        <div style={{ fontSize: 10, color: "#555", marginBottom: 2 }}>
                          {d.kind} · {fmtBytes(d.size_bytes)} · {fmtTime(d.created_at)}
                        </div>
                        <div style={{ fontSize: 11, color: "#888", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                          {d.path}
                        </div>
                        {d.description && (
                          <div style={{ fontSize: 11, color: "#666", marginTop: 4 }}>{d.description}</div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {detailTab === "actions" && (
                <div>
                  {actions.length === 0 && (
                    <div style={{ color: "#555", fontSize: 12, textAlign: "center", padding: 20 }}>
                      No action history yet.
                    </div>
                  )}
                  <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                    {actions.map((a) => (
                      <div key={a.id} style={{ padding: "8px 12px", borderRadius: 6, background: "#111125", border: "1px solid #1a1a30" }}>
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 2 }}>
                          <span style={{ fontSize: 12, fontWeight: 600, color: "#ccc" }}>{a.action_type}</span>
                          <StatusBadge status={a.status} />
                        </div>
                        <div style={{ fontSize: 10, color: "#555" }}>
                          {a.target_path} · {fmtTime(a.started_at)}
                          {a.completed_at && ` → ${fmtTime(a.completed_at)}`}
                        </div>
                        {a.result_summary && (
                          <div style={{ fontSize: 11, color: "#777", marginTop: 4 }}>{a.result_summary}</div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {detailTab === "templates" && (
                <div>
                  <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: 8 }}>
                    {TEMPLATES.map((t) => (
                      <div
                        key={t.id}
                        style={{
                          padding: "12px 14px", borderRadius: 8, background: "#111125",
                          border: "1px solid #1a1a30", display: "flex", flexDirection: "column", gap: 6,
                        }}
                      >
                        <div style={{ fontSize: 12, fontWeight: 700, color: "#e0e0e0" }}>{t.name}</div>
                        <div style={{ fontSize: 10, color: "#666", textTransform: "uppercase", letterSpacing: "0.3px" }}>{t.category}</div>
                        <div style={{ fontSize: 11, color: "#888", lineHeight: 1.4, flex: 1 }}>{t.description}</div>
                        <button
                          style={{ ...btnBase, background: "var(--nt-accent, #6c5ce7)", color: "#fff", width: "100%", fontSize: 11 }}
                          onClick={() => handleApplyTemplate(t.id)}
                        >
                          {actionLoading === `tmpl_${t.id}` ? "Applying..." : "Apply Template"}
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

            </div>
          </>
        )}
      </div>

      {/* new session modal */}
      {showNewForm && (
        <div style={{
          position: "fixed", inset: 0, background: "rgba(0,0,0,0.6)", display: "flex",
          alignItems: "center", justifyContent: "center", zIndex: 1000,
        }} onClick={() => setShowNewForm(false)}>
          <div style={{
            background: "#141428", border: "1px solid #2d2d4a", borderRadius: 12,
            padding: 24, width: 480, maxHeight: "80vh", overflow: "auto",
          }} onClick={(e) => e.stopPropagation()}>
            <div style={{ fontSize: 16, fontWeight: 700, color: "#e0e0e0", marginBottom: 16 }}>
              New Cowork Session
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Workspace Path *</label>
                <input style={tagInputStyle} placeholder="/path/to/project" value={newWorkspace} onChange={(e) => setNewWorkspace(e.target.value)} />
              </div>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Description</label>
                <textarea
                  style={{ ...tagInputStyle, minHeight: 60, resize: "vertical", fontFamily: "inherit" }}
                  placeholder="What should the agent work on?"
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                />
              </div>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Session Name (optional)</label>
                <input style={tagInputStyle} placeholder="My cowork session" value={newName} onChange={(e) => setNewName(e.target.value)} />
              </div>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Tags (comma-separated)</label>
                <input style={tagInputStyle} placeholder="code-review, frontend" value={newTagsStr} onChange={(e) => setNewTagsStr(e.target.value)} />
              </div>
            </div>

            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
              <button style={{ ...btnBase, background: "#2d2d4a", color: "#e0e0e0" }} onClick={() => setShowNewForm(false)}>Cancel</button>
              <button
                style={{ ...btnBase, background: "var(--nt-accent, #6c5ce7)", color: "#fff", opacity: newWorkspace.trim() ? 1 : 0.4 }}
                disabled={!newWorkspace.trim()}
                onClick={handleStart}
              >
                {actionLoading === "start" ? "Starting..." : "Start Session"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* config modal */}
      {showConfig && config && (
        <div style={{
          position: "fixed", inset: 0, background: "rgba(0,0,0,0.6)", display: "flex",
          alignItems: "center", justifyContent: "center", zIndex: 1000,
        }} onClick={() => setShowConfig(false)}>
          <div style={{
            background: "#141428", border: "1px solid #2d2d4a", borderRadius: 12,
            padding: 24, width: 420,
          }} onClick={(e) => e.stopPropagation()}>
            <div style={{ fontSize: 16, fontWeight: 700, color: "#e0e0e0", marginBottom: 16 }}>
              Cowork Configuration
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Max Files Per Scan</label>
                <input
                  style={tagInputStyle}
                  type="number"
                  value={config.max_files_per_scan}
                  onChange={(e) => setConfig({ ...config, max_files_per_scan: parseInt(e.target.value) || 0 })}
                />
              </div>
              <div>
                <label style={{ fontSize: 11, fontWeight: 600, color: "#888", display: "block", marginBottom: 4 }}>Max File Size (KB)</label>
                <input
                  style={tagInputStyle}
                  type="number"
                  value={config.max_file_size_kb}
                  onChange={(e) => setConfig({ ...config, max_file_size_kb: parseInt(e.target.value) || 0 })}
                />
              </div>
              {(["auto_save", "allow_file_create", "allow_file_modify", "allow_file_delete"] as const).map((key) => (
                <div key={key} style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <input
                    type="checkbox"
                    id={key}
                    checked={config[key]}
                    onChange={(e) => setConfig({ ...config, [key]: e.target.checked })}
                    style={{ accentColor: "var(--nt-accent, #6c5ce7)" }}
                  />
                  <label htmlFor={key} style={{ fontSize: 12, color: "#ccc", cursor: "pointer" }}>
                    {key.split("_").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ")}
                  </label>
                </div>
              ))}
            </div>

            <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
              <button style={{ ...btnBase, background: "#2d2d4a", color: "#e0e0e0" }} onClick={() => setShowConfig(false)}>Close</button>
              <button
                style={{ ...btnBase, background: "var(--nt-accent, #6c5ce7)", color: "#fff" }}
                onClick={() => { handleSaveConfig(); setShowConfig(false); }}
              >
                {actionLoading === "config" ? "Saving..." : "Save"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* error toast */}
      {error && (
        <div style={{
          position: "fixed", bottom: 20, left: "50%", transform: "translateX(-50%)",
          background: "#e74c3c", color: "#fff", padding: "8px 16px", borderRadius: 8,
          fontSize: 12, fontWeight: 600, zIndex: 2000, cursor: "pointer",
        }} onClick={() => setError(null)}>
          {error}
        </div>
      )}
    </div>
  );
};

export default CoworkPanel;
