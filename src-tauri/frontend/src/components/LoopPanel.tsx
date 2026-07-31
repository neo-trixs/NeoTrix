import React, { useState, useEffect, useCallback } from "react";
import type { LoopSchedule, LoopExecution, LoopStats } from "../commands";

const CRON_PRESETS: Record<string, string> = {
  every_1h: "0 * * * *",
  every_6h: "0 */6 * * *",
  every_24h: "0 0 * * *",
  every_monday: "0 0 * * 1",
  every_weekday: "0 9 * * 1-5",
};

const CRON_LABELS: Record<string, string> = {
  "0 * * * *": "Every hour",
  "0 */6 * * *": "Every 6 hours",
  "0 0 * * *": "Every 24 hours",
  "0 0 * * 1": "Every Monday",
  "0 9 * * 1-5": "Weekdays 9 AM",
};

const TASK_TYPES = [
  { id: "pr_inspection", label: "PR Inspection" },
  { id: "deploy_monitor", label: "Deploy Monitor" },
  { id: "code_scan", label: "Code Scan" },
  { id: "reminder", label: "Reminder" },
  { id: "custom", label: "Custom" },
];

const STATUS_ICONS: Record<string, string> = {
  running: "\u25B6\uFE0F",
  completed: "\u2705",
  failed: "\u274C",
};

function fmtTime(iso: string | null): string {
  if (!iso) return "\u2014";
  const d = new Date(iso);
  const now = Date.now();
  const diff = now - d.getTime();
  if (diff < 60000) return "just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString();
}

function fmtDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

const LoopPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<"schedules" | "history" | "stats">("schedules");
  const [schedules, setSchedules] = useState<LoopSchedule[]>([]);
  const [stats, setStats] = useState<LoopStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDesc, setNewDesc] = useState("");
  const [newCron, setNewCron] = useState("0 * * * *");
  const [newTaskType, setNewTaskType] = useState("pr_inspection");

  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [historyMap, setHistoryMap] = useState<Record<string, LoopExecution[]>>({});
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const fetchAll = useCallback(async () => {
    try {
      const mod = await import("../commands");
      const [sched, st] = await Promise.all([
        mod.loopList().catch(() => []),
        mod.loopStats().catch(() => null),
      ]);
      setSchedules(sched);
      if (st) setStats(st);
      setError(null);
    } catch {
      setError("Failed to fetch loop data");
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    fetchAll();
    const timer = setInterval(fetchAll, 10000);
    return () => clearInterval(timer);
  }, [fetchAll]);

  const fetchHistory = useCallback(async (id: string) => {
    try {
      const mod = await import("../commands");
      const history = await mod.loopExecutionHistory(id, 10);
      setHistoryMap((prev) => ({ ...prev, [id]: history }));
    } catch { }
  }, []);

  const toggleExpand = useCallback((id: string) => {
    if (expandedId === id) {
      setExpandedId(null);
    } else {
      setExpandedId(id);
      if (!historyMap[id]) fetchHistory(id);
    }
  }, [expandedId, historyMap, fetchHistory]);

  const handleCreate = useCallback(async () => {
    if (!newName.trim() || !newCron.trim()) return;
    setActionLoading("create");
    try {
      const mod = await import("../commands");
      await mod.loopCreate(newName.trim(), newDesc.trim(), newCron.trim(), newTaskType);
      setShowCreateForm(false);
      setNewName("");
      setNewDesc("");
      setNewCron("0 * * * *");
      setNewTaskType("pr_inspection");
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [newName, newDesc, newCron, newTaskType, fetchAll]);

  const handleToggleEnabled = useCallback(async (sched: LoopSchedule) => {
    setActionLoading(sched.id);
    try {
      const mod = await import("../commands");
      if (sched.enabled) await mod.loopDisable(sched.id);
      else await mod.loopEnable(sched.id);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleRunNow = useCallback(async (id: string) => {
    setActionLoading(id);
    try {
      const mod = await import("../commands");
      await mod.loopExecuteNow(id);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleDelete = useCallback(async (id: string) => {
    setActionLoading(id);
    try {
      const mod = await import("../commands");
      await mod.loopDelete(id);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const successRate = (s: LoopSchedule): string => {
    const total = s.run_count;
    if (total === 0) return "\u2014";
    return `${((s.success_count / total) * 100).toFixed(0)}%`;
  };

  if (loading) {
    return (
      <div className="lg-flex-col" style={{ height: "100%", padding: "var(--nt-gap-sm)", gap: "var(--nt-gap-md)" }}>
        <div className="lg-skeleton" style={{ height: 80 }} />
        <div className="lg-skeleton" style={{ flex: 1 }} />
      </div>
    );
  }

  if (error) {
    return (
      <div className="lg-empty">
        <div className="lg-empty-icon">\u26A0\uFE0F</div>
        <div className="lg-empty-text">{error}</div>
        <button className="lg-btn" onClick={fetchAll}>Retry</button>
      </div>
    );
  }

  return (
    <div className="lg-flex-col" style={{ height: "100%", padding: "var(--nt-gap-sm)", gap: "var(--nt-gap-md)", overflow: "hidden" }}>
      {/* Stats Bar */}
      <div className="lg-glass-strong" style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(90px, 1fr))",
        gap: "var(--nt-gap-sm)",
        padding: "var(--nt-gap-md)",
        borderRadius: "var(--nt-radius-md)",
        flexShrink: 0,
      }}>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{stats?.total_schedules ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Total</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>{stats?.active_schedules ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Active</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{stats?.executed_today ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Executed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: stats && stats.failed_today > 0 ? "var(--nt-danger)" : "var(--nt-success)" }}>
            {stats?.failed_today ?? 0}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Failed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>
            {stats ? `${(stats.success_rate * 100).toFixed(0)}%` : "\u2014"}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Success</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", whiteSpace: "nowrap" }}>
            {stats?.next_scheduled_run ? fmtTime(stats.next_scheduled_run) : "\u2014"}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Next Run</div>
        </div>
      </div>

      {/* Tabs */}
      <div style={{ display: "flex", alignItems: "center", gap: "var(--nt-gap-sm)", flexShrink: 0 }}>
        {(["schedules", "history", "stats"] as const).map((tab) => (
          <button
            key={tab}
            className="lg-btn"
            onClick={() => setActiveTab(tab)}
            style={{
              flex: 1,
              fontWeight: activeTab === tab ? 700 : 400,
              color: activeTab === tab ? "var(--nt-primary)" : "var(--nt-text-secondary)",
              background: activeTab === tab ? "var(--nt-glass-bg)" : "transparent",
              border: "var(--nt-edge-width) solid var(--nt-glass-border)",
              borderRadius: "var(--nt-radius-sm)",
              padding: "6px 12px",
              fontSize: 12,
              textTransform: "capitalize",
              cursor: "pointer",
            }}
          >
            {tab}
          </button>
        ))}
        {activeTab === "schedules" && (
          <button
            className="lg-btn"
            onClick={() => setShowCreateForm(true)}
            style={{
              padding: "6px 14px",
              fontSize: 12,
              fontWeight: 600,
              background: "var(--nt-primary)",
              color: "#fff",
              border: "none",
              borderRadius: "var(--nt-radius-sm)",
              cursor: "pointer",
              whiteSpace: "nowrap",
            }}
          >
            + New Schedule
          </button>
        )}
      </div>

      {/* Tab Content */}
      <div className="lg-scrollbar" style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: "var(--nt-gap-md)" }}>
        {/* ── Schedules Tab ── */}
        {activeTab === "schedules" && (
          <>
            {/* Create Form */}
            {showCreateForm && (
              <div className="lg-glass-strong" style={{
                padding: "var(--nt-gap-md)",
                borderRadius: "var(--nt-radius-md)",
                display: "flex",
                flexDirection: "column",
                gap: "var(--nt-gap-sm)",
                flexShrink: 0,
              }}>
                <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>New Schedule</div>
                <input
                  className="lg-input"
                  placeholder="Name"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  style={{
                    padding: "6px 10px",
                    borderRadius: "var(--nt-radius-sm)",
                    border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                    background: "var(--nt-glass-bg)",
                    color: "var(--nt-text)",
                    fontSize: 12,
                    outline: "none",
                  }}
                />
                <input
                  className="lg-input"
                  placeholder="Description (optional)"
                  value={newDesc}
                  onChange={(e) => setNewDesc(e.target.value)}
                  style={{
                    padding: "6px 10px",
                    borderRadius: "var(--nt-radius-sm)",
                    border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                    background: "var(--nt-glass-bg)",
                    color: "var(--nt-text)",
                    fontSize: 12,
                    outline: "none",
                  }}
                />
                <div style={{ display: "flex", gap: "var(--nt-gap-sm)", alignItems: "center" }}>
                  <select
                    value={newCron}
                    onChange={(e) => setNewCron(e.target.value)}
                    style={{
                      flex: 1,
                      padding: "6px 10px",
                      borderRadius: "var(--nt-radius-sm)",
                      border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                      background: "var(--nt-glass-bg)",
                      color: "var(--nt-text)",
                      fontSize: 12,
                      outline: "none",
                    }}
                  >
                    {Object.entries(CRON_PRESETS).map(([key, expr]) => (
                      <option key={key} value={expr}>{CRON_LABELS[expr] ?? expr}</option>
                    ))}
                  </select>
                  <input
                    placeholder="Custom cron"
                    value={Object.values(CRON_PRESETS).includes(newCron) ? "" : newCron}
                    onChange={(e) => setNewCron(e.target.value)}
                    style={{
                      flex: 1,
                      padding: "6px 10px",
                      borderRadius: "var(--nt-radius-sm)",
                      border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                      background: "var(--nt-glass-bg)",
                      color: "var(--nt-text)",
                      fontSize: 12,
                      outline: "none",
                      fontFamily: "monospace",
                    }}
                  />
                </div>
                <select
                  value={newTaskType}
                  onChange={(e) => setNewTaskType(e.target.value)}
                  style={{
                    padding: "6px 10px",
                    borderRadius: "var(--nt-radius-sm)",
                    border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                    background: "var(--nt-glass-bg)",
                    color: "var(--nt-text)",
                    fontSize: 12,
                    outline: "none",
                  }}
                >
                  {TASK_TYPES.map((t) => (
                    <option key={t.id} value={t.id}>{t.label}</option>
                  ))}
                </select>
                <div style={{ display: "flex", gap: "var(--nt-gap-sm)", justifyContent: "flex-end" }}>
                  <button className="lg-btn" onClick={() => setShowCreateForm(false)} style={{ fontSize: 12 }}>Cancel</button>
                  <button
                    className="lg-btn"
                    onClick={handleCreate}
                    disabled={actionLoading === "create" || !newName.trim()}
                    style={{ fontSize: 12, fontWeight: 600 }}
                  >
                    {actionLoading === "create" ? "Creating..." : "Create"}
                  </button>
                </div>
              </div>
            )}

            {/* Schedule List */}
            {schedules.length === 0 && !showCreateForm && (
              <div className="lg-empty">
                <div className="lg-empty-icon">\uD83D\uDD52</div>
                <div className="lg-empty-text">No schedules</div>
                <div className="lg-empty-hint">Create a cron schedule to automate recurring tasks</div>
              </div>
            )}

            {schedules.map((sched) => (
              <div key={sched.id} className="lg-fade-in" style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <div className="lg-glass-hover" style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "var(--nt-gap-sm)",
                  padding: "10px 12px",
                  borderRadius: "var(--nt-radius-sm)",
                  background: "var(--nt-glass-bg)",
                  backdropFilter: "saturate(180%) blur(var(--nt-blur-sm))",
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  cursor: "pointer",
                  transition: "all var(--nt-transition-fast)",
                }} onClick={() => toggleExpand(sched.id)}>
                  {/* Enabled indicator */}
                  <span style={{
                    fontSize: 16,
                    flexShrink: 0,
                    opacity: sched.enabled ? 1 : 0.3,
                    filter: sched.enabled ? "none" : "grayscale(1)",
                  }}
                    title={sched.enabled ? "Enabled" : "Disabled"}
                  >
                    {sched.enabled ? "\uD83D\uDD14" : "\uD83D\uDD15"}
                  </span>

                  {/* Info */}
                  <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                      <span style={{ fontSize: 13, fontWeight: 600, color: "var(--nt-text)" }}>{sched.name}</span>
                      <span className="lg-badge" style={{ fontSize: 9 }}>{sched.task_type}</span>
                      {sched.enabled ? (
                        <span className="lg-badge lg-badge-success" style={{ fontSize: 9 }}>Enabled</span>
                      ) : (
                        <span className="lg-badge lg-badge-danger" style={{ fontSize: 9 }}>Disabled</span>
                      )}
                    </div>
                    <div style={{ fontSize: 11, color: "var(--nt-text-secondary)", display: "flex", gap: 12 }}>
                      <span>{sched.description || "\u2014"}</span>
                    </div>
                    <div style={{ fontSize: 10, color: "var(--nt-text-muted)", display: "flex", gap: 12 }}>
                      <span style={{ fontFamily: "monospace" }}>{sched.cron_expr}</span>
                      <span>{CRON_LABELS[sched.cron_expr] ?? ""}</span>
                      <span>Next: {fmtTime(sched.next_run_at)}</span>
                      <span>{sched.run_count} runs</span>
                      <span>Success: {successRate(sched)}</span>
                    </div>
                  </div>

                  {/* Actions */}
                  <div style={{ display: "flex", gap: 2, flexShrink: 0 }} onClick={(e) => e.stopPropagation()}>
                    <button
                      className="lg-btn lg-btn-icon lg-btn-ghost"
                      onClick={() => handleRunNow(sched.id)}
                      disabled={actionLoading === sched.id}
                      title="Run Now"
                      style={{ fontSize: 12, padding: "2px 6px" }}
                    >
                      \u25B6\uFE0F
                    </button>
                    <button
                      className="lg-btn lg-btn-icon lg-btn-ghost"
                      onClick={() => handleToggleEnabled(sched)}
                      disabled={actionLoading === sched.id}
                      title={sched.enabled ? "Disable" : "Enable"}
                      style={{ fontSize: 12, padding: "2px 6px" }}
                    >
                      {sched.enabled ? "\u23F8\uFE0F" : "\u25B6"}
                    </button>
                    <button
                      className="lg-btn lg-btn-icon lg-btn-ghost"
                      onClick={() => handleDelete(sched.id)}
                      disabled={actionLoading === sched.id}
                      title="Delete"
                      style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-danger)" }}
                    >
                      \u2716
                    </button>
                  </div>
                </div>

                {/* Execution history */}
                {expandedId === sched.id && (
                  <div style={{
                    marginLeft: 24,
                    padding: "8px 10px",
                    borderRadius: "var(--nt-radius-sm)",
                    background: "var(--nt-glass-bg)",
                    border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                    display: "flex",
                    flexDirection: "column",
                    gap: 4,
                  }}>
                    <div style={{ fontSize: 10, fontWeight: 600, color: "var(--nt-text-secondary)", textTransform: "uppercase", letterSpacing: 0.5 }}>
                      Recent Runs
                    </div>
                    {(!historyMap[sched.id] || historyMap[sched.id].length === 0) && (
                      <div style={{ fontSize: 11, color: "var(--nt-text-muted)", padding: "4px 0" }}>
                        No execution history yet
                      </div>
                    )}
                    {historyMap[sched.id]?.map((exec) => (
                      <div key={exec.id} style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 8,
                        padding: "4px 6px",
                        borderRadius: "var(--nt-radius-sm)",
                        fontSize: 11,
                        color: "var(--nt-text)",
                      }}>
                        <span title={exec.status}>
                          {STATUS_ICONS[exec.status] ?? "\u2753"}
                        </span>
                        <span style={{ flex: 1, color: "var(--nt-text-secondary)" }}>{exec.result_summary || "\u2014"}</span>
                        <span style={{ color: "var(--nt-text-muted)", fontFamily: "monospace" }}>{fmtDuration(exec.duration_ms)}</span>
                        <span style={{ color: "var(--nt-text-muted)" }}>{fmtTime(exec.started_at)}</span>
                        {exec.error && (
                          <span style={{ color: "var(--nt-danger)", fontSize: 10, maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                            {exec.error}
                          </span>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}
          </>
        )}

        {/* ── History Tab ── */}
        {activeTab === "history" && (
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            {schedules.length === 0 ? (
              <div className="lg-empty">
                <div className="lg-empty-text">No schedules yet</div>
              </div>
            ) : (
              schedules.map((sched) => (
                <div key={sched.id} className="lg-glass-hover" style={{
                  padding: "8px 10px",
                  borderRadius: "var(--nt-radius-sm)",
                  background: "var(--nt-glass-bg)",
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  display: "flex",
                  alignItems: "center",
                  gap: "var(--nt-gap-sm)",
                }}>
                  <span style={{ fontSize: 13, fontWeight: 600, color: "var(--nt-text)", minWidth: 120 }}>{sched.name}</span>
                  <span style={{ fontSize: 11, color: "var(--nt-text-secondary)", flex: 1 }}>
                    {sched.run_count} runs ({sched.success_count} ok, {sched.fail_count} fail)
                  </span>
                  <button
                    className="lg-btn lg-btn-icon lg-btn-ghost"
                    onClick={() => { setActiveTab("schedules"); toggleExpand(sched.id); }}
                    style={{ fontSize: 11, padding: "2px 6px" }}
                  >
                    View
                  </button>
                </div>
              ))
            )}
          </div>
        )}

        {/* ── Stats Tab ── */}
        {activeTab === "stats" && (
          <div className="lg-glass-strong" style={{
            padding: "var(--nt-gap-md)",
            borderRadius: "var(--nt-radius-md)",
            display: "flex",
            flexDirection: "column",
            gap: "var(--nt-gap-sm)",
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>Loop Statistics</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--nt-gap-sm)" }}>
              {[
                { label: "Total Schedules", value: stats?.total_schedules ?? 0 },
                { label: "Active Schedules", value: stats?.active_schedules ?? 0 },
                { label: "Executed Today", value: stats?.executed_today ?? 0 },
                { label: "Failed Today", value: stats?.failed_today ?? 0 },
                { label: "Avg Duration", value: stats ? fmtDuration(stats.avg_duration_ms) : "\u2014" },
                { label: "Success Rate", value: stats ? `${(stats.success_rate * 100).toFixed(1)}%` : "\u2014" },
                { label: "Next Scheduled", value: stats?.next_scheduled_run ? new Date(stats.next_scheduled_run).toLocaleString() : "\u2014" },
              ].map((row) => (
                <div key={row.label} style={{
                  display: "flex",
                  justifyContent: "space-between",
                  padding: "6px 8px",
                  borderRadius: "var(--nt-radius-sm)",
                  background: "var(--nt-glass-bg)",
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  fontSize: 12,
                }}>
                  <span style={{ color: "var(--nt-text-secondary)" }}>{row.label}</span>
                  <span style={{ color: "var(--nt-text)", fontWeight: 600 }}>{row.value}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

LoopPanel.displayName = "LoopPanel";

export default LoopPanel;
