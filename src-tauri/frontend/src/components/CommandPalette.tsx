import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useStore } from "../stores";
import { useNavigate } from "react-router-dom";

interface CommandItem {
  id: string;
  label: string;
  icon: string;
  section: string;
  shortcut?: string;
  keywords: string[];
  action: () => void;
}

const DEFAULT_ITEMS: CommandItem[] = [
  { id: "agent-manager", label: "Agent Manager", icon: "🤖", section: "Manage", keywords: ["agents", "manage agents", "ai", "assistants"], action: () => useStore.getState().setShowAgentManager(true) },
  { id: "privacy-filter", label: "Privacy Filter", icon: "🛡️", section: "Manage", keywords: ["privacy", "pii", "redact", "filter"], action: () => useStore.getState().setShowPrivacyFilter(true) },
  { id: "sandbox", label: "Sandbox Manager", icon: "📦", section: "Manage", keywords: ["docker", "container", "isolate", "run code"], action: () => useStore.getState().setShowSandboxManager(true) },
  { id: "identity", label: "Identity Manager", icon: "🪪", section: "Manage", keywords: ["keys", "access", "auth", "credentials"], action: () => useStore.getState().setShowIdentityManager(true) },

  { id: "evolution", label: "Evolution Panel", icon: "🧬", section: "Panels", keywords: ["seal", "pipeline", "train", "evolve"], action: () => useStore.getState().setEvolutionVisible(true) },
  { id: "sync", label: "Sync Panel", icon: "🔄", section: "Panels", keywords: ["backup", "cloud", "sync"], action: () => useStore.getState().setSyncVisible(true) },
  { id: "split-view", label: "Split View", icon: "🖥️", section: "Panels", keywords: ["compare", "side by side", "dual"], action: () => useStore.getState().setSplitViewActive(true) },
  { id: "agent-maker", label: "Agent Maker", icon: "⚗️", section: "Panels", keywords: ["create agent", "custom", "builder"], action: () => useStore.getState().setAgentMakerActive(true) },

  { id: "file-tree", label: "File Tree", icon: "📁", section: "Toggle", keywords: ["files", "explorer", "project", "browse"], action: () => { const st = useStore.getState(); st.setShowFileTree(!st.showFileTree); } },
  { id: "sidebar", label: "Toggle Sidebar", icon: "📋", section: "Toggle", keywords: ["collapse", "panel", "session list"], action: () => { const st = useStore.getState(); st.setSidebarCollapsed(!st.sidebarCollapsed); } },

  { id: "search", label: "Search Messages", icon: "🔍", section: "System", keywords: ["find", "search messages", "filter"], shortcut: "Cmd+F", action: () => useStore.getState().setShowSearch(true) },
  { id: "settings", label: "Settings", icon: "⚙️", section: "System", keywords: ["preferences", "config", "options", "prefs"], action: () => { /* navigate handled externally */ } },
  { id: "theme", label: "Toggle Theme", icon: "🎨", section: "System", keywords: ["dark", "light", "mode", "appearance"], action: () => { const st = useStore.getState(); const order: Array<"light" | "dark" | "system"> = ["light", "dark", "system"]; const idx = order.indexOf(st.settings.theme); st.setSettings({ ...st.settings, theme: order[(idx + 1) % order.length] }); } },
  { id: "shortcuts", label: "Keyboard Shortcuts", icon: "⌨️", section: "System", keywords: ["keys", "hotkeys", "help"], action: () => useStore.getState().setShowShortcuts(true) },
];

function highlight(text: string, query: string): React.ReactNode {
  if (!query.trim()) return text;
  const parts = text.split(new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "gi"));
  return parts.map((part, i) =>
    part.toLowerCase() === query.toLowerCase()
      ? <mark key={i} className="cp-mark">{part}</mark>
      : part
  );
}

const CommandPalette: React.FC = () => {
  const show = useStore((s) => s.showCommandPalette);
  const setShow = useStore((s) => s.setShowCommandPalette);
  const navigate = useNavigate();

  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const items = useMemo(() => {
    if (!query.trim()) return DEFAULT_ITEMS;
    const q = query.toLowerCase();
    return DEFAULT_ITEMS.filter(item =>
      item.label.toLowerCase().includes(q) ||
      item.keywords.some(k => k.toLowerCase().includes(q)) ||
      item.section.toLowerCase().includes(q)
    );
  }, [query]);

  useEffect(() => { setSelectedIdx(0); }, [query]);

  useEffect(() => {
    if (show) {
      setQuery("");
      setSelectedIdx(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [show]);

  const execute = useCallback((item: CommandItem) => {
    setShow(false);
    if (item.id === "settings") {
      navigate("/settings");
      return;
    }
    item.action();
  }, [setShow, navigate]);

  useEffect(() => {
    if (!show) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") { e.preventDefault(); setShow(false); return; }
      if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIdx(i => Math.min(i + 1, items.length - 1)); return; }
      if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIdx(i => Math.max(i - 1, 0)); return; }
      if (e.key === "Enter" && items[selectedIdx]) { e.preventDefault(); execute(items[selectedIdx]); return; }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [show, items, selectedIdx, execute]);

  useEffect(() => {
    const el = listRef.current?.children[selectedIdx] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIdx]);

  if (!show) return null;

  const grouped = items.reduce<Record<string, CommandItem[]>>((acc, item) => {
    (acc[item.section] = acc[item.section] || []).push(item);
    return acc;
  }, {});

  let globalIdx = 0;

  return (
    <div className="cp-overlay" onClick={() => setShow(false)}>
      <div className="cp-modal" onClick={(e) => e.stopPropagation()}>
        <div className="cp-input-wrap">
          <span className="cp-input-icon">➤</span>
          <input
            ref={inputRef}
            className="cp-input"
            type="text"
            placeholder="Type a command or search..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <span className="cp-input-hint">Esc to close</span>
        </div>

        <div className="cp-list" ref={listRef}>
          {Object.entries(grouped).map(([section, sectionItems]) => (
            <div key={section} className="cp-section">
              <div className="cp-section-label">{section}</div>
              {sectionItems.map((item) => {
                const idx = globalIdx++;
                return (
                  <div
                    key={item.id}
                    className={`cp-item ${idx === selectedIdx ? "cp-item-selected" : ""}`}
                    onClick={() => execute(item)}
                    onMouseEnter={() => setSelectedIdx(idx)}
                  >
                    <span className="cp-item-icon">{item.icon}</span>
                    <span className="cp-item-label">{highlight(item.label, query)}</span>
                    <div className="cp-item-spacer" />
                    {item.shortcut && <span className="cp-item-shortcut">{item.shortcut}</span>}
                  </div>
                );
              })}
            </div>
          ))}
          {items.length === 0 && (
            <div className="cp-empty">No matching commands</div>
          )}
        </div>

        <div className="cp-footer">
          <span><kbd className="cp-kbd">↑</kbd><kbd className="cp-kbd">↓</kbd> navigate</span>
          <span><kbd className="cp-kbd">↵</kbd> select</span>
          <span><kbd className="cp-kbd">Esc</kbd> close</span>
        </div>
      </div>
    </div>
  );
};

export default CommandPalette;
