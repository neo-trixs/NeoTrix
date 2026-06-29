import React, { useState } from "react";
import { useStore } from "../store";
import * as api from "../lib/api";
import type { ProviderConfig } from "../types";

const STEPS = [
  {
    title: "Welcome to NeoTrix",
    icon: "N",
    description:
      "Your AI-native agent desktop. NeoTrix brings together a consciousness-driven workflow engine, multi-model provider support, and deep project integration into one seamless environment.",
    tips: [
      "Powered by E8 64-state reasoning kernel",
      "VSA 4096-bit unified vector representation",
      "Self-evolving through SEAL meta-layer",
    ],
  },
  {
    title: "Configure Provider",
    icon: "P",
    description: "Connect an AI provider to get started. Your API key stays local.",
  },
  {
    title: "Open a Project",
    icon: "F",
    description: "Point NeoTrix to your project folder for context-aware assistance.",
  },
  {
    title: "You're All Set!",
    icon: "✓",
    description: "Your NeoTrix environment is ready. Here are a few tips to get started.",
    tips: [
      "Cmd+K opens the command palette",
      "Cmd+L focuses the input field",
      "Shift+Cmd+E toggles the evolution dashboard",
      "Shift+Cmd+R toggles project rules",
      "Type / for keyboard shortcuts anytime",
    ],
  },
];

const MODELS: { provider: ProviderConfig["id"]; name: string; label: string; models: string[] }[] = [
  { provider: "anthropic", name: "Anthropic", label: "Claude", models: ["claude-sonnet-4-20250514", "claude-3-opus-20240229", "claude-3-haiku-20240307"] },
  { provider: "openai", name: "OpenAI", label: "GPT", models: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo"] },
];

const Onboarding: React.FC = () => {
  const setShowOnboarding = useStore((s) => s.setShowOnboarding);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const setProjectPath = useStore((s) => s.setProjectPath);
  const statusText = useStore((s) => s.statusText);

  const [step, setStep] = useState(0);
  const [provider, setProvider] = useState<ProviderConfig["id"]>("anthropic");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("claude-sonnet-4-20250514");
  const [projectPathValue, setProjectPathValue] = useState("");
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<"idle" | "success" | "error">("idle");

  const providerInfo = MODELS.find((m) => m.provider === provider);

  const handleProviderChange = (newProvider: ProviderConfig["id"]) => {
    setProvider(newProvider);
    const info = MODELS.find((m) => m.provider === newProvider);
    if (info) setModel(info.models[0]);
    setTestResult("idle");
  };

  const handleTestConnection = async () => {
    if (!apiKey.trim()) return;
    setTesting(true);
    setTestResult("idle");
    try {
      const config: ProviderConfig = {
        id: provider,
        name: providerInfo?.name || "",
        model,
        apiKey: apiKey.trim(),
        learningRate: 0.05,
      };
      const ok = await api.testProviderConnection(config);
      setTestResult(ok ? "success" : "error");
    } catch {
      setTestResult("error");
    }
    setTesting(false);
  };

  const handleSelectProject = async () => {
    const path = await api.openFileDialog();
    if (path) {
      setProjectPathValue(path);
    }
  };

  const handleComplete = () => {
    if (apiKey.trim()) {
      const config: ProviderConfig = {
        id: provider,
        name: providerInfo?.name || "",
        model,
        apiKey: apiKey.trim(),
        learningRate: 0.05,
      };
      setProviderConfig(config);
    }
    if (projectPathValue) {
      setProjectPath(projectPathValue);
    }
    try {
      localStorage.setItem("neotrix_onboarding_done", "true");
    } catch {}
    setShowOnboarding(false);
  };

  const isLastStep = step === STEPS.length - 1;
  const canProceed = (() => {
    if (step === 1) return apiKey.trim().length > 0;
    return true;
  })();

  const handleSkip = () => {
    try {
      localStorage.setItem("neotrix_onboarding_done", "true");
    } catch {}
    setShowOnboarding(false);
  };

  return (
    <div className="overlay">
      <div className="onboarding-dialog overlay-panel">
        <div className="onboarding-header">
          <button className="btn-ghost btn-sm" onClick={handleSkip}>
            Skip
          </button>
          <span className="onboarding-step-counter">
            {step + 1} / {STEPS.length}
          </span>
        </div>

        <div className="onboarding-body">
          <div className="onboarding-icon-wrapper">
            <div className="onboarding-icon">{STEPS[step].icon}</div>
          </div>

          <h2 className="onboarding-step-title">{STEPS[step].title}</h2>
          <p className="onboarding-step-desc">{STEPS[step].description}</p>

          {step === 0 && STEPS[step].tips && (
            <div className="onboarding-tips">
              {STEPS[step].tips!.map((tip, i) => (
                <div key={i} className="onboarding-tip">
                  <span className="onboarding-tip-bullet">✦</span>
                  <span>{tip}</span>
                </div>
              ))}
            </div>
          )}

          {step === 1 && (
            <div className="onboarding-provider-form">
              <div className="onboarding-form-row">
                <label className="onboarding-form-label">Provider</label>
                <div className="onboarding-provider-tabs">
                  {MODELS.map((m) => (
                    <button
                      key={m.provider}
                      className={`onboarding-provider-tab ${provider === m.provider ? "active" : ""}`}
                      onClick={() => handleProviderChange(m.provider)}
                    >
                      {m.name}
                    </button>
                  ))}
                </div>
              </div>
              <div className="onboarding-form-row">
                <label className="onboarding-form-label">API Key</label>
                <input
                  className="onboarding-input"
                  type="password"
                  placeholder={provider === "anthropic" ? "sk-ant-..." : "sk-..."}
                  value={apiKey}
                  onChange={(e) => { setApiKey(e.target.value); setTestResult("idle"); }}
                />
              </div>
              <div className="onboarding-form-row">
                <label className="onboarding-form-label">Model</label>
                <select className="onboarding-select" value={model} onChange={(e) => setModel(e.target.value)}>
                  {(providerInfo?.models || []).map((m) => (
                    <option key={m} value={m}>{m}</option>
                  ))}
                </select>
              </div>
              <div className="onboarding-form-actions">
                <button
                  className="btn btn-secondary btn-sm"
                  onClick={handleTestConnection}
                  disabled={testing || !apiKey.trim()}
                >
                  {testing ? "Testing..." : "Test connection"}
                </button>
                {testResult === "success" && (
                  <span className="onboarding-test-ok">Connected ✓</span>
                )}
                {testResult === "error" && (
                  <span className="onboarding-test-err">Connection failed</span>
                )}
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="onboarding-project-section">
              <div className="onboarding-project-display">
                {projectPathValue ? (
                  <span className="onboarding-project-path">{projectPathValue}</span>
                ) : (
                  <span className="onboarding-project-placeholder">No project selected</span>
                )}
              </div>
              <button className="btn btn-secondary" onClick={handleSelectProject}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                </svg>
                Browse files
              </button>
            </div>
          )}

          {step === 3 && STEPS[step].tips && (
            <div className="onboarding-tips">
              {STEPS[step].tips!.map((tip, i) => (
                <div key={i} className="onboarding-tip">
                  <span className="onboarding-tip-bullet">⌘</span>
                  <span>{tip}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="onboarding-footer">
          <div className="onboarding-dots">
            {STEPS.map((_, i) => (
              <div key={i} className={`onboarding-dot ${i === step ? "active" : ""} ${i < step ? "done" : ""}`} />
            ))}
          </div>
          <div className="onboarding-nav">
            {step > 0 && (
              <button className="btn btn-secondary btn-sm" onClick={() => setStep(step - 1)}>
                ← Back
              </button>
            )}
            {isLastStep ? (
              <button className="btn btn-primary btn-sm" onClick={handleComplete}>
                Get started →
              </button>
            ) : (
              <button
                className="btn btn-primary btn-sm"
                disabled={!canProceed}
                onClick={() => setStep(step + 1)}
              >
                Next →
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Onboarding;
