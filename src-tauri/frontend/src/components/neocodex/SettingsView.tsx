import React, { useEffect, useState } from "react";
import { useStore } from "../../stores";
import { invoke } from "@tauri-apps/api/core";
import styles from "./SettingsView.module.css";

type Tab = "providers" | "theme" | "advanced" | "about";

export function SettingsView() {
  const [activeTab, setActiveTab] = useState<Tab>("providers");
  const [providers, setProviders] = useState<Record<string, { name: string; hasKey: boolean; model: string }>>({});
  const [loading, setLoading] = useState(true);
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [newKey, setNewKey] = useState("");
  const [theme, setTheme] = useState<"light" | "dark" | "system">("system");

  useEffect(() => {
    const load = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        // Load provider configs
        const result = await invoke("neocodex_provider_config") as string;
        const parts = result.split(" ");
        const parsed = {
          provider_count: Number(parts[0]?.split("=")[1] || 0),
          resolvable: parts[1]?.split("=")[1] === "true",
          active_model: parts[2]?.split("=")[1] || "unknown",
        };
        // Mock provider list - in real impl, fetch from Tauri
        setProviders({
          anthropic: { name: "Anthropic (Claude)", hasKey: true, model: "claude-3-5-sonnet" },
          openai: { name: "OpenAI", hasKey: false, model: "gpt-4o" },
          gemini: { name: "Google Gemini", hasKey: false, model: "gemini-1.5-pro" },
          ollama: { name: "Ollama (Local)", hasKey: true, model: "llama3.1" },
        });
      } catch (e) {
        console.error("Failed to load settings:", e);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const handleSaveKey = async (providerId: string) => {
    if (!newKey.trim()) return;
    try {
      await invoke("save_api_key", { key: newKey, provider: providerId });
      setProviders((prev) => ({ ...prev, [providerId]: { ...prev[providerId], hasKey: true } }));
      setEditingKey(null);
      setNewKey("");
    } catch (e) {
      console.error("Failed to save key:", e);
    }
  };

  const handleDeleteKey = async (providerId: string) => {
    try {
      await invoke("delete_api_key", { provider: providerId });
      setProviders((prev) => ({ ...prev, [providerId]: { ...prev[providerId], hasKey: false } }));
    } catch (e) {
      console.error("Failed to delete key:", e);
    }
  };

  const handleThemeChange = (newTheme: "light" | "dark" | "system") => {
    setTheme(newTheme);
    document.documentElement.setAttribute("data-theme", newTheme);
    localStorage.setItem("theme", newTheme);
  };

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "providers", label: "Providers", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M3 7h8M3 4h8M3 10h8" strokeLinecap="round"/></svg> },
    { id: "theme", label: "外观", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="4"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1M4.5 4.5l.7.7M8.8 8.8l.7.7M4.5 9.5l.7-.7M8.8 5.2l.7-.7" strokeLinecap="round"/></svg> },
    { id: "advanced", label: "高级", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="5"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
    { id: "about", label: "关于", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="5"/><path d="M7 5v2M7 9v.01" strokeLinecap="round"/></svg> },
  ];

  if (loading) {
    return <div className={styles.container}><div className={styles.skeleton} /></div>;
  }

  return (
    <div className={styles.container}>
      <header className={styles.header}>
        <h2>设置</h2>
      </header>

      <div className={styles.tabs}>
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`${styles.tab} ${activeTab === tab.id ? styles.active : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      <div className={styles.content}>
        {activeTab === "providers" && (
          <ProvidersPanel
            providers={providers}
            loading={loading}
            editingKey={editingKey}
            newKey={newKey}
            setEditingKey={setEditingKey}
            setNewKey={setNewKey}
            onSaveKey={handleSaveKey}
            onDeleteKey={handleDeleteKey}
          />
        )}
        {activeTab === "theme" && <ThemePanel theme={theme} onThemeChange={handleThemeChange} />}
        {activeTab === "advanced" && <AdvancedPanel />}
        {activeTab === "about" && <AboutPanel />}
      </div>
    </div>
  );
}

function ProvidersPanel({
  providers,
  loading,
  editingKey,
  newKey,
  setEditingKey,
  setNewKey,
  onSaveKey,
  onDeleteKey,
}: {
  providers: Record<string, { name: string; hasKey: boolean; model: string }>;
  loading: boolean;
  editingKey: string | null;
  newKey: string;
  setEditingKey: (k: string | null) => void;
  setNewKey: (k: string) => void;
  onSaveKey: (id: string) => void;
  onDeleteKey: (id: string) => void;
}) {
  if (loading) return <div className={styles.skeleton} />;

  return (
    <div className={styles.panel}>
      <h3>API Providers</h3>
      <p className={styles.hint}>配置 LLM Provider 的 API Key。本地存储，仅在调用时发送。</p>
      <div className={styles.providerList}>
        {Object.entries(providers).map(([id, provider]) => (
          <div key={id} className={styles.providerCard}>
            <div className={styles.providerInfo}>
              <div className={styles.providerHeader}>
                <span className={styles.providerName}>{provider.name}</span>
                <span className={styles.providerModel}>{provider.model}</span>
              </div>
              <div className={styles.providerStatus}>
                <span className={styles.statusDot} style={{ background: provider.hasKey ? "var(--success)" : "var(--warning)" }} />
                <span>{provider.hasKey ? "已配置" : "未配置"}</span>
              </div>
            </div>
            <div className={styles.providerActions}>
              {editingKey === id ? (
                <>
                  <input
                    type="password"
                    value={newKey}
                    onChange={(e) => setNewKey(e.target.value)}
                    placeholder="输入 API Key"
                    className={styles.keyInput}
                    autoFocus
                  />
                  <button className={styles.btnPrimary} onClick={() => onSaveKey(id)}>保存</button>
                  <button className={styles.btnSecondary} onClick={() => setEditingKey(null)}>取消</button>
                </>
              ) : provider.hasKey ? (
                <button className={styles.btnDanger} onClick={() => onDeleteKey(id)}>删除 Key</button>
              ) : (
                <button className={styles.btnPrimary} onClick={() => setEditingKey(id)}>添加 Key</button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function ThemePanel({ theme, onThemeChange }: { theme: "light" | "dark" | "system"; onThemeChange: (t: "light" | "dark" | "system") => void }) {
  const options = [
    { value: "light" as const, label: "浅色", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="4"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1M4.5 4.5l.7.7M8.8 8.8l.7.7M4.5 9.5l.7-.7M8.8 5.2l.7-.7" strokeLinecap="round"/></svg> },
    { value: "dark" as const, label: "深色", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M7 12a5 5 0 010-10 5 5 0 000 10z"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
    { value: "system" as const, label: "跟随系统", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="3" y="3" width="8" height="8" rx="1"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
  ];

  return (
    <div className={styles.panel}>
      <h3>主题模式</h3>
      <div className={styles.themeOptions}>
        {options.map((opt) => (
          <button
            key={opt.value}
            className={`${styles.themeOption} ${theme === opt.value ? styles.active : ""}`}
            onClick={() => onThemeChange(opt.value)}
          >
            {opt.icon}
            <span>{opt.label}</span>
            {theme === opt.value && <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="var(--accent-primary)" strokeWidth="2"><path d="M3 7l3 3 5-5" strokeLinecap="round" strokeLinejoin="round"/></svg>}
          </button>
        ))}
      </div>
    </div>
  );
}

function AdvancedPanel() {
  return (
    <div className={styles.panel}>
      <h3>高级设置</h3>
      <div className={styles.advancedGrid}>
        <div className={styles.advancedCard}>
          <h4>进化循环</h4>
          <label className={styles.toggleLabel}>
            <input type="checkbox" defaultChecked /> 自动进化
          </label>
          <label className={styles.toggleLabel}>
            <input type="checkbox" /> 自动修复
          </label>
        </div>
        <div className={styles.advancedCard}>
          <h4>上下文管理</h4>
          <label className={styles.toggleLabel}>
            <input type="checkbox" defaultChecked /> 自动压缩
          </label>
          <label className={styles.toggleLabel}>
            <input type="checkbox" /> 保留完整历史
          </label>
        </div>
        <div className={styles.advancedCard}>
          <h4>开发者</h4>
          <label className={styles.toggleLabel}>
            <input type="checkbox" /> 调试模式
          </label>
          <label className={styles.toggleLabel}>
            <input type="checkbox" /> 显示原始事件
          </label>
        </div>
      </div>
    </div>
  );
}

function AboutPanel() {
  return (
    <div className={styles.panel}>
      <h3>关于 NeoCodex</h3>
      <div className={styles.aboutInfo}>
        <div className={styles.aboutRow}><span>版本</span><span>0.1.0-dev</span></div>
        <div className={styles.aboutRow}><span>架构</span><span>Rust + Tauri 2 + React 18</span></div>
        <div className={styles.aboutRow}><span>核心</span><span>NeoTrix 自进化架构</span></div>
        <div className={styles.aboutRow}><span>协议</span><span>ReAct + EvolutionLoop + SelfAudit</span></div>
      </div>
      <div className={styles.links}>
        <a href="https://github.com/neotrix" target="_blank" rel="noopener">GitHub</a>
        <a href="https://docs.neotrix.ai" target="_blank" rel="noopener">文档</a>
        <a href="https://discord.gg/neotrix" target="_blank" rel="noopener">Discord</a>
      </div>
    </div>
  );
}

export default SettingsView;