import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getXterm, getXtermAddonFit } from "../lib/xterm-loader";

interface Props {
  sessionId: string;
  onClose?: () => void;
  onStatusChange?: (msg: string) => void;
}

const terminalRegistry = new Map<string, { term: any; fit: any; ptyId: string | null }>();

export function closeTerminal(sessionId: string) {
  const entry = terminalRegistry.get(sessionId);
  if (!entry) return;
  if (entry.ptyId) {
    invoke("pty_close", { id: entry.ptyId }).catch(() => {});
  }
  entry.term.dispose();
  terminalRegistry.delete(sessionId);
}

export function resizeTerminal(sessionId: string, cols: number, rows: number) {
  const entry = terminalRegistry.get(sessionId);
  if (entry?.ptyId) {
    invoke("pty_resize", { id: entry.ptyId, cols, rows }).catch(() => {});
  }
}

const Terminal: React.FC<Props> = ({ sessionId, onClose, onStatusChange }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const initialized = useRef(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    let disposed = false;

    async function init() {
      const { Terminal: XTerm } = await getXterm();
      const { FitAddon } = await getXtermAddonFit();

      if (disposed) return;

      const term = new XTerm({
        cursorBlink: true,
        cursorStyle: "bar",
        fontSize: 13,
        fontFamily: "'SF Mono', Menlo, 'Fira Code', monospace",
        theme: {
          background: "#000000",
          foreground: "#f0f0f0",
          cursor: "#f0f0f0",
          selectionBackground: "#007aff55",
          black: "#1d1f21",
          red: "#cc6666",
          green: "#b5bd68",
          yellow: "#f0c674",
          blue: "#81a2be",
          magenta: "#b294bb",
          cyan: "#8abeb7",
          white: "#c5c8c6",
          brightBlack: "#666666",
          brightRed: "#cc6666",
          brightGreen: "#b5bd68",
          brightYellow: "#f0c674",
          brightBlue: "#81a2be",
          brightMagenta: "#b294bb",
          brightCyan: "#8abeb7",
          brightWhite: "#ffffff",
        },
        allowTransparency: true,
        cols: 80,
        rows: 24,
      });

      const fit = new FitAddon();
      term.loadAddon(fit);

      const entry = { term, fit, ptyId: null as string | null };
      terminalRegistry.set(sessionId, entry);

      if (containerRef.current) {
        term.open(containerRef.current);
      }

      setLoading(false);

      const doFit = () => {
        try { fit.fit(); } catch { }
      };
      doFit();
      const resizeTimer = setTimeout(doFit, 100);
      const ro = new ResizeObserver(() => doFit());
      if (containerRef.current) ro.observe(containerRef.current);

      invoke<string>("pty_spawn")
        .then((ptyId) => {
          entry.ptyId = ptyId;
          onStatusChange?.("终端已连接");

          const dims = fit.proposeDimensions();
          if (dims) {
            invoke("pty_resize", { id: ptyId, cols: dims.cols, rows: dims.rows }).catch(() => {});
          }

          const unlisten = listen<string>(`pty-output-${ptyId}`, (event) => {
            term.write(event.payload);
          });

          term.onData((data) => {
            invoke("pty_write", { id: ptyId, data }).catch(() => {});
          });

          term.onResize(({ cols, rows }: { cols: number; rows: number }) => {
            invoke("pty_resize", { id: ptyId, cols, rows }).catch(() => {});
          });

          return () => {
            unlisten.then((fn) => fn());
          };
        })
        .catch((e: unknown) => {
          term.write(`\x1b[31m终端启动失败: ${e}\x1b[0m\r\n`);
          onStatusChange?.(`终端错误: ${e}`);
        });

      return () => {
        disposed = true;
        clearTimeout(resizeTimer);
        ro.disconnect();
        closeTerminal(sessionId);
      };
    }

    const cleanupPromise = init();

    return () => {
      disposed = true;
      cleanupPromise.then((cleanup) => cleanup?.());
    };
  }, [sessionId, onStatusChange]);

  return (
    <div className="terminal-panel glass-panel">
      <div className="terminal-header">
        <span className="terminal-title">终端</span>
        <button className="btn-icon" onClick={onClose} title="关闭终端">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>
      <div className="terminal-body" ref={containerRef}>
        {loading && <div className="terminal-loading">加载终端...</div>}
      </div>
    </div>
  );
};

export default Terminal;
