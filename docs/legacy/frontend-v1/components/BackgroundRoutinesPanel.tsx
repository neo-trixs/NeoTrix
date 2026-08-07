import React, { useState, useEffect } from "react";
import "./BackgroundRoutinesPanel.css";

interface BackgroundRoutinesPanelProps {
  tasks: BackgroundTask[];
  onAddTask: (task: { name: string; prompt: string; schedule: string }) => void;
  onRunTask: (id: string) => void;
  onPauseTask: (id: string) => void;
  onDeleteTask: (id: string) => void;
}

interface BackgroundTask {
  id: string;
  name: string;
  prompt: string;
  schedule: string;
  lastRun: string | null;
  status: "idle" | "running" | "paused" | "error";
}

export function BackgroundRoutinesPanel({
  tasks,
  onAddTask,
  onRunTask,
  onPauseTask,
  onDeleteTask,
}: BackgroundRoutinesPanelProps): JSX.Element {
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [prompt, setPrompt] = useState("");
  const [schedule, setSchedule] = useState("*/5 * * * *");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !prompt.trim()) return;
    onAddTask({ name, prompt, schedule });
    setName("");
    setPrompt("");
    setShowForm(false);
  };

  const statusIcon = (status: string) => {
    switch (status) {
      case "running": return "🔄";
      case "paused": return "⏸️";
      case "error": return "❌";
      default: return "💤";
    }
  };

  return (
    <div className="background-routines-panel">
      <div className="routines-header">
        <h3>Background Routines</h3>
        <button className="add-btn" onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "+ New Task"}
        </button>
      </div>
      {showForm && (
        <form className="routine-form" onSubmit={handleSubmit}>
          <input
            className="routine-input"
            placeholder="Task name..."
            value={name}
            onChange={e => setName(e.target.value)}
          />
          <textarea
            className="routine-prompt"
            placeholder="What should this task do?"
            value={prompt}
            onChange={e => setPrompt(e.target.value)}
            rows={3}
          />
          <input
            className="routine-input"
            placeholder="Cron schedule (e.g., */5 * * * *)"
            value={schedule}
            onChange={e => setSchedule(e.target.value)}
          />
          <button type="submit" className="routine-submit">Create Task</button>
        </form>
      )}
      <div className="routines-list">
        {tasks.map(task => (
          <div key={task.id} className="routine-item">
            <span className="routine-status">{statusIcon(task.status)}</span>
            <span className="routine-name">{task.name}</span>
            <span className="routine-schedule">{task.schedule}</span>
            <div className="routine-actions">
              <button onClick={() => onRunTask(task.id)}>▶</button>
              <button onClick={() => onPauseTask(task.id)}>⏸</button>
              <button onClick={() => onDeleteTask(task.id)}>✕</button>
            </div>
          </div>
        ))}
        {tasks.length === 0 && (
          <div className="routines-empty">No background tasks yet. Create one to get started.</div>
        )}
      </div>
    </div>
  );
}
