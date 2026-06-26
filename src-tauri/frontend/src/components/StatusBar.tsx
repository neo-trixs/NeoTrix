import React, { useState, useRef, useEffect } from "react";
import { useStore } from "../store";

const EMOTION_COLORS: Record<string, string> = {
  neutral: "#aeaeb2", happy: "#34c759", sad: "#0a84ff",
  angry: "#ff3b30", curious: "#ff9500", anxious: "#af52de",
};

const PROVIDER_COLORS: Record<string, string> = {
  anthropic: "#d97757", openai: "#19c37d", gemini: "#4285f4",
  ollama: "#000", groq: "#f97316", deepseek: "#4f46e5",
};

const StatusBar: React.FC = () => {
  const statusText = useStore((s) => s.statusText);
  const agentBusy = useStore((s) => s.agentBusy);
  const sessionIndex = useStore((s) => s.activeSessionIndex);
  const sessionCount = useStore((s) => s.sessions.length);
  const currentModel = useStore((s) => s.currentModel);
  const contextUsage = useStore((s) => s.contextUsage);
  const agentMode = useStore((s) => s.agentMode);
  const currentGitBranch = useStore((s) => s.currentGitBranch);
  const setAgentMode = useStore((s) => s.setAgentMode);
  const showTerminal = useStore((s) => s.showTerminal);
  const setShowTerminal = useStore((s) => s.setShowTerminal);
  const setShowSettings = useStore((s) => s.setShowSettings);
  const settings = useStore((s) => s.settings);
  const setSettings = useStore((s) => s.setSettings);
  const consciousnessData = useStore((s) => s.consciousnessData);
  const setConsciousnessDashboardVisible = useStore((s) => s.setConsciousnessDashboardVisible);

  const [modelMenuOpen, setModelMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const MODE_LABEL: Record<string, string> = { chat: "Chat", plan: "Plan", agent: "Agent" };
  const MODE_CYCLE: Record<string, string> = { chat: "plan", plan: "agent", agent: "chat" };

  const ctxColor = contextUsage > 80 ? "var(--danger)" : contextUsage > 60 ? "#f5a623" : "var(--mac-text-muted)";
  const modelColor = PROVIDER_COLORS[currentModel?.provider || ""] || "var(--mac-text-muted)";

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setModelMenuOpen(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const estimatedTokens = Math.round(contextUsage * (currentModel?.context_length || 100000) / 100);

  return (
    <div className={`status-bar ${agentBusy ? "busy" : ""}`}>
      <div className="status-left">
        <span className={`status-dot ${agentBusy ? "busy" : "idle"}`} />
        <span className="status-text">{statusText}</span>
        <span className="status-sep" />
        <span className="status-git">{currentGitBranch}</span>
        <span className="status-sep" />
        <button
          className={`status-btn ${showTerminal ? 'active' : ''}`}
          onClick={() => setShowTerminal(!showTerminal)}
          title="Toggle Terminal (Cmd+`)"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="4 5, 7 8, 4 11" />
            <line x1="9" y1="10" x2="11" y2="10" />
          </svg>
        </button>
        <span className="status-sep" />
        {/* Model badge — clickable like Codex/Claude Code */}
        <div className="status-model-badge" ref={menuRef} style={{ position: "relative" }}>
          <button className="status-model-btn" onClick={() => setModelMenuOpen(!modelMenuOpen)}>
            <span className="model-dot" style={{ background: modelColor }} />
            <span className="model-label">{currentModel?.name || "\u2014"}</span>
            <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M2 3l2 2 2-2" />
            </svg>
          </button>
          {modelMenuOpen && (
            <div className="status-model-dropdown">
              <div className="status-model-dropdown-item">
                <span className="dropdown-label">Model</span>
                <span className="dropdown-value">{currentModel?.name || "—"}</span>
              </div>
              <div className="status-model-dropdown-item">
                <span className="dropdown-label">Provider</span>
                <span className="dropdown-value">{currentModel?.provider || "—"}</span>
              </div>
              <div className="status-model-dropdown-item">
                <span className="dropdown-label">Context</span>
                <span className="dropdown-value">{(currentModel?.context_length || 100000).toLocaleString()} tokens</span>
              </div>
              <div className="status-model-dropdown-divider" />
              <button className="status-model-dropdown-action" onClick={() => { setModelMenuOpen(false); setShowSettings(true); }}>
                Change model...
              </button>
            </div>
          )}
        </div>
        <span className="status-sep" />
        {/* Context usage bar — Cursor/Claude Code style visual indicator */}
        <div className="status-ctx-bar" title={`${estimatedTokens.toLocaleString()} / ${(currentModel?.context_length || 100000).toLocaleString()} tokens`}>
          <div className="status-ctx-track">
            <div className="status-ctx-fill" style={{ width: `${contextUsage}%`, background: ctxColor }} />
          </div>
          <span className="status-ctx-label">{estimatedTokens >= 1000 ? `${(estimatedTokens / 1000).toFixed(1)}k` : estimatedTokens}</span>
        </div>
      </div>
      <div className="status-right">
        <button className="status-mode-btn" onClick={() => setAgentMode(MODE_CYCLE[agentMode] as any)}>
          {MODE_LABEL[agentMode]}
        </button>
        <span className="status-sep" />
        <span className="status-item">S{sessionIndex + 1}/{sessionCount}</span>
        <span className="status-sep" />
        {consciousnessData && (
          <>
            <button
              className="status-brain-btn"
              onClick={() => setConsciousnessDashboardVisible(true)}
              title="Consciousness Dashboard"
            >
              <span className="brain-dot" style={{ background: EMOTION_COLORS[consciousnessData.emotion] || "#aeaeb2" }} />
              <span className="brain-cscore">C(t) {(consciousnessData.c_score * 100).toFixed(0)}%</span>
            </button>
            <span className="status-sep" />
          </>
        )}
        <button className="status-btn" onClick={() => setSettings({ ...settings, theme: settings.theme === "dark" ? "light" : "dark" })} title="Toggle theme">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            {settings.theme === "dark" ? (
              <path d="M7 1a6 6 0 100 12 4.5 4.5 0 010-9 4 4 0 010-3z" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            ) : (
              <>
                <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.3" />
                <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </>
            )}
          </svg>
        </button>
        <span className="status-sep" />
        <button className="status-btn" onClick={() => setShowSettings(true)} title="Settings">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.3" />
            <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default StatusBar;
