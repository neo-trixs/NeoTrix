import React, { useState, useEffect, useMemo, useCallback, useRef } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";

interface DocSection {
  id: string;
  title: string;
  level: number;
  content: string;
}

export interface DocViewerProps {
  content: string;
  title?: string;
  initialSection?: string;
  onClose?: () => void;
  onSectionChange?: (sectionId: string) => void;
}

const DocViewer: React.FC<DocViewerProps> = ({ content, title, initialSection, onClose, onSectionChange }) => {
  const [activeSection, setActiveSection] = useState(initialSection || "");
  const [searchQuery, setSearchQuery] = useState("");
  const [fontSize, setFontSize] = useState(14);
  const contentRef = useRef<HTMLDivElement>(null);
  const [isSearchOpen, setIsSearchOpen] = useState(false);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const sections: DocSection[] = useMemo(() => {
    const lines = content.split("\n");
    const result: DocSection[] = [];
    let currentId = "intro";
    let currentTitle = "Introduction";
    let currentLevel = 1;
    let currentContent: string[] = [];

    for (const line of lines) {
      const headingMatch = line.match(/^(#{1,6})\s+(.+)/);
      if (headingMatch) {
        if (currentContent.length > 0) {
          result.push({
            id: currentId,
            title: currentTitle,
            level: currentLevel,
            content: currentContent.join("\n"),
          });
        }
        currentLevel = headingMatch[1].length;
        currentTitle = headingMatch[2];
        currentId = currentTitle.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
        currentContent = [line];
      } else {
        currentContent.push(line);
      }
    }
    if (currentContent.length > 0) {
      result.push({
        id: currentId,
        title: currentTitle,
        level: currentLevel,
        content: currentContent.join("\n"),
      });
    }
    return result;
  }, [content]);

  const searchResults = useMemo(() => {
    if (!searchQuery.trim()) return [];
    const q = searchQuery.toLowerCase();
    return sections
      .filter(s => s.content.toLowerCase().includes(q))
      .map(s => {
        const idx = s.content.toLowerCase().indexOf(q);
        const start = Math.max(0, idx - 40);
        const end = Math.min(s.content.length, idx + q.length + 40);
        return {
          sectionId: s.id,
          context: (start > 0 ? "..." : "") + s.content.slice(start, end) + (end < s.content.length ? "..." : ""),
        };
      });
  }, [searchQuery, sections]);

  const visibleSections = useMemo(() => {
    if (!searchQuery.trim()) return sections;
    const matchedIds = new Set(searchResults.map(r => r.sectionId));
    return sections.filter(s => matchedIds.has(s.id));
  }, [sections, searchResults, searchQuery]);

  useEffect(() => {
    if (initialSection) {
      setActiveSection(initialSection);
      const el = document.getElementById(`doc-section-${initialSection}`);
      if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
    } else if (sections.length > 0 && !activeSection) {
      setActiveSection(sections[0].id);
    }
  }, [initialSection, sections, activeSection]);

  useEffect(() => {
    if (isSearchOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isSearchOpen]);

  const handleSectionClick = useCallback((sectionId: string) => {
    setActiveSection(sectionId);
    setSearchQuery("");
    setIsSearchOpen(false);
    onSectionChange?.(sectionId);
    const el = document.getElementById(`doc-section-${sectionId}`);
    if (el) el.scrollIntoView({ behavior: "smooth", block: "start" });
  }, [onSectionChange]);

  const handleSearchKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setIsSearchOpen(false);
      setSearchQuery("");
    }
  }, []);

  const renderSection = useCallback((section: DocSection) => {
    const html = marked.parse(section.content, { breaks: true }) as string;
    const clean = DOMPurify.sanitize(html, {
      ALLOWED_TAGS: ["p", "br", "strong", "em", "code", "pre", "ul", "ol", "li", "a", "h1", "h2", "h3", "h4", "h5", "h6", "blockquote", "hr", "table", "thead", "tbody", "tr", "th", "td", "span", "div", "img", "svg", "path", "circle", "rect", "line", "text"],
      ALLOWED_ATTR: ["href", "target", "rel", "src", "alt", "class", "style", "width", "height", "viewBox", "fill", "stroke", "strokeWidth", "d", "cx", "cy", "r", "x", "y", "rx", "ry", "xmlns", "textAnchor", "fontSize", "fontWeight"],
      ALLOW_DATA_ATTR: false,
    });
    return clean;
  }, []);

  const tocStyle: React.CSSProperties = {
    width: 220,
    flexShrink: 0,
    overflowY: "auto",
    borderRight: "1px solid var(--mac-border)",
    padding: "12px 0",
  };

  const tocItemStyle = (id: string, level: number): React.CSSProperties => ({
    padding: "5px 12px 5px " + (12 + (level - 1) * 16) + "px",
    fontSize: level === 1 ? 13 : 12,
    fontWeight: id === activeSection ? 600 : 400,
    color: id === activeSection ? "var(--mac-primary)" : "var(--mac-text-secondary)",
    cursor: "pointer",
    borderRadius: 4,
    margin: "1px 6px",
    transition: "all 0.15s ease",
    background: id === activeSection ? "var(--mac-active)" : "transparent",
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  });

  const searchItemStyle: React.CSSProperties = {
    padding: "6px 12px",
    fontSize: 12,
    color: "var(--mac-text-secondary)",
    cursor: "pointer",
    borderRadius: 4,
    margin: "1px 6px",
    transition: "background 0.15s ease",
  };

  return (
    <div className="glass-panel" style={{
      display: "flex",
      flexDirection: "column",
      overflow: "hidden",
      height: "100%",
      width: "100%",
    }}>
      <div style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "10px 16px",
        borderBottom: "1px solid var(--mac-border)",
        flexShrink: 0,
        gap: 8,
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, minWidth: 0, flex: 1 }}>
          {onClose && (
            <button className="btn-icon" onClick={onClose} title="Back" style={{ flexShrink: 0 }}>
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <path d="M10 3L5 8l5 5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </button>
          )}
          <span style={{
            fontSize: 13,
            fontWeight: 600,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}>
            {title || "Documentation"}
          </span>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 4, flexShrink: 0 }}>
          <button
            className={`btn-icon${isSearchOpen ? " active" : ""}`}
            onClick={() => setIsSearchOpen(!isSearchOpen)}
            title="Search"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <circle cx="6" cy="6" r="4.5" stroke="currentColor" strokeWidth="1.3" />
              <path d="M9.5 9.5L13 13" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
            </svg>
          </button>

          <button
            className="btn-icon"
            onClick={() => setFontSize(s => Math.min(24, s + 2))}
            title="Increase font size"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <text x="0" y="11" fontSize="12" fontWeight="bold" fill="currentColor">A</text>
              <path d="M11 5v6M8 8h6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
            </svg>
          </button>

          <button
            className="btn-icon"
            onClick={() => setFontSize(s => Math.max(10, s - 2))}
            title="Decrease font size"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <text x="0" y="11" fontSize="10" fontWeight="bold" fill="currentColor">A</text>
              <path d="M8 8h6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
            </svg>
          </button>

          {onClose && (
            <button className="btn-icon" onClick={onClose} title="Close" style={{ marginLeft: 4 }}>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </svg>
            </button>
          )}
        </div>
      </div>

      {isSearchOpen && (
        <div style={{
          padding: "8px 16px",
          borderBottom: "1px solid var(--mac-border)",
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}>
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" style={{ flexShrink: 0, color: "var(--mac-text-muted)" }}>
            <circle cx="6" cy="6" r="4.5" stroke="currentColor" strokeWidth="1.3" />
            <path d="M9.5 9.5L13 13" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
          <input
            ref={searchInputRef}
            type="text"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            onKeyDown={handleSearchKeyDown}
            placeholder="Search documentation..."
            style={{
              flex: 1,
              border: "none",
              outline: "none",
              background: "none",
              fontSize: 13,
              fontFamily: "inherit",
              color: "var(--mac-text)",
            }}
          />
          {searchQuery && (
            <button
              className="btn-icon"
              onClick={() => { setSearchQuery(""); setIsSearchOpen(false); }}
              title="Clear"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                <path d="M2 2l8 8M10 2l-8 8" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
              </svg>
            </button>
          )}
        </div>
      )}

      <div style={{ display: "flex", flex: 1, overflow: "hidden", minHeight: 0 }}>
        <div style={tocStyle}>
          {searchQuery.trim() && searchResults.length === 0 && (
            <div style={{ padding: "12px 16px", fontSize: 12, color: "var(--mac-text-muted)", textAlign: "center" }}>
              No results found
            </div>
          )}

          {searchQuery.trim() && searchResults.length > 0 && (
            <>
              <div style={{ padding: "4px 16px 8px", fontSize: 11, color: "var(--mac-text-muted)", fontWeight: 500 }}>
                {searchResults.length} result{searchResults.length !== 1 ? "s" : ""}
              </div>
              {searchResults.map(r => (
                <div
                  key={r.sectionId}
                  style={searchItemStyle}
                  onClick={() => handleSectionClick(r.sectionId)}
                  onMouseEnter={e => { (e.currentTarget as HTMLElement).style.background = "var(--mac-hover)"; }}
                  onMouseLeave={e => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
                >
                  <div style={{ fontSize: 11, fontWeight: 600, color: "var(--mac-text)", marginBottom: 2 }}>
                    {sections.find(s => s.id === r.sectionId)?.title || r.sectionId}
                  </div>
                  <div style={{ fontSize: 11, color: "var(--mac-text-muted)", lineHeight: 1.4 }}>{r.context}</div>
                </div>
              ))}
            </>
          )}

          {!searchQuery.trim() && visibleSections.map(s => (
            <div
              key={s.id}
              style={tocItemStyle(s.id, s.level)}
              onClick={() => handleSectionClick(s.id)}
              onMouseEnter={e => {
                if (s.id !== activeSection) {
                  (e.currentTarget as HTMLElement).style.background = "var(--mac-hover)";
                }
              }}
              onMouseLeave={e => {
                if (s.id !== activeSection) {
                  (e.currentTarget as HTMLElement).style.background = "transparent";
                }
              }}
            >
              {s.title}
            </div>
          ))}
        </div>

        <div
          ref={contentRef}
          style={{
            flex: 1,
            overflowY: "auto",
            padding: 24,
            fontSize,
            lineHeight: 1.7,
            color: "var(--mac-text)",
          }}
        >
          {visibleSections.length === 0 && searchQuery.trim() && (
            <div style={{ textAlign: "center", padding: 40, color: "var(--mac-text-muted)" }}>
              <svg width="32" height="32" viewBox="0 0 32 32" fill="none" style={{ marginBottom: 12, opacity: 0.4 }}>
                <circle cx="14" cy="14" r="10" stroke="currentColor" strokeWidth="1.5" />
                <path d="M21 21l8 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
              <div style={{ fontSize: 14, fontWeight: 500, marginBottom: 4 }}>No results for "{searchQuery}"</div>
              <div style={{ fontSize: 12 }}>Try different keywords or browse the table of contents</div>
            </div>
          )}

          {visibleSections.map(section => {
            const html = renderSection(section);
            const isActive = section.id === activeSection;
            return (
              <div
                key={section.id}
                id={`doc-section-${section.id}`}
                style={{
                  marginBottom: 16,
                  scrollMarginTop: 12,
                }}
              >
                <div
                  className="doc-content"
                  style={{
                    fontSize,
                    lineHeight: 1.7,
                  }}
                  dangerouslySetInnerHTML={{ __html: html }}
                />
              </div>
            );
          })}

          <style>{`
            .doc-content h1 { font-size: ${fontSize + 10}px; font-weight: 700; letter-spacing: -0.5px; margin: 24px 0 12px; padding-bottom: 8px; border-bottom: 1px solid var(--mac-border); }
            .doc-content h2 { font-size: ${fontSize + 6}px; font-weight: 600; margin: 20px 0 8px; letter-spacing: -0.3px; }
            .doc-content h3 { font-size: ${fontSize + 3}px; font-weight: 600; margin: 16px 0 6px; }
            .doc-content h4 { font-size: ${fontSize + 1}px; font-weight: 600; margin: 12px 0 4px; }
            .doc-content p { margin-bottom: 10px; }
            .doc-content code {
              background: rgba(0,0,0,0.06); padding: 1px 5px; border-radius: 3px;
              font-size: ${fontSize - 1}px; font-family: 'SF Mono', Menlo, monospace;
            }
            .doc-content pre {
              background: rgba(0,0,0,0.04); padding: 12px 16px; border-radius: 6px;
              overflow-x: auto; margin-bottom: 12px; border: 1px solid var(--mac-border);
            }
            .doc-content pre code {
              background: none; padding: 0; font-size: ${fontSize - 1}px;
              color: var(--mac-text);
            }
            .doc-content a {
              color: var(--mac-primary); text-decoration: none;
            }
            .doc-content a:hover { text-decoration: underline; }
            .doc-content blockquote {
              border-left: 3px solid var(--mac-primary); padding: 8px 16px;
              margin: 12px 0; background: var(--mac-primary-light);
              border-radius: 0 4px 4px 0;
            }
            .doc-content ul, .doc-content ol { margin: 8px 0; padding-left: 20px; }
            .doc-content li { margin-bottom: 4px; }
            .doc-content table { width: 100%; border-collapse: collapse; margin: 12px 0; font-size: ${fontSize - 1}px; }
            .doc-content th, .doc-content td {
              padding: 8px 12px; text-align: left;
              border-bottom: 1px solid var(--mac-border);
            }
            .doc-content th { font-weight: 600; background: var(--mac-hover); }
            .doc-content hr { border: none; border-top: 1px solid var(--mac-border); margin: 24px 0; }
            .doc-content img { max-width: 100%; border-radius: 6px; margin: 12px 0; }

            [data-theme="dark"] .doc-content code { background: rgba(255,255,255,0.08); }
            [data-theme="dark"] .doc-content pre { background: rgba(0,0,0,0.3); border-color: rgba(255,255,255,0.08); }
          `}</style>
        </div>
      </div>
    </div>
  );
};

export default DocViewer;
