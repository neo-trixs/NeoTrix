import React from "react";
import { useStore } from "../store";
import type { Message } from "../types";

const TaskProgressPanel: React.FC = () => {
  const agentBusy = useStore((s) => s.agentBusy);
  const activeMessages = useStore((s) => s.activeMessages());

  if (!agentBusy) return null;

  const allSteps: NonNullable<Message["steps"]> = [];
  for (const msg of activeMessages) {
    if (msg.steps) allSteps.push(...msg.steps);
  }

  return (
    <div className="task-progress-panel glass-panel">
      <div className="task-progress-header">
        <span className="task-progress-dot" />
        <span>Agent Working...</span>
      </div>
      {allSteps.length > 0 ? (
        <ul className="task-progress-list">
          {allSteps.map((step, i) => (
            <li key={i} className={`task-progress-item task-progress-${step.status}`}>
              <span className="task-progress-icon">
                {step.status === "done" ? "✅" : step.status === "running" ? "🔄" : "⏳"}
              </span>
              <span className="task-progress-label">{step.label}</span>
            </li>
          ))}
        </ul>
      ) : (
        <div className="task-progress-thinking">
          <span className="task-progress-thinking-dot" />
          <span className="task-progress-thinking-dot" />
          <span className="task-progress-thinking-dot" />
          <span>Thinking...</span>
        </div>
      )}
      <style>{`
        @keyframes task-pulse {
          0%, 100% { opacity: 0.5; transform: scale(1); }
          50% { opacity: 0.8; transform: scale(1.15); }
        }
        @keyframes task-bounce {
          0%, 80%, 100% { transform: translateY(0); opacity: 0.3; }
          40% { transform: translateY(-4px); opacity: 0.9; }
        }
        .task-progress-panel {
          margin: 8px 0; padding: 10px 14px; display: flex; flex-direction: column; gap: 6px;
          border-left: 3px solid var(--mac-primary, #0a84ff);
        }
        .task-progress-header {
          display: flex; align-items: center; gap: 8px; font-size: 0.8em; font-weight: 600;
          color: var(--mac-text-secondary, #86868b);
        }
        .task-progress-dot {
          width: 8px; height: 8px; border-radius: 50%;
          background: var(--mac-primary, #0a84ff);
          animation: task-pulse 1.2s ease-in-out infinite;
        }
        .task-progress-list {
          list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px;
        }
        .task-progress-item {
          display: flex; align-items: center; gap: 6px; font-size: 0.8em; padding: 3px 6px;
          border-radius: 4px; transition: background 0.15s;
        }
        .task-progress-item:hover { background: var(--mac-hover, rgba(0,0,0,0.04)); }
        .task-progress-done { opacity: 0.7; }
        .task-progress-done .task-progress-label { color: var(--mac-text-secondary, #86868b); }
        .task-progress-running { opacity: 1; background: var(--mac-primary-light, rgba(0,122,255,0.08)); }
        .task-progress-running .task-progress-label { color: var(--mac-primary, #0a84ff); font-weight: 500; }
        .task-progress-pending { opacity: 0.45; }
        .task-progress-icon { flex-shrink: 0; font-size: 0.85em; }
        .task-progress-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .task-progress-thinking {
          display: flex; align-items: center; gap: 4px; font-size: 0.8em;
          color: var(--mac-text-secondary, #86868b); padding: 4px 6px;
        }
        .task-progress-thinking-dot {
          width: 5px; height: 5px; border-radius: 50%; background: var(--mac-text-secondary, #86868b);
          animation: task-bounce 1.4s ease-in-out infinite;
        }
        .task-progress-thinking-dot:nth-child(2) { animation-delay: 0.16s; }
        .task-progress-thinking-dot:nth-child(3) { animation-delay: 0.32s; }
      `}</style>
    </div>
  );
};

export default TaskProgressPanel;
