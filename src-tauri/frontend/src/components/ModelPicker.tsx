import React, { useState, useEffect, useRef, useMemo } from "react";
import * as api from "../lib/api";
import { useStore } from "../store";

interface Props {
  onClose: () => void;
  onSwitch: (modelId: string) => void;
}

const PROVIDER_NAMES: Record<string, string> = {
  anthropic: "Anthropic",
  openai: "OpenAI",
  gemini: "Google",
  ollama: "Ollama",
  groq: "Groq",
  deepseek: "DeepSeek",
  neotrix: "NeoTrix",
  "neotrix-zen": "NeoTrix Zen",
  "lm-studio": "LM Studio",
  "llama-cpp": "llama.cpp",
  vllm: "vLLM",
};

const ModelPicker: React.FC<Props> = ({ onClose, onSwitch }) => {
  const [models, setModels] = useState<api.ModelInfo[]>([]);
  const [search, setSearch] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(true);
  const inputRef = useRef<HTMLInputElement>(null);
  const providerConfig = useStore((s) => s.providerConfig);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const addNotification = useStore((s) => s.addNotification);

  useEffect(() => {
    api.listModels().then(setModels).catch(() => setModels([])).finally(() => setLoading(false));
    inputRef.current?.focus();
  }, []);

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    if (!q) return models;
    return models.filter(m =>
      m.name.toLowerCase().includes(q) ||
      m.provider.toLowerCase().includes(q) ||
      m.id.toLowerCase().includes(q)
    );
  }, [models, search]);

  const handleSelect = async (model: api.ModelInfo) => {
    const parts = model.id.split("/");
    const providerId = parts[0];
    const modelId = parts[1] || model.name;
    const newConfig = {
      ...providerConfig,
      id: providerId as any,
      model: modelId,
      name: `${PROVIDER_NAMES[model.provider] || model.provider} - ${model.name}`,
    };
    setProviderConfig(newConfig);
    await api.saveProviderConfig(newConfig).catch(() => {});
    onSwitch(model.id);
    addNotification({ type: "success", message: `Switched to ${model.name}`, duration: 3000 });
    onClose();
  };

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIndex(i => Math.min(i + 1, filtered.length - 1)); }
      if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIndex(i => Math.max(i - 1, 0)); }
      if (e.key === "Enter" && filtered[selectedIndex]) { e.preventDefault(); handleSelect(filtered[selectedIndex]); }
      if (e.key === "Escape") { e.preventDefault(); onClose(); }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [filtered, selectedIndex]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [search]);

  const grouped = useMemo(() => {
    const groups: Record<string, api.ModelInfo[]> = {};
    for (const m of filtered) {
      const p = PROVIDER_NAMES[m.provider] || m.provider;
      if (!groups[p]) groups[p] = [];
      groups[p].push(m);
    }
    return groups;
  }, [filtered]);

  return (
    <div className="model-picker-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="model-picker-panel glass-panel">
        <div className="model-picker-search">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.4">
            <circle cx="6" cy="6" r="4.5" />
            <line x1="9.5" y1="9.5" x2="13" y2="13" />
          </svg>
          <input
            ref={inputRef}
            value={search}
            onChange={e => { setSearch(e.target.value); }}
            placeholder="Search models..."
            spellCheck={false}
          />
          {loading && <span className="model-picker-loading">Loading...</span>}
        </div>
        <div className="model-picker-list">
          {!loading && filtered.length === 0 && (
            <div className="model-picker-empty">No models match. Set API keys and try again.</div>
          )}
          {Object.entries(grouped).map(([provider, providerModels]) => (
            <div key={provider} className="model-picker-group">
              <div className="model-picker-provider-label">{provider}</div>
              {providerModels.map((m) => {
                const flatIndex = filtered.indexOf(m);
                return (
                  <div
                    key={m.id}
                    className={`model-picker-item ${flatIndex === selectedIndex ? "selected" : ""}`}
                    onClick={() => handleSelect(m)}
                    onMouseEnter={() => setSelectedIndex(flatIndex)}
                  >
                    <div className="model-picker-item-info">
                      <span className="model-picker-item-name">{m.name}</span>
                      <span className="model-picker-item-tier">{m.tier}</span>
                    </div>
                  </div>
                );
              })}
            </div>
          ))}
        </div>
        <div className="model-picker-footer">
          <span>↑↓ Navigate · Enter Select · Esc Close</span>
        </div>
      </div>
    </div>
  );
};

export default ModelPicker;
