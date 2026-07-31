import React, { useEffect, useState } from "react";
import { useStore } from "../../stores";
import type { NeoCodexProviderConfig } from "../../types";
import styles from "./ModelSelector.module.css";

export function ModelSelector({ onConfigChange }: { onConfigChange?: (config: string) => void }) {
  const [config, setConfig] = useState<NeoCodexProviderConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const result = await invoke("neocodex_provider_config") as string;
        // Parse "provider_count=24 resolvable=true active_model=gpt-4"
        const parts = result.split(" ");
        const parsed: NeoCodexProviderConfig = {
          provider_count: Number(parts[0]?.split("=")[1] || 0),
          resolvable: parts[1]?.split("=")[1] === "true",
          active_model: parts[2]?.split("=")[1] || "unknown",
        };
        setConfig(parsed);
      } catch (e) {
        console.error("Failed to fetch provider config:", e);
      } finally {
        setLoading(false);
      }
    };
    fetchConfig();
  }, []);

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
          <div className={styles.dropdownItem}>
            <span className={styles.label}>可用 Providers</span>
            <span className={styles.value}>{config.provider_count}</span>
          </div>
          <div className={styles.dropdownItem}>
            <span className={styles.label}>当前模型</span>
            <span className={styles.value}>{config.active_model}</span>
          </div>
          <div className={styles.dropdownItem}>
            <span className={styles.label}>状态</span>
            <span className={styles.value}>
              <span className={styles.statusDot} style={{ background: config.resolvable ? "var(--success)" : "var(--warning)" }} />
              {config.resolvable ? "可用" : "离线"}
            </span>
          </div>
          <button
            className={styles.refreshBtn}
            onClick={() => {
              setLoading(true);
              // Re-fetch would be implemented here
            }}
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