import React, { useEffect } from "react";
import { SettingsView } from "../components/neocodex";
import styles from "./SettingsDrawer.module.css";

export function SettingsDrawer({ open, onClose }: { open: boolean; onClose: () => void }) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div className={styles.backdrop} onClick={onClose} data-testid="settings-drawer-backdrop">
      <aside
        className={styles.drawer}
        role="dialog"
        aria-modal="true"
        aria-label="Settings"
        data-testid="settings-drawer"
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.header}>
          <span className={styles.title}>设置</span>
          <button className={styles.close} onClick={onClose} aria-label="关闭设置" data-testid="settings-drawer-close">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>
        <SettingsView />
      </aside>
    </div>
  );
}
