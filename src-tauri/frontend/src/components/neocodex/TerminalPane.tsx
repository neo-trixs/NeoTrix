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

export function TerminalPane() {
  const [tabs, setTabs] = useState<TabMeta[]>([{ id: nextTabId(), name: "终端 1" }]);
  const [activeId, setActiveId] = useState<string | null>(null);

  // First tab becomes active once rendered.
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
      const idx = prev.findIndex((t) => t.id === id);
      const next = prev.filter((t) => t.id !== id);
      if (next.length === 0) {
        // Always keep at least one tab alive; spawn a fresh one.
        setActiveId(null);
        return [{ id: nextTabId(), name: "终端 1" }];
      }
      if (activeId === id) {
        const neighbor = next[Math.min(idx, next.length - 1)];
        setActiveId(neighbor.id);
      }
      return next;
    });
  };

  const renameTab = (id: string, name: string) => {
    setTabs((prev) => prev.map((t) => (t.id === id ? { ...t, name: name || t.name } : t)));
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
            <TerminalTab active={t.id === active?.id} onExit={() => closeTab(t.id)} />
          </div>
        ))}
      </div>
    </div>
  );
}

function TerminalTab({ active, onExit }: { active: boolean; onExit: () => void }) {
  const [lines, setLines] = useState<string[]>(["$ "]);
  const [input, setInput] = useState("");
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const outRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const bufRef = useRef("");
  const sessionRef = useRef<string | null>(null);
  const cleanupRef = useRef<Array<() => void>>([]);

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
      return next.slice(-500);
    });
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const sid = await invoke("pty_spawn", { cols: 100, rows: 24 }) as string;
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

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
    if (e.key === "Enter") send();
    if (e.key === "l" && e.ctrlKey) {
      e.preventDefault();
      setLines(["$ "]);
      bufRef.current = "";
    }
  };

  useEffect(() => {
    if (active) inputRef.current?.focus();
  }, [active]);

  return (
    <div className={styles.tabBody}>
      <div className={styles.output} ref={outRef}>
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
          placeholder="运行命令 (Ctrl+L 清屏, exit 关闭标签)"
          autoFocus={active}
          data-testid={active ? "terminal-input" : "terminal-input-hidden"}
        />
        <button type="button" className={styles.send} onClick={send} data-testid="terminal-send">⏎</button>
      </div>
    </div>
  );
}

export default TerminalPane;
