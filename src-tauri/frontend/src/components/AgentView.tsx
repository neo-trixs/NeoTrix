import React, { useState, useEffect, useCallback } from "react";

interface AgentViewSession {
  id: string;
  name: string;
  surface: string;
  status: string;
  current_action: string;
  progress_pct: number;
  started_at: number;
  last_active_at: number;
  cpu_pct: number;
  memory_mb: number;
  tokens_used: number;
  tasks_completed: number;
  error_count: number;
}

interface AgentViewSummary {
  total_sessions: number;
  active_sessions: number;
  waiting_input: number;
  completed_today: number;
  failed_today: number;
  avg_cpu: number;
  avg_memory: number;
}

interface AgentViewConfig {
  enabled: boolean;
  poll_interval_ms: number;
  max_sessions: number;
  show_completed: boolean;
  group_by: string;
}

const SURFACE_ICONS: Record<string, string> = {
  cli: "\uD83D\uDCBB",
  desktop: "\uD83D\uDDA5\uFE0F",
  web: "\uD83C\uDF10",
  mobile: "\uD83D\uDCF1",
  background: "\u2699\uFE0F",
};

const STATUS_ICONS: Record<string, string> = {
  running: "\uD83D\uDFE2",
  idle: "\u26AA",
  waiting_input: "\uD83D\uDFE1",
  completed: "\uD83D\uDFE2",
  failed: "\uD83D\uDD34",
};

const STATUS_LABELS: Record<string, string> = {
  running: "Running",
  idle: "Idle",
  waiting_input: "Waiting",
  completed: "Done",
  failed: "Failed",
};

const SURFACE_LABELS: Record<string, string> = {
  cli: "CLI",
  desktop: "Desktop",
  web: "Web",
  mobile: "Mobile",
  background: "Background",
};

function formatTime(secs: number): string {
  const d = Math.floor((Date.now() / 1000 - secs) / 60);
  if (d < 1) return "just now";
  if (d < 60) return `${d}m ago`;
  const h = Math.floor(d / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

function formatNumber(n: number): string {
  if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return n.toLocaleString();
}

const AgentView: React.FC = () => {
  const [summary, setSummary] = useState<AgentViewSummary | null>(null);
  const [sessions, setSessions] = useState<AgentViewSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [config, setConfig] = useState<AgentViewConfig | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const fetchAll = useCallback(async () => {
    try {
      const mod = await import("../commands");
      const [s, l, c] = await Promise.all([
        mod.agentViewSummary().catch(() => null),
        mod.agentViewList().catch(() => []),
        mod.agentViewConfig().catch(() => null),
      ]);
      if (s) setSummary(s);
      if (l.length > 0) setSessions(l);
      if (c) setConfig(c);
      setError(null);
    } catch {
      setError("Failed to fetch agent view data");
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    fetchAll();
    const timer = setInterval(fetchAll, 5000);
    return () => clearInterval(timer);
  }, [fetchAll]);

  const handleAction = useCallback(async (id: string, action: "pause" | "resume" | "cancel") => {
    setActionLoading(id);
    try {
      const mod = await import("../commands");
      if (action === "pause") await mod.agentViewPause(id);
      else if (action === "resume") await mod.agentViewResume(id);
      else await mod.agentViewCancel(id);
      await fetchAll();
    } catch { }
    setActionLoading(null);
  }, [fetchAll]);

  const handleTick = useCallback(async () => {
    try {
      const mod = await import("../commands");
      await mod.agentViewTick();
      await fetchAll();
    } catch { }
  }, [fetchAll]);

  const groupedSessions = sessions.reduce<Record<string, AgentViewSession[]>>((acc, s) => {
    const key = s.status;
    if (!acc[key]) acc[key] = [];
    acc[key].push(s);
    return acc;
  }, {});

  const statusOrder = ["running", "waiting_input", "idle", "completed", "failed"];

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
      {/* Summary Bar */}
      <div className="lg-glass-strong" style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(100px, 1fr))",
        gap: "var(--nt-gap-sm)",
        padding: "var(--nt-gap-md)",
        borderRadius: "var(--nt-radius-md)",
        flexShrink: 0,
      }}>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{summary?.total_sessions ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Total</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>{summary?.active_sessions ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Active</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-warning)" }}>{summary?.waiting_input ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Waiting</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>{summary?.completed_today ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Completed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-danger)" }}>{summary?.failed_today ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Failed</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{summary?.avg_cpu.toFixed(1) ?? 0}%</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Avg CPU</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{summary ? `${summary.avg_memory.toFixed(0)}MB` : "0MB"}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Avg Memory</div>
        </div>
      </div>

      {/* Toolbar */}
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", flexShrink: 0 }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: "var(--nt-text)" }}>
          Agent Sessions
          <span className="lg-badge" style={{ marginLeft: 8 }}>{sessions.length}</span>
        </div>
        <button className="lg-btn" onClick={handleTick} title="Simulate tick">
          \u25B6 Advance
        </button>
      </div>

      {/* Session Groups */}
      <div className="lg-scrollbar" style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: "var(--nt-gap-md)" }}>
        {statusOrder.map((status) => {
          const group = groupedSessions[status];
          if (!group || group.length === 0) return null;
          if (!config?.show_completed && (status === "completed" || status === "failed")) return null;

          return (
            <div key={status} className="lg-fade-in" style={{ display: "flex", flexDirection: "column", gap: 2 }}>
              <div style={{
                fontSize: 10,
                fontWeight: 600,
                color: "var(--nt-text-secondary)",
                textTransform: "uppercase",
                letterSpacing: 0.5,
                padding: "4px 8px",
              }}>
                {STATUS_ICONS[status] ?? "\u26AA"} {STATUS_LABELS[status] ?? status} ({group.length})
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {group.map((session) => (
                  <div key={session.id} className="lg-glass-hover" style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--nt-gap-sm)",
                    padding: "8px 10px",
                    borderRadius: "var(--nt-radius-sm)",
                    background: "var(--nt-glass-bg)",
                    backdropFilter: "saturate(180%) blur(var(--nt-blur-sm))",
                    border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                    transition: "all var(--nt-transition-fast)",
                  }}>
                    {/* Surface Icon */}
                    <span style={{ fontSize: 16, flexShrink: 0 }} title={SURFACE_LABELS[session.surface] ?? session.surface}>
                      {SURFACE_ICONS[session.surface] ?? "\u2753"}
                    </span>

                    {/* Info */}
                    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2 }}>
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        <span style={{ fontSize: 13, fontWeight: 600, color: "var(--nt-text)" }}>{session.name}</span>
                        <span className={`lg-badge ${session.status === "running" ? "lg-badge-success" : session.status === "failed" ? "lg-badge-danger" : session.status === "waiting_input" ? "lg-badge-warning" : ""}`}
                          style={{ fontSize: 9 }}
                        >
                          {STATUS_ICONS[session.status] ?? "\u26AA"} {STATUS_LABELS[session.status] ?? session.status}
                        </span>
                        <span className="lg-badge" style={{ fontSize: 9 }}>
                          {SURFACE_ICONS[session.surface] ?? ""} {SURFACE_LABELS[session.surface] ?? session.surface}
                        </span>
                      </div>
                      <div style={{ fontSize: 11, color: "var(--nt-text-secondary)" }}>
                        {session.current_action}
                      </div>
                    </div>

                    {/* Progress */}
                    <div style={{ width: 120, display: "flex", flexDirection: "column", gap: 2, flexShrink: 0 }}>
                      <div style={{ height: 4, background: "var(--nt-glass-border)", borderRadius: 2, overflow: "hidden" }}>
                        <div style={{
                          height: "100%",
                          width: `${session.progress_pct}%`,
                          background: session.status === "failed" ? "var(--nt-danger)" :
                            session.status === "completed" ? "var(--nt-success)" : "var(--nt-primary)",
                          borderRadius: 2,
                          transition: "width 0.5s ease",
                        }} />
                      </div>
                      <div style={{ fontSize: 9, color: "var(--nt-text-muted)", textAlign: "right" }}>
                        {session.progress_pct.toFixed(0)}%
                      </div>
                    </div>

                    {/* Metrics */}
                    <div style={{ display: "flex", gap: 10, fontSize: 10, color: "var(--nt-text-muted)", flexShrink: 0 }}>
                      <span title="CPU">{session.cpu_pct.toFixed(0)}%</span>
                      <span title="Memory">{session.memory_mb.toFixed(0)}MB</span>
                      <span title="Tokens">{formatNumber(session.tokens_used)}t</span>
                      <span title="Tasks">{session.tasks_completed}tasks</span>
                      {session.error_count > 0 && (
                        <span style={{ color: "var(--nt-danger)" }} title="Errors">
                          {session.error_count}err
                        </span>
                      )}
                    </div>

                    {/* Last active */}
                    <div style={{ fontSize: 9, color: "var(--nt-text-muted)", flexShrink: 0, minWidth: 50, textAlign: "right" }}>
                      {formatTime(session.last_active_at)}
                    </div>

                    {/* Actions */}
                    <div style={{ display: "flex", gap: 2, flexShrink: 0 }}>
                      {(session.status === "running") && (
                        <button
                          className="lg-btn lg-btn-icon lg-btn-ghost"
                          onClick={() => handleAction(session.id, "pause")}
                          disabled={actionLoading === session.id}
                          title="Pause"
                          style={{ fontSize: 12, padding: "2px 6px" }}
                        >
                          \u23F8\uFE0F
                        </button>
                      )}
                      {(session.status === "idle" || session.status === "waiting_input") && (
                        <button
                          className="lg-btn lg-btn-icon lg-btn-ghost"
                          onClick={() => handleAction(session.id, "resume")}
                          disabled={actionLoading === session.id}
                          title="Resume"
                          style={{ fontSize: 12, padding: "2px 6px" }}
                        >
                          \u25B6
                        </button>
                      )}
                      {(session.status === "running" || session.status === "idle" || session.status === "waiting_input") && (
                        <button
                          className="lg-btn lg-btn-icon lg-btn-ghost"
                          onClick={() => handleAction(session.id, "cancel")}
                          disabled={actionLoading === session.id}
                          title="Cancel"
                          style={{ fontSize: 12, padding: "2px 6px", color: "var(--nt-danger)" }}
                        >
                          \u2716
                        </button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}

        {sessions.length === 0 && (
          <div className="lg-empty">
            <div className="lg-empty-icon">\uD83E\uDD16</div>
            <div className="lg-empty-text">No agent sessions</div>
            <div className="lg-empty-hint">Sessions will appear here when agents are running</div>
          </div>
        )}
      </div>
    </div>
  );
};

AgentView.displayName = "AgentView";

export default AgentView;
