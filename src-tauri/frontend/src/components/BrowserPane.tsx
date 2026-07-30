import React, { useState, useRef } from "react";

interface BrowserPaneProps {
  onClose: () => void;
}

const BrowserPane: React.FC<BrowserPaneProps> = ({ onClose }) => {
  const [url, setUrl] = useState("https://neotrix.ai");
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [loading, setLoading] = useState(false);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  const navigate = (newUrl: string) => {
    if (!newUrl) return;
    const trimmed = newUrl.startsWith("http") ? newUrl : "https://" + newUrl;
    setHistory((prev) => {
      const sliced = prev.slice(0, historyIndex + 1);
      return [...sliced, trimmed];
    });
    setHistoryIndex((prev) => prev + 1);
    setLoading(true);
  };

  const goBack = () => {
    if (historyIndex > 0) {
      setHistoryIndex((prev) => prev - 1);
      setLoading(true);
    }
  };

  const goForward = () => {
    if (historyIndex < history.length - 1) {
      setHistoryIndex((prev) => prev + 1);
      setLoading(true);
    }
  };

  const currentUrl = history[historyIndex] || "";

  const handleIframeLoad = () => {
    setLoading(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      navigate(url);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: "var(--bg-primary, #ffffff)" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 4, padding: "4px 8px", borderBottom: "1px solid var(--border-color, #e1e4e8)", background: "var(--bg-secondary, #f6f8fa)" }}>
        <button onClick={goBack} disabled={historyIndex <= 0} style={{ padding: "2px 6px", cursor: historyIndex > 0 ? "pointer" : "default", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", fontSize: 12 }}>←</button>
        <button onClick={goForward} disabled={historyIndex >= history.length - 1} style={{ padding: "2px 6px", cursor: historyIndex < history.length - 1 ? "pointer" : "default", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", fontSize: 12 }}>→</button>
        <button onClick={() => window.location.reload()} style={{ padding: "2px 6px", cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", fontSize: 12 }}>⟳</button>
        <input type="text" value={url} onChange={(e) => setUrl(e.target.value)} onKeyDown={handleKeyDown} placeholder="Enter URL..." style={{ flex: 1, padding: "3px 8px", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, fontSize: 12, background: "var(--bg-primary, #ffffff)", color: "var(--text-primary, #1a1a2e)", outline: "none" }} />
        <button onClick={() => navigate(url)} style={{ padding: "2px 8px", cursor: "pointer", border: "1px solid var(--accent, #007aff)", borderRadius: 4, background: "var(--accent, #007aff)", color: "#fff", fontSize: 11 }}>Go</button>
        <button onClick={onClose} style={{ padding: "2px 6px", cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", fontSize: 12 }}>✕</button>
      </div>
      <div style={{ flex: 1, position: "relative" }}>
        {loading && (
          <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "var(--bg-primary, #ffffff)", zIndex: 10 }}>
            <div style={{ width: 24, height: 24, border: "2px solid var(--border-color, #e1e4e8)", borderTopColor: "var(--accent, #007aff)", borderRadius: "50%", animation: "spin 0.8s linear infinite" }} />
          </div>
        )}
        {currentUrl ? (
          <iframe src={currentUrl} onLoad={handleIframeLoad} style={{ width: "100%", height: "100%", border: "none", display: loading ? "none" : "block" }} sandbox="allow-same-origin allow-scripts allow-popups" />
        ) : (
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--text-muted, #8b949e)", fontSize: 13 }}>Enter a URL to browse</div>
        )}
      </div>
      <div style={{ padding: "2px 8px", borderTop: "1px solid var(--border-color, #e1e4e8)", fontSize: 10, color: "var(--text-muted, #8b949e)", background: "var(--bg-secondary, #f6f8fa)" }}>
        {currentUrl ? `Preview: ${currentUrl}` : "No page loaded"}
      </div>
    </div>
  );
};

export default BrowserPane;
