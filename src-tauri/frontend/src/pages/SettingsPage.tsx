import React, { useState, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useStore } from "../stores";
import * as api from "../lib/api";
import Settings from "../components/Settings";
import ProviderConfig from "../components/ProviderConfig";
import ProviderStatusPanel from "../components/ProviderStatusPanel";
import KnowledgeBase from "../components/KnowledgeBase";
import type { ProviderConfig as ProviderConfigType, KnowledgeEntry, AppSettings } from "../types";
import "./SettingsPage.styles.css";

type SectionId = "profile" | "appearance" | "provider" | "knowledge" | "shortcuts" | "legacy";

interface SidebarItem {
  id: SectionId;
  label: string;
  icon: string;
}

const SIDEBAR_GROUPS: { label: string; items: SidebarItem[] }[] = [
  {
    label: "General",
    items: [
      { id: "profile", label: "Profile", icon: "user" },
      { id: "appearance", label: "Appearance", icon: "sun" },
      { id: "shortcuts", label: "Shortcuts", icon: "keyboard" },
    ],
  },
  {
    label: "System",
    items: [
      { id: "provider", label: "Provider", icon: "server" },
      { id: "knowledge", label: "Knowledge Base", icon: "database" },
      { id: "legacy", label: "Advanced", icon: "settings" },
    ],
  },
];

const SECTION_ICONS: Record<string, JSX.Element> = {
  user: (
    <svg viewBox="0 0 16 16">
      <path d="M8 8a3 3 0 100-6 3 3 0 000 6z" />
      <path d="M13 14c0-2.21-2.239-4-5-4s-5 1.79-5 4" />
    </svg>
  ),
  sun: (
    <svg viewBox="0 0 16 16">
      <circle cx="8" cy="8" r="3" />
      <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.05 3.05l1.41 1.41M11.54 11.54l1.41 1.41M3.05 12.95l1.41-1.41M11.54 4.46l1.41-1.41" />
    </svg>
  ),
  keyboard: (
    <svg viewBox="0 0 16 16">
      <rect x="1" y="3" width="14" height="10" rx="1.5" />
      <path d="M4 6h1M7 6h1M10 6h1M4 8.5h1M7 8.5h1M10 8.5h1M5.5 11h4" />
    </svg>
  ),
  server: (
    <svg viewBox="0 0 16 16">
      <rect x="1" y="1" width="14" height="5.5" rx="1" />
      <rect x="1" y="9.5" width="14" height="5.5" rx="1" />
      <circle cx="3.5" cy="3.75" r="0.75" />
      <circle cx="3.5" cy="12.25" r="0.75" />
    </svg>
  ),
  database: (
    <svg viewBox="0 0 16 16">
      <ellipse cx="8" cy="4" rx="6" ry="2" />
      <path d="M2 4v4c0 1.105 2.686 2 6 2s6-.895 6-2V4" />
      <path d="M2 8v4c0 1.105 2.686 2 6 2s6-.895 6-2V8" />
    </svg>
  ),
  settings: (
    <svg viewBox="0 0 16 16">
      <circle cx="8" cy="8" r="2.5" />
      <path d="M8 1.5v1M8 13.5v1M1.5 8h1M13.5 8h1M3.52 3.52l.7.7M11.78 11.78l.7.7M3.52 12.48l.7-.7M11.78 4.22l.7-.7" strokeLinecap="round" />
    </svg>
  ),
};

const SHORTCUTS: { keys: string; label: string }[] = [
  { keys: "⌘N", label: "New session" },
  { keys: "⌘K", label: "Command palette" },
  { keys: "⌘,", label: "Open settings" },
  { keys: "⌘B", label: "Toggle sidebar" },
  { keys: "⌘F", label: "Search sessions" },
  { keys: "⌘E", label: "Evolution panel" },
  { keys: "⌘⇧B", label: "Collapse sidebar" },
  { keys: "⌘⇧[", label: "Previous session" },
  { keys: "⌘⇧]", label: "Next session" },
  { keys: "⌘1-8", label: "Jump to session N" },
  { keys: "Esc", label: "Close modal / stop generation" },
];

const SettingsPage: React.FC = () => {
  const navigate = useNavigate();

  const settings = useStore((s) => s.settings);
  const providerConfig = useStore((s) => s.providerConfig);
  const knowledgeBase = useStore((s) => s.knowledgeBase);
  const setKnowledgeBase = useStore((s) => s.setKnowledgeBase);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const setSettings = useStore((s) => s.setSettings);
  const setStatusText = useStore((s) => s.setStatusText);

  const [activeSection, setActiveSection] = useState<SectionId>("profile");
  const [showLegacy, setShowLegacy] = useState(false);

  const handleSaveSettings = (newSettings: AppSettings) => {
    setSettings(newSettings);
    setStatusText("Settings saved");
  };

  const handleSaveProvider = (config: ProviderConfigType) => {
    setProviderConfig(config);
    setStatusText(`Provider saved: ${config.name} / ${config.model}`);
  };

  const handleTestProvider = async (config: ProviderConfigType): Promise<boolean> => {
    setStatusText("Testing connection...");
    const ok = await api.testProviderConnection(config);
    setStatusText(ok ? "Connection OK" : "Connection failed");
    return ok;
  };

  const handleAddKnowledge = (entry: Omit<KnowledgeEntry, "id" | "created" | "updated">) => {
    const newEntry: KnowledgeEntry = {
      ...entry,
      id: `k-${Date.now()}`,
      created: Date.now(),
      updated: Date.now(),
    };
    setKnowledgeBase([...knowledgeBase, newEntry]);
    setStatusText(`Knowledge added: ${entry.title}`);
  };

  const handleDeleteKnowledge = (id: string) => {
    setKnowledgeBase(knowledgeBase.filter((e) => e.id !== id));
  };

  const handleSearchKnowledge = async (query: string) => {
    if (!query.trim()) return;
    setStatusText(`Searching: ${query}`);
    const results = await api.searchKnowledge(query);
    setStatusText(results.length > 0 ? `Found ${results.length} results` : "No results");
  };

  const handleClose = () => navigate("/");

  const renderIcon = (icon: string) => SECTION_ICONS[icon] || null;

  const sidebarItems = useMemo(
    () => SIDEBAR_GROUPS.flatMap((g) => g.items),
    []
  );

  const sectionTitle = sidebarItems.find((i) => i.id === activeSection)?.label ?? "Settings";

  return (
    <div className="st-overlay">
      {showLegacy && (
        <Settings
          settings={settings}
          providerConfig={providerConfig}
          knowledgeBase={knowledgeBase}
          onSaveSettings={handleSaveSettings}
          onSaveProvider={handleSaveProvider}
          onTestProvider={handleTestProvider}
          onAddKnowledge={handleAddKnowledge}
          onDeleteKnowledge={handleDeleteKnowledge}
          onSearchKnowledge={handleSearchKnowledge}
          onClose={() => setShowLegacy(false)}
        />
      )}

      <div className="st-modal" role="dialog" aria-modal="true" aria-label="Settings">
        {/* ─── Sidebar ─── */}
        <div className="st-sidebar" role="tablist" aria-label="Settings sections">
          <div className="st-search">
            <svg className="st-search-icon" viewBox="0 0 16 16">
              <circle cx="7" cy="7" r="4.5" />
              <path d="M10.5 10.5l3 3" />
            </svg>
            <input type="text" placeholder="Search settings..." />
            <span className="st-kbd">⌘K</span>
          </div>

          {SIDEBAR_GROUPS.map((group) => (
            <React.Fragment key={group.label}>
              <div className="st-grp">{group.label}</div>
              {group.items.map((item) => (
                <button
                  key={item.id}
                  className={`st-item${activeSection === item.id ? " on" : ""}`}
                  role="tab"
                  aria-selected={activeSection === item.id}
                  onClick={() => setActiveSection(item.id)}
                >
                  {renderIcon(item.icon)}
                  {item.label}
                </button>
              ))}
            </React.Fragment>
          ))}
        </div>

        {/* ─── Content ─── */}
        <div className="st-content">
          <div className="st-top">
            <h2 className="st-title">{sectionTitle}</h2>
            <button className="st-close" onClick={handleClose} aria-label="Close settings">
              <svg viewBox="0 0 16 16">
                <path d="M4 4l8 8M12 4l-8 8" />
              </svg>
            </button>
          </div>

          <div className="st-body">
            {/* ═══ Profile ═══ */}
            <div className={`st-section${activeSection === "profile" ? " open" : ""}`}>
              <h3 className="st-h2">Profile</h3>
              <div className="st-card">
                <div className="st-card-row" style={{ marginBottom: 16 }}>
                  <div className="st-av">N</div>
                  <div className="st-card-info">
                    <div className="st-lbl">NeoTrix User</div>
                    <div className="st-desc">AI-native developer toolkit</div>
                  </div>
                </div>
                <div className="st-field">
                  <label className="st-lbl">Display Name</label>
                  <input
                    className="st-input"
                    type="text"
                    defaultValue="NeoTrix User"
                    placeholder="Enter your name"
                  />
                </div>
                <div className="st-field">
                  <label className="st-lbl">Email</label>
                  <input
                    className="st-input"
                    type="text"
                    defaultValue="user@neotrix.ai"
                    placeholder="Enter your email"
                  />
                </div>
              </div>
            </div>

            {/* ═══ Appearance ═══ */}
            <div className={`st-section${activeSection === "appearance" ? " open" : ""}`}>
              <h3 className="st-h2">Appearance</h3>
              <div className="st-card">
                <div className="st-card-row" style={{ marginBottom: 12 }}>
                  <div className="st-card-info">
                    <div className="st-lbl">Theme</div>
                    <div className="st-desc">Choose light, dark, or system default</div>
                  </div>
                  <select
                    className="st-input"
                    style={{ width: 140 }}
                    value={settings.theme}
                    onChange={(e) =>
                      handleSaveSettings({
                        ...settings,
                        theme: e.target.value as AppSettings["theme"],
                      })
                    }
                  >
                    <option value="light">Light</option>
                    <option value="dark">Dark</option>
                    <option value="system">System</option>
                  </select>
                </div>
                <div className="st-card-row">
                  <div className="st-card-info">
                    <div className="st-lbl">Font Size</div>
                    <div className="st-desc">{settings.fontSize}px — code and UI text</div>
                  </div>
                  <input
                    type="range"
                    min="11"
                    max="20"
                    step="1"
                    value={settings.fontSize}
                    onChange={(e) =>
                      handleSaveSettings({
                        ...settings,
                        fontSize: parseInt(e.target.value),
                      })
                    }
                    style={{ width: 120, accentColor: "var(--nt-primary)" }}
                  />
                </div>
              </div>
              <div className="st-card">
                <div className="st-card-row" style={{ marginBottom: 12 }}>
                  <div className="st-card-info">
                    <div className="st-lbl">Language</div>
                    <div className="st-desc">Interface language</div>
                  </div>
                  <select
                    className="st-input"
                    style={{ width: 140 }}
                    value={settings.language}
                    onChange={(e) =>
                      handleSaveSettings({
                        ...settings,
                        language: e.target.value as AppSettings["language"],
                      })
                    }
                  >
                    <option value="en-US">English</option>
                    <option value="zh-CN">中文</option>
                  </select>
                </div>
              </div>
            </div>

            {/* ═══ Provider ═══ */}
            <div className={`st-section${activeSection === "provider" ? " open" : ""}`}>
              <h3 className="st-h2">Provider</h3>
              <div className="st-card" style={{ padding: 0, border: "none", background: "transparent" }}>
                <ProviderConfig config={providerConfig} onSave={handleSaveProvider} onTest={handleTestProvider} />
              </div>
              <div className="st-card" style={{ padding: 0, border: "none", background: "transparent" }}>
                <ProviderStatusPanel />
              </div>
            </div>

            {/* ═══ Knowledge Base ═══ */}
            <div className={`st-section${activeSection === "knowledge" ? " open" : ""}`}>
              <h3 className="st-h2">Knowledge Base</h3>
              <div className="st-card" style={{ padding: 0, border: "none", background: "transparent" }}>
                <KnowledgeBase
                  entries={knowledgeBase}
                  onAdd={handleAddKnowledge}
                  onDelete={handleDeleteKnowledge}
                  onSearch={handleSearchKnowledge}
                />
              </div>
            </div>

            {/* ═══ Shortcuts ═══ */}
            <div className={`st-section${activeSection === "shortcuts" ? " open" : ""}`}>
              <h3 className="st-h2">Keyboard Shortcuts</h3>
              <div className="st-card">
                {SHORTCUTS.map((sc) => (
                  <div key={sc.keys} className="st-shortcut-row">
                    <kbd>{sc.keys}</kbd>
                    <span>{sc.label}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* ═══ Legacy / Advanced ═══ */}
            <div className={`st-section${activeSection === "legacy" ? " open" : ""}`}>
              <h3 className="st-h2">Advanced Settings</h3>
              <div className="st-card">
                <div className="st-card-row">
                  <div className="st-card-info">
                    <div className="st-lbl">Full Settings Dialog</div>
                    <div className="st-desc">
                      Open the legacy full settings panel with all options including privacy, about, and API
                      configuration.
                    </div>
                  </div>
                  <button className="btn-primary" onClick={() => setShowLegacy(true)}>
                    Open
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
