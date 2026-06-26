import React, { useState } from "react";
import * as api from "../lib/api";
import type { RemoteSessionInfo } from "../types";
import QRDisplay from "./QRDisplay";

const RemoteControlPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [session, setSession] = useState<RemoteSessionInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleStart = async () => {
    setLoading(true);
    setError("");
    try {
      const result = await api.remoteStart();
      setSession(result);
    } catch (e) {
      setError(String(e));
    }
    setLoading(false);
  };

  const handleStop = async () => {
    if (!session) return;
    try {
      await api.remoteStop(session.id);
      setSession(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {}
  };

  return (
    <div className="settings-overlay" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="settings-panel glass-panel">
        <div className="settings-header">
          <h2>远程控制</h2>
          <button className="btn-icon" onClick={onClose}>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M2 2l8 8M10 2l-8 8"/></svg>
          </button>
        </div>

        <div className="settings-body">
          {!session ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 16, padding: "8px 0" }}>
              <p style={{ fontSize: 13, lineHeight: 1.6, color: "var(--mac-text-secondary)" }}>
                启动远程控制后，可以从手机、浏览器或其它设备连接到当前 NeoTrix 会话。
                远程设备通过 A2A 协议中继进行通信。
              </p>
              <div style={{ display: "flex", gap: 8 }}>
                <button className="btn-primary" onClick={handleStart} disabled={loading}>
                  {loading ? "启动中..." : "启动远程控制"}
                </button>
              </div>
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 16, padding: "8px 0" }}>
              <div className="remote-active-bar">
                <span className="proxy-status-dot on" />
                <span style={{ fontSize: 13, fontWeight: 500 }}>远程控制已激活</span>
              </div>

              {session.qr_svg && <QRDisplay svgContent={session.qr_svg} />}

              <div className="settings-group">
                <label>中继链接</label>
                <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                  <code style={{ flex: 1, padding: "6px 8px", background: "var(--mac-hover)", borderRadius: 4, fontSize: 12, wordBreak: "break-all" }}>
                    {session.relay_url}
                  </code>
                  <button className="btn-icon" onClick={() => copyToClipboard(session.relay_url ?? "")} title="复制链接">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3"><rect x="3" y="3" width="8" height="8" rx="1"/><path d="M1 9V2a1 1 0 011-1h6"/></svg>
                  </button>
                </div>
              </div>

              <div className="settings-group">
                <label>NeoTrix 协议链接 (QR)</label>
                <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                  <code style={{ flex: 1, padding: "6px 8px", background: "var(--mac-hover)", borderRadius: 4, fontSize: 12, wordBreak: "break-all" }}>
                    {session.qr_url}
                  </code>
                  <button className="btn-icon" onClick={() => copyToClipboard(session.qr_url ?? "")} title="复制链接">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3"><rect x="3" y="3" width="8" height="8" rx="1"/><path d="M1 9V2a1 1 0 011-1h6"/></svg>
                  </button>
                </div>
              </div>

              <div className="proxy-info">
                <div className="proxy-info-row">
                  <span>会话 ID</span>
                  <span style={{ fontFamily: "'SF Mono', Menlo, monospace", fontSize: 11 }}>{session.id.slice(0, 20)}...</span>
                </div>
                <div className="proxy-info-row">
                  <span>状态</span>
                  <span>{session.state}</span>
                </div>
              </div>

              <button className="btn-danger" onClick={handleStop}>停止远程控制</button>
            </div>
          )}

          {error && (
            <div style={{ padding: "8px 12px", background: "rgba(255,59,48,0.08)", borderRadius: 6, fontSize: 12, color: "var(--danger)" }}>
              {error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default RemoteControlPanel;
