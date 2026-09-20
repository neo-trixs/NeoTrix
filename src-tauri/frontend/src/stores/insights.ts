/* ════════════════════════════════════════════
   stores/insights.ts — 洞察页状态（Phase 2 B1: mock-first）

   数据形状对齐后端（接线时零转换）：
   - DailyActivity / WeeklySummary ← insights_cmds.rs:151/189
   - UsageLedgerReport ← nt_core_telemetry ProviderUsageLedger
     （活动记录·非权威账单 — grok-bot 诚实标注纪律）

   ⚠️ seam 策略同 stores/kb.ts: mock 先行，P3-M4 切换为
   invoke('insights_daily'/'insights_weekly'/...) 时组件零改动。
   ════════════════════════════════════════════ */
import { createSignal } from 'solid-js'

/* ── 后端形状镜像 ── */

export interface DailyActivity {
  date: string
  total_events: number
  active_minutes: number
  sessions_count: number
  commands_executed: number
  files_edited: number
  searches_performed: number
  reviews_done: number
  errors_count: number
  top_project: string | null
  categories: Record<string, number>
}

export interface ActivityInsight {
  title: string
  detail: string
}

export interface WeeklySummary {
  week_start: string
  week_end: string
  total_active_hours: number
  avg_daily_hours: number
  most_active_day: string
  projects_worked: string[]
  top_category: string
  insights: ActivityInsight[]
  overall_productivity_score: number
}

/** per-provider 用量条目 — 活动记录，非权威账单 */
export interface ProviderUsageEntry {
  provider: string
  requestCount: number
  promptTokens: number
  completionTokens: number
}

export interface UsageLedgerReport {
  /** 诚实标注：恒 true — 此账本是本地活动记录，不能当计费依据 */
  activityRecord: boolean
  entries: ProviderUsageEntry[]
}

export interface InsightsDataSource {
  getLedger(): Promise<UsageLedgerReport>
  getDaily(date?: string): Promise<DailyActivity>
  getWeekly(weekStart?: string): Promise<WeeklySummary>
}

/* ── Mock 实现 ── */

function daysAgoIso(n: number): string {
  const d = new Date(Date.now() - n * 86_400_000)
  return d.toISOString().slice(0, 10)
}

function mockDataSource(): InsightsDataSource {
  const ledger: UsageLedgerReport = {
    activityRecord: true,
    entries: [
      { provider: 'cli-session', requestCount: 42, promptTokens: 18_400, completionTokens: 6_120 },
      { provider: 'llm7', requestCount: 17, promptTokens: 9_250, completionTokens: 3_480 },
      { provider: 'pollinations', requestCount: 8, promptTokens: 3_100, completionTokens: 1_260 },
    ],
  }
  const daily: DailyActivity = {
    date: daysAgoIso(0),
    total_events: 156,
    active_minutes: 214,
    sessions_count: 6,
    commands_executed: 48,
    files_edited: 23,
    searches_performed: 11,
    reviews_done: 3,
    errors_count: 2,
    top_project: 'neotrix',
    categories: { chat: 62, coding: 41, review: 12, search: 11, absorb: 30 },
  }
  const weekly: WeeklySummary = {
    week_start: daysAgoIso(6),
    week_end: daysAgoIso(0),
    total_active_hours: 19.5,
    avg_daily_hours: 2.8,
    most_active_day: '周二',
    projects_worked: ['neotrix', 'knowledge-corpus'],
    top_category: 'chat',
    insights: [
      { title: '专注时段稳定', detail: '连续 5 天在 20:00-22:00 出现活跃峰值' },
      { title: '审查习惯养成', detail: '本周完成 12 次 rev-officer 审查，环比 +50%' },
    ],
    overall_productivity_score: 78,
  }
  return {
    async getLedger() {
      return { ...ledger, entries: ledger.entries.map((e) => ({ ...e })) }
    },
    async getDaily() {
      return { ...daily, categories: { ...daily.categories } }
    },
    async getWeekly() {
      return { ...weekly, insights: weekly.insights.map((i) => ({ ...i })), projects_worked: [...weekly.projects_worked] }
    },
  }
}

const dataSource: InsightsDataSource = mockDataSource()

/** 可注入数据源（测试用），默认 mock */
export function createInsightsStore(source: InsightsDataSource = dataSource) {
  const [ledger, setLedger] = createSignal<UsageLedgerReport | null>(null)
  const [daily, setDaily] = createSignal<DailyActivity | null>(null)
  const [weekly, setWeekly] = createSignal<WeeklySummary | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  async function refresh() {
    setLoading(true)
    setError(null)
    try {
      const [l, d, w] = await Promise.all([source.getLedger(), source.getDaily(), source.getWeekly()])
      setLedger(l)
      setDaily(d)
      setWeekly(w)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  return { ledger, daily, weekly, loading, error, refresh }
}

export type InsightsStore = ReturnType<typeof createInsightsStore>
