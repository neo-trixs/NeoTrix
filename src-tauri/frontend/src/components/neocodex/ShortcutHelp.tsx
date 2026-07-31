import React, { useEffect, useRef } from "react";
import styles from "./ShortcutHelp.module.css";

const SHORTCUTS: Array<{ keys: string; label: string }> = [
  { keys: "⌘N", label: "新建会话" },
  { keys: "⌘K", label: "命令面板" },
  { keys: "⌘B", label: "折叠/展开侧栏" },
  { keys: "⌘,", label: "设置" },
  { keys: "Ctrl+Tab", label: "切换会话" },
  { keys: "Esc", label: "返回对话" },
  { keys: "⌘/", label: "快捷键帮助" },
];

export function ShortcutHelp({ open, onClose }: { open: boolean; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (open) ref.current?.focus();
  }, [open]);

  if (!open) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div
        ref={ref}
        className={styles.modal}
        role="dialog"
        aria-label="快捷键"
        tabIndex={-1}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.key === "Escape" && onClose()}
      >
        <div className={styles.header}>
          <h3>快捷键</h3>
          <button className={styles.close} onClick={onClose} aria-label="关闭">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M4 4l8 8M12 4l-8 8" strokeLinecap="round"/>
            </svg>
          </button>
        </div>
        <div className={styles.list}>
          {SHORTCUTS.map((s) => (
            <div key={s.keys} className={styles.row}>
              <span className={styles.label}>{s.label}</span>
              <span className={styles.keys}>{s.keys}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default ShortcutHelp;
