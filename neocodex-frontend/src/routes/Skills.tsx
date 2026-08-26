/* ════════════════════════════════════════════
   routes/Skills.tsx — 技能中心 (Phase 2 B4, 直连后端)

   吸收 Claude Code skills 面板模式：
   - 域分组卡片网格（domain 色点 + 行数 + 描述截断）
   - 搜索走后端 skill_search（防抖 300ms）
   - 点击卡片 → 详情侧滑（skill_get）
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onMount } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { ArrowLeft, Sparkles, Search, Loader2, X, FileCode2 } from 'lucide-solid'
import { clsx } from 'clsx'
import { skillList, skillSearch, skillGet, type SkillInfo } from '../api/skills'
import { errText } from '../api'

/** domain → 强调色（NT 域色系，未知域回退 nt-io） */
function domainColor(domain: string): string {
  const known: Record<string, string> = {
    core: 'bg-nt-core-500',
    mind: 'bg-nt-mind-500',
    memory: 'bg-nt-memory-500',
    world: 'bg-nt-world-500',
    act: 'bg-nt-act-500',
    io: 'bg-nt-io-500',
    shield: 'bg-nt-shield-500',
    repair: 'bg-nt-repair-500',
  }
  return known[domain.toLowerCase()] ?? 'bg-nt-io-500'
}

export function Skills() {
  const navigate = useNavigate()
  const [skills, setSkills] = createSignal<SkillInfo[]>([])
  const [total, setTotal] = createSignal(0)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [keyword, setKeyword] = createSignal('')
  const [detail, setDetail] = createSignal<SkillInfo | null>(null)
  const [detailLoading, setDetailLoading] = createSignal(false)

  let searchTimer: ReturnType<typeof setTimeout> | undefined

  async function load(q?: string) {
    setLoading(true)
    setError(null)
    try {
      if (q && q.trim()) {
        setSkills(await skillSearch(q.trim()))
        setTotal(skills().length)
      } else {
        const r = await skillList()
        setSkills(r.skills)
        setTotal(r.total)
      }
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(() => void load())

  function onInput(v: string) {
    setKeyword(v)
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(() => void load(v), 300)
  }

  async function openDetail(name: string) {
    setDetailLoading(true)
    try {
      setDetail(await skillGet(name))
    } catch (e) {
      setError(errText(e))
    } finally {
      setDetailLoading(false)
    }
  }

  /** 按域分组保持稳定排序 */
  function grouped(): [string, SkillInfo[]][] {
    const map = new Map<string, SkillInfo[]>()
    for (const s of skills()) {
      const key = s.domain || 'other'
      if (!map.has(key)) map.set(key, [])
      map.get(key)!.push(s)
    }
    return [...map.entries()].sort((a, b) => a[0].localeCompare(b[0]))
  }

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
          <Sparkles class="w-4 h-4 text-nt-io-600" />
          技能中心
        </h1>
        <span class="text-11px text-text-muted">{total()} 个技能</span>
        <div class="flex-1" />
        <div class="relative">
          <Search class="w-3.5 h-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            type="search"
            value={keyword()}
            onInput={(e) => onInput(e.currentTarget.value)}
            placeholder="搜索技能"
            aria-label="搜索技能"
            class="w-56 h-8 pl-8 pr-3 rounded-lg text-13px bg-bg-secondary border border-border-primary/50 focus:border-nt-io-500 focus:outline-none placeholder:text-text-muted"
          />
        </div>
      </header>

      {/* 内容 */}
      <main class="flex-1 overflow-y-auto p-5">
        <Show when={!loading()} fallback={
          <div class="flex items-center justify-center py-24 text-text-muted" role="status">
            <Loader2 class="w-5 h-5 animate-spin mr-2" /> 加载中…
          </div>
        }>
          <Show when={!error()} fallback={
            <div class="text-center py-24">
              <p class="text-13px text-text-muted mb-3">{error()}</p>
              <button class="text-13px text-nt-io-600 hover:underline" onClick={() => void load(keyword())}>重试</button>
            </div>
          }>
            <Show when={skills().length > 0} fallback={
              <div class="text-center py-24 text-text-muted text-13px">
                {keyword() ? '没有匹配的技能' : '未发现任何技能'}
              </div>
            }>
              <div class="max-w-5xl mx-auto space-y-6">
                <For each={grouped()}>
                  {([domain, items]) => (
                    <section aria-label={`技能域 ${domain}`}>
                      <h2 class="flex items-center gap-2 text-12px font-semibold text-text-muted mb-2 uppercase tracking-wide">
                        <span class={`w-2 h-2 rounded-full ${domainColor(domain)}`} />
                        {domain}
                        <span class="font-normal">({items.length})</span>
                      </h2>
                      <div class="grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
                        <For each={items}>
                          {(s) => (
                            <button
                              class="group text-left rounded-xl border border-border-primary/50 bg-bg-secondary p-3.5 hover:border-nt-io-500/50 hover:shadow-sm transition-all"
                              onClick={() => void openDetail(s.name)}
                              aria-label={`技能 ${s.name}`}
                            >
                              <div class="flex items-center gap-2">
                                <span class={`w-2 h-2 rounded-full shrink-0 ${domainColor(s.domain)}`} />
                                <span class="text-13px font-medium truncate">{s.name}</span>
                                <span class="ml-auto inline-flex items-center gap-0.5 text-11px text-text-muted shrink-0">
                                  <FileCode2 class="w-3 h-3" />
                                  {s.line_count}
                                </span>
                              </div>
                              <p class="text-12px text-text-muted line-clamp-2 mt-1.5 min-h-[32px]" title={s.description}>
                                {s.description || '暂无描述'}
                              </p>
                            </button>
                          )}
                        </For>
                      </div>
                    </section>
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </Show>
      </main>

      {/* 详情侧滑 */}
      <Show when={detail()}>
        {(d) => (
          <div class="fixed inset-0 z-50 flex justify-end bg-black/30" onClick={() => setDetail(null)}>
            <aside
              class="w-[400px] max-w-[90vw] h-full bg-bg-primary border-l border-border-primary shadow-xl overflow-y-auto"
              onClick={(e) => e.stopPropagation()}
              role="dialog"
              aria-label={`技能详情 ${d().name}`}
            >
              <div class="sticky top-0 bg-bg-primary/95 backdrop-blur px-5 py-4 border-b border-border-primary/40 flex items-start gap-2">
                <div class="min-w-0 flex-1">
                  <h2 class="text-14px font-semibold flex items-center gap-2">
                    <span class={`w-2 h-2 rounded-full shrink-0 ${domainColor(d().domain)}`} />
                    {d().name}
                  </h2>
                  <p class="text-11px text-text-muted mt-0.5 truncate" title={d().path}>{d().path}</p>
                </div>
                <button
                  class="p-1.5 rounded-md hover:bg-white/60 text-text-muted hover:text-text-primary"
                  onClick={() => setDetail(null)}
                  aria-label="关闭详情"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>
              <div class="p-5 space-y-4">
                <div class="flex gap-4 text-12px">
                  <span class="inline-flex items-center gap-1 text-text-muted">
                    <span class={`w-2 h-2 rounded-full ${domainColor(d().domain)}`} />
                    {d().domain}
                  </span>
                  <span class="inline-flex items-center gap-1 text-text-muted">
                    <FileCode2 class="w-3.5 h-3.5" />
                    {d().line_count} 行
                  </span>
                </div>
                <div>
                  <h3 class="text-12px font-semibold text-text-muted mb-1">描述</h3>
                  <p class="text-13px leading-relaxed">{d().description || '暂无描述'}</p>
                </div>
              </div>
            </aside>
          </div>
        )}
      </Show>

      {/* 详情加载中指示 */}
      <Show when={detailLoading()}>
        <div class="fixed bottom-5 right-5 z-50 flex items-center gap-2 rounded-lg bg-bg-primary border border-border-primary px-3 py-2 text-12px text-text-muted shadow-lg">
          <Loader2 class="w-3.5 h-3.5 animate-spin" /> 加载技能…
        </div>
      </Show>
    </div>
  )
}
