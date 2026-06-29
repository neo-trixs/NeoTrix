import React from "react";
import { useStore } from "../store";

const Gauge: React.FC<{ label: string; value: number; color: string; format?: (v: number) => string }> = ({ label, value, color, format }) => (
  <div className="evo-gauge">
    <div className="evo-gauge-header">
      <span className="evo-gauge-label">{label}</span>
      <span className="evo-gauge-value" style={{ color }}>{format ? format(value) : (value * 100).toFixed(0)}%</span>
    </div>
    <div className="evo-gauge-track">
      <div className="evo-gauge-fill" style={{ width: `${Math.min(value * 100, 100)}%`, backgroundColor: color }} />
    </div>
  </div>
);

const StatRow: React.FC<{ label: string; value: string | number; color?: string }> = ({ label, value, color }) => (
  <div className="evo-stat-row">
    <span className="evo-stat-label">{label}</span>
    <span className="evo-stat-value" style={color ? { color } : undefined}>{value}</span>
  </div>
);

const SectionCard: React.FC<{ title: string; accent: string; children: React.ReactNode }> = ({ title, accent, children }) => (
  <div className="evo-card" style={{ borderTopColor: accent }}>
    <div className="evo-card-title">{title}</div>
    <div className="evo-card-body">{children}</div>
  </div>
);

const EvolutionPanel: React.FC = () => {
  const setEvolutionVisible = useStore((s) => s.setEvolutionVisible);
  const evolutionState = useStore((s) => s.evolutionState);
  const s = evolutionState;

  return (
    <div className="evolution-panel">
      <div className="evolution-panel-toolbar">
        <div className="evolution-panel-toolbar-left">
          <h2>Evolution Dashboard</h2>
          <span className="evo-iteration-badge">v{s.iteration}</span>
        </div>
        <div className="evolution-panel-toolbar-right">
          <span className="evo-strategy-badge">{s.strategy}</span>
          <button className="btn-icon" onClick={() => setEvolutionVisible(false)} title="Close">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        </div>
      </div>

      <div className="evolution-panel-body">
        <div className="evo-columns">
          <div className="evo-column">
            <SectionCard title="Intrinsic Motivation" accent="#0a84ff">
              <Gauge label="R_int (Reward)" value={s.intrinsicReward} color="#0a84ff" />
              <Gauge label="Confidence" value={s.confidence} color="#34c759" />
              <Gauge label="Error Rate" value={s.errorRate} color="#ff3b30" format={(v) => (v * 100).toFixed(1) + "%"} />
              <Gauge label="Novelty Score" value={s.noveltyScore} color="#ff9500" />
              <StatRow label="Explore Mode" value={s.shouldExplore ? "Active" : "Inactive"} color={s.shouldExplore ? "#34c759" : "#aeaeb2"} />
            </SectionCard>

            <SectionCard title="Self-Repair" accent="#ff9500">
              <StatRow label="Repairs Triggered" value={s.repairsCount} color="#ff9500" />
              <StatRow label="Self-Repairs" value={s.selfRepairs} color="#34c759" />
            </SectionCard>
          </div>

          <div className="evo-column">
            <SectionCard title="Cognitive Health" accent="#ff9500">
              <Gauge label="Stability Score" value={s.stabilityScore} color="#ff9500" />
              <StatRow label="Active Flags" value={s.flagsCount} color={s.flagsCount > 0 ? "#ff3b30" : "#34c759"} />
              <div className="evo-flags-list">
                {s.flagsCount > 0 ? (
                  <div className="evo-flag-item">⚠ {s.flagsCount} flag(s) — review suggested</div>
                ) : (
                  <div className="evo-flag-item evo-flag-clear">✓ No active flags</div>
                )}
              </div>
              <div className="evo-repair-suggestion">
                {s.flagsCount > 2
                  ? "Repair: Increase exploration rate + reduce learning rate"
                  : s.flagsCount > 0
                    ? "Repair: Review recent absorb history"
                    : "System nominal — no repair needed"}
              </div>
            </SectionCard>

            <SectionCard title="Archive" accent="#34c759">
              <StatRow label="Snapshots" value={s.archiveSnapshots} color="#34c759" />
              <StatRow label="Context Usage" value={(s.contextUsage * 100).toFixed(0) + "%"} color={s.contextUsage > 0.8 ? "#ff9500" : "#34c759"} />
              <StatRow label="Latest Iteration" value={`#${s.iteration}`} />
              <StatRow label="Strategy" value={s.strategy} />
            </SectionCard>
          </div>
        </div>
      </div>
    </div>
  );
};

export default EvolutionPanel;
