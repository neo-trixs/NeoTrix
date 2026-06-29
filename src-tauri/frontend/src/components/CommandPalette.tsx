import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../store";

interface Command {
  id: string;
  name: string;
  shortcut: string;
  description: string;
  action: (st: ReturnType<typeof useStore.getState>) => void;
}

const COMMANDS: Command[] = [
  {
    id: "new-session", name: "New Session", shortcut: "Cmd+N",
    description: "Create a new empty session",
    action: (st) => st.addSession(),
  },
  {
    id: "search", name: "Search Messages", shortcut: "Cmd+R",
    description: "Search across all sessions",
    action: (st) => { st.setShowSearch(true); st.setShowCommandPalette(false); },
  },
  {
    id: "evolution", name: "Toggle Evolution", shortcut: "Cmd+E",
    description: "Show or hide evolution dashboard",
    action: (st) => st.setEvolutionVisible(!st.evolutionVisible),
  },
  {
    id: "file-tree", name: "Toggle File Tree", shortcut: "Cmd+B",
    description: "Show or hide file explorer",
    action: (st) => st.setShowFileTree(!st.showFileTree),
  },
  {
    id: "terminal", name: "Toggle Terminal", shortcut: "Cmd+T",
    description: "Show or hide terminal panel",
    action: (st) => st.setShowTerminal(!st.showTerminal),
  },
  {
    id: "agent-flow", name: "Toggle Agent Flow", shortcut: "Cmd+F",
    description: "Show or hide agent flow graph",
    action: (st) => st.setAgentFlowActive(!st.agentFlowActive),
  },
  {
    id: "moment-feed", name: "Toggle Moment Feed", shortcut: "",
    description: "Toggle moment feed sidebar",
    action: (st) => st.setMomentFeedVisible(!st.momentFeedVisible),
  },
  {
    id: "settings", name: "Toggle Settings", shortcut: "Cmd+,",
    description: "Open application settings",
    action: (st) => st.setShowSettings(!st.showSettings),
  },
  {
    id: "split-view", name: "Toggle Split View", shortcut: "",
    description: "Toggle split view mode",
    action: (st) => st.setSplitViewActive(!st.splitViewActive),
  },
  {
    id: "virtual-os", name: "Toggle Virtual OS", shortcut: "",
    description: "Toggle virtual OS desktop",
    action: (st) => st.setVirtualOSActive(!st.virtualOSActive),
  },
  {
    id: "dark-mode", name: "Toggle Dark Mode", shortcut: "",
    description: "Switch between light, dark, and system theme",
    action: (st) => {
      const order: Array<"light" | "dark" | "system"> = ["light", "dark", "system"];
      const idx = order.indexOf(st.settings.theme);
      const next = order[(idx + 1) % order.length];
      st.setSettings({ ...st.settings, theme: next });
    },
  },
  {
    id: "shortcuts", name: "Help & Shortcuts", shortcut: "Cmd+/",
    description: "Show keyboard shortcuts reference",
    action: (st) => st.setShowShortcuts(!st.showShortcuts),
  },
];

function fuzzyMatch(text: string, query: string): boolean {
  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  let qi = 0;
  for (let i = 0; i < lower.length && qi < q.length; i++) {
    if (lower[i] === q[qi]) qi++;
  }
  return qi === q.length;
}

const CommandPalette: React.FC = () => {
  const setShowCommandPalette = useStore((s) => s.setShowCommandPalette);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = useMemo(() => {
    if (!query.trim()) return COMMANDS;
    return COMMANDS.filter((cmd) => fuzzyMatch(cmd.name + " " + cmd.description, query));
  }, [query]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  const executeCommand = useCallback((cmd: Command) => {
    const st = useStore.getState();
    cmd.action(st);
    setShowCommandPalette(false);
  }, [setShowCommandPalette]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowCommandPalette(false);
        return;
      }
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, filtered.length - 1));
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        return;
      }
      if (e.key === "Enter" && filtered[selectedIndex]) {
        e.preventDefault();
        executeCommand(filtered[selectedIndex]);
        return;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [filtered, selectedIndex, setShowCommandPalette, executeCommand]);

  useEffect(() => {
    if (!listRef.current) return;
    const el = listRef.current.querySelector(".cmd-item.selected") as HTMLElement | null;
    if (el) el.scrollIntoView({ block: "nearest" });
  }, [selectedIndex]);

  return (
    <div className="cmd-overlay" onMouseDown={(e) => {
      if (e.target === e.currentTarget) setShowCommandPalette(false);
    }}>
      <div className="cmd-panel glass-panel">
        <div className="cmd-input-wrap">
          <svg className="cmd-search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M4 8a4 4 0 1 1 8 0" />
            <path d="M4 8v2a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2V8" />
            <path d="M6 11v2h4v-2" />
          </svg>
          <input
            ref={inputRef}
            className="cmd-input"
            type="text"
            placeholder="Type a command or search…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            spellCheck={false}
          />
          {query && (
            <button className="cmd-clear" onClick={() => setQuery("")} tabIndex={-1}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <line x1="3" y1="3" x2="11" y2="11" />
                <line x1="11" y1="3" x2="3" y2="11" />
              </svg>
            </button>
          )}
        </div>

        <div className="cmd-hints">
          <span className="cmd-hint-key">↑↓</span> Navigate
          <span className="cmd-hint-key" style={{ marginLeft: 12 }}>↵</span> Execute
          <span className="cmd-hint-key" style={{ marginLeft: 12 }}>Esc</span> Close
        </div>

        <div className="cmd-results" ref={listRef}>
          {filtered.length === 0 && (
            <div className="cmd-empty">
              <p>No commands found for <strong>"{query}"</strong></p>
            </div>
          )}
          {filtered.map((cmd, i) => (
            <div
              key={cmd.id}
              className={`cmd-item ${i === selectedIndex ? "selected" : ""}`}
              onClick={() => executeCommand(cmd)}
              onMouseEnter={() => setSelectedIndex(i)}
            >
              <div className="cmd-item-info">
                <span className="cmd-item-name">{cmd.name}</span>
                <span className="cmd-item-desc">{cmd.description}</span>
              </div>
              {cmd.shortcut && <span className="cmd-item-shortcut">{cmd.shortcut}</span>}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;
