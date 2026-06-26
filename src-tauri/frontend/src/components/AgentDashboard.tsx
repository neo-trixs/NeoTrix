import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";

interface AgentStatus {
  running: boolean;
  current_task: string | null;
  uptime_secs: number;
}

interface AgentTask {
  id: string;
  prompt: string;
  status: "started" | "stopped" | "completed";
  timestamp: number;
  duration_secs: number;
}

const LS_KEY = "neotrix_agent_tasks";

function loadTasks(): AgentTask[] {
  try {
    const raw = localStorage.getItem(LS_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveTasks(tasks: AgentTask[]) {
  try {
    localStorage.setItem(LS_KEY, JSON.stringify(tasks.slice(0, 50)));
  } catch {}
}

function formatDuration(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
  return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
}

const AgentDashboard: React.FC = () => {
  const setAgentDashboardVisible = useStore((s) => s.setAgentDashboardVisible);

  const [status, setStatus] = useState<AgentStatus>({
    running: false,
    current_task: null,
    uptime_secs: 0,
  });
  const [prompt, setPrompt] = useState("");
  const [starting, setStarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [tasks, setTasks] = useState<AgentTask[]>(loadTasks);
  const [statusLoading, setStatusLoading] = useState(true);

  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<AgentStatus>("cmd_agent_status");
      setStatus(s);
      setStatusLoading(false);
      return s;
    } catch {
      setStatusLoading(false);
      return null;
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleStart = async () => {
    if (!prompt.trim() || starting) return;
    setStarting(true);
    try {
      await invoke("cmd_agent_start", { prompt: prompt.trim() });
      const task: AgentTask = {
        id: `task-${Date.now()}`,
        prompt: prompt.trim(),
        status: "started",
        timestamp: Date.now(),
        duration_secs: 0,
      };
      const updated = [task, ...tasks];
      setTasks(updated);
      saveTasks(updated);
      setPrompt("");
      await fetchStatus();
    } catch (e) {
      console.error("Failed to start agent:", e);
    }
    setStarting(false);
  };

  const handleStop = async () => {
    if (stopping) return;
    setStopping(true);
    try {
      await invoke("cmd_agent_stop");
      const updated = tasks.map((t) =>
        t.id === tasks[0]?.id && t.status === "started"
          ? { ...t, status: "stopped" as const, duration_secs: status.uptime_secs }
          : t
      );
      setTasks(updated);
      saveTasks(updated);
      await fetchStatus();
    } catch (e) {
      console.error("Failed to stop agent:", e);
    }
    setStopping(false);
  };

  const handleRerun = (task: AgentTask) => {
    setPrompt(task.prompt);
  };

  const recentTasks = tasks.slice(0, 10);

  return (
    <div className="evolution-panel" style={{ zIndex: 1000 }}>
      <div className="evolution-panel-toolbar">
        <div className="evolution-panel-toolbar-left">
          <h2>Agent Dashboard</h2>
        </div>
        <div className="evolution-panel-toolbar-right">
          <button className="btn-icon" onClick={() => setAgentDashboardVisible(false)} title="Close">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        </div>
      </div>

      <div className="evolution-panel-body">
        <div className="evo-columns">
          <div className="evo-column">
            {/* --- Status Card --- */}
            <div className="evo-card" style={{ borderTopColor: status.running ? "#34c759" : "#aeaeb2" }}>
              <div className="evo-card-title">Agent Status</div>
              <div className="evo-card-body">
                {statusLoading ? (
                  <div className="evo-stat-row">
                    <span className="evo-stat-label">Loading...</span>
                  </div>
                ) : (
                  <>
                    <div className="evo-stat-row">
                      <span className="evo-stat-label">Status</span>
                      <span className="evo-stat-value" style={{ color: status.running ? "#34c759" : "#aeaeb2" }}>
                        <span
                          style={{
                            display: "inline-block",
                            width: 8,
                            height: 8,
                            borderRadius: "50%",
                            marginRight: 6,
                            backgroundColor: status.running ? "#34c759" : "#aeaeb2",
                            animation: status.running ? "none" : undefined,
                            boxShadow: status.running ? "0 0 8px rgba(52,199,89,0.6)" : undefined,
                          }}
                        />
                        {status.running ? "Running" : "Idle"}
                      </span>
                    </div>
                    <div className="evo-stat-row">
                      <span className="evo-stat-label">Current Task</span>
                      <span className="evo-stat-value" style={{ maxWidth: 240, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {status.current_task || "—"}
                      </span>
                    </div>
                    <div className="evo-stat-row">
                      <span className="evo-stat-label">Uptime</span>
                      <span className="evo-stat-value">{formatDuration(status.uptime_secs)}</span>
                    </div>
                  </>
                )}
              </div>
            </div>

            {/* --- Start/Stop Controls --- */}
            <div className="evo-card" style={{ borderTopColor: "#0a84ff" }}>
              <div className="evo-card-title">Controls</div>
              <div className="evo-card-body">
                <textarea
                  className="evo-input"
                  placeholder="Enter agent task prompt..."
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  rows={3}
                  style={{
                    width: "100%",
                    padding: "8px 10px",
                    borderRadius: 6,
                    border: "1px solid var(--border-color, rgba(128,128,128,0.2))",
                    background: "var(--bg-secondary, rgba(128,128,128,0.05))",
                    color: "inherit",
                    fontSize: 13,
                    resize: "vertical",
                    fontFamily: "inherit",
                    marginBottom: 10,
                    boxSizing: "border-box",
                  }}
                />
                <div style={{ display: "flex", gap: 8 }}>
                  <button
                    className="btn-primary"
                    onClick={handleStart}
                    disabled={!prompt.trim() || starting || status.running}
                    style={{
                      flex: 1,
                      padding: "10px 0",
                      fontSize: 14,
                      fontWeight: 600,
                      borderRadius: 8,
                      border: "none",
                      cursor: !prompt.trim() || starting || status.running ? "not-allowed" : "pointer",
                      opacity: !prompt.trim() || starting || status.running ? 0.5 : 1,
                      background: "#0a84ff",
                      color: "#fff",
                    }}
                  >
                    {starting ? "Starting..." : "Start Agent"}
                  </button>
                  <button
                    className="btn-danger"
                    onClick={handleStop}
                    disabled={!status.running || stopping}
                    style={{
                      flex: 1,
                      padding: "10px 0",
                      fontSize: 14,
                      fontWeight: 600,
                      borderRadius: 8,
                      border: "none",
                      cursor: !status.running || stopping ? "not-allowed" : "pointer",
                      opacity: !status.running || stopping ? 0.5 : 1,
                      background: "#ff3b30",
                      color: "#fff",
                    }}
                  >
                    {stopping ? "Stopping..." : "Stop Agent"}
                  </button>
                </div>
              </div>
            </div>

            {/* --- Parallel Agents Coming Soon --- */}
            <div className="evo-card" style={{ borderTopColor: "#ff9500" }}>
              <div className="evo-card-title">Parallel Agents</div>
              <div className="evo-card-body">
                <div
                  style={{
                    padding: "12px 14px",
                    background: "rgba(255,149,0,0.08)",
                    borderRadius: 8,
                    fontSize: 12,
                    lineHeight: 1.6,
                    color: "var(--mac-text-secondary, #86868b)",
                  }}
                >
                  <strong style={{ color: "#ff9500" }}>Coming Soon</strong> — Multi-agent orchestration
                  is on the roadmap. Future releases will support:
                  <ul style={{ margin: "6px 0 0 0", paddingLeft: 16 }}>
                    <li>Running multiple agents concurrently</li>
                    <li>Agent-to-agent communication (A2A protocol)</li>
                    <li>Role-based agent teams (planner, coder, reviewer)</li>
                    <li>Parallel task execution with dependency graphs</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>

          <div className="evo-column">
            {/* --- Task History --- */}
            <div className="evo-card" style={{ borderTopColor: "#34c759" }}>
              <div className="evo-card-title">Recent Tasks</div>
              <div className="evo-card-body" style={{ padding: 0 }}>
                {recentTasks.length === 0 ? (
                  <div
                    style={{
                      padding: 24,
                      textAlign: "center",
                      fontSize: 12,
                      color: "var(--mac-text-secondary, #86868b)",
                    }}
                  >
                    No tasks yet. Start an agent above.
                  </div>
                ) : (
                  <div style={{ maxHeight: 360, overflowY: "auto" }}>
                    {recentTasks.map((task) => (
                      <div
                        key={task.id}
                        className="evo-stat-row"
                        style={{
                          padding: "8px 12px",
                          cursor: "pointer",
                          borderBottom: "1px solid var(--border-color, rgba(128,128,128,0.08))",
                          transition: "background 0.15s",
                        }}
                        onClick={() => handleRerun(task)}
                        onMouseEnter={(e) => (e.currentTarget.style.background = "var(--mac-hover, rgba(128,128,128,0.06))")}
                        onMouseLeave={(e) => (e.currentTarget.style.background = "transparent")}
                        title="Click to re-run"
                      >
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", width: "100%" }}>
                          <div style={{ flex: 1, minWidth: 0 }}>
                            <div
                              style={{
                                fontSize: 12,
                                fontWeight: 500,
                                overflow: "hidden",
                                textOverflow: "ellipsis",
                                whiteSpace: "nowrap",
                              }}
                            >
                              {task.prompt}
                            </div>
                            <div style={{ fontSize: 10, opacity: 0.5, marginTop: 2 }}>
                              {new Date(task.timestamp).toLocaleTimeString()} · {task.status}
                              {task.duration_secs > 0 ? ` · ${formatDuration(task.duration_secs)}` : ""}
                            </div>
                          </div>
                          <span
                            style={{
                              width: 6,
                              height: 6,
                              borderRadius: "50%",
                              flexShrink: 0,
                              marginLeft: 8,
                              backgroundColor:
                                task.status === "started"
                                  ? "#34c759"
                                  : task.status === "stopped"
                                    ? "#ff3b30"
                                    : "#aeaeb2",
                            }}
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes pulse-dot {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
        .evolution-panel [style*="boxShadow"] {
          animation: pulse-dot 1.5s ease-in-out infinite;
        }
      `}</style>
    </div>
  );
};

export default AgentDashboard;
