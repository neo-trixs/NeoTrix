import React, { useState } from "react";
import type { ProviderConfig as ProviderConfigType, ProviderId } from "../types";

interface Props {
  config: ProviderConfigType;
  onSave: (config: ProviderConfigType) => void;
  onTest?: (config: ProviderConfigType) => Promise<boolean>;
}

const PROVIDER_OPTIONS: { id: ProviderId; name: string }[] = [
  { id: "anthropic", name: "Anthropic Claude" },
  { id: "openai", name: "OpenAI" },
  { id: "gemini", name: "Google Gemini" },
  { id: "ollama", name: "Ollama (本地)" },
];

/**
 * ProviderConfig — LLM provider configuration panel.
 * Supports provider selection, model config, API key, base URL, learning rate, and connection testing.
 */
const ProviderConfig: React.FC<Props> = ({ config, onSave, onTest }) => {
  const [provider, setProvider] = useState(config.id);
  const [model, setModel] = useState(config.model);
  const [apiKey, setApiKey] = useState(config.apiKey);
  const [baseUrl, setBaseUrl] = useState(config.baseUrl || "");
  const [learningRate, setLearningRate] = useState(config.learningRate);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"success" | "fail" | null>(null);

  const handleSave = () => {
    onSave({ id: provider, name: PROVIDER_OPTIONS.find((p) => p.id === provider)?.name || provider, model, apiKey, baseUrl: baseUrl || undefined, learningRate });
  };

  const handleTest = async () => {
    if (!onTest) return;
    setTesting(true);
    setTestResult(null);
    try {
      const ok = await onTest({ id: provider, name: provider, model, apiKey, baseUrl: baseUrl || undefined, learningRate });
      setTestResult(ok ? "success" : "fail");
    } catch {
      setTestResult("fail");
    }
    setTesting(false);
  };

  return (
    <div className="provider-config">
      <h3 className="provider-title">LLM Provider 配置</h3>

      <div className="settings-group">
        <label>Provider</label>
        <select value={provider} onChange={(e) => setProvider(e.target.value as ProviderId)}>
          {PROVIDER_OPTIONS.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
      </div>

      <div className="settings-group">
        <label>模型</label>
        <input type="text" value={model} onChange={(e) => setModel(e.target.value)} placeholder="模型名称" />
      </div>

      <div className="settings-group">
        <label>API Key</label>
        <input type="password" value={apiKey} onChange={(e) => setApiKey(e.target.value)} placeholder="sk-..." />
      </div>

      <div className="settings-group">
        <label>Base URL (可选)</label>
        <input type="text" value={baseUrl} onChange={(e) => setBaseUrl(e.target.value)} placeholder="https://api.example.com" />
      </div>

      <div className="settings-group">
        <label>学习率 ({learningRate.toFixed(2)})</label>
        <input type="range" min="0.01" max="0.5" step="0.01" value={learningRate} onChange={(e) => setLearningRate(parseFloat(e.target.value))} />
      </div>

      <div className="provider-actions">
        {onTest && (
          <button className="btn-secondary" onClick={handleTest} disabled={testing}>
            {testing ? "测试中..." : testResult === "success" ? "✅ 连接成功" : testResult === "fail" ? "❌ 连接失败" : "测试连接"}
          </button>
        )}
        <button className="btn-primary" onClick={handleSave}>保存配置</button>
      </div>
    </div>
  );
};

export default ProviderConfig;
