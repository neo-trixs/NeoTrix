import { useState } from "react";
import type { ToolCall } from "../types";
import DiffPreview from "./DiffPreview";

function toolIcon(tool: string): string {
  switch (tool) {
    case "ReadFile": return "📖";
    case "WriteFile": return "✏️";
    case "Glob": return "🔍";
    case "Grep": return "🔎";
    case "RunCommand": return "💻";
    case "ReadDir": return "📂";
    default: return "⚙";
  }
}

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

function argSummary(args: Record<string, string>): string {
  const v = Object.values(args)[0] || "";
  return v.length > 60 ? v.slice(0, 60) + "…" : v;
}

function isDiffContent(content: string): boolean {
  return /^diff --git a\//m.test(content) || /^--- a\//m.test(content);
}

export default function ToolCallCard({
  tool,
  args,
  status,
  result,
  duration_ms: durationMs,
}: ToolCall) {
  const [expanded, setExpanded] = useState(false);
  const hasDiff = typeof result === "string" && isDiffContent(result);

  return (
    <div className={`tcc tcc-${status}`}>
      {/* header */}
      <div
        className="tcc-header"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="tcc-icon">{toolIcon(tool)}</span>
        <strong className="tcc-name">{tool}</strong>
        <div className="tcc-status">
          {status === "running" && <span className="tcc-status-badge tcc-running"><span className="tcc-spinner" /> running…</span>}
          {status === "success" && <span className="tcc-status-badge tcc-ok">✓ {durationMs ? formatDuration(durationMs) : "done"}</span>}
          {status === "error" && <span className="tcc-status-badge tcc-fail">✗ failed</span>}
        </div>
        <span className="tcc-toggle">{expanded ? "▾" : "▸"}</span>
      </div>

      {/* collapsed: arg preview */}
      {!expanded && status !== "running" && (
        <div className="tcc-preview">{argSummary(args)}</div>
      )}

      {/* running: subtle progress bar */}
      {status === "running" && (
        <div className="tcc-progress">
          <div className="tcc-progress-bar" />
        </div>
      )}

      {/* expanded: full detail */}
      {expanded && (
        <div className="tcc-detail">
          {Object.entries(args).map(([k, v]) => (
            <div key={k} className="tcc-arg">
              <span className="tcc-arg-key">{k}</span>
              <pre className="tcc-arg-val">{v}</pre>
            </div>
          ))}
          {result && (
            <div className="tcc-result">
              <div className="tcc-result-label">
                {status === "error" ? "Error" : "Result"}
              </div>
              {hasDiff ? (
                <DiffPreview content={result} defaultCollapsed={false} />
              ) : (
                <pre className="tcc-result-text">{result}</pre>
              )}
            </div>
          )}
        </div>
      )}

      <style>{`
        @keyframes tcc-spin { to { transform: rotate(360deg); } }
        @keyframes tcc-progress { 0% { width: 0%; } 50% { width: 70%; } 100% { width: 90%; } }

        .tcc {
          margin: 8px 0;
          border: 1px solid var(--border-color, #e0e0e0);
          border-radius: 8px;
          overflow: hidden;
          font-size: 13px;
          font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, monospace;
          transition: box-shadow 0.2s;
        }
        .tcc:hover { box-shadow: 0 1px 6px rgba(0,0,0,0.06); }

        .tcc-running { border-left: 3px solid #007aff; }
        .tcc-success { border-left: 3px solid #34C759; }
        .tcc-error { border-left: 3px solid #FF453A; }

        .tcc-header {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          cursor: pointer;
          user-select: none;
          background: var(--bg-secondary, rgba(0,0,0,0.02));
        }
        .tcc-header:hover { background: var(--bg-hover, rgba(0,0,0,0.04)); }

        .tcc-icon { font-size: 16px; line-height: 1; }
        .tcc-name { flex: 1; font-size: 13px; }
        .tcc-status { display: flex; align-items: center; }

        .tcc-status-badge {
          font-size: 11px;
          padding: 2px 8px;
          border-radius: 10px;
          font-weight: 500;
          display: flex;
          align-items: center;
          gap: 4px;
        }
        .tcc-running { background: rgba(0,122,255,0.1); color: #007aff; }
        .tcc-ok { background: rgba(52,199,89,0.1); color: #34C759; }
        .tcc-fail { background: rgba(255,69,58,0.1); color: #FF453A; }

        .tcc-spinner {
          display: inline-block;
          width: 10px;
          height: 10px;
          border: 2px solid rgba(0,122,255,0.2);
          border-top-color: #007aff;
          border-radius: 50%;
          animation: tcc-spin 0.6s linear infinite;
        }

        .tcc-toggle { font-size: 10px; opacity: 0.4; }
        .tcc-preview { padding: 4px 12px 8px; color: var(--text-secondary, #666); font-size: 12px; }

        .tcc-progress {
          height: 2px;
          background: rgba(0,0,0,0.04);
        }
        .tcc-progress-bar {
          height: 100%;
          background: var(--mac-primary);
          animation: tcc-progress 3s ease-out forwards;
          border-radius: 0 2px 2px 0;
        }

        .tcc-detail {
          padding: 8px 12px;
          background: var(--bg-tertiary, rgba(0,0,0,0.02));
          border-top: 1px solid var(--border-color, rgba(0,0,0,0.06));
        }

        .tcc-arg {
          display: flex;
          gap: 8px;
          margin-bottom: 4px;
        }
        .tcc-arg-key {
          color: var(--text-tertiary, #888);
          min-width: 56px;
          flex-shrink: 0;
          font-size: 12px;
        }
        .tcc-arg-val {
          margin: 0;
          white-space: pre-wrap;
          word-break: break-all;
          font-size: 12px;
        }

        .tcc-result { margin-top: 8px; }
        .tcc-result-label {
          color: var(--text-tertiary, #888);
          margin-bottom: 4px;
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        .tcc-result-text {
          margin: 0;
          white-space: pre-wrap;
          word-break: break-all;
          font-size: 12px;
          background: var(--bg-code, rgba(0,0,0,0.03));
          padding: 8px;
          border-radius: 4px;
          max-height: 200px;
          overflow: auto;
        }
      `}</style>
    </div>
  );
}
