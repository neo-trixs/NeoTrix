import React, { useState, useRef, useEffect } from "react";
import type { ProviderConfig } from "../types";

interface Props {
  sessionName: string;
  providerConfig: ProviderConfig;
  onRename: (name: string) => void;
  onExport: () => void;
  onImport: () => void;
  onClear: () => void;
  onDelete: () => void;
}

const MODELS = [
  { provider: "Anthropic", models: ["Claude 4 Opus", "Claude 4 Sonnet", "Claude 3.5 Haiku"] },
  { provider: "OpenAI", models: ["GPT-5.5", "GPT-5", "GPT-4.5"] },
  { provider: "Google", models: ["Gemini 3 Pro", "Gemini 3 Flash", "Gemini 2.5 Pro"] },
  { provider: "DeepSeek", models: ["R1.5", "R1", "V4"] },
];

const TopBar: React.FC<Props> = ({ sessionName, providerConfig, onRename, onExport, onImport, onClear, onDelete }) => {
  const [modelOpen, setModelOpen] = useState(false);
  const [kebabOpen, setKebabOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [renameValue, setRenameValue] = useState(sessionName);
  const renamingRef = useRef<HTMLInputElement>(null);
  const kebabRef = useRef<HTMLDivElement>(null);
  const modelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (renaming) renamingRef.current?.focus();
  }, [renaming]);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (kebabRef.current && !kebabRef.current.contains(e.target as Node)) setKebabOpen(false);
      if (modelRef.current && !modelRef.current.contains(e.target as Node)) setModelOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const commitRename = () => {
    if (renameValue.trim()) onRename(renameValue.trim());
    setRenaming(false);
  };

  return (
    <div style={{
      display: "flex", alignItems: "center", gap: 8,
      padding: "6px 12px", borderBottom: "0.5px solid var(--nt-border, rgba(0,0,0,0.06))",
      background: "var(--nt-canvas, #FAF8F4)", flexShrink: 0, minHeight: 36,
    }}>
      {/* Session name */}
      {renaming ? (
        <input
          ref={renamingRef}
          value={renameValue}
          onChange={(e) => setRenameValue(e.target.value)}
          onBlur={commitRename}
          onKeyDown={(e) => { if (e.key === "Enter") commitRename(); if (e.key === "Escape") setRenaming(false); }}
          style={{
            fontSize: 12, fontWeight: 500, border: "0.5px solid var(--nt-accent, #007aff)",
            borderRadius: 4, padding: "2px 6px", background: "var(--nt-bg, #fff)",
            color: "var(--nt-text)", fontFamily: "inherit", outline: "none", maxWidth: 200,
          }}
        />
      ) : (
        <span
          style={{ fontSize: 12, fontWeight: 500, color: "var(--nt-text)", cursor: "pointer" }}
          onClick={() => { setRenameValue(sessionName); setRenaming(true); }}
          title="Click to rename"
        >
          {sessionName}
        </span>
      )}

      {/* Model selector */}
      <div ref={modelRef} style={{ position: "relative" }}>
        <button
          onClick={() => setModelOpen(!modelOpen)}
          style={{
            display: "flex", alignItems: "center", gap: 4, padding: "2px 8px",
            fontSize: 11, fontWeight: 500, background: "var(--nt-glass-bg, rgba(255,255,255,0.72))",
            border: "0.5px solid var(--nt-border, rgba(0,0,0,0.06))",
            borderRadius: 6, color: "var(--nt-text)", cursor: "pointer", fontFamily: "inherit",
          }}
        >
          <span>{providerConfig.name} {providerConfig.model}</span>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="none"><path d="M2 3l2 2 2-2" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round"/></svg>
        </button>
        {modelOpen && (
          <div style={{
            position: "absolute", top: "100%", left: 0, marginTop: 4,
            background: "var(--nt-surface-elevated, #fff)",
            border: "0.5px solid var(--nt-border-strong, rgba(0,0,0,0.10))",
            borderRadius: 8, boxShadow: "0 4px 16px rgba(0,0,0,0.08)", zIndex: 100,
            minWidth: 180, padding: 4,
          }}>
            {MODELS.map((group) => (
              <div key={group.provider}>
                <div style={{ fontSize: 9, fontWeight: 600, color: "var(--nt-text-muted)", padding: "4px 8px 2px", textTransform: "uppercase", letterSpacing: 0.3 }}>
                  {group.provider}
                </div>
                {group.models.map((m) => (
                  <div key={m}
                    onClick={() => setModelOpen(false)}
                    style={{
                      padding: "4px 8px", fontSize: 11, cursor: "pointer",
                      borderRadius: 4, display: "flex", alignItems: "center", gap: 6,
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "var(--nt-hover, rgba(0,0,0,0.04))")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "none")}
                  >
                    {m}
                  </div>
                ))}
              </div>
            ))}
            <div style={{ borderTop: "0.5px solid var(--nt-border)", margin: "3px 0" }} />
            <div style={{ padding: "4px 8px", fontSize: 11, color: "var(--nt-text-muted)", cursor: "pointer" }}>
              Open Settings
            </div>
          </div>
        )}
      </div>

      <div style={{ flex: 1 }} />

      {/* Kebab menu */}
      <div ref={kebabRef} style={{ position: "relative" }}>
        <button
          onClick={() => setKebabOpen(!kebabOpen)}
          style={{
            width: 22, height: 22, display: "flex", alignItems: "center", justifyContent: "center",
            background: "none", border: "none", borderRadius: 4, color: "var(--nt-text-muted)",
            cursor: "pointer", fontSize: 14, lineHeight: 1,
          }}
        >
          ⋯
        </button>
        {kebabOpen && (
          <div style={{
            position: "absolute", top: "100%", right: 0, marginTop: 2,
            background: "var(--nt-surface-elevated, #fff)",
            border: "0.5px solid var(--nt-border-strong, rgba(0,0,0,0.10))",
            borderRadius: 8, boxShadow: "0 4px 16px rgba(0,0,0,0.08)", zIndex: 100,
            minWidth: 140, padding: 4,
          }}>
            <MenuItem label="Export" icon="↓" onClick={() => { onExport(); setKebabOpen(false); }} />
            <MenuItem label="Import" icon="↑" onClick={() => { onImport(); setKebabOpen(false); }} />
            <div style={{ height: 1, background: "var(--nt-border)", margin: "3px 4px" }} />
            <MenuItem label="Clear" icon="✕" onClick={() => { onClear(); setKebabOpen(false); }} />
            <MenuItem label="Delete" icon="🗑" onClick={() => { onDelete(); setKebabOpen(false); }} danger />
          </div>
        )}
      </div>
    </div>
  );
};

function MenuItem({ label, icon, onClick, danger }: { label: string; icon: string; onClick: () => void; danger?: boolean }) {
  return (
    <div
      onClick={onClick}
      style={{
        padding: "4px 8px", fontSize: 11, cursor: "pointer", borderRadius: 4,
        display: "flex", alignItems: "center", gap: 6,
        color: danger ? "var(--nt-danger, #ff3b30)" : "var(--nt-text)",
      }}
      onMouseEnter={(e) => (e.currentTarget.style.background = "var(--nt-hover, rgba(0,0,0,0.04))")}
      onMouseLeave={(e) => (e.currentTarget.style.background = "none")}
    >
      <span>{icon}</span>
      <span>{label}</span>
    </div>
  );
}

export default TopBar;