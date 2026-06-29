import React, { useCallback, useEffect, useState, useRef } from "react";
import * as api from "../lib/api";
import type { ProxyStatus, ProxySourceInfo, ProxyConnectivity } from "../types";

interface Props {
  onClose: () => void;
}

function linkIcon(direct: boolean, icon: string): string {
  return direct ? icon : "⛔";
}

const MODE_LABEL: Record<string, string> = {
  auto: "🤖 智能",
  rule: "📋 规则",
  proxy: "🔀 代理",
  direct: "🌐 直连",
};

const ProxyDialog: React.FC<Props> = ({ onClose }) => {
  const [status, setStatus] = useState<ProxyStatus | null>(null);
  const [sources, setSources] = useState<ProxySourceInfo[]>([]);
  const [connectivity, setConnectivity] = useState<ProxyConnectivity | null>(null);
  const [fetchCount, setFetchCount] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [fetching, setFetching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const [s, src, conn] = await Promise.all([
        api.proxyStatus(),
        api.proxySourceStatus(),
        api.proxyConnectivity(),
      ]);
      setStatus(s);
      setSources(src);
      setConnectivity(conn);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    load().finally(() => setLoading(false));
    timerRef.current = setInterval(load, 15_000);
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [load]);

  const handleStartDaemon = async () => {
    try {
      await api.proxyStartDaemon();
      setStatus((prev) => prev ? { ...prev, running: true } : null);
      setTimeout(load, 1000);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleStopDaemon = async () => {
    try {
      await api.proxyStopDaemon();
      setStatus((prev) => prev ? { ...prev, running: false } : null);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleFetch = async () => {
    setFetching(true);
    setFetchCount(null);
    try {
      const count = await api.proxyTriggerFetch(200);
      setFetchCount(count);
      load();
    } catch (e) {
      setError(String(e));
    }
    setFetching(false);
  };

  const uptimeStr = (secs: number) => {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    return `${h}h ${m}m ${s}s`;
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-panel proxy-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>🛡 代理管理</h2>
          <button className="dialog-close" onClick={onClose}>✕</button>
        </div>

        <div className="dialog-body">
          {error && <div className="proxy-error">{error}</div>}

          {loading ? (
            <div className="proxy-loading">加载中...</div>
          ) : (
            <>
              {/* Connectivity Health */}
              <section className="proxy-section">
                <h3>连通性 <span className="conn-mode-badge">{connectivity?.active_mode ?? "—"}</span></h3>
                <div className="conn-grid">
                  <div className={`conn-card ${connectivity?.direct_reachable ? "conn-ok" : "conn-dead"}`}>
                    <span className="conn-icon">{linkIcon(!!connectivity?.direct_reachable, "🌐")}</span>
                    <span className="conn-label">直连</span>
                    {connectivity?.direct_latency_ms != null && (
                      <span className="conn-latency">{connectivity.direct_latency_ms.toFixed(0)}ms</span>
                    )}
                  </div>
                  <div className={`conn-card ${(connectivity?.proxy_healthy_count ?? 0) >= 3 ? "conn-ok" : (connectivity?.proxy_healthy_count ?? 0) > 0 ? "conn-degraded" : "conn-dead"}`}>
                    <span className="conn-icon">{linkIcon((connectivity?.proxy_healthy_count ?? 0) > 0, "🔀")}</span>
                    <span className="conn-label">代理</span>
                    <span className="conn-count">{connectivity?.proxy_healthy_count ?? 0}/{connectivity?.proxy_total_count ?? 0}</span>
                    {connectivity?.proxy_avg_latency_ms != null && (
                      <span className="conn-latency">{connectivity.proxy_avg_latency_ms.toFixed(0)}ms</span>
                    )}
                  </div>
                </div>
              </section>

              {/* Daemon Status */}
              <section className="proxy-section">
                <h3>代理守护进程</h3>
                <div className="proxy-daemon-status">
                  <div className="status-row">
                    <span className="label">状态</span>
                    <span className={`badge ${status?.running ? "badge-on" : "badge-off"}`}>
                      {status?.running ? "运行中" : "未运行"}
                    </span>
                  </div>
                  {status?.running && (
                    <>
                      <div className="status-row">
                        <span className="label">模式</span>
                        <span className="value">
                          {MODE_LABEL[connectivity?.active_mode ?? ""] ?? connectivity?.active_mode ?? status?.mode ?? "—"}
                        </span>
                      </div>
                      <div className="status-row">
                        <span className="label">端口</span>
                        <span className="value">{status.port}</span>
                      </div>
                      <div className="status-row">
                        <span className="label">运行时间</span>
                        <span className="value">{uptimeStr(status.uptime_secs)}</span>
                      </div>
                      <div className="status-row">
                        <span className="label">活跃连接</span>
                        <span className="value">{status.active_count}</span>
                      </div>
                    </>
                  )}
                  <div className="daemon-actions">
                    {!status?.running ? (
                      <button className="btn-primary" onClick={handleStartDaemon}>启动守护进程</button>
                    ) : (
                      <button className="btn-danger" onClick={handleStopDaemon}>停止守护进程</button>
                    )}
                  </div>
                </div>
              </section>

              {/* Proxy Sources */}
              <section className="proxy-section">
                <h3>代理来源</h3>
                <div className="source-list">
                  <div className="source-header">
                    <span className="col-name">来源</span>
                    <span className="col-success">成功</span>
                    <span className="col-fail">失败</span>
                    <span className="col-status">状态</span>
                  </div>
                  {sources.map((s) => (
                    <div key={s.name} className={`source-row ${s.on_cooldown ? "cooldown" : ""}`}>
                      <span className="col-name">{s.name}</span>
                      <span className="col-success">{s.total_successes}</span>
                      <span className="col-fail">{s.total_failures}</span>
                      <span className={`col-status ${s.on_cooldown ? "status-cooldown" : "status-ok"}`}>
                        {s.on_cooldown ? `冷却 (连续${s.consecutive_failures}次失败)` : "正常"}
                      </span>
                    </div>
                  ))}
                </div>
                {sources.length === 0 && <div className="proxy-empty">暂无来源数据</div>}
              </section>

              {/* Fetch Action */}
              <section className="proxy-section">
                <h3>代理获取</h3>
                <p className="proxy-desc">从 Proxifly (3000+ 代理) 等免费源拉取最新代理节点</p>
                <div className="fetch-actions">
                  <button className="btn-primary" onClick={handleFetch} disabled={fetching}>
                    {fetching ? "拉取中..." : "立即拉取"}
                  </button>
                  {fetchCount !== null && (
                    <span className="fetch-result">已获取 {fetchCount} 个代理</span>
                  )}
                </div>
              </section>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProxyDialog;
