import React from "react";
import { useStore } from "../store";

const PIPELINE_STEPS = [
  { id: "GATHER", label: "Gather", icon: "📡" },
  { id: "GATE", label: "Gate", icon: "🚪" },
  { id: "PROPOSE", label: "Propose", icon: "💡" },
  { id: "COMPETE", label: "Compete", icon: "⚔️" },
  { id: "REASON", label: "Reason", icon: "🧠" },
  { id: "JUDGE", label: "Judge", icon: "⚖️" },
  { id: "VERIFY", label: "Verify", icon: "✅" },
  { id: "ACT", label: "Act", icon: "⚡" },
  { id: "VETO", label: "Veto", icon: "🛑" },
  { id: "RECORD", label: "Record", icon: "💾" },
  { id: "METRIC", label: "Metric", icon: "📊" },
  { id: "META", label: "Meta", icon: "🔄" },
  { id: "SLEEP", label: "Sleep", icon: "🌙" },
];

const ConsciousnessPipeline: React.FC = () => {
  const step = useStore((s) => s.consciousnessStep);
  const data = useStore((s) => s.consciousnessData);
  const agentBusy = useStore((s) => s.agentBusy);

  if (!agentBusy) return null;

  const cScore = data?.c_score ?? 0.5;
  const coherence = data?.coherence ?? 0.5;
  const emotion = data?.emotion ?? "neutral";
  const reflexivity = data?.reflexivity ?? 0.3;

  const emotionColors: Record<string, string> = {
    neutral: "#aeaeb2", happy: "#34c759", sad: "#0a84ff",
    angry: "#ff3b30", curious: "#ff9500", anxious: "#af52de",
  };
  const ec = emotionColors[emotion] || "#aeaeb2";

  return (
    <div className="consciousness-pipeline glass-panel">
      <div className="cp-header">
        <div className="cp-header-left">
          <span className="cp-title">Consciousness Cycle</span>
          <span className="cp-badge" style={{ color: ec }}>
            {emotion}
          </span>
        </div>
        <div className="cp-metrics">
          <span className="cp-metric" title="Consciousness Score">
            <span className="cp-metric-dot" style={{ background: "#0a84ff" }} />
            C(t) {(cScore * 100).toFixed(0)}%
          </span>
          <span className="cp-metric" title="Coherence">
            <span className="cp-metric-dot" style={{ background: "#34c759" }} />
            Coh {(coherence * 100).toFixed(0)}%
          </span>
          <span className="cp-metric" title="Reflexivity">
            <span className="cp-metric-dot" style={{ background: "#af52de" }} />
            Ref {(reflexivity * 100).toFixed(0)}%
          </span>
        </div>
      </div>

      <div className="cp-steps">
        {PIPELINE_STEPS.map((s, i) => {
          const isActive = i === step;
          const isPast = i < step;
          const isFuture = i > step;
          const progress = isPast ? 1 : isActive ? 0.5 : 0;
          return (
            <React.Fragment key={s.id}>
              {i > 0 && (
                <div className={`cp-connector ${isPast ? "cp-connector-done" : isActive ? "cp-connector-active" : ""}`}>
                  <div className="cp-connector-fill" style={{ width: `${progress * 100}%` }} />
                </div>
              )}
              <div
                className={`cp-step ${isActive ? "cp-step-active" : isPast ? "cp-step-done" : "cp-step-pending"}`}
                title={`${s.label} — ${isActive ? "running" : isPast ? "completed" : "pending"}`}
              >
                <div className="cp-step-icon">{s.icon}</div>
                <div className="cp-step-label">{s.label}</div>
                <div className="cp-step-bar">
                  <div
                    className="cp-step-bar-fill"
                    style={{
                      width: `${progress * 100}%`,
                      background: isActive ? "#0a84ff" : "#34c759",
                    }}
                  />
                </div>
              </div>
            </React.Fragment>
          );
        })}
      </div>

      <style>{`
        .consciousness-pipeline {
          margin: 0 0 8px 0;
          padding: 10px 14px;
          border-radius: 10px;
          background: color-mix(in srgb, var(--mac-surface-1, #141420) 85%, transparent);
          border: 1px solid color-mix(in srgb, var(--mac-border, #2a2a3e) 50%, transparent);
          backdrop-filter: blur(20px);
          -webkit-backdrop-filter: blur(20px);
        }
        .cp-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          margin-bottom: 8px;
        }
        .cp-header-left {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        .cp-title {
          font-size: 0.75em;
          font-weight: 600;
          color: var(--mac-text-secondary, #888);
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }
        .cp-badge {
          font-size: 0.7em;
          text-transform: uppercase;
          letter-spacing: 0.03em;
          font-weight: 500;
          padding: 1px 6px;
          border-radius: 4px;
          background: color-mix(in srgb, currentColor 10%, transparent);
        }
        .cp-metrics {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        .cp-metric {
          font-size: 0.65em;
          color: var(--mac-text-secondary, #888);
          display: flex;
          align-items: center;
          gap: 4px;
          white-space: nowrap;
        }
        .cp-metric-dot {
          width: 5px;
          height: 5px;
          border-radius: 50%;
          display: inline-block;
          flex-shrink: 0;
        }
        .cp-steps {
          display: flex;
          align-items: center;
          gap: 0;
          overflow-x: auto;
          padding: 4px 0;
        }
        .cp-step {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 2px;
          flex-shrink: 0;
          transition: all 0.3s ease;
          padding: 4px 0;
          width: 52px;
        }
        .cp-step-icon {
          font-size: 0.85em;
          line-height: 1;
          transition: transform 0.3s ease;
        }
        .cp-step-active .cp-step-icon {
          transform: scale(1.25);
          animation: cp-pulse 1.5s ease-in-out infinite;
        }
        .cp-step-label {
          font-size: 0.6em;
          font-weight: 500;
          white-space: nowrap;
          letter-spacing: 0.02em;
        }
        .cp-step-done .cp-step-label { color: var(--mac-primary, #7c5cfc); }
        .cp-step-active .cp-step-label { color: #0a84ff; font-weight: 700; }
        .cp-step-pending .cp-step-label { color: var(--mac-text-muted, #555); }
        .cp-step-done .cp-step-icon { filter: none; }
        .cp-step-pending .cp-step-icon { opacity: 0.35; filter: grayscale(0.8); }
        .cp-step-bar {
          width: 100%;
          height: 2px;
          background: var(--mac-surface-2, #1e1e2e);
          border-radius: 1px;
          overflow: hidden;
          margin-top: 1px;
        }
        .cp-step-bar-fill {
          height: 100%;
          border-radius: 1px;
          transition: width 0.5s ease;
        }
        .cp-connector {
          flex-shrink: 0;
          width: 14px;
          height: 2px;
          background: var(--mac-surface-2, #1e1e2e);
          border-radius: 1px;
          overflow: hidden;
          margin-bottom: 16px;
        }
        .cp-connector-done { background: color-mix(in srgb, var(--mac-primary, #7c5cfc) 30%, transparent); }
        .cp-connector-active { background: color-mix(in srgb, #0a84ff 30%, transparent); }
        .cp-connector-fill {
          height: 100%;
          background: var(--mac-primary, #7c5cfc);
          transition: width 0.5s ease;
        }
        .cp-connector-active .cp-connector-fill {
          background: #0a84ff;
          animation: cp-flow 1s linear infinite;
        }
        @keyframes cp-pulse {
          0%, 100% { transform: scale(1.25); opacity: 1; }
          50% { transform: scale(1.4); opacity: 0.7; }
        }
        @keyframes cp-flow {
          0% { width: 0%; }
          100% { width: 100%; }
        }
      `}</style>
    </div>
  );
};

export default ConsciousnessPipeline;
