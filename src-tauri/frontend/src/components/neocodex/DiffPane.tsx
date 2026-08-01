import React, { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import styles from "./DiffPane.module.css";

export function DiffPane({ onOpenFile }: { onOpenFile?: (path: string) => void }) {
  const [blocks, setBlocks] = useState<Array<{ type: string; content: string; line_start: number }>>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [scope, setScope] = useState<"unstaged" | "staged" | "file">("unstaged");
  const [filePath, setFilePath] = useState("");
  const [commitMsg, setCommitMsg] = useState("");
  const [committing, setCommitting] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      let res: any[];
      if (scope === "file" && filePath) {
        res = await invoke("cmd_diff_file", { path: filePath });
      } else if (scope === "staged") {
        res = await invoke("cmd_diff_staged");
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
  }, [scope, filePath]);

  useEffect(() => {
    load();
  }, [load]);

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
    setLoading(true);
    setError("");
    try {
      await invoke("cmd_diff_stage", { paths: null });
      if (scope !== "staged") setScope("staged");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleUnstage = async () => {
    setLoading(true);
    setError("");
    try {
      await invoke("cmd_diff_unstage", { paths: null });
      if (scope !== "unstaged") setScope("unstaged");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
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
    } catch (e) {
      setError(String(e));
    } finally {
      setCommitting(false);
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <span className={styles.title}>Diff</span>
        <div className={styles.scopes}>
          {(["unstaged", "staged", "file"] as const).map((s) => (
            <button
              key={s}
              type="button"
              className={`${styles.scopeBtn} ${scope === s ? styles.scopeActive : ""}`}
              onClick={() => setScope(s)}
              data-testid={`diff-scope-${s}`}
            >
              {s === "unstaged" ? "未暂存" : s === "staged" ? "已暂存" : "文件"}
            </button>
          ))}
        </div>
        <button type="button" className={styles.refresh} onClick={load} title="刷新" data-testid="diff-refresh">↻</button>
      </div>
      {scope === "file" && (
        <div className={styles.fileInputRow}>
          <input
            className={styles.fileInput}
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            placeholder="文件路径（相对仓库根）"
          />
          <button type="button" className={styles.applyBtn} onClick={load}>查看</button>
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
      </div>
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
  );
}
