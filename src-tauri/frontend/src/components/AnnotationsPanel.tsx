import React, { useState, useEffect, useCallback } from "react";
import type { PageAnnotation } from "../commands";
import {
  annotationList,
  annotationCreate,
  annotationDelete,
  annotationResolve,
  annotationUnresolve,
  annotationSearch,
  annotationStats,
} from "../commands";

const TYPE_COLORS: Record<string, string> = {
  highlight: "#3b82f6",
  comment: "#6b7280",
  question: "#eab308",
  task: "#f97316",
  issue: "#ef4444",
};

const TYPE_BADGE_BG: Record<string, string> = {
  highlight: "rgba(59,130,246,0.15)",
  comment: "rgba(107,114,128,0.15)",
  question: "rgba(234,179,8,0.15)",
  task: "rgba(249,115,22,0.15)",
  issue: "rgba(239,68,68,0.15)",
};

function formatTime(iso: string): string {
  const d = new Date(iso);
  const now = Date.now();
  const diff = now - d.getTime();
  if (diff < 60000) return "just now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
  return d.toLocaleDateString();
}

const containerStyle: React.CSSProperties = {
  height: "100%",
  display: "flex",
  flexDirection: "column",
  background: "#0a0a0f",
  color: "#e0e0e0",
  fontFamily: "'SF Mono', 'Cascadia Code', 'Fira Code', monospace",
  fontSize: 13,
  overflow: "hidden",
};

const scrollStyle: React.CSSProperties = {
  flex: 1,
  overflowY: "auto",
  padding: 16,
};

const headerStyle: React.CSSProperties = {
  padding: "12px 16px",
  borderBottom: "1px solid #1a1a2e",
  display: "flex",
  alignItems: "center",
  gap: 8,
};

const inputStyle: React.CSSProperties = {
  background: "#12121f",
  border: "1px solid #2a2a3e",
  borderRadius: 6,
  padding: "6px 10px",
  color: "#e0e0e0",
  fontSize: 12,
  fontFamily: "inherit",
  outline: "none",
  width: "100%",
};

const btnStyle: React.CSSProperties = {
  background: "#1a1a2e",
  border: "1px solid #2a2a3e",
  borderRadius: 6,
  padding: "6px 12px",
  color: "#c0c0c0",
  fontSize: 12,
  fontFamily: "inherit",
  cursor: "pointer",
};

const primaryBtnStyle: React.CSSProperties = {
  ...btnStyle,
  background: "#3b82f6",
  border: "1px solid #3b82f6",
  color: "#fff",
};

const dangerBtnStyle: React.CSSProperties = {
  ...btnStyle,
  border: "1px solid #ef4444",
  color: "#ef4444",
};

const cardStyle: React.CSSProperties = {
  background: "#12121f",
  border: "1px solid #1a1a2e",
  borderRadius: 8,
  padding: 12,
  marginBottom: 8,
};

const badgeStyle = (color: string): React.CSSProperties => ({
  display: "inline-flex",
  alignItems: "center",
  padding: "2px 8px",
  borderRadius: 4,
  fontSize: 11,
  fontWeight: 600,
  background: TYPE_BADGE_BG[color] || "rgba(107,114,128,0.15)",
  color: TYPE_COLORS[color] || "#6b7280",
  textTransform: "uppercase" as const,
  letterSpacing: "0.5px",
});

const AnnotationCard: React.FC<{
  a: PageAnnotation;
  onResolve: (id: string) => void;
  onDelete: (id: string) => void;
}> = ({ a, onResolve, onDelete }) => {
  const [deleting, setDeleting] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const handleDelete = () => {
    if (!confirmDelete) {
      setConfirmDelete(true);
      return;
    }
    setDeleting(true);
    onDelete(a.id);
  };

  return (
    <div style={cardStyle}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", marginBottom: 6 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, flex: 1, minWidth: 0 }}>
          <span style={badgeStyle(a.annotation_type)}>{a.annotation_type}</span>
          {a.resolved ? (
            <span style={{ fontSize: 11, color: "#4ade80" }}>✅ Resolved</span>
          ) : (
            <span style={{ fontSize: 11, color: "#fbbf24" }}>◉ Unresolved</span>
          )}
        </div>
        <div style={{ display: "flex", gap: 4, flexShrink: 0 }}>
          {!a.resolved ? (
            <button style={{ ...btnStyle, padding: "3px 8px", fontSize: 11 }} onClick={() => onResolve(a.id)}>
              Resolve
            </button>
          ) : (
            <button style={{ ...btnStyle, padding: "3px 8px", fontSize: 11 }} onClick={() => onResolve(a.id)}>
              Unresolve
            </button>
          )}
          <button
            style={{
              ...(confirmDelete ? dangerBtnStyle : { ...btnStyle, padding: "3px 8px", fontSize: 11 }),
              padding: "3px 8px",
              fontSize: 11,
            }}
            onClick={handleDelete}
            disabled={deleting}
          >
            {confirmDelete ? "Confirm?" : "Delete"}
          </button>
        </div>
      </div>

      {a.page_title && (
        <div style={{ fontSize: 13, fontWeight: 600, color: "#e0e0e0", marginBottom: 4 }}>
          {a.page_title}
        </div>
      )}

      <div style={{ fontSize: 11, color: "#6b7280", marginBottom: 6 }}>
        {a.url}
      </div>

      {a.highlighted_text && (
        <div style={{
          background: "rgba(59,130,246,0.08)",
          borderLeft: "3px solid #3b82f6",
          padding: "6px 10px",
          borderRadius: 4,
          fontSize: 12,
          color: "#94a3b8",
          fontStyle: "italic",
          marginBottom: 6,
          wordBreak: "break-word",
        }}>
          "{a.highlighted_text}"
        </div>
      )}

      <div style={{ fontSize: 12, color: "#c0c0c0", lineHeight: 1.5, marginBottom: 6, whiteSpace: "pre-wrap", wordBreak: "break-word" }}>
        {a.comment}
      </div>

      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", fontSize: 11, color: "#6b7280" }}>
        <div style={{ display: "flex", gap: 8 }}>
          <span>@{a.author}</span>
          <span>{formatTime(a.created_at)}</span>
        </div>
        <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
          {a.tags.map((tag) => (
            <span key={tag} style={{
              background: "rgba(139,92,246,0.12)",
              color: "#a78bfa",
              padding: "1px 6px",
              borderRadius: 3,
              fontSize: 10,
            }}>
              #{tag}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
};

const AnnotationsPanel: React.FC = () => {
  // Data
  const [annotations, setAnnotations] = useState<PageAnnotation[]>([]);
  const [stats, setStats] = useState<{
    total_annotations: number;
    unresolved: number;
    resolved_today: number;
    collections: number;
    urls_tracked: number;
    top_tags: [string, number][];
  } | null>(null);
  const [loading, setLoading] = useState(true);

  // Filters
  const [urlFilter, setUrlFilter] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [resolvedFilter, setResolvedFilter] = useState<"all" | "unresolved" | "resolved">("all");

  // Create form
  const [showForm, setShowForm] = useState(false);
  const [formUrl, setFormUrl] = useState("");
  const [formTitle, setFormTitle] = useState("");
  const [formSelector, setFormSelector] = useState("");
  const [formHighlight, setFormHighlight] = useState("");
  const [formComment, setFormComment] = useState("");
  const [formType, setFormType] = useState("comment");
  const [formTags, setFormTags] = useState("");

  // Collections
  const [collections, setCollections] = useState<{ name: string; count: number }[]>([]);
  const [collectionName, setCollectionName] = useState("");
  const [showCollectionForm, setShowCollectionForm] = useState(false);

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      let result: PageAnnotation[];
      if (searchQuery.trim()) {
        result = await annotationSearch(searchQuery.trim());
      } else if (urlFilter.trim()) {
        result = await annotationList(urlFilter.trim(), undefined, 1);
      } else {
        result = await annotationList(undefined, undefined, 1);
      }

      if (resolvedFilter === "unresolved") {
        result = result.filter((a) => !a.resolved);
      } else if (resolvedFilter === "resolved") {
        result = result.filter((a) => a.resolved);
      }

      setAnnotations(result);
    } catch {
      setAnnotations([]);
    }

    try {
      const s = await annotationStats();
      setStats(s);
      setCollections([]);
    } catch {
      setStats(null);
    }

    setLoading(false);
  }, [urlFilter, searchQuery, resolvedFilter]);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleCreate = async () => {
    if (!formUrl.trim() || !formComment.trim()) return;
    const tags = formTags.trim() ? formTags.split(",").map((t) => t.trim()).filter(Boolean) : undefined;
    try {
      await annotationCreate(
        formUrl.trim(),
        formTitle.trim() || formUrl.trim(),
        formSelector.trim(),
        formHighlight.trim(),
        formComment.trim(),
        formType,
        tags,
      );
      setFormUrl("");
      setFormTitle("");
      setFormSelector("");
      setFormHighlight("");
      setFormComment("");
      setFormTags("");
      setShowForm(false);
      loadData();
    } catch (err) {
      console.error("create annotation failed", err);
    }
  };

  const handleResolve = async (id: string) => {
    const a = annotations.find((x) => x.id === id);
    if (!a) return;
    try {
      if (a.resolved) {
        await annotationUnresolve(id);
      } else {
        await annotationResolve(id);
      }
      loadData();
    } catch (err) {
      console.error("resolve/unresolve failed", err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await annotationDelete(id);
      loadData();
    } catch (err) {
      console.error("delete failed", err);
    }
  };

  const handleCreateCollection = async () => {
    if (!collectionName.trim()) return;
    const ids = annotations.filter((a) => !a.resolved).map((a) => a.id);
    if (ids.length === 0) return;
    try {
      setCollections((prev) => [...prev, { name: collectionName.trim(), count: ids.length }]);
      setCollectionName("");
      setShowCollectionForm(false);
    } catch (err) {
      console.error("create collection failed", err);
    }
  };

  // Group by URL
  const grouped = annotations.reduce<Record<string, PageAnnotation[]>>((acc, a) => {
    const key = a.url || "unknown";
    if (!acc[key]) acc[key] = [];
    acc[key].push(a);
    return acc;
  }, {});

  return (
    <div style={containerStyle}>
      {/* Header */}
      <div style={headerStyle}>
        <span style={{ fontSize: 14 }}>💬</span>
        <span style={{ fontWeight: 600, fontSize: 14 }}>Annotations</span>
        <div style={{ flex: 1 }} />
        <button style={primaryBtnStyle} onClick={() => setShowForm(!showForm)}>
          {showForm ? "✕ Close" : "+ New"}
        </button>
      </div>

      <div style={scrollStyle}>
        {/* Stats bar */}
        {stats && (
          <div style={{
            display: "flex",
            gap: 16,
            padding: "8px 12px",
            background: "#12121f",
            border: "1px solid #1a1a2e",
            borderRadius: 8,
            marginBottom: 12,
            flexWrap: "wrap",
            fontSize: 12,
          }}>
            <span>Total: <strong style={{ color: "#e0e0e0" }}>{stats.total_annotations}</strong></span>
            <span>Unresolved: <strong style={{ color: "#fbbf24" }}>{stats.unresolved}</strong></span>
            <span>Resolved today: <strong style={{ color: "#4ade80" }}>{stats.resolved_today}</strong></span>
            <span>URLs: <strong style={{ color: "#60a5fa" }}>{stats.urls_tracked}</strong></span>
            <span>Collections: <strong style={{ color: "#a78bfa" }}>{stats.collections}</strong></span>
            {stats.top_tags.length > 0 && (
              <span>
                Top tags:{" "}
                {stats.top_tags.slice(0, 3).map(([tag, count]) => (
                  <span key={tag} style={{ color: "#a78bfa", marginRight: 4 }}>
                    #{tag}({count})
                  </span>
                ))}
              </span>
            )}
          </div>
        )}

        {/* Search & filter */}
        <div style={{ display: "flex", gap: 8, marginBottom: 12, flexWrap: "wrap" }}>
          <div style={{ flex: 1, minWidth: 180 }}>
            <input
              style={inputStyle}
              placeholder="Search across all annotations..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <div style={{ width: 200 }}>
            <input
              style={inputStyle}
              placeholder="Filter by URL..."
              value={urlFilter}
              onChange={(e) => setUrlFilter(e.target.value)}
            />
          </div>
          <select
            style={{ ...inputStyle, width: 140, cursor: "pointer" }}
            value={resolvedFilter}
            onChange={(e) => setResolvedFilter(e.target.value as "all" | "unresolved" | "resolved")}
          >
            <option value="all">All</option>
            <option value="unresolved">Unresolved</option>
            <option value="resolved">Resolved</option>
          </select>
        </div>

        {/* Create form */}
        {showForm && (
          <div style={{ ...cardStyle, marginBottom: 12 }}>
            <div style={{ fontSize: 12, fontWeight: 600, color: "#c0c0c0", marginBottom: 8 }}>
              ✏️ New Annotation
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
              <input style={inputStyle} placeholder="URL *" value={formUrl} onChange={(e) => setFormUrl(e.target.value)} />
              <input style={inputStyle} placeholder="Page title" value={formTitle} onChange={(e) => setFormTitle(e.target.value)} />
              <div style={{ display: "flex", gap: 6 }}>
                <input style={{ ...inputStyle, flex: 1 }} placeholder="CSS selector" value={formSelector} onChange={(e) => setFormSelector(e.target.value)} />
                <select style={{ ...inputStyle, width: 130, cursor: "pointer" }} value={formType} onChange={(e) => setFormType(e.target.value)}>
                  <option value="comment">💬 Comment</option>
                  <option value="highlight">🔵 Highlight</option>
                  <option value="question">❓ Question</option>
                  <option value="task">📋 Task</option>
                  <option value="issue">🚨 Issue</option>
                </select>
              </div>
              <textarea
                style={{ ...inputStyle, minHeight: 60, resize: "vertical" }}
                placeholder="Highlighted text (quoted)"
                value={formHighlight}
                onChange={(e) => setFormHighlight(e.target.value)}
              />
              <textarea
                style={{ ...inputStyle, minHeight: 60, resize: "vertical" }}
                placeholder="Your comment *"
                value={formComment}
                onChange={(e) => setFormComment(e.target.value)}
              />
              <input style={inputStyle} placeholder="Tags (comma-separated)" value={formTags} onChange={(e) => setFormTags(e.target.value)} />
              <div style={{ display: "flex", gap: 6, justifyContent: "flex-end" }}>
                <button style={btnStyle} onClick={() => setShowForm(false)}>Cancel</button>
                <button style={primaryBtnStyle} onClick={handleCreate}>Create</button>
              </div>
            </div>
          </div>
        )}

        {/* Collection management */}
        <div style={{ display: "flex", gap: 8, marginBottom: 12, flexWrap: "wrap", alignItems: "center" }}>
          <button style={btnStyle} onClick={() => setShowCollectionForm(!showCollectionForm)}>
            📁 {showCollectionForm ? "Cancel" : "New Collection"}
          </button>
          {showCollectionForm && (
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <input
                style={{ ...inputStyle, width: 200 }}
                placeholder="Collection name"
                value={collectionName}
                onChange={(e) => setCollectionName(e.target.value)}
              />
              <button style={primaryBtnStyle} onClick={handleCreateCollection}>Create</button>
            </div>
          )}
          {collections.length > 0 && (
            <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
              {collections.map((c, i) => (
                <span key={i} style={{
                  background: "rgba(139,92,246,0.12)",
                  color: "#a78bfa",
                  padding: "2px 8px",
                  borderRadius: 4,
                  fontSize: 11,
                }}>
                  📁 {c.name} ({c.count})
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Annotations list */}
        {loading ? (
          <div style={{ textAlign: "center", padding: 40, color: "#6b7280" }}>Loading annotations...</div>
        ) : Object.keys(grouped).length === 0 ? (
          <div style={{ textAlign: "center", padding: 40, color: "#6b7280" }}>
            No annotations found.
          </div>
        ) : (
          Object.entries(grouped).map(([url, items]) => (
            <div key={url} style={{ marginBottom: 16 }}>
              <div style={{
                display: "flex",
                alignItems: "center",
                gap: 8,
                padding: "6px 0",
                borderBottom: "1px solid #1a1a2e",
                marginBottom: 8,
              }}>
                <span style={{ fontSize: 10, color: "#6b7280" }}>🔗</span>
                <span style={{ fontSize: 12, fontWeight: 600, color: "#60a5fa", wordBreak: "break-all" }}>
                  {url}
                </span>
                <span style={{ fontSize: 11, color: "#6b7280" }}>({items.length})</span>
              </div>
              {items.map((a) => (
                <AnnotationCard key={a.id} a={a} onResolve={handleResolve} onDelete={handleDelete} />
              ))}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default AnnotationsPanel;
