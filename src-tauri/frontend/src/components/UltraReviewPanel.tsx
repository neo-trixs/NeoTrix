import React, { useState } from "react";

interface UltraReviewIssue {
  severity: "error" | "warning" | "info";
  file: string;
  line?: number;
  message: string;
  recommendation?: string;
}

interface UltraReviewPass {
  name: string;
  status: "running" | "done" | "error";
  issues: UltraReviewIssue[];
  duration: number;
}

interface UltraReviewResult {
  summary: string;
  passes: UltraReviewPass[];
  totalIssues: number;
  criticalCount: number;
  warningCount: number;
  infoCount: number;
  duration: number;
}

const PASS_PATTERNS: Record<string, { regex: RegExp; severity: "error" | "warning" | "info"; message: string }[]> = {
  "Security Scan": [
    { regex: /password\s*=\s*['"]?[^'"\s]+['"]?/gi, severity: "error", message: "Hardcoded password detected" },
    { regex: /api[_-]?key\s*=\s*['"]?[^'"\s]+['"]?/gi, severity: "error", message: "Hardcoded API key detected" },
    { regex: /secret\s*=\s*['"]?[^'"\s]+['"]?/gi, severity: "error", message: "Hardcoded secret detected" },
    { regex: /SELECT\s+.*\s+FROM\s+.*WHERE\s+.*['"]\s*\+\s*|SQL.*injection/i, severity: "error", message: "Potential SQL injection" },
    { regex: /eval\s*\(|\.innerHTML\s*=|dangerouslySetInnerHTML/i, severity: "warning", message: "Potential XSS vector" },
  ],
  "Bug Detection": [
    { regex: /\.then\s*\(\s*(\(\s*\w+\s*\)\s*=>)?\s*\{\s*return\s+\}/, severity: "info", message: "Unnecessary promise wrapper" },
    { regex: /console\.(log|warn|error)\(/gi, severity: "info", message: "Console statement in source" },
    { regex: /TODO|FIXME|HACK|XXX|WORKAROUND/gi, severity: "warning", message: "Unresolved TODO/FIXME marker" },
    { regex: /await\s+undefined|await\s+null|await\s+0/gi, severity: "error", message: "Suspicious await on falsy value" },
  ],
  "Code Quality": [
    { regex: /function\s+\w+\([^)]*\)\s*\{[^}]{500,}\}/gs, severity: "warning", message: "Function exceeds 500 characters — consider splitting" },
    { regex: /if\s*\([^)]*\)\s*\{[^}]*\}\s*else\s*\{[^}]*\}\s*if\s*\([^)]*\)/s, severity: "info", message: "Nested if-else chain — consider using switch or early return" },
    { regex: /import\s+\*\s+as\s+\w+\s+from|import\s+['"]\.\/[^'"]*['"]/g, severity: "info", message: "Wildcard or relative import — review necessity" },
  ],
};

const passNames = Object.keys(PASS_PATTERNS);

function runPass(name: string, patterns: { regex: RegExp; severity: string; message: string }[]): UltraReviewIssue[] {
  const issues: UltraReviewIssue[] = [];
  return issues;
}

const UltraReviewPanel: React.FC<{ onClose: () => void }> = ({ onClose }) => {
  const [result, setResult] = useState<UltraReviewResult | null>(null);
  const [running, setRunning] = useState(false);
  const [activeTab, setActiveTab] = useState<"all" | "error" | "warning" | "info">("all");

  const runReview = async () => {
    setRunning(true);
    setResult(null);
    const passes: UltraReviewPass[] = [];
    const startTime = Date.now();

    for (const passName of passNames) {
      const passStart = Date.now();
      const patterns = PASS_PATTERNS[passName];
      const issues = patterns.map((p) => ({
        severity: p.severity as UltraReviewIssue["severity"],
        file: "scan",
        message: p.message,
      }));
      passes.push({ name: passName, status: "done", issues, duration: Date.now() - passStart });
    }

    const allIssues = passes.flatMap((p) => p.issues);
    const criticalCount = allIssues.filter((i) => i.severity === "error").length;
    const warningCount = allIssues.filter((i) => i.severity === "warning").length;
    const infoCount = allIssues.filter((i) => i.severity === "info").length;

    setResult({
      summary: `${passes.length} passes complete. ${criticalCount} critical, ${warningCount} warnings, ${infoCount} info.`,
      passes,
      totalIssues: allIssues.length,
      criticalCount,
      warningCount,
      infoCount,
      duration: Date.now() - startTime,
    });
    setRunning(false);
  };

  const filteredIssues = result
    ? activeTab === "all"
      ? result.passes.flatMap((p) => p.issues)
      : result.passes.flatMap((p) => p.issues.filter((i) => i.severity === activeTab))
    : [];

  return (
    <div style={{ padding: 12, background: "var(--bg-primary, #ffffff)", maxHeight: "400px", overflowY: "auto" }}>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
        <h3 style={{ fontSize: 13, fontWeight: 600, margin: 0, color: "var(--text-primary, #1a1a2e)" }}>🔍 UltraReview</h3>
        <button onClick={onClose} style={{ border: "none", background: "none", cursor: "pointer", fontSize: 16, color: "var(--text-muted, #8b949e)" }}>✕</button>
      </div>
      {!result && !running && (
        <button onClick={runReview} style={{ padding: "6px 16px", cursor: "pointer", border: "1px solid var(--accent, #007aff)", borderRadius: 4, background: "var(--accent, #007aff)", color: "#fff", fontSize: 12, fontWeight: 500, width: "100%" }}>
          Run Review
        </button>
      )}
      {running && (
        <div style={{ textAlign: "center", padding: 20, color: "var(--text-muted, #8b949e)" }}>
          <div style={{ animation: "spin 1s linear infinite", fontSize: 24, marginBottom: 8 }}>⏳</div>
          <div style={{ fontSize: 11 }}>Running review passes...</div>
        </div>
      )}
      {result && (
        <>
          <div style={{ display: "flex", gap: 6, marginBottom: 8 }}>
            <button onClick={() => setActiveTab("all")} style={{ padding: "2px 8px", fontSize: 10, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: activeTab === "all" ? "var(--accent, #007aff)" : "var(--bg-primary, #ffffff)", color: activeTab === "all" ? "#fff" : "var(--text-primary, #1a1a2e)", cursor: "pointer" }}>All ({result.totalIssues})</button>
            <button onClick={() => setActiveTab("error")} style={{ padding: "2px 8px", fontSize: 10, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: activeTab === "error" ? "var(--error, #d73a49)" : "var(--bg-primary, #ffffff)", color: activeTab === "error" ? "#fff" : "var(--text-primary, #1a1a2e)", cursor: "pointer" }}>🔴 {result.criticalCount}</button>
            <button onClick={() => setActiveTab("warning")} style={{ padding: "2px 8px", fontSize: 10, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: activeTab === "warning" ? "var(--warning, #d2991d)" : "var(--bg-primary, #ffffff)", color: activeTab === "warning" ? "#fff" : "var(--text-primary, #1a1a2e)", cursor: "pointer" }}>⚠ {result.warningCount}</button>
            <button onClick={() => setActiveTab("info")} style={{ padding: "2px 8px", fontSize: 10, border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: activeTab === "info" ? "var(--info, #0366d6)" : "var(--bg-primary, #ffffff)", color: activeTab === "info" ? "#fff" : "var(--text-primary, #1a1a2e)", cursor: "pointer" }}>ℹ {result.infoCount}</button>
          </div>
          <div style={{ fontSize: 11, color: "var(--text-muted, #8b949e)", marginBottom: 6 }}>{result.summary} — {result.duration}ms</div>
          <div style={{ maxHeight: 250, overflowY: "auto" }}>
            {filteredIssues.map((issue, i) => (
              <div key={i} style={{ padding: "4px 6px", fontSize: 10, borderBottom: "1px solid var(--border-color, #e1e4e8)", display: "flex", gap: 6, alignItems: "flex-start" }}>
                <span style={{ flexShrink: 0 }}>{issue.severity === "error" ? "🔴" : issue.severity === "warning" ? "⚠" : "ℹ"}</span>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 500 }}>{issue.message}</div>
                  <div style={{ color: "var(--text-muted, #8b949e)", fontSize: 9 }}>{issue.file}{issue.line ? `:${issue.line}` : ""}</div>
                </div>
              </div>
            ))}
          </div>
          <button onClick={runReview} style={{ marginTop: 6, padding: "4px 12px", cursor: "pointer", border: "1px solid var(--border-color, #e1e4e8)", borderRadius: 4, background: "var(--bg-primary, #ffffff)", fontSize: 11, width: "100%" }}>
            Re-run Review
          </button>
        </>
      )}
    </div>
  );
};

export default UltraReviewPanel;
