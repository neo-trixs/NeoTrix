import React, { useState, useEffect, useCallback, useRef } from "react";
import { kbFeed, kbSearch, kbGetNode, kbGetRelated } from "../lib/api";
import type { KbSearchResult, KbNode } from "../lib/api";

const NODE_COLORS: Record<string, string> = {
  Repository: "#E85454",
  Concept: "#4A90D9",
  Paper: "#7B68EE",
  Insight: "#F5A623",
  Resource: "#50C878",
  Article: "#FF6B6B",
  CodeSnippet: "#00D68F",
  Tool: "#FFA94D",
  Person: "#A29BFE",
  default: "#888",
};

const TYPE_OPTIONS = [
  "All",
  "Repository",
  "Concept",
  "Paper",
  "Insight",
  "Resource",
  "Article",
  "CodeSnippet",
  "Tool",
];

const SORT_OPTIONS = [
  { id: "recent", label: "Recent" },
  { id: "top", label: "Top" },
  { id: "confidence", label: "Confidence" },
];

function timeAgo(ts: number): string {
  const sec = Math.floor(Date.now() / 1000 - ts);
  if (sec < 60) return `${sec}s ago`;
  if (sec < 3600) return `${Math.floor(sec / 60)}m ago`;
  if (sec < 86400) return `${Math.floor(sec / 3600)}h ago`;
  if (sec < 2592000) return `${Math.floor(sec / 86400)}d ago`;
  return `${Math.floor(sec / 2592000)}mo ago`;
}

function SkeletonCard() {
  return (
    <div
      style={{
        padding: 16,
        borderRadius: 10,
        border: "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        background: "var(--nt-glass-L2-bg, #181820)",
        display: "flex",
        flexDirection: "column",
        gap: 10,
      }}
    >
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <div
          style={{
            width: 52,
            height: 20,
            borderRadius: 10,
            background: "var(--nt-gray-300, #404048)",
            opacity: 0.3,
          }}
        />
        <div
          style={{
            flex: 1,
            height: 16,
            borderRadius: 4,
            background: "var(--nt-gray-300, #404048)",
            opacity: 0.3,
          }}
        />
      </div>
      <div
        style={{
          width: "85%",
          height: 12,
          borderRadius: 4,
          background: "var(--nt-gray-300, #404048)",
          opacity: 0.2,
        }}
      />
      <div
        style={{
          width: "60%",
          height: 12,
          borderRadius: 4,
          background: "var(--nt-gray-300, #404048)",
          opacity: 0.2,
        }}
      />
      <div
        style={{
          width: "40%",
          height: 10,
          borderRadius: 4,
          background: "var(--nt-gray-300, #404048)",
          opacity: 0.15,
        }}
      />
    </div>
  );
}

function DetailPanel({
  item,
  onClose,
}: {
  item: KbSearchResult;
  onClose: () => void;
}) {
  const [detail, setDetail] = useState<KbNode | null>(null);
  const [related, setRelated] = useState<KbSearchResult[]>([]);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const node = await kbGetNode(item.id);
        if (node) setDetail(node);
      } catch {
        /* backend not available */
      }
      try {
        const rel = await kbGetRelated(item.id);
        setRelated(rel);
      } catch {
        /* backend not available */
      }
    })();
  }, [item.id]);

  const nodeType = detail?.node_type || item.node_type;
  const color = NODE_COLORS[nodeType] || NODE_COLORS.default;

  return (
    <div
      style={{
        width: 340,
        flexShrink: 0,
        borderLeft: "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        background: "var(--nt-glass-L1-bg, #16161E)",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "14px 16px",
          borderBottom: "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        }}
      >
        <span
          style={{
            fontSize: 13,
            fontWeight: 600,
            color: "var(--nt-text, #EDEDED)",
          }}
        >
          Details
        </span>
        <button
          onClick={onClose}
          style={{
            background: "none",
            border: "none",
            color: "var(--nt-text-secondary, #808088)",
            cursor: "pointer",
            padding: 4,
            borderRadius: 4,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <svg
            viewBox="0 0 14 14"
            width={14}
            height={14}
            fill="none"
            stroke="currentColor"
            strokeWidth="1.3"
            strokeLinecap="round"
          >
            <path d="M3 3l8 8M11 3l-8 8" />
          </svg>
        </button>
      </div>
      <div
        style={{
          flex: 1,
          overflowY: "auto",
          padding: 16,
          display: "flex",
          flexDirection: "column",
          gap: 14,
        }}
      >
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <span
            style={{
              display: "inline-block",
              padding: "2px 8px",
              borderRadius: 10,
              fontSize: 10,
              fontWeight: 600,
              color: "#fff",
              background: color,
              letterSpacing: 0.3,
              textTransform: "uppercase",
              alignSelf: "flex-start",
            }}
          >
            {nodeType}
          </span>
          <span
            style={{
              fontSize: 15,
              fontWeight: 600,
              color: "var(--nt-text, #EDEDED)",
              lineHeight: 1.3,
            }}
          >
            {detail?.title || item.title}
          </span>
        </div>

        {(detail?.content || item.summary) && (
          <div
            style={{
              fontSize: 13,
              lineHeight: 1.6,
              color: "var(--nt-text-secondary, #808088)",
            }}
          >
            <span>
              {expanded
                ? (detail?.content || item.summary || "")
                : (detail?.content || item.summary || "").slice(0, 280)}
            </span>
            {(detail?.content || item.summary || "").length > 280 && (
              <button
                onClick={() => setExpanded(!expanded)}
                style={{
                  display: "block",
                  marginTop: 6,
                  background: "none",
                  border: "none",
                  color: "var(--nt-primary, #FF6B6B)",
                  cursor: "pointer",
                  fontSize: 12,
                  padding: 0,
                  fontWeight: 500,
                }}
              >
                {expanded ? "Show less" : "Show more"}
              </button>
            )}
          </div>
        )}

        {(detail?.url || item.url) && (
          <div>
            <a
              href={(detail?.url || item.url)!}
              target="_blank"
              rel="noopener noreferrer"
              style={{
                fontSize: 12,
                color: "var(--nt-primary, #FF6B6B)",
                textDecoration: "none",
                display: "flex",
                alignItems: "center",
                gap: 4,
              }}
            >
              <svg
                viewBox="0 0 12 12"
                width={12}
                height={12}
                fill="none"
                stroke="currentColor"
                strokeWidth="1.3"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M5 2H2.5a.5.5 0 00-.5.5v7a.5.5 0 00.5.5h7a.5.5 0 00.5-.5V7" />
                <path d="M7 2h3v3M9.5 2.5L6 6" />
              </svg>
              {(detail?.url || item.url)!.length > 40
                ? (detail?.url || item.url)!.slice(0, 40) + "..."
                : detail?.url || item.url}
            </a>
          </div>
        )}

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
            padding: "10px 12px",
            borderRadius: 8,
            background: "var(--nt-glass-L2-bg, #181820)",
            border: "1px solid var(--nt-border, rgba(255,255,255,0.06))",
          }}
        >
          <MetaRow
            label="Confidence"
            value={(detail?.confidence ?? item.confidence).toFixed(2)}
            bar={detail?.confidence ?? item.confidence}
            color="var(--nt-primary, #FF6B6B)"
          />
          <MetaRow
            label="Importance"
            value={(detail?.importance ?? item.importance).toFixed(2)}
            bar={detail?.importance ?? item.importance}
            color="var(--nt-yellow, #FFD54F)"
          />
          {detail?.domain && (
            <MetaRow label="Domain" value={detail.domain} />
          )}
          <MetaRow label="Created" value={timeAgo(item.created_at)} />
        </div>

        {detail?.metadata &&
          Object.keys(detail.metadata).length > 0 && (
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              <span
                style={{
                  fontSize: 11,
                  fontWeight: 600,
                  color: "var(--nt-text-secondary, #808088)",
                  textTransform: "uppercase",
                  letterSpacing: 0.5,
                }}
              >
                Metadata
              </span>
              {Object.entries(detail.metadata).map(([key, val]) => (
                <div
                  key={key}
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    fontSize: 12,
                    color: "var(--nt-text-secondary, #808088)",
                    padding: "2px 0",
                  }}
                >
                  <span>{key}</span>
                  <span style={{ color: "var(--nt-text, #EDEDED)" }}>
                    {String(val)}
                  </span>
                </div>
              ))}
            </div>
          )}

        {related.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            <span
              style={{
                fontSize: 11,
                fontWeight: 600,
                color: "var(--nt-text-secondary, #808088)",
                textTransform: "uppercase",
                letterSpacing: 0.5,
              }}
            >
              Related ({related.length})
            </span>
            {related.map((r) => (
              <div
                key={r.id}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  padding: "8px 10px",
                  borderRadius: 6,
                  background: "var(--nt-glass-L2-bg, #181820)",
                  border:
                    "1px solid var(--nt-border, rgba(255,255,255,0.04))",
                  cursor: "pointer",
                }}
              >
                <div
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: "50%",
                    background:
                      NODE_COLORS[r.node_type] || NODE_COLORS.default,
                    flexShrink: 0,
                  }}
                />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      fontSize: 12,
                      fontWeight: 500,
                      color: "var(--nt-text, #EDEDED)",
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {r.title}
                  </div>
                  <div
                    style={{
                      fontSize: 10,
                      color: "var(--nt-text-muted, #505058)",
                    }}
                  >
                    {r.node_type}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function MetaRow({
  label,
  value,
  bar,
  color,
}: {
  label: string;
  value: string;
  bar?: number;
  color?: string;
}) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          fontSize: 11,
          color: "var(--nt-text-muted, #505058)",
        }}
      >
        <span>{label}</span>
        <span style={{ color: "var(--nt-text-secondary, #808088)" }}>
          {value}
        </span>
      </div>
      {bar !== undefined && (
        <div
          style={{
            height: 3,
            borderRadius: 2,
            background: "var(--nt-gray-300, #404048)",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              width: `${Math.round(bar * 100)}%`,
              height: "100%",
              borderRadius: 2,
              background: color || "var(--nt-primary, #FF6B6B)",
              transition: "width 0.3s ease",
            }}
          />
        </div>
      )}
    </div>
  );
}

function FeedCard({
  item,
  onClick,
}: {
  item: KbSearchResult;
  onClick: () => void;
}) {
  const color = NODE_COLORS[item.node_type] || NODE_COLORS.default;

  return (
    <div
      onClick={onClick}
      style={{
        padding: "14px 16px",
        borderRadius: 10,
        border: "1px solid var(--nt-border, rgba(255,255,255,0.06))",
        background: "var(--nt-glass-L2-bg, #181820)",
        cursor: "pointer",
        display: "flex",
        flexDirection: "column",
        gap: 8,
        transition: "border-color 0.15s, box-shadow 0.15s",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor =
          "var(--nt-border-strong, rgba(255,255,255,0.10))";
        e.currentTarget.style.boxShadow =
          "var(--nt-shadow-md, 0 4px 16px rgba(0,0,0,0.35))";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor =
          "var(--nt-border, rgba(255,255,255,0.06))";
        e.currentTarget.style.boxShadow = "none";
      }}
    >
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <span
          style={{
            display: "inline-block",
            padding: "2px 8px",
            borderRadius: 10,
            fontSize: 10,
            fontWeight: 600,
            color: "#fff",
            background: color,
            letterSpacing: 0.3,
            textTransform: "uppercase",
            whiteSpace: "nowrap",
            flexShrink: 0,
          }}
        >
          {item.node_type}
        </span>
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "var(--nt-text, #EDEDED)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          {item.title}
        </span>
      </div>

      {(item.summary || item.content) && (
        <span
          style={{
            fontSize: 13,
            lineHeight: 1.5,
            color: "var(--nt-text-secondary, #808088)",
            display: "-webkit-box",
            WebkitLineClamp: 2,
            WebkitBoxOrient: "vertical",
            overflow: "hidden",
          }}
        >
          {(item.summary || item.content || "").slice(0, 200)}
        </span>
      )}

      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          marginTop: 2,
        }}
      >
        {item.domain && (
          <span
            style={{
              fontSize: 11,
              color: "var(--nt-text-muted, #505058)",
              display: "flex",
              alignItems: "center",
              gap: 4,
            }}
          >
            <svg
              viewBox="0 0 10 10"
              width={10}
              height={10}
              fill="none"
              stroke="currentColor"
              strokeWidth="1.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="5" cy="5" r="4" />
              <path d="M5 1v8M1 5h8" />
            </svg>
            {item.domain.replace(/^https?:\/\//, "").split("/")[0]}
          </span>
        )}

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            flex: 1,
            maxWidth: 120,
          }}
        >
          <div
            style={{
              flex: 1,
              height: 3,
              borderRadius: 2,
              background: "var(--nt-gray-300, #404048)",
              overflow: "hidden",
            }}
          >
            <div
              style={{
                width: `${Math.round(item.confidence * 100)}%`,
                height: "100%",
                borderRadius: 2,
                background: color,
              }}
            />
          </div>
          <span
            style={{
              fontSize: 10,
              color: "var(--nt-text-muted, #505058)",
              minWidth: 30,
            }}
          >
            {Math.round(item.confidence * 100)}%
          </span>
        </div>

        <span
          style={{
            fontSize: 11,
            color: "var(--nt-text-muted, #505058)",
            marginLeft: "auto",
            whiteSpace: "nowrap",
          }}
        >
          {timeAgo(item.created_at)}
        </span>
      </div>
    </div>
  );
}

const ExplorePage: React.FC = () => {
  const [items, setItems] = useState<KbSearchResult[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState("recent");
  const [typeFilter, setTypeFilter] = useState("All");
  const [searchQuery, setSearchQuery] = useState("");
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);
  const [allCount, setAllCount] = useState(0);
  const [selectedItem, setSelectedItem] = useState<KbSearchResult | null>(
    null
  );
  const sentinelRef = useRef<HTMLDivElement>(null);
  const LIMIT = 25;

  const loadFeed = useCallback(
    async (newOffset: number, append: boolean) => {
      if (!append) setLoading(true);
      else setLoadingMore(true);
      setError(null);
      try {
        const res = await kbFeed(LIMIT, newOffset, sortBy);
        if (res.length > 0) {
          setItems((prev) => (append ? [...prev, ...res] : res));
          setOffset(newOffset + res.length);
          setHasMore(res.length >= LIMIT);
          setAllCount((prev) => (append ? prev : res.length));
        } else {
          if (!append) setItems([]);
          setHasMore(false);
        }
      } catch (e) {
        setError("Failed to load feed");
        console.error(e);
      } finally {
        setLoading(false);
        setLoadingMore(false);
      }
    },
    [sortBy]
  );

  useEffect(() => {
    setItems([]);
    setOffset(0);
    setHasMore(true);
    loadFeed(0, false);
  }, [loadFeed]);

  useEffect(() => {
    if (!sentinelRef.current || !hasMore || loading || loadingMore) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !loadingMore) {
          loadFeed(offset, true);
        }
      },
      { rootMargin: "200px" }
    );
    observer.observe(sentinelRef.current);
    return () => observer.disconnect();
  }, [hasMore, loading, loadingMore, offset, loadFeed]);

  // Client-side filtering
  const filtered = items.filter((n) => {
    if (typeFilter !== "All" && n.node_type !== typeFilter) return false;
    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      return (
        n.title.toLowerCase().includes(q) ||
        (n.summary && n.summary.toLowerCase().includes(q)) ||
        (n.content && n.content.toLowerCase().includes(q))
      );
    }
    return true;
  });

  const typeSet = new Set(items.map((i) => i.node_type));

  return (
    <div
      style={{
        display: "flex",
        height: "100%",
        background: "var(--nt-bg, #181820)",
        color: "var(--nt-text, #EDEDED)",
        fontFamily: "var(--nt-font-family, system-ui)",
      }}
    >
      <div
        style={{
          flex: 1,
          display: "flex",
          flexDirection: "column",
          minWidth: 0,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 8,
            padding: "8px 20px",
            borderBottom:
              "1px solid var(--nt-border, rgba(255,255,255,0.06))",
            background: "var(--nt-glass-L1-bg, #16161E)",
            fontSize: 12,
            color: "var(--nt-text-secondary, #808088)",
          }}
        >
          <span style={{ fontWeight: 500 }}>
            <span style={{ color: "var(--nt-text, #EDEDED)" }}>
              {allCount.toLocaleString()}
            </span>{" "}
            nodes
          </span>
          <span>·</span>
          <span>{typeSet.size} types</span>
          <span>·</span>
          <span>
            showing{" "}
            <span style={{ color: "var(--nt-text, #EDEDED)" }}>
              {filtered.length}
            </span>
          </span>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            padding: "12px 20px",
            borderBottom:
              "1px solid var(--nt-border, rgba(255,255,255,0.06))",
            background: "var(--nt-glass-L1-bg, #16161E)",
            flexWrap: "wrap",
          }}
        >
          <div
            style={{
              display: "flex",
              gap: 0,
              borderRadius: 6,
              overflow: "hidden",
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
            }}
          >
            {SORT_OPTIONS.map((s) => (
              <button
                key={s.id}
                onClick={() => setSortBy(s.id)}
                style={{
                  padding: "5px 14px",
                  fontSize: 12,
                  fontWeight: sortBy === s.id ? 600 : 400,
                  border: "none",
                  background:
                    sortBy === s.id
                      ? "var(--nt-primary, #FF6B6B)"
                      : "transparent",
                  color:
                    sortBy === s.id
                      ? "#fff"
                      : "var(--nt-text-secondary, #808088)",
                  cursor: "pointer",
                  transition: "background 0.15s, color 0.15s",
                }}
              >
                {s.label}
              </button>
            ))}
          </div>

          <div
            style={{
              width: 1,
              height: 20,
              background:
                "var(--nt-border, rgba(255,255,255,0.06))",
            }}
          />

          <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
            {TYPE_OPTIONS.slice(0, 6).map((t) => (
              <button
                key={t}
                onClick={() => setTypeFilter(t)}
                style={{
                  padding: "3px 10px",
                  borderRadius: 12,
                  fontSize: 11,
                  fontWeight: typeFilter === t ? 600 : 400,
                  border: `1px solid ${
                    typeFilter === t
                      ? "var(--nt-primary, #FF6B6B)"
                      : "var(--nt-border, rgba(255,255,255,0.06))"
                  }`,
                  background:
                    typeFilter === t
                      ? "var(--nt-primary-light-bg, rgba(232,84,84,0.10))"
                      : "transparent",
                  color:
                    typeFilter === t
                      ? "var(--nt-primary, #FF6B6B)"
                      : "var(--nt-text-secondary, #808088)",
                  cursor: "pointer",
                  transition: "all 0.15s",
                }}
              >
                {t}
              </button>
            ))}
            {TYPE_OPTIONS.length > 6 && (
              <select
                value={typeFilter}
                onChange={(e) => setTypeFilter(e.target.value)}
                style={{
                  padding: "3px 8px",
                  borderRadius: 12,
                  fontSize: 11,
                  border:
                    "1px solid var(--nt-border, rgba(255,255,255,0.06))",
                  background: "transparent",
                  color: "var(--nt-text-secondary, #808088)",
                  cursor: "pointer",
                  outline: "none",
                }}
              >
                {TYPE_OPTIONS.slice(6).map((t) => (
                  <option key={t} value={t}>
                    {t}
                  </option>
                ))}
              </select>
            )}
          </div>

          <div style={{ flex: 1 }} />

          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 6,
              padding: "5px 12px",
              borderRadius: 8,
              border:
                "1px solid var(--nt-border, rgba(255,255,255,0.06))",
              background: "var(--nt-glass-L2-bg, #181820)",
            }}
          >
            <svg
              viewBox="0 0 14 14"
              width={14}
              height={14}
              fill="none"
              stroke="var(--nt-text-muted, #505058)"
              strokeWidth="1.3"
              strokeLinecap="round"
            >
              <circle cx="6.5" cy="6.5" r="3.5" />
              <path d="M9 9l3.5 3.5" />
            </svg>
            <input
              type="text"
              placeholder="Search feed..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              style={{
                border: "none",
                background: "none",
                outline: "none",
                color: "var(--nt-text, #EDEDED)",
                fontSize: 12,
                width: 180,
              }}
            />
          </div>
        </div>

        <div
          style={{
            flex: 1,
            overflowY: "auto",
            padding: "12px 20px",
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          {loading && items.length === 0 && (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              {[1, 2, 3].map((i) => (
                <SkeletonCard key={i} />
              ))}
            </div>
          )}

          {error && (
            <div
              style={{
                flex: 1,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: 12,
                color: "var(--nt-text-secondary, #808088)",
                padding: 40,
              }}
            >
              <svg
                viewBox="0 0 32 32"
                width={32}
                height={32}
                fill="none"
                stroke="var(--nt-text-muted, #505058)"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="16" cy="16" r="12" />
                <path d="M16 10v6M16 20v.01" />
              </svg>
              <span style={{ fontSize: 14 }}>{error}</span>
              <button
                onClick={() => loadFeed(0, false)}
                style={{
                  padding: "6px 16px",
                  borderRadius: 6,
                  border: "1px solid var(--nt-primary, #FF6B6B)",
                  background: "transparent",
                  color: "var(--nt-primary, #FF6B6B)",
                  cursor: "pointer",
                  fontSize: 12,
                  fontWeight: 500,
                }}
              >
                Retry
              </button>
            </div>
          )}

          {!loading && !error && filtered.length === 0 && (
            <div
              style={{
                flex: 1,
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: 16,
                color: "var(--nt-text-secondary, #808088)",
                padding: 40,
              }}
            >
              <svg
                viewBox="0 0 48 48"
                width={48}
                height={48}
                fill="none"
                stroke="var(--nt-text-muted, #505058)"
                strokeWidth="1.2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="24" cy="24" r="18" opacity={0.3} />
                <circle cx="24" cy="24" r="10" opacity={0.2} />
                <circle cx="24" cy="24" r="4" opacity={0.15} />
                <path d="M24 6v4M24 38v4M6 24h4M38 24h4" opacity={0.3} />
              </svg>
              <span style={{ fontSize: 16, fontWeight: 500 }}>
                No results found
              </span>
              <span
                style={{
                  fontSize: 13,
                  color: "var(--nt-text-muted, #505058)",
                  textAlign: "center",
                  maxWidth: 300,
                }}
              >
                Try adjusting your filters or search query to discover
                knowledge nodes
              </span>
            </div>
          )}

          {!loading &&
            filtered.map((item) => (
              <FeedCard
                key={item.id}
                item={item}
                onClick={() => setSelectedItem(item)}
              />
            ))}

          <div ref={sentinelRef} style={{ height: 1 }} />

          {loadingMore && (
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 8,
              }}
            >
              {[1, 2].map((i) => (
                <SkeletonCard key={`more-${i}`} />
              ))}
            </div>
          )}

          {!hasMore && items.length > 0 && (
            <div
              style={{
                textAlign: "center",
                padding: "16px 0",
                fontSize: 12,
                color: "var(--nt-text-muted, #505058)",
              }}
            >
              — You've reached the end —
            </div>
          )}
        </div>
      </div>

      {selectedItem && (
        <DetailPanel
          key={selectedItem.id}
          item={selectedItem}
          onClose={() => setSelectedItem(null)}
        />
      )}
    </div>
  );
};

export default ExplorePage;
