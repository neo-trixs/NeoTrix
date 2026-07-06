import React, { useEffect, useRef } from "react";
import { useStore } from "../stores";
import { useNavigate } from "react-router-dom";
import styles from "./UserPopover.module.css";

const UserPopover: React.FC = () => {
  const navigate = useNavigate();
  const popoverOpen = useStore((s) => s.userPopoverOpen);
  const displayName = useStore((s) => s.userDisplayName);
  const setPopoverOpen = useStore((s) => s.setUserPopoverOpen);
  const setSettings = useStore((s) => s.setSettings);
  const settings = useStore((s) => s.settings);
  const setShowShortcuts = useStore((s) => s.setShowShortcuts);
  const setSessions = useStore((s) => s.setSessions);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!popoverOpen) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setPopoverOpen(false);
      }
    };
    const escHandler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setPopoverOpen(false);
    };
    document.addEventListener("mousedown", handler);
    document.addEventListener("keydown", escHandler);
    return () => {
      document.removeEventListener("mousedown", handler);
      document.removeEventListener("keydown", escHandler);
    };
  }, [popoverOpen, setPopoverOpen]);

  const cycleTheme = () => {
    const order: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
    const idx = order.indexOf(settings.theme);
    setSettings({ ...settings, theme: order[(idx + 1) % order.length] });
    setPopoverOpen(false);
  };

  const handleLogout = () => {
    setPopoverOpen(false);
    setSessions([{ id: "default", name: "默认会话", messages: [] }]);
  };

  if (!popoverOpen) return null;

  const avatarInitial = displayName.charAt(0).toUpperCase() || "N";

  return (
    <div className={styles.popover} ref={ref} role="menu">
      <div className={styles.header}>
        <div className={styles.avatar}>{avatarInitial}</div>
        <div className={styles.name}>{displayName || "Neo"}</div>
      </div>
      <div className={styles.divider} />
      <button className={styles.item} role="menuitem" onClick={() => { setPopoverOpen(false); navigate("/settings"); }}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M7 9a2 2 0 100-4 2 2 0 000 4z" stroke="currentColor" strokeWidth="1.2"/><path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.5 2.5l1 1M10.5 10.5l1 1M2.5 11.5l1-1M10.5 3.5l1-1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/></svg>
        <span>设置</span>
      </button>
      <button className={styles.item} role="menuitem" onClick={cycleTheme}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><circle cx="7" cy="7" r="2.5" stroke="currentColor" strokeWidth="1.2"/><path d="M7 1v1M7 12v1M1 7h1M12 7h1M2.5 2.5l.5.5M11 11l.5.5M2.5 11.5l.5-.5M11 3l.5-.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/></svg>
        <span>主题切换 · {settings.theme === "light" ? "浅色" : settings.theme === "dark" ? "深色" : "跟随系统"}</span>
      </button>
      <button className={styles.item} role="menuitem" onClick={() => { setPopoverOpen(false); setShowShortcuts(true); }}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><rect x="1.5" y="2.5" width="11" height="9" rx="1.5" stroke="currentColor" strokeWidth="1.2"/><path d="M5 5l2 2-2 2M9 5L7 7l2 2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/></svg>
        <span>帮助</span>
      </button>
      <div className={styles.divider} />
      <button className={`${styles.item} ${styles.logout}`} role="menuitem" onClick={handleLogout}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M5 12.5H2.5a1 1 0 01-1-1v-9a1 1 0 011-1H5M9.5 10L12 7l-2.5-3M12 7H5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/></svg>
        <span>退出登录</span>
      </button>
    </div>
  );
};

export default UserPopover;
