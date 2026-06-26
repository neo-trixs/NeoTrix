import React, { useState, useEffect, useCallback } from "react";
import { useStore } from "../store";
import KnowledgePanel from "./KnowledgePanel";
import FileTree from "./FileTree";
import DiffView from "./DiffView";
import * as api from "../lib/api";

interface FileDiffInfo {
  file: string;
  added: number;
  removed: number;
  diff: string;
}

type TabId = "review" | "diff" | "context" | "knowledge" | "files";

const TABS: { id: TabId; label: string }[] = [
  { id: "review", label: "Review" },
  { id: "diff", label: "Diff" },
  { id: "context", label: "Context" },
  { id: "knowledge", label: "Knowledge" },
  { id: "files", label: "Files" },
];

const RightPanel: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabId>("context");
  const projectPath = useStore((s) => s.projectPath);
  const setStatusText = useStore((s) => s.setStatusText);
  const [diffs, setDiffs] = useState<{ staged: FileDiffInfo[]; unstaged: FileDiffInfo[] }>({ staged: [], unstaged: [] });
  const [diffLoading, setDiffLoading] = useState(false);
  const [diffError, setDiffError] = useState<string | null>(null);
  const [selectedDiffFile, setSelectedDiffFile] = useState<string | null>(null);

  const fetchDiffs = useCallback(async () => {
    setDiffLoading(true);
    setDiffError(null);
    try {
      const [staged, unstaged] = await Promise.all([
        api.getDiffStaged(),
        api.getDiffUnstaged(),
      ]);
      const sd = staged as unknown as FileDiffInfo[];
      const usd = unstaged as unknown as FileDiffInfo[];
      setDiffs({ staged: sd, unstaged: usd });
      setSelectedDiffFile((prev) => {
        const all = [...sd, ...usd];
        if (all.length === 0) return null;
        if (prev && all.some((f) => f.file === prev)) return prev;
        return all[0].file;
      });
    } catch (e) {
      setDiffError(String(e));
    } finally {
      setDiffLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDiffs();
  }, [fetchDiffs]);

  const allFiles = [...diffs.staged, ...diffs.unstaged];
  const totalAdded = allFiles.reduce((s, f) => s + f.added, 0);
  const totalRemoved = allFiles.reduce((s, f) => s + f.removed, 0);
  const hasDiffs = allFiles.length > 0;
  const selectedDiff = allFiles.find((f) => f.file === selectedDiffFile);

  const renderContent = () => {
    switch (activeTab) {
      case "review":
        if (diffLoading) {
          return (
            <div className="right-panel-content">
              <div className="review-empty">
                <span className="text-sm text-muted">Loading diffs...</span>
              </div>
            </div>
          );
        }
        if (diffError) {
          return (
            <div className="right-panel-content">
              <div className="review-empty">
                <span className="text-sm text-muted" style={{ color: "var(--color-error)" }}>Failed to load diffs</span>
                <span className="text-xs text-muted">{diffError}</span>
              </div>
            </div>
          );
        }
        if (!hasDiffs) {
          return (
            <div className="right-panel-content">
              <div className="review-empty">
                <svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.3">
                  <path d="M16 4v24M4 16h24" strokeLinecap="round" />
                  <circle cx="16" cy="16" r="12" />
                </svg>
                <span>No changes to review</span>
                <span className="text-sm text-muted">Changes will appear here when the agent proposes edits</span>
              </div>
            </div>
          );
        }
        return (
          <div className="right-panel-content">
            <div className="diff-summary-bar">
              {allFiles.length} file{allFiles.length !== 1 ? "s" : ""} changed, +{totalAdded} additions, -{totalRemoved} deletions
            </div>
            {diffs.staged.length > 0 && (
              <div className="diff-group">
                <div className="diff-group-title">Staged ({diffs.staged.length})</div>
                {diffs.staged.map((f) => (
                  <DiffView key={f.file} file={f.file} added={f.added} removed={f.removed} diff={f.diff} />
                ))}
              </div>
            )}
            {diffs.unstaged.length > 0 && (
              <div className="diff-group">
                <div className="diff-group-title">Unstaged ({diffs.unstaged.length})</div>
                {diffs.unstaged.map((f) => (
                  <DiffView key={f.file} file={f.file} added={f.added} removed={f.removed} diff={f.diff} />
                ))}
              </div>
            )}
          </div>
        );

      case "diff":
        if (diffLoading) {
          return (
            <div className="right-panel-content">
              <div className="review-empty">
                <span className="text-sm text-muted">Loading diffs...</span>
              </div>
            </div>
          );
        }
        if (!hasDiffs) {
          return (
            <div className="right-panel-content">
              <div className="review-empty">
                <svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1.5" opacity="0.3">
                  <path d="M8 16h16M16 8v16" strokeLinecap="round" />
                </svg>
                <span>No diffs available</span>
                <span className="text-sm text-muted">File diffs will appear here after agent edits</span>
              </div>
            </div>
          );
        }
        return (
          <div className="right-panel-content">
            <div className="diff-filepicker">
              <select
                value={selectedDiffFile ?? ""}
                onChange={(e) => setSelectedDiffFile(e.target.value)}
                className="diff-select"
              >
                {allFiles.map((f) => (
                  <option key={f.file} value={f.file}>
                    {f.file} (+{f.added}/-{f.removed})
                  </option>
                ))}
              </select>
            </div>
            {selectedDiff && (
              <DiffView
                file={selectedDiff.file}
                added={selectedDiff.added}
                removed={selectedDiff.removed}
                diff={selectedDiff.diff}
              />
            )}
          </div>
        );

      case "context":
        return <ContextView />;

      case "knowledge":
        return (
          <div className="right-panel-content">
            <KnowledgePanel />
          </div>
        );

      case "files":
        return (
          <div className="right-panel-content">
            {projectPath ? (
              <FileTree
                rootPath={projectPath}
                onClose={() => setActiveTab("context")}
                onStatusChange={setStatusText}
              />
            ) : (
              <div className="review-empty">
                <span className="text-sm text-muted">No project open</span>
                <span className="text-xs text-muted">Open a project folder to browse files</span>
              </div>
            )}
          </div>
        );
    }
  };

  return (
    <div className="app-right-panel glass-strong">
      <div className="right-panel-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={`right-panel-tab ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
        <button
          className="diff-refresh-btn"
          onClick={fetchDiffs}
          title="Refresh diffs"
          disabled={diffLoading}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="23 4 23 10 17 10" />
            <polyline points="1 20 1 14 7 14" />
            <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15" />
          </svg>
        </button>
      </div>
      {renderContent()}
    </div>
  );
};

const ContextView: React.FC = () => {
  const contextChips = useStore((s) => s.contextChips);

  return (
    <div className="right-panel-content">
      <div className="context-section">
        <div className="context-section-title">Active References</div>
        {contextChips.length === 0 ? (
          <div className="review-empty" style={{ padding: "16px 8px" }}>
            <span className="text-sm text-muted">No context references</span>
            <span className="text-xs text-muted">Use @ in chat to reference files</span>
          </div>
        ) : (
          contextChips.map((chip) => (
            <div key={chip.id} className="context-entry">
              <div className="context-entry-title">{chip.label}</div>
              <div className="context-entry-meta">{chip.type}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default RightPanel;
