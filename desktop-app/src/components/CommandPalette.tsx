import { useState, useEffect, useRef, useCallback } from 'react';

interface Command {
  id: string;
  label: string;
  shortcut?: string;
  icon: React.ReactNode;
  action: () => void;
}

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
  commands: Command[];
}

export default function CommandPalette({ open, onClose, commands }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIdx, setSelectedIdx] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = commands.filter(c =>
    c.label.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (open) {
      setQuery('');
      setSelectedIdx(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      onClose();
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIdx(i => Math.min(i + 1, filtered.length - 1));
      return;
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIdx(i => Math.max(i - 1, 0));
      return;
    }
    if (e.key === 'Enter' && filtered[selectedIdx]) {
      filtered[selectedIdx].action();
      onClose();
    }
  }, [filtered, selectedIdx, onClose]);

  if (!open) return null;

  return (
    <div className="palette-overlay" onClick={onClose}>
      <div className="palette-modal" onClick={e => e.stopPropagation()} onKeyDown={handleKeyDown}>
        <div className="palette-input-row">
          <svg className="palette-search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            ref={inputRef}
            className="palette-input"
            type="text"
            placeholder="搜索命令..."
            value={query}
            onChange={e => { setQuery(e.target.value); setSelectedIdx(0); }}
          />
        </div>
        <div className="palette-results">
          {filtered.map((c, i) => (
            <button
              key={c.id}
              className={`palette-item ${i === selectedIdx ? 'selected' : ''}`}
              onClick={() => { c.action(); onClose(); }}
              onMouseEnter={() => setSelectedIdx(i)}
            >
              <span className="palette-item-icon">{c.icon}</span>
              <span className="palette-item-label">{c.label}</span>
              {c.shortcut && <span className="palette-item-shortcut">{c.shortcut}</span>}
            </button>
          ))}
          {filtered.length === 0 && (
            <div className="palette-empty">No matching commands</div>
          )}
        </div>
      </div>
    </div>
  );
}
