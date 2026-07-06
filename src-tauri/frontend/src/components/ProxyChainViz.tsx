import React from "react";
import type { ProxyStatus, ProxyConnectivity } from "../types";
import styles from "./ProxyChainViz.module.css";

interface Props {
  status: ProxyStatus | null;
  connectivity: ProxyConnectivity | null;
}

interface ChainLink {
  id: string;
  label: string;
  icon: string;
  status: "active" | "degraded" | "inactive";
  detail?: string;
  color: string;
}

const ProxyChainViz: React.FC<Props> = ({ status, connectivity }) => {
  const daemonRunning = status?.running ?? false;
  const proxyActive = daemonRunning && status?.mode !== "off";
  const healthyCount = connectivity?.proxy_healthy_count ?? 0;

  const links: ChainLink[] = [
    {
      id: "app",
      label: "NeoTrix",
      icon: "🖥",
      status: "active",
      detail: "Desktop App",
      color: "var(--nt-primary)",
    },
    {
      id: "local-proxy",
      label: "本地代理",
      icon: "🔌",
      status: daemonRunning ? "active" : "inactive",
      detail: daemonRunning ? `:${status?.port ?? 11080}` : "未运行",
      color: daemonRunning ? "var(--nt-primary)" : "var(--nt-text-muted)",
    },
    {
      id: "isp",
      label: "ISP",
      icon: "🌐",
      status: connectivity?.direct_reachable ? "active" : "degraded",
      detail: connectivity?.direct_reachable ? (connectivity.direct_latency_ms != null ? `${connectivity.direct_latency_ms.toFixed(0)}ms` : "可达") : "不可达",
      color: connectivity?.direct_reachable ? "var(--nt-success)" : "var(--nt-danger)",
    },
    {
      id: "proxy-node",
      label: "代理节点",
      icon: "🔀",
      status: healthyCount > 0 ? "active" : "inactive",
      detail: healthyCount > 0 ? `${healthyCount} 可用` : "无节点",
      color: healthyCount > 3 ? "var(--nt-success)" : healthyCount > 0 ? "var(--nt-warning)" : "var(--nt-text-muted)",
    },
    {
      id: "target",
      label: "目标网络",
      icon: "🎯",
      status: proxyActive && healthyCount > 0 ? "active" : "inactive",
      detail: proxyActive ? "通过代理" : "直连",
      color: proxyActive ? "var(--nt-success)" : "var(--nt-text-muted)",
    },
  ];

  const activeCount = links.filter(l => l.status === "active").length;
  const totalLinks = links.length;
  const overallHealth = activeCount === totalLinks ? "optimal" : activeCount >= totalLinks - 1 ? "good" : "degraded";

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <span className={styles.title}>链路状态</span>
        <span className={`${styles.healthBadge} ${styles[overallHealth]}`}>
          {overallHealth === "optimal" ? "🟢 全链路正常" : overallHealth === "good" ? "🟡 部分降级" : "🔴 链路异常"}
        </span>
      </div>
      <div className={styles.chain}>
        {links.map((link, i) => (
          <React.Fragment key={link.id}>
            <div className={`${styles.link} ${styles[link.status]}`}>
              <div className={styles.linkIcon} style={{ background: link.color + "20", color: link.color }}>
                {link.icon}
              </div>
              <div className={styles.linkInfo}>
                <span className={styles.linkLabel}>{link.label}</span>
                <span className={styles.linkDetail}>{link.detail}</span>
              </div>
              <span className={`${styles.linkStatus} ${styles[link.status]}`}>
                {link.status === "active" ? "✓" : link.status === "degraded" ? "⚠" : "✗"}
              </span>
            </div>
            {i < links.length - 1 && (
              <div className={`${styles.connector} ${link.status === "active" ? styles.connectorActive : styles.connectorInactive}`}>
                <div className={styles.connectorLine} />
                <svg width="10" height="10" viewBox="0 0 10 10" className={styles.connectorArrow}>
                  <polyline points="2,2 8,5 2,8" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
            )}
          </React.Fragment>
        ))}
      </div>
      {daemonRunning && (
        <div className={styles.modeInfo}>
          当前模式: <strong>{status?.mode}</strong> | 延迟: {connectivity?.proxy_avg_latency_ms != null ? `${connectivity.proxy_avg_latency_ms.toFixed(0)}ms avg` : "—"}
        </div>
      )}
    </div>
  );
};

export default ProxyChainViz;
