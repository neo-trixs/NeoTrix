import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./DiffPane.module.css";

interface ChangedFile {
  status: string;
  path: string;
}

export function DiffPane() {
  const [blocks, setBlocks] = useState<Array<{ type: string; content: string; line_start: number }>>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [scope, setScope] = useState<"unstaged" | "staged" | "file" | "base">("unstaged");
  const [filePath, setFilePath] = useState("");
  const [baseBranch, setBaseBranch] = useState("main");
  const [commitMsg, setCommitMsg] = useState("");
  const [committing, setCommitting] = useState(false);
  const [changedFiles, setChangedFiles] = useState<{ staged: ChangedFile[]; unstaged: ChangedFile[]; untracked: ChangedFile[] }>({ staged: [], unstaged: [], untracked: [] });
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [reviewing, setReviewing] = useState(false);
  const [review, setReview] = useState<ReviewResult | null>(null);

  interface ReviewIssue {
    line: number;
    severity: string;
    category: string;
    message: string;
    suggestion?: string | null;
  }
  interface ReviewFileResult {
    path: string;
    additions: number;
    deletions: number;
    issues: ReviewIssue[];
  }
  interface ReviewResult {
    pr_title: string;
    total_files: number;
    total_issues: number;
    critical: number;
    warning: number;
    info: number;
    files: ReviewFileResult[];
    summary: string;
    score: number;
  }

  const loadFiles = useCallback(async () => {
    try {
      const res = await invoke<any>("cmd_diff_changed_files");
      if (res) {
        setChangedFiles({
          staged: res.staged || [],
          unstaged: res.unstaged || [],
          untracked: res.untracked || [],
        });
      }
    } catch (e) {
      console.error("Load changed files failed:", e);
    }
  }, []);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      let res: any[];
      if (scope === "file" && filePath) {
        res = await invoke("cmd_diff_file", { path: filePath });
      } else if (scope === "staged") {
        res = await invoke("cmd_diff_staged");
      } else if (scope === "base") {
        res = await invoke("cmd_diff_base", { base: baseBranch });
      } else if (scope === "file") {
        setBlocks([]);
        setLoading(false);
        return;
      } else {
        res = await invoke("cmd_diff_unstaged");
      }
      setBlocks(res || []);
    } catch (e) {
      setError(String(e));
      setBlocks([]);
    } finally {
      setLoading(false);
    }
  }, [scope, filePath, baseBranch]);

  // Auto-load on scope change for the git scopes. The file-path scope loads
  // exclusively via the explicit "查看"/refresh buttons / list clicks, avoiding
  // an IPC request per keystroke and double-loads from selectFile.
  useEffect(() => {
    if (scope === "file") return;
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scope]);

  // Keep the changed-file list fresh (initial + after stage/unstage/commit).
  useEffect(() => {
    loadFiles();
  }, [loadFiles]);

  const selectFile = async (path: string) => {
    setActiveFile(path);
    setFilePath(path);
    setScope("file");
    if (!path) return;
    setLoading(true);
    setError("");
    try {
      const res = await invoke<any[]>("cmd_diff_file", { path });
      setBlocks(res || []);
    } catch (e) {
      setError(String(e));
      setBlocks([]);
    } finally {
      setLoading(false);
    }
  };

  const stats = useCallback(() => {
    let added = 0;
    let removed = 0;
    for (const b of blocks) {
      if (b.type === "added") added++;
      if (b.type === "removed") removed++;
    }
    return { added, removed };
  }, [blocks]);

  const { added, removed } = stats();

  const handleStage = async () => {
    setError("");
    try {
      await invoke("cmd_diff_stage", { paths: null });
      await loadFiles();
      if (scope !== "staged") setScope("staged");
    } catch (e) {
      setError(String(e));
    }
  };

  const handleUnstage = async () => {
    setError("");
    try {
      await invoke("cmd_diff_unstage", { paths: null });
      await loadFiles();
      if (scope !== "unstaged") setScope("unstaged");
    } catch (e) {
      setError(String(e));
    }
  };

  // Per-file review (Claude Code Manual / Codex review parity): accept = stage
  // just this file, reject = discard just this file's working-tree changes.
  const handleStageFile = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setError("");
    try {
      await invoke("cmd_diff_stage", { paths: [path] });
      await loadFiles();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleRejectFile = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setError("");
    try {
      await invoke("cmd_diff_restore", { paths: [path] });
      await loadFiles();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleCommit = async () => {
    if (!commitMsg.trim()) return;
    setCommitting(true);
    setError("");
    try {
      await invoke("cmd_diff_commit", { message: commitMsg });
      setCommitMsg("");
      setBlocks([]);
      await loadFiles();
    } catch (e) {
      setError(String(e));
    } finally {
      setCommitting(false);
    }
  };

  const runReview = async () => {
    setReviewing(true);
    setError("");
    setReview(null);
    try {
      const res = await invoke<ReviewResult>("cmd_diff_review");
      setReview(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setReviewing(false);
    }
  };

  const severityClass = (s: string) =>
    s === "critical" ? styles.sevCritical : s === "warning" ? styles.sevWarning : styles.sevInfo;

  const statusLabel = (s: string) => {
    switch (s.trim()) {
      case "M": case "MM": return "已修改";
      case "A": case "AM": return "新增";
      case "D": return "已删除";
      case "R": return "重命名";
      case "C": return "复制";
      default: return "变更";
    }
  };

  const renderFileGroup = (title: string, files: ChangedFile[], bucket: "staged" | "unstaged" | "untracked") => {
    if (files.length === 0) return null;
    return (
      <div className={styles.fileGroup}>
        <div className={styles.fileGroupTitle}>{title} ({files.length})</div>
        {files.map((f) => (
          <div
            key={f.path}
            className={`${styles.fileItem} ${activeFile === f.path ? styles.fileItemActive : ""}`}
            onClick={() => selectFile(f.path)}
            title={f.path}
            data-testid={`diff-file-${f.path}`}
          >
            <span className={styles.fileStatus} data-status={f.status.trim()}>{statusLabel(f.status)}</span>
            <span className={styles.filePath}>{f.path}</span>
            <span className={styles.fileActions}>
              {bucket === "unstaged" && (
                <button
                  type="button"
                  className={styles.fileAccept}
                  onClick={(e) => handleStageFile(f.path, e)}
                  title="接受此文件改动 (stage)"
                  data-testid={`diff-accept-${f.path}`}
                >
                  ✓
                </button>
              )}
              {bucket !== "untracked" && bucket === "staged" && (
                <button
                  type="button"
                  className={styles.fileReject}
                  onClick={(e) => { e.stopPropagation(); handleUnstage(); }}
                  title="取消暂存此文件"
                  data-testid={`diff-unstage-${f.path}`}
                >
                  ↩
                </button>
              )}
              {(bucket === "unstaged" || bucket === "untracked") && (
                <button
                  type="button"
                  className={styles.fileReject}
                  onClick={(e) => handleRejectFile(f.path, e)}
                  title="拒绝此文件改动 (restore)"
                  data-testid={`diff-reject-${f.path}`}
                >
                  ✕
                </button>
              )}
            </span>
          </div>
        ))}
      </div>
    );
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span className={styles.title}>Diff</span>
        <div className={styles.scopes}>
          {(["unstaged", "staged", "base", "file"] as const).map((s) => (
            <button
              key={s}
              type="button"
              className={`${styles.scopeBtn} ${scope === s ? styles.scopeActive : ""}`}
              onClick={() => setScope(s)}
              data-testid={`diff-scope-${s}`}
            >
              {s === "unstaged" ? "未暂存" : s === "staged" ? "已暂存" : s === "base" ? "基线分支" : "文件"}
            </button>
          ))}
        </div>
        <button type="button" className={styles.refresh} onClick={() => { loadFiles(); load(); }} title="刷新" data-testid="diff-refresh">↻</button>
      </div>
      <div className={styles.split}>
        <div className={styles.fileList}>
          {renderFileGroup("已暂存", changedFiles.staged, "staged")}
          {renderFileGroup("未暂存", changedFiles.unstaged, "unstaged")}
          {renderFileGroup("未跟踪", changedFiles.untracked, "untracked")}
          {changedFiles.staged.length === 0 && changedFiles.unstaged.length === 0 && changedFiles.untracked.length === 0 && (
            <div className={styles.fileEmpty}>无改动文件</div>
          )}
        </div>
        <div className={styles.diffCol}>
          {scope === "file" && (
            <div className={styles.fileInputRow}>
              <input
                className={styles.fileInput}
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
                placeholder="文件路径（相对仓库根）"
              />
              <button type="button" className={styles.applyBtn} onClick={() => selectFile(filePath)}>查看</button>
            </div>
          )}
          {scope === "base" && (
            <div className={styles.fileInputRow}>
              <input
                className={styles.fileInput}
                value={baseBranch}
                onChange={(e) => setBaseBranch(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && load()}
                placeholder="基线分支（如 main / origin/main）"
                data-testid="diff-base-branch"
              />
              <button type="button" className={styles.applyBtn} onClick={() => load()} data-testid="diff-base-load">对比</button>
            </div>
          )}
          <div className={styles.stats}>
            <span className={styles.added}>+{added}</span>
            <span className={styles.removed}>-{removed}</span>
            {loading && <span className={styles.muted}>加载中…</span>}
            {error && <span className={styles.errorText}>{error}</span>}
          </div>
          <div className={styles.actions}>
            <button type="button" className={styles.actionBtn} onClick={handleStage} disabled={loading} data-testid="diff-stage-all" title="暂存全部改动">
              暂存
            </button>
            <button type="button" className={styles.actionBtn} onClick={handleUnstage} disabled={loading} data-testid="diff-unstage-all" title="取消暂存">
              取消暂存
            </button>
            <button type="button" className={styles.actionBtn} onClick={runReview} disabled={reviewing} data-testid="diff-review" title="对工作区变更运行静态代码审查">
              {reviewing ? "审查中…" : "AI 审查"}
            </button>
          </div>
          {review && (
            <div className={styles.reviewPanel} data-testid="diff-review-panel">
              <div className={styles.reviewHeader}>
                <span className={styles.reviewTitle}>审查结果</span>
                <span className={styles.reviewScore}>得分 {review.score}/100</span>
                <button type="button" className={styles.reviewClose} onClick={() => setReview(null)} title="关闭">✕</button>
              </div>
              <div className={styles.reviewSummary}>
                {review.summary}
                {review.total_issues === 0 && <span className={styles.reviewOk}> 未发现问题。</span>}
              </div>
              {review.files.map((f) => (
                <div key={f.path} className={styles.reviewFile}>
                  <div className={styles.reviewFilePath}>{f.path}</div>
                  {f.issues.length === 0 ? (
                    <div className={styles.reviewClean}>无问题</div>
                  ) : (
                    f.issues.map((iss, i) => (
                      <div key={i} className={`${styles.reviewIssue} ${severityClass(iss.severity)}`}>
                        <span className={styles.reviewSev}>{iss.severity}</span>
                        <span className={styles.reviewLine}>L{iss.line}</span>
                        <span className={styles.reviewCategory}>{iss.category}</span>
                        <span className={styles.reviewMsg}>{iss.message}</span>
                        {iss.suggestion && <div className={styles.reviewSuggest}>建议: {iss.suggestion}</div>}
                      </div>
                    ))
                  )}
                </div>
              ))}
            </div>
          )}
          <div className={styles.commitRow}>
            <input
              className={styles.commitInput}
              value={commitMsg}
              onChange={(e) => setCommitMsg(e.target.value)}
              placeholder="提交信息…"
              data-testid="diff-commit-msg"
            />
            <button
              type="button"
              className={styles.commitBtn}
              onClick={handleCommit}
              disabled={committing || !commitMsg.trim()}
              data-testid="diff-commit"
            >
              {committing ? "提交中…" : "提交"}
            </button>
          </div>
          <div className={styles.body}>
            {!loading && blocks.length === 0 && !error && (
              <div className={styles.empty}>无改动</div>
            )}
            {blocks.map((b, i) => (
              <div key={i} className={`${styles.block} ${styles[b.type] || styles.unchanged}`}>
                {b.content || "\u00a0"}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
