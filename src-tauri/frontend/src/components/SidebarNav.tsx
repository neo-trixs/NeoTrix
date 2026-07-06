import React, { useState, useRef, useEffect } from "react";
import styles from "./SidebarNav.module.css";

export interface SidebarItemData {
  id: string;
  icon: string;
  label: string;
  badge?: number;
  badgeHighlight?: boolean;
}

interface SidebarNavProps {
  items: SidebarItemData[];
  selected: string;
  onSelect: (id: string) => void;
  searchText?: string;
  onSearchChange?: (text: string) => void;
  footer?: React.ReactNode;
  /** Optional: if provided, renders a search field in the sidebar */
  showSearch?: boolean;
}

const SidebarNav: React.FC<SidebarNavProps> = ({
  items, selected, onSelect, searchText = "",
  onSearchChange, footer, showSearch = false,
}) => {
  const [collapsed, setCollapsed] = useState(false);
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = listRef.current?.querySelector(`[data-item-id="${selected}"]`);
    el?.scrollIntoView({ block: "center", behavior: "smooth" });
  }, [selected]);

  const sidebarWidth = collapsed ? 64 : 220;

  return (
    <div
      className={styles.root}
      style={{ width: sidebarWidth, minWidth: sidebarWidth }}
    >
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.logoArea}>
          <div className={styles.logoDot} />
          {!collapsed && <span className={styles.logoText}>NeoTrix</span>}
        </div>
        <button
          className={styles.collapseBtn}
          onClick={() => setCollapsed(!collapsed)}
          title={collapsed ? "Expand Sidebar" : "Collapse Sidebar"}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
            <rect x="1.5" y="2.5" width="13" height="11" rx="2" />
            <path d={collapsed ? "M6 5l3 3-3 3" : "M10 5L7 8l3 3"} />
          </svg>
        </button>
      </div>

      {/* Search */}
      {!collapsed && showSearch && (
        <div className={styles.searchField}>
          <svg className={styles.searchIcon} width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
            <circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            className={styles.searchInput}
            placeholder="Search Settings"
            value={searchText}
            onChange={(e) => onSearchChange?.(e.target.value)}
          />
          {searchText && (
            <button className={styles.searchClear} onClick={() => onSearchChange?.("")}>
              <svg width="11" height="11" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="12" r="10"/><path d="M8 8l8 8M16 8l-8 8" stroke="#fff" strokeWidth="2" strokeLinecap="round" fill="none"/></svg>
            </button>
          )}
        </div>
      )}

      {/* Items */}
      <div className={styles.items} ref={listRef}>
        {items.map((item) => {
          const active = selected === item.id;
          return (
            <button
              key={item.id}
              data-item-id={item.id}
              className={`${styles.item}${active ? ` ${styles.active}` : ""}`}
              onClick={() => onSelect(item.id)}
              title={collapsed ? item.label : undefined}
            >
              {collapsed ? (
                <div className={styles.itemCollapsed}>
                  <span className={styles.itemIcon} dangerouslySetInnerHTML={{ __html: item.icon }} />
                  {item.badge !== undefined && item.badge > 0 && (
                    <span className={`${styles.badgeDot}${item.badgeHighlight ? ` ${styles.highlight}` : ""}`} />
                  )}
                </div>
              ) : (
                <>
                  <span className={styles.itemIcon} dangerouslySetInnerHTML={{ __html: item.icon }} />
                  <span className={styles.itemLabel}>{item.label}</span>
                  {item.badge !== undefined && item.badge > 0 && (
                    <span className={`${styles.itemBadge}${item.badgeHighlight ? ` ${styles.highlight}` : ""}`}>
                      {item.badge > 99 ? "99+" : item.badge}
                    </span>
                  )}
                </>
              )}
            </button>
          );
        })}
      </div>

      {/* Footer */}
      {!collapsed && footer && (
        <div className={styles.footer}>{footer}</div>
      )}
    </div>
  );
};

export default SidebarNav;
