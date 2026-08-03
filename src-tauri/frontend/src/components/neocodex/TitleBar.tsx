import React from "react";
import styles from "./TitleBar.module.css";

export interface TitleBarProps {
  currentProject: string | null;
  activeSessionName?: string;
  sessionProject?: string;
  renaming: boolean;
  titleDraft: string;
  onTitleDraftChange: (v: string) => void;
  onTitleCommit: () => void;
  onTitleCancel: () => void;
  onTitleStartRename: () => void;
  gitStatus?: { branch: string; dirty: boolean } | null;
  usagePct: number | null;
  themeLabel: string;
  focusMode: boolean;
  onToggleTheme: () => void;
  onOpenSettings: () => void;
  onToggleFocus: () => void;
  onOpenUsage: () => void;
  onToggleSidebar: () => void;
  onWorkspaceClick: () => void;
  onSearchClick: () => void;
  onNewSession: () => void;
}

export function TitleBar({
  currentProject,
  activeSessionName,
  sessionProject,
  renaming,
  titleDraft,
  onTitleDraftChange,
  onTitleCommit,
  onTitleCancel,
  onTitleStartRename,
  gitStatus,
  usagePct,
  themeLabel,
  focusMode,
  onToggleTheme,
  onOpenSettings,
  onToggleFocus,
  onOpenUsage,
  onToggleSidebar,
  onWorkspaceClick,
  onSearchClick,
  onNewSession,
}: TitleBarProps) {
  return (
    <header className={styles.titlebar} data-testid="titlebar">
      <div className={styles.left}>
        <button className={styles.iconBtn} onClick={onToggleSidebar} title="收起/展开侧栏 (⌘B)" aria-label="切换侧栏">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <rect x="1.5" y="2" width="13" height="12" rx="1.5" />
            <path d="M6 2v12" />
          </svg>
        </button>
        <button className={styles.workspaceSwitcher} onClick={onWorkspaceClick} title="切换工作区" data-testid="titlebar-workspace">
          <svg width="13" height="13" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <rect x="1.5" y="2.5" width="11" height="9" rx="1.5" />
            <path d="M5 5h4M5 7.5h4" />
          </svg>
          <span className={styles.workspaceName}>{currentProject ? currentProject.split(/[\\/]/).filter(Boolean).pop() : "本地"}</span>
          <span className={styles.chevron}>▾</span>
        </button>
      </div>

      <div className={styles.center}>
        {renaming ? (
          <input
            className={styles.titleInput}
            value={titleDraft}
            onChange={(e) => onTitleDraftChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") onTitleCommit();
              if (e.key === "Escape") onTitleCancel();
            }}
            onBlur={onTitleCommit}
            autoFocus
            data-testid="session-title-input"
          />
        ) : (
          <button
            className={styles.sessionTitle}
            onClick={onTitleStartRename}
            title="点击重命名会话"
            data-testid="session-title"
          >
            {activeSessionName || "未命名会话"}
          </button>
        )}
        {sessionProject && (
          <span className={styles.projectChip} title={sessionProject}>
            {sessionProject.split(/[\\/]/).filter(Boolean).pop()}
          </span>
        )}
        {gitStatus && (
          <span className={`${styles.branchChip} ${gitStatus.dirty ? styles.branchChipDirty : ""}`} title={gitStatus.dirty ? "有未提交改动" : "工作区干净"}>
            {gitStatus.branch}
          </span>
        )}
      </div>

      <div className={styles.right}>
        <button className={styles.iconBtn} onClick={onSearchClick} title="全局搜索 (⌘P)" aria-label="全局搜索">
          <svg width="15" height="15" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="6" cy="6" r="3.5" />
            <path d="M9 9l3 3" strokeLinecap="round" />
          </svg>
        </button>
        <button className={styles.iconBtn} onClick={onNewSession} title="新建会话 (⌘N)" aria-label="新建会话">
          <svg width="15" height="15" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <path d="M7 2v10M2 7h10" />
          </svg>
        </button>
        <button className={styles.usagePill} onClick={onOpenUsage} title="上下文用量" data-testid="titlebar-usage">
          <svg width="11" height="11" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="7" cy="7" r="5" />
            <path d="M7 3v4l3 3" strokeLinecap="round" />
          </svg>
          {usagePct !== null ? `${Math.round(usagePct * 100)}%` : "—"}
        </button>
        <button className={styles.iconBtn} onClick={onToggleTheme} title={themeLabel} aria-label="切换主题">
          <svg width="15" height="15" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="7" cy="7" r="2.6" />
            <path d="M7 1v2M7 11v2M1 7h2M11 7h2M2.9 2.9l1.4 1.4M9.7 9.7l1.4 1.4M2.9 11.1l1.4-1.4M9.7 4.3l1.4-1.4" strokeLinecap="round" />
          </svg>
        </button>
        <button className={styles.iconBtn} onClick={onToggleFocus} title={focusMode ? "退出专注" : "专注模式"} aria-label={focusMode ? "退出专注" : "专注模式"}>
          <svg width="15" height="15" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="7" cy="7" r="3" />
            <path d="M7 1v2M7 11v2M1 7h2M11 7h2" strokeLinecap="round" />
          </svg>
        </button>
        <button className={styles.iconBtn} onClick={onOpenSettings} title="设置 (⌘,)" aria-label="打开设置">
          <svg width="15" height="15" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="7" cy="7" r="2.2" />
            <path d="M7 1.5v1.8M7 10.7v1.8M1.5 7h1.8M10.7 7h1.8M3.1 3.1l1.3 1.3M9.6 9.6l1.3 1.3M3.1 10.9l1.3-1.3M9.6 4.4l1.3-1.3" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </header>
  );
}
