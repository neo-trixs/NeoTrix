import React, { useEffect, useCallback } from "react";
import type { ProxyStatus } from "../types";
import { proxyStatus as fetchProxyStatus, proxySetMode, proxyStartDaemon, proxyStopDaemon } from "../lib/api";

interface Props {
  status: ProxyStatus;
  onStatusChange: (status: ProxyStatus) => void;
  onClose: () => void;
}

const ProxyPanel: React.FC<Props> = ({ status, onStatusChange, onClose }) => {
  const refresh = useCallback(async () => {
    const s = await fetchProxyStatus();
    onStatusChange(s);
  }, [onStatusChange]);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, 5000);
    return () => clearInterval(timer);
  }, [refresh]);

  const handleToggle = async () => {
    if (status.running && status.mode !== "off") {
      await proxySetMode("off");
    } else {
      if (!status.running) {
        await proxyStartDaemon();
      }
      await proxySetMode("geo");
    }
    refresh();
  };

  const daemonRunning = status.running;

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel proxy-panel glass-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>代理</h2>
          <button className="btn-icon" onClick={onClose}>✕</button>
        </div>

        <div className="proxy-status-bar">
          <span className={`proxy-status-dot ${daemonRunning ? "on" : "off"}`} />
          <span className="proxy-status-text">
            {daemonRunning ? "守护进程运行中" : "守护进程未运行"}
          </span>
        </div>

        <div className="proxy-toggle-section">
          <div className="proxy-toggle-row">
            <div>
              <div className="proxy-toggle-label">系统代理</div>
              <div className="proxy-toggle-desc">
                {daemonRunning && status.mode !== "off"
                  ? "已开启 — 系统流量经 NeoTrix 代理路由"
                  : "已关闭 — 仅项目内部使用代理"}
              </div>
            </div>
            <button
              className={`proxy-toggle-btn ${daemonRunning && status.mode !== "off" ? "active" : ""}`}
              onClick={handleToggle}
            >
              <div className="proxy-toggle-knob" />
            </button>
          </div>
        </div>

        {daemonRunning && (
          <div className="proxy-info">
            <div className="proxy-info-row"><span>PID</span><span>{status.pid}</span></div>
            <div className="proxy-info-row"><span>端口</span><span>{status.port}</span></div>
            <div className="proxy-info-row"><span>运行时长</span><span>{formatUptime(status.uptime_secs)}</span></div>
            <div className="proxy-info-row"><span>活跃连接</span><span>{status.active_count}</span></div>
          </div>
        )}

        <div className="settings-footer">
          <button className="btn-secondary" onClick={onClose}>关闭</button>
          {daemonRunning && (
            <button className="btn-danger" onClick={async () => { await proxyStopDaemon(); refresh(); }}>
              停止守护进程
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

function formatUptime(secs: number): string {
  if (secs < 60) return `${secs}秒`;
  if (secs < 3600) return `${Math.floor(secs / 60)}分`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return `${h}时${m}分`;
}

export default ProxyPanel;
