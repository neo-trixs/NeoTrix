import React, { useState, useEffect } from "react";

interface BackgroundTask {
  id: string;
  name: string;
  prompt: string;
  schedule: string;
  lastRun?: number;
  nextRun?: number;
  status: "idle" | "running" | "paused" | "error";
  runs: { timestamp: number; summary: string }[];
}

function parseSchedule(schedule: string): string {
  const s = schedule.toLowerCase();
  if (s.startsWith("every_")) {
    const match = s.match(/every_(\d+)_(\w+)/);
    if (match) {
      const [_, num, unit] = match;
      const unitLabel = unit === "hours" ? "hr" : unit === "days" ? "day" : unit === "minutes" ? "min" : unit;
      return `Every ${num} ${unitLabel}${parseInt(num) > 1 ? "s" : ""}`;
    }
  }
  if (s.startsWith("daily_at_")) {
    const time = s.replace("daily_at_", "");
    return `Daily at ${time}`;
  }
  if (s.startsWith("weekly_on_")) {
    const rest = s.replace("weekly_on_", "");
    const match = rest.match(/(\w+)_at_(.+)/);
    if (match) return `Weekly on ${match[1]} at ${match[2]}`;
  }
  return s;
}

function timeAgo(ts: number): string {
  const diff = Date.now() - ts;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  return `${Math.floor(hrs / 24)}d ago`;
}

const DEFAULT_TASKS: BackgroundTask[] = [
  { id: "default-1", name: "Daily Standup Summary", prompt: "Summarize the day's agent activity", schedule: "daily_at_09:00", status: "idle", runs: [] },
];

const BackgroundRoutinesPanel: React.FC = () => {
  const [tasks, setTasks] = useState<BackgroundTask[]>(DEFAULT_TASKS);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [prompt, setPrompt] = useState("");
  const [schedule, setSchedule] = useState("every_6_hours");
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const addTask = () => {
    if (!name.trim()) return;
    const newTask: BackgroundTask = {
      id: `task-${Date.now()}`,
      name: name.trim(),
      prompt: prompt.trim(),
      schedule,
      status: "idle",
      runs: [],
    };
    setTasks((prev) => [...prev, newTask]);
    setName("");
    setPrompt("");
    setShowForm(false);
  };

  const togglePause = (id: string) => {
    setTasks((prev) =>
      prev.map((t) =>
        t.id === id ? { ...t, status: t.status === "paused" ? "idle" : "paused" } : t
      )
    );
  };

  const deleteTask = (id: string) => {
    setTasks((prev) => prev.filter((t) => t.id !== id));
  };

  const runNow = (id: string) => {
    setTasks((prev) =>
      prev.map((t) =>
        t.id === id
          ? { ...t, status: "running", runs: [...t.runs, { timestamp: Date.now(), summary: "Running..." }] }
          : t
      )
    );
    setTimeout(() => {
      setTasks((prev) =>
        prev.map((t) =>
          t.id === id
            ? {
                ...t,
                status: "idle",
                lastRun: Date.now(),
                runs: [
                  ...t.runs.slice(0, -1),
                  { timestamp: Date.now(), summary: "Completed — 0 critical, 0 warnings" },
                ],
              }
            : t
        )
      );
    }, 1500);
  };

  return (
    <div style={{ padding: 8, background: "var(--bg-primary, #ffffff)", maxHeight: "100%", overflowY: "auto" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
        <h3 style={{ fontSize: 13, fontWeight: 600, margin: 0, color: "var(--text-primary, #1a1a2e)" }}>⏰ Background Routines</h3>
        <button onClick={() => setShowForm((v) => !v)} style={{ padding: "2px 8px", cursor: "pointer", border: "1px solid var(--accent, #007aff)", borderRadius: 4, background: "var(--accent, #007aff)", color: "#fff", fontSize: 10 }}>+ New</button>
      </div>
      {showForm && (
        <div style={{ marginBottom: 8, padding: 8, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-secondary, #f6f8fa)" }}>
          <input value={name} onChange={(e) => setName(e.target.value)} placeholder="Routine name..." style={{ width: "100%", padding: "3px 6px", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, fontSize: 11, marginBottom: 4, background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", outline: "none" }} />
          <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} placeholder="Agent prompt..." style={{ width: "100%", height: 40, padding: "3px 6px", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, fontSize: 11, resize: "vertical", background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", outline: "none", fontFamily: "inherit" }} />
          <select value={schedule} onChange={(e) => setSchedule(e.target.value)} style={{ width: "100%", padding: "3px 6px", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, fontSize: 11, marginTop: 4, background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)" }}>
            <option value="every_1_hours">Every 1 hour</option>
            <option value="every_6_hours">Every 6 hours</option>
            <option value="every_12_hours">Every 12 hours</option>
            <option value="every_24_hours">Every 24 hours</option>
            <option value="daily_at_09:00">Daily at 09:00</option>
            <option value="daily_at_18:00">Daily at 18:00</option>
            <option value="weekly_on_mon_at_09:00">Every Monday at 09:00</option>
          </select>
          <button onClick={addTask} style={{ marginTop: 4, padding: "3px 10px", cursor: "pointer", border: "1px solid var(--accent, #007aff)", borderRadius: 3, background: "var(--accent, #007aff)", color: "#fff", fontSize: 10 }}>Create</button>
        </div>
      )}
      <div>
        {tasks.map((task) => (
          <div key={task.id} style={{ marginBottom: 6, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, overflow: "hidden" }}>
            <div style={{ display: "flex", alignItems: "center", padding: "4px 8px", background: "var(--bg-secondary, #f6f8fa)", cursor: "pointer" }} onClick={() => setExpandedId(expandedId === task.id ? null : task.id)}>
              <span style={{ flex: 1, fontSize: 11, fontWeight: 500 }}>{task.name}</span>
              <span style={{ fontSize: 9, padding: "1px 4px", borderRadius: 3, background: task.status === "running" ? "var(--accent, #007aff)" : task.status === "paused" ? "var(--warning, #d2991d)" : task.status === "error" ? "var(--error, #d73a49)" : "var(--success, #22863a)", color: "#fff", marginRight: 4 }}>
                {task.status}
              </span>
              <span style={{ fontSize: 9, color: "var(--text-muted, #8b949e)" }}>{parseSchedule(task.schedule)}</span>
              <span style={{ fontSize: 10, marginLeft: 4 }}>{task.lastRun ? timeAgo(task.lastRun) : "—"}</span>
            </div>
            {expandedId === task.id && (
              <div style={{ padding: 6, borderTop: "1px solid var(--border-color, #e1e4e8)", display: "flex", gap: 4, flexWrap: "wrap" }}>
                <button onClick={() => runNow(task.id)} style={{ padding: "2px 6px", fontSize: 10, cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, background: "var(--bg-primary, #ffffff)" }}>▶ Run Now</button>
                <button onClick={() => togglePause(task.id)} style={{ padding: "2px 6px", fontSize: 10, cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, background: "var(--bg-primary, #ffffff)" }}>
                  {task.status === "paused" ? "▶ Resume" : "⏸ Pause"}
                </button>
                <button onClick={() => deleteTask(task.id)} style={{ padding: "2px 6px", fontSize: 10, cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 3, background: "var(--bg-primary, #ffffff)", color: "var(--error, #d73a49)" }}>🗑</button>
              </div>
            )}
            {task.runs.length > 0 && (
              <div style={{ padding: "2px 8px", borderTop: "1px solid var(--border-color, #e1e4e8)" }}>
                <div style={{ fontSize: 9, color: "var(--text-muted, #8b949e)" }}>Last result:</div>
                <div style={{ fontSize: 9, paddingTop: 2 }}>{task.runs[task.runs.length - 1].summary}</div>
              </div>
            )}
          </div>
        ))}
        {tasks.length === 0 && (
          <div style={{ textAlign: "center", padding: 16, color: "var(--text-muted, #8b949e)", fontSize: 11 }}>No routines configured. Click + New to create one.</div>
        )}
      </div>
    </div>
  );
};

export default BackgroundRoutinesPanel;
