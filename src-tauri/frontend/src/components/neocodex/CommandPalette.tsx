import React, { useEffect, useMemo, useRef, useState } from "react";
import styles from "./CommandPalette.module.css";

export interface PaletteItem {
  id: string;
  label: string;
  hint?: string;
  onSelect: () => void;
}

function fuzzyScore(label: string, query: string): number {
  if (!query) return 0;
  const l = label.toLowerCase();
  const q = query.toLowerCase();
  if (l.startsWith(q)) return 100 - l.length / 100;
  let score = 0;
  let qi = 0;
  let last = -2;
  for (let i = 0; i < l.length && qi < q.length; i++) {
    if (l[i] === q[qi]) {
      score += i === last + 1 ? 3 : 1;
      last = i;
      qi++;
    }
  }
  if (qi < q.length) return -1;
  return score - l.length / 100;
}

export function CommandPalette({ open, items, onClose }: { open: boolean; items: PaletteItem[]; onClose: () => void }) {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const paletteRef = useRef<HTMLDivElement>(null);
  const lastFocusedRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    if (open) {
      setQuery("");
      setActive(0);
      lastFocusedRef.current = document.activeElement as HTMLElement | null;
      inputRef.current?.focus();
    } else if (lastFocusedRef.current) {
      lastFocusedRef.current.focus?.();
      lastFocusedRef.current = null;
    }
  }, [open]);

  // Trap Tab/Shift+Tab inside the palette so keyboard users can't walk into
  // the page behind the overlay (parity with Codex/Claude palettes).
  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Tab") return;
      const node = paletteRef.current;
      if (!node) return;
      const focusables = node.querySelectorAll<HTMLElement>('button, [href], input, [tabindex]:not([tabindex="-1"])');
      if (focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items
      .map((i) => ({ item: i, score: fuzzyScore(i.label, q) }))
      .filter((x) => x.score >= 0)
      .sort((a, b) => b.score - a.score)
      .map((x) => x.item);
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
    <div className={styles.overlay} onClick={onClose} data-testid="palette-overlay">
      <div
        ref={paletteRef}
        className={styles.palette}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKey}
        data-testid="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label="命令面板"
      >
        <input
          ref={inputRef}
          className={styles.input}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索会话或执行命令…"
          data-testid="palette-input"
        />
        <div className={styles.list} data-testid="palette-list">
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
              data-testid={`palette-item-${i}`}
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
