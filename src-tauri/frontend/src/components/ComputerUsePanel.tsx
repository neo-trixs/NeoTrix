import React, { useState } from "react";

interface ScreenCapture {
  imageBase64: string;
  width: number;
  height: number;
}

interface WindowInfo {
  title: string;
  pid: number;
  appName: string;
}

const ComputerUsePanel: React.FC = () => {
  const [screenshot, setScreenshot] = useState<ScreenCapture | null>(null);
  const [windows, setWindows] = useState<WindowInfo[]>([]);
  const [frontmost, setFrontmost] = useState<string>("");
  const [capturing, setCapturing] = useState(false);
  const [analysisInput, setAnalysisInput] = useState("");

  const captureScreen = async () => {
    setCapturing(true);
    try {
      const result = await (window as any).__TAURI__.invoke("cmd_capture_screen") as ScreenCapture;
      setScreenshot(result);
      setWindows([]);
      setFrontmost("");
    } catch (e) {
      console.error("Capture failed:", e);
    } finally {
      setCapturing(false);
    }
  };

  const listWindows = async () => {
    try {
      const result = await (window as any).__TAURI__.invoke("cmd_get_window_list") as WindowInfo[];
      setWindows(result);
    } catch (e) {
      console.error("List windows failed:", e);
    }
  };

  const getFrontmost = async () => {
    try {
      const result = await (window as any).__TAURI__.invoke("cmd_get_frontmost_app") as { app_name: string; title: string };
      setFrontmost(`${result.app_name}: ${result.title}`);
    } catch (e) {
      console.error("Get frontmost failed:", e);
    }
  };

  return (
    <div style={{ padding: 8, background: "var(--bg-primary, #ffffff)", maxHeight: "100%", overflowY: "auto" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
        <h3 style={{ fontSize: 13, fontWeight: 600, margin: 0, color: "var(--text-primary, #1a1a2e)" }}>🖥 Desktop View</h3>
      </div>
      <div style={{ display: "flex", gap: 4, marginBottom: 8 }}>
        <button onClick={captureScreen} disabled={capturing} style={{ flex: 1, padding: "4px 8px", cursor: capturing ? "default" : "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: capturing ? "var(--border-color, #e1e4e8)" : "var(--accent, #007aff)", color: "#fff", fontSize: 11, fontWeight: 500 }}>
          {capturing ? "Capturing..." : "📷 Capture"}
        </button>
        <button onClick={listWindows} style={{ flex: 1, padding: "4px 8px", cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", fontSize: 11 }}>Windows</button>
        <button onClick={getFrontmost} style={{ flex: 1, padding: "4px 8px", cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", fontSize: 11 }}>Foreground</button>
      </div>
      {frontmost && (
        <div style={{ fontSize: 10, color: "var(--text-muted, #8b949e)", marginBottom: 6, padding: "2px 6px", background: "var(--bg-secondary, #f6f8fa)", borderRadius: 3 }}>
          Active: {frontmost}
        </div>
      )}
      {windows.length > 0 && (
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 10, color: "var(--text-muted, #8b949e)", marginBottom: 4 }}>Open Windows</div>
          <div style={{ maxHeight: 100, overflowY: "auto", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4 }}>
            {windows.map((w, i) => (
              <div key={i} style={{ padding: "3px 6px", fontSize: 10, borderBottom: "1px solid var(--border-color, #e1e4e8)", display: "flex", justifyContent: "space-between" }}>
                <span>{w.appName}</span>
                <span style={{ color: "var(--text-muted, #8b949e)", maxWidth: "60%", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{w.title}</span>
              </div>
            ))}
          </div>
        </div>
      )}
      {screenshot && (
        <div style={{ marginBottom: 8 }}>
          <div style={{ fontSize: 10, color: "var(--text-muted, #8b949e)", marginBottom: 4 }}>Screen Capture ({screenshot.width}×{screenshot.height})</div>
          <img src={`data:image/png;base64,${screenshot.imageBase64}`} alt="Screen capture" style={{ width: "100%", borderRadius: 4, border: "1px solid var(--border-color, #e1e4e8)" }} />
        </div>
      )}
      <div style={{ marginTop: 4 }}>
        <textarea value={analysisInput} onChange={(e) => setAnalysisInput(e.target.value)} placeholder="Describe what you see on screen..." style={{ width: "100%", height: 40, padding: 6, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, fontSize: 11, resize: "vertical", background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", outline: "none", fontFamily: "inherit" }} />
      </div>
    </div>
  );
};

export default ComputerUsePanel;
