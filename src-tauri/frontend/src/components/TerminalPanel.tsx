import { useEffect, useRef, useState } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { invoke } from "@tauri-apps/api/core";
import "@xterm/xterm/css/xterm.css";

const MAX_HISTORY = 100;

const TerminalPanel: React.FC = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const historyRef = useRef<string[]>([]);
  const historyIndexRef = useRef(-1);
  const currentLineRef = useRef("");
  const savedInputRef = useRef("");
  const disposedRef = useRef(false);
  const resizeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (disposedRef.current) return;

    const term = new Terminal({
      theme: {
        background: "#0f0f11",
        foreground: "#e8e8ed",
        cursor: "#0a84ff",
        selectionBackground: "rgba(10,132,255,0.3)",
        black: "#1a1a1e",
        red: "#ff453a",
        green: "#30d158",
        yellow: "#ffd60a",
        blue: "#0a84ff",
        magenta: "#bf5af2",
        cyan: "#5e5ce6",
        white: "#e8e8ed",
      },
      fontFamily: "'JetBrains Mono', 'SF Mono', 'Fira Code', monospace",
      fontSize: 13,
      cursorBlink: true,
      allowTransparency: false,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    if (containerRef.current) {
      term.open(containerRef.current);
    }

    try {
      fitAddon.fit();
    } catch {
      // container may not be visible yet
    }

    term.write("\r\nNeoTrix Terminal v0.2\r\nType /help for commands\r\n");
    setLoading(false);

    term.onData((data) => {
      if (disposedRef.current) return;

      if (data === "\r") {
        const cmd = currentLineRef.current;
        term.write("\r\n");
        currentLineRef.current = "";
        historyIndexRef.current = -1;

        if (cmd.trim() === "") return;

        historyRef.current.push(cmd);
        if (historyRef.current.length > MAX_HISTORY) {
          historyRef.current.shift();
        }

        if (cmd.startsWith("/")) {
          invoke<string>("execute_command", { command: cmd })
            .then((result) => {
              if (!disposedRef.current) term.write(result + "\r\n");
            })
            .catch((err) => {
              if (!disposedRef.current) {
                term.write(`\r\n\x1b[31mError: ${err}\x1b[0m\r\n`);
              }
            });
        } else {
          invoke<string>("cli_command", { input: cmd })
            .then((result) => {
              if (!disposedRef.current) term.write(result + "\r\n");
            })
            .catch(() => {
              if (!disposedRef.current) {
                term.write("NeoTrix CLI mode coming soon\r\n");
              }
            });
        }
        return;
      }

      if (data === "\x7f") {
        if (currentLineRef.current.length > 0) {
          currentLineRef.current = currentLineRef.current.slice(0, -1);
          term.write("\b \b");
        }
        return;
      }

      if (data === "\x03") {
        term.write("^C\r\n");
        currentLineRef.current = "";
        historyIndexRef.current = -1;
        return;
      }

      if (data === "\u0015") {
        currentLineRef.current = "";
        term.write("\r\x1b[K");
        return;
      }

      if (data === "\x0c") {
        term.write("\x1b[2J\x1b[H");
        term.write("NeoTrix Terminal v0.2\r\nType /help for commands\r\n");
        return;
      }

      if (data === "\x1b[A") {
        if (historyRef.current.length === 0) return;
        if (historyIndexRef.current === -1) {
          savedInputRef.current = currentLineRef.current;
          historyIndexRef.current = historyRef.current.length - 1;
        } else if (historyIndexRef.current > 0) {
          historyIndexRef.current--;
        } else {
          return;
        }
        const entry = historyRef.current[historyIndexRef.current];
        term.write("\r\x1b[K" + entry);
        currentLineRef.current = entry;
        return;
      }

      if (data === "\x1b[B") {
        if (historyIndexRef.current === -1) return;
        historyIndexRef.current++;
        if (historyIndexRef.current >= historyRef.current.length) {
          historyIndexRef.current = -1;
          term.write("\r\x1b[K" + savedInputRef.current);
          currentLineRef.current = savedInputRef.current;
        } else {
          const entry = historyRef.current[historyIndexRef.current];
          term.write("\r\x1b[K" + entry);
          currentLineRef.current = entry;
        }
        return;
      }

      currentLineRef.current += data;
      term.write(data);
    });

    const doFit = () => {
      try {
        fitAddon.fit();
      } catch {
        // container may have zero dimensions
      }
    };

    const debouncedFit = () => {
      if (resizeTimeoutRef.current) clearTimeout(resizeTimeoutRef.current);
      resizeTimeoutRef.current = setTimeout(doFit, 100);
    };

    const ro = new ResizeObserver(debouncedFit);
    if (containerRef.current) ro.observe(containerRef.current);
    window.addEventListener("resize", debouncedFit);

    return () => {
      disposedRef.current = true;
      if (resizeTimeoutRef.current) clearTimeout(resizeTimeoutRef.current);
      window.removeEventListener("resize", debouncedFit);
      ro.disconnect();
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
    };
  }, []);

  return (
    <div className="h-full w-full bg-[#0f0f11] relative overflow-hidden">
      {loading && (
        <div className="absolute inset-0 flex items-center justify-center text-[#8e8e93] text-sm z-10">
          Connecting...
        </div>
      )}
      <div ref={containerRef} className="h-full w-full" />
    </div>
  );
};

export default TerminalPanel;
