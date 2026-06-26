import React, { useCallback, useRef, useState } from "react";
import type { Session } from "../types";
import { useStore } from "../store";

interface Props {
  sessions: Session[];
  activeSession: number;
  onSelect: (index: number) => void;
  onNew: () => void;
  onClose: (index: number) => void;
  onRename: (index: number, name: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
  onDuplicate: (index: number) => void;
  onCloseOthers: (index: number) => void;
}

const TabBar: React.FC<Props> = ({
  sessions,
  activeSession,
  onSelect,
  onNew,
  onClose,
  onRename,
  onReorder,
  onDuplicate,
  onCloseOthers,
}) => {
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editValue, setEditValue] = useState("");
  const [contextMenu, setContextMenu] = useState<{ index: number; x: number; y: number } | null>(null);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const editRef = useRef<HTMLInputElement>(null);

  const handleDoubleClick = useCallback((index: number, name: string) => {
    setEditingIndex(index);
    setEditValue(name);
    setTimeout(() => {
      editRef.current?.focus();
      editRef.current?.select();
    }, 10);
  }, []);

  const handleRenameConfirm = useCallback(() => {
    if (editingIndex !== null && editValue.trim()) {
      onRename(editingIndex, editValue.trim());
    }
    setEditingIndex(null);
    setEditValue("");
  }, [editingIndex, editValue, onRename]);

  const handleContextMenu = useCallback((e: React.MouseEvent, index: number) => {
    e.preventDefault();
    setContextMenu({ index, x: e.clientX, y: e.clientY });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const handleDragStart = useCallback((e: React.DragEvent, index: number) => {
    setDragIndex(index);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", String(index));
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    setDragOverIndex(index);
  }, []);

  const handleDragLeave = useCallback(() => {
    setDragOverIndex(null);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent, toIndex: number) => {
      e.preventDefault();
      if (dragIndex !== null && dragIndex !== toIndex) {
        onReorder(dragIndex, toIndex);
      }
      setDragIndex(null);
      setDragOverIndex(null);
    },
    [dragIndex, onReorder]
  );

  const handleDragEnd = useCallback(() => {
    setDragIndex(null);
    setDragOverIndex(null);
  }, []);

  const agentMode = useStore((s) => s.agentMode);

  React.useEffect(() => {
    if (contextMenu) {
      const handler = () => closeContextMenu();
      window.addEventListener("click", handler);
      return () => window.removeEventListener("click", handler);
    }
  }, [contextMenu, closeContextMenu]);

  return (
    <div className="tab-bar" role="tablist" aria-label="Session tabs">
      <div className="tab-list">
        {sessions.map((s, i) => (
          <div
            key={s.id}
            role="tab"
            aria-selected={i === activeSession}
            className={`tab-item${i === activeSession ? " active" : ""}${
              dragOverIndex === i ? " drag-over" : ""
            }${dragIndex === i ? " dragging" : ""}`}
            onClick={() => {
              if (editingIndex === null) onSelect(i);
              closeContextMenu();
            }}
            onDoubleClick={() => handleDoubleClick(i, s.name)}
            onContextMenu={(e) => handleContextMenu(e, i)}
            draggable={editingIndex !== i}
            onDragStart={(e) => handleDragStart(e, i)}
            onDragOver={(e) => handleDragOver(e, i)}
            onDragLeave={handleDragLeave}
            onDrop={(e) => handleDrop(e, i)}
            onDragEnd={handleDragEnd}
          >
            {editingIndex === i ? (
              <input
                ref={editRef}
                className="tab-rename-input"
                value={editValue}
                onChange={(e) => setEditValue(e.target.value)}
                onBlur={handleRenameConfirm}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleRenameConfirm();
                  if (e.key === "Escape") setEditingIndex(null);
                  e.stopPropagation();
                }}
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <>
                <span className={`tab-mode-dot ${agentMode}`} />
                <span className="tab-label">{s.name}</span>
              </>
            )}
            <div className="tab-meta">{s.messages.length}</div>
            {sessions.length > 1 && (
              <button
                className="tab-close-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  onClose(i);
                  closeContextMenu();
                }}
                title="Close session (Cmd+W)"
              >
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                  <path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                </svg>
              </button>
            )}
          </div>
        ))}
      </div>
      <button className="tab-new-btn" onClick={onNew} title="New session (Cmd+T)">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M7 2v10M2 7h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        </svg>
      </button>

      {contextMenu && (
        <div
          className="tab-context-menu glass-panel"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            className="tab-context-item"
            onClick={() => { onClose(contextMenu.index); closeContextMenu(); }}
          >
            Close Tab
          </button>
          <button
            className="tab-context-item"
            onClick={() => { onCloseOthers(contextMenu.index); closeContextMenu(); }}
          >
            Close Other Tabs
          </button>
          <button
            className="tab-context-item"
            onClick={() => {
              handleDoubleClick(contextMenu.index, sessions[contextMenu.index].name);
              closeContextMenu();
            }}
          >
            Rename
          </button>
          <button
            className="tab-context-item"
            onClick={() => { onDuplicate(contextMenu.index); closeContextMenu(); }}
          >
            Duplicate Session
          </button>
        </div>
      )}
    </div>
  );
};

export default TabBar;
