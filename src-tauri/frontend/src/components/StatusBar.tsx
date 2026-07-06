import React from "react";
import { useStore } from "../stores";
import type { ProxyStatus } from "../types";
import styles from "./StatusBar.module.css";
import { hexagramToName } from "./consciousness/E8Indicator";

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

  const e8State = useStore((s) => s.e8State);
  const gwtResonance = useStore((s) => s.gwtResonance);
  const sealStatus = useStore((s) => s.sealStatus);

  return (
    <div className={`${styles.bar} ${agentBusy ? styles.busy : ""}`} role="status" aria-label="Application status" data-testid="status-bar" data-busy={agentBusy}>
      <div className={styles.left}>
        <span className={`${styles.dot} ${agentBusy ? styles.dotBusy : styles.dotIdle}`} />
        <span className="status-text">{text}</span>
        {agentBusy && <span className="status-spinner">⏳</span>}

        {agentBusy && (
          <span className="status-cons-item">
            E8: {hexagramToName(e8State.hexagram)}
          </span>
        )}
      </div>
      <div className={styles.right}>
        <span className="status-cons-item">
          {gwtResonance.activeCount}/{gwtResonance.totalCount} experts
        </span>
        <span className="status-item">会话 {sessionIndex}/{sessionCount}</span>
        {terminalStatus && <span className={`status-item ${styles.terminalStatus}`}>{terminalStatus}</span>}
        <button className={`${styles.btn} ${styles.proxyIndicator} ${sysproxyOn ? styles.proxyOn : styles.proxyOff}`} onClick={onOpenProxy} aria-label="Toggle system proxy" title={sysproxyOn ? "系统代理已开启" : "系统代理已关闭"}>
          {sysproxyOn ? "🛡" : "🔓"}
        </button>
        <button className={styles.btn} onClick={onToggleTheme} aria-label="Toggle theme" title={theme === "dark" ? "切换浅色模式" : "切换深色模式"}>
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
        <button className={`${styles.btn} ${showTerminal ? "active" : ""}`} onClick={onToggleTerminal} aria-label="Toggle terminal" title="终端">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 3l4 4-4 4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M8 11h4" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>
        <button className={styles.btn} onClick={onSelectProject} aria-label="Select project" title="选择项目">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 4h4l1.5-1.5H12a1 1 0 011 1v7a1 1 0 01-1 1H2a1 1 0 01-1-1V5a1 1 0 011-1z" stroke="currentColor" strokeWidth="1.3" />
          </svg>
        </button>
        <button className={styles.btn} onClick={onOpenSettings} aria-label="Open settings" title="设置">
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
