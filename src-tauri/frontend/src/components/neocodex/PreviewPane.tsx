import React, { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./PreviewPane.module.css";

type PreviewMode = "iframe" | "external";
type DeviceType = "desktop" | "tablet" | "mobile";

interface ConsoleEntry {
  level: "info" | "warn" | "error";
  message: string;
  timestamp: number;
  source: string;
}

interface PreviewSession {
  id: string;
  url: string;
  title: string;
  width: number;
  height: number;
  status: "loading" | "ready" | "error" | "closed";
  started_at: number;
}

const DEVICE_PRESETS: Record<DeviceType, { width: number; height: number; label: string }> = {
  desktop: { width: 1280, height: 720, label: "Desktop" },
  tablet: { width: 768, height: 1024, label: "Tablet" },
  mobile: { width: 375, height: 667, label: "Mobile" },
};

export function PreviewPane() {
  const [url, setUrl] = useState("http://localhost:5173");
  const [state, setState] = useState<{ title?: string; url?: string; error?: string }>({});
  const [opening, setOpening] = useState(false);
  const [mode, setMode] = useState<PreviewMode>("iframe");
  const [device, setDevice] = useState<DeviceType>("desktop");
  const [consoleLogs, setConsoleLogs] = useState<ConsoleEntry[]>([]);
  const [showConsole, setShowConsole] = useState(false);
  const [autoReload, setAutoReload] = useState(false);
  const [sessions, setSessions] = useState<PreviewSession[]>([]);
  const [currentSessionId, setCurrentSessionId] = useState<string | null>(null);

  const iframeRef = useRef<HTMLIFrameElement>(null);
  const reloadTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const consolePollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      const list = await invoke<PreviewSession[]>("preview_list");
      setSessions(list);
    } catch {
      // ignore
    }
  }, []);

  const pollConsoleLogs = useCallback(async () => {
    try {
      const logs = await invoke<ConsoleEntry[]>("chrome_debug_get_console_logs");
      setConsoleLogs(logs);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    loadSessions();
    consolePollRef.current = setInterval(pollConsoleLogs, 2000);
    return () => {
      if (consolePollRef.current) clearInterval(consolePollRef.current);
    };
  }, [loadSessions, pollConsoleLogs]);

  const open = async () => {
    setOpening(true);
    try {
      if (mode === "iframe") {
        const sessionId = await invoke<string>("preview_start", { url, width: DEVICE_PRESETS[device].width, height: DEVICE_PRESETS[device].height });
        setCurrentSessionId(sessionId);
        await loadSessions();
        const s = await invoke("browser_open", { url });
        setState((s as any) ?? {});
      } else {
        const s = await invoke("browser_open", { url });
        setState((s as any) ?? {});
      }
    } catch (e) {
      setState({ title: String(e), url, error: String(e) });
    } finally {
      setOpening(false);
    }
  };

  const reload = async () => {
    if (currentSessionId) {
      await invoke("preview_reload", { session_id: currentSessionId });
      await loadSessions();
    }
    if (iframeRef.current) {
      iframeRef.current.src = iframeRef.current.src;
    }
  };

  const openExternal = async () => {
    await invoke("browser_open", { url });
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const fileUrl = URL.createObjectURL(file);
      setUrl(fileUrl);
      open();
    }
  };

  const handleIframeLoad = () => {
    if (currentSessionId) {
      setSessions((prev) =>
        prev.map((s) =>
          s.id === currentSessionId ? { ...s, status: "ready" as const } : s
        )
      );
    }
  };

  const handleIframeError = () => {
    if (currentSessionId) {
      setSessions((prev) =>
        prev.map((s) =>
          s.id === currentSessionId ? { ...s, status: "error" as const } : s
        )
      );
    }
    setState({ title: "Failed to load", url, error: "Failed to load the preview" });
  };

  useEffect(() => {
    if (autoReload && currentSessionId) {
      reloadTimeoutRef.current = setTimeout(() => {
        reload();
      }, 3000);
    }
    return () => {
      if (reloadTimeoutRef.current) clearTimeout(reloadTimeoutRef.current);
    };
  }, [autoReload, currentSessionId]);

  const devicePreset = DEVICE_PRESETS[device];
  const iframeStyle: React.CSSProperties = mode === "iframe" ? {
    width: devicePreset.width,
    height: devicePreset.height,
    border: "1px solid var(--border-primary, rgba(255, 255, 255, 0.08))",
    borderRadius: "4px",
    background: "white",
    transform: `scale(${Math.min(1, (window.innerWidth - 48) / devicePreset.width)})`,
    transformOrigin: "top left",
  } : {};

  return (
    <div className={styles.panel} data-testid="preview-pane">
      <div className={styles.header}>
        <span className={styles.title}>预览</span>
        <div className={styles.urlGroup}>
          <input
            className={styles.urlInput}
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && open()}
            placeholder="http://localhost:5173"
            data-testid="preview-url"
          />
          <input
            type="file"
            className={styles.fileInput}
            accept=".html,.htm,.pdf,.png,.jpg,.jpeg,.gif,.svg,.webp"
            onChange={handleFileSelect}
            data-testid="preview-file-input"
          />
          <label className={styles.fileBtn} htmlFor="preview-file-input">
            文件
          </label>
          <select
            className={styles.modeSelect}
            value={mode}
            onChange={(e) => setMode(e.target.value as PreviewMode)}
            data-testid="preview-mode"
          >
            <option value="iframe">内嵌预览</option>
            <option value="external">外部浏览器</option>
          </select>
        </div>
        <div className={styles.actions}>
          <button
            type="button"
            className={styles.openBtn}
            onClick={open}
            disabled={opening}
            data-testid="preview-open"
          >
            {opening ? "打开中…" : "打开"}
          </button>
          {mode === "iframe" && currentSessionId && (
            <>
              <button
                type="button"
                className={styles.reloadBtn}
                onClick={reload}
                data-testid="preview-reload"
                title="刷新"
              >
                ⟳
              </button>
              <button
                type="button"
                className={styles.autoReloadBtn}
                onClick={() => setAutoReload((v) => !v)}
                data-testid="preview-auto-reload"
                title={autoReload ? "停止自动刷新" : "开始自动刷新"}
              >
                {autoReload ? "⏹" : "⟲"}
              </button>
            </>
          )}
          <button
            type="button"
            className={styles.externalBtn}
            onClick={openExternal}
            data-testid="preview-external"
            title="在外部浏览器打开"
          >
            ↗
          </button>
        </div>
      </div>

      {mode === "iframe" && (
        <div className={styles.toolbar}>
          <div className={styles.deviceToolbar}>
            <span className={styles.toolbarLabel}>设备:</span>
            {Object.entries(DEVICE_PRESETS).map(([key, preset]) => (
              <button
                key={key}
                type="button"
                className={`${styles.deviceBtn} ${device === key ? styles.deviceBtnActive : ""}`}
                onClick={() => setDevice(key as DeviceType)}
                data-testid={`preview-device-${key}`}
                title={preset.label}
              >
                {preset.label}
              </button>
            ))}
          </div>
          <div className={styles.consoleToggle}>
            <label className={styles.toggleLabel}>
              <input
                type="checkbox"
                checked={showConsole}
                onChange={async (e) => {
                  setShowConsole(e.target.checked);
                  if (e.target.checked) {
                    await pollConsoleLogs();
                  }
                }}
                data-testid="preview-console-toggle"
              />
              控制台 ({consoleLogs.length})
            </label>
          </div>
        </div>
      )}

      <div className={styles.content}>
        {mode === "iframe" ? (
          <div className={styles.iframeWrapper} style={{ overflow: "auto" }}>
            <iframe
              ref={iframeRef}
              src={url}
              style={iframeStyle}
              onLoad={handleIframeLoad}
              onError={handleIframeError}
              sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals allow-downloads"
              data-testid="preview-iframe"
            />
          </div>
        ) : (
          <div className={styles.hint} data-testid="preview-hint">
            当前模式：外部浏览器。点击"打开"在系统默认浏览器中打开 {url}。
          </div>
        )}
      </div>

      {showConsole && (
        <div className={styles.consolePanel} data-testid="preview-console">
          <div className={styles.consoleHeader}>
            <span className={styles.consoleTitle}>控制台日志</span>
            <button
              type="button"
              className={styles.clearBtn}
              onClick={() => invoke("chrome_debug_clear_console_logs").then(pollConsoleLogs)}
              data-testid="preview-console-clear"
            >
              清空
            </button>
          </div>
          <div className={styles.consoleList}>
            {consoleLogs.length === 0 ? (
              <div className={styles.consoleEmpty}>暂无日志</div>
            ) : (
              consoleLogs.slice().reverse().map((log, idx) => (
                <div
                  key={idx}
                  className={`${styles.consoleEntry} ${styles[`console${log.level.charAt(0).toUpperCase() + log.level.slice(1)}`]} `}
                  data-testid={`preview-console-entry-${idx}`}
                >
                  <span className={styles.consoleTime}>
                    {new Date(log.timestamp * 1000).toLocaleTimeString()}
                  </span>
                  <span className={styles.consoleLevel}>[{log.level}]</span>
                  <span className={styles.consoleMessage}>{log.message}</span>
                  <span className={styles.consoleSource}>{log.source}</span>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      <div className={`${styles.hint} ${state.error ? styles.hintError : ""}`} data-testid="preview-status">
        {state.error
          ? `打开失败: ${state.error}`
          : state.title
          ? `已打开: ${state.title}`
          : "输入 dev server 地址，在内置浏览器预览应用。也可直接打开 HTML/PDF/图片路径。"}
      </div>
    </div>
  );
}