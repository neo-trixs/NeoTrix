import React, { useState } from "react";
import type { AppSettings } from "../types";
import ProviderConfig from "./ProviderConfig";
import type { ProviderConfig as ProviderConfigType } from "../types";
import KnowledgeBase from "./KnowledgeBase";
import type { KnowledgeEntry } from "../types";

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

type SettingsTab = "general" | "provider" | "knowledge";

/**
 * Settings — multi-tab settings dialog (Provider config, General preferences, Knowledge base).
 */
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
  const [activeTab, setActiveTab] = useState<SettingsTab>("provider");
  const [localSettings, setLocalSettings] = useState(settings);

  const handleSaveSettings = () => {
    onSaveSettings(localSettings);
    onClose();
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel settings-panel-wide glass-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>设置</h2>
          <button className="btn-icon" onClick={onClose}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>

        <div className="settings-tabs">
          <button className={`settings-tab ${activeTab === "provider" ? "active" : ""}`} onClick={() => setActiveTab("provider")}>
            Provider
          </button>
          <button className={`settings-tab ${activeTab === "general" ? "active" : ""}`} onClick={() => setActiveTab("general")}>
            通用
          </button>
          <button className={`settings-tab ${activeTab === "knowledge" ? "active" : ""}`} onClick={() => setActiveTab("knowledge")}>
            知识库
          </button>
        </div>

        <div className="settings-body">
          {activeTab === "provider" && (
            <ProviderConfig config={providerConfig} onSave={onSaveProvider} onTest={onTestProvider} />
          )}

          {activeTab === "general" && (
            <div className="general-settings">
              <div className="settings-group">
                <label>主题</label>
                <select value={localSettings.theme} onChange={(e) => setLocalSettings({ ...localSettings, theme: e.target.value as AppSettings["theme"] })}>
                  <option value="light">浅色</option>
                  <option value="dark">深色</option>
                  <option value="system">跟随系统</option>
                </select>
              </div>

              <div className="settings-group">
                <label>字体大小 ({localSettings.fontSize}px)</label>
                <input type="range" min="11" max="20" step="1" value={localSettings.fontSize} onChange={(e) => setLocalSettings({ ...localSettings, fontSize: parseInt(e.target.value) })} />
              </div>

              <div className="settings-group">
                <label>语言</label>
                <select value={localSettings.language} onChange={(e) => setLocalSettings({ ...localSettings, language: e.target.value as AppSettings["language"] })}>
                  <option value="zh-CN">中文</option>
                  <option value="en-US">English</option>
                </select>
              </div>

              <div className="settings-group">
                <label>
                  <input type="checkbox" checked={localSettings.autoSave} onChange={(e) => setLocalSettings({ ...localSettings, autoSave: e.target.checked })} />
                  <span style={{ marginLeft: 8 }}>自动保存会话</span>
                </label>
              </div>

              <div className="settings-group">
                <label>终端路径</label>
                <input type="text" value={localSettings.terminalPath} onChange={(e) => setLocalSettings({ ...localSettings, terminalPath: e.target.value })} placeholder="/bin/zsh" />
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
        </div>

        <div className="settings-footer">
          <button className="btn-secondary" onClick={onClose}>关闭</button>
          {activeTab === "general" && <button className="btn-primary" onClick={handleSaveSettings}>保存</button>}
        </div>
      </div>
    </div>
  );
};

export default Settings;
