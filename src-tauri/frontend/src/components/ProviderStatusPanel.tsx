import React, { useEffect, useState } from "react";
import { getProviderStatus, type ProviderStateInfo } from "../lib/api";
import styles from "./ProviderStatusPanel.module.css";

const CIRCUIT_COLORS: Record<string, string> = {
  Closed: "var(--nt-success)",
  HalfOpen: "var(--nt-warning)",
  Open: "var(--nt-danger)",
};

const ProviderStatusPanel: React.FC = () => {
  const [providers, setProviders] = useState<ProviderStateInfo[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const fetch = async () => {
      setLoading(true);
      const data = await getProviderStatus();
      if (!cancelled) {
        setProviders(data);
        setLoading(false);
      }
    };
    fetch();
    const interval = setInterval(fetch, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  if (loading && providers.length === 0) {
    return (
      <div className="provider-status-panel">
        <h4>Provider Status</h4>
        <p className={styles.loading}>Loading provider status...</p>
      </div>
    );
  }

  if (providers.length === 0) {
    return (
      <div className="provider-status-panel">
        <h4>Provider Status</h4>
        <p className={styles.loading}>No providers registered.</p>
      </div>
    );
  }

  return (
    <div className="provider-status-panel">
      <div className={styles.headerWrap}>
        <h4 className={styles.headerTitle}>Provider Status</h4>
        <span className={styles.headerHint}>(auto-refresh 5s)</span>
      </div>
      <div className={styles.list}>
        {providers.map((p) => (
          <div
            key={p.name}
            className={`${styles.card} ${p.available ? styles.cardAvailable : styles.cardUnavailable}`}
          >
            <div className={styles.left}>
              <span
                className={styles.dot}
                style={{ background: CIRCUIT_COLORS[p.circuit_state] || "var(--nt-text-muted)" }}
              />
              <span className={styles.name}>{p.name}</span>
              {p.is_free && (
                <span className={styles.freeBadge}>free</span>
              )}
            </div>
            <div className={styles.right}>
              <span>score: <b>{p.composite_score}</b></span>
              <span>success: <b>{p.success_rate}</b></span>
              <span>calls: <b>{p.total_calls}</b></span>
              <span>{p.circuit_state}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ProviderStatusPanel;
