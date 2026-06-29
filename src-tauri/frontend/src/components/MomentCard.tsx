import React, { useState } from "react";
import type { FeedItem } from "../types";
import * as api from "../lib/api";

interface Props {
  item: FeedItem;
  onInsight?: (id: string) => void;
}

function timeAgo(ts: number): string {
  const diff = Date.now() - ts * 1000;
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "刚刚";
  if (mins < 60) return `${mins}分钟前`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}小时前`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}天前`;
  const weeks = Math.floor(days / 7);
  return `${weeks}周前`;
}

const typeIcon: Record<string, string> = {
  article: "📄",
  image: "🖼",
  video: "🎬",
  live: "🔴",
  social: "💬",
};

const MomentCard: React.FC<Props> = ({ item, onInsight }) => {
  const [expanded, setExpanded] = useState(false);
  const [insight, setInsight] = useState<string | null>(item.neotrix_insight || null);
  const [loadingInsight, setLoadingInsight] = useState(false);

  const handleOpen = () => {
    if (item.source_url) {
      window.open(item.source_url, "_blank", "noopener,noreferrer");
    }
  };

  const handleInsight = async () => {
    if (insight) return;
    setLoadingInsight(true);
    try {
      const result = await api.feedInsight(item.id);
      setInsight(result);
      onInsight?.(item.id);
    } catch {
      setInsight("暂时无法获取深度分析");
    }
    setLoadingInsight(false);
  };

  return (
    <div className="moment-card" onClick={() => setExpanded(!expanded)}>
      <div className={`moment-card-type-badge ${item.content_type}`}>
        {typeIcon[item.content_type ?? "article"] || "📄"} {item.content_type}
      </div>

      <div className="moment-card-source">
        <span>{item.source_name}</span>
        <span>·</span>
        <span>{timeAgo(item.published_at ?? 0)}</span>
      </div>

      <div className="moment-card-title" onClick={(e) => { e.stopPropagation(); handleOpen(); }}>
        {item.title}
      </div>

      <div className={`moment-card-desc ${expanded ? "expanded" : ""}`}>
        {item.description}
      </div>
      {(item.description?.length ?? 0) > 120 && !expanded && (
        <span className="moment-card-expand" onClick={(e) => { e.stopPropagation(); setExpanded(true); }}>
          展开
        </span>
      )}

      {item.image_url && (
        <img
          className="moment-card-image"
          src={item.image_url}
          alt={item.title}
          loading="lazy"
          onClick={(e) => { e.stopPropagation(); handleOpen(); }}
          onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
        />
      )}

      {(item.tags?.length ?? 0) > 0 && (
        <div className="moment-card-tags">
          {item.tags?.map((tag) => (
            <span key={tag} className="moment-card-tag">{tag}</span>
          ))}
        </div>
      )}

      {!insight && !loadingInsight && (
        <button className="moment-insight-btn" onClick={(e) => { e.stopPropagation(); handleInsight(); }}>
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5">
            <circle cx="8" cy="8" r="6" />
            <path d="M8 5.5v3M8 11v.5" strokeLinecap="round" />
          </svg>
          AI 深度分析
        </button>
      )}

      {loadingInsight && (
        <div className="moment-insight-loading">
          <span className="loading-spinner-sm" />
          分析中...
        </div>
      )}

      {insight && (
        <div className="moment-insight-box">
          <div className="moment-insight-label">NeoTrix 洞察</div>
          {insight}
        </div>
      )}

      <div className="moment-card-heat" style={{ width: `${Math.min((item.score ?? 0) * 10, 100)}%` }} />
    </div>
  );
};

export default MomentCard;
