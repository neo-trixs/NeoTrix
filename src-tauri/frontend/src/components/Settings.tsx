import React, { useState } from "react";
import type { AppSettings } from "../types";
import ProviderConfig from "./ProviderConfig";
import type { ProviderConfig as ProviderConfigType } from "../types";
import ProviderStatusPanel from "./ProviderStatusPanel";
import KnowledgeBase from "./KnowledgeBase";
import type { KnowledgeEntry } from "../types";
import styles from "./Settings.module.css";

interface Props {
  settings: AppSettings;
  providerConfig: ProviderConfigType;
  knowledgeBase: KnowledgeEntry[];
  onSaveSettings: (settings: AppSettings) => void;
  onSaveProvider: (config: ProviderConfigType) => void;
  onTestProvider?: (config: ProviderConfigType) => Promise<boolean>;
  onAddKnowledge: (entry: Omit<KnowledgeEntry, "id" | "created" | "updated">) => void;
  onDeleteKnowledge: (id: string) => void;
  onSearchKnowledge: (query: string) => void;
  onClose: () => void;
}

type SettingsTab = "provider" | "general" | "knowledge" | "api" | "privacy" | "shortcuts" | "about";

const TABS: { id: SettingsTab; label: string }[] = [
  { id: "provider", label: "Provider" },
  { id: "general", label: "通用" },
  { id: "api", label: "API" },
  { id: "knowledge", label: "知识库" },
  { id: "privacy", label: "隐私" },
  { id: "shortcuts", label: "快捷键" },
  { id: "about", label: "关于" },
];

const Settings: React.FC<Props> = ({
  settings,
  providerConfig,
  knowledgeBase,
  onSaveSettings,
  onSaveProvider,
  onTestProvider,
  onAddKnowledge,
  onDeleteKnowledge,
  onSearchKnowledge,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [localSettings, setLocalSettings] = useState(settings);

  const handleSaveSettings = () => {
    onSaveSettings(localSettings);
    onClose();
  };

  return (
    <div className={styles.settingsOverlay} onClick={onClose}>
      <div className={`${styles.settingsPanel} glass-panel`} role="dialog" aria-modal="true" aria-label="Settings" onClick={(e) => e.stopPropagation()}>
        <div className={styles.settingsHeader}>
          <h2>设置</h2>
          <button className="btn-icon" onClick={onClose} aria-label="Close settings">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>

        <div className={styles.settingsLayout}>
          <div className={styles.settingsSidebar} role="tablist" aria-label="Settings tabs">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                className={`${styles.settingsTab}${activeTab === tab.id ? ` ${styles.settingsTabActive}` : ""}`}
                role="tab"
                aria-selected={activeTab === tab.id}
                onClick={() => setActiveTab(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </div>

          <div className={styles.settingsBody}>
            {activeTab === "provider" && (
              <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
                <ProviderConfig config={providerConfig} onSave={onSaveProvider} onTest={onTestProvider} />
                <ProviderStatusPanel />
              </div>
            )}

            {activeTab === "general" && (
              <div className={styles.generalSettings}>
                <div className={styles.settingsGroup}>
                  <label>主题</label>
                  <select value={localSettings.theme} onChange={(e) => setLocalSettings({ ...localSettings, theme: e.target.value as AppSettings["theme"] })}>
                    <option value="light">浅色</option>
                    <option value="dark">深色</option>
                    <option value="system">跟随系统</option>
                  </select>
                </div>

                <div className={styles.settingsGroup}>
                  <label>字体大小 ({localSettings.fontSize}px)</label>
                  <input type="range" min="11" max="20" step="1" value={localSettings.fontSize} onChange={(e) => setLocalSettings({ ...localSettings, fontSize: parseInt(e.target.value) })} />
                </div>

                <div className={styles.settingsGroup}>
                  <label>语言</label>
                  <select value={localSettings.language} onChange={(e) => setLocalSettings({ ...localSettings, language: e.target.value as AppSettings["language"] })}>
                    <option value="zh-CN">中文</option>
                    <option value="en-US">English</option>
                  </select>
                </div>

                <div className={styles.settingsGroup}>
                  <label>默认模型</label>
                  <input type="text" value={localSettings.defaultModel} onChange={(e) => setLocalSettings({ ...localSettings, defaultModel: e.target.value })} placeholder="GatewayV2" />
                </div>

                <div className={styles.settingsGroup}>
                  <label>Temperature ({localSettings.temperature})</label>
                  <input type="range" min="0" max="2" step="0.1" value={localSettings.temperature} onChange={(e) => setLocalSettings({ ...localSettings, temperature: parseFloat(e.target.value) })} />
                </div>

                <div className={styles.settingsGroup}>
                  <label>最大 Token</label>
                  <select value={localSettings.maxTokens} onChange={(e) => setLocalSettings({ ...localSettings, maxTokens: parseInt(e.target.value) })}>
                    <option value="4096">4,096</option>
                    <option value="8192">8,192</option>
                    <option value="16384">16,384</option>
                    <option value="32768">32,768</option>
                  </select>
                </div>

                <div className={styles.settingsGroup}>
                  <label>
                    <input type="checkbox" checked={localSettings.autoSave} onChange={(e) => setLocalSettings({ ...localSettings, autoSave: e.target.checked })} />
                    <span style={{ marginLeft: 8 }}>自动保存会话</span>
                  </label>
                </div>

                <div className={styles.settingsGroup}>
                  <label>终端路径</label>
                  <input type="text" value={localSettings.terminalPath} onChange={(e) => setLocalSettings({ ...localSettings, terminalPath: e.target.value })} placeholder="/bin/zsh" />
                </div>
              </div>
            )}

            {activeTab === "api" && (
              <div className={styles.generalSettings}>
                <div className={styles.settingsGroup}>
                  <label>API Key</label>
                  <input type="password" value={providerConfig.apiKey} placeholder="输入 API Key..." style={{ width: "100%" }} onChange={() => {}} />
                  <span style={{ fontSize: 11, color: "var(--nt-text-muted)", marginTop: 4 }}>API Key 存储在系统密钥库中，不会明文保存</span>
                </div>
                <div className={styles.settingsGroup}>
                  <label>Base URL (可选)</label>
                  <input type="text" value={providerConfig.baseUrl || ""} placeholder="https://api.anthropic.com" style={{ width: "100%" }} onChange={() => {}} />
                </div>
              </div>
            )}

            {activeTab === "knowledge" && (
              <KnowledgeBase
                entries={knowledgeBase}
                onAdd={onAddKnowledge}
                onDelete={onDeleteKnowledge}
                onSearch={onSearchKnowledge}
              />
            )}

            {activeTab === "privacy" && (
              <div className={styles.generalSettings}>
                <div className={styles.settingsGroup}>
                  <label>
                    <input type="checkbox" checked={localSettings.privacyStoreMessages} onChange={(e) => setLocalSettings({ ...localSettings, privacyStoreMessages: e.target.checked })} />
                    <span style={{ marginLeft: 8 }}>对话存储</span>
                  </label>
                </div>
                <div className={styles.settingsGroup}>
                  <label>
                    <input type="checkbox" checked={localSettings.privacyTelemetry} onChange={(e) => setLocalSettings({ ...localSettings, privacyTelemetry: e.target.checked })} />
                    <span style={{ marginLeft: 8 }}>使用数据收集</span>
                  </label>
                </div>
                <div className={styles.settingsGroup}>
                  <label>
                    <input type="checkbox" checked={localSettings.privacyLocalFirst} onChange={(e) => setLocalSettings({ ...localSettings, privacyLocalFirst: e.target.checked })} />
                    <span style={{ marginLeft: 8 }}>本地处理优先</span>
                  </label>
                </div>
                <div className={styles.settingsGroup}>
                  <label>
                    <input type="checkbox" checked={localSettings.privacyPreflightCheck} onChange={(e) => setLocalSettings({ ...localSettings, privacyPreflightCheck: e.target.checked })} />
                    <span style={{ marginLeft: 8 }}>发送前隐私审查</span>
                  </label>
                </div>
              </div>
            )}

            {activeTab === "shortcuts" && (
              <div className={styles.generalSettings}>
                <div className={styles.shortcutRow}><kbd>⌘N</kbd><span>新建会话</span></div>
                <div className={styles.shortcutRow}><kbd>⌘K</kbd><span>命令面板</span></div>
                <div className={styles.shortcutRow}><kbd>⌘,</kbd><span>打开设置</span></div>
                <div className={styles.shortcutRow}><kbd>⌘B</kbd><span>切换侧栏</span></div>
                <div className={styles.shortcutRow}><kbd>⌘F</kbd><span>搜索会话</span></div>
                <div className={styles.shortcutRow}><kbd>⌘E</kbd><span>演化面板</span></div>
                <div className={styles.shortcutRow}><kbd>⌘⇧B</kbd><span>折叠侧栏</span></div>
                <div className={styles.shortcutRow}><kbd>⌘⇧[</kbd><span>上一会话</span></div>
                <div className={styles.shortcutRow}><kbd>⌘⇧]</kbd><span>下一会话</span></div>
                <div className={styles.shortcutRow}><kbd>⌘1-8</kbd><span>跳到第 N 会话</span></div>
                <div className={styles.shortcutRow}><kbd>Esc</kbd><span>关闭弹窗 / 停止生成</span></div>
              </div>
            )}

            {activeTab === "about" && (
              <div className={styles.generalSettings}>
                <div style={{ textAlign: "center", padding: "24px 0" }}>
                  <div style={{ fontSize: 24, fontWeight: 700, marginBottom: 8 }}>NeoTrix</div>
                  <div style={{ fontSize: 13, color: "var(--nt-text-muted)", marginBottom: 4 }}>AI-native developer toolkit</div>
                  <div style={{ fontSize: 12, color: "var(--nt-text-muted)", marginBottom: 16 }}>v0.19.0 · MIT License</div>
                  <div style={{ fontSize: 12, color: "var(--nt-text-muted)" }}>
                    E8 State-Space Reasoning · VSA HyperCube · GWT Attention Routing
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        <div className={styles.settingsFooter}>
          <button className="btn-secondary" onClick={onClose}>关闭</button>
          {activeTab === "general" && <button className="btn-primary" onClick={handleSaveSettings}>保存</button>}
        </div>
      </div>
    </div>
  );
};

export default Settings;
