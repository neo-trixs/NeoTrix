import React, { useState } from "react";
import GlassPanel from "./GlassPanel";

interface SandboxInstance {
  id: string;
  name: string;
  runtime: "python" | "node" | "rust" | "linux";
  status: "running" | "stopped" | "error";
  memory: string;
  cpu: string;
  uptime: string;
  network: "isolated" | "bridge" | "host";
}

const MOCK_INSTANCES: SandboxInstance[] = [
  { id: "sb-1", name: "code-exec-1", runtime: "python", status: "running", memory: "256MB", cpu: "0.5", uptime: "12m 34s", network: "isolated" },
  { id: "sb-2", name: "data-pipeline", runtime: "node", status: "running", memory: "512MB", cpu: "1.0", uptime: "3h 22m", network: "bridge" },
  { id: "sb-3", name: "rust-compile", runtime: "rust", status: "stopped", memory: "1GB", cpu: "2.0", uptime: "0m", network: "isolated" },
];

const SANDBOX_PROVIDERS = [
  { id: "local-docker" as const, icon: "\uD83D\uDC33", label: "local docker" },
  { id: "apple-container" as const, icon: "\uD83C\uDF4E", label: "apple container" },
  { id: "modal" as const, icon: "\u26A1", label: "modal" },
  { id: "e2b" as const, icon: "\uD83D\uDD12", label: "e2b" },
];

const RUNTIME_IMAGES: Record<string, { image: string; icon: string }> = {
  python: { image: "python:3.11-slim", icon: "\uD83D\uDC0D" },
  node: { image: "node:18-alpine", icon: "\uD83D\uDFE2" },
  rust: { image: "rust:latest", icon: "\uD83E\uDD80" },
  linux: { image: "ubuntu:22.04", icon: "\uD83D\uDC27" },
};

const RESOURCE_LABELS: Array<[string, keyof SandboxInstance]> = [
  ["Memory", "memory"],
  ["CPU", "cpu"],
  ["Uptime", "uptime"],
  ["Network", "network"],
];

const SECURITY_SETTINGS: Array<[string, string]> = [
  ["Network isolation", "enabled"],
  ["File system", "read-only"],
  ["Process limits", "32"],
  ["Timeout", "30m"],
];

function SandboxResourceGrid({ sb }: { sb: SandboxInstance }) {
  return (
    <div className="lg-card lg-card-padded">
      <div className="lg-label">Resources</div>
      <div className="resource-grid">
        {RESOURCE_LABELS.map(([k, v]) => (
          <div key={k} className="resource-row">
            <span className="resource-label">{k}</span>
            <span className="resource-value">{String(sb[v])}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function SandboxSecurityCard({ sb }: { sb: SandboxInstance }) {
  return (
    <div className="lg-card lg-card-padded">
      <div className="lg-label">Security</div>
      <div className="security-grid">
        {SECURITY_SETTINGS.map(([k, v]) => (
          <div key={k} className="security-row">
            <span>{k}</span>
            <span className="lg-badge">{v}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

const SandboxManager: React.FC = () => {
  const [instances, setInstances] = useState<SandboxInstance[]>(MOCK_INSTANCES);
  const [selectedSandbox, setSelectedSandbox] = useState("sb-1");
  const [provider, setProvider] = useState<"local-docker" | "apple-container" | "modal" | "e2b">("local-docker");

  const toggleSandbox = (id: string) => {
    setInstances((prev) => prev.map((s) =>
      s.id === id ? { ...s, status: s.status === "running" ? "stopped" as const : "running" as const } : s
    ));
  };

  const selected = instances.find((i) => i.id === selectedSandbox);
  const runningCount = instances.filter((i) => i.status === "running").length;

  return (
    <div className="lg-split-panel">
      <GlassPanel
        variant="strong"
        header={<><span>\uD83D\uDCE6 Sandbox Manager</span><span className="lg-badge">{runningCount} active</span></>}
        className="lg-flex-col"
      >
        <div className="sandbox-provider-bar">
          {SANDBOX_PROVIDERS.map((p) => (
            <button
              key={p.id}
              className={`lg-btn ${provider === p.id ? "lg-btn-primary" : ""}`}
              onClick={() => setProvider(p.id)}
            >
              {p.icon} {p.label}
            </button>
          ))}
        </div>
        <div className="lg-list lg-flex-1 lg-scroll-auto">
          {instances.map((sb) => {
            const rt = RUNTIME_IMAGES[sb.runtime];
            return (
              <div
                key={sb.id}
                className={`lg-list-item ${selectedSandbox === sb.id ? "active" : ""}`}
                onClick={() => setSelectedSandbox(sb.id)}
              >
                <div className="lg-list-item-content">
                  <span className="sandbox-runtime-icon">{rt.icon}</span>
                  <div className="sandbox-meta-col">
                    <div className="sandbox-name">{sb.name}</div>
                    <div className="sandbox-image-line">{rt.image} \u00B7 {sb.network} net</div>
                  </div>
                </div>
                <div className="sandbox-list-actions">
                  <span className={`lg-badge ${sb.status === "running" ? "lg-badge-success" : sb.status === "error" ? "lg-badge-danger" : ""}`}>
                    {sb.status === "running" ? "\u25CF" : "\u25CB"} {sb.status}
                  </span>
                  <button
                    className={`lg-btn lg-btn-icon ${sb.status === "running" ? "lg-btn-primary" : "lg-btn-ghost"}`}
                    onClick={(e) => { e.stopPropagation(); toggleSandbox(sb.id); }}
                  >
                    {sb.status === "running" ? "\u23F9" : "\u25B6"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
        <div className="lg-divider" />
        <div className="sandbox-bottom-actions">
          <button className="lg-btn lg-btn-primary">\uff0b New Sandbox</button>
          <button className="lg-btn">\uD83D\uDCCA Stats</button>
        </div>
      </GlassPanel>

      <GlassPanel variant="clear" header="Sandbox Detail" className="lg-side-panel">
        {selected ? (
          <div className="sandbox-detail-column">
            <div className="lg-card lg-card-padded">
              <div className="lg-label">Runtime</div>
              <div className="sandbox-runtime-display">
                {RUNTIME_IMAGES[selected.runtime].icon} {selected.runtime}
              </div>
              <span className="lg-badge">{RUNTIME_IMAGES[selected.runtime].image}</span>
            </div>
            <SandboxResourceGrid sb={selected} />
            <SandboxSecurityCard sb={selected} />
            <div className="lg-card lg-card-padded">
              <div className="lg-label">Agent Access</div>
              <div className="sandbox-agent-badges">
                <span className="lg-badge lg-badge-primary">claude-code</span>
                <span className="lg-badge lg-badge-primary">codex</span>
                <span className="lg-badge lg-badge-primary">+2</span>
              </div>
            </div>
            <button className="lg-btn sandbox-destroy-btn">
              \uD83D\uDDD1 Destroy Sandbox
            </button>
          </div>
        ) : (
          <div className="lg-empty"><div className="lg-empty-text">Select a sandbox</div></div>
        )}
      </GlassPanel>
    </div>
  );
};

SandboxManager.displayName = "SandboxManager";

export default SandboxManager;
