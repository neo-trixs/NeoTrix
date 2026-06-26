import React, { useState, useCallback } from "react";
import type { AgentPreset, AgentMakerPreset, ModelTier } from "../types";
import { useStore } from "../store";

const AVAILABLE_TOOLS = [
  "web_search", "web_scrape", "code_exec", "file_read", "file_write",
  "git_ops", "terminal", "diff_view", "permission_request",
];

const AVAILABLE_KNOWLEDGE = [
  "project_context", "user_preferences", "system_docs", "reasoning_memory",
  "world_model", "capability_vector",
];

const BUILTIN_PRESETS: AgentMakerPreset[] = [
  {
    id: "personal-assistant",
    name: "Personal Assistant",
    description: "Balanced conversational agent for day-to-day tasks and general-purpose interactions",
    category: "General",
    icon: "🤖",
    systemPrompt: "You are a helpful, balanced conversational assistant. Respond thoughtfully and concisely. Adapt your tone to match the user.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.7,
    defaultMaxTokens: 4096,
    suggestedTools: ["web_search", "file_read", "file_write"],
    suggestedKnowledge: ["user_preferences", "project_context"],
  },
  {
    id: "deep-researcher",
    name: "Deep Researcher",
    description: "Web research specialist with thorough multi-source analysis and citation tracking",
    category: "Research",
    icon: "🔬",
    systemPrompt: "You are a thorough research assistant. Search multiple sources, cross-reference findings, and provide well-cited answers. Prioritize accuracy over speed. Use structured formats with clear sections.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.3,
    defaultMaxTokens: 8192,
    suggestedTools: ["web_search", "web_scrape", "file_read", "file_write"],
    suggestedKnowledge: ["reasoning_memory", "world_model"],
  },
  {
    id: "hashcoder",
    name: "HashCoder",
    description: "Code generation specialist with multi-file awareness and type-safe output",
    category: "Development",
    icon: "💻",
    systemPrompt: "You are an expert software engineer. Write clean, idiomatic, well-typed code following best practices. Always consider edge cases, error handling, and performance. Provide complete implementations.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.5,
    defaultMaxTokens: 8192,
    suggestedTools: ["code_exec", "file_read", "file_write", "git_ops", "diff_view"],
    suggestedKnowledge: ["project_context", "system_docs"],
  },
  {
    id: "papers",
    name: "Papers",
    description: "Academic paper analysis specialist for digesting research and extracting key insights",
    category: "Research",
    icon: "📄",
    systemPrompt: "You are an academic research analyst. Analyze papers critically: summarize contributions, evaluate methodology, identify limitations, and suggest future work. Use precise technical terminology.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.3,
    defaultMaxTokens: 8192,
    suggestedTools: ["web_search", "web_scrape", "file_read"],
    suggestedKnowledge: ["reasoning_memory", "capability_vector"],
  },
  {
    id: "medical-lexi",
    name: "Medical Lexi",
    description: "Medical terminology specialist with precise clinical language understanding",
    category: "Specialized",
    icon: "🏥",
    systemPrompt: "You are a medical terminology expert. Provide precise definitions, context, and usage for medical terms. Always include relevant standards (ICD-10, SNOMED, LOINC). Disclaim that you are not a substitute for professional medical advice.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.2,
    defaultMaxTokens: 4096,
    suggestedTools: ["web_search", "file_read"],
    suggestedKnowledge: ["reasoning_memory"],
  },
  {
    id: "ats-auditor",
    name: "ATS Auditor",
    description: "Resume/CV auditor specializing in ATS compatibility and resume optimization",
    category: "Career",
    icon: "📋",
    systemPrompt: "You are an ATS (Applicant Tracking System) expert. Analyze resumes for keyword density, formatting compatibility, section structure, and role-specific optimization. Provide actionable improvement suggestions.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "medium",
    defaultTemperature: 0.4,
    defaultMaxTokens: 4096,
    suggestedTools: ["file_read", "file_write"],
    suggestedKnowledge: ["user_preferences"],
  },
  {
    id: "lite",
    name: "Lite",
    description: "Fast, minimal agent optimized for quick responses and low token usage",
    category: "General",
    icon: "⚡",
    systemPrompt: "Respond as concisely as possible. Use minimal formatting. Prefer single-line answers. Skip pleasantries. Be direct.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "free",
    defaultTemperature: 0.5,
    defaultMaxTokens: 1024,
    suggestedTools: [],
    suggestedKnowledge: [],
  },
  {
    id: "url-reader",
    name: "URL Reader",
    description: "Content extraction specialist for reading and summarizing web pages",
    category: "Utility",
    icon: "🌐",
    systemPrompt: "You extract and summarize web content. Given a URL, fetch and analyze the page content, then provide a clear summary with key points, structure, and relevant metadata.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "medium",
    defaultTemperature: 0.3,
    defaultMaxTokens: 4096,
    suggestedTools: ["web_search", "web_scrape"],
    suggestedKnowledge: [],
  },
  {
    id: "deep-research-brief",
    name: "Deep Research Brief",
    description: "Structured research report generator with executive summaries and evidence mapping",
    category: "Research",
    icon: "📊",
    systemPrompt: "You generate structured deep research briefs. Output must include: Executive Summary, Key Findings (with evidence mapping), Methodology Notes, Contradictions & Gaps, and Actionable Recommendations. Use data-driven reasoning.",
    defaultModel: "claude-sonnet-4-20250514",
    defaultTier: "high",
    defaultTemperature: 0.4,
    defaultMaxTokens: 8192,
    suggestedTools: ["web_search", "web_scrape", "file_read", "file_write"],
    suggestedKnowledge: ["reasoning_memory", "world_model", "capability_vector"],
  },
];

function tierLabel(tier: ModelTier): string {
  const map: Record<ModelTier, string> = {
    free: "Free", low: "Low Cost", medium: "Standard", high: "High Quality", custom: "Custom",
  };
  return map[tier];
}

function tierColor(tier: ModelTier): string {
  const map: Record<ModelTier, string> = {
    free: "#8e8e93", low: "#ff9500", medium: "#007aff", high: "#34c759", custom: "#5856d6",
  };
  return map[tier];
}

interface TemplateCardProps {
  preset: AgentMakerPreset;
  onSelect: (preset: AgentMakerPreset) => void;
}

const TemplateCard: React.FC<TemplateCardProps> = ({ preset, onSelect }) => (
  <div className="agent-template-card" onClick={() => onSelect(preset)}>
    <div className="agent-card-icon">{preset.icon}</div>
    <div className="agent-card-body">
      <div className="agent-card-name">{preset.name}</div>
      <div className="agent-card-desc">{preset.description}</div>
      <div className="agent-card-meta">
        <span className="agent-card-category" style={{ background: `${tierColor(preset.defaultTier)}18`, color: tierColor(preset.defaultTier) }}>
          {preset.category}
        </span>
        <span className="agent-card-tier">{tierLabel(preset.defaultTier)}</span>
      </div>
    </div>
    <button
      className="agent-card-btn"
      onClick={(e) => { e.stopPropagation(); onSelect(preset); }}
    >
      Create
    </button>
  </div>
);

interface PreviewCardProps {
  name: string;
  description: string;
  systemPrompt: string;
  modelTier: ModelTier;
  temperature: number;
  maxTokens: number;
  tools: string[];
  knowledgeSources: string[];
}

const PreviewCard: React.FC<PreviewCardProps> = ({ name, description, systemPrompt, modelTier, temperature, maxTokens, tools, knowledgeSources }) => (
  <div className="agent-preview">
    <div className="agent-preview-header">
      <span className="agent-preview-dot" style={{ background: tierColor(modelTier) }} />
      <span className="agent-preview-name">{name || "Unnamed Agent"}</span>
    </div>
    {description && <div className="agent-preview-desc">{description}</div>}
    <div className="agent-preview-stats">
      <span>{tierLabel(modelTier)}</span>
      <span>·</span>
      <span>Temp {temperature.toFixed(1)}</span>
      <span>·</span>
      <span>{maxTokens} tok</span>
    </div>
    <div className="agent-preview-prompt">{systemPrompt ? systemPrompt.slice(0, 120) + (systemPrompt.length > 120 ? "…" : "") : "No system prompt"}</div>
    {tools.length > 0 && (
      <div className="agent-preview-tags">
        {tools.map((t) => <span key={t} className="agent-preview-tag tool-tag">{t}</span>)}
      </div>
    )}
    {knowledgeSources.length > 0 && (
      <div className="agent-preview-tags">
        {knowledgeSources.map((k) => <span key={k} className="agent-preview-tag knowledge-tag">{k}</span>)}
      </div>
    )}
  </div>
);

const AgentMaker: React.FC = () => {
  const customPresets = useStore((s) => s.customPresets);
  const addCustomPreset = useStore((s) => s.addCustomPreset);
  const removeCustomPreset = useStore((s) => s.removeCustomPreset);

  const [mode, setMode] = useState<"gallery" | "editor">("gallery");
  const [editingId, setEditingId] = useState<string | null>(null);

  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [systemPrompt, setSystemPrompt] = useState("");
  const [modelTier, setModelTier] = useState<ModelTier>("medium");
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(4096);
  const [tools, setTools] = useState<string[]>([]);
  const [knowledgeSources, setKnowledgeSources] = useState<string[]>([]);

  const [toast, setToast] = useState<string | null>(null);

  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 2000);
  }, []);

  const handleSelectTemplate = useCallback((preset: AgentMakerPreset) => {
    setName(preset.name);
    setDescription(preset.description);
    setSystemPrompt(preset.systemPrompt);
    setModelTier(preset.defaultTier);
    setTemperature(preset.defaultTemperature);
    setMaxTokens(preset.defaultMaxTokens);
    setTools([...preset.suggestedTools]);
    setKnowledgeSources([...preset.suggestedKnowledge]);
    setEditingId(null);
    setMode("editor");
  }, []);

  const handleSave = useCallback(() => {
    if (!name.trim()) { showToast("Name is required"); return; }
    const preset: AgentPreset = {
      id: editingId || `custom-${Date.now()}`,
      name,
      description,
      systemPrompt,
      model: modelTier === "custom" ? "" : `model-${modelTier}`,
      modelTier,
      temperature,
      tools,
      knowledgeSources,
      maxTokens,
      isBuiltin: false,
    };
    addCustomPreset(preset);
    showToast("Agent saved!");
    setMode("gallery");
  }, [name, description, systemPrompt, modelTier, temperature, maxTokens, tools, knowledgeSources, editingId, addCustomPreset, showToast]);

  const handleExport = useCallback(() => {
    const data = { name, description, systemPrompt, modelTier, temperature, maxTokens, tools, knowledgeSources };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${name.replace(/[^a-zA-Z0-9]/g, "_").toLowerCase() || "agent"}.json`;
    a.click();
    URL.revokeObjectURL(url);
    showToast("Exported!");
  }, [name, description, systemPrompt, modelTier, temperature, maxTokens, tools, knowledgeSources, showToast]);

  const handleImport = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const data = JSON.parse(reader.result as string);
          setName(data.name || "");
          setDescription(data.description || "");
          setSystemPrompt(data.systemPrompt || "");
          setModelTier(data.modelTier || "medium");
          setTemperature(data.temperature ?? 0.7);
          setMaxTokens(data.maxTokens ?? 4096);
          setTools(data.tools || []);
          setKnowledgeSources(data.knowledgeSources || []);
          setEditingId(null);
          setMode("editor");
          showToast("Imported!");
        } catch { showToast("Invalid JSON file"); }
      };
      reader.readAsText(file);
    };
    input.click();
  }, [showToast]);

  const handleEditCustom = useCallback((preset: AgentPreset) => {
    setName(preset.name);
    setDescription(preset.description);
    setSystemPrompt(preset.systemPrompt);
    setModelTier(preset.modelTier);
    setTemperature(preset.temperature);
    setMaxTokens(preset.maxTokens);
    setTools([...preset.tools]);
    setKnowledgeSources([...preset.knowledgeSources]);
    setEditingId(preset.id);
    setMode("editor");
  }, []);

  const toggleTool = useCallback((tool: string) => {
    setTools((prev) => prev.includes(tool) ? prev.filter((t) => t !== tool) : [...prev, tool]);
  }, []);

  const toggleKnowledge = useCallback((k: string) => {
    setKnowledgeSources((prev) => prev.includes(k) ? prev.filter((x) => x !== k) : [...prev, k]);
  }, []);

  if (mode === "editor") {
    return (
      <div className="agent-maker-container">
        <div className="agent-maker-header">
          <button className="btn-icon" onClick={() => setMode("gallery")} title="Back to gallery">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M10 4L6 8l4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
          <h2>{editingId ? "Edit Agent" : "Create Agent"}</h2>
          <div style={{ flex: 1 }} />
          <button className="btn-secondary" onClick={handleExport} title="Export as JSON">
            Export
          </button>
          <button className="btn-primary" onClick={handleSave}>Save Agent</button>
        </div>

        <div className="agent-editor-layout">
          <div className="agent-editor">
            <div className="agent-editor-section">
              <label className="agent-editor-label">Name</label>
              <input className="agent-editor-input" type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder="My Agent" />
            </div>

            <div className="agent-editor-section">
              <label className="agent-editor-label">Description</label>
              <input className="agent-editor-input" type="text" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="What does this agent do?" />
            </div>

            <div className="agent-editor-section">
              <label className="agent-editor-label">System Prompt</label>
              <textarea className="agent-editor-textarea" value={systemPrompt} onChange={(e) => setSystemPrompt(e.target.value)} placeholder="You are a..." rows={6} />
            </div>

            <div className="agent-editor-row">
              <div className="agent-editor-section">
                <label className="agent-editor-label">Model Tier</label>
                <select className="agent-editor-select" value={modelTier} onChange={(e) => setModelTier(e.target.value as ModelTier)}>
                  <option value="free">Free</option>
                  <option value="low">Low Cost</option>
                  <option value="medium">Standard</option>
                  <option value="high">High Quality</option>
                  <option value="custom">Custom</option>
                </select>
              </div>

              <div className="agent-editor-section">
                <label className="agent-editor-label">Temperature ({temperature.toFixed(1)})</label>
                <input className="agent-editor-range" type="range" min="0" max="2" step="0.1" value={temperature} onChange={(e) => setTemperature(parseFloat(e.target.value))} />
              </div>

              <div className="agent-editor-section">
                <label className="agent-editor-label">Max Tokens ({maxTokens})</label>
                <input className="agent-editor-range" type="range" min="256" max="8192" step="256" value={maxTokens} onChange={(e) => setMaxTokens(parseInt(e.target.value))} />
              </div>
            </div>

            <div className="agent-editor-section">
              <label className="agent-editor-label">Tools</label>
              <div className="agent-editor-toggles">
                {AVAILABLE_TOOLS.map((tool) => (
                  <label key={tool} className="agent-editor-toggle">
                    <input type="checkbox" checked={tools.includes(tool)} onChange={() => toggleTool(tool)} />
                    <span>{tool.replace(/_/g, " ")}</span>
                  </label>
                ))}
              </div>
            </div>

            <div className="agent-editor-section">
              <label className="agent-editor-label">Knowledge Sources</label>
              <div className="agent-editor-toggles">
                {AVAILABLE_KNOWLEDGE.map((k) => (
                  <label key={k} className="agent-editor-toggle">
                    <input type="checkbox" checked={knowledgeSources.includes(k)} onChange={() => toggleKnowledge(k)} />
                    <span>{k.replace(/_/g, " ")}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div className="agent-editor-preview-col">
            <label className="agent-editor-label">Preview</label>
            <PreviewCard
              name={name}
              description={description}
              systemPrompt={systemPrompt}
              modelTier={modelTier}
              temperature={temperature}
              maxTokens={maxTokens}
              tools={tools}
              knowledgeSources={knowledgeSources}
            />
            <button className="btn-secondary" onClick={handleImport} style={{ marginTop: 8 }}>
              Import JSON
            </button>
          </div>
        </div>

        {toast && <div className="agent-toast">{toast}</div>}
      </div>
    );
  }

  return (
    <div className="agent-maker-container">
      <div className="agent-maker-header">
        <h2>Agent Maker</h2>
        <div style={{ display: "flex", gap: 6 }}>
          <button className="btn-secondary" onClick={handleImport}>Import</button>
          <button className="btn-primary" onClick={() => { setName(""); setDescription(""); setSystemPrompt(""); setModelTier("medium"); setTemperature(0.7); setMaxTokens(4096); setTools([]); setKnowledgeSources([]); setEditingId(null); setMode("editor"); }}>
            + New Agent
          </button>
        </div>
      </div>

      <div className="agent-template-grid">
        <div className="agent-template-section-title">Built-in Templates</div>
        {BUILTIN_PRESETS.map((preset) => (
          <TemplateCard key={preset.id} preset={preset} onSelect={handleSelectTemplate} />
        ))}

        {customPresets.length > 0 && (
          <>
            <div className="agent-template-section-title" style={{ marginTop: 16 }}>Custom Agents</div>
            {customPresets.map((preset) => (
              <div key={preset.id} className="agent-template-card custom-card" onClick={() => handleEditCustom(preset)}>
                <div className="agent-card-icon">🧩</div>
                <div className="agent-card-body">
                  <div className="agent-card-name">{preset.name}</div>
                  <div className="agent-card-desc">{preset.description || "No description"}</div>
                  <div className="agent-card-meta">
                    <span className="agent-card-tier">{tierLabel(preset.modelTier)}</span>
                  </div>
                </div>
                <button className="agent-card-btn-sm" onClick={(e) => { e.stopPropagation(); removeCustomPreset(preset.id); }} title="Delete">
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                    <path d="M3 4h8M5.5 4V3a1 1 0 011-1h1a1 1 0 011 1v1M4 4v7a1 1 0 001 1h4a1 1 0 001-1V4" />
                  </svg>
                </button>
              </div>
            ))}
          </>
        )}
      </div>

      {toast && <div className="agent-toast">{toast}</div>}
    </div>
  );
};

export default AgentMaker;
