import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../store";
import ProviderConfig from "./ProviderConfig";
import type { ProviderConfig as ProviderConfigType } from "../types";
import * as api from "../lib/api";

const Onboarding: React.FC = () => {
  const [step, setStep] = useState(0);
  const [dontShowAgain, setDontShowAgain] = useState(false);

  const providerConfig = useStore((s) => s.providerConfig);
  const setProviderConfig = useStore((s) => s.setProviderConfig);
  const setProjectPath = useStore((s) => s.setProjectPath);
  const setShowFileTree = useStore((s) => s.setShowFileTree);
  const setShowOnboarding = useStore((s) => s.setShowOnboarding);

  const dismiss = (persist: boolean) => {
    if (persist || dontShowAgain) {
      localStorage.setItem("neotrix_onboarding_done", "true");
    }
    setShowOnboarding(false);
  };

  const handleSelectProject = async () => {
    try {
      await invoke("read_dir_recursive", { path: ".", maxDepth: 1 });
      setProjectPath(".");
      setShowFileTree(true);
    } catch {}
    dismiss(false);
  };

  const [avatarName, setAvatarName] = useState("");

  const handleSaveName = async () => {
    if (avatarName.trim()) {
      try { await api.setUserIdentity(avatarName.trim()); } catch {}
    }
    setStep(2);
  };

  const handleSaveProvider = (config: ProviderConfigType) => {
    setProviderConfig(config);
    setStep(3);
  };

  const steps = [
    {
      title: "欢迎使用 NeoTrix",
      content: (
        <div className="onboarding-step-content">
          <p>NeoTrix 是一个 AI 驱动的开发助手，具备自迭代推理能力。</p>
          <p>在开始使用前，我们需要完成一些基础配置。</p>
        </div>
      ),
    },
    {
      title: "设置你的分身身份",
      content: (
        <div className="onboarding-step-content">
          <p>为你的 AI 分身取一个名字。这个名字将用于构建你的专属用户画像，所有对话数据将加密存储在本地 Merkle 链中。</p>
          <input
            className="agent-editor-input"
            type="text"
            placeholder="输入你的分身名称..."
            value={avatarName}
            onChange={(e) => setAvatarName(e.target.value)}
            onKeyDown={(e) => { if (e.key === "Enter") handleSaveName(); }}
            style={{ marginTop: 8 }}
          />
        </div>
      ),
    },
    {
      title: "配置 LLM Provider",
      content: (
        <div className="onboarding-step-content">
          <p>请配置您的 LLM Provider API Key，以便 NeoTrix 可以正常工作。</p>
          <ProviderConfig config={providerConfig} onSave={handleSaveProvider} />
        </div>
      ),
    },
    {
      title: "选择项目文件夹",
      content: (
        <div className="onboarding-step-content">
          <p>选择一个项目文件夹作为工作区。NeoTrix 会分析项目结构并提供代码辅助。</p>
          <div className="onboarding-actions">
            <button className="btn-primary" onClick={handleSelectProject}>
              选择当前目录
            </button>
          </div>
        </div>
      ),
    },
  ];

  const current = steps[step];

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-modal glass-panel">
        <div className="onboarding-header">
          <div className="onboarding-steps">
            {steps.map((_, i) => (
              <div key={i} className={`onboarding-step-dot${i === step ? " active" : ""}${i < step ? " done" : ""}`} />
            ))}
          </div>
          <span className="onboarding-step-label">{step + 1} / {steps.length}</span>
        </div>

        <div className="onboarding-body">
          <h2 className="onboarding-title">{current.title}</h2>
          {current.content}
        </div>

        <div className="onboarding-footer">
          <label className="onboarding-checkbox">
            <input
              type="checkbox"
              checked={dontShowAgain}
              onChange={(e) => setDontShowAgain(e.target.checked)}
            />
            <span>不再显示</span>
          </label>
          <div className="onboarding-footer-actions">
            <button className="btn-secondary" onClick={() => dismiss(false)}>
              跳过
            </button>
            {step > 0 && (
              <button className="btn-secondary" onClick={() => setStep(step - 1)}>
                上一步
              </button>
            )}
            {step < steps.length - 1 && (
              <button
                className="btn-primary"
                onClick={() => {
                  if (step === 1) handleSaveName();
                  else setStep(step + 1);
                }}
              >
                下一步
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Onboarding;
