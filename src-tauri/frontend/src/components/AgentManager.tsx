import React, { useState } from "react";
import GlassPanel from "./GlassPanel";

interface AgentRuntime {
  id: string;
  name: string;
  description: string;
  harness: string;
  status: "running" | "idle" | "error";
  model: string;
  provider: string;
}

const BUILTIN_AGENTS: AgentRuntime[] = [
  { id: "neotrix-e8", name: "NeoTrix E8", description: "E8 64-state reasoning engine", harness: "native", status: "running", model: "e8-v1", provider: "internal" },
  { id: "claude-code", name: "Claude Code", description: "Anthropic Claude coding agent", harness: "claude-sdk", status: "idle", model: "claude-sonnet-4", provider: "anthropic" },
  { id: "codex", name: "Codex", description: "OpenAI Codex agent", harness: "codex-native", status: "idle", model: "gpt-4.1", provider: "openai" },
  { id: "opencode", name: "OpenCode", description: "Open-source coding agent", harness: "opencode", status: "idle", model: "deepseek-v4", provider: "openrouter" },
];

const DEFAULT_YAML = `name: my-custom-agent
prompt: You are a specialized research assistant.
executor:
  harness: claude-sdk
tools:
  web_search:
    type: function
    callable: search
  code_review:
    type: agent
    prompt: Review code changes and provide feedback.`;

const AGENT_ICONS: Record<string, string> = {
  native: "\uD83E\uDDE0",
  "claude-sdk": "\uD83E\uDD16",
  "codex-native": "\u26A1",
  opencode: "\uD83D\uDD25",
};

const AGENT_POLICIES: Array<[string, boolean]> = [
  ["Shell Access", true],
  ["File Write", true],
  ["Network", false],
  ["Budget Cap", true],
];

function AgentDetail({ agent }: { agent: AgentRuntime }) {
  return (
    <div className="agent-detail-panel">
      <div className="lg-card lg-card-padded">
        <div className="lg-label">Status</div>
        <div className="agent-detail-status">
          <span className={`lg-badge ${agent.status === "running" ? "lg-badge-success" : ""}`}>
            {agent.status === "running" ? "\u25CF Running" : "\u25CB Idle"}
          </span>
          <span className="lg-badge">{agent.harness}</span>
        </div>
      </div>
      <div className="lg-card lg-card-padded">
        <div className="lg-label">Model</div>
        <div className="agent-detail-value">{agent.model}</div>
        <div className="agent-detail-meta">via {agent.provider}</div>
      </div>
      <div className="lg-card lg-card-padded">
        <div className="lg-label">Policies</div>
        <div className="agent-policy-list">
          {AGENT_POLICIES.map(([name, on]) => (
            <div key={name} className="agent-policy-row">
              <span className="agent-policy-name">{name}</span>
              <button className={`lg-toggle ${on ? "active" : ""}`}>
                <span className="lg-toggle-knob" />
              </button>
            </div>
          ))}
        </div>
      </div>
      <div className="lg-card lg-card-padded">
        <div className="lg-label">Session Stats</div>
        <div className="agent-stats-text">
          <div>Context: 1.2k / 8k tokens</div>
          <div>Tools called: 14</div>
          <div>Duration: 12m 34s</div>
        </div>
      </div>
    </div>
  );
}

const AgentManager: React.FC = () => {
  const [agents, setAgents] = useState<AgentRuntime[]>(BUILTIN_AGENTS);
  const [activeAgent, setActiveAgent] = useState("neotrix-e8");
  const [showYamlEditor, setShowYamlEditor] = useState(false);
  const [yamlContent, setYamlContent] = useState(DEFAULT_YAML);

  const toggleAgent = (id: string) => {
    setAgents((prev) => prev.map((a) =>
      a.id === id ? { ...a, status: a.status === "running" ? "idle" as const : "running" as const } : a
    ));
  };

  const active = agents.find((a) => a.id === activeAgent);
  const runningCount = agents.filter((a) => a.status === "running").length;

  return (
    <div className="lg-split-panel">
      <GlassPanel
        variant="strong"
        header={<><span>\uD83E\uDD16 Agent Orchestrator</span><span className="lg-badge">{runningCount} active</span></>}
        className="lg-flex-col"
      >
        <div className="lg-list lg-flex-1 lg-scroll-auto">
          {agents.map((agent) => (
            <div
              key={agent.id}
              className={`lg-list-item ${activeAgent === agent.id ? "active" : ""}`}
              onClick={() => setActiveAgent(agent.id)}
            >
              <div className="lg-list-item-content">
                <span className="agent-icon-lg">{AGENT_ICONS[agent.harness] || "\uD83E\uDD16"}</span>
                <div className="agent-meta-col">
                  <div className="agent-name">{agent.name}</div>
                  <div className="agent-desc">{agent.description}</div>
                </div>
              </div>
              <div className="agent-list-actions">
                <span className={`lg-badge ${agent.status === "running" ? "lg-badge-success" : ""}`}>
                  {agent.status === "running" ? "\u25CF Running" : "\u25CB Idle"}
                </span>
                <span className="lg-badge agent-model-badge">{agent.model}</span>
                <button
                  className={`lg-btn lg-btn-icon ${agent.status === "running" ? "lg-btn-primary" : "lg-btn-ghost"}`}
                  onClick={(e) => { e.stopPropagation(); toggleAgent(agent.id); }}
                  title={agent.status === "running" ? "Stop agent" : "Start agent"}
                >
                  {agent.status === "running" ? "\u23F9" : "\u25B6"}
                </button>
              </div>
            </div>
          ))}
        </div>
        <div className="lg-divider" />
        <div className="agent-actions-row">
          <button className="lg-btn lg-btn-primary" onClick={() => setShowYamlEditor(!showYamlEditor)}>
            {showYamlEditor ? "\u2715 Close" : "\uff0b New Agent"}
          </button>
          <button className="lg-btn">\uD83D\uDCCB Import YAML</button>
        </div>
        {showYamlEditor && (
          <div className="lg-glass yaml-editor-panel">
            <div className="yaml-editor-label">Agent YAML Definition</div>
            <textarea
              className="lg-input yaml-editor-textarea"
              value={yamlContent}
              onChange={(e) => setYamlContent(e.target.value)}
            />
            <div className="yaml-editor-actions">
              <button className="lg-btn lg-btn-primary">\u25B6 Deploy</button>
              <button className="lg-btn">Save as Preset</button>
            </div>
          </div>
        )}
      </GlassPanel>

      <GlassPanel variant="clear" header="Agent Detail" className="lg-side-panel">
        {active ? <AgentDetail agent={active} /> : <div className="lg-empty"><div className="lg-empty-text">Select an agent</div></div>}
      </GlassPanel>
    </div>
  );
};

AgentManager.displayName = "AgentManager";

export default AgentManager;
