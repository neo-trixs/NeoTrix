import React, { useCallback, useEffect, useState, useRef } from "react";
import * as api from "../lib/api";
import type { ProxyStatus, ProxyConnectivity, ProxyNodeInfo, ProxyConfigData, ProxySourceInfo } from "../types";
import WorldMap from "../components/WorldMap";
import ProxyChainViz from "../components/ProxyChainViz";
import styles from "./ProxyPage.module.css";

type TabId = "overview" | "map" | "nodes" | "subscriptions" | "settings";

const MODE_LABELS: Record<string, string> = {
  off: "关闭", geo: "地理位置", stealth: "隐身", tor: "Tor",
};

const MODE_ICONS: Record<string, string> = {
  off: "⛔", geo: "🌍", stealth: "🕶", tor: "🧅",
};

function fmtUptime(secs: number): string {
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return `${h}h ${m}m`;
}

function latencyColor(ms: number | null): string {
  if (ms == null) return "var(--nt-text-muted)";
  if (ms < 200) return "var(--nt-success)";
  if (ms < 500) return "var(--nt-warning)";
  return "var(--nt-danger)";
}

function speedEmoji(tier: string): string {
  switch (tier) {
    case "Fast": return "🚀";
    case "Medium": return "⏱";
    case "Slow": return "🐢";
    default: return "❓";
  }
}

const ProxyPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabId>("overview");
  const [status, setStatus] = useState<ProxyStatus | null>(null);
  const [connectivity, setConnectivity] = useState<ProxyConnectivity | null>(null);
  const [sources, setSources] = useState<ProxySourceInfo[]>([]);
  const [nodes, setNodes] = useState<ProxyNodeInfo[]>([]);
  const [subscriptions, setSubscriptions] = useState<string[]>([]);
  const [config, setConfig] = useState<ProxyConfigData | null>(null);
  const [loading, setLoading] = useState(true);
  const [fetching, setFetching] = useState(false);
  const [newSubUrl, setNewSubUrl] = useState("");
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<ProxyNodeInfo | null>(null);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadAll = useCallback(async () => {
    try {
      const [s, conn, src, nds, subs, cfg] = await Promise.all([
        api.proxyStatus(),
        api.proxyConnectivity(),
        api.proxySourceStatus(),
        api.proxyPoolNodes(),
        api.proxySubList(),
        api.proxyConfigGet(),
      ]);
      setStatus(s);
      setConnectivity(conn);
      setSources(src);
      setNodes(nds);
      setSubscriptions(subs);
      setConfig(cfg);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    loadAll().finally(() => setLoading(false));
    timerRef.current = setInterval(loadAll, 10000);
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [loadAll]);

  const showStatus = (msg: string) => {
    setStatusMsg(msg);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  const handleStart = async () => {
    try { const r = await api.proxyStartDaemon(); showStatus(r); await loadAll(); }
    catch (e) { showStatus(String(e)); }
  };

  const handleStop = async () => {
    try { await api.proxyStopDaemon(); showStatus("daemon stopped"); await loadAll(); }
    catch (e) { showStatus(String(e)); }
  };

  const handleSetMode = async (mode: string) => {
    try { await api.proxySetMode(mode); showStatus(`mode: ${mode}`); await loadAll(); }
    catch (e) { showStatus(String(e)); }
  };

  const handleFetch = async () => {
    setFetching(true);
    try { const count = await api.proxyTriggerFetch(500); showStatus(`fetched ${count} proxies`); await loadAll(); }
    catch (e) { showStatus(String(e)); }
    setFetching(false);
  };

  const handleAddSub = async () => {
    const url = newSubUrl.trim();
    if (!url) return;
    try { await api.proxySubAdd(url); setNewSubUrl(""); showStatus("subscription added"); await loadAll(); }
    catch (e) { showStatus(String(e)); }
  };

  const handleRemoveSub = async (url: string) => {
    try { await api.proxySubRemove(url); showStatus("subscription removed"); await loadAll(); }
    catch (e) { showStatus(String(e)); }
  };

  const handleConfigSave = async () => {
    if (!config) return;
    try { await api.proxyConfigSet(config); showStatus("config saved"); }
    catch (e) { showStatus(String(e)); }
  };

  const healthyCount = nodes.filter(n => n.healthy).length;
  const fastCount = nodes.filter(n => n.healthy && n.latency_ms != null && n.latency_ms < 200).length;
  const mediumCount = nodes.filter(n => n.healthy && n.latency_ms != null && n.latency_ms >= 200 && n.latency_ms < 500).length;
  const slowCount = nodes.filter(n => n.healthy && n.latency_ms != null && n.latency_ms >= 500).length;
  const deadCount = nodes.length - fastCount - mediumCount - slowCount;

  return (
    <div className={styles.page}>
      {statusMsg && <div className={styles.toast}>{statusMsg}</div>}

      {/* Top: Chain Status + Quick Controls */}
      <div className={styles.topBar}>
        <div className={styles.topLeft}>
          <ProxyChainViz status={status} connectivity={connectivity} />
        </div>
        <div className={styles.topRight}>
          {/* Daemon Controls */}
          <div className={styles.controlCard}>
            <div className={styles.controlHeader}>
              <span className={`${styles.statusDot} ${status?.running ? styles.statusOn : styles.statusOff}`} />
              <span>{status?.running ? "运行中" : "已停止"}</span>
            </div>
            {!status?.running ? (
              <button className="btn-primary" onClick={handleStart}>🚀 启动</button>
            ) : (
              <button className="btn-danger" onClick={handleStop}>⛔ 停止</button>
            )}
          </div>
          {/* Mode Grid */}
          <div className={styles.modeGrid}>
            {["off", "geo", "stealth", "tor"].map(m => (
              <button key={m}
                className={`${styles.modeBtn} ${status?.mode === m ? styles.modeBtnActive : ""}`}
                onClick={() => handleSetMode(m)}
                disabled={!status?.running}
              >
                <span className={styles.modeIcon}>{MODE_ICONS[m]}</span>
                <span className={styles.modeLabel}>{MODE_LABELS[m]}</span>
              </button>
            ))}
          </div>
          {/* Quick Stats */}
          <div className={styles.quickStats}>
            <div className={styles.quickStat}><span className={styles.qsValue}>{nodes.length}</span><span className={styles.qsLabel}>总节点</span></div>
            <div className={styles.quickStat}><span className={styles.qsValue}>{healthyCount}</span><span className={styles.qsLabel}>健康</span></div>
            <div className={styles.quickStat}><span className={styles.qsValue}>{subscriptions.length}</span><span className={styles.qsLabel}>订阅</span></div>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className={styles.tabs}>
        {(["overview", "map", "nodes", "subscriptions", "settings"] as TabId[]).map(tab => (
          <button key={tab}
            className={`${styles.tab} ${activeTab === tab ? styles.tabActive : ""}`}
            onClick={() => setActiveTab(tab)}
          >
            {tab === "overview" && "📊 概览"}
            {tab === "map" && "🗺 地图"}
            {tab === "nodes" && `🔌 节点 ${nodes.length > 0 ? `(${healthyCount}/${nodes.length})` : ""}`}
            {tab === "subscriptions" && `📋 订阅 (${subscriptions.length})`}
            {tab === "settings" && "⚙️ 设置"}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className={styles.content}>
        {loading ? (
          <div className={styles.loading}>加载中...</div>
        ) : (
          <>
            {activeTab === "overview" && (
              <div className={styles.tabContent}>
                {/* Daemon Info */}
                <div className={styles.card}>
                  <h3 className={styles.cardTitle}>守护进程</h3>
                  <div className={styles.infoGrid}>
                    <div className={styles.infoRow}><span>PID</span><span>{status?.pid || "—"}</span></div>
                    <div className={styles.infoRow}><span>端口</span><span>{status?.port || 11080}</span></div>
                    <div className={styles.infoRow}><span>运行时间</span><span>{status?.running ? fmtUptime(status.uptime_secs) : "—"}</span></div>
                    <div className={styles.infoRow}><span>活跃连接</span><span>{status?.active_count || 0}</span></div>
                    <div className={styles.infoRow}><span>模式</span><span className={styles.modeLabel}>{status?.running ? MODE_LABELS[status.mode] || status.mode : "—"}</span></div>
                  </div>
                </div>
                {/* Connectivity Details */}
                <div className={styles.card}>
                  <h3 className={styles.cardTitle}>连通性</h3>
                  <div className={styles.connGrid}>
                    <div className={`${styles.connItem} ${connectivity?.direct_reachable ? styles.connOk : styles.connDead}`}>
                      <span className={styles.connIcon}>🌐</span>
                      <span>直连</span>
                      {connectivity?.direct_latency_ms != null && <span className={styles.connMs}>{connectivity.direct_latency_ms.toFixed(0)}ms</span>}
                    </div>
                    <div className={`${styles.connItem} ${healthyCount > 0 ? styles.connOk : styles.connDead}`}>
                      <span className={styles.connIcon}>🔀</span>
                      <span>代理</span>
                      <span className={styles.connMs}>{healthyCount}/{nodes.length}</span>
                    </div>
                    <div className={`${styles.connItem} ${connectivity?.proxy_avg_latency_ms != null ? styles.connOk : styles.connDead}`}>
                      <span className={styles.connIcon}>⏱</span>
                      <span>平均延迟</span>
                      {connectivity?.proxy_avg_latency_ms != null
                        ? <span className={styles.connMs}>{connectivity.proxy_avg_latency_ms.toFixed(0)}ms</span>
                        : <span className={styles.connMs}>—</span>}
                    </div>
                    <div className={`${styles.connItem} ${status?.running ? styles.connOk : styles.connDead}`}>
                      <span className={styles.connIcon}>📡</span>
                      <span>守护进程</span>
                      <span className={styles.connMs}>{status?.running ? "运行中" : "已停止"}</span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === "map" && (
              <div className={styles.tabContent}>
                <WorldMap nodes={nodes} onNodeClick={setSelectedNode} />
                {selectedNode && (
                  <div className={styles.nodeDetail}>
                    <span className={styles.ndClose} onClick={() => setSelectedNode(null)}>✕</span>
                    <div className={styles.ndRow}><span>地址</span><span className={styles.ndUrl}>{selectedNode.url}</span></div>
                    <div className={styles.ndRow}><span>标签</span><span>{selectedNode.tag || "—"}</span></div>
                    <div className={styles.ndRow}><span>延迟</span><span style={{ color: latencyColor(selectedNode.latency_ms) }}>{selectedNode.latency_ms != null ? `${selectedNode.latency_ms.toFixed(0)}ms` : "—"}</span></div>
                    <div className={styles.ndRow}><span>地区</span><span>{selectedNode.geo_tag || "—"}</span></div>
                    <div className={styles.ndRow}><span>IP</span><span className={styles.ndMono}>{selectedNode.ip_addr || "—"}</span></div>
                    <div className={styles.ndRow}><span>速度</span><span>{speedEmoji(selectedNode.speed_tier)} {selectedNode.speed_tier}</span></div>
                    <div className={styles.ndRow}><span>评分</span><span>{selectedNode.score.toFixed(3)}</span></div>
                    <div className={styles.ndRow}><span>成功/失败</span><span>{selectedNode.success_count}/{selectedNode.fail_count}</span></div>
                  </div>
                )}
              </div>
            )}

            {activeTab === "nodes" && (
              <div className={styles.tabContent}>
                <div className={styles.sectionHeader}>
                  <h3 className={styles.cardTitle}>代理节点 ({nodes.length})</h3>
                  <button className="btn-primary" onClick={handleFetch} disabled={fetching}>
                    {fetching ? "拉取中..." : "🔄 拉取"}
                  </button>
                </div>
                {/* Health Distribution Bar */}
                <div className={styles.healthBarContainer}>
                  <div className={styles.healthBar}>
                    {fastCount > 0 && <div className={styles.hbFast} style={{ width: `${(fastCount / (nodes.length || 1)) * 100}%` }} title={`快速: ${fastCount}`} />}
                    {mediumCount > 0 && <div className={styles.hbMedium} style={{ width: `${(mediumCount / (nodes.length || 1)) * 100}%` }} title={`中等: ${mediumCount}`} />}
                    {slowCount > 0 && <div className={styles.hbSlow} style={{ width: `${(slowCount / (nodes.length || 1)) * 100}%` }} title={`慢速: ${slowCount}`} />}
                    {deadCount > 0 && <div className={styles.hbDead} style={{ width: `${(deadCount / (nodes.length || 1)) * 100}%` }} title={`离线: ${deadCount}`} />}
                  </div>
                  <div className={styles.healthLegend}>
                    <span>🚀 {fastCount}</span>
                    <span>⏱ {mediumCount}</span>
                    <span>🐢 {slowCount}</span>
                    <span>❌ {deadCount}</span>
                  </div>
                </div>
                {/* Node Table */}
                <div className={styles.nodeTable}>
                  <div className={styles.ntHead}>
                    <span className={styles.colSpeed}></span>
                    <span className={styles.colTag}>标签</span>
                    <span className={styles.colLatency}>延迟</span>
                    <span className={styles.colGeo}>地区</span>
                    <span className={styles.colScore}>评分</span>
                    <span className={styles.colSf}>成功/失败</span>
                    <span className={styles.colStatus}>状态</span>
                  </div>
                  {nodes.length === 0 && <div className={styles.empty}>暂无节点。请添加订阅并拉取。</div>}
                  {nodes.map((n, i) => (
                    <div key={i} className={`${styles.ntRow} ${n.healthy ? "" : styles.ntRowDead}`}
                      onClick={() => setSelectedNode(selectedNode?.url === n.url ? null : n)}
                    >
                      <span className={styles.colSpeed}>{speedEmoji(n.speed_tier)}</span>
                      <span className={styles.colTag} title={n.url}>{n.tag || n.url.split("://")[1]?.split("@")[1]?.split(":")[0] || n.url.slice(0, 40)}</span>
                      <span className={styles.colLatency} style={{ color: latencyColor(n.latency_ms) }}>{n.latency_ms != null ? `${n.latency_ms.toFixed(0)}ms` : "—"}</span>
                      <span className={styles.colGeo}>{n.geo_tag || "—"}</span>
                      <span className={styles.colScore}>{n.score.toFixed(2)}</span>
                      <span className={styles.colSf}>{n.success_count}/{n.fail_count}</span>
                      <span className={styles.colStatus}>{n.healthy ? "🟢" : "🔴"}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {activeTab === "subscriptions" && (
              <div className={styles.tabContent}>
                <div className={styles.sectionHeader}>
                  <h3 className={styles.cardTitle}>订阅地址 ({subscriptions.length})</h3>
                  <button className="btn-secondary" onClick={handleFetch} disabled={fetching}>
                    {fetching ? "拉取中..." : "🔄 全部拉取"}
                  </button>
                </div>
                <div className={styles.addSubRow}>
                  <input className={styles.subInput}
                    placeholder="https://example.com/subscribe?token=..."
                    value={newSubUrl} onChange={e => setNewSubUrl(e.target.value)}
                    onKeyDown={e => e.key === "Enter" && handleAddSub()}
                  />
                  <button className="btn-primary" onClick={handleAddSub} disabled={!newSubUrl.trim()}>添加</button>
                </div>
                <div className={styles.subList}>
                  {subscriptions.length === 0 && <div className={styles.empty}>暂无订阅。输入上方 URL 添加。</div>}
                  {subscriptions.map((url, i) => (
                    <div key={i} className={styles.subRow}>
                      <span className={styles.subIdx}>{i + 1}</span>
                      <span className={styles.subUrl} title={url}>{url}</span>
                      <button className={styles.subRemove} onClick={() => handleRemoveSub(url)}>✕</button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {activeTab === "settings" && (
              <div className={styles.tabContent}>
                <h3 className={styles.cardTitle}>代理配置</h3>
                {config && (
                  <div className={styles.configForm}>
                    <div className={styles.cfgRow}><label>本地端口</label><input type="number" value={config.local_port} onChange={e => setConfig({ ...config, local_port: parseInt(e.target.value) || 11080 })} /></div>
                    <div className={styles.cfgRow}><label>SOCKS 端口</label><input type="number" value={config.socks_port} onChange={e => setConfig({ ...config, socks_port: parseInt(e.target.value) || 9050 })} /></div>
                    <div className={styles.cfgRow}><label>最小节点数</label><input type="number" value={config.min_nodes} onChange={e => setConfig({ ...config, min_nodes: parseInt(e.target.value) || 5 })} /></div>
                    <div className={styles.cfgRow}><label>健康检查间隔 (秒)</label><input type="number" value={config.health_check_interval_secs} onChange={e => setConfig({ ...config, health_check_interval_secs: parseInt(e.target.value) || 60 })} /></div>
                    <div className={styles.cfgRow}>
                      <label>选择策略</label>
                      <select value={config.selection_strategy} onChange={e => setConfig({ ...config, selection_strategy: e.target.value })}>
                        <option value="auto">自动</option>
                        <option value="fastest">最快</option>
                        <option value="least_latency">最低延迟</option>
                        <option value="weighted_random">加权随机</option>
                        <option value="round_robin">轮询</option>
                      </select>
                    </div>
                    <div className={styles.cfgRow}><label>直连超时 (秒)</label><input type="number" value={config.direct_timeout_secs} onChange={e => setConfig({ ...config, direct_timeout_secs: parseInt(e.target.value) || 3 })} /></div>
                    <div className={styles.cfgRow}>
                      <label>系统代理</label>
                      <label className={styles.toggleLabel}><input type="checkbox" checked={config.system_proxy_enabled} onChange={e => setConfig({ ...config, system_proxy_enabled: e.target.checked })} />{config.system_proxy_enabled ? "已启用" : "已禁用"}</label>
                    </div>
                    <button className="btn-primary" onClick={handleConfigSave}>保存配置</button>
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default ProxyPage;
