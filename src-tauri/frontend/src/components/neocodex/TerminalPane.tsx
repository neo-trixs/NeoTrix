import React, { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./TerminalPane.module.css";

export function TerminalPane() {
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [lines, setLines] = useState<string[]>(["$ "]);
  const [input, setInput] = useState("");
  const [ready, setReady] = useState(false);
  const [error, setError] = useState("");
  const outRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const bufRef = useRef("");

  useEffect(() => {
    if (outRef.current) outRef.current.scrollTop = outRef.current.scrollHeight;
  }, [lines]);

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
    let unlistenOutput: (() => void) | undefined;
    let unlistenExit: (() => void) | undefined;
    (async () => {
      try {
        const sid = await invoke("pty_spawn", { cols: 100, rows: 24 }) as string;
        setSessionId(sid);
        unlistenOutput = await listen<string>(`pty-output-${sid}`, (ev) => {
          append(ev.payload);
          setReady(true);
        });
        unlistenExit = await listen<number>(`pty-exit-${sid}`, () => {
          setReady(false);
        });
        setReady(true);
      } catch (e) {
        setError(String(e));
      }
    })();
    return () => {
      unlistenOutput?.();
      unlistenExit?.();
      if (sessionId) invoke("pty_close", { sessionId }).catch(() => {});
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const send = async () => {
    if (!sessionId) return;
    const text = input;
    setInput("");
    if (text === "exit" || text === "exit\n") {
      await invoke("pty_close", { sessionId }).catch(() => {});
      return;
    }
    await invoke("pty_write", { sessionId, data: text + "\n" }).catch((e) => setError(String(e)));
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

  return (
    <div className={styles.panel} data-testid="terminal-pane">
      <div className={styles.header}>
        <span className={styles.title}>终端</span>
        <span className={`${styles.dot} ${ready ? styles.dotOn : ""}`} title={ready ? "运行中" : "已退出"} />
      </div>
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
          placeholder="运行命令 (Ctrl+L 清屏, exit 退出)"
          autoFocus
          data-testid="terminal-input"
        />
        <button type="button" className={styles.send} onClick={send} data-testid="terminal-send">⏎</button>
      </div>
    </div>
  );
}
