import { describe, it, expect } from 'vitest'
import { createInsightsStore, type InsightsDataSource } from './insights'

/** 形状哨兵: mock 数据结构必须镜像后端 insights_cmds.rs 契约 */
const fakeSource: InsightsDataSource = {
  async getLedger() {
    return {
      activityRecord: true,
      entries: [{ provider: 'llm7', requestCount: 5, promptTokens: 100, completionTokens: 40 }],
    }
  },
  async getDaily() {
    return {
      date: '2026-08-25',
      total_events: 10,
      active_minutes: 30,
      sessions_count: 2,
      commands_executed: 4,
      files_edited: 1,
      searches_performed: 1,
      reviews_done: 0,
      errors_count: 0,
      top_project: 'neotrix',
      categories: { chat: 6 },
    }
  },
  async getWeekly() {
    return {
      week_start: '2026-08-19',
      week_end: '2026-08-25',
      total_active_hours: 3.5,
      avg_daily_hours: 0.5,
      most_active_day: '周一',
      projects_worked: ['neotrix'],
      top_category: 'chat',
      insights: [{ title: 't', detail: 'd' }],
      overall_productivity_score: 60,
    }
  },
}

describe('insights store', () => {
  it('refresh 聚合三数据源且 loading 状态正确流转', async () => {
    const ins = createInsightsStore(fakeSource)
    expect(ins.loading()).toBe(false)
    const p = ins.refresh()
    expect(ins.loading()).toBe(true)
    await p
    expect(ins.loading()).toBe(false)
    expect(ins.error()).toBeNull()
    expect(ins.daily()?.total_events).toBe(10)
    expect(ins.weekly()?.overall_productivity_score).toBe(60)
  })

  it('ledger 携带 activity_record 诚实标注 (活动记录≠权威账单)', async () => {
    const ins = createInsightsStore()
    await ins.refresh()
    expect(ins.ledger()?.activityRecord).toBe(true)
  })

  it('dataSource 异常时 error 置位、数据保持 null 且不抛出', async () => {
    const failing: InsightsDataSource = {
      getLedger: () => Promise.reject(new Error('backend down')),
      getDaily: () => Promise.reject(new Error('backend down')),
      getWeekly: () => Promise.reject(new Error('backend down')),
    }
    const ins = createInsightsStore(failing)
    await ins.refresh()
    expect(ins.error()).toBe('backend down')
    expect(ins.ledger()).toBeNull()
    expect(ins.daily()).toBeNull()
    expect(ins.weekly()).toBeNull()
    expect(ins.loading()).toBe(false)
  })

  it('mock 种子账本含 cli-session 后端 (Phase 1 会话复用后端可见性)', async () => {
    const ins = createInsightsStore()
    await ins.refresh()
    const providers = ins.ledger()?.entries.map((e) => e.provider) ?? []
    expect(providers).toContain('cli-session')
    expect(providers.length).toBeGreaterThanOrEqual(3)
  })

  it('形状镜像后端契约字段名 (snake_case 对齐 insights_cmds.rs)', async () => {
    const ins = createInsightsStore()
    await ins.refresh()
    const d = ins.daily()!
    expect(Object.keys(d)).toContain('active_minutes')
    expect(Object.keys(d)).toContain('sessions_count')
    expect(Object.keys(d)).toContain('errors_count')
    const w = ins.weekly()!
    expect(Object.keys(w)).toContain('week_start')
    expect(Object.keys(w)).toContain('avg_daily_hours')
  })
})
