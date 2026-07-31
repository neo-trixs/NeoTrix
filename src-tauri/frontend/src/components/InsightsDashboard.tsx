import React, { useState, useEffect, useCallback } from "react";
import {
  insightsDaily,
  insightsWeekly,
  insightsInsights,
  insightsTrend,
  insightsConfig,
  insightsSetConfig,
  insightsStats,
  insightsCardList,
  insightsGenerateCard,
  insightsCardShare,
} from "../commands";
import type {
  DailyActivity,
  ActivityInsight,
  UsageCard,
} from "../commands";

/* ────────────────────────────────────────────
   Inline dark‑theme palette
   ──────────────────────────────────────────── */
const C = {
  bg: "#0b0b14",
  card: "#14141f",
  cardBorder: "#1e1e2f",
  accent: "#7c5cfc",
  accentDim: "#5a3fc8",
  text: "#e8e8ed",
  muted: "#6b6b80",
  dim: "#3a3a4a",
  green: "#34d399",
  red: "#f87171",
  orange: "#fb923c",
  blue: "#60a5fa",
};

function tabStyle(active: boolean): React.CSSProperties {
  return {
    padding: "7px 18px",
    borderRadius: "7px",
    border: "none",
    background: active ? C.accent : "transparent",
    color: active ? "#fff" : C.muted,
    fontSize: "13px",
    fontWeight: active ? 600 : 400,
    cursor: "pointer",
    transition: "all 0.15s",
  };
}

function changeBadge(pos: boolean): React.CSSProperties {
  return {
    display: "inline-flex",
    alignItems: "center",
    gap: "3px",
    fontSize: "12px",
    fontWeight: 600,
    color: pos ? C.green : C.red,
    background: pos ? "rgba(52,211,153,0.12)" : "rgba(248,113,113,0.12)",
    padding: "2px 8px",
    borderRadius: "6px",
  };
}

function chartBar(hPct: number, color: string): React.CSSProperties {
  return {
    flex: "1",
    height: `${Math.max(hPct, 2)}%`,
    background: color,
    borderRadius: "3px 3px 0 0",
    transition: "height 0.3s",
    minWidth: "6px",
  };
}

function toggleTrack(on: boolean): React.CSSProperties {
  return {
    width: "38px",
    height: "22px",
    borderRadius: "11px",
    background: on ? C.accent : C.dim,
    position: "relative" as const,
    cursor: "pointer",
    transition: "background 0.2s",
    flexShrink: 0,
  };
}

function toggleThumb(on: boolean): React.CSSProperties {
  return {
    width: "18px",
    height: "18px",
    borderRadius: "50%",
    background: "#fff",
    position: "absolute" as const,
    top: "2px",
    left: on ? "18px" : "2px",
    transition: "left 0.2s",
  };
}

/* ── helpers ──────────────────────────────── */
function shortDate(d: string): string {
  const m = d.match(/^\d{4}-(\d{2})-(\d{2})/);
  return m ? `${m[1]}/${m[2]}` : d.slice(5);
}

/* ── component ────────────────────────────── */
const InsightsDashboard: React.FC = () => {
  const [period, setPeriod] = useState<"today" | "week" | "month" | "all">("today");

  const [daily, setDaily] = useState<DailyActivity | null>(null);
  const [weekly, setWeekly] = useState<any>(null);
  const [insights, setInsights] = useState<ActivityInsight[]>([]);
  const [trend, setTrend] = useState<any>(null);
  const [cfg, setCfg] = useState<any>(null);
  const [stats, setStats] = useState<any>(null);
  const [cards, setCards] = useState<UsageCard[]>([]);

  const [generatedCard, setGeneratedCard] = useState<UsageCard | null>(null);
  const [shareUrl, setShareUrl] = useState<string | null>(null);
  const [cardTheme, setCardTheme] = useState<"dark" | "light">("dark");
  const [cardTitle, setCardTitle] = useState("");
  const [generating, setGenerating] = useState(false);

  const [loading, setLoading] = useState(true);

  /* ── load helpers ─────────────────────── */
  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      if (period === "today") {
        const [d, i] = await Promise.all([insightsDaily(), insightsInsights("today")]);
        setDaily(d);
        setInsights(i ?? []);
        setWeekly(null);
        setTrend(null);
      } else if (period === "week") {
        const [w, i] = await Promise.all([insightsWeekly(), insightsInsights("week")]);
        setWeekly(w);
        setInsights(i ?? []);
        setDaily(null);
        setTrend(null);
      } else if (period === "month") {
        const [i, t] = await Promise.all([insightsInsights("month"), insightsTrend(30)]);
        setInsights(i ?? []);
        setTrend(t);
        setDaily(null);
        setWeekly(null);
      } else {
        const [i, t] = await Promise.all([insightsInsights("all"), insightsTrend(90)]);
        setInsights(i ?? []);
        setTrend(t);
        setDaily(null);
        setWeekly(null);
      }
    } catch (e: any) {
      console.error("Failed to load insights data", e);
    } finally {
      setLoading(false);
    }
  }, [period]);

  const loadMeta = useCallback(async () => {
    try {
      const [c, st, cl] = await Promise.all([
        insightsConfig(),
        insightsStats(),
        insightsCardList(),
      ]);
      setCfg(c);
      setStats(st);
      setCards(cl);
    } catch {
      // silent
    }
  }, []);

  useEffect(() => { loadData(); }, [loadData]);
  useEffect(() => { loadMeta(); }, [loadMeta]);

  /* ── handlers ─────────────────────────── */
  const handleConfigToggle = async (key: string, val: boolean) => {
    const next = { ...cfg, [key]: val };
    try {
      await insightsSetConfig(next);
      setCfg(next);
    } catch { /* */ }
  };

  const handleGenerateCard = async () => {
    setGenerating(true);
    setShareUrl(null);
    try {
      const card = await insightsGenerateCard(period, cardTheme, cardTitle || undefined);
      setGeneratedCard(card);
    } catch { /* */ } finally {
      setGenerating(false);
    }
  };

  const handleShareCard = async () => {
    if (!generatedCard) return;
    try {
      const url = await insightsCardShare(generatedCard.id);
      setShareUrl(url);
    } catch { /* */ }
  };

  /* ── render helpers ───────────────────── */
  const renderSummary = () => {
    if (period === "today" && daily) {
      return (
        <div style={s.card}>
          <div style={s.cardTitle}>Today&apos;s Activity</div>
          <div style={s.grid3}>
            <div>
              <div style={s.metricLabel}>Active Minutes</div>
              <div style={s.metricValue}>{daily.active_minutes}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Sessions</div>
              <div style={s.metricValue}>{daily.sessions_count}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Commands</div>
              <div style={s.metricValue}>{daily.commands_executed}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Files Edited</div>
              <div style={s.metricValue}>{daily.files_edited}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Searches</div>
              <div style={s.metricValue}>{daily.searches_performed}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Reviews</div>
              <div style={s.metricValue}>{daily.reviews_done}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Errors</div>
              <div style={{ ...s.metricValue, color: daily.errors_count > 0 ? C.red : C.green }}>
                {daily.errors_count}
              </div>
            </div>
            <div>
              <div style={s.metricLabel}>Events</div>
              <div style={s.metricValue}>{daily.total_events}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Top Project</div>
              <div style={s.metricValue}>{daily.top_project ?? "—"}</div>
            </div>
          </div>
        </div>
      );
    }
    if (period === "week" && weekly) {
      return (
        <div style={s.card}>
          <div style={s.cardTitle}>Weekly Summary</div>
          <div style={s.grid3}>
            <div>
              <div style={s.metricLabel}>Total Active Hours</div>
              <div style={s.metricValue}>{(weekly.total_active_hours ?? 0).toFixed(1)}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Avg Daily</div>
              <div style={s.metricValue}>{(weekly.avg_daily_hours ?? 0).toFixed(1)}h</div>
            </div>
            <div>
              <div style={s.metricLabel}>Most Active Day</div>
              <div style={{ ...s.metricValue, fontSize: "13px" }}>{weekly.most_active_day ?? "—"}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Projects</div>
              <div style={s.metricValue}>{(weekly.projects_worked ?? []).length}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Top Category</div>
              <div style={s.metricValue}>{weekly.top_category ?? "—"}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Productivity Score</div>
              <div style={{ ...s.metricValue, color: (weekly.overall_productivity_score ?? 0) >= 70 ? C.green : C.orange }}>
                {weekly.overall_productivity_score ?? "—"}
              </div>
            </div>
          </div>
        </div>
      );
    }
    if ((period === "month" || period === "all") && trend) {
      const arr = trend.active_minutes ?? [];
      const totalMin = arr.reduce((a: number, b: number) => a + b, 0);
      const avgMin = arr.length ? Math.round(totalMin / arr.length) : 0;
      return (
        <div style={s.card}>
          <div style={s.cardTitle}>{period === "month" ? "Last 30 Days" : "All Time"} Activity</div>
          <div style={s.grid3}>
            <div>
              <div style={s.metricLabel}>Total Active Minutes</div>
              <div style={s.metricValue}>{totalMin}</div>
            </div>
            <div>
              <div style={s.metricLabel}>Avg Daily</div>
              <div style={s.metricValue}>{avgMin}m</div>
            </div>
            <div>
              <div style={s.metricLabel}>Trend</div>
              <div style={{ ...s.metricValue, color: trend.trend_direction === "up" ? C.green : C.red }}>
                {trend.trend_direction === "up" ? "↑" : "↓"} {Math.abs(trend.change_pct ?? 0).toFixed(1)}%
              </div>
            </div>
          </div>
        </div>
      );
    }
    return !loading ? <div style={{ ...s.card, ...s.emptyText } as React.CSSProperties}>No data for this period</div> : null;
  };

  const renderInsightCard = (ins: ActivityInsight, i: number) => (
    <div key={i} style={s.insightCard}>
      <div style={s.insightEmoji}>{ins.emoji}</div>
      <div style={s.insightTitle}>{ins.title}</div>
      <div style={s.insightDesc}>{ins.description}</div>
      <div style={s.flexBetween}>
        <div style={s.insightValue}>{ins.value}</div>
        {ins.change_pct != null && (
          <div style={changeBadge(ins.is_positive)}>
            {ins.is_positive ? "↑" : "↓"} {Math.abs(ins.change_pct).toFixed(1)}%
          </div>
        )}
      </div>
    </div>
  );

  const renderTrendChart = () => {
    if (!trend?.dates?.length) return null;
    const { dates, active_minutes, commands } = trend;
    const allVals = [...(active_minutes ?? []), ...(commands ?? [])];
    const maxVal = Math.max(...allVals, 1);

    return (
      <div style={s.card}>
        <div style={s.cardTitle}>Activity Trend</div>
        <div style={{ display: "flex", gap: "16px", marginBottom: "8px" }}>
          <div style={s.flexCenter}>
            <div style={{ width: 10, height: 10, borderRadius: 2, background: C.accent }} />
            <span style={{ fontSize: 11, color: C.muted }}>Active min</span>
          </div>
          <div style={s.flexCenter}>
            <div style={{ width: 10, height: 10, borderRadius: 2, background: C.blue }} />
            <span style={{ fontSize: 11, color: C.muted }}>Commands</span>
          </div>
        </div>
        <div style={s.chartContainer}>
          {dates.map((date: string, i: number) => {
            const h1 = ((active_minutes?.[i] ?? 0) / maxVal) * 100;
            const h2 = ((commands?.[i] ?? 0) / maxVal) * 100;
            return (
              <div key={date} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", gap: 2, minWidth: 6 }}>
                <div style={{ display: "flex", gap: 2, width: "100%", height: "100%", alignItems: "flex-end" }}>
                  <div style={chartBar(h1, C.accent)} title={`${date}: ${active_minutes?.[i] ?? 0}min`} />
                  <div style={chartBar(h2, C.blue)} title={`${date}: ${commands?.[i] ?? 0}cmd`} />
                </div>
                {dates.length <= 31 && (
                  <div style={s.chartLabel}>{shortDate(date)}</div>
                )}
              </div>
            );
          })}
        </div>
      </div>
    );
  };

  const renderCardPreview = () => {
    if (!generatedCard) return null;
    const isLight = generatedCard.theme === "light";
    const bg = isLight ? "#f5f5f5" : "#1a1a2e";
    const fg = isLight ? "#1a1a2e" : "#e8e8ed";
    const mu = isLight ? "#888" : "#6b6b80";
    const bd = isLight ? "#ddd" : "#2a2a3e";

    return (
      <div style={{ ...s.cardPreview, background: bg, color: fg, borderColor: bd }}>
        <div style={s.flexBetween}>
          <div>
            <div style={{ fontSize: 18, fontWeight: 700 } as React.CSSProperties}>{generatedCard.title}</div>
            <div style={{ fontSize: 12, color: mu } as React.CSSProperties}>{generatedCard.subtitle}</div>
          </div>
          <div style={{ fontSize: 11, color: mu, textAlign: "right" } as React.CSSProperties}>
            <div>{generatedCard.period}</div>
            <div>{generatedCard.generated_at?.slice(0, 10)}</div>
          </div>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 } as React.CSSProperties}>
          {Object.entries(generatedCard.stats).map(([k, v]) => (
            <div key={k} style={{ padding: "8px 12px", background: isLight ? "#eee" : "#252540", borderRadius: 8 } as React.CSSProperties}>
              <div style={{ fontSize: 11, color: mu, textTransform: "capitalize" } as React.CSSProperties}>{k.replace(/_/g, " ")}</div>
              <div style={{ fontSize: 16, fontWeight: 700 } as React.CSSProperties}>{v}</div>
            </div>
          ))}
        </div>
        <div style={s.flexBetween}>
          <button
            style={s.btnOutline}
            onClick={() => setCardTheme(isLight ? "dark" : "light")}
          >
            Toggle {isLight ? "Dark" : "Light"}
          </button>
          <div style={s.flexCenter}>
            {shareUrl && (
              <span style={{ fontSize: 11, color: C.green, maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis" } as React.CSSProperties}>
                {shareUrl}
              </span>
            )}
            <button style={s.btn} onClick={handleShareCard}>
              Share Card
            </button>
          </div>
        </div>
      </div>
    );
  };

  const renderConfig = () => {
    if (!cfg) return null;
    const toggles: { key: string; label: string }[] = [
      { key: "track_activity", label: "Track Activity" },
      { key: "show_notifications", label: "Show Notifications" },
      { key: "weekly_summary_enabled", label: "Weekly Summary" },
    ];
    return (
      <div style={s.card}>
        <div style={s.cardTitle}>Configuration</div>
        {toggles.map((t) => (
          <div key={t.key} style={s.configRow}>
            <span style={{ fontSize: 13 } as React.CSSProperties}>{t.label}</span>
            <div style={toggleTrack(!!(cfg as any)[t.key])} onClick={() => handleConfigToggle(t.key, !(cfg as any)[t.key])}>
              <div style={toggleThumb(!!(cfg as any)[t.key])} />
            </div>
          </div>
        ))}
        <div style={s.configRow}>
          <span style={{ fontSize: 13 } as React.CSSProperties}>Retention Days</span>
          <span style={{ fontSize: 14, fontWeight: 600 } as React.CSSProperties}>{cfg.retention_days ?? 90}</span>
        </div>
      </div>
    );
  };

  const renderCardHistory = () => {
    if (!cards?.length) {
      return (
        <div style={s.card}>
          <div style={s.cardTitle}>Card History</div>
          <div style={s.emptyText}>No usage cards generated yet</div>
        </div>
      );
    }
    return (
      <div style={s.card}>
        <div style={s.cardTitle}>Card History ({cards.length})</div>
        {cards.map((c) => (
          <div key={c.id} style={s.historyItem}>
            <div>
              <div style={{ fontSize: 13, fontWeight: 600 } as React.CSSProperties}>{c.title}</div>
              <div style={{ fontSize: 11, color: C.muted } as React.CSSProperties}>
                {c.period} · {c.generated_at?.slice(0, 10)}
              </div>
            </div>
            <div style={s.flexCenter}>
              {c.share_url && (
                <span style={{ fontSize: 11, color: C.green } as React.CSSProperties}>Shared</span>
              )}
              <div style={{ fontSize: 11, color: C.muted } as React.CSSProperties}>
                {Object.keys(c.stats).length} stats
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  };

  /* ── loading ───────────────────────────── */
  if (loading && !daily && !weekly && !trend) {
    return (
      <div style={s.wrapper}>
        <div style={{ color: C.muted, textAlign: "center", paddingTop: "80px" } as React.CSSProperties}>
          Loading insights…
        </div>
      </div>
    );
  }

  /* ── main render ───────────────────────── */
  return (
    <div style={s.wrapper}>
      {/* ── header + period tabs ─────────── */}
      <div style={s.headerRow}>
        <div style={s.headerTitle}>Activity Insights</div>
        <div style={s.tabRow}>
          {(["today", "week", "month", "all"] as const).map((p) => (
            <button key={p} style={tabStyle(p === period)} onClick={() => setPeriod(p)}>
              {p === "today" ? "Today" : p === "week" ? "Week" : p === "month" ? "Month" : "All"}
            </button>
          ))}
        </div>
      </div>

      {/* ── stats bar ────────────────────── */}
      {stats && (
        <div style={s.statsBar}>
          <div style={s.statPill}>
            <div style={{ ...s.statIcon, background: "rgba(124,92,252,0.15)" } as React.CSSProperties}>📊</div>
            <div>
              <div style={s.statLabel}>Events Tracked</div>
              <div style={s.statValue}>{(stats.total_events_tracked ?? 0).toLocaleString()}</div>
            </div>
          </div>
          <div style={s.statPill}>
            <div style={{ ...s.statIcon, background: "rgba(96,165,250,0.15)" } as React.CSSProperties}>📅</div>
            <div>
              <div style={s.statLabel}>Days Active</div>
              <div style={s.statValue}>{stats.days_active ?? 0}</div>
            </div>
          </div>
          <div style={s.statPill}>
            <div style={{ ...s.statIcon, background: "rgba(52,211,153,0.15)" } as React.CSSProperties}>🔥</div>
            <div>
              <div style={s.statLabel}>Current Streak</div>
              <div style={s.statValue}>{stats.current_streak_days ?? 0}d</div>
            </div>
          </div>
          <div style={s.statPill}>
            <div style={{ ...s.statIcon, background: "rgba(251,146,60,0.15)" } as React.CSSProperties}>🏆</div>
            <div>
              <div style={s.statLabel}>Longest Streak</div>
              <div style={s.statValue}>{stats.longest_streak_days ?? 0}d</div>
            </div>
          </div>
          <div style={s.statPill}>
            <div style={{ ...s.statIcon, background: "rgba(124,92,252,0.15)" } as React.CSSProperties}>⏱️</div>
            <div>
              <div style={s.statLabel}>Avg Daily</div>
              <div style={s.statValue}>{stats.avg_daily_active_mins ?? 0}m</div>
            </div>
          </div>
        </div>
      )}

      {/* ── summary ──────────────────────── */}
      {renderSummary()}

      {/* ── AI insights cards ────────────── */}
      {insights.length > 0 && (
        <div>
          <div style={s.sectionLabel}>AI Insights</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))", gap: "16px" } as React.CSSProperties}>
            {insights.map((ins, i) => renderInsightCard(ins, i))}
          </div>
        </div>
      )}

      {/* ── trend chart ──────────────────── */}
      {renderTrendChart()}

      {/* ── card generator ───────────────── */}
      <div style={s.card}>
        <div style={s.cardTitle}>Usage Card Generator</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "12px", alignItems: "center", marginBottom: "16px" } as React.CSSProperties}>
          <input
            placeholder="Card title (optional)"
            value={cardTitle}
            onChange={(e) => setCardTitle(e.target.value)}
            style={{
              flex: "1 1 200px",
              padding: "8px 14px",
              borderRadius: 8,
              border: `1px solid ${C.dim}`,
              background: C.bg,
              color: C.text,
              fontSize: 13,
              outline: "none",
            }}
          />
          <button style={s.btnOutline} onClick={() => setCardTheme(cardTheme === "dark" ? "light" : "dark")}>
            Theme: {cardTheme === "dark" ? "Dark" : "Light"}
          </button>
          <button style={s.btn} onClick={handleGenerateCard} disabled={generating}>
            {generating ? "Generating…" : "Generate Card"}
          </button>
        </div>

        {renderCardPreview()}
      </div>

      {/* ── card history ─────────────────── */}
      {renderCardHistory()}

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "20px" } as React.CSSProperties}>
        {/* ── config ─────────────────────── */}
        {renderConfig()}
      </div>
    </div>
  );
};

/* ── static style map ───────────────────── */
const s: Record<string, React.CSSProperties> = {
  wrapper: {
    minHeight: "100vh",
    background: C.bg,
    color: C.text,
    fontFamily: "Inter, system-ui, sans-serif",
    padding: "32px 40px",
    display: "flex",
    flexDirection: "column",
    gap: "24px",
  },
  headerRow: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    gap: "16px",
  },
  headerTitle: {
    fontSize: "22px",
    fontWeight: 700,
    letterSpacing: "-0.01em",
  },
  tabRow: {
    display: "flex",
    gap: "4px",
    background: C.card,
    borderRadius: "10px",
    padding: "4px",
    border: `1px solid ${C.cardBorder}`,
  },
  statsBar: {
    display: "flex",
    gap: "16px",
    flexWrap: "wrap",
  },
  statPill: {
    background: C.card,
    border: `1px solid ${C.cardBorder}`,
    borderRadius: "10px",
    padding: "12px 20px",
    display: "flex",
    alignItems: "center",
    gap: "10px",
    flex: "1 0 auto",
    minWidth: "140px",
  },
  statIcon: {
    width: "32px",
    height: "32px",
    borderRadius: "8px",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: "16px",
  },
  statLabel: {
    fontSize: "11px",
    color: C.muted,
    fontWeight: 500,
    textTransform: "uppercase",
    letterSpacing: "0.04em",
  },
  statValue: {
    fontSize: "18px",
    fontWeight: 700,
  },
  grid3: {
    display: "grid",
    gridTemplateColumns: "1fr 1fr 1fr",
    gap: "20px",
  },
  card: {
    background: C.card,
    border: `1px solid ${C.cardBorder}`,
    borderRadius: "14px",
    padding: "20px",
  },
  cardTitle: {
    fontSize: "13px",
    fontWeight: 600,
    color: C.muted,
    textTransform: "uppercase",
    letterSpacing: "0.05em",
    marginBottom: "14px",
  },
  metricLabel: {
    fontSize: "13px",
    color: C.muted,
  },
  metricValue: {
    fontSize: "14px",
    fontWeight: 600,
  },
  insightCard: {
    background: C.card,
    border: `1px solid ${C.cardBorder}`,
    borderRadius: "14px",
    padding: "20px",
    display: "flex",
    flexDirection: "column",
    gap: "10px",
  },
  insightEmoji: {
    fontSize: "24px",
  },
  insightTitle: {
    fontSize: "15px",
    fontWeight: 600,
  },
  insightDesc: {
    fontSize: "12px",
    color: C.muted,
    lineHeight: 1.4,
  },
  insightValue: {
    fontSize: "20px",
    fontWeight: 700,
  },
  chartContainer: {
    display: "flex",
    alignItems: "flex-end",
    gap: "3px",
    height: "140px",
    paddingTop: "8px",
  },
  chartLabel: {
    fontSize: "9px",
    color: C.muted,
    textAlign: "center",
    marginTop: "4px",
  },
  btn: {
    padding: "8px 18px",
    borderRadius: "8px",
    border: "none",
    background: C.accent,
    color: "#fff",
    fontSize: "13px",
    fontWeight: 600,
    cursor: "pointer",
  },
  btnOutline: {
    padding: "6px 14px",
    borderRadius: "8px",
    border: `1px solid ${C.dim}`,
    background: "transparent",
    color: C.text,
    fontSize: "12px",
    cursor: "pointer",
    fontWeight: 500,
  },
  configRow: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "8px 0",
  },
  cardPreview: {
    background: C.card,
    border: `1px solid ${C.cardBorder}`,
    borderRadius: "14px",
    padding: "24px",
    display: "flex",
    flexDirection: "column",
    gap: "16px",
  },
  historyItem: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "10px 0",
    borderBottom: `1px solid ${C.dim}`,
  },
  flexBetween: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "12px",
  },
  flexCenter: {
    display: "flex",
    alignItems: "center",
    gap: "8px",
  },
  sectionLabel: {
    fontSize: "15px",
    fontWeight: 600,
    marginBottom: "12px",
  },
  emptyText: {
    color: C.muted,
    fontSize: "13px",
    padding: "20px 0",
    textAlign: "center",
  },
};

export default InsightsDashboard;
