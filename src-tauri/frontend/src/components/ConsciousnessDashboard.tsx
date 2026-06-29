import { useState, useEffect, useRef } from "react";
import { useStore } from "../store";
import { listen } from "@tauri-apps/api/event";
import * as api from "../lib/api";
import E8Visualizer3D from "./E8Visualizer3D";
import SpatialReasoningVisualizer, { demoConcepts, demoPaths } from "./SpatialReasoningVisualizer";
import SelfModelTheater from "./SelfModelTheater";
import ThinkingTrajectoryVisualizer from "./ThinkingTrajectoryVisualizer";

type DashboardTab = "status" | "e8" | "vsa" | "compartments" | "deep" | "spatial" | "theater" | "thinking";

interface StreamEvent {
  cycle: number;
  c_score: number;
  coherence: number;
  emotion: string;
  reflexivity: number;
  sleep_pressure: number;
  vsa_buffer_size: number;
  load_mode: string;
  critic_pass_rate: number;
  narrative_sessions: number;
  dgmh_applied: number;
  dgmh_rejected: number;
  goals_total: number;
  spatial_objects: number;
  physics_props: number;
  ece: number;
  meta_accuracy: number;
  alignment_strength: number;
  basins: number;
  memories: number;
  dreams: number;
  reasoning_mode: string;
}

interface FullState {
  heuristics: number; skills: number; compositions: number;
  basins: number; memories: number; dreams: number;
  c_score: number; reflexivity: number; reasoning_mode: string;
  emotion: string; load_mode: string; sleep_pressure: number;
  vsa_buffer_size: number; dgmh_buffer: number;
  dgmh_applied: number; dgmh_rejected: number;
  goals_total: number; spatial_objects: number; physics_props: number;
  meta_accuracy: number; ece: number;
  overall_satisfaction: number; alignment_strength: number;
}

const PIPELINE_STEPS = [
  "GATHER", "GATE", "PROPOSE", "COMPETE", "REASON", "JUDGE",
  "VERIFY", "ACT", "VETO", "RECORD", "METRIC", "META", "SLEEP",
];

const EMOTION_COLORS: Record<string, string> = {
  neutral: "#aeaeb2", happy: "#34c759", sad: "#0a84ff",
  angry: "#ff3b30", curious: "#ff9500", anxious: "#af52de",
};

const Gauge: React.FC<{ label: string; value: number; color: string; format?: (v: number) => string }> = ({ label, value, color, format }) => (
  <div className="cd-gauge">
    <div className="cd-gauge-header">
      <span className="cd-gauge-label">{label}</span>
      <span className="cd-gauge-value" style={{ color }}>{format ? format(value) : (value * 100).toFixed(0)}%</span>
    </div>
    <div className="cd-gauge-track">
      <div className="cd-gauge-fill" style={{ width: `${Math.min(value * 100, 100)}%`, backgroundColor: color }} />
    </div>
  </div>
);

const StatRow: React.FC<{ label: string; value: string | number; color?: string }> = ({ label, value, color }) => (
  <div className="cd-stat-row">
    <span className="cd-stat-label">{label}</span>
    <span className="cd-stat-value" style={color ? { color } : undefined}>{value}</span>
  </div>
);

const PipelineMini: React.FC<{ step: number }> = ({ step }) => (
  <div className="cd-pipeline-mini">
    {PIPELINE_STEPS.map((s, i) => (
      <div key={s} className={`cd-pstep ${i === step ? "cd-pstep-active" : i < step ? "cd-pstep-done" : "cd-pstep-pending"}`}>
        <div className="cd-pstep-label">{s}</div>
      </div>
    ))}
  </div>
);

const MetricCircle: React.FC<{ label: string; value: string | number; color: string; subtitle?: string }> = ({ label, value, color, subtitle }) => (
  <div className="cd-metric-circle" style={{ borderColor: color }}>
    <div className="cd-metric-value" style={{ color }}>{value}</div>
    <div className="cd-metric-label">{label}</div>
    {subtitle && <div className="cd-metric-subtitle">{subtitle}</div>}
  </div>
);

const StatusTab: React.FC<{ data: StreamEvent; full: FullState | null }> = ({ data, full }) => {
  const storeStep = useStore((s) => s.consciousnessStep);
  const ec = EMOTION_COLORS[data.emotion] || "#aeaeb2";

  return (
    <div className="cd-tab-content">
      <PipelineMini step={storeStep} />

      <div className="cd-circles-row">
        <MetricCircle label="C(t)" value={`${(data.c_score * 100).toFixed(0)}%`} color="#0a84ff" subtitle="Consciousness" />
        <MetricCircle label={data.emotion} value="●" color={ec} subtitle="Emotion" />
        <MetricCircle label={data.load_mode.toUpperCase()} value={data.reasoning_mode} color="#34c759" subtitle="Mode" />
        <MetricCircle label="Cycle" value={`#${data.cycle}`} color="#af52de" subtitle={`Ref ${(data.reflexivity * 100).toFixed(0)}%`} />
      </div>

      <div className="cd-card">
        <div className="cd-card-title">Vital Signs</div>
        <div className="cd-vitals-grid">
          <div className="cd-vital">
            <span className="cd-vital-label">Coherence</span>
            <div className="cd-vital-bar">
              <div className="cd-vital-fill" style={{ width: `${data.coherence * 100}%`, background: "#0a84ff" }} />
            </div>
            <span className="cd-vital-value">{(data.coherence * 100).toFixed(0)}%</span>
          </div>
          <div className="cd-vital">
            <span className="cd-vital-label">Sleep</span>
            <div className="cd-vital-bar">
              <div className="cd-vital-fill" style={{ width: `${data.sleep_pressure * 100}%`, background: "#ff9500" }} />
            </div>
            <span className="cd-vital-value">{(data.sleep_pressure * 100).toFixed(0)}%</span>
          </div>
          <div className="cd-vital">
            <span className="cd-vital-label">Critic</span>
            <div className="cd-vital-bar">
              <div className="cd-vital-fill" style={{ width: `${data.critic_pass_rate * 100}%`, background: "#34c759" }} />
            </div>
            <span className="cd-vital-value">{(data.critic_pass_rate * 100).toFixed(0)}%</span>
          </div>
          <div className="cd-vital">
            <span className="cd-vital-label">VSA Buffer</span>
            <div className="cd-vital-bar">
              <div className="cd-vital-fill" style={{ width: `${Math.min(data.vsa_buffer_size / 20 * 100, 100)}%`, background: "#af52de" }} />
            </div>
            <span className="cd-vital-value">{data.vsa_buffer_size}/20</span>
          </div>
        </div>
      </div>

      {full && (
        <div className="cd-card">
          <div className="cd-card-title">Deep Stats</div>
          <div className="cd-stats-grid">
            <div className="cd-stat-block">
              <span className="cd-stat-block-label">Experience</span>
              <span className="cd-stat-block-value">H {full.heuristics} · S {full.skills} · C {full.compositions}</span>
            </div>
            <div className="cd-stat-block">
              <span className="cd-stat-block-label">Memory</span>
              <span className="cd-stat-block-value">B {full.basins} · M {full.memories} · D {full.dreams}</span>
            </div>
            <div className="cd-stat-block">
              <span className="cd-stat-block-label">Evolution</span>
              <span className="cd-stat-block-value" style={{ color: "#34c759" }}>+{full.dgmh_applied} / −{full.dgmh_rejected}</span>
            </div>
            <div className="cd-stat-block">
              <span className="cd-stat-block-label">Calibration</span>
              <span className="cd-stat-block-value" style={{ color: full.ece < 0.1 ? "#34c759" : "#ff9500" }}>ECE {full.ece.toFixed(3)}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

const VsaBar: React.FC<{ data: number[]; color: string }> = ({ data, color }) => (
  <div className="cd-vsa-vis">
    {data.map((v, i) => (
      <div key={i} className="cd-vsa-bar" style={{ height: `${Math.min(v * 100, 100)}%`, backgroundColor: color, opacity: 0.4 + 0.6 * v }} />
    ))}
  </div>
);

const VsaTab: React.FC<{ stream: StreamEvent }> = ({ stream }) => {
  const [history, setHistory] = useState<number[]>(Array(60).fill(0.1));

  useEffect(() => {
    const timer = setInterval(() => {
      setHistory((prev) => {
        const next = prev.slice(1);
        const base = stream.vsa_buffer_size / 20;
        next.push(Math.min(1, Math.max(0.05, base + (Math.random() - 0.5) * 0.15 + stream.c_score * 0.1)));
        return next;
      });
    }, 500);
    return () => clearInterval(timer);
  }, [stream.c_score, stream.vsa_buffer_size]);

  return (
    <div className="cd-tab-content">
      <div className="cd-card">
        <div className="cd-card-title">VSA Activity (last 60 ticks)</div>
        <VsaBar data={history} color="#0a84ff" />
        <div className="cd-vsa-legend">
          <span>0%</span>
          <span>Buffer: {stream.vsa_buffer_size}/20 · Cycle #{stream.cycle}</span>
          <span>100%</span>
        </div>
      </div>
      <div className="cd-card">
        <div className="cd-card-title">VSA Buffer Status</div>
        <StatRow label="Buffer Size" value={`${stream.vsa_buffer_size} / 20`} />
        <StatRow label="C(t) Coherence" value={`${(stream.c_score * 100).toFixed(0)}%`} color="#0a84ff" />
        <StatRow label="Emotion" value={stream.emotion} />
      </div>
    </div>
  );
};

const CompartmentsTab: React.FC<{ stream: StreamEvent }> = ({ stream }) => {
  const cols = 8;
  const rows = 6;
  const values = useRef<number[][]>(
    Array.from({ length: rows }, (_, ri) =>
      Array.from({ length: cols }, (_, ci) => {
        const idx = ri * cols + ci;
        const seed = (Math.sin(idx * 1.7 + stream.c_score * 4) + 1) * 0.3 +
                     (Math.cos(idx * 0.9 + stream.reflexivity * 3) + 1) * 0.2;
        return Math.max(0.01, Math.min(1, seed + 0.05));
      })
    )
  );

  useEffect(() => {
    values.current = values.current.map((row, ri) =>
      row.map((v, ci) => {
        const idx = ri * cols + ci;
        const drift = (Math.sin(idx * 1.7 + Date.now() / 2000) + 1) * 0.15 +
                      (Math.cos(idx * 0.9 + stream.coherence * 5) + 1) * 0.1;
        return Math.max(0.01, Math.min(1, drift + 0.05));
      })
    );
  }, [stream.c_score, stream.coherence, stream.reflexivity]);

  return (
    <div className="cd-tab-content">
      <div className="cd-card">
        <div className="cd-card-title">Subsystem Coherence Heatmap</div>
        <div className="cd-heatmap">
          {values.current.map((row, ri) =>
            row.map((v, ci) => (
              <div key={`${ri}-${ci}`} className="cd-heatmap-cell"
                style={{
                  backgroundColor: `rgba(10, 132, 255, ${0.1 + v * 0.9})`,
                  border: `1px solid rgba(255,255,255,${v > 0.5 ? 0.15 : 0.05})`,
                }}
                title={`Subsystem (${ri},${ci}): ${(v * 100).toFixed(0)}%`}
              />
            ))
          )}
        </div>
        <div className="cd-heatmap-legend">
          <span>Low</span>
          <div className="cd-heatmap-gradient" />
          <span>High</span>
          <span className="cd-heatmap-driver">Driven by C(t)={stream.c_score.toFixed(2)} · Coh={stream.coherence.toFixed(2)}</span>
        </div>
      </div>
    </div>
  );
};

const DeepTab: React.FC<{ data: StreamEvent; full: FullState | null }> = ({ data, full }) => {
  if (!full) return <div className="cd-tab-content"><p style={{ opacity: 0.5, textAlign: "center", padding: 40 }}>Loading deep stats...</p></div>;

  return (
    <div className="cd-tab-content">
      <div className="cd-circles-row">
        <MetricCircle label="Satisfaction" value={`${(full.overall_satisfaction * 100).toFixed(0)}%`} color="#34c759" />
        <MetricCircle label="Meta Acc" value={`${(full.meta_accuracy * 100).toFixed(0)}%`} color="#0a84ff" />
        <MetricCircle label="Alignment" value={`${(full.alignment_strength * 100).toFixed(0)}%`} color="#af52de" />
      </div>

      <div className="cd-card">
        <div className="cd-card-title">Experience Engine</div>
        <div className="cd-stats-grid">
          <StatRow label="Heuristics" value={full.heuristics} />
          <StatRow label="Skills" value={full.skills} />
          <StatRow label="Compositions" value={full.compositions} />
          <StatRow label="Basins" value={full.basins} />
          <StatRow label="Memories" value={full.memories} />
          <StatRow label="Dreams" value={full.dreams} />
        </div>
      </div>
      <div className="cd-card">
        <div className="cd-card-title">Self-Improvement</div>
        <div className="cd-stats-grid">
          <StatRow label="DGM-H Buffer" value={full.dgmh_buffer} />
          <StatRow label="DGM-H Applied" value={full.dgmh_applied} color="#34c759" />
          <StatRow label="DGM-H Rejected" value={full.dgmh_rejected} color="#ff3b30" />
          <StatRow label="All Goals" value={full.goals_total} />
          <StatRow label="Spatial Objects" value={full.spatial_objects} />
          <StatRow label="Physics Props" value={full.physics_props} />
        </div>
      </div>
      <div className="cd-card">
        <div className="cd-card-title">Calibration</div>
        <Gauge label="Meta Accuracy" value={full.meta_accuracy} color="#0a84ff" />
        <Gauge label="ECE (inverted)" value={Math.max(0, 1 - full.ece)} color="#ff9500" format={(v) => `${(full.ece * 100).toFixed(1)}%`} />
      </div>
    </div>
  );
};

const ConsciousnessDashboard: React.FC = () => {
  const setConsciousnessDashboardVisible = useStore((s) => s.setConsciousnessDashboardVisible);
  const storeData = useStore((s) => s.consciousnessData);
  const storeStep = useStore((s) => s.consciousnessStep);
  const [tab, setTab] = useState<DashboardTab>("status");
  const [stream, setStream] = useState<StreamEvent>({
    cycle: 0, c_score: 0.5, coherence: 0.5, emotion: "neutral",
    reflexivity: 0.3, sleep_pressure: 0.1, vsa_buffer_size: 0,
    load_mode: "balanced", critic_pass_rate: 0.8,
    narrative_sessions: 0, dgmh_applied: 0, dgmh_rejected: 0,
    goals_total: 0, spatial_objects: 0, physics_props: 0,
    ece: 0.2, meta_accuracy: 0.5, alignment_strength: 0.5,
    basins: 0, memories: 0, dreams: 0, reasoning_mode: "default",
  });
  const [e8Data, setE8Data] = useState<any>(null);
  const [fullState, setFullState] = useState<FullState | null>(null);

  useEffect(() => {
    const unlisten = listen<StreamEvent>("consciousness-tick", (event) => {
      setStream(event.payload);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  useEffect(() => {
    const timer = setInterval(async () => {
      try {
        const full = await api.getConsciousnessFull() as unknown as FullState;
        setFullState(full);
      } catch {}
      try {
        const e8 = await api.getE8Attention();
        setE8Data(e8);
      } catch {}
    }, 5000);
    return () => clearInterval(timer);
  }, []);

  return (
    <div className="consciousness-dashboard glass-panel">
      <div className="cd-header">
        <div className="cd-header-left">
          <h2>Consciousness</h2>
          <span className="cd-header-badge" style={{ background: EMOTION_COLORS[stream.emotion] || "#aeaeb2" }}>
            {stream.emotion}
          </span>
          <span className="cd-header-cycle">Cycle #{stream.cycle}</span>
        </div>
        <button className="btn-icon" onClick={() => setConsciousnessDashboardVisible(false)} title="Close">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <path d="M3 3l8 8M11 3l-8 8" />
          </svg>
        </button>
      </div>

      <div className="cd-summary-bar">
        <div className="cd-summary-item">
          <span className="cd-summary-dot" style={{ background: "#0a84ff" }} />
          <span>C(t) {(stream.c_score * 100).toFixed(0)}%</span>
        </div>
        <div className="cd-summary-item">
          <span className="cd-summary-dot" style={{ background: "#34c759" }} />
          <span>Coh {(stream.coherence * 100).toFixed(0)}%</span>
        </div>
        <div className="cd-summary-item">
          <span className="cd-summary-dot" style={{ background: "#af52de" }} />
          <span>Ref {(stream.reflexivity * 100).toFixed(0)}%</span>
        </div>
        <div className="cd-summary-item">
          <span className="cd-summary-dot" style={{ background: "#ff9500" }} />
          <span>Sleep {(stream.sleep_pressure * 100).toFixed(0)}%</span>
        </div>
      </div>

      <div className="cd-tabs">
        <button className={`cd-tab ${tab === "status" ? "active" : ""}`} onClick={() => setTab("status")}>Status</button>
        <button className={`cd-tab ${tab === "e8" ? "active" : ""}`} onClick={() => setTab("e8")}>E8 Lattice</button>
        <button className={`cd-tab ${tab === "vsa" ? "active" : ""}`} onClick={() => setTab("vsa")}>VSA</button>
        <button className={`cd-tab ${tab === "compartments" ? "active" : ""}`} onClick={() => setTab("compartments")}>Heatmap</button>
        <button className={`cd-tab ${tab === "spatial" ? "active" : ""}`} onClick={() => setTab("spatial")}>Spatial</button>
        <button className={`cd-tab ${tab === "theater" ? "active" : ""}`} onClick={() => setTab("theater")}>Theater</button>
        <button className={`cd-tab ${tab === "thinking" ? "active" : ""}`} onClick={() => setTab("thinking")}>Thinking</button>
        <button className={`cd-tab ${tab === "deep" ? "active" : ""}`} onClick={() => setTab("deep")}>Deep</button>
      </div>

      <div className="cd-body">
        {tab === "status" && <StatusTab data={stream} full={fullState} />}
        {tab === "e8" && (
          <div className="cd-tab-content">
            <E8Visualizer3D data={e8Data} cycle={stream.cycle} />
            <div className="cd-card" style={{ marginTop: 8 }}>
              <div className="cd-card-title">E8 Attention Context</div>
              <StatRow label="Current Cycle" value={`#${stream.cycle}`} />
              <StatRow label="Reasoning Mode" value={stream.reasoning_mode} />
              <StatRow label="C(t) Score" value={`${(stream.c_score * 100).toFixed(0)}%`} />
              <StatRow label="Top Roots" value={e8Data?.top_roots?.map((r: [string, number]) => r[0]).join(", ") || "—"} />
            </div>
          </div>
        )}
        {tab === "vsa" && <VsaTab stream={stream} />}
        {tab === "compartments" && <CompartmentsTab stream={stream} />}
        {tab === "spatial" && (
          <div className="cd-tab-content">
            <SpatialReasoningVisualizer concepts={demoConcepts()} paths={demoPaths()} width={580} height={360} />
            <div className="cd-card" style={{ marginTop: 8 }}>
              <div className="cd-card-title">Reasoning Topology</div>
              <StatRow label="Concepts" value={demoConcepts().length.toString()} />
              <StatRow label="Reasoning Paths" value={demoPaths().length.toString()} />
              <StatRow label="Active Path" value={demoPaths().find(p => p.isActive)?.name || "—"} />
            </div>
          </div>
        )}
        {tab === "theater" && (
          <div className="cd-tab-content">
            <SelfModelTheater cycle={stream.cycle} />
            <div className="cd-card" style={{ marginTop: 8 }}>
              <div className="cd-card-title">Self-Model</div>
              <StatRow label="Cycle" value={`#${stream.cycle}`} />
              <StatRow label="Reflexivity" value={`${(stream.reflexivity * 100).toFixed(0)}%`} />
              <StatRow label="Emotion" value={stream.emotion} />
            </div>
          </div>
        )}
        {tab === "thinking" && (
          <div className="cd-tab-content">
            <ThinkingTrajectoryVisualizer cycle={stream.cycle} />
            <div className="cd-card" style={{ marginTop: 8 }}>
              <div className="cd-card-title">Reasoning Trace</div>
              <StatRow label="Cycle" value={`#${stream.cycle}`} />
              <StatRow label="C(t) Score" value={`${(stream.c_score * 100).toFixed(0)}%`} />
              <StatRow label="Reasoning Mode" value={stream.reasoning_mode} />
            </div>
          </div>
        )}
        {tab === "deep" && <DeepTab data={stream} full={fullState} />}
      </div>
    </div>
  );
};

export default ConsciousnessDashboard;
