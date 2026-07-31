import React, { useEffect, useState } from "react";
import { useStore } from "../../stores";
import type { NeoCodexHealthReport } from "../../types";
import styles from "./HealthPanel.module.css";

export function HealthPanel({ compact = false }: { compact?: boolean }) {
  const [health, setHealth] = useState<NeoCodexHealthReport | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchHealth = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const report = await invoke("neocodex_health_report") as NeoCodexHealthReport;
        setHealth(report);
      } catch (e) {
        console.error("Failed to fetch health:", e);
      } finally {
        setLoading(false);
      }
    };
    fetchHealth();
    const interval = setInterval(fetchHealth, 10000);
    return () => clearInterval(interval);
  }, []);

  if (loading || !health) {
    return compact ? (
      <div className={styles.skeleton} />
    ) : (
      <div className={styles.container}>
        <div className={styles.skeleton} style={{ height: "200px" }} />
      </div>
    );
  }

  const checks = [
    { label: "Provider 可达", pass: health.provider_resolvable, detail: `${health.provider_count} providers, ${health.provider_model}` },
    { label: "会话可写", pass: health.session_writable, detail: "" },
    { label: "上下文健康", pass: health.context_usage < 0.9, detail: `${(health.context_usage * 100).toFixed(0)}% (${health.context_turns} turns)` },
    { label: "Token 预算", pass: health.cost_spent < health.cost_budget, detail: `$${health.cost_spent.toFixed(4)} / $${health.cost_budget.toFixed(2)}` },
    { label: "工具接地", pass: !health.tool_grounding_degraded, detail: health.tool_grounding_degraded ? "降级" : "正常" },
    { label: "进化循环", pass: health.evolution_iterations > 0, detail: `${health.evolution_iterations} iterations` },
    { label: "意识连接", pass: health.consciousness_attached, detail: "" },
    { label: "大脑连接", pass: health.brain_attached, detail: "" },
  ];

  const passCount = checks.filter((c) => c.pass).length;
  const totalCount = checks.length;
  const healthScore = Math.round((passCount / totalCount) * 100);

  if (compact) {
    return (
      <div className={styles.compact}>
        <div className={styles.compactScore} style={{ color: healthScore >= 80 ? "var(--success)" : healthScore >= 50 ? "var(--warning)" : "var(--danger)" }}>
          {healthScore}%
        </div>
        <div className={styles.compactDetail}>
          <span>{passCount}/{totalCount} 项通过</span>
          <span className={styles.dot} />
          <span>{health.mode}</span>
          <span className={styles.dot} />
          <span>T{health.turn_count}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div className={styles.titleRow}>
          <h3 className={styles.title}>NeoCodex 健康报告</h3>
          <span className={styles.modeBadge}>{health.mode}</span>
        </div>
        <div className={styles.score} style={{ color: healthScore >= 80 ? "var(--success)" : healthScore >= 50 ? "var(--warning)" : "var(--danger)" }}>
          {healthScore}% <span className={styles.scoreLabel}>健康度</span>
        </div>
      </div>

      <div className={styles.grid}>
{checks.map((check, idx) => (
          <div key={idx} className={styles.checkCard}>
            <div className={styles.checkRow}>
              <span className={styles.checkIcon}>
                {check.pass ? (
                  <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="var(--success)" strokeWidth="2">
                    <path d="M3 7l3 3 5-5" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                ) : (
                  <svg width="16" height="16" viewBox="0 0 14 14" fill="none" stroke="var(--danger)" strokeWidth="2">
                    <path d="M7 2v10M2 7h10" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                )}
              </span>
              <span className={styles.checkLabel}>{check.label}</span>
            </div>
            {check.detail && <span className={styles.checkDetail}>{check.detail}</span>}
          </div>
        ))}
      </div>

      <div className={styles.stats}>
        <Stat label="轮次" value={health.turn_count} />
        <Stat label="工具调用" value={health.tool_call_count} />
        <Stat label="Tokens" value={health.tokens_used.toLocaleString()} />
        <Stat label="上下文" value={`${health.context_turns} turns`} />
        <Stat label="进化" value={`${health.evolution_iterations}`} />
        <Stat label="成本" value={`$${health.cost_spent.toFixed(4)}`} />
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string | number }) {
  return (
    <div className={styles.stat}>
      <span className={styles.statValue}>{value}</span>
      <span className={styles.statLabel}>{label}</span>
    </div>
  );
}

export default HealthPanel;