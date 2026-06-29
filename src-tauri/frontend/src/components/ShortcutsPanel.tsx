import React, { useEffect } from "react";

interface ShortcutEntry {
  keys: string[];
  label: string;
}

const CATEGORIES: { title: string; shortcuts: ShortcutEntry[] }[] = [
  {
    title: "Sessions",
    shortcuts: [
      { keys: ["⌘", "N"], label: "New Session" },
      { keys: ["⌘", "W"], label: "Close Session" },
      { keys: ["⌘", "T"], label: "Toggle Terminal" },
      { keys: ["⌘", "⇧", "["], label: "Previous Session" },
      { keys: ["⌘", "⇧", "]"], label: "Next Session" },
      { keys: ["⌘", "1…8"], label: "Switch to Session" },
    ],
  },
  {
    title: "Navigation",
    shortcuts: [
      { keys: ["⌘", "B"], label: "Toggle File Tree" },
      { keys: ["⌘", "E"], label: "Evolution Dashboard" },
      { keys: ["⌘", "F"], label: "Agent Flow" },
      { keys: ["⌘", ","], label: "Settings" },
    ],
  },
  {
    title: "Editor",
    shortcuts: [
      { keys: ["⌘", "⇧", "I"], label: "Open Editor" },
    ],
  },
  {
    title: "General",
    shortcuts: [
      { keys: ["⌘", "/"], label: "Show / Hide Shortcuts" },
      { keys: ["⎋"], label: "Close Panel" },
    ],
  },
];

interface ShortcutsPanelProps {
  onClose: () => void;
}

const ShortcutsPanel: React.FC<ShortcutsPanelProps> = ({ onClose }) => {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onClose]);

  return (
    <div className="shortcuts-overlay" onClick={onClose}>
      <div className="shortcuts-panel glass-panel" onClick={(e) => e.stopPropagation()}>
        <div className="shortcuts-header">
          <h2>Keyboard Shortcuts</h2>
          <button className="shortcuts-close" onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <line x1="2" y1="2" x2="12" y2="12" />
              <line x1="12" y1="2" x2="2" y2="12" />
            </svg>
          </button>
        </div>
        <div className="shortcuts-body">
          {CATEGORIES.map((cat) => (
            <div className="shortcuts-group" key={cat.title}>
              <div className="shortcuts-group-title">{cat.title}</div>
              <div className="shortcuts-items">
                {cat.shortcuts.map((sc) => (
                  <div className="shortcuts-row" key={sc.label}>
                    <span className="shortcuts-label">{sc.label}</span>
                    <span className="shortcuts-keys">
                      {sc.keys.map((k, i) => (
                        <React.Fragment key={i}>
                          {i > 0 && <span className="shortcuts-plus">+</span>}
                          <kbd className="shortcuts-key">{k}</kbd>
                        </React.Fragment>
                      ))}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ShortcutsPanel;
