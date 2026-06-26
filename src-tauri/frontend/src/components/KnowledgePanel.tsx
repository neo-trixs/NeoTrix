import { useEffect, useState } from "react";
import * as api from "../lib/api";

type Tab = "stats" | "awakening";

const KnowledgePanel: React.FC = () => {
  const [tab, setTab] = useState<Tab>("stats");

  return (
    <div className="h-full flex flex-col p-6 overflow-y-auto">
      <div className="flex gap-0 border-b border-border mb-6">
        <button onClick={() => setTab("stats")} className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${tab === "stats" ? "text-text-primary border-accent" : "text-text-muted border-transparent"}`}>
          Brain
        </button>
        <button onClick={() => setTab("awakening")} className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${tab === "awakening" ? "text-text-primary border-accent" : "text-text-muted border-transparent"}`}>
          Awakening Φ
        </button>
      </div>

      {tab === "stats" && <BrainStatsView />}
      {tab === "awakening" && <AwakeningView />}
    </div>
  );
};

const AwakeningView: React.FC = () => {
  const [data, setData] = useState({ phi: 0, fcs: 0, usk: 0, lastRun: 0 });

  useEffect(() => {
    const tick = async () => {
      try {
        const m = await api.getConsciousnessMetrics();
        setData({ phi: m.phi, fcs: m.fcs, usk: m.usk, lastRun: Date.now() });
      } catch {
        setData({ phi: 0, fcs: 0, usk: 0, lastRun: Date.now() });
      }
    };
    tick();
    const interval = setInterval(tick, 4000);
    return () => clearInterval(interval);
  }, []);

  const metrics = [
    { label: "Φ (Phi)", value: data.phi, color: "#0a84ff", fmt: (v: number) => v.toFixed(1) },
    { label: "FCS", value: data.fcs * 100, color: "#34c759", fmt: (v: number) => `${v.toFixed(0)}%` },
    { label: "USK", value: data.usk * 100, color: "#af52de", fmt: (v: number) => v.toFixed(0) },
  ];

  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-8">
      <div className="flex gap-10">
        {metrics.map(m => (
          <div key={m.label} className="flex flex-col items-center gap-3">
            <div
              className="evo-icon active"
              style={{
                borderColor: m.color,
                color: m.color,
                boxShadow: `0 0 24px ${m.color}33`,
                width: 80, height: 80, fontSize: 22,
                animation: "pulse-soft 4s ease-in-out infinite",
              }}
            >
              {m.fmt(m.value)}
            </div>
            <span className="text-xs font-medium" style={{ color: m.color }}>{m.label}</span>
          </div>
        ))}
      </div>
      <div className="text-xs text-text-muted">consciousness metrics — auto-computed</div>
    </div>
  );
};

const BrainStatsView: React.FC = () => {
  const [stats, setStats] = useState<api.BrainStats | null>(null);
  const [showVector, setShowVector] = useState(false);

  useEffect(() => {
    const fetch = async () => { try { setStats(await api.getBrainStats()); } catch {} };
    fetch();
    const interval = setInterval(fetch, 5000);
    return () => clearInterval(interval);
  }, []);

  const capVector = stats?.capability_vector ?? [];
  const dimNames = stats?.dimension_names ?? [];
  const maxCap = Math.max(...capVector, 1);
  const COLORS = ["#0a84ff", "#34c759", "#ff9500", "#ff3b30", "#af52de", "#5ac8fa", "#ff2d55", "#5856d6"];

  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {[
          ["Cap Sum", stats?.capability_sum.toFixed(4) ?? "—", "#0a84ff"],
          ["Iterations", stats?.iteration ?? "—", "#34c759"],
          ["Absorbs", stats?.absorb_count ?? "—", "#ff9500"],
          ["Memory", stats?.memory_count ?? "—", "#af52de"],
        ].map(([label, value, color]) => (
          <div key={label as string} className="border border-border rounded-lg p-3 bg-surface-1">
            <div className="text-xs text-text-muted mb-1">{label}</div>
            <div className="text-lg font-semibold" style={{ color: color as string }}>{value}</div>
          </div>
        ))}
      </div>
      <button onClick={() => setShowVector(!showVector)} className="self-start text-xs text-accent bg-none border-none cursor-pointer">
        {showVector ? "Hide" : "Show"} Capability Vector
      </button>
      {showVector && (
        <div className="border border-border rounded-lg p-4 bg-surface-1">
          <div className="flex flex-col gap-1.5">
            {capVector.map((v: number, i: number) => (
              <div key={i} className="flex items-center gap-2">
                <span className="text-xs text-text-muted w-16 truncate text-right shrink-0">{dimNames[i] ?? `dim_${i}`}</span>
                <div className="flex-1 h-3 bg-border rounded-full overflow-hidden">
                  <div className="h-full rounded-full transition-all duration-500" style={{ width: `${(v / maxCap) * 100}%`, backgroundColor: COLORS[i % COLORS.length] }} />
                </div>
                <span className="text-xs font-mono text-text-muted w-8 text-right shrink-0">{v.toFixed(3)}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default KnowledgePanel;
