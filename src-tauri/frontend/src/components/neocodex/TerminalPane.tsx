import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./TerminalPane.module.css";

interface TabMeta {
  id: string;
  name: string;
}

let tabSeq = 0;
const nextTabId = () => `term-${Date.now()}-${tabSeq++}`;

const DEFAULT_SCROLLBACK = 2000;

export function TerminalPane() {
  const [tabs, setTabs] = useState<TabMeta[]>([{ id: nextTabId(), name: "终端 1" }]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const scrollbackRef = useRef(DEFAULT_SCROLLBACK);

  useEffect(() => {
    if (!activeId && tabs.length > 0) setActiveId(tabs[0].id);
  }, [tabs, activeId]);

  const addTab = () => {
    const id = nextTabId();
    setTabs((prev) => [...prev, { id, name: `终端 ${prev.length + 1}` }]);
    setActiveId(id);
  };

  const closeTab = (id: string) => {
    setTabs((prev) => {
      const next = prev.filter((t) => t.id !== id);
      if (next.length === 0) {
        return [{ id: nextTabId(), name: "终端 1" }];
      }
      return next;
    });
    setActiveId((prevActive) => {
      if (prevActive !== id) return prevActive;
      const remaining = tabs.filter((t) => t.id !== id);
      if (remaining.length === 0) return null;
      const idx = tabs.findIndex((t) => t.id === id);
      return remaining[Math.min(idx, remaining.length - 1)].id;
    });
  };

  const renameTab = (id: string, name: string) => {
    setTabs((prev) => prev.map((t) => (t.id === id ? { ...t, name: name || t.name } : t)));
  };

  const setScrollback = (lines: number) => {
    scrollbackRef.current = Math.max(100, lines);
  };

  const active = tabs.find((t) => t.id === activeId) || tabs[0];

  return (
    <div className={styles.panel} data-testid="terminal-pane">
      <div className={styles.header}>
        <div className={styles.tabBar} role="tablist" aria-label="终端标签页">
          {tabs.map((t) => (
            <div
              key={t.id}
              role="tab"
              aria-selected={t.id === active?.id}
              className={`${styles.tab} ${t.id === active?.id ? styles.tabActive : ""}`}
              data-testid={`terminal-tab-${t.id}`}
              onClick={() => setActiveId(t.id)}
            >
              <input
                className={styles.tabName}
                value={t.name}
                onChange={(e) => renameTab(t.id, e.target.value)}
                onFocus={(e) => e.target.select()}
                title="双击重命名"
                spellCheck={false}
              />
              <button
                type="button"
                className={styles.tabClose}
                onClick={(e) => { e.stopPropagation(); closeTab(t.id); }}
                title="关闭标签"
                aria-label={`关闭 ${t.name}`}
              >
                ✕
              </button>
            </div>
          ))}
          <button type="button" className={styles.tabAdd} onClick={addTab} title="新建终端" data-testid="terminal-add" aria-label="新建终端">
            +
          </button>
        </div>
        <span className={styles.title}>终端</span>
      </div>
      <div className={styles.tabBody}>
        {tabs.map((t) => (
          <div key={t.id} className={styles.tabPane} hidden={t.id !== active?.id}>
            <TerminalTab
              active={t.id === active?.id}
              onExit={() => closeTab(t.id)}
              scrollback={scrollbackRef.current}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

function TerminalTab({ active, onExit, scrollback }: { active: boolean; onExit: () => void; scrollback: number }) {
  const [lines, setLines] = useState<string[]>(["$ "]);
  const [input, setInput] = useState("");
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const [hasBell, setHasBell] = useState(false);
  const outRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const bufRef = useRef("");
  const sessionRef = useRef<string | null>(null);
  const cleanupRef = useRef<Array<() => void>>([]);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const dimensionsRef = useRef({ cols: 100, rows: 24 });
  const activeRef = useRef(active);
  const scrollbackRef = useRef(scrollback);

  activeRef.current = active;
  scrollbackRef.current = scrollback;

  const append = useCallback((chunk: string) => {
    bufRef.current += chunk;
    const parts = bufRef.current.split("\n");
    bufRef.current = parts.pop() ?? "";
    setLines((prev) => {
      const next = [...prev];
      for (const p of parts) {
        if (p === "\r" || p === "") { next.push(""); continue; }
        const trimmed = p.replace(/\r/g, "");
        next.push(trimmed);
      }
      if (!activeRef.current) setHasBell(true);
      return next.slice(-scrollbackRef.current);
    });
  }, []);

  const clearBell = useCallback(() => {
    setHasBell(false);
  }, []);

  useEffect(() => {
    if (active) {
      inputRef.current?.focus();
      clearBell();
    }
  }, [active, clearBell]);

  const handleResize = useCallback(async () => {
    const sid = sessionRef.current;
    if (!sid) return;
    const outEl = outRef.current;
    if (!outEl) return;
    const fontSize = 12;
    const lineHeight = 1.45;
    const charWidth = fontSize * 0.6;
    const cols = Math.max(10, Math.floor(outEl.clientWidth / charWidth));
    const rows = Math.max(5, Math.floor(outEl.clientHeight / (fontSize * lineHeight)));
    if (cols !== dimensionsRef.current.cols || rows !== dimensionsRef.current.rows) {
      dimensionsRef.current = { cols, rows };
      try {
        await invoke("pty_resize", { sessionId: sid, cols, rows });
      } catch (e) {
        console.warn("pty_resize failed:", e);
      }
    }
  }, []);

  useEffect(() => {
    const outEl = outRef.current;
    if (outEl) {
      resizeObserverRef.current = new ResizeObserver(handleResize);
      resizeObserverRef.current.observe(outEl);
      handleResize();
    }
    return () => {
      resizeObserverRef.current?.disconnect();
      resizeObserverRef.current = null;
    };
  }, [handleResize]);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const sid = await invoke("pty_spawn", { cols: dimensionsRef.current.cols, rows: dimensionsRef.current.rows }) as string;
        if (cancelled) {
          invoke("pty_close", { sessionId: sid }).catch(() => {});
          return;
        }
        sessionRef.current = sid;
        const offOutput = await listen<string>(`pty-output-${sid}`, (ev) => {
          append(ev.payload);
          setReady(true);
        });
        const offExit = await listen<number>(`pty-exit-${sid}`, () => {
          setReady(false);
        });
        if (cancelled) {
          offOutput();
          offExit();
          invoke("pty_close", { sessionId: sid }).catch(() => {});
          return;
        }
        cleanupRef.current = [offOutput, offExit];
        setReady(true);
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    })();
    return () => {
      cancelled = true;
      cleanupRef.current.forEach((fn) => fn());
      cleanupRef.current = [];
      const sid = sessionRef.current;
      if (sid) invoke("pty_close", { sessionId: sid }).catch(() => {});
      sessionRef.current = null;
    };
  }, [append]);

  const send = async () => {
    const sid = sessionRef.current;
    if (!sid) return;
    const text = input;
    setInput("");
    if (text === "exit" || text === "exit\n") {
      await invoke("pty_close", { sessionId: sid }).catch(() => {});
      sessionRef.current = null;
      onExit();
      return;
    }
    await invoke("pty_write", { sessionId: sid, data: text + "\n" }).catch((e) => setError(String(e)));
    setLines((prev) => [...prev, `$ ${text}`]);
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      send();
    }
    if (e.key === "l" && e.ctrlKey) {
      e.preventDefault();
      setLines(["$ "]);
      bufRef.current = "";
    }
    if (e.key === "c" && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      copySelection();
    }
  };

  const copySelection = useCallback(() => {
    const selection = window.getSelection();
    if (selection && selection.toString()) {
      navigator.clipboard.writeText(selection.toString()).catch(() => {});
    }
  }, []);

  const onContextMenu = (e: React.MouseEvent<HTMLDivElement>) => {
    e.preventDefault();
    const selection = window.getSelection();
    if (selection && selection.toString()) {
      navigator.clipboard.writeText(selection.toString()).catch(() => {});
    }
  };

  return (
    <div className={styles.tabBody}>
      <div
        className={`${styles.output} ${hasBell && !active ? styles.hasBell : ""}`}
        ref={outRef}
        onContextMenu={onContextMenu}
        data-testid="terminal-output"
      >
        {lines.map((l, i) => (
          <div key={i} className={styles.line}>{l || "\u00a0"}</div>
        ))}
        {error && <div className={styles.error}>{error}</div>}
      </div>
      <div className={styles.inputRow}>
        <span className={styles.prompt}>$</span>
        <input
          ref={inputRef}
          className={styles.input}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder="运行命令 (Ctrl+L 清屏, Ctrl+Shift+C 复制, exit 关闭标签)"
          autoFocus={active}
          data-testid={active ? "terminal-input" : "terminal-input-hidden"}
        />
        <button type="button" className={styles.send} onClick={send} data-testid="terminal-send">⏎</button>
      </div>
    </div>
  );
}

export default TerminalPane;