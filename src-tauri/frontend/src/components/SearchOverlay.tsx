import React, { useCallback, useEffect, useRef, useMemo, useState } from "react";
import { useStore } from "../store";
import type { Session, Message } from "../types";

interface SearchResult {
  sessionIndex: number;
  sessionName: string;
  messageIndex: number;
  message: Message;
  matchStart: number;
  matchEnd: number;
}

const SEARCH_ROLES: Record<string, string> = {
  user: "You",
  assistant: "Assistant",
  system: "System",
  error: "Error",
};

const ROLE_COLORS: Record<string, string> = {
  user: "#007aff",
  assistant: "#34c759",
  system: "#ff9500",
  error: "#ff3b30",
};

function highlightText(text: string, query: string): React.ReactNode {
  if (!query) return text;
  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  const parts: React.ReactNode[] = [];
  let idx = 0;
  let pos = lower.indexOf(q, idx);
  while (pos !== -1) {
    if (pos > idx) parts.push(text.slice(idx, pos));
    parts.push(<mark key={pos} className="search-match">{text.slice(pos, pos + q.length)}</mark>);
    idx = pos + q.length;
    pos = lower.indexOf(q, idx);
  }
  if (idx < text.length) parts.push(text.slice(idx));
  return parts.length > 0 ? parts : text;
}

function getMessagePreview(content: string, query: string, maxLen: number = 120): string {
  if (!query) return content.length > maxLen ? content.slice(0, maxLen) + "..." : content;
  const lower = content.toLowerCase();
  const q = query.toLowerCase();
  const idx = lower.indexOf(q);
  if (idx === -1) return content.length > maxLen ? content.slice(0, maxLen) + "..." : content;
  const start = Math.max(0, idx - Math.floor((maxLen - q.length) / 2));
  const end = Math.min(content.length, start + maxLen);
  let preview = content.slice(start, end);
  if (start > 0) preview = "…" + preview;
  if (end < content.length) preview = preview + "…";
  return preview;
}

const SearchOverlay: React.FC = () => {
  const sessions = useStore((s) => s.sessions);
  const searchQuery = useStore((s) => s.searchQuery);
  const setSearchQuery = useStore((s) => s.setSearchQuery);
  const setShowSearch = useStore((s) => s.setShowSearch);
  const setActiveSessionIndex = useStore((s) => s.setActiveSessionIndex);

  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);

  const results = useMemo<SearchResult[]>(() => {
    if (!searchQuery.trim()) return [];
    const q = searchQuery.toLowerCase();
    const out: SearchResult[] = [];
    for (let si = 0; si < sessions.length; si++) {
      const session = sessions[si];
      for (let mi = 0; mi < session.messages.length; mi++) {
        const msg = session.messages[mi];
        const content = msg.content || "";
        const lower = content.toLowerCase();
        let idx = lower.indexOf(q);
        while (idx !== -1) {
          out.push({
            sessionIndex: si,
            sessionName: session.name,
            messageIndex: mi,
            message: msg,
            matchStart: idx,
            matchEnd: idx + q.length,
          });
          idx = lower.indexOf(q, idx + 1);
        }
      }
    }
    return out;
  }, [searchQuery, sessions]);

  const groupedResults = useMemo(() => {
    const map = new Map<number, { sessionName: string; results: SearchResult[] }>();
    for (const r of results) {
      if (!map.has(r.sessionIndex)) {
        map.set(r.sessionIndex, { sessionName: r.sessionName, results: [] });
      }
      map.get(r.sessionIndex)!.results.push(r);
    }
    return Array.from(map.entries());
  }, [results]);

  const totalCount = results.length;

  useEffect(() => {
    if (searchQuery.trim()) {
      setSelectedIndex(0);
    }
  }, [searchQuery]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [sessions]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const jumpToResult = useCallback((result: SearchResult) => {
    setActiveSessionIndex(result.sessionIndex);
    setShowSearch(false);
    setSearchQuery("");

    setTimeout(() => {
      const container = document.querySelector(".chat-messages");
      if (!container) return;
      const messageEls = container.querySelectorAll("[data-message-timestamp]");
      const targetTimestamp = result.message.timestamp;
      if (targetTimestamp) {
        for (const el of messageEls) {
          if (Number(el.getAttribute("data-message-timestamp")) === targetTimestamp) {
            el.scrollIntoView({ behavior: "smooth", block: "center" });
            (el as HTMLElement).style.outline = "2px solid var(--mac-primary)";
            (el as HTMLElement).style.outlineOffset = "-2px";
            el.classList.add("search-highlighted-message");
            setTimeout(() => {
              (el as HTMLElement).style.outline = "";
              (el as HTMLElement).style.outlineOffset = "";
              el.classList.remove("search-highlighted-message");
            }, 2000);
            break;
          }
        }
      }
    }, 100);
  }, [setActiveSessionIndex, setShowSearch, setSearchQuery]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowSearch(false);
        setSearchQuery("");
        return;
      }

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, totalCount - 1));
        return;
      }

      if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
        return;
      }

      if (e.key === "Enter" && results[selectedIndex]) {
        e.preventDefault();
        jumpToResult(results[selectedIndex]);
        return;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [results, selectedIndex, totalCount, setShowSearch, setSearchQuery, jumpToResult]);

  useEffect(() => {
    if (!listRef.current) return;
    const selectedEl = listRef.current.querySelector(".search-result-item.selected") as HTMLElement | null;
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: "nearest" });
    }
  }, [selectedIndex]);

  let flatIndex = 0;

  return (
    <div className="search-overlay" onMouseDown={(e) => {
      if (e.target === e.currentTarget) {
        setShowSearch(false);
        setSearchQuery("");
      }
    }}>
      <div className="search-panel glass-panel">
        <div className="search-input-wrap">
          <svg className="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="6.5" cy="6.5" r="5" />
            <line x1="10" y1="10" x2="14" y2="14" />
          </svg>
          <input
            ref={inputRef}
            className="search-input"
            type="text"
            placeholder="Search messages across all sessions…"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            spellCheck={false}
          />
          {searchQuery && (
            <button className="search-clear" onClick={() => setSearchQuery("")} tabIndex={-1}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <line x1="3" y1="3" x2="11" y2="11" />
                <line x1="11" y1="3" x2="3" y2="11" />
              </svg>
            </button>
          )}
          <span className="search-count">{totalCount > 0 ? `${totalCount} result${totalCount !== 1 ? "s" : ""}` : ""}</span>
        </div>

        <div className="search-hints">
          <span className="search-hint-key">↑↓</span> Navigate
          <span className="search-hint-key" style={{ marginLeft: 12 }}>↵</span> Jump
          <span className="search-hint-key" style={{ marginLeft: 12 }}>Esc</span> Close
        </div>

        <div className="search-results" ref={listRef}>
          {!searchQuery.trim() && (
            <div className="search-empty">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round" style={{ opacity: 0.3 }}>
                <circle cx="11" cy="11" r="8" />
                <line x1="21" y1="21" x2="16.65" y2="16.65" />
              </svg>
              <p>Type to search across all sessions</p>
            </div>
          )}

          {searchQuery.trim() && results.length === 0 && (
            <div className="search-empty">
              <p>No results found for <strong>"{searchQuery}"</strong></p>
            </div>
          )}

          {groupedResults.map(([sessionIndex, group]) => (
            <div key={sessionIndex} className="search-group">
              <div className="search-group-header">
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="1" y="2" width="10" height="8" rx="1" />
                  <line x1="1" y1="5" x2="11" y2="5" />
                </svg>
                <span>{group.sessionName}</span>
                <span className="search-group-count">{group.results.length}</span>
              </div>

              {group.results.map((result) => {
                const currentFlatIndex = flatIndex++;
                const isSelected = currentFlatIndex === selectedIndex;
                const roleColor = ROLE_COLORS[result.message.role] || "#86868b";

                return (
                  <div
                    key={`${result.sessionIndex}-${result.messageIndex}-${result.matchStart}`}
                    className={`search-result-item ${isSelected ? "selected" : ""}`}
                    onClick={() => jumpToResult(result)}
                    onMouseEnter={() => setSelectedIndex(currentFlatIndex)}
                  >
                    <div className="search-result-header">
                      <span className="search-result-role" style={{ color: roleColor }}>
                        {SEARCH_ROLES[result.message.role] || result.message.role}
                      </span>
                      <span className="search-result-session-tag">{group.sessionName}</span>
                    </div>
                    <div className="search-result-preview">
                      {highlightText(
                        getMessagePreview(result.message.content, searchQuery),
                        searchQuery
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default SearchOverlay;
