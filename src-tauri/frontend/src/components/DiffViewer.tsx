import React, { useState } from "react";
import type { DiffBlock } from "../types";
import styles from "./DiffViewer.module.css";

interface Props {
  diffBlocks: DiffBlock[];
  filename?: string;
  onApply?: () => void;
  onReject?: () => void;
}

const typeLineMap: Record<string, string> = {
  added: styles.lineAdded,
  removed: styles.lineRemoved,
  unchanged: "",
};

const DiffViewer: React.FC<Props> = ({ diffBlocks, filename, onApply, onReject }) => {
  const [collapsed, setCollapsed] = useState(false);

  if (collapsed) {
    return (
      <div className={`${styles.viewer} ${styles.collapsed}`} onClick={() => setCollapsed(false)}>
        <span className={styles.summary}>
          {filename || "diff"} · {diffBlocks.filter((b) => b.type === "added").length} 处添加 / {diffBlocks.filter((b) => b.type === "removed").length} 处删除
        </span>
        <span className={styles.expand}>展开</span>
      </div>
    );
  }

  return (
    <div className={`${styles.viewer} glass-panel`}>
      <div className={styles.header}>
        <span className="diff-filename">{filename || "文件变更"}</span>
        <div className={styles.actions}>
          {onApply && <button className={styles.btnApply} onClick={onApply}>接受</button>}
          {onReject && <button className={styles.btnReject} onClick={onReject}>拒绝</button>}
          <button className="btn-icon" onClick={() => setCollapsed(true)} title="折叠">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M4 9l3-3 3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>
      <div className={styles.stats}>
        <span className={styles.statAdded}>+{diffBlocks.filter((b) => b.type === "added").length}</span>
        <span className={styles.statRemoved}>-{diffBlocks.filter((b) => b.type === "removed").length}</span>
        <span className={styles.statUnchanged}>={diffBlocks.filter((b) => b.type === "unchanged").length}</span>
      </div>
      <div className={styles.content}>
        {diffBlocks.map((block, i) => (
          <div key={i} className={`${styles.line} ${typeLineMap[block.type] || ""}`} data-testid="diff-line" data-diff-type={block.type}>
            <span className={styles.lineNum}>{block.lineStart}</span>
            <span className={styles.lineSign}>{block.type === "added" ? "+" : block.type === "removed" ? "-" : " "}</span>
            <span className={styles.lineText}>{block.content}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default DiffViewer;
