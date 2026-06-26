import React, { useCallback } from "react";

interface DiffViewProps {
  file: string;
  added: number;
  removed: number;
  diff: string;
  onAccept?: () => void;
  onReject?: () => void;
  status?: "accepted" | "rejected";
}

const DiffView: React.FC<DiffViewProps> = ({ file, added, removed, diff, onAccept, onReject, status }) => {
  const lines = diff.split("\n");

  return (
    <div className="diff-view">
      <div className="diff-header">
        <span className="diff-filename">{file}</span>
        <span className="diff-stats">
          <span className="diff-stats-added">+{added}</span>
          <span className="diff-stats-removed">-{removed}</span>
        </span>
        <span className="diff-actions">
          {status === "accepted" && <span className="diff-status accepted">✅ Accepted</span>}
          {status === "rejected" && <span className="diff-status rejected">❌ Rejected</span>}
          {!status && onAccept && <button className="btn-accept" onClick={() => onAccept()} title="Accept diff (Cmd+Enter)">✓</button>}
          {!status && onReject && <button className="btn-reject" onClick={() => onReject()} title="Reject diff (Cmd+Backspace)">✕</button>}
        </span>
      </div>
      <pre className="diff-body">
        {lines.map((line, i) => {
          let cls = "diff-line";
          if (line.startsWith("+")) cls += " added";
          else if (line.startsWith("-")) cls += " removed";
          else if (line.startsWith("@@")) cls += " hunk";
          return (
            <div key={i} className={cls}>
              <span className="diff-linenum">{i + 1}</span>
              <span className="diff-line-content">{line}</span>
            </div>
          );
        })}
      </pre>
    </div>
  );
};

export default DiffView;
