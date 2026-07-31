import React, { useState, useEffect, useCallback } from "react";
import type {
  ScanResult,
  VulnerabilityFinding,
  ScanSummary,
} from "../commands";

const SEVERITY_COLORS: Record<string, string> = {
  Critical: "#ff4444",
  High: "#ff8800",
  Medium: "#ffbb00",
  Low: "#44aaff",
  Info: "#888888",
};

const STATUS_COLORS: Record<string, string> = {
  Open: "#ff4444",
  Verified: "#ff8800",
  Fixed: "#44cc44",
  WontFix: "#888888",
  FalsePositive: "#8888ff",
};

function scoreColor(score: number): string {
  if (score >= 80) return "var(--nt-success)";
  if (score >= 50) return "var(--nt-warning)";
  return "var(--nt-danger)";
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  const now = Date.now();
  const diff = now - d.getTime();
  if (diff < 60000) return "just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString();
}

function severityBadge(sev: string) {
  const c = SEVERITY_COLORS[sev] ?? "#888";
  return (
    <span style={{
      display: "inline-flex", alignItems: "center", gap: 4,
      padding: "1px 8px", borderRadius: 10, fontSize: 10, fontWeight: 700,
      background: `${c}22`, color: c, border: `1px solid ${c}44`,
    }}>
      <span style={{ width: 6, height: 6, borderRadius: "50%", background: c }} />
      {sev}
    </span>
  );
}

function statusBadge(st: string) {
  const c = STATUS_COLORS[st] ?? "#888";
  return (
    <span style={{
      display: "inline-flex", alignItems: "center", gap: 4,
      padding: "1px 8px", borderRadius: 10, fontSize: 10, fontWeight: 600,
      background: `${c}18`, color: c, border: `1px solid ${c}33`,
    }}>
      {st}
    </span>
  );
}

function ScoreGauge({ score }: { score: number }) {
  const r = 36;
  const circ = 2 * Math.PI * r;
  const offset = circ - (score / 100) * circ;
  const color = scoreColor(score);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
      <svg width={90} height={90} viewBox="0 0 90 90">
        <circle cx="45" cy="45" r={r} fill="none" stroke="var(--nt-glass-border)" strokeWidth={6} />
        <circle cx="45" cy="45" r={r} fill="none" stroke={color} strokeWidth={6}
          strokeDasharray={circ} strokeDashoffset={offset}
          strokeLinecap="round" transform="rotate(-90 45 45)"
          style={{ transition: "stroke-dashoffset 0.6s ease, stroke 0.3s" }}
        />
        <text x="45" y="45" textAnchor="middle" dominantBaseline="central"
          fill={color} fontSize={22} fontWeight={700}
        >
          {score}
        </text>
      </svg>
    </div>
  );
}

const SecurityScanPanel: React.FC = () => {
  const [summary, setSummary] = useState<ScanSummary | null>(null);
  const [scans, setScans] = useState<ScanResult[]>([]);
  const [findings, setFindings] = useState<Record<string, VulnerabilityFinding[]>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [depth, setDepth] = useState<string>("standard");
  const [targetPath, setTargetPath] = useState<string>(".");
  const [expandedScan, setExpandedScan] = useState<string | null>(null);
  const [expandedFinding, setExpandedFinding] = useState<string | null>(null);
  const [patchLoading, setPatchLoading] = useState<string | null>(null);
  const [statusLoading, setStatusLoading] = useState<string | null>(null);
  const [quickCheck, setQuickCheck] = useState<{ critical_count: number; has_critical: boolean; summary: string } | null>(null);

  const fetchAll = useCallback(async () => {
    try {
      const mod = await import("../commands");
      const [s, l] = await Promise.all([
        mod.securityScanSummary().catch(() => null),
        mod.securityScanList().catch(() => []),
      ]);
      if (s) setSummary(s);
      if (l.length > 0) setScans(l);
      setError(null);
    } catch {
      setError("Failed to load security scan data");
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    fetchAll();
    const timer = setInterval(fetchAll, 10000);
    return () => clearInterval(timer);
  }, [fetchAll]);

  const fetchFindings = useCallback(async (scanId: string) => {
    if (findings[scanId]) return;
    try {
      const mod = await import("../commands");
      const f = await mod.securityScanFindings(scanId);
      setFindings((prev) => ({ ...prev, [scanId]: f }));
    } catch { }
  }, [findings]);

  const handleNewScan = useCallback(async () => {
    setScanning(true);
    try {
      const mod = await import("../commands");
      await mod.securityScanStart(targetPath, depth);
      await fetchAll();
    } catch (e) {
      setError(`Scan failed: ${e}`);
    }
    setScanning(false);
  }, [targetPath, depth, fetchAll]);

  const handleQuickCheck = useCallback(async () => {
    try {
      const mod = await import("../commands");
      const qc = await mod.securityScanQuickCheck();
      setQuickCheck(qc);
    } catch { }
  }, []);

  const handleToggleScan = useCallback((scanId: string) => {
    if (expandedScan === scanId) {
      setExpandedScan(null);
      setExpandedFinding(null);
    } else {
      setExpandedScan(scanId);
      fetchFindings(scanId);
    }
  }, [expandedScan, fetchFindings]);

  const handleApplyPatch = useCallback(async (findingId: string) => {
    setPatchLoading(findingId);
    try {
      const mod = await import("../commands");
      await mod.securityScanApplyPatch(findingId);
      if (expandedScan) fetchFindings(expandedScan);
      await fetchAll();
    } catch { }
    setPatchLoading(null);
  }, [expandedScan, fetchFindings, fetchAll]);

  const handleMarkStatus = useCallback(async (findingId: string, status: string) => {
    setStatusLoading(findingId);
    try {
      const mod = await import("../commands");
      await mod.securityScanMarkStatus(findingId, status);
      if (expandedScan) {
        setFindings((prev) => ({
          ...prev,
          [expandedScan]: (prev[expandedScan] ?? []).map((f) =>
            f.id === findingId ? { ...f, status: status as VulnerabilityFinding["status"] } : f
          ),
        }));
      }
      await fetchAll();
    } catch { }
    setStatusLoading(null);
  }, [expandedScan, fetchAll]);

  const handleFixAll = useCallback(async (scanId: string) => {
    const f = findings[scanId];
    if (!f || f.length === 0) return;
    const ids = f.filter((x) => x.status === "Open" || x.status === "Verified").map((x) => x.id);
    if (ids.length === 0) return;
    try {
      const mod = await import("../commands");
      await mod.securityScanFixAll(ids);
      if (expandedScan) fetchFindings(expandedScan);
      await fetchAll();
    } catch { }
  }, [findings, expandedScan, fetchFindings, fetchAll]);

  if (loading) {
    return (
      <div className="lg-flex-col" style={{ height: "100%", padding: "var(--nt-gap-sm)", gap: "var(--nt-gap-md)" }}>
        <div className="lg-skeleton" style={{ height: 80 }} />
        <div className="lg-skeleton" style={{ height: 60 }} />
        <div className="lg-skeleton" style={{ flex: 1 }} />
      </div>
    );
  }

  return (
    <div className="lg-flex-col" style={{ height: "100%", padding: "var(--nt-gap-sm)", gap: "var(--nt-gap-md)", overflow: "hidden" }}>
      {/* Summary Stats Bar */}
      <div className="lg-glass-strong" style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(90px, 1fr))",
        gap: "var(--nt-gap-sm)",
        padding: "var(--nt-gap-md)",
        borderRadius: "var(--nt-radius-md)",
        flexShrink: 0,
        alignItems: "center",
      }}>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{summary?.total_scans ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Scans</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-text)" }}>{summary?.total_findings ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Findings</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-danger)" }}>{summary?.open_critical ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Critical</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-warning)" }}>{summary?.open_high ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>High</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "#ffbb00" }}>{summary?.open_medium ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Medium</div>
        </div>
        <div style={{ textAlign: "center" }}>
          <div style={{ fontSize: 20, fontWeight: 700, color: "var(--nt-success)" }}>{summary?.fixed_today ?? 0}</div>
          <div style={{ fontSize: 10, color: "var(--nt-text-secondary)" }}>Fixed Today</div>
        </div>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center" }}>
          <ScoreGauge score={summary?.security_score ?? 0} />
          <div style={{ fontSize: 9, color: "var(--nt-text-secondary)", textAlign: "center", marginLeft: 4 }}>
            Security<br />Score
          </div>
        </div>
      </div>

      {/* Toolbar */}
      <div style={{ display: "flex", alignItems: "center", gap: 8, flexShrink: 0, flexWrap: "wrap" }}>
        <input
          type="text"
          value={targetPath}
          onChange={(e) => setTargetPath(e.target.value)}
          placeholder="Target path"
          style={{
            flex: 1, minWidth: 120,
            padding: "6px 10px", borderRadius: "var(--nt-radius-sm)",
            background: "var(--nt-glass-bg)", border: "var(--nt-edge-width) solid var(--nt-glass-border)",
            color: "var(--nt-text)", fontSize: 12, outline: "none",
          }}
        />
        <select
          value={depth}
          onChange={(e) => setDepth(e.target.value)}
          style={{
            padding: "6px 10px", borderRadius: "var(--nt-radius-sm)",
            background: "var(--nt-glass-bg)", border: "var(--nt-edge-width) solid var(--nt-glass-border)",
            color: "var(--nt-text)", fontSize: 12, outline: "none",
          }}
        >
          <option value="shallow">Shallow</option>
          <option value="standard">Standard</option>
          <option value="deep">Deep</option>
        </select>
        <button className="lg-btn" onClick={handleNewScan} disabled={scanning} style={{ whiteSpace: "nowrap" }}>
          {scanning ? "\u23F3 Scanning..." : "\uD83D\uDD0D New Scan"}
        </button>
        <button className="lg-btn" onClick={handleQuickCheck} style={{ whiteSpace: "nowrap" }}>
          \u26A1 Quick Check
        </button>
        {quickCheck && (
          <span style={{ fontSize: 11, color: quickCheck.has_critical ? "var(--nt-danger)" : "var(--nt-success)" }}>
            {quickCheck.has_critical
              ? `\u26A0 ${quickCheck.critical_count} critical`
              : "\u2705 No critical issues"}
          </span>
        )}
      </div>

      {/* Error */}
      {error && (
        <div className="lg-empty" style={{ flexShrink: 0 }}>
          <div className="lg-empty-icon">\u26A0\uFE0F</div>
          <div className="lg-empty-text">{error}</div>
          <button className="lg-btn" onClick={fetchAll}>Retry</button>
        </div>
      )}

      {/* Scans List */}
      <div className="lg-scrollbar" style={{ flex: 1, overflow: "auto", display: "flex", flexDirection: "column", gap: 4 }}>
        {scans.length === 0 && !error && (
          <div className="lg-empty">
            <div className="lg-empty-icon">\uD83D\uDEE1\uFE0F</div>
            <div className="lg-empty-text">No scans yet</div>
            <div className="lg-empty-hint">Run a scan to detect vulnerabilities</div>
          </div>
        )}

        {scans.map((scan) => {
          const isOpen = expandedScan === scan.scan_id;
          const scanFindings = findings[scan.scan_id] ?? [];
          const sevColor = scoreColor(scan.overall_score);

          return (
            <div key={scan.scan_id} className="lg-fade-in" style={{ display: "flex", flexDirection: "column" }}>
              {/* Scan Header */}
              <div
                className="lg-glass-hover"
                onClick={() => handleToggleScan(scan.scan_id)}
                style={{
                  display: "flex", alignItems: "center", gap: 8,
                  padding: "8px 10px", borderRadius: isOpen ? "var(--nt-radius-sm) var(--nt-radius-sm) 0 0" : "var(--nt-radius-sm)",
                  background: "var(--nt-glass-bg)",
                  backdropFilter: "saturate(180%) blur(var(--nt-blur-sm))",
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  cursor: "pointer", transition: "all var(--nt-transition-fast)",
                }}
              >
                <span style={{ fontSize: 11, color: "var(--nt-text-muted)", flexShrink: 0, transition: "transform 0.2s", transform: isOpen ? "rotate(90deg)" : "rotate(0deg)" }}>
                  \u25B6
                </span>
                <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                    <span style={{ fontSize: 12, fontWeight: 600, color: "var(--nt-text)" }}>
                      {scan.target_path}
                    </span>
                    <span className="lg-badge" style={{ fontSize: 9 }}>{scan.total_files_scanned} files</span>
                    <span style={{ fontSize: 10, color: "var(--nt-text-muted)" }}>
                      {formatTime(scan.started_at)}
                    </span>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 4, fontSize: 10, color: "var(--nt-text-secondary)" }}>
                    <span>{severityBadge("Critical")} {scan.critical}</span>
                    <span>{severityBadge("High")} {scan.high}</span>
                    <span>{severityBadge("Medium")} {scan.medium}</span>
                    <span>{severityBadge("Low")} {scan.low}</span>
                    <span>{severityBadge("Info")} {scan.info}</span>
                  </div>
                </div>
                <div style={{ textAlign: "right", flexShrink: 0 }}>
                  <div style={{ fontSize: 16, fontWeight: 700, color: sevColor }}>{scan.overall_score}</div>
                  <div style={{ fontSize: 9, color: "var(--nt-text-muted)" }}>{formatDuration(scan.duration_ms)}</div>
                </div>
                <span style={{ fontSize: 9, color: "var(--nt-text-muted)", flexShrink: 0 }}>
                  {scan.scan_id.slice(0, 8)}
                </span>
              </div>

              {/* Expanded Findings */}
              {isOpen && (
                <div style={{
                  border: "var(--nt-edge-width) solid var(--nt-glass-border)",
                  borderTop: "none",
                  borderRadius: "0 0 var(--nt-radius-sm) var(--nt-radius-sm)",
                  background: "var(--nt-glass-bg)",
                  overflow: "hidden",
                }}>
                  {/* Fix All */}
                  {scanFindings.length > 0 && (
                    <div style={{ padding: "6px 10px", display: "flex", justifyContent: "flex-end" }}>
                      <button
                        className="lg-btn"
                        style={{ fontSize: 11, padding: "3px 10px" }}
                        onClick={() => handleFixAll(scan.scan_id)}
                      >
                        \uD83D\uDEE1 Fix All Open
                      </button>
                    </div>
                  )}

                  {scanFindings.length === 0 && (
                    <div style={{ padding: "12px 10px", textAlign: "center", fontSize: 11, color: "var(--nt-text-muted)" }}>
                      No findings for this scan
                    </div>
                  )}

                  {scanFindings.map((finding) => {
                    const isFindingOpen = expandedFinding === finding.id;
                    const sev = finding.severity;
                    const sevHex = SEVERITY_COLORS[sev] ?? "#888";

                    return (
                      <div key={finding.id} style={{ borderTop: "1px solid var(--nt-glass-border)" }}>
                        {/* Finding Header */}
                        <div
                          onClick={() => setExpandedFinding(isFindingOpen ? null : finding.id)}
                          style={{
                            display: "flex", alignItems: "center", gap: 8,
                            padding: "8px 10px", cursor: "pointer",
                            background: isFindingOpen ? "var(--nt-glass-bg)" : "transparent",
                            transition: "all var(--nt-transition-fast)",
                          }}
                        >
                          <span style={{ fontSize: 10, color: "var(--nt-text-muted)", transition: "transform 0.2s", transform: isFindingOpen ? "rotate(90deg)" : "rotate(0deg)" }}>
                            \u25B6
                          </span>
                          <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 2 }}>
                            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                              {severityBadge(finding.severity)}
                              <span style={{ fontSize: 12, fontWeight: 600, color: "var(--nt-text)" }}>
                                {finding.title}
                              </span>
                              {statusBadge(finding.status)}
                            </div>
                            <div style={{ fontSize: 10, color: "var(--nt-text-muted)" }}>
                              {finding.file_path}:{finding.line_start}-{finding.line_end}
                            </div>
                          </div>
                        </div>

                        {/* Expanded Finding Detail */}
                        {isFindingOpen && (
                          <div style={{
                            padding: "8px 10px 10px 24px",
                            display: "flex", flexDirection: "column", gap: 8,
                            borderTop: "1px solid var(--nt-glass-border)",
                          }}>
                            {/* Description */}
                            <div style={{ fontSize: 11, color: "var(--nt-text-secondary)", lineHeight: 1.5 }}>
                              {finding.description}
                            </div>

                            {/* CWE/CVE */}
                            {(finding.cwe_id || finding.cve_id) && (
                              <div style={{ display: "flex", gap: 8 }}>
                                {finding.cwe_id && (
                                  <a href={`https://cwe.mitre.org/data/definitions/${finding.cwe_id.replace("CWE-", "")}.html`}
                                    target="_blank" rel="noopener noreferrer"
                                    style={{ fontSize: 10, color: "var(--nt-info)", textDecoration: "underline" }}
                                  >
                                    {finding.cwe_id}
                                  </a>
                                )}
                                {finding.cve_id && (
                                  <a href={`https://www.cve.org/CVERecord?id=${finding.cve_id}`}
                                    target="_blank" rel="noopener noreferrer"
                                    style={{ fontSize: 10, color: "var(--nt-danger)", textDecoration: "underline" }}
                                  >
                                    {finding.cve_id}
                                  </a>
                                )}
                              </div>
                            )}

                            {/* Confidence Bar */}
                            <div>
                              <div style={{ display: "flex", justifyContent: "space-between", fontSize: 10, color: "var(--nt-text-muted)", marginBottom: 2 }}>
                                <span>Confidence</span>
                                <span>{(finding.confidence * 100).toFixed(0)}%</span>
                              </div>
                              <div style={{ height: 4, background: "var(--nt-glass-border)", borderRadius: 2, overflow: "hidden" }}>
                                <div style={{
                                  height: "100%", width: `${finding.confidence * 100}%`,
                                  background: finding.confidence >= 0.8 ? "var(--nt-success)"
                                    : finding.confidence >= 0.5 ? "var(--nt-warning)" : "var(--nt-danger)",
                                  borderRadius: 2, transition: "width 0.3s",
                                }} />
                              </div>
                            </div>

                            {/* Remediation */}
                            <div style={{
                              fontSize: 11, color: "var(--nt-text-secondary)", lineHeight: 1.5,
                              padding: "6px 8px", borderRadius: "var(--nt-radius-sm)",
                              background: "var(--nt-glass-bg)", border: "1px solid var(--nt-glass-border)",
                            }}>
                              <div style={{ fontSize: 10, fontWeight: 600, color: "var(--nt-success)", marginBottom: 2 }}>
                                \uD83D\uDD27 Remediation
                              </div>
                              {finding.remediation}
                            </div>

                            {/* Actions */}
                            <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                              {finding.patch_suggestion && finding.status === "Open" && (
                                <button
                                  className="lg-btn"
                                  style={{ fontSize: 11, padding: "3px 10px" }}
                                  onClick={() => handleApplyPatch(finding.id)}
                                  disabled={patchLoading === finding.id}
                                >
                                  {patchLoading === finding.id ? "\u23F3..." : "\uD83D\uDD28 Apply Patch"}
                                </button>
                              )}
                              {finding.status === "Open" && (
                                <>
                                  <button
                                    className="lg-btn"
                                    style={{ fontSize: 11, padding: "3px 10px", color: "var(--nt-success)" }}
                                    onClick={() => handleMarkStatus(finding.id, "Fixed")}
                                    disabled={statusLoading === finding.id}
                                  >
                                    \u2705 Fixed
                                  </button>
                                  <button
                                    className="lg-btn"
                                    style={{ fontSize: 11, padding: "3px 10px", color: "var(--nt-text-secondary)" }}
                                    onClick={() => handleMarkStatus(finding.id, "WontFix")}
                                    disabled={statusLoading === finding.id}
                                  >
                                    \uD83D\uDEAB WontFix
                                  </button>
                                  <button
                                    className="lg-btn"
                                    style={{ fontSize: 11, padding: "3px 10px", color: "var(--nt-info)" }}
                                    onClick={() => handleMarkStatus(finding.id, "FalsePositive")}
                                    disabled={statusLoading === finding.id}
                                  >
                                    \uD83D\uDE45 False Positive
                                  </button>
                                </>
                              )}
                              {finding.status === "Verified" && (
                                <button
                                  className="lg-btn"
                                  style={{ fontSize: 11, padding: "3px 10px", color: "var(--nt-success)" }}
                                  onClick={() => handleMarkStatus(finding.id, "Fixed")}
                                  disabled={statusLoading === finding.id}
                                >
                                  \u2705 Mark Fixed
                                </button>
                              )}
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};

SecurityScanPanel.displayName = "SecurityScanPanel";

export default SecurityScanPanel;
