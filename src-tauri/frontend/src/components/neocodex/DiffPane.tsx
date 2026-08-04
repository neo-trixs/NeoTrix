import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./DiffPane.module.css";

interface ChangedFile {
  status: string;
  path: string;
}

interface FileDiff {
  path: string;
  blocks: Array<{ type: string; content: string; line_start: number }>;
  additions: number;
  deletions: number;
}

export function DiffPane() {
  const [blocks, setBlocks] = useState<Array<{ type: string; content: string; line_start: number }>>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [scope, setScope] = useState<"unstaged" | "staged" | "file" | "base" | "neocodex">("unstaged");
  const [filePath, setFilePath] = useState("");
  const [baseBranch, setBaseBranch] = useState("main");
  const [commitMsg, setCommitMsg] = useState("");
  const [committing, setCommitting] = useState(false);
  const [changedFiles, setChangedFiles] = useState<{ staged: ChangedFile[]; unstaged: ChangedFile[]; untracked: ChangedFile[] }>({ staged: [], unstaged: [], untracked: [] });
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const [reviewing, setReviewing] = useState(false);
  const [review, setReview] = useState<ReviewResult | null>(null);
  const [fileDiffs, setFileDiffs] = useState<Map<string, FileDiff>>(new Map());
  const [neocodexLoading, setNeocodexLoading] = useState(false);

  // Diff line comments (Claude Code Desktop parity): click a diff line to
  // comment; Enter adds a comment; ⌘Enter submits all comments to the agent
  // through the normal chat send pipeline.
  const [comments, setComments] = useState<Map<number, string>>(new Map());
  const [commentTarget, setCommentTarget] = useState<number | null>(null);
  const [draft, setDraft] = useState("");
  const commentInputRef = useRef<HTMLInputElement>(null);

  const openComment = (line: number, existing: string) => {
    setCommentTarget(line);
    setDraft(existing);
  };

  const saveComment = () => {
    if (commentTarget === null) return;
    const text = draft.trim();
    if (!text) return;
    setComments((prev) => {
      const next = new Map(prev);
      next.set(commentTarget, text);
      return next;
    });
    setCommentTarget(null);
    setDraft("");
  };

  const removeComment = (line: number) => {
    setComments((prev) => {
      const next = new Map(prev);
      next.delete(line);
      return next;
    });
    if (commentTarget === line) {
      setCommentTarget(null);
      setDraft("");
    }
  };

  const submitComments = () => {
    if (comments.size === 0) return;
    const file = activeFile || filePath || "(当前文件)";
    const body = [...comments.entries()]
      .sort((a, b) => a[0] - b[0])
      .map(([ln, text]) => `${file}:${ln} — ${text}`)
      .join("\n");
    const content = `针对以下代码变更的审阅意见，请据此修改代码：\n\n${body}`;
    window.dispatchEvent(new CustomEvent("neotrix:diff-submit-comments", { detail: { content } }));
    setComments(new Map());
    setCommentTarget(null);
    setDraft("");
  };

  useEffect(() => {
    setComments(new Map());
    setCommentTarget(null);
    setDraft("");
  }, [activeFile]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName;
      const inEditable = tag === "INPUT" || tag === "TEXTAREA" || target?.isContentEditable;
      if (inEditable) return;
      if (e.key === "Enter" && (e.metaKey || e.ctrlKey) && comments.size > 0) {
        e.preventDefault();
        submitComments();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [comments, activeFile, filePath]);

  const loadRef = useRef<() => Promise<void>>();
  const loadNeocodexDiffsRef = useRef<() => Promise<void>>();

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

  const loadNeocodexDiffs = useCallback(async () => {
    setNeocodexLoading(true);
    setError("");
    try {
      const res = await invoke<any>("neocodex_get_diff");
      if (res) {
        const diffsMap = new Map<string, FileDiff>();
        for (const [path, blocks] of Object.entries(res)) {
          const blockArray = blocks as Array<{ type: string; content: string; line_start: number }>;
          let additions = 0;
          let deletions = 0;
          for (const b of blockArray) {
            if (b.type === "added") additions++;
            if (b.type === "removed") deletions++;
          }
          diffsMap.set(path, { path, blocks: blockArray, additions, deletions });
        }
        setFileDiffs(diffsMap);
        // Also update changedFiles for the file list UI
        const changed: { staged: ChangedFile[]; unstaged: ChangedFile[]; untracked: ChangedFile[] } = { staged: [], unstaged: [], untracked: [] };
        for (const [path, diff] of diffsMap) {
          if (diff.blocks.some(b => b.type === "added" && diff.blocks[0] === b)) {
            changed.untracked.push({ status: "??", path });
          } else {
            changed.unstaged.push({ status: "M", path });
          }
        }
        setChangedFiles(changed);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setNeocodexLoading(false);
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
      } else if (scope === "neocodex") {
        await loadNeocodexDiffs();
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

  // Keep refs in sync with latest callbacks
  useEffect(() => {
    loadRef.current = load;
  }, [load]);

  useEffect(() => {
    loadNeocodexDiffsRef.current = loadNeocodexDiffs;
  }, [loadNeocodexDiffs]);

  // Auto-load on scope change only (not on baseBranch/filePath changes)
  useEffect(() => {
    if (scope === "file") return;
    if (scope === "neocodex") {
      loadNeocodexDiffsRef.current?.();
    } else {
      loadRef.current?.();
    }
  }, [scope]);

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

  const selectNeocodexFile = (path: string) => {
    setActiveFile(path);
    const diff = fileDiffs.get(path);
    if (diff) {
      setBlocks(diff.blocks);
    } else {
      setBlocks([]);
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

  const handleUnstageFile = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setError("");
    try {
      await invoke("cmd_diff_unstage", { paths: [path] });
      await loadFiles();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleStageFile = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setError("");
    try {
      if (scope === "neocodex") {
        await invoke("neocodex_apply_diff", { path, action: "accept" });
        await loadNeocodexDiffs();
      } else {
        await invoke("cmd_diff_stage", { paths: [path] });
        await loadFiles();
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const handleRejectFile = async (path: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setError("");
    try {
      if (scope === "neocodex") {
        await invoke("neocodex_apply_diff", { path, action: "reject" });
        await loadNeocodexDiffs();
      } else {
        await invoke("cmd_diff_restore", { paths: [path] });
        await loadFiles();
      }
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
        {files.map((f) => {
          const diff = fileDiffs.get(f.path);
          const fileAdditions = diff?.additions ?? 0;
          const fileDeletions = diff?.deletions ?? 0;
          return (
            <div
              key={f.path}
              className={`${styles.fileItem} ${activeFile === f.path ? styles.fileItemActive : ""}`}
              onClick={() => scope === "neocodex" ? selectNeocodexFile(f.path) : selectFile(f.path)}
              title={f.path}
              data-testid={`diff-file-${f.path}`}
            >
              <span className={styles.fileStatus} data-status={f.status.trim()}>{statusLabel(f.status)}</span>
              <span className={styles.filePath}>{f.path}</span>
              {(fileAdditions > 0 || fileDeletions > 0) && (
                <span className={styles.fileStats}>
                  <span className={styles.added}>+{fileAdditions}</span>
                  <span className={styles.removed}>-{fileDeletions}</span>
                </span>
              )}
              <span className={styles.fileActions}>
                {bucket === "unstaged" || bucket === "untracked" ? (
                  <button
                    type="button"
                    className={styles.fileAccept}
                    onClick={(e) => handleStageFile(f.path, e)}
                    title={bucket === "untracked" ? "接受此新文件 (stage add)" : "接受此文件改动 (stage)"}
                    data-testid={`diff-accept-${f.path}`}
                  >
                    ✓
                  </button>
                ) : null}
                {bucket !== "untracked" && bucket === "staged" && (
                  <button
                    type="button"
                    className={styles.fileReject}
                    onClick={(e) => { handleUnstageFile(f.path, e); }}
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
          );
        })}
      </div>
    );
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span className={styles.title}>Diff</span>
        <div className={styles.scopes}>
          {(["unstaged", "staged", "base", "file", "neocodex"] as const).map((s) => (
            <button
              key={s}
              type="button"
              className={`${styles.scopeBtn} ${scope === s ? styles.scopeActive : ""}`}
              onClick={() => setScope(s)}
              data-testid={`diff-scope-${s}`}
            >
              {s === "unstaged" ? "未暂存" : s === "staged" ? "已暂存" : s === "base" ? "基线分支" : s === "file" ? "文件" : "会话"}
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
          {(scope === "file" || scope === "neocodex") && (
            <div className={styles.fileInputRow}>
              <input
                className={styles.fileInput}
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
                placeholder={scope === "neocodex" ? "当前会话文件（点击列表选择）" : "文件路径（相对仓库根）"}
                disabled={scope === "neocodex"}
              />
              {scope === "file" && <button type="button" className={styles.applyBtn} onClick={() => selectFile(filePath)}>查看</button>}
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
            {(loading || neocodexLoading) && <span className={styles.muted}>加载中…</span>}
            {error && <span className={styles.errorText}>{error}</span>}
          </div>
          {comments.size > 0 && (
            <div className={styles.commentBar} data-testid="diff-comment-bar">
              <span className={styles.commentCount}>{comments.size} 条评论</span>
              <button
                type="button"
                className={styles.commentSubmitBtn}
                onClick={submitComments}
                data-testid="diff-comment-submit"
                title="提交所有评论给 Agent 处理"
              >
                提交给 Agent（⌘Enter）
              </button>
            </div>
          )}
          <div className={styles.actions}>
            <button type="button" className={styles.actionBtn} onClick={handleStage} disabled={loading || neocodexLoading} data-testid="diff-stage-all" title="暂存全部改动">
              暂存
            </button>
            <button type="button" className={styles.actionBtn} onClick={handleUnstage} disabled={loading || neocodexLoading} data-testid="diff-unstage-all" title="取消暂存">
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
            {!loading && !neocodexLoading && blocks.length === 0 && !error && (
              <div className={styles.empty}>无改动</div>
            )}
            {blocks.map((b, i) => {
              const line = b.line_start ?? 0;
              const comment = comments.get(line);
              const isEditing = commentTarget === line;
              return (
                <React.Fragment key={i}>
                  <div
                    className={`${styles.block} ${styles[b.type] || styles.unchanged} ${comment ? styles.commented : ""}`}
                    onClick={() => openComment(line, comment || "")}
                    data-line={line}
                    data-testid={`diff-line-${line}`}
                  >
                    <span className={styles.lineNo}>{line}</span>
                    <span className={styles.lineContent}>{b.content || "\u00a0"}</span>
                    {comment && <span className={styles.commentBadge} data-testid={`diff-comment-badge-${line}`}>💬</span>}
                  </div>
                  {isEditing && (
                    <div className={styles.commentBox} data-testid={`diff-comment-box-${line}`}>
                      <input
                        autoFocus
                        ref={commentInputRef}
                        className={styles.commentInput}
                        value={draft}
                        onChange={(e) => setDraft(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" && !e.shiftKey) {
                            e.preventDefault();
                            saveComment();
                          }
                          if (e.key === "Escape") {
                            setCommentTarget(null);
                            setDraft("");
                          }
                        }}
                        placeholder="针对此行写评论…（Enter 添加，⌘Enter 提交全部）"
                        data-testid="diff-comment-input"
                      />
                      <div className={styles.commentActions}>
                        <button type="button" className={styles.commentSaveBtn} onClick={saveComment} disabled={!draft.trim()} data-testid="diff-comment-save">
                          添加评论
                        </button>
                        {comment && (
                          <button type="button" className={styles.commentCancelBtn} onClick={() => removeComment(line)} data-testid="diff-comment-delete">
                            删除
                          </button>
                        )}
                        <button type="button" className={styles.commentCancelBtn} onClick={() => { setCommentTarget(null); setDraft(""); }}>
                          取消
                        </button>
                      </div>
                    </div>
                  )}
                </React.Fragment>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}