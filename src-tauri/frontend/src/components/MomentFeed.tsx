import React, { useEffect, useState, useCallback, useRef } from "react";
import { useStore } from "../store";
import * as api from "../lib/api";
import { listen } from "@tauri-apps/api/event";
import MomentCard from "./MomentCard";
import type { FeedItem, FeedTag, FeedState } from "../types";

const DEFAULT_TAGS = ["AI", "科技", "科学", "开源", "商业", "社会", "设计", "宇宙", "生物", "政治"];

const MomentFeed: React.FC = () => {
  const feedState = useStore((s) => s.feedState);
  const feedLoading = useStore((s) => s.feedLoading);
  const setFeedState = useStore((s) => s.setFeedState);
  const setFeedLoading = useStore((s) => s.setFeedLoading);
  const setMomentFeedVisible = useStore((s) => s.setMomentFeedVisible);

  const [activeTag, setActiveTag] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [tags, setTags] = useState<FeedTag[]>([]);
  const [timelinesExpanded, setTimelinesExpanded] = useState(false);
  const searchTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const loadFeed = useCallback(async (tag?: string | null, search?: string) => {
    setFeedLoading(true);
    try {
      const result = await api.feedRefresh(tag ?? undefined, search || undefined);
      setFeedState(result);
      if (result.tags.length > 0) {
        setTags(result.tags);
      }
    } catch (e) {
      console.error("Feed refresh failed:", e);
      setFeedState(null);
    }
    setFeedLoading(false);
  }, [setFeedState, setFeedLoading]);

  useEffect(() => {
    loadFeed(null, undefined);
  }, []);

  useEffect(() => {
    const unlisten = listen<FeedState>("feed-update", (event) => {
      setFeedState(event.payload);
      if (event.payload.tags.length > 0) {
        setTags(event.payload.tags);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [setFeedState]);

  const handleTagClick = (tagName: string) => {
    const next = activeTag === tagName ? null : tagName;
    setActiveTag(next);
    loadFeed(next, searchQuery || undefined);
  };

  const handleSearch = (value: string) => {
    setSearchQuery(value);
    if (searchTimer.current) clearTimeout(searchTimer.current);
    searchTimer.current = setTimeout(() => {
      loadFeed(activeTag, value || undefined);
    }, 400);
  };

  const handleRefresh = () => {
    loadFeed(activeTag, searchQuery || undefined);
  };

  const filteredItems: FeedItem[] = feedState?.items || [];

  const mergedTags: FeedTag[] = tags.length > 0
    ? tags
    : DEFAULT_TAGS.map((name) => ({ name, count: 0, is_active: name === activeTag }));

  return (
    <div className="moment-feed-panel">
      <div className="moment-feed-header">
        <h2>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" style={{ marginRight: 6, verticalAlign: -2 }}>
            <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
          </svg>
          资讯流
        </h2>
        <div className="moment-feed-header-actions">
          <button className="btn-icon" onClick={handleRefresh} title="刷新" disabled={feedLoading}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" className={feedLoading ? "spin" : ""}>
              <path d="M2 8a6 6 0 0 1 10-4M14 8a6 6 0 0 1-10 4" />
              <path d="M12 2h3v3M4 14H1v-3" />
            </svg>
          </button>
          <button className="btn-icon" onClick={() => setMomentFeedVisible(false)} title="关闭">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M4 4l8 8M12 4l-8 8" />
            </svg>
          </button>
        </div>
      </div>

      <div className="moment-feed-search">
        <input
          type="text"
          placeholder="搜索资讯..."
          value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
        />
      </div>

      <div className="moment-tag-bar">
        {mergedTags.map((tag) => (
          <button
            key={tag.name}
            className={`moment-tag ${activeTag === tag.name ? "active" : ""}`}
            onClick={() => handleTagClick(tag.name)}
          >
            {tag.name}
            {tag.count > 0 && <span className="moment-tag-count">{tag.count}</span>}
          </button>
        ))}
      </div>

      <div className="moment-feed-content">
        {feedLoading && filteredItems.length === 0 ? (
          <div className="moment-feed-loading">
            <div className="loading-spinner" />
          </div>
        ) : filteredItems.length === 0 ? (
          <div className="moment-feed-empty">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1" opacity="0.4">
              <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
            </svg>
            <span>暂无资讯</span>
            <span className="moment-feed-empty-hint">点击刷新按钮获取最新内容</span>
          </div>
        ) : (
          filteredItems.map((item) => (
            <MomentCard key={item.id} item={item} />
          ))
        )}
      </div>

      {feedState && feedState.timelines.length > 0 && (
        <div className="moment-timelines-section">
          <div
            className="moment-timelines-header"
            onClick={() => setTimelinesExpanded(!timelinesExpanded)}
          >
            <span className="moment-timelines-title">事件脉络</span>
            <span className="moment-timelines-count">{feedState.timelines.length} 条</span>
            <svg
              width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5"
              style={{ transform: timelinesExpanded ? "rotate(180deg)" : "none", transition: "transform 0.2s" }}
            >
              <path d="M4 6l4 4 4-4" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </div>
          {timelinesExpanded && (
            <div className="moment-timelines-list">
              {feedState.timelines.map((tl) => (
                <div key={tl.id} className="moment-timeline-item">
                  <div className="moment-timeline-title">{tl.title}</div>
                  <div className="moment-timeline-meta">
                    {new Date(tl.start_time * 1000).toLocaleDateString()} - {new Date(tl.end_time * 1000).toLocaleDateString()}
                    · {tl.item_ids.length} 条资讯
                  </div>
                  {tl.key_events.length > 0 && (
                    <ul className="moment-timeline-events">
                      {tl.key_events.slice(0, 3).map((ev, i) => (
                        <li key={i}>{ev}</li>
                      ))}
                      {tl.key_events.length > 3 && <li className="moment-timeline-more">+{tl.key_events.length - 3} 件</li>}
                    </ul>
                  )}
                  {tl.neotrix_summary && (
                    <div className="moment-insight-box" style={{ marginTop: 4, fontSize: 10 }}>
                      <div className="moment-insight-label">脉络总结</div>
                      {tl.neotrix_summary}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {feedState && (
        <div className="moment-feed-footer">
          <span className="moment-feed-footer-text">
            共 {feedState.total_count} 条资讯 · 最后更新 {new Date((feedState.last_refresh ?? 0) * 1000).toLocaleTimeString()}
          </span>
          <button className="moment-feed-footer-close" onClick={() => setMomentFeedVisible(false)}>
            关闭
          </button>
        </div>
      )}
    </div>
  );
};

export default MomentFeed;
