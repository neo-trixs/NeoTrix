import React, { useRef, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";

const MODEL_COLORS: Record<string, string> = {
  anthropic: "#d97757", openai: "#19c37d", gemini: "#4285f4",
  ollama: "#666", groq: "#f97316", deepseek: "#4f46e5",
};

interface Props {
  onSubmit: (v: string) => void;
  disabled: boolean;
}

const ChipColors: Record<string, string> = {
  file: "#0a7dff", folder: "#30a46c", codebase: "#8b5cf6",
  docs: "#e54d2e", web: "#f5a623",
};

const MODE_LABEL: Record<string, string> = { chat: "Chat", plan: "Plan", agent: "Agent" };
const MODE_CYCLE: Record<string, string> = { chat: "plan", plan: "agent", agent: "chat" };

const InputPanel: React.FC<Props> = ({ onSubmit, disabled }) => {
  const [value, setValue] = useState("");
  const [isRecording, setIsRecording] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const contextChips = useStore((s) => s.contextChips);
  const addContextChip = useStore((s) => s.addContextChip);
  const removeContextChip = useStore((s) => s.removeContextChip);
  const agentMode = useStore((s) => s.agentMode);
  const setAgentMode = useStore((s) => s.setAgentMode);
  const currentModel = useStore((s) => s.currentModel);
  const contextUsage = useStore((s) => s.contextUsage);

  useEffect(() => {
    inputRef.current?.focus();
  }, [disabled]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      if (value.trim() && !disabled) {
        onSubmit(value);
        setValue("");
      }
    }
    if (e.key === "Enter" && e.altKey) {
      setValue((v) => v + "\n");
    }
    if (e.key === "@" && !disabled) {
      addContextChip({ id: `chip-${Date.now()}`, label: "current-file.rs", type: "file" });
    }
  };

  const handleMicClick = async () => {
    if (isRecording) return;
    setIsRecording(true);
    try {
      const text: string = await invoke("transcribe_audio");
      if (text) setValue((v) => v + (v ? " " : "") + text);
    } catch {
    } finally {
      setIsRecording(false);
    }
  };

  return (
    <div className="input-panel">
      {contextChips.length > 0 && (
        <div className="context-bar">
          {contextChips.map((chip) => (
            <span
              key={chip.id}
              className="context-chip"
              style={{ borderColor: ChipColors[chip.type] || "#888" }}
            >
              <svg width="10" height="10" viewBox="0 0 10 10" fill={ChipColors[chip.type] || "#888"}>
                {chip.type === "file" ? <path d="M2 1h3l2 2v6H2V1z" /> :
                 chip.type === "folder" ? <path d="M1 2h3l1 1h4v5H1V2z" /> :
                 <circle cx="5" cy="5" r="4" />}
              </svg>
              <span className="context-chip-label">{chip.label}</span>
              <button className="context-chip-remove" onClick={() => removeContextChip(chip.id)}>
                <svg width="8" height="8" viewBox="0 0 8 8" stroke="currentColor" strokeWidth="1.2"><path d="M2 2l4 4M6 2l-4 4" /></svg>
              </button>
            </span>
          ))}
        </div>
      )}

      <div className="input-mode-selector">
        <button
          className={`input-mode-btn ${agentMode === "chat" ? "active" : ""}`}
          onClick={() => setAgentMode("chat")}
        >
          <span className="input-mode-dot" style={{ background: "#34c759" }} />
          Chat
        </button>
        <button
          className={`input-mode-btn ${agentMode === "plan" ? "active" : ""}`}
          onClick={() => setAgentMode("plan")}
        >
          <span className="input-mode-dot" style={{ background: "#ff9500" }} />
          Plan
        </button>
        <button
          className={`input-mode-btn ${agentMode === "agent" ? "active" : ""}`}
          onClick={() => setAgentMode("agent")}
        >
          <span className="input-mode-dot" style={{ background: "#007aff" }} />
          Agent
        </button>
        <span style={{ marginLeft: 8, fontSize: 11, color: "var(--text-muted)" }}>
          {agentMode === "plan" ? "Explore only \u2014 no changes" :
           agentMode === "agent" ? "Can modify files" :
           "Chat mode"}
        </span>
      </div>

      <div className="input-row">
        <div className="input-textarea-wrap">
          <textarea
            ref={inputRef}
            className="input-field"
            placeholder={
              agentMode === "agent"
                ? "Describe what you want to build..."
                : agentMode === "plan"
                ? "Describe the plan first..."
                : "Ask anything... (@ to reference files)"
            }
            value={value}
            onChange={(e) => setValue(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={disabled}
            rows={1}
          />
          <div className="input-actions">
            <button
              className={`input-action-btn ${isRecording ? "active" : ""}`}
              onClick={handleMicClick}
              disabled={disabled}
              title="Voice input"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M12 2a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <path d="M12 19v3" />
                <path d="M8 22h8" />
              </svg>
            </button>
            <button
              className="input-action-btn"
              title="Attach context"
              onClick={() => {
                addContextChip({ id: `chip-${Date.now()}`, label: `file-${contextChips.length + 1}.rs`, type: "file" });
              }}
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round">
                <path d="M7 1v12M1 7h12" />
              </svg>
            </button>
          </div>
        </div>

        <button
          className="btn-send"
          onClick={() => { if (value.trim() && !disabled) { onSubmit(value); setValue(""); } }}
          disabled={disabled || !value.trim()}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2 7l10-4-4 10-2-4-4-2z" />
          </svg>
          Send
        </button>
      </div>
    </div>
  );
};

export default InputPanel;
