import React, { useState } from "react";
import ManagementLayout from "../components/ManagementLayout";

const AgentsPanel: React.FC = () => (
  <div className="mg-agent-grid mg-slide-in">
    {[
      { name: "Researcher", color: "linear-gradient(135deg,#007aff,#5856d6)", initial: "R", type: "research", desc: "Deep web research, paper analysis, and information synthesis with citation tracking.", tasks: 23, success: 94, latency: "1.2s" },
      { name: "Coder", color: "linear-gradient(135deg,#ff9500,#ff3b30)", initial: "C", type: "code", desc: "Full-stack development with auto-testing, code review, and refactoring support.", tasks: 47, success: 91, latency: "3.8s" },
      { name: "Memory Keeper", color: "linear-gradient(135deg,#34c759,#30b350)", initial: "M", type: "storage", desc: "Knowledge base curator, embedding maintenance, and conversation evolution mining.", tasks: 156, success: 99, latency: "0.4s" },
      { name: "Shield", color: "linear-gradient(135deg,#5856d6,#af52de)", initial: "S", type: "security", desc: "Permission enforcement, privacy filtering, secret scanning, and sandbox isolation.", tasks: 89, success: 100, latency: "0.1s" },
    ].map((a) => (
      <div className="mg-agent-card" key={a.name}>
        <div className="card-header">
          <div className="card-avatar" style={{ background: a.color }}>{a.initial}</div>
          <span className="card-name">{a.name}</span>
          <span className="card-type">{a.type}</span>
        </div>
        <div className="card-desc">{a.desc}</div>
        <div className="card-stats">
          <span>⚡ {a.tasks} tasks</span>
          <span>✓ {a.success}% success</span>
          <span>⏱ {a.latency} avg</span>
        </div>
      </div>
    ))}
  </div>
);

const ModelsPanel: React.FC = () => (
  <div className="mg-provider-grid mg-slide-in">
    {[
      { name: "GPT-4o", provider: "OpenAI · 128k context", color: "#1a1a2e", dot: "online" },
      { name: "Claude 3.5 Sonnet", provider: "Anthropic · 200k context", color: "#10a37f", dot: "online" },
      { name: "Gemini 2.0 Pro", provider: "Google · 1M context", color: "#4285f4", dot: "online" },
      { name: "Groq Llama-3", provider: "Groq · 8k context", color: "#000", dot: "busy" },
      { name: "Mistral Large", provider: "Mistral AI · 32k context", color: "#6c47ff", dot: "offline" },
    ].map((m) => (
      <div className="mg-provider-card" key={m.name}>
        <div className="mg-provider-icon" style={{ background: m.color }}>{m.name[0]}</div>
        <div className="mg-provider-info">
          <div className="mg-provider-name">{m.name}</div>
          <div className="mg-provider-status">{m.provider}</div>
        </div>
        <div className={`mg-provider-dot ${m.dot}`} />
      </div>
    ))}
  </div>
);

const ProvidersPanel: React.FC = () => (
  <div className="mg-provider-grid mg-slide-in">
    {[
      { name: "Anthropic", url: "api.anthropic.com · 45ms", color: "#10a37f", dot: "online" },
      { name: "OpenAI", url: "api.openai.com · 62ms", color: "#1a1a2e", dot: "online" },
      { name: "Google Gemini", url: "generativelanguage.googleapis.com · 88ms", color: "#4285f4", dot: "online" },
      { name: "Groq", url: "api.groq.com · 120ms", color: "#000", dot: "busy" },
      { name: "Mistral", url: "api.mistral.ai · unreachable", color: "#6c47ff", dot: "offline" },
      { name: "Pollinations", url: "pollinations.ai · free tier", color: "#f97316", dot: "online" },
    ].map((p) => (
      <div className="mg-provider-card" key={p.name}>
        <div className="mg-provider-icon" style={{ background: p.color }}>{p.name[0]}</div>
        <div className="mg-provider-info">
          <div className="mg-provider-name">{p.name}</div>
          <div className="mg-provider-status">{p.url}</div>
        </div>
        <div className={`mg-provider-dot ${p.dot}`} />
      </div>
    ))}
  </div>
);

const MemoryPanel: React.FC = () => (
  <div className="mg-slide-in">
    <div className="mg-memory-stats">
      {[
        { value: "2,451", label: "Total Nodes" },
        { value: "892", label: "Conversations" },
        { value: "1,847", label: "Embeddings" },
        { value: "19", label: "Relations" },
      ].map((s) => (
        <div className="mg-memory-stat" key={s.label}>
          <div className="stat-value">{s.value}</div>
          <div className="stat-label">{s.label}</div>
        </div>
      ))}
    </div>
    <div className="mg-provider-grid" style={{ gridTemplateColumns: "1fr 1fr" }}>
      {[
        { icon: "📄", name: "Recent Conversations", desc: "Last 7 days · 23 records" },
        { icon: "🏷️", name: "Tags", desc: "e8_state, specialist, concept · 156 tags" },
        { icon: "🔗", name: "Graph Edges", desc: "HyperCube VSA · 3.2k connections" },
        { icon: "🧠", name: "Evolution Records", desc: "RecurringError, StrategyDiscovery · 47 patterns" },
      ].map((t) => (
        <div className="mg-tool-item" key={t.name}>
          <div className="tool-icon" style={{ background: "rgba(255,255,255,0.7)", fontSize: 18 }}>{t.icon}</div>
          <div className="tool-info"><div className="tool-name">{t.name}</div><div className="tool-desc">{t.desc}</div></div>
        </div>
      ))}
    </div>
  </div>
);

const SkillsPanel: React.FC = () => (
  <div className="mg-empty-state mg-slide-in">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round"><path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2z"/></svg>
    <p>Skills let you extend NeoTrix with custom capabilities. Registered skills appear here with their maturity level (1–6) and trigger conditions.</p>
  </div>
);

const ToolsPanel: React.FC = () => (
  <div className="mg-tool-grid mg-slide-in">
    {[
      { icon: "🛠", name: "MCP Registry", desc: "11 registered MCP servers · Stdio/HTTP/WS/SSE" },
      { icon: "🔍", name: "Web Search", desc: "AnySearch provider · fallback to web scrape" },
      { icon: "📂", name: "File System", desc: "Read/write/search · sandbox enforced" },
      { icon: "▶", name: "Code Execution", desc: "Docker sandbox · 5 runtimes" },
      { icon: "🔐", name: "Secret Scanner", desc: "13 regex patterns · Gitleaks integration" },
      { icon: "🌐", name: "Browser", desc: "CamoFox anti-detection · stealth mode" },
    ].map((t) => (
      <div className="mg-tool-item" key={t.name}>
        <div className="tool-icon">{t.icon}</div>
        <div className="tool-info"><div className="tool-name">{t.name}</div><div className="tool-desc">{t.desc}</div></div>
      </div>
    ))}
  </div>
);

const SandboxPanel: React.FC = () => (
  <div className="mg-empty-state mg-slide-in">
    <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" strokeLinecap="round"><rect x="2" y="3" width="20" height="14" rx="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
    <p>Sandbox environments for safe code execution. Configure runtimes, resource limits, and network policies per environment.</p>
  </div>
);

const ToggleRow: React.FC<{ label: string; desc: string; defaultOn?: boolean }> = ({ label, desc, defaultOn }) => {
  const [on, setOn] = useState(defaultOn ?? false);
  return (
    <div className="mg-setting-row">
      <div><div className="mg-setting-label">{label}</div><div className="mg-setting-desc">{desc}</div></div>
      <div className={`mg-toggle${on ? " on" : ""}`} onClick={() => setOn(!on)} />
    </div>
  );
};

const IdentityPanel: React.FC = () => (
  <div className="mg-settings-list mg-slide-in">
    <ToggleRow label="Dev Identity" desc="Default persona for development tasks" defaultOn />
    <ToggleRow label="Research Identity" desc="Academic persona for paper analysis" />
    <ToggleRow label="Anonymous Mode" desc="Strip all identifying information from requests" />
  </div>
);

const PrivacyPanel: React.FC = () => (
  <div className="mg-settings-list mg-slide-in">
    <ToggleRow label="PII Scrubbing" desc="Automatically redact emails, phones, SSNs" defaultOn />
    <ToggleRow label="Local-Only Mode" desc="Never send data to remote providers" />
    <ToggleRow label="Conversation Logging" desc="Store conversations for evolution mining" defaultOn />
  </div>
);

const InsightsPanel: React.FC = () => (
  <div className="mg-slide-in">
    <div className="mg-memory-stats">
      {[
        { value: "89.2%", label: "Avg Success Rate" },
        { value: "347", label: "E8 Iterations" },
        { value: "1.8s", label: "Avg Response" },
        { value: "24", label: "Active Capabilities" },
      ].map((s) => (
        <div className="mg-memory-stat" key={s.label}>
          <div className="stat-value">{s.value}</div>
          <div className="stat-label">{s.label}</div>
        </div>
      ))}
    </div>
    <div className="mg-insight-grid">
      <div className="mg-insight-card">
        <h4>Token Usage (7d)</h4>
        <div className="mg-chart-area">
          {[60,85,45,90,70,55,30].map((h,i) => <div key={i} className="mg-chart-bar" style={{ height: `${h}%` }} />)}
        </div>
      </div>
      <div className="mg-insight-card">
        <h4>Provider Health</h4>
        <div className="mg-chart-area">
          {[95,88,92,60,0,78].map((h,i) => <div key={i} className="mg-chart-bar" style={{ height: `${h}%`, background: h > 80 ? "#34c759" : h > 0 ? "#ff9500" : "#ff3b30" }} />)}
        </div>
      </div>
      <div className="mg-insight-card">
        <h4>Capability Maturity</h4>
        <div className="mg-chart-area">
          {[100,83,66,50,33,16].map((h,i) => <div key={i} className="mg-chart-bar" style={{ height: `${h}%`, background: "#5856d6" }} />)}
        </div>
      </div>
      <div className="mg-insight-card">
        <h4>Memory Growth</h4>
        <div className="mg-chart-area">
          {[20,35,42,58,71,85,100].map((h,i) => <div key={i} className="mg-chart-bar" style={{ height: `${h}%`, background: "#34c759" }} />)}
        </div>
      </div>
    </div>
  </div>
);

const SettingsPanel: React.FC = () => (
  <div className="mg-settings-list mg-slide-in">
    <ToggleRow label="Appearance" desc="Light / Dark / System theme" defaultOn />
    <ToggleRow label="Auto-update" desc="Automatically check for updates on startup" defaultOn />
    <ToggleRow label="Telemetry" desc="Send anonymous usage data to improve NeoTrix" />
    <ToggleRow label="Notifications" desc="Show desktop notifications for agent events" defaultOn />
    <ToggleRow label="E8 Auto-iteration" desc="Background reasoning engine self-improvement" defaultOn />
  </div>
);

const panels: Record<string, React.FC> = {
  agents: AgentsPanel,
  models: ModelsPanel,
  providers: ProvidersPanel,
  memory: MemoryPanel,
  skills: SkillsPanel,
  tools: ToolsPanel,
  sandbox: SandboxPanel,
  identity: IdentityPanel,
  privacy: PrivacyPanel,
  insights: InsightsPanel,
  settings: SettingsPanel,
};

const ManagePage: React.FC = () => (
  <ManagementLayout>
    {(tab) => {
      const Panel = panels[tab] || panels.agents;
      return <Panel />;
    }}
  </ManagementLayout>
);

export default ManagePage;
