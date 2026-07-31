import React, { useState, useEffect, useCallback } from "react";

const TASK_TYPES = [
  { id: "click", label: "Click" },
  { id: "type", label: "Type" },
  { id: "navigate", label: "Navigate" },
  { id: "capture", label: "Capture" },
  { id: "script", label: "Script" },
  { id: "watch", label: "Watch" },
];

const STATUS_BADGE: Record<string, { label: string; cls: string }> = {
  queued: { label: "Queued", cls: "lg-badge" },
  running: { label: "Running", cls: "lg-badge lg-badge-primary" },
  completed: { label: "Completed", cls: "lg-badge lg-badge-success" },
  failed: { label: "Failed", cls: "lg-badge lg-badge-danger" },
  cancelled: { label: "Cancelled", cls: "lg-badge lg-badge-warning" },
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

function fmtDuration(ms: number | null): string {
  if (ms === null) return "\u2014";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

const BackgroundCUPanel: React.FC = () => {
  const [tasks, setTasks] = useState<any[]>([]);
  const [stats, setStats] = useState<any>(null);
  const [config, setConfigState] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const [showForm, setShowForm] = useState(false);
  const [newType, setNewType] = useState("click");
  const [newTarget, setNewTarget] = useState("");
  const [newParams, setNewParams] = useState("{}");
  const [filterStatus, setFilterStatus] = useState("");

  const [showConfig, setShowConfig] = useState(false);

  const fetchAll = useCallback(async () => {
    try {
      const mod = await import("../commands");
      const [taskList, st, cfg] = await Promise.all([
        mod.computerBgList(filterStatus || undefined).catch(() => []),
        mod.computerBgStats().catch(() => null),
        mod.computerBgConfig().catch(() => null),
      ]);
      setTasks(taskList);
      if (st) setStats(st);
      if (cfg) setConfigState(cfg);
      setError(null);
    } catch {
      setError("Failed to fetch background tasks");
    }
    setLoading(false);
  }, [filterStatus]);

  useEffect(() => {
    fetchAll();
    const timer = setInterval(fetchAll, 5000);
    return () => clearInterval(timer);
  }, [fetchAll]);

  const handleSubmit = useCallback(async () => {
    if (!newTarget.trim()) return;
    setActionLoading("submit");
    try {
      let parsed: Record<string, unknown> = {};
      try { parsed = JSON.parse(newParams); } catch { parsed = {}; }
      const mod = await import("../commands");
      await mod.computerBgSubmit(newType, newTarget.trim(), parsed);
      setShowForm(false);
      setNewTarget("");
      setNewParams("{}");
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [newType, newTarget, newParams, fetchAll]);

  const handleCancel = useCallback(async (taskId: string) => {
    setActionLoading(taskId);
    try {
      const mod = await import("../commands");
      await mod.computerBgCancel(taskId);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleRetry = useCallback(async (taskId: string) => {
    setActionLoading(taskId);
    try {
      const mod = await import("../commands");
      await mod.computerBgRetry(taskId);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleClear = useCallback(async () => {
    setActionLoading("clear");
    try {
      const mod = await import("../commands");
      await mod.computerBgClear(false);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleConfigSave = useCallback(async () => {
    if (!config) return;
    setActionLoading("config");
    try {
      const mod = await import("../commands");
      await mod.computerBgSetConfig({
        enabled: config.enabled,
        max_concurrent_tasks: config.max_concurrent_tasks,
        poll_interval_ms: config.poll_interval_ms,
        auto_retry: config.auto_retry,
        max_retries: config.max_retries,
      });
      setShowConfig(false);
    } catch { }
    setActionLoading(null);
  }, [config]);

  if (loading) {
    return (
      <div style={{ height: "100%", padding: "var(--nt-gap-sm)", display: "flex", flexDirection: "column", gap: "var(--nt-gap-md)" }}>
        <div className="lg-skeleton" style={{ height: 80 }} />
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
      {/* Stats Bar */}
      <div className="lg-glass-strong" style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(80px, 1fr))",
        gap: "var(--nt-gap-sm)",
        padding: "var(--nt-gap-md)",
        borderRadius: "var(--nt-radius-md)",
        flexShrink: 0,
      }}>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{stats?.total_created ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Created</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>{stats?.total_completed ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Completed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: stats && stats.total_failed > 0 ? "var(--nt-danger)" : "var(--nt-text)" }}>
            {stats?.total_failed ?? 0}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Failed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-primary)" }}>{stats?.currently_running ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Running</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>
            {stats ? `${(stats.success_rate * 100).toFixed(0)}%` : "\u2014"}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Success</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", whiteSpace: "nowrap" }}>
            {stats ? fmtDuration(stats.avg_duration_ms) : "\u2014"}
          </div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Avg Dur</div>
        </div>
      </div>

      {/* Toolbar */}
      <div style={{ display: "flex", alignItems: "center", gap: "var(--nt-gap-sm)", flexShrink: 0 }}>
        <select
          value={filterStatus}
          onChange={(e) => setFilterStatus(e.target.value)}
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
          <option value="">All statuses</option>
          <option value="queued">Queued</option>
          <option value="running">Running</option>
          <option value="completed">Completed</option>
          <option value="failed">Failed</option>
          <option value="cancelled">Cancelled</option>
        </select>
        <button
          className="lg-btn"
          onClick={() => setShowForm(true)}
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
          + New Task
        </button>
        <button
          className="lg-btn lg-btn-ghost"
          onClick={handleClear}
          disabled={actionLoading === "clear"}
          style={{ fontSize: 12, padding: "6px 10px", marginLeft: "auto" }}
        >
          Clear Completed
        </button>
        <button
          className="lg-btn lg-btn-ghost"
          onClick={() => setShowConfig(!showConfig)}
          style={{ fontSize: 12, padding: "6px 10px" }}
        >
          {'\u2699\uFE0F'}
        </button>
      </div>

      {/* Config Panel */}
      {showConfig && config && (
        <div className="lg-glass-strong" style={{
          padding: "var(--nt-gap-md)",
          borderRadius: "var(--nt-radius-md)",
          display: "flex",
          flexDirection: "column",
          gap: "var(--nt-gap-sm)",
          flexShrink: 0,
        }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>Configuration</div>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--nt-gap-sm)" }}>
            <label style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Enabled</span>
              <input type="checkbox" checked={config.enabled} onChange={(e) => setConfigState({ ...config, enabled: e.target.checked })} />
            </label>
            <label style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Max Concurrent</span>
              <input
                type="number" min={1} max={50}
                value={config.max_concurrent_tasks}
                onChange={(e) => setConfigState({ ...config, max_concurrent_tasks: parseInt(e.target.value) || 1 })}
                className="lg-input"
                style={{ padding: "4px 8px", fontSize: 12 }}
              />
            </label>
            <label style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Poll Interval (ms)</span>
              <input
                type="number" min={100} step={100}
                value={config.poll_interval_ms}
                onChange={(e) => setConfigState({ ...config, poll_interval_ms: parseInt(e.target.value) || 1000 })}
                className="lg-input"
                style={{ padding: "4px 8px", fontSize: 12 }}
              />
            </label>
            <label style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Auto Retry</span>
              <input type="checkbox" checked={config.auto_retry} onChange={(e) => setConfigState({ ...config, auto_retry: e.target.checked })} />
            </label>
            <label style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <span style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Max Retries</span>
              <input
                type="number" min={0} max={20}
                value={config.max_retries}
                onChange={(e) => setConfigState({ ...config, max_retries: parseInt(e.target.value) || 0 })}
                className="lg-input"
                style={{ padding: "4px 8px", fontSize: 12 }}
              />
            </label>
          </div>
          <div style={{ display: "flex", gap: "var(--nt-gap-sm)", justifyContent: "flex-end" }}>
            <button className="lg-btn" onClick={() => setShowConfig(false)} style={{ fontSize: 12 }}>Cancel</button>
            <button className="lg-btn" onClick={handleConfigSave} disabled={actionLoading === "config"} style={{ fontSize: 12, fontWeight: 600 }}>
              {actionLoading === "config" ? "Saving..." : "Save"}
            </button>
          </div>
        </div>
      )}

      {/* Task List */}
      <div className="lg-scrollbar" style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: "var(--nt-gap-sm)" }}>
        {/* Create Form */}
        {showForm && (
          <div className="lg-glass-strong" style={{
            padding: "var(--nt-gap-md)",
            borderRadius: "var(--nt-radius-md)",
            display: "flex",
            flexDirection: "column",
            gap: "var(--nt-gap-sm)",
            flexShrink: 0,
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "var(--nt-text)", marginBottom: 4 }}>New Background Task</div>
            <div style={{ display: "flex", gap: "var(--nt-gap-sm)" }}>
              <select
                value={newType}
                onChange={(e) => setNewType(e.target.value)}
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
              <input
                className="lg-input"
                placeholder="Target (URL, selector, script name...)"
                value={newTarget}
                onChange={(e) => setNewTarget(e.target.value)}
                style={{ flex: 1, padding: "6px 10px", fontSize: 12 }}
              />
            </div>
            <textarea
              className="lg-input"
              placeholder='Params JSON (optional, e.g. {"key": "value"})'
              value={newParams}
              onChange={(e) => setNewParams(e.target.value)}
              rows={3}
              style={{ padding: "6px 10px", fontSize: 12, fontFamily: "monospace", resize: "vertical" }}
            />
            <div style={{ display: "flex", gap: "var(--nt-gap-sm)", justifyContent: "flex-end" }}>
              <button className="lg-btn" onClick={() => setShowForm(false)} style={{ fontSize: 12 }}>Cancel</button>
              <button
                className="lg-btn"
                onClick={handleSubmit}
                disabled={actionLoading === "submit" || !newTarget.trim()}
                style={{ fontSize: 12, fontWeight: 600 }}
              >
                {actionLoading === "submit" ? "Submitting..." : "Submit"}
              </button>
            </div>
          </div>
        )}

        {tasks.length === 0 && !showForm && (
          <div className="lg-empty">
            <div className="lg-empty-icon">{'\uD83E\uDD16'}</div>
            <div className="lg-empty-text">No background tasks</div>
            <div className="lg-empty-hint">Submit a click, type, navigate, capture, script, or watch task</div>
          </div>
        )}

        {tasks.map((task) => {
          const badge = STATUS_BADGE[task.status] ?? { label: task.status, cls: "lg-badge" };
          return (
            <div key={task.id} className="lg-fade-in" style={{
              display: "flex",
              flexDirection: "column",
              gap: 2,
            }}>
              <div className="lg-glass-hover" style={{
                display: "flex",
                flexDirection: "column",
                gap: 4,
                padding: "10px 12px",
                borderRadius: "var(--nt-radius-sm)",
                background: "var(--nt-glass-bg)",
                backdropFilter: "saturate(180%) blur(var(--nt-blur-sm))",
                border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                transition: "all var(--nt-transition-fast)",
              }}>
                {/* Row 1: Status + Type + Target + Actions */}
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span className={badge.cls} style={{ fontSize: 9 }}>{badge.label}</span>
                  <span style={{ fontSize: 11, fontWeight: 600, color: "var(--nt-text)", fontFamily: "monospace" }}>{task.task_type}</span>
                  <span style={{ fontSize: 12, color: "var(--nt-text-secondary)", flex: 1, minWidth: 0, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    {task.target}
                  </span>
                  <span style={{ fontSize: 10, color: "var(--nt-text-muted)" }}>{fmtTime(task.created_at)}</span>
                  <div style={{ display: "flex", gap: 2, flexShrink: 0 }}>
                    {task.status === "running" && (
                      <button
                        className="lg-btn lg-btn-icon lg-btn-ghost"
                        onClick={() => handleCancel(task.id)}
                        disabled={actionLoading === task.id}
                        title="Cancel"
                        style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-warning)" }}
                      >
                        {'\u23F9\uFE0F'}
                      </button>
                    )}
                    {task.status === "failed" && (
                      <button
                        className="lg-btn lg-btn-icon lg-btn-ghost"
                        onClick={() => handleRetry(task.id)}
                        disabled={actionLoading === task.id}
                        title="Retry"
                        style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-primary)" }}
                      >
                        {'\uD83D\uDD04'}
                      </button>
                    )}
                    {(task.status === "queued" || task.status === "cancelled") && (
                      <button
                        className="lg-btn lg-btn-icon lg-btn-ghost"
                        onClick={() => handleCancel(task.id)}
                        disabled={actionLoading === task.id}
                        title="Cancel"
                        style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-text-muted)" }}
                      >
                        {'\u2716'}
                      </button>
                    )}
                  </div>
                </div>
                {/* Progress bar */}
                {(task.status === "running" || task.status === "queued") && (
                  <div style={{ width: "100%", height: 4, background: "var(--nt-glass-border)", borderRadius: 2, overflow: "hidden" }}>
                    <div style={{
                      width: `${task.progress_pct ?? 0}%`,
                      height: "100%",
                      background: task.status === "running" ? "var(--nt-primary)" : "var(--nt-text-muted)",
                      borderRadius: 2,
                      transition: "width 0.5s ease",
                    }} />
                  </div>
                )}
                {/* Duration + Error */}
                <div style={{ display: "flex", alignItems: "center", gap: 8, fontSize: 10, color: "var(--nt-text-muted)" }}>
                  {task.duration_ms !== null && <span>Duration: {fmtDuration(task.duration_ms)}</span>}
                  {task.retry_count > 0 && <span>Retries: {task.retry_count}</span>}
                  {task.error && (
                    <span style={{ color: "var(--nt-danger)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 300 }}>
                      {'\u26A0'} {task.error}
                    </span>
                  )}
                </div>
                {/* Result */}
                {task.result && task.status === "completed" && (
                  <div style={{ fontSize: 11, color: "var(--nt-text-secondary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                    Result: {task.result}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

BackgroundCUPanel.displayName = "BackgroundCUPanel";

export default BackgroundCUPanel;
