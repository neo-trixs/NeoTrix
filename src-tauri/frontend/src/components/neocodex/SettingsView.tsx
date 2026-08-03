import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../../stores";
import type { NeoCodexProviderConfig, AppSettings } from "../../types";
import styles from "./SettingsView.module.css";

type Tab = "providers" | "theme" | "advanced" | "about" | "mcp";

export function SettingsView() {
  const settings = useStore((s) => s.settings);
  const setSettings = useStore((s) => s.setSettings);
  const [activeTab, setActiveTab] = useState<Tab>("providers");
  const [providers, setProviders] = useState<Record<string, { name: string; hasKey: boolean; model: string }>>({});
  const [loading, setLoading] = useState(true);
  const [configError, setConfigError] = useState("");
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [newKey, setNewKey] = useState("");

  const loadProviders = async () => {
    setLoading(true);
    setConfigError("");
    try {
      const configResult = await invoke<NeoCodexProviderConfig | null>("neocodex_provider_config");
      if (!configResult) {
        setConfigError("未能读取提供商配置（后端返回空）");
        return;
      }
      const map: Record<string, { name: string; hasKey: boolean; model: string }> = {};
      for (const p of configResult.providers ?? []) {
        map[p.name] = { name: p.name, hasKey: p.resolvable, model: p.model };
      }
      setProviders(map);
    } catch (e) {
      setConfigError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProviders();
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
    setSettings({ ...settings, theme: newTheme });
  };

  const handleLanguageChange = (lang: string) => {
    setSettings({ ...settings, language: lang as "zh-CN" | "en-US" });
  };

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "providers", label: "Providers", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M3 7h8M3 4h8M3 10h8" strokeLinecap="round"/></svg> },
    { id: "theme", label: "外观", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="4"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1M4.5 4.5l.7.7M8.8 8.8l.7.7M4.5 9.5l.7-.7M8.8 5.2l.7-.7" strokeLinecap="round"/></svg> },
    { id: "advanced", label: "高级", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="5"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
    { id: "about", label: "关于", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="5"/><path d="M7 5v2M7 9v.01" strokeLinecap="round"/></svg> },
    { id: "mcp", label: "MCP", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M7 2v10M2 7h10M4.5 2.5l2.5 2.5 2.5-2.5M4.5 11.5L7 9l2.5 2.5" strokeLinecap="round" strokeLinejoin="round"/></svg> },
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
            data-testid={`settings-tab-${tab.id}`}
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
            configError={configError}
            onRetry={loadProviders}
            editingKey={editingKey}
            newKey={newKey}
            setEditingKey={setEditingKey}
            setNewKey={setNewKey}
            onSaveKey={handleSaveKey}
            onDeleteKey={handleDeleteKey}
          />
        )}
        {activeTab === "theme" && <ThemePanel theme={settings.theme} onThemeChange={handleThemeChange} fontSize={settings.fontSize} onFontSizeChange={(v) => setSettings({ ...settings, fontSize: v })} language={settings.language} onLanguageChange={handleLanguageChange} accent={settings.accent} onAccentChange={(v) => setSettings({ ...settings, accent: v })} />}
        {activeTab === "advanced" && <AdvancedPanel settings={settings} onChange={(patch) => setSettings({ ...settings, ...patch })} />}
        {activeTab === "about" && <AboutPanel />}
        {activeTab === "mcp" && <McpPanel />}
      </div>
    </div>
  );
}

function ProvidersPanel({
  providers,
  loading,
  configError,
  onRetry,
  editingKey,
  newKey,
  setEditingKey,
  setNewKey,
  onSaveKey,
  onDeleteKey,
}: {
  providers: Record<string, { name: string; hasKey: boolean; model: string }>;
  loading: boolean;
  configError?: string;
  onRetry?: () => void;
  editingKey: string | null;
  newKey: string;
  setEditingKey: (k: string | null) => void;
  setNewKey: (k: string) => void;
  onSaveKey: (id: string) => void;
  onDeleteKey: (id: string) => void;
}) {
  if (loading) return <div className={styles.skeleton} />;

  if (configError) {
    return (
      <div className={styles.panel}>
        <h3>API Providers</h3>
        <div className={styles.errorBox}>
          <span>无法加载提供商配置：{configError}</span>
          {onRetry && <button className={styles.btnPrimary} onClick={onRetry}>重试</button>}
        </div>
      </div>
    );
  }

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

function ThemePanel({ theme, onThemeChange, fontSize, onFontSizeChange, language, onLanguageChange, accent, onAccentChange }: { theme: "light" | "dark" | "system"; onThemeChange: (t: "light" | "dark" | "system") => void; fontSize: number; onFontSizeChange: (v: number) => void; language: string; onLanguageChange: (v: string) => void; accent: string; onAccentChange: (v: string) => void }) {
  const options = [
    { value: "light" as const, label: "浅色", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><circle cx="7" cy="7" r="4"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1M4.5 4.5l.7.7M8.8 8.8l.7.7M4.5 9.5l.7-.7M8.8 5.2l.7-.7" strokeLinecap="round"/></svg> },
    { value: "dark" as const, label: "深色", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M7 12a5 5 0 010-10 5 5 0 000 10z"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
    { value: "system" as const, label: "跟随系统", icon: <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="3" y="3" width="8" height="8" rx="1"/><path d="M7 3v1M7 10v1M3 7h1M10 7h1" strokeLinecap="round"/></svg> },
  ];

  const accents = [
    { value: "default", label: "默认" },
    { value: "red", label: "绯红" },
    { value: "amber", label: "琥珀" },
    { value: "emerald", label: "翡翠" },
    { value: "cyan", label: "青碧" },
    { value: "violet", label: "紫罗兰" },
    { value: "blue", label: "蔚蓝" },
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
      <div className={styles.fontSizeRow}>
        <span className={styles.fontSizeLabel}>强调色</span>
        <div className={styles.accentOptions}>
          {accents.map((a) => (
            <button
              key={a.value}
              type="button"
              data-testid={`settings-accent-${a.value}`}
              className={`${styles.accentSwatch} ${accent === a.value ? styles.accentSwatchActive : ""}`}
              title={a.label}
              onClick={() => onAccentChange(a.value)}
              style={a.value !== "default" ? { background: { red: "#FF6B6B", amber: "#FFB74D", emerald: "#66BB6A", cyan: "#4DD0E1", violet: "#B388FF", blue: "#64B5F6" }[a.value] } : undefined}
            >
              {a.value === "default" ? "默认" : ""}
            </button>
          ))}
        </div>
      </div>
      <div className={styles.fontSizeRow}>
        <span className={styles.fontSizeLabel}>字体大小</span>
        <div className={styles.fontSizeControls}>
          <button className={styles.fontSizeBtn} onClick={() => onFontSizeChange(Math.max(11, fontSize - 1))} title="减小">A-</button>
          <span className={styles.fontSizeValue}>{fontSize}px</span>
          <button className={styles.fontSizeBtn} onClick={() => onFontSizeChange(Math.min(20, fontSize + 1))} title="增大">A+</button>
        </div>
      </div>
      <div className={styles.fontSizeRow}>
        <span className={styles.fontSizeLabel}>语言</span>
        <select
          className={styles.selectField}
          value={language}
          onChange={(e) => onLanguageChange(e.target.value)}
          data-testid="settings-language"
        >
          <option value="zh-CN">简体中文</option>
          <option value="en-US">English</option>
        </select>
      </div>
    </div>
  );
}

function AdvancedPanel({ settings, onChange }: { settings: AppSettings; onChange: (patch: Partial<AppSettings>) => void }) {
  const bools: Array<{ key: keyof AppSettings; label: string }> = [
    { key: "autoSave", label: "自动进化" },
    { key: "privacyPreflightCheck", label: "自动修复" },
    { key: "privacyLocalFirst", label: "自动压缩" },
    { key: "privacyStoreMessages", label: "保留完整历史" },
    { key: "privacyTelemetry", label: "调试模式" },
    { key: "notifyOnComplete", label: "任务完成时发送系统通知" },
  ];
  const groups: Array<{ title: string; keys: (keyof AppSettings)[] }> = [
    { title: "进化循环", keys: ["autoSave", "privacyPreflightCheck"] },
    { title: "上下文管理", keys: ["privacyLocalFirst", "privacyStoreMessages"] },
    { title: "通知", keys: ["notifyOnComplete"] },
    { title: "开发者", keys: ["privacyTelemetry"] },
  ];
  const numberField = (key: keyof AppSettings, label: string, min: number, max: number, step = 1) => (
    <label key={key} className={styles.fieldLabel}>
      <span>{label}</span>
      <input
        type="number"
        min={min}
        max={max}
        step={step}
        value={Number(settings[key]) || 0}
        onChange={(e) => onChange({ [key]: Number(e.target.value) } as Partial<AppSettings>)}
        data-testid={`settings-${String(key)}`}
      />
    </label>
  );
  return (
    <div className={styles.panel}>
      <h3>高级设置</h3>
      <div className={styles.advancedGrid}>
        {groups.map((g) => (
          <div key={g.title} className={styles.advancedCard}>
            <h4>{g.title}</h4>
            {g.keys.map((k) => {
              const b = bools.find((x) => x.key === k)!;
              return (
                <label key={k} className={styles.toggleLabel}>
                  <input
                    type="checkbox"
                    checked={Boolean(settings[k])}
                    onChange={(e) => onChange({ [k]: e.target.checked } as Partial<AppSettings>)}
                  />{" "}
                  {b.label}
                </label>
              );
            })}
          </div>
        ))}
        <div className={styles.advancedCard}>
          <h4>模型与对话</h4>
          <label className={styles.fieldLabel}>
            <span>默认模型</span>
            <input
              type="text"
              value={String(settings.defaultModel || "")}
              onChange={(e) => onChange({ defaultModel: e.target.value })}
              data-testid="settings-defaultModel"
            />
          </label>
          {numberField("temperature", "温度 (0–2)", 0, 2, 0.1)}
          {numberField("maxTokens", "最大 Token", 512, 200000, 256)}
          {numberField("maxSessions", "最大会话数", 1, 100)}
          <label className={styles.fieldLabel}>
            <span>终端路径</span>
            <input
              type="text"
              value={String(settings.terminalPath || "")}
              onChange={(e) => onChange({ terminalPath: e.target.value })}
              placeholder="/bin/zsh"
              data-testid="settings-terminalPath"
            />
          </label>
        </div>
      </div>
    </div>
  );
}

function AboutPanel() {
  const [version, setVersion] = useState("—");
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [update, setUpdate] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [latest, setLatest] = useState<string | null>(null);
  const addNotification = useStore((s) => s.addNotification);

  useEffect(() => {
    invoke<string>("neocodex_app_version").then(setVersion).catch(() => {});
  }, []);

  const checkUpdate = async () => {
    setChecking(true);
    setUpdate(null);
    setError(null);
    setLatest(null);
    try {
      const res = await invoke<any>("neocodex_check_update");
      if (!res) {
        setError("检查更新失败：后端无响应");
      } else if (res.error) {
        setError(`检查失败: ${res.error}`);
      } else if (res.available) {
        setLatest(res.latest);
        setUpdate(`发现新版本 v${res.latest}（当前 v${res.current}）。`);
      } else {
        setUpdate("已是最新版本。");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setChecking(false);
    }
  };

  const downloadUpdate = async () => {
    setDownloading(true);
    setError(null);
    try {
      await invoke("neocodex_download_update");
      addNotification({ type: "success", message: "更新已下载，应用即将重启", duration: 5000 });
    } catch (e) {
      setError(String(e));
      addNotification({ type: "error", message: `下载更新失败: ${e}`, duration: 6000 });
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className={styles.panel}>
      <h3>关于 NeoCodex</h3>
      <div className={styles.aboutInfo}>
        <div className={styles.aboutRow}><span>版本</span><span>v{version}</span></div>
        <div className={styles.aboutRow}><span>架构</span><span>Rust + Tauri 2 + React 18</span></div>
        <div className={styles.aboutRow}><span>核心</span><span>NeoTrix 自进化架构</span></div>
        <div className={styles.aboutRow}><span>协议</span><span>ReAct + EvolutionLoop + SelfAudit</span></div>
      </div>
      <div className={styles.updateRow}>
        <button type="button" className={styles.updateBtn} onClick={checkUpdate} disabled={checking || downloading} data-testid="about-check-update">
          {checking ? "检查中…" : "检查更新"}
        </button>
        {latest && (
          <button type="button" className={styles.updateBtn} onClick={downloadUpdate} disabled={downloading} data-testid="about-download-update">
            {downloading ? "下载中…" : "立即下载并重启"}
          </button>
        )}
        {update && <span className={styles.updateOk}>{update}</span>}
        {error && <span className={styles.updateErr}>{error}</span>}
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

interface McpServerInfo { name: string; transport: string; tool_count: number; healthy: boolean }
interface McpToolInfo { name: string; description: string; server: string }

function McpPanel() {
  const [servers, setServers] = useState<McpServerInfo[]>([]);
  const [tools, setTools] = useState<McpToolInfo[]>([]);
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");
  const [args, setArgs] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const addNotification = useStore((s) => s.addNotification);

  const load = async () => {
    try {
      const [s, t] = await Promise.all([
        invoke<McpServerInfo[]>("neocodex_mcp_list"),
        invoke<McpToolInfo[]>("neocodex_mcp_tools"),
      ]);
      setServers(s || []);
      setTools(t || []);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => { load(); }, []);

  const handleRegister = async () => {
    if (!name.trim() || !command.trim()) {
      setError("服务名与命令不能为空");
      return;
    }
    setBusy(true);
    setError("");
    try {
      const argList = args.split(/\s+/).filter(Boolean);
      const list = await invoke<McpServerInfo[]>("neocodex_mcp_register", { name, command, args: argList });
      setServers(list || []);
      setName(""); setCommand(""); setArgs("");
      addNotification({ type: "success", message: `MCP 服务 ${name} 已注册`, duration: 3000 });
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.mcpTitle}>MCP 服务器</div>
      <p className={styles.hint}>注册 MCP 服务器后，其工具可通过 mcp_call 供 Agent 调用（Codex/Claude MCP 对标）。</p>
      <div className={styles.mcpRow}>
        <input type="text" placeholder="服务名 (如 filesystem)" value={name} onChange={(e) => setName(e.target.value)} className={styles.mcpInput} data-testid="mcp-name" />
      </div>
      <div className={styles.mcpRow}>
        <input type="text" placeholder="命令 (如 npx)" value={command} onChange={(e) => setCommand(e.target.value)} className={styles.mcpInput} data-testid="mcp-command" />
      </div>
      <div className={styles.mcpRow}>
        <input type="text" placeholder="参数 (空格分隔，可选)" value={args} onChange={(e) => setArgs(e.target.value)} className={styles.mcpInput} data-testid="mcp-args" />
      </div>
      <button type="button" className={styles.updateBtn} onClick={handleRegister} disabled={busy} data-testid="mcp-register">
        {busy ? "注册中…" : "注册 Stdio 服务器"}
      </button>

      {error && <div className={styles.updateErr}>{error}</div>}

      <div className={styles.mcpSectionSub}>已注册 ({servers.length})</div>
      {servers.length === 0 && <div className={styles.mcpEmpty}>尚未注册 MCP 服务器</div>}
      {servers.map((s) => (
        <div key={s.name} className={styles.mcpServer} data-testid={`mcp-server-${s.name}`}>
          <span className={styles.mcpServerName}>{s.name}</span>
          <span className={styles.mcpServerMeta}>{s.transport} · {s.tool_count} 工具{s.healthy ? "" : " · 未健康检查"}</span>
        </div>
      ))}

      <div className={styles.mcpSectionSub}>可用工具 ({tools.length})</div>
      {tools.length === 0 && <div className={styles.mcpEmpty}>无可用 MCP 工具</div>}
      {tools.map((t) => (
        <div key={`${t.server}:${t.name}`} className={styles.mcpTool} data-testid={`mcp-tool-${t.name}`}>
          <span className={styles.mcpToolName}>{t.name}</span>
          <span className={styles.mcpToolServer}>{t.server}</span>
          <span className={styles.mcpToolDesc}>{t.description}</span>
        </div>
      ))}
    </div>
  );
}