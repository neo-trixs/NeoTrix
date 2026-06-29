import React, { useEffect, useState, useCallback } from "react";
import { sandboxStatus, sandboxExecute, sandboxKillAll } from "../lib/api";
import type { SandboxJobRequest } from "../lib/api";

interface Props {
  onClose: () => void;
}

const SandboxControl: React.FC<Props> = ({ onClose }) => {
  const [status, setStatus] = useState<Record<string, unknown>>({});
  const [cmd, setCmd] = useState("");
  const [args, setArgs] = useState("");
  const [timeoutSecs, setTimeoutSecs] = useState(30);
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    const s = await sandboxStatus();
    setStatus(s);
  }, []);

  useEffect(() => {
    refresh();
    const timer = setInterval(refresh, 5000);
    return () => clearInterval(timer);
  }, [refresh]);

  const handleRun = async () => {
    if (!cmd.trim()) return;
    setLoading(true);
    setResult(null);
    const req: SandboxJobRequest = {
      command: cmd.trim(),
      args: args.split(/\s+/).filter(Boolean),
      timeout_secs: timeoutSecs,
      allow_network: false,
    };
    const r = await sandboxExecute(req);
    setResult(
      `exit: ${r.exit_code ?? "killed"} | ${r.duration_ms}ms${r.timed_out ? " (timeout)" : ""}\n` +
      `--- stdout ---\n${r.stdout || "(empty)"}\n--- stderr ---\n${r.stderr || "(empty)"}`
    );
    setLoading(false);
    refresh();
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel sandbox-panel glass-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Sandbox</h2>
          <button className="btn-icon" onClick={onClose}>✕</button>
        </div>

        <div className="sandbox-status-bar">
          <span className={`sandbox-status-dot ${status.available ? "on" : "off"}`} />
          <span>沙箱 {status.available ? "可用" : "不可用"} ({status.platform ?? "?"})</span>
        </div>

        {status.current_mode && (
          <div className="sandbox-info">
            <div className="proxy-info-row"><span>模式</span><span>{String(status.current_mode)}</span></div>
            <div className="proxy-info-row"><span>超时</span><span>{String(status.default_timeout_secs)}s</span></div>
            <div className="proxy-info-row"><span>保护路径</span><span>{String((status.protected_paths as string[])?.join(", "))}</span></div>
          </div>
        )}

        <div className="sandbox-exec-section">
          <label className="sandbox-label">命令</label>
          <input className="sandbox-input" value={cmd} onChange={(e) => setCmd(e.target.value)} placeholder="e.g. /bin/echo" />

          <label className="sandbox-label">参数</label>
          <input className="sandbox-input" value={args} onChange={(e) => setArgs(e.target.value)} placeholder="e.g. hello world" />

          <label className="sandbox-label">超时 (秒)</label>
          <input className="sandbox-input" type="number" min={1} max={300} value={timeoutSecs} onChange={(e) => setTimeoutSecs(Number(e.target.value))} />

          <div className="sandbox-actions">
            <button className="btn-primary" onClick={handleRun} disabled={loading || !cmd.trim()}>
              {loading ? "执行中..." : "执行"}
            </button>
            <button className="btn-danger" onClick={async () => { await sandboxKillAll(); refresh(); }}>
              终止所有
            </button>
          </div>
        </div>

        {result && (
          <pre className="sandbox-result">{result}</pre>
        )}

        <div className="settings-footer">
          <button className="btn-secondary" onClick={onClose}>关闭</button>
        </div>
      </div>
    </div>
  );
};

export default SandboxControl;
