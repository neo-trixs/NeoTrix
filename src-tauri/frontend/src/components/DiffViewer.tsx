import React, { useState } from "react";
import type { DiffBlock } from "../types";

interface Props {
  diffBlocks: DiffBlock[];
  filename?: string;
  onApply?: () => void;
  onReject?: () => void;
}

/**
 * DiffViewer — displays file diffs with per-line change highlighting.
 * Supports collapsible view, add/remove/unchanged block types, and apply/reject actions.
 */
const DiffViewer: React.FC<Props> = ({ diffBlocks, filename, onApply, onReject }) => {
  const [collapsed, setCollapsed] = useState(false);

  if (collapsed) {
    return (
      <div className="diff-viewer diff-collapsed" onClick={() => setCollapsed(false)}>
        <span className="diff-summary">
          {filename || "diff"} · {diffBlocks.filter((b) => b.type === "added").length} 处添加 / {diffBlocks.filter((b) => b.type === "removed").length} 处删除
        </span>
        <span className="diff-expand">展开</span>
      </div>
    );
  }

  return (
    <div className="diff-viewer glass-panel">
      <div className="diff-header">
        <span className="diff-filename">{filename || "文件变更"}</span>
        <div className="diff-actions">
          {onApply && <button className="btn-diff-apply" onClick={onApply}>接受</button>}
          {onReject && <button className="btn-diff-reject" onClick={onReject}>拒绝</button>}
          <button className="btn-icon" onClick={() => setCollapsed(true)} title="折叠">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M4 9l3-3 3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>
      <div className="diff-stats">
        <span className="diff-stat-added">+{diffBlocks.filter((b) => b.type === "added").length}</span>
        <span className="diff-stat-removed">-{diffBlocks.filter((b) => b.type === "removed").length}</span>
        <span className="diff-stat-unchanged">={diffBlocks.filter((b) => b.type === "unchanged").length}</span>
      </div>
      <div className="diff-content">
        {diffBlocks.map((block, i) => (
          <div key={i} className={`diff-line diff-${block.type}`}>
            <span className="diff-line-num">{block.lineStart}</span>
            <span className="diff-line-sign">{block.type === "added" ? "+" : block.type === "removed" ? "-" : " "}</span>
            <span className="diff-line-text">{block.content}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default DiffViewer;
