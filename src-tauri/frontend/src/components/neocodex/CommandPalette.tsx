import React, { useEffect, useMemo, useRef, useState } from "react";
import styles from "./CommandPalette.module.css";

export interface PaletteItem {
  id: string;
  label: string;
  hint?: string;
  onSelect: () => void;
}

export function CommandPalette({ open, items, onClose }: { open: boolean; items: PaletteItem[]; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActive(0);
      inputRef.current?.focus();
    }
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter((i) => i.label.toLowerCase().includes(q));
  }, [items, query]);

  useEffect(() => {
    setActive(0);
  }, [filtered.length]);

  if (!open) return null;

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((a) => Math.min(a + 1, Math.max(filtered.length - 1, 0)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((a) => Math.max(a - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = filtered[active];
      if (item) {
        item.onSelect();
        onClose();
      }
    } else if (e.key === "Escape") {
      onClose();
    }
  };

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.palette} onClick={(e) => e.stopPropagation()} onKeyDown={handleKey}>
        <input
          ref={inputRef}
          className={styles.input}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索会话或执行命令…"
        />
        <div className={styles.list}>
          {filtered.length === 0 && <div className={styles.empty}>无匹配项</div>}
          {filtered.map((item, i) => (
            <button
              key={item.id}
              className={`${styles.item} ${i === active ? styles.itemActive : ""}`}
              onMouseEnter={() => setActive(i)}
              onClick={() => {
                item.onSelect();
                onClose();
              }}
            >
              <span className={styles.itemLabel}>{item.label}</span>
              {item.hint && <span className={styles.itemHint}>{item.hint}</span>}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

export default CommandPalette;
