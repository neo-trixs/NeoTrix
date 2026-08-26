/* ════════════════════════════════════════════
   routes/MemoryManager.tsx — 记忆管理页 (Phase 2 B6, 直连后端)

   吸收 Chatbox 数据管理模式：三面板
   - 统计: total/avg_confidence/usage/categories
   - 时间线: 近 14 天创建柱状 + top_topic
   - 搜索: 后端 memory_search + 结果列表 (kind 徽章 + 置顶标)
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, BrainCircuit, Search, Loader2, RefreshCw, Pin } from 'lucide-solid'
import { clsx } from 'clsx'
import { memoryStats, memorySearch, memoryTimeline, type MemoryEntry, type MemoryTimelineEntry } from '../api/memory'
import type { MemoryStats } from '../api/types'
import { errText } from '../api'

function fmtBytes(n: number): string {
  if (n >= 1_048_576) return `${(n / 1_048_576).toFixed(1)} MB`
  if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`
  return `${n} B`
}

export function MemoryManager() {
  const navigate = useNavigate()
  const [stats, setStats] = createSignal<MemoryStats | null>(null)
  const [timeline, setTimeline] = createSignal<MemoryTimelineEntry[]>([])
  const [results, setResults] = createSignal<MemoryEntry[] | null>(null)
  const [query, setQuery] = createSignal('')
  const [searching, setSearching] = createSignal(false)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  async function load() {
    setLoading(true)
    setError(null)
    try {
      const [s, t] = await Promise.all([memoryStats(), memoryTimeline(14)])
      setStats(s)
      setTimeline(t)
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(() => void load())

  async function doSearch() {
    const q = query().trim()
    if (!q) {
      setResults(null)
      return
    }
    setSearching(true)
    setError(null)
    try {
      setResults(await memorySearch(q))
    } catch (e) {
      setError(errText(e))
    } finally {
      setSearching(false)
    }
  }

  const maxCreated = () => Math.max(1, ...timeline().map((t) => t.entries_created))

  return (
    <div class="min-h-screen bg-bg-primary text-text-primary flex flex-col">
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
          <BrainCircuit class="w-4 h-4 text-nt-io-600" />
          记忆管理
        </h1>
        <div class="flex-1" />
        <button
          class="flex items-center gap-1.5 h-8 px-3 rounded-lg text-13px text-text-muted hover:text-text-primary hover:bg-white/40 transition-colors"
          onClick={() => void load()}
          aria-label="刷新记忆"
          title="刷新"
        >
          <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
        </button>
      </header>

      <main class="flex-1 overflow-y-auto p-5">
        <Show when={!loading()} fallback={
          <div class="flex items-center justify-center py-24 text-text-muted" role="status">
            <Loader2 class="w-5 h-5 animate-spin mr-2" /> 加载中…
          </div>
        }>
          <Show when={!error()} fallback={
            <div class="text-center py-24">
              <p class="text-13px text-text-muted mb-3">{error()}</p>
              <button class="text-13px text-nt-io-600 hover:underline" onClick={() => void load()}>重试</button>
            </div>
          }>
            <div class="max-w-5xl mx-auto space-y-4">
              {/* ── 统计面板 ── */}
              <Show when={stats()}>
                {(s) => (
                  <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4 grid grid-cols-2 sm:grid-cols-4 gap-3" aria-label="记忆统计">
                    <div><p class="text-lg font-semibold">{s().total_entries}</p><p class="text-11px text-text-muted">总条目</p></div>
                    <div><p class="text-lg font-semibold">{Math.round(s().avg_confidence * 100)}%</p><p class="text-11px text-text-muted">平均置信度</p></div>
                    <div><p class="text-lg font-semibold">{fmtBytes(s().memory_usage_bytes)}</p><p class="text-11px text-text-muted">占用</p></div>
                    <div><p class="text-lg font-semibold">{s().total_categories}</p><p class="text-11px text-text-muted">类别</p></div>
                  </section>
                )}
              </Show>

              {/* ── 时间线 ── */}
              <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4" aria-label="记忆时间线">
                <h2 class="text-13px font-semibold mb-3">近 14 天创建</h2>
                <Show when={timeline().length > 0} fallback={<p class="text-12px text-text-muted py-4 text-center">暂无时间线数据</p>}>
                  <div class="flex items-end gap-1.5 h-20">
                    <For each={timeline()}>
                      {(t) => (
                        <div class="flex-1 flex flex-col items-center gap-1 group" title={`${t.date}: ${t.entries_created} 条 · 热题 ${t.top_topic}`}>
                          <div
                            class="w-full rounded-t bg-nt-io-500/70 hover:bg-nt-io-600 transition-colors"
                            style={{ height: `${Math.max(4, Math.round((t.entries_created / maxCreated()) * 64))}px` }}
                          />
                          <span class="text-[10px] text-text-muted">{t.date.slice(8)}</span>
                        </div>
                      )}
                    </For>
                  </div>
                </Show>
              </section>

              {/* ── 搜索 ── */}
              <section class="rounded-xl border border-border-primary/50 bg-bg-secondary p-4" aria-label="记忆搜索">
                <h2 class="text-13px font-semibold mb-3">搜索记忆</h2>
                <form class="flex gap-2" onSubmit={(e) => { e.preventDefault(); void doSearch() }}>
                  <div class="relative flex-1">
                    <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
                    <input
                      type="search"
                      value={query()}
                      onInput={(e) => setQuery(e.currentTarget.value)}
                      placeholder="按内容搜索…"
                      aria-label="搜索记忆"
                      class="w-full h-9 pl-8 pr-3 rounded-lg text-13px bg-bg-primary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none placeholder:text-text-muted"
                    />
                  </div>
                  <button
                    type="submit"
                    class="h-9 px-4 rounded-lg text-13px font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors disabled:opacity-40"
                    disabled={searching()}
                  >
                    {searching() ? '搜索中…' : '搜索'}
                  </button>
                </form>
                <Show when={results()}>
                  {(r) => (
                    <div class="mt-4 space-y-2">
                      <p class="text-11px text-text-muted">{r().length} 条结果</p>
                      <For each={r().slice(0, 20)}>
                        {(e) => (
                          <article class="rounded-lg border border-border-primary/40 p-3" aria-label={`记忆 ${e.summary || e.id}`}>
                            <div class="flex items-center gap-2 text-11px text-text-muted mb-1">
                              <span class="px-1.5 py-0.5 rounded bg-nt-io-500/10 text-nt-io-700 font-medium">{e.kind}</span>
                              <Show when={e.is_pinned}><Pin class="w-3 h-3 text-nt-io-600" /></Show>
                              <span>{new Date(e.created_at).toLocaleDateString()}</span>
                              <span class="ml-auto">{e.access_count} 次访问 · 置信 {Math.round(e.confidence * 100)}%</span>
                            </div>
                            <p class="text-13px line-clamp-2">{e.content}</p>
                            <Show when={e.tags.length > 0}>
                              <div class="flex gap-1 mt-1.5 flex-wrap">
                                <For each={e.tags}>{(tag) => <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-black/5 text-text-muted">#{tag}</span>}</For>
                              </div>
                            </Show>
                          </article>
                        )}
                      </For>
                    </div>
                  )}
                </Show>
              </section>
            </div>
          </Show>
        </Show>
      </main>
    </div>
  )
}
