import React, { useState } from "react";
import "./ComputerUsePanel.css";

interface ComputerUsePanelProps {
  onAction: (action: string, params: Record<string, unknown>) => void;
}

export function ComputerUsePanel({ onAction }: ComputerUsePanelProps): JSX.Element {
  const [activeTab, setActiveTab] = useState("screenshot");
  const [screenshotPreview, setScreenshotPreview] = useState<string | null>(null);
  const [selectedApp, setSelectedApp] = useState("");
  const [command, setCommand] = useState("");
  const [output, setOutput] = useState("");

  const tabs = ["screenshot", "app-switch", "shell", "clipboard"];

  const handleScreenshot = async () => {
    try {
      const result = await window.__neotrix__.captureScreen();
      setScreenshotPreview(result.image_base64);
    } catch (e) {
      setOutput(`Error: ${e}`);
    }
  };

  const handleRunCommand = async () => {
    if (!command.trim()) return;
    setOutput("Running...");
    try {
      const result = await window.__neotrix__.executeRemote(selectedApp, command);
      setOutput(result);
    } catch (e) {
      setOutput(`Error: ${e}`);
    }
  };

  return (
    <div className="computer-use-panel">
      <div className="computer-tabs">
        {tabs.map(tab => (
          <button
            key={tab}
            className={`computer-tab ${activeTab === tab ? "active" : ""}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab}
          </button>
        ))}
      </div>
      <div className="computer-content">
        {activeTab === "screenshot" && (
          <div className="screenshot-view">
            <button className="action-btn" onClick={handleScreenshot}>Capture Screen</button>
            {screenshotPreview && (
              <img src={screenshotPreview} alt="Screenshot" className="screenshot-preview" />
            )}
          </div>
        )}
        {activeTab === "app-switch" && (
          <div className="app-switch-view">
            <select value={selectedApp} onChange={e => setSelectedApp(e.target.value)}>
              <option value="">Select app...</option>
              <option value="Terminal">Terminal</option>
              <option value="Finder">Finder</option>
              <option value="Safari">Safari</option>
              <option value="VS Code">VS Code</option>
            </select>
            <button className="action-btn" onClick={() => selectedApp && onAction("switch_app", { app: selectedApp })}>
              Switch to {selectedApp}
            </button>
          </div>
        )}
        {activeTab === "shell" && (
          <div className="shell-view">
            <input
              className="shell-input"
              value={command}
              onChange={e => setCommand(e.target.value)}
              placeholder="Enter command..."
              onKeyDown={e => { if (e.key === "Enter") handleRunCommand(); }}
            />
            <button className="action-btn" onClick={handleRunCommand}>Run</button>
            <pre className="shell-output">{output}</pre>
          </div>
        )}
        {activeTab === "clipboard" && (
          <div className="clipboard-view">
            <button className="action-btn" onClick={() => onAction("get_clipboard", {})}>Read Clipboard</button>
            <button className="action-btn" onClick={() => onAction("write_clipboard", { text: "copied from ComputerUsePanel" })}>Write to Clipboard</button>
          </div>
        )}
      </div>
    </div>
  );
}
