import React, { useEffect } from "react";

interface ShortcutRow {
  keys: string[];
  label: string;
}

const SECTIONS: { title: string; shortcuts: ShortcutRow[] }[] = [
  {
    title: "Compose",
    shortcuts: [
      { keys: ["Enter"], label: "Send message" },
      { keys: ["Shift", "Enter"], label: "New line" },
      { keys: ["Alt", "Enter"], label: "Multi-line toggle" },
    ],
  },
  {
    title: "Navigation",
    shortcuts: [
      { keys: ["Cmd", "K"], label: "Command palette" },
      { keys: ["Cmd", "L"], label: "Focus input" },
      { keys: ["Cmd", ","], label: "Settings" },
      { keys: ["Cmd", "."], label: "Switch mode (Chat→Plan→Agent→Chat)" },
      { keys: ["Cmd", "/"], label: "Show shortcuts (this modal)" },
    ],
  },
  {
    title: "Agent Control",
    shortcuts: [
      { keys: ["Ctrl", "Esc"], label: "Interrupt agent" },
      { keys: ["Cmd", "Shift", "N"], label: "New session" },
    ],
  },
  {
    title: "Display",
    shortcuts: [
      { keys: ["Cmd", "B"], label: "Toggle sidebar" },
      { keys: ["Cmd", "T"], label: "Toggle terminal" },
      { keys: ["Cmd", "D"], label: "Toggle dark mode" },
    ],
  },
];

interface ShortcutsModalProps {
  onClose: () => void;
}

const ShortcutsModal: React.FC<ShortcutsModalProps> = ({ onClose }) => {
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
          {SECTIONS.map((sec) => (
            <div className="shortcuts-group" key={sec.title}>
              <div className="shortcuts-group-title">{sec.title}</div>
              <div className="shortcuts-items">
                {sec.shortcuts.map((sc) => (
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

export default ShortcutsModal;
