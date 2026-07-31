import React, { useCallback, useEffect, useState } from "react";
import type { NeoCodexProviderConfig } from "../../types";
import styles from "./ModelSelector.module.css";

export function ModelSelector({ onConfigChange }: { onConfigChange?: (config: string) => void }) {
  const [config, setConfig] = useState<NeoCodexProviderConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [switching, setSwitching] = useState<string | null>(null);
  const [expanded, setExpanded] = useState(false);

  const fetchConfig = useCallback(async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<NeoCodexProviderConfig>("neocodex_provider_config");
      setConfig(result);
      onConfigChange?.(result.active_model);
    } catch (e) {
      console.error("Failed to fetch provider config:", e);
    } finally {
      setLoading(false);
    }
  }, [onConfigChange]);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  const handleSwitch = async (name: string) => {
    if (name === config?.active_model) return;
    setSwitching(name);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("neocodex_set_provider", { name });
      await fetchConfig();
    } catch (e) {
      console.error("Switch provider failed:", e);
    } finally {
      setSwitching(null);
    }
  };

  if (loading) {
    return (
      <div className={styles.container}>
        <div className={styles.skeleton} />
      </div>
    );
  }

  if (!config) return null;

  return (
    <div className={styles.container}>
      <button
        className={styles.trigger}
        onClick={() => setExpanded(!expanded)}
        aria-expanded={expanded}
      >
        <span className={styles.modelName}>{config.active_model}</span>
        <span className={styles.badge} style={{ background: config.resolvable ? "var(--success)" : "var(--warning)" }}>
          {config.resolvable ? "可用" : "离线"}
        </span>
        <svg
          className={styles.chevron + (expanded ? " expanded" : "")}
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
        >
          <path d="M4 5l3 3 3-3" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>

      {expanded && (
        <div className={styles.dropdown}>
          <div className={styles.dropdownHeader}>
            <span className={styles.label}>可用 Providers ({config.provider_count})</span>
          </div>
          <div className={styles.providerList}>
            {config.providers.map((p) => (
              <button
                key={p.name}
                className={styles.providerItem}
                onClick={() => handleSwitch(p.name)}
                disabled={switching !== null}
              >
                <span className={styles.providerName}>{p.name}</span>
                <span className={styles.providerModel}>{p.model}</span>
                <span className={styles.providerStatus}>
                  {p.resolvable ? (
                    <span className={styles.okDot} title="可解析" />
                  ) : (
                    <span className={styles.offDot} title="离线" />
                  )}
                </span>
                {switching === p.name && <span className={styles.switching}>切换中...</span>}
              </button>
            ))}
          </div>
          <button
            className={styles.refreshBtn}
            onClick={() => { setLoading(true); fetchConfig(); }}
            disabled={loading}
          >
            {loading ? "刷新中..." : "刷新 Provider"}
          </button>
        </div>
      )}
    </div>
  );
}

export default ModelSelector;