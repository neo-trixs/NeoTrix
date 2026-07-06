import React, { useState, useCallback } from "react";
import GlassPanel from "./GlassPanel";

interface ScrubResult {
  original: string;
  scrubbed: string;
  detected: Array<{ type: string; value: string; placeholder: string }>;
}

interface PiiPattern {
  type: string;
  regex: RegExp;
  severity: "high" | "medium" | "low";
}

const PII_PATTERNS: PiiPattern[] = [
  { type: "Email", regex: /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g, severity: "high" },
  { type: "Phone", regex: /\+?1?\s*\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}/g, severity: "high" },
  { type: "SSN", regex: /\d{3}-\d{2}-\d{4}/g, severity: "high" },
  { type: "IP Address", regex: /\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b/g, severity: "medium" },
  { type: "API Key", regex: /sk-[A-Za-z0-9]{20,}/g, severity: "high" },
  { type: "URL", regex: /https?:\/\/[^\s]+/g, severity: "medium" },
];

const SEVERITY_ICONS: Record<string, string> = {
  high: "\uD83D\uDD34",
  medium: "\uD83D\uDFE1",
  low: "\uD83D\uDFE2",
};

const SAMPLE_PII_DATA = `Contact: John Doe, john.doe@example.com, +1 (555) 123-4567
SSN: 123-45-6789
Server: 192.168.1.1:443
Key: sk-proj-abc123def456ghi789`;

const RECENT_ACTIVITY = [
  { time: "12:34:22", action: "2 PII items scrubbed", model: "claude-sonnet-4" },
  { time: "12:30:15", action: "Blocked: leak attempt", model: "gpt-4.1" },
  { time: "12:28:01", action: "1 email placeholder unscrubbed", model: "claude-sonnet-4" },
];

function scrubText(text: string, patterns: PiiPattern[]): ScrubResult {
  const detected: ScrubResult["detected"] = [];
  let scrubbed = text;

  for (const { type, regex } of patterns) {
    let match: RegExpExecArray | null;
    const r = new RegExp(regex.source, "g");
    while ((match = r.exec(text)) !== null) {
      const idx = detected.length + 1;
      const placeholder = `[${type.toUpperCase().replace(/\s+/g, "_")}_${idx}]`;
      detected.push({ type, value: match[0], placeholder });
      scrubbed = scrubbed.replace(match[0], placeholder);
    }
  }
  return { original: text, scrubbed, detected };
}

const PrivacyFilter: React.FC = () => {
  const [enabled, setEnabled] = useState(true);
  const [filterMode, setFilterMode] = useState<"auto" | "strict" | "custom">("auto");
  const [scrubTest, setScrubTest] = useState("");
  const [result, setResult] = useState<ScrubResult | null>(null);

  const handleScrub = useCallback(() => {
    const text = scrubTest || SAMPLE_PII_DATA;
    const r = scrubText(text, PII_PATTERNS);
    setResult(r);
  }, [scrubTest]);

  const detectionCount = result?.detected.length ?? 0;

  return (
    <div className="lg-split-panel">
      <GlassPanel
        variant="strong"
        header={<><span>\uD83D\uDEE1 Privacy Filter</span><span className={`lg-badge ${detectionCount > 0 ? "lg-badge-danger" : "lg-badge-success"}`}>{detectionCount > 0 ? `\uD83D\uDD34 ${detectionCount} detected` : "\uD83D\uDFE2 clear"}</span></>}
        className="lg-flex-col"
      >
        <div className="privacy-main-column">
          <div className="lg-card lg-card-padded lg-card-row">
            <div>
              <div className="privacy-card-title">On-Device Privacy Filter</div>
              <div className="privacy-card-subtitle">Scrub PII before sending to cloud models</div>
            </div>
            <button className={`lg-toggle ${enabled ? "active" : ""}`} onClick={() => setEnabled(!enabled)}>
              <span className="lg-toggle-knob" />
            </button>
          </div>

          <div className="lg-card lg-card-padded">
            <div className="lg-label">Filter Mode</div>
            <div className="filter-mode-row">
              {(["auto", "strict", "custom"] as const).map((mode) => (
                <button key={mode} className={`lg-btn ${filterMode === mode ? "lg-btn-primary" : ""}`} onClick={() => setFilterMode(mode)}>
                  {mode}
                </button>
              ))}
            </div>
          </div>

          <div className="lg-card lg-card-padded">
            <div className="lg-label">Detected PII Patterns ({detectionCount})</div>
            <div className="pii-pattern-grid">
              {PII_PATTERNS.map((p) => (
                <div key={p.type} className="pii-pattern-chip">
                  <span>{SEVERITY_ICONS[p.severity]}</span>
                  <span>{p.type}</span>
                </div>
              ))}
              <div className="pii-pattern-chip">
                <span>{"\uD83D\uDFE2"}</span>
                <span>Custom Regex</span>
              </div>
            </div>
          </div>

          <div className="scrub-test-section">
            <div className="lg-label">Test Scrub</div>
            <textarea
              className="lg-input scrub-test-input"
              value={scrubTest}
              onChange={(e) => setScrubTest(e.target.value)}
              placeholder={SAMPLE_PII_DATA}
            />
            <button className="lg-btn lg-btn-primary" onClick={handleScrub}>
              {"\uD83D\uDD0D"} Test Scrub
            </button>
          </div>

          {result && (
            <div className="scrub-results">
              <div className="lg-glass lg-glass-padded">
                <div className="lg-label">Scrubbed Output</div>
                <pre className="scrub-output-pre">{result.scrubbed}</pre>
              </div>
              {result.detected.length > 0 && (
                <div className="lg-glass lg-glass-padded">
                  <div className="lg-label">Detected Items</div>
                  {result.detected.map((d, i) => (
                    <div key={i} className="scrub-detected-row">
                      <span className="lg-badge scrub-type-badge">{d.type}</span>
                      <code className="scrub-value-text">{d.value}</code>
                      <span className="scrub-arrow">{"\u2192"}</span>
                      <code className="scrub-placeholder-text">{d.placeholder}</code>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </GlassPanel>

      <GlassPanel variant="clear" header={<><span>{"\u2699\uFE0F"} Config</span><span className="lg-badge">v1</span></>} className="lg-side-panel">
        <div className="privacy-config-column">
          <div className="privacy-config-field">
            <div className="lg-label">Redact Mode</div>
            <select className="lg-select" defaultValue="placeholder">
              <option value="placeholder">[PERSON_n] placeholders</option>
              <option value="mask">P\u2731\u2731\u2731\u2731\u2731\u2731\u2731D</option>
              <option value="drop">Drop entirely</option>
            </select>
          </div>
          <div className="privacy-config-field">
            <div className="lg-label">Unscrub on Reply</div>
            <button className="lg-toggle active"><span className="lg-toggle-knob" /></button>
            <div className="lg-hint-text">Restore placeholders in streaming replies</div>
          </div>
          <div className="lg-divider" />
          <div className="lg-label">Custom Patterns</div>
          <div className="lg-glass lg-add-pattern-btn">
            + Add custom regex pattern
          </div>
          <div className="lg-divider" />
          <div className="lg-label">Recent Activity</div>
          <div className="recent-activity-list">
            {RECENT_ACTIVITY.map((e, i) => (
              <div key={i} className="recent-activity-row">
                <span className="recent-time">{e.time}</span>
                <span className="recent-action-text">{e.action}</span>
                <span className="lg-badge">{e.model}</span>
              </div>
            ))}
          </div>
        </div>
      </GlassPanel>
    </div>
  );
};

PrivacyFilter.displayName = "PrivacyFilter";

export default PrivacyFilter;
