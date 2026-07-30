import React, { useState, useEffect } from "react";
import "./BrowserPane.css";

interface BrowserPaneProps {
  onNavigate: (url: string) => void;
}

export function BrowserPane({ onNavigate }: BrowserPaneProps): JSX.Element {
  const [url, setUrl] = useState("https://neotrix.ai");
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const navigate = (targetUrl: string) => {
    setLoading(true);
    setUrl(targetUrl);
    setHistory(prev => {
      const trimmed = prev.slice(0, historyIndex + 1);
      return [...trimmed, targetUrl];
    });
    setHistoryIndex(prev => prev + 1);
    onNavigate(targetUrl);
    setTimeout(() => setLoading(false), 500);
  };

  const goBack = () => {
    if (historyIndex > 0) {
      const newIndex = historyIndex - 1;
      setHistoryIndex(newIndex);
      setUrl(history[newIndex]);
      onNavigate(history[newIndex]);
    }
  };

  const goForward = () => {
    if (historyIndex < history.length - 1) {
      const newIndex = historyIndex + 1;
      setHistoryIndex(newIndex);
      setUrl(history[newIndex]);
      onNavigate(history[newIndex]);
    }
  };

  return (
    <div className="browser-pane">
      <div className="browser-toolbar">
        <button className="nav-btn" onClick={goBack} disabled={historyIndex <= 0}>◀</button>
        <button className="nav-btn" onClick={goForward} disabled={historyIndex >= history.length - 1}>▶</button>
        <input
          className="url-bar"
          value={url}
          onChange={e => setUrl(e.target.value)}
          onKeyDown={e => { if (e.key === "Enter") navigate(url); }}
          placeholder="Enter URL..."
        />
        <button className="nav-btn" onClick={() => navigate(url)}>Go</button>
      </div>
      <div className="browser-content">
        {loading && <div className="browser-loader">Loading...</div>}
        <iframe
          src={url}
          className="browser-frame"
          title="Web Browser"
          sandbox="allow-scripts allow-same-origin allow-forms"
          onLoad={() => setLoading(false)}
        />
      </div>
    </div>
  );
}
