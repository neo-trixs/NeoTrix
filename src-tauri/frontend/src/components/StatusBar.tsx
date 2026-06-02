import React from "react";
import type { ProxyStatus } from "../types";

interface Props {
  text: string;
  agentBusy: boolean;
  sessionIndex: number;
  sessionCount: number;
  showTerminal?: boolean;
  terminalStatus?: string;
  onOpenSettings: () => void;
  onSelectProject: () => void;
  onToggleTerminal?: () => void;
  onToggleTheme?: () => void;
  onOpenProxy?: () => void;
  proxyStatus?: ProxyStatus;
  theme?: string;
}

const StatusBar: React.FC<Props> = ({ text, agentBusy, sessionIndex, sessionCount, showTerminal, terminalStatus, onOpenSettings, onSelectProject, onToggleTerminal, onToggleTheme, onOpenProxy, proxyStatus, theme }) => {
  const sysproxyOn = proxyStatus?.running && proxyStatus.mode !== "off";
  return (
    <div className={`status-bar ${agentBusy ? "busy" : ""}`}>
      <div className="status-left">
        <span className={`status-dot ${agentBusy ? "busy" : "idle"}`} />
        <span className="status-text">{text}</span>
        {agentBusy && <span className="status-spinner">⏳</span>}
      </div>
      <div className="status-right">
        <span className="status-item">会话 {sessionIndex}/{sessionCount}</span>
        {terminalStatus && <span className="status-item terminal-status">{terminalStatus}</span>}
        <button className={`status-btn proxy-indicator ${sysproxyOn ? "proxy-on" : "proxy-off"}`} onClick={onOpenProxy} title={sysproxyOn ? "系统代理已开启" : "系统代理已关闭"}>
          {sysproxyOn ? "🛡" : "🔓"}
        </button>
        <button className="status-btn" onClick={onToggleTheme} title={theme === "dark" ? "切换浅色模式" : "切换深色模式"}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            {theme === "dark" ? (
              <path d="M7 1a6 6 0 100 12 4.5 4.5 0 010-9 4 4 0 010-3z" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            ) : (
              <>
                <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.3" />
                <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </>
            )}
          </svg>
        </button>
        <button className={`status-btn ${showTerminal ? "active" : ""}`} onClick={onToggleTerminal} title="终端">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 3l4 4-4 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M8 11h4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>
        <button className="status-btn" onClick={onSelectProject} title="选择项目">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 4h4l1.5-1.5H12a1 1 0 011 1v7a1 1 0 01-1 1H2a1 1 0 01-1-1V5a1 1 0 011-1z" stroke="currentColor" strokeWidth="1.3" />
          </svg>
        </button>
        <button className="status-btn" onClick={onOpenSettings} title="设置">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.3" />
            <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </div>
  );
};

export default StatusBar;
