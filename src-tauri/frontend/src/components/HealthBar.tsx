import React from "react";
import type { BrainHealth } from "../types";

interface Props {
  health: BrainHealth;
  onOpenSettings: () => void;
}

const HealthBar: React.FC<Props> = ({ health, onOpenSettings }) => {
  const score = health.health_score;
  const color = score > 70 ? "#34d399" : score > 40 ? "#fbbf24" : "#ef4444";
  const bgColor = score > 70 ? "rgba(52,211,153,0.12)" : score > 40 ? "rgba(251,191,36,0.12)" : "rgba(239,68,68,0.12)";

  return (
    <div className="health-bar">
      <div className="health-bar-left">
        <div className="health-gauge" style={{ background: bgColor }}>
          <span className="health-dot" style={{ background: color }} />
          <span className="health-value" style={{ color }}>{Math.round(score)}</span>
        </div>
        <div className="health-meta">
          <span className="health-tag">{health.degradation || "full"}</span>
          <span className="health-divider" />
          <span className="health-tag">{health.cognitive_load || "balanced"}</span>
          <span className="health-divider" />
          <span className="health-iter">iter {health.iteration || 0}</span>
        </div>
      </div>
      <div className="health-bar-right">
        <button className="health-btn" onClick={onOpenSettings} title="Settings">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.2" />
            <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default HealthBar;
