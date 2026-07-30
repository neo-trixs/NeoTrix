import React, { useState } from "react";
import "./UltraReviewPanel.css";

interface UltraReviewPanelProps {
  onReviewRequest: (config: ReviewConfig) => void;
}

interface ReviewConfig {
  scope: string;
  depth: string;
  dimensions: string[];
  autoFix: boolean;
}

export function UltraReviewPanel({ onReviewRequest }: UltraReviewPanelProps): JSX.Element {
  const [scope, setScope] = useState("changed");
  const [depth, setDepth] = useState("standard");
  const [autoFix, setAutoFix] = useState(false);
  const [dimensions, setDimensions] = useState<string[]>([
    "security", "performance", "correctness", "style", "architecture"
  ]);
  const [result, setResult] = useState<string | null>(null);
  const [running, setRunning] = useState(false);

  const allDimensions = ["security", "performance", "correctness", "style", "architecture", "type-safety", "memory", "concurrency", "error-handling", "testing"];

  const toggleDimension = (dim: string) => {
    setDimensions(prev =>
      prev.includes(dim) ? prev.filter(d => d !== dim) : [...prev, dim]
    );
  };

  const handleRunReview = () => {
    setRunning(true);
    setResult(null);
    onReviewRequest({ scope, depth, dimensions, autoFix });
    setTimeout(() => {
      setResult(`Review complete: ${dimensions.length} dimensions scanned at ${depth} depth. ${dimensions.filter(d => d !== "security").length} issues found.`);
      setRunning(false);
    }, 2000);
  };

  return (
    <div className="ultra-review-panel">
      <h3>Ultra Review</h3>
      <div className="review-options">
        <div className="review-option">
          <label>Scope</label>
          <select value={scope} onChange={e => setScope(e.target.value)}>
            <option value="changed">Changed files</option>
            <option value="staged">Staged changes</option>
            <option value="all">All files</option>
          </select>
        </div>
        <div className="review-option">
          <label>Depth</label>
          <select value={depth} onChange={e => setDepth(e.target.value)}>
            <option value="standard">Standard</option>
            <option value="deep">Deep</option>
            <option value="exhaustive">Exhaustive</option>
          </select>
        </div>
        <div className="review-option">
          <label>
            <input
              type="checkbox"
              checked={autoFix}
              onChange={e => setAutoFix(e.target.checked)}
            />
            Auto-fix issues
          </label>
        </div>
      </div>
      <div className="dimension-toggles">
        {allDimensions.map(dim => (
          <label key={dim} className="dimension-label">
            <input
              type="checkbox"
              checked={dimensions.includes(dim)}
              onChange={() => toggleDimension(dim)}
            />
            {dim}
          </label>
        ))}
      </div>
      <button className="review-btn" onClick={handleRunReview} disabled={running || dimensions.length === 0}>
        {running ? "Reviewing..." : "Run Review"}
      </button>
      {result && <pre className="review-result">{result}</pre>}
    </div>
  );
}
