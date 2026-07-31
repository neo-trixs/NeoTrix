import React, { useState, useEffect, useCallback } from "react";
import * as api from "../commands";

const TEMPLATES = [
  { name: "CI/CD", description: "Optimized for automated CI/CD pipeline execution with fast iteration and minimal overhead", config: { model: "gpt-4o-mini", approval_mode: "auto", sandbox_mode: "enabled", web_search_enabled: false, context_compaction: "aggressive", max_tokens: 4096, temperature: 0.2, theme: "dark", custom_instructions: "Focus on speed and reliability. Auto-approve low-risk operations.", mcp_servers: [], plugins: [] } },
  { name: "Exploration", description: "Broad research mode with web search, high token limit, and permissive tools", config: { model: "claude-sonnet-4", approval_mode: "semi", sandbox_mode: "relaxed", web_search_enabled: true, context_compaction: "moderate", max_tokens: 32768, temperature: 0.7, theme: "dark", custom_instructions: "Explore broadly. Use web search. Try multiple approaches.", mcp_servers: [], plugins: [] } },
  { name: "Security Audit", description: "Strict safety profile for security scanning with locked-down execution", config: { model: "claude-sonnet-4", approval_mode: "strict", sandbox_mode: "enforced", web_search_enabled: false, context_compaction: "moderate", max_tokens: 16384, temperature: 0.1, theme: "dark", custom_instructions: "Maximum safety. Require approval for all mutations. Log everything.", mcp_servers: [], plugins: [] } },
  { name: "Deep Research", description: "Extended research sessions with long context and thorough analysis", config: { model: "claude-opus-4", approval_mode: "semi", sandbox_mode: "relaxed", web_search_enabled: true, context_compaction: "minimal", max_tokens: 65536, temperature: 0.5, theme: "dark", custom_instructions: "Deep research mode. Use web search extensively. Provide thorough citations.", mcp_servers: [], plugins: [] } },
  { name: "Quick Fix", description: "Lightweight profile for rapid bug fixes and small edits", config: { model: "gpt-4o-mini", approval_mode: "auto", sandbox_mode: "enabled", web_search_enabled: false, context_compaction: "aggressive", max_tokens: 4096, temperature: 0.3, theme: "dark", custom_instructions: "Fast fixes only. Keep responses concise. Auto-approve known-safe patterns.", mcp_servers: [], plugins: [] } },
];

function emptyConfig(): api.ProfileConfig {
  return { model: "", approval_mode: "auto", sandbox_mode: "enabled", web_search_enabled: false, context_compaction: "moderate", max_tokens: 16384, temperature: 0.5, theme: "dark", custom_instructions: null, mcp_servers: [], plugins: [] };
}

const ProfileManager: React.FC = () => {
  const [profiles, setProfiles] = useState<api.ProfileInfo[]>([]);
  const [summary, setSummary] = useState<{ total_profiles: number; active_profile: string; default_profile: string; profiles_by_model: Record<string, number> } | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [editing, setEditing] = useState<string | null>(null);
  const [form, setForm] = useState({ name: "", description: "", config: emptyConfig() });
  const [showCreate, setShowCreate] = useState(false);
  const [importText, setImportText] = useState("");
  const [showImport, setShowImport] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [p, s] = await Promise.all([api.profileList(), api.profileSummary()]);
      setProfiles(p);
      setSummary(s);
    } catch (e) { setError(String(e)); }
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  async function handleActivate(name: string) {
    try { await api.profileActivate(name); await load(); } catch (e) { setError(String(e)); }
  }

  async function handleDelete(name: string) {
    try { await api.profileDelete(name); setConfirmDelete(null); await load(); } catch (e) { setError(String(e)); }
  }

  async function handleDuplicate(name: string) {
    try {
      const newName = `${name} (copy)`;
      await api.profileDuplicate(name, newName);
      await load();
    } catch (e) { setError(String(e)); }
  }

  async function handleReset(name: string) {
    try { await api.profileReset(name); await load(); } catch (e) { setError(String(e)); }
  }

  async function handleExport(name: string) {
    try {
      const json = await api.profileExport(name);
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a"); a.href = url; a.download = `${name}.json`; a.click();
      URL.revokeObjectURL(url);
    } catch (e) { setError(String(e)); }
  }

  async function handleImport() {
    try {
      await api.profileImport(importText);
      setShowImport(false);
      setImportText("");
      await load();
    } catch (e) { setError(String(e)); }
  }

  async function handleCreate() {
    if (!form.name.trim()) return;
    try {
      await api.profileCreate(form.name, form.description, form.config);
      setShowCreate(false);
      setForm({ name: "", description: "", config: emptyConfig() });
      await load();
    } catch (e) { setError(String(e)); }
  }

  async function handleUpdate(name: string) {
    try {
      await api.profileUpdate(name, form.description || undefined, form.config);
      setEditing(null);
      await load();
    } catch (e) { setError(String(e)); }
  }

  async function handleCreateFromTemplate(tmpl: typeof TEMPLATES[number]) {
    try {
      await api.profileCreate(tmpl.name, tmpl.description, tmpl.config as api.ProfileConfig);
      await load();
    } catch (e) { setError(String(e)); }
  }

  function startEdit(p: api.ProfileInfo) {
    setEditing(p.name);
    setForm({ name: p.name, description: p.description || "", config: { ...p.config } });
  }

  function closeEdit() {
    setEditing(null);
    setForm({ name: "", description: "", config: emptyConfig() });
  }

  const setCfg = (k: keyof api.ProfileConfig, v: unknown) =>
    setForm((f) => ({ ...f, config: { ...f.config, [k]: v } }));

  const activeName = summary?.active_profile || "";

  if (loading) return <div className="p-6 text-gray-400">Loading profiles...</div>;

  return (
    <div className="p-6 max-w-6xl mx-auto text-gray-200">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-white">Profile Manager</h1>
          <p className="text-sm text-gray-400 mt-1">
            {summary ? (
              <>{summary.total_profiles} profiles &middot; Active: <span className="text-green-400 font-medium">{summary.active_profile || "none"}</span> &middot; Default: <span className="text-yellow-400">{summary.default_profile || "none"}</span></>
            ) : "Loading..."}
          </p>
        </div>
        <div className="flex gap-2">
          <button onClick={() => { setShowCreate(true); closeEdit(); }} className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-500 transition-colors">+ New Profile</button>
          <button onClick={() => setShowImport(true)} className="px-3 py-1.5 bg-gray-700 text-gray-200 text-sm rounded hover:bg-gray-600 transition-colors">Import</button>
        </div>
      </div>

      {error && <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded text-red-300 text-sm">{error}
        <button onClick={() => setError("")} className="float-right text-red-400 hover:text-red-200">✕</button>
      </div>}

      {/* Create form */}
      {showCreate && (
        <div className="mb-6 p-4 bg-gray-800/70 border border-gray-700 rounded-lg">
          <h2 className="text-lg font-semibold text-white mb-3">Create Profile</h2>
          <div className="grid grid-cols-2 gap-3 mb-3">
            <div><label className="block text-xs text-gray-400 mb-1">Name</label>
              <input value={form.name} onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" placeholder="my-profile" /></div>
            <div><label className="block text-xs text-gray-400 mb-1">Description</label>
              <input value={form.description} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" placeholder="Optional description" /></div>
          </div>
          {renderConfigFields(form.config, setCfg)}
          <div className="flex gap-2 mt-3">
            <button onClick={handleCreate} disabled={!form.name.trim()} className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-500 disabled:opacity-40 transition-colors">Create</button>
            <button onClick={() => setShowCreate(false)} className="px-3 py-1.5 bg-gray-700 text-gray-300 text-sm rounded hover:bg-gray-600 transition-colors">Cancel</button>
          </div>
        </div>
      )}

      {/* Import */}
      {showImport && (
        <div className="mb-6 p-4 bg-gray-800/70 border border-gray-700 rounded-lg">
          <h2 className="text-lg font-semibold text-white mb-2">Import Profile</h2>
          <textarea value={importText} onChange={(e) => setImportText(e.target.value)} rows={5} className="w-full bg-gray-900 border border-gray-700 rounded p-2 text-sm text-white font-mono focus:outline-none focus:border-blue-500" placeholder="Paste JSON..." />
          <div className="flex gap-2 mt-2">
            <button onClick={handleImport} disabled={!importText.trim()} className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-500 disabled:opacity-40 transition-colors">Import</button>
            <button onClick={() => { setShowImport(false); setImportText(""); }} className="px-3 py-1.5 bg-gray-700 text-gray-300 text-sm rounded hover:bg-gray-600 transition-colors">Cancel</button>
          </div>
        </div>
      )}

      {/* Templates */}
      <div className="mb-6">
        <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-2">Templates</h2>
        <div className="grid grid-cols-5 gap-2">
          {TEMPLATES.map((t) => (
            <div key={t.name} className="p-3 bg-gray-800/50 border border-gray-700 rounded-lg hover:border-gray-500 transition-colors group">
              <div className="text-sm font-medium text-white">{t.name}</div>
              <div className="text-xs text-gray-400 mt-1 line-clamp-2">{t.description}</div>
              <button onClick={() => handleCreateFromTemplate(t)} className="mt-2 text-xs text-blue-400 hover:text-blue-300 opacity-0 group-hover:opacity-100 transition-opacity">Create from template →</button>
            </div>
          ))}
        </div>
      </div>

      {/* Profile list */}
      <div className="space-y-2">
        {profiles.length === 0 && <div className="text-center py-8 text-gray-500">No profiles yet. Create one or import a JSON file.</div>}
        {profiles.map((p) => (
          <div key={p.name} className={`rounded-lg border transition-colors ${p.name === activeName ? "bg-gray-800 border-green-700" : "bg-gray-800/50 border-gray-700 hover:border-gray-500"}`}>
            {editing === p.name ? (
              /* Edit mode */
              <div className="p-4">
                <h3 className="text-sm font-semibold text-white mb-3">Editing: {p.name}</h3>
                <div className="grid grid-cols-2 gap-3 mb-3">
                  <div><label className="block text-xs text-gray-400 mb-1">Description</label>
                    <input value={form.description} onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" /></div>
                </div>
                {renderConfigFields(form.config, setCfg)}
                <div className="flex gap-2 mt-3">
                  <button onClick={() => handleUpdate(p.name)} className="px-3 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-500 transition-colors">Save</button>
                  <button onClick={closeEdit} className="px-3 py-1.5 bg-gray-700 text-gray-300 text-sm rounded hover:bg-gray-600 transition-colors">Cancel</button>
                </div>
              </div>
            ) : (
              /* View mode */
              <div className="p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-semibold text-white">{p.name}</span>
                      {p.is_default && <span className="px-1.5 py-0.5 bg-yellow-700/60 text-yellow-300 text-xs rounded">default</span>}
                      {p.is_active && <span className="px-1.5 py-0.5 bg-green-700/60 text-green-300 text-xs rounded">active</span>}
                    </div>
                    {p.description && <div className="text-xs text-gray-400 mt-1">{p.description}</div>}
                    <div className="flex items-center gap-3 mt-1.5 text-xs text-gray-500">
                      <span>Model: <span className="text-gray-300">{p.config.model || "—"}</span></span>
                      <span>Tokens: <span className="text-gray-300">{p.config.max_tokens}</span></span>
                      <span>Temp: <span className="text-gray-300">{p.config.temperature}</span></span>
                      <span>Approval: <span className="text-gray-300">{p.config.approval_mode}</span></span>
                      <span>Created: <span className="text-gray-300">{new Date(p.created_at).toLocaleDateString()}</span></span>
                    </div>
                  </div>
                  <div className="flex items-center gap-1 ml-4 shrink-0">
                    {!p.is_active && (
                      <button onClick={() => handleActivate(p.name)} className="px-2 py-1 text-xs bg-green-700 text-green-200 rounded hover:bg-green-600 transition-colors" title="Activate">Activate</button>
                    )}
                    <button onClick={() => startEdit(p)} className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors">Edit</button>
                    <button onClick={() => handleDuplicate(p.name)} className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors">Duplicate</button>
                    <button onClick={() => handleExport(p.name)} className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors">Export</button>
                    <button onClick={() => handleReset(p.name)} className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors">Reset</button>
                    {confirmDelete === p.name ? (
                      <div className="flex items-center gap-1">
                        <span className="text-xs text-red-400">Sure?</span>
                        <button onClick={() => handleDelete(p.name)} className="px-2 py-1 text-xs bg-red-700 text-red-200 rounded hover:bg-red-600 transition-colors">Delete</button>
                        <button onClick={() => setConfirmDelete(null)} className="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition-colors">No</button>
                      </div>
                    ) : (
                      <button onClick={() => {
                        if (p.is_default || p.is_active) { setError(`Cannot delete the ${p.is_default ? "default" : "active"} profile. Activate another profile first.`); return; }
                        setConfirmDelete(p.name);
                      }} className="px-2 py-1 text-xs bg-gray-700 text-red-300 rounded hover:bg-red-700 transition-colors">Delete</button>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

function renderConfigFields(cfg: api.ProfileConfig, set: (k: keyof api.ProfileConfig, v: unknown) => void) {
  return (
    <div className="grid grid-cols-3 gap-3">
      <div><label className="block text-xs text-gray-400 mb-1">Model</label>
        <input value={cfg.model} onChange={(e) => set("model", e.target.value)} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" placeholder="gpt-4o" /></div>
      <div><label className="block text-xs text-gray-400 mb-1">Approval Mode</label>
        <select value={cfg.approval_mode} onChange={(e) => set("approval_mode", e.target.value)} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500">
          <option value="auto">Auto</option>
          <option value="semi">Semi (ask for risky)</option>
          <option value="strict">Strict (ask for all)</option>
        </select></div>
      <div><label className="block text-xs text-gray-400 mb-1">Sandbox Mode</label>
        <select value={cfg.sandbox_mode} onChange={(e) => set("sandbox_mode", e.target.value)} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500">
          <option value="enabled">Enabled</option>
          <option value="enforced">Enforced</option>
          <option value="relaxed">Relaxed</option>
        </select></div>
      <div><label className="block text-xs text-gray-400 mb-1">Web Search</label>
        <label className="flex items-center gap-2 mt-1.5 cursor-pointer">
          <div className={`w-8 h-4 rounded-full transition-colors relative ${cfg.web_search_enabled ? "bg-blue-600" : "bg-gray-600"}`} onClick={() => set("web_search_enabled", !cfg.web_search_enabled)}>
            <div className={`absolute top-0.5 w-3 h-3 bg-white rounded-full transition-transform ${cfg.web_search_enabled ? "translate-x-4" : "translate-x-0.5"}`} />
          </div>
          <span className="text-sm text-gray-300">{cfg.web_search_enabled ? "Enabled" : "Disabled"}</span>
        </label></div>
      <div><label className="block text-xs text-gray-400 mb-1">Context Compaction</label>
        <select value={cfg.context_compaction} onChange={(e) => set("context_compaction", e.target.value)} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500">
          <option value="minimal">Minimal</option>
          <option value="moderate">Moderate</option>
          <option value="aggressive">Aggressive</option>
        </select></div>
      <div><label className="block text-xs text-gray-400 mb-1">Max Tokens</label>
        <input type="number" value={cfg.max_tokens} onChange={(e) => set("max_tokens", parseInt(e.target.value) || 0)} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" min={1024} max={131072} /></div>
      <div><label className="block text-xs text-gray-400 mb-1">Temperature</label>
        <div className="flex items-center gap-2">
          <input type="range" min={0} max={100} value={Math.round(cfg.temperature * 100)} onChange={(e) => set("temperature", Math.round(parseInt(e.target.value)) / 100)} className="flex-1 accent-blue-500" />
          <span className="text-sm text-gray-300 w-8 text-right">{cfg.temperature.toFixed(2)}</span>
        </div></div>
      <div className="col-span-3"><label className="block text-xs text-gray-400 mb-1">Custom Instructions</label>
        <textarea value={cfg.custom_instructions || ""} onChange={(e) => set("custom_instructions", e.target.value || null)} rows={2} className="w-full bg-gray-900 border border-gray-700 rounded px-2 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500" placeholder="Optional system instructions..." /></div>
    </div>
  );
}

export default ProfileManager;
