/* ════════════════════════════════════════════
   routes/Insights.tsx — 洞察页 (Phase 2 B1)

   吸收 LobeChat analytics + grok-bot Usage & Billing 模式：
   - 成本卡：agent_status 经 api/query 3s TTL 缓存（与 CostDashboard 同源）
   - 用量账本：activity_record=true 诚实标注 — 活动记录≠权威账单
   - 日/周活动卡：形状对齐 insights_daily/insights_weekly
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Gauge, RefreshCw, Loader2, Wallet, Coins, Database, Info } from 'lucide-solid'
import { clsx } from 'clsx'
import type { AgentStatus } from '../api/types'
import { neocodex } from '../api'
import type { ProviderUsageRow } from '../api/types'
import { query } from '../api/query'
import { createInsightsStore } from '../stores/insights'

function fmtTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
  return String(n)
}

export function Insights() {
  const navigate = useNavigate()
  const ins = createInsightsStore()
  // P3-M5 已接线: 与 CostDashboard 共享 'agent_status' 3s TTL 缓存
  const [status, setStatus] = createSignal<AgentStatus | null>(null)

  async function loadStatus(force = false) {
    try {
      setStatus(await query<AgentStatus>('agent_status', () => neocodex.agentStatus(), { ttlMs: 3000, force }))
    } catch {
      /* 静默 — 成本卡显示占位符 */
    }
  }

  // P3-M4: 用量账本真源 (provider_usage_snapshot), 空账本回退 mock 种子
  const [liveLedger, setLiveLedger] = createSignal<ProviderUsageRow[] | null>(null)
  onMount(() => {
    void ins.refresh()
    void loadStatus()
  })

  const budgetPct = () => {
    const s = status()
    if (!s || s.cost_budget <= 0) return 0
    return Math.min(100, Math.round((s.cost_spent / s.cost_budget) * 100))
  }

  const maxLedgerRequests = () => Math.max(1, ...(ins.ledger()?.entries ?? []).map((e) => e.requestCount))

  return (
    <div class="min-h-screen mac-safe bg-bg-primary text-text-primary flex flex-col">
      {/* 顶栏 */}
      <header class="flex items-center gap-3 px-5 h-12 border-b border-border-primary/40 shrink-0">
        <button
          class="flex items-center gap-1.5 text-13px text-text-muted hover:text-text-primary transition-colors"
          onClick={() => navigate('/chat')}
          aria-label="返回对话"
        >
          <ArrowLeft class="w-4 h-4" />
          对话
        </button>
        <h1 class="text-14px font-semibold flex items-center gap-1.5">
          <Gauge class="w-4 h-4 text-nt-io-600" />
          洞察
        </h1>
        <div class="flex-1" />
        <button
          class="flex items-center gap-1.5 h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary hover:bg-white/40 transition-colors"
          onClick={() => { void ins.refresh(); void loadStatus(true) }}
          aria-label="刷新洞察"
          title="刷新"
        >
          <RefreshCw class={clsx('w-4 h-4', ins.loading() && 'animate-spin')} />
        </button>
      </header>

      <main class="flex-1 overflow-y-auto p-5">
        <Show when={!ins.loading()} fallback={
          <div class="flex items-center justify-center py-24 text-text-muted" role="status">
            <Loader2 class="w-5 h-5 animate-spin mr-2" /> 加载中…
          </div>
        }>
          <Show when={!ins.error()} fallback={
            <div class="text-center py-24">
              <p class="text-13px text-text-muted mb-3">{ins.error()}</p>
              <button class="text-13px text-nt-io-600 hover:underline" onClick={() => void ins.refresh()}>重试</button>
            </div>
          }>
            <div class="max-w-5xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-4">
              {/* ── 成本卡 ── */}
              <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4 lg:col-span-2" aria-label="成本概览">
                <div class="flex items-center gap-2 mb-3">
                  <Wallet class="w-4 h-4 text-nt-io-600" />
                  <h2 class="text-13px font-semibold">成本概览</h2>
                  <span class="ml-auto text-12px text-text-muted">{status()?.provider_model}</span>
                </div>
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                  <div>
                    <p class="text-11px text-text-muted">已花费</p>
                    <p class="text-lg font-semibold">${status()?.cost_spent.toFixed(2) ?? '—'}</p>
                  </div>
                  <div>
                    <p class="text-11px text-text-muted">预算</p>
                    <p class="text-lg font-semibold">${status()?.cost_budget ?? '—'}</p>
                  </div>
                  <div>
                    <p class="text-11px text-text-muted">累计 Token</p>
                    <p class="text-lg font-semibold inline-flex items-center gap-1"><Coins class="w-3.5 h-3.5 text-nt-io-600" />{fmtTokens(status()?.tokens_used ?? 0)}</p>
                  </div>
                  <div>
                    <p class="text-11px text-text-muted">进化迭代</p>
                    <p class="text-lg font-semibold">{status()?.evolution_iterations ?? '—'}</p>
                  </div>
                </div>
                {/* 预算进度条 */}
                <div class="mt-3">
                  <div class="h-1.5 rounded-full bg-black/10 overflow-hidden" role="progressbar" aria-valuenow={budgetPct()} aria-valuemin={0} aria-valuemax={100} aria-label="预算使用率">
                    <div class="h-full rounded-full bg-gradient-to-r bg-nt-io-500 transition-all" style={{ width: `${budgetPct()}%` }} />
                  </div>
                  <p class="text-11px text-text-muted mt-1">预算使用率 {budgetPct()}%</p>
                </div>
              </section>

              {/* ── 用量账本 ── */}
              <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4" aria-label="模型用量账本">
                <div class="flex items-center gap-2 mb-1">
                  <Database class="w-4 h-4 text-nt-io-600" />
                  <h2 class="text-13px font-semibold">模型用量账本</h2>
                </div>
                <p class="flex items-start gap-1 text-11px text-text-muted mb-3">
                  <Info class="w-3 h-3 mt-0.5 shrink-0" />
                  活动记录，非权威账单 — 与服务商实际计费可能存在差异
                </p>
                <table class="w-full text-12px">
                  <thead>
                    <tr class="text-left text-text-muted border-b border-border-primary/40">
                      <th class="pb-1.5 font-medium">后端</th>
                      <th class="pb-1.5 font-medium text-right">请求</th>
                      <th class="pb-1.5 font-medium text-right">Prompt</th>
                      <th class="pb-1.5 font-medium text-right">Completion</th>
                      <th class="pb-1.5 pl-4 font-medium w-[30%]">占比</th>
                    </tr>
                  </thead>
                  <tbody>
                    <Show when={!liveLedger()} fallback={
                  <For each={liveLedger() ?? []}>
                    {(e) => (
                      <tr class="border-b border-border-primary/20 last:border-0">
                        <td class="py-1.5 font-medium truncate max-w-[120px]" title={e.provider}>{e.provider}</td>
                        <td class="py-1.5 text-right tabular-nums">{e.request_count}</td>
                        <td class="py-1.5 text-right tabular-nums text-text-muted">{fmtTokens(e.prompt_tokens)}</td>
                        <td class="py-1.5 text-right tabular-nums text-text-muted">{fmtTokens(e.completion_tokens)}</td>
                        <td class="py-1.5 pl-4">
                          <div class="h-1.5 rounded-full bg-black/10 overflow-hidden">
                            <div class="h-full bg-nt-io-500/70 rounded-full" style={{ width: `${Math.round((e.request_count / Math.max(1, ...(liveLedger() ?? []).map((x) => x.request_count))) * 100)}%` }} />
                          </div>
                        </td>
                      </tr>
                    )}
                  </For>
                }>
                <For each={ins.ledger()?.entries ?? []}>
                      {(e) => (
                        <tr class="border-b border-border-primary/20 last:border-0">
                          <td class="py-1.5 font-medium truncate max-w-[120px]" title={e.provider}>{e.provider}</td>
                          <td class="py-1.5 text-right tabular-nums">{e.requestCount}</td>
                          <td class="py-1.5 text-right tabular-nums text-text-muted">{fmtTokens(e.promptTokens)}</td>
                          <td class="py-1.5 text-right tabular-nums text-text-muted">{fmtTokens(e.completionTokens)}</td>
                          <td class="py-1.5 pl-4">
                            <div class="h-1.5 rounded-full bg-black/10 overflow-hidden">
                              <div class="h-full bg-nt-io-500/70 rounded-full" style={{ width: `${Math.round((e.requestCount / maxLedgerRequests()) * 100)}%` }} />
                            </div>
                          </td>
                        </tr>
                      )}
                    </For>
                </Show>
                  </tbody>
                </table>
              </section>

              {/* ── 今日活动 ── */}
              <Show when={ins.daily()}>
                {(d) => (
                  <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4" aria-label="今日活动">
                    <h2 class="text-13px font-semibold mb-3">今日活动 <span class="text-11px text-text-muted font-normal">{d().date}</span></h2>
                    <div class="grid grid-cols-3 gap-y-3 gap-x-2 text-center">
                      <div><p class="text-base font-semibold">{d().active_minutes}</p><p class="text-11px text-text-muted">活跃分钟</p></div>
                      <div><p class="text-base font-semibold">{d().sessions_count}</p><p class="text-11px text-text-muted">会话</p></div>
                      <div><p class="text-base font-semibold">{d().commands_executed}</p><p class="text-11px text-text-muted">命令</p></div>
                      <div><p class="text-base font-semibold">{d().files_edited}</p><p class="text-11px text-text-muted">文件编辑</p></div>
                      <div><p class="text-base font-semibold">{d().searches_performed}</p><p class="text-11px text-text-muted">搜索</p></div>
                      <div><p class={clsx('text-base font-semibold', d().errors_count > 0 && 'text-red-500')}>{d().errors_count}</p><p class="text-11px text-text-muted">错误</p></div>
                    </div>
                    <Show when={d().top_project}>
                      <p class="text-11px text-text-muted mt-3">最活跃项目: <span class="text-text-primary font-medium">{d().top_project}</span></p>
                    </Show>
                  </section>
                )}
              </Show>

              {/* ── 本周摘要 ── */}
              <Show when={ins.weekly()}>
                {(w) => (
                  <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4 lg:col-span-2" aria-label="本周摘要">
                    <div class="flex items-center gap-2 mb-3">
                      <h2 class="text-13px font-semibold">本周摘要</h2>
                      <span class="text-11px text-text-muted">{w().week_start} ~ {w().week_end}</span>
                      <span
                        class="ml-auto inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-11px font-medium bg-nt-io-500/10 text-nt-io-700"
                        aria-label={`生产力评分 ${w().overall_productivity_score}`}
                      >
                        生产力 {w().overall_productivity_score}
                      </span>
                    </div>
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-3">
                      <div><p class="text-base font-semibold">{w().total_active_hours}h</p><p class="text-11px text-text-muted">总活跃</p></div>
                      <div><p class="text-base font-semibold">{w().avg_daily_hours}h</p><p class="text-11px text-text-muted">日均</p></div>
                      <div><p class="text-base font-semibold">{w().most_active_day}</p><p class="text-11px text-text-muted">最活跃日</p></div>
                      <div><p class="text-base font-semibold truncate" title={w().top_category}>{w().top_category}</p><p class="text-11px text-text-muted">主导类别</p></div>
                    </div>
                    <ul class="space-y-1.5">
                      <For each={w().insights}>
                        {(i) => (
                          <li class="flex items-start gap-2 text-12px">
                            <span class="w-1.5 h-1.5 rounded-full bg-nt-io-500 mt-1.5 shrink-0" />
                            <span><span class="font-medium">{i.title}</span> — <span class="text-text-muted">{i.detail}</span></span>
                          </li>
                        )}
                      </For>
                    </ul>
                  </section>
                )}
              </Show>
            </div>
          </Show>
        </Show>
      </main>
    </div>
  )
}
