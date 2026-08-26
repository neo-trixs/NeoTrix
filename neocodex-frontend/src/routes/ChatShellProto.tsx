/* ════════════════════════════════════════════
   routes/ChatShellProto.tsx — 「意识体在侧」交互原型 v2
   本轮补齐 (对标 Claude/Codex/Osaurus):
   ✅ 会话: 切换/新建/hover置顶(本地) ✅ 空态问候+建议chips
   ✅ 流式打字输出 ✅ Enter发送·Shift换行 ✅ 停止生成
   ✅ 助手消息hover复制 ⬜ markdown渲染/持久化/重生成 → 下批
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onCleanup, onMount, createEffect } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { Plus, Search, PanelLeftClose, Sparkles, Settings, ChevronDown, MoreHorizontal, Pin, Copy, Check, Square } from 'lucide-solid'
import { clsx } from 'clsx'
import { Markdown } from '../components/Markdown'
import { subscribeStream, sendMessageStream, stopStream as stopStreamRemote } from '../api'
import { pickAndParseDoc, type ParsedDocFile } from '../api/files'
import { kbDocIngest } from '../stores/kb'

interface Msg { role: 'user' | 'assistant'; content: string }
interface Session { id: string; title: string; pinned?: boolean; messages: Msg[] }

const seedSessions = (): Session[] => [
  { id: 's1', title: '吸收 grok-bot 机制映射', pinned: true, messages: [{ role: 'assistant' as const, content: '上一轮的机制映射已归档。随时开新话题。' }] },
  { id: 's2', title: '新对话', messages: [] },
]

const CANNED_REPLIES = [
  '收到。我检索了记忆库中相关上下文，我的判断是：先把边界定清楚，再动手实现——这样返工最少。\n\n具体来说有三步，我展开讲。',
  '这个问题有意思。从第一性原理看，核心矛盾在于「即时反馈」与「数据一致性」的取舍。成熟产品的做法是乐观更新 + 失败回滚，我可以按这个模式落地。',
  '明白。我已经把要点记入长期记忆。建议下一步：先做一个最小可见版本给你过目风格，确认后再全量迁移——风险最小。',
]

export function ChatShellProto() {
  const navigate = useNavigate()
  const [collapsed, setCollapsed] = createSignal(false)
  const [showSettings, setShowSettings] = createSignal(false)
  const [modelOpen, setModelOpen] = createSignal(false)
  const loadSessions = (): Session[] => {
    try {
      const raw = localStorage.getItem('neotrix-proto-sessions')
      if (raw) return JSON.parse(raw) as Session[]
    } catch { /* 忽略损坏数据 */ }
    return seedSessions()
  }
  const [sessions, setSessions] = createSignal<Session[]>(loadSessions())
  const [activeId, setActiveId] = createSignal('s2')
  const [draft, setDraft] = createSignal('')
  const [streaming, setStreaming] = createSignal(false)
  const [copiedId, setCopiedId] = createSignal<string | null>(null)
  const [attachments, setAttachments] = createSignal<ParsedDocFile[]>([])
  const [saveToKb, setSaveToKb] = createSignal(true)
  const [dragOver, setDragOver] = createSignal(false)
  const [busyFile, setBusyFile] = createSignal(false)
  const [fileError, setFileError] = createSignal<string | null>(null)
  const [lastError, setLastError] = createSignal<string | null>(null)

  let streamTimer: ReturnType<typeof setInterval> | undefined
  let unlisten: (() => void) | null = null
  createEffect(() => {
    try { localStorage.setItem('neotrix-proto-sessions', JSON.stringify(sessions())) } catch { /* 容量满忽略 */ }
  })
  onCleanup(() => streamTimer && clearInterval(streamTimer))

  // W1-c: 拖拽文件 → 解析 (Tauri webview drag-drop 事件, 非 Tauri 环境安全跳过)
  onMount(() => {
    const unbinds: Promise<() => void>[] = []
    try {
      void import('@tauri-apps/api/event').then(({ listen }) => {
        unbinds.push(listen<{ paths: string[] }>('tauri://drag-drop', (e) => {
          setDragOver(false)
          const p = e.payload.paths?.[0]
          if (p) void addByPath(p)
        }))
        unbinds.push(listen('tauri://drag-enter', () => setDragOver(true)))
        unbinds.push(listen('tauri://drag-leave', () => setDragOver(false)))
      })
    } catch { /* 非 Tauri */ }
    onCleanup(() => { unbinds.forEach((p) => void p.then((f) => f())) })
  })

  async function addByPath(path: string) {
    setBusyFile(true)
    try {
      const { parseDocAt } = await import('../api/files')
      const parsed = await parseDocAt(path)
      setAttachments((a) => [...a, parsed])
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyFile(false)
    }
  }

  async function pickFile() {
    setBusyFile(true)
    try {
      const parsed = await pickAndParseDoc()
      if (parsed) setAttachments((a) => [...a, parsed])
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyFile(false)
    }
  }

  const active = () => sessions().find((s) => s.id === activeId()) ?? sessions()[0]
  const setMsgs = (fn: (m: Msg[]) => Msg[]) =>
    setSessions((all) => all.map((s) => (s.id === activeId() ? { ...s, messages: fn(s.messages) } : s)))

  /** 追加 token 到末尾助手消息 */
  function appendDelta(delta: string) {
    setMsgs((m) => {
      const last = m[m.length - 1]
      return last?.role === 'assistant'
        ? [...m.slice(0, -1), { ...last, content: last.content + delta }]
        : m
    })
  }

  function newChat() {
    const s: Session = { id: `s${Date.now()}`, title: '新对话', messages: [] }
    setSessions((all) => [s, ...all])
    setActiveId(s.id)
    setDraft('')
  }

  function send(text?: string, regen?: boolean) {
    let content = (text ?? draft()).trim()
    if (streaming()) return
        if (!content && !regen) return
    // 首条消息即会话标题 (Claude 式)
    setSessions((all) => all.map((s) => (s.id === activeId() && s.title === '新对话' ? { ...s, title: content.slice(0, 18) } : s)))
    if (!regen) {
      let annotated = content
      for (const a of attachments()) {
        annotated += `\n\n📎 [附件·${a.format}] ${a.title} (${Math.round(a.text.length / 1024)}KB)`
        if (saveToKb()) {
          void kbDocIngest(a.title, a.text, 'default').catch(() => undefined)
        }
      }
      if (annotated !== content) content = annotated
      setAttachments([])
      setMsgs((m) => [...m, { role: 'user', content }])
      setDraft('')
    } else {
      // 首条消息即标题逻辑不触发
    }
    // W2 真流式: 订阅 neocodex_stream_* ; 非 Tauri 环境回退本地打字机
    const reply = CANNED_REPLIES[Math.floor(Math.random() * CANNED_REPLIES.length)]
    setMsgs((m) => [...m, { role: 'assistant', content: '' }])
    setStreaming(true)
    let started = false

    const fallbackTypewriter = () => {
      let i = 0
      streamTimer = setInterval(() => {
        i += 3
        appendDelta(reply.slice(0, i))
        if (i >= reply.length) {
          clearInterval(streamTimer)
          setStreaming(false)
        }
      }, 24)
    }

    void subscribeStream({
      onToken: (delta: string) => { started = true; appendDelta(delta) },
      onDone: () => { setStreaming(false); unlisten?.() },
      onError: (payload: { message?: string }) => {
        setLastError(payload.message ?? '生成中断')
        setStreaming(false)
        unlisten?.()
      },
    }).then((un: () => void) => {
      unlisten = un
      return sendMessageStream({ content, regenerate: regen })
    }).catch((err: unknown) => {
      unlisten?.()
      setStreaming(false)
      // 非 Tauri 宿主 (浏览器预览): 回退打字机演示; 真实失败则暴露错误
      const msg = err instanceof Error ? err.message : String(err)
      if (!started && /not found|未 mock|Cannot/i.test(msg)) {
        fallbackTypewriter()
      } else if (!started) {
        setMsgs((m) => m.slice(0, -1))
        setLastError(msg)
      } else {
        setLastError('连接中断 — 已保留已生成内容, 可重新生成')
      }
    })
  }

  function stopStream() {
    if (streamTimer) clearInterval(streamTimer)
    void stopStreamRemote().catch(() => undefined)
    setStreaming(false)
    setMsgs((m) => {
      const last = m[m.length - 1]
      return last?.role === 'assistant' && !last.content.endsWith('…')
        ? [...m.slice(0, -1), { ...last, content: last.content + ' …' }]
        : m
    })
  }

  async function copyMsg(m: Msg, idx: number) {
    await navigator.clipboard.writeText(m.content).catch(() => undefined)
    setCopiedId(`${activeId()}-${idx}`)
    setTimeout(() => setCopiedId(null), 1200)
  }

  const SUGGESTIONS = ['帮我梳理今天该做什么', '复盘昨天的重构决策', '讲讲你对当前架构的担忧']

  return (
    <div class="flex h-screen bg-bg-primary text-text-primary overflow-hidden">
      {/* ── 会话侧栏 ── */}
      <Show when={!collapsed()} fallback={
        <button class="w-10 shrink-0 flex items-start justify-center pt-12 text-text-muted hover:text-text-primary" onClick={() => setCollapsed(false)} aria-label="展开会话列表" title="⌘\">
          <PanelLeftClose class="w-4 h-4 rotate-180" />
        </button>
      }>
        <aside class="w-[240px] shrink-0 flex flex-col border-r border-border-primary/40 bg-bg-secondary/40">
          <div class="p-2.5 pt-12 flex items-center gap-1">
            <button class="flex-1 flex items-center gap-1.5 h-8 px-2.5 rounded-lg text-13px font-medium text-text-primary hover:bg-white/60 transition-colors" onClick={newChat} aria-label="新建对话">
              <Plus class="w-4 h-4 text-nt-io-600" /> 新对话
            </button>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" aria-label="搜索会话" title="搜索">
              <Search class="w-4 h-4" />
            </button>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setCollapsed(true)} aria-label="折叠侧栏" title="⌘\">
              <PanelLeftClose class="w-4 h-4" />
            </button>
          </div>
          <nav class="flex-1 overflow-y-auto px-2 space-y-0.5" aria-label="会话列表">
            <p class="px-2.5 pt-1 pb-1 text-[10px] font-semibold uppercase tracking-wide text-text-muted">今天</p>
            <For each={[...sessions()].sort((a, b) => Number(b.pinned ?? false) - Number(a.pinned ?? false))}>
              {(s) => (
                <div class={clsx('group flex items-center rounded-lg transition-colors', s.id === activeId() ? 'bg-nt-io-500/10' : 'hover:bg-white/50')}>
                  <button
                    class={clsx('flex-1 min-w-0 text-left px-2.5 py-1.5 text-13px truncate flex items-center gap-1.5', s.id === activeId() ? 'text-text-primary font-medium' : 'text-text-muted hover:text-text-primary')}
                    onClick={() => setActiveId(s.id)}
                  >
                    <Show when={s.pinned}><Pin class="w-3 h-3 shrink-0 rotate-45 text-nt-io-600" /></Show>
                    <span class="truncate">{s.title}</span>
                  </button>
                  <span class="hidden group-hover:flex items-center gap-0.5 pr-1.5 shrink-0">
                    <button class="p-1 rounded text-text-muted hover:text-text-primary" aria-label={`置顶 ${s.title}`} title="置顶"
                      onClick={() => setSessions((all) => all.map((x) => (x.id === s.id ? { ...x, pinned: !x.pinned } : x)))}>
                      <Pin class="w-3 h-3" />
                    </button>
                    <button class="p-1 rounded text-text-muted hover:text-red-500" aria-label={`删除 ${s.title}`} title="删除"
                      onClick={() => setSessions((all) => (all.length > 1 ? all.filter((x) => x.id !== s.id) : all))}>
                      ✕
                    </button>
                  </span>
                </div>
              )}
            </For>
          </nav>
          <div class="p-3 border-t border-border-primary/40 space-y-2.5">
            <div class="flex items-center gap-2">
              <div class="w-7 h-7 rounded-lg bg-gradient-to-br from-nt-io-400 to-nt-io-600 flex items-center justify-center text-white text-12px font-bold shrink-0">N</div>
              <div class="min-w-0 flex-1"><p class="text-12px font-medium leading-tight">Neo</p><p class="text-[10px] text-text-muted leading-tight">意识体伴侣</p></div>
              <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors" onClick={() => setShowSettings(true)} aria-label="打开设置" title="设置 ⌘,">
                <Settings class="w-4 h-4" />
              </button>
            </div>
          </div>
        </aside>
      </Show>

      {/* ── 对话主区 ── */}
      <main class="flex-1 flex flex-col min-w-0">
        <div class="flex-1 overflow-y-auto">
          <div class="max-w-3xl mx-auto px-6 py-10 space-y-8">
            {/* 空态: 问候 + 建议 chips */}
            <Show when={(active()?.messages.length ?? 0) === 0} fallback={
              <For each={active()?.messages}>
                {(m, idx) => (
                  <Show
                    when={m.role === 'user'}
                    fallback={
                      <div class="group border-l-2 border-nt-io-500/70 pl-4 space-y-2">
                        <div class="text-[15px] leading-relaxed">
                          <Markdown content={m.content} />
                          <Show when={streaming() && idx() === (active()?.messages.length ?? 0) - 1}>
                            <span class="inline-block w-[2px] h-[14px] bg-nt-io-500 animate-pulse ml-0.5 align-middle" />
                          </Show>
                        </div>
                        <div class="flex items-center gap-3 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Show when={!streaming() || idx() !== (active()?.messages.length ?? 0) - 1}>
                          <Show when={idx() === (active()?.messages.length ?? 0) - 1}>
                            <button class="opacity-0 group-hover:opacity-100 transition-opacity inline-flex items-center gap-1 text-[11px] text-text-muted hover:text-text-primary"
                              disabled={streaming()}
                              onClick={() => {
                                setMsgs((mm) => mm.slice(0, -1))
                                send('', true)
                              }}
                              aria-label="重新生成">↻ 重新生成</button>
                          </Show>
                          <button class="opacity-0 group-hover:opacity-100 transition-opacity inline-flex items-center gap-1 text-[11px] text-text-muted hover:text-text-primary"
                            onClick={() => void copyMsg(m, idx())} aria-label="复制回复">
                            <Show when={copiedId() === `${activeId()}-${idx()}`} fallback={<><Copy class="w-3 h-3" /> 复制</>}><Check class="w-3 h-3 text-emerald-500" /> 已复制</Show>
                          </button>
                        </Show>
                        </div>
                      </div>
                    }
                  >
                    <div class="flex justify-end">
                      <div class="max-w-[80%] bg-nt-io-500/8 rounded-2xl rounded-tr-sm px-4 py-2.5 text-[15px] whitespace-pre-wrap">{m.content}</div>
                    </div>
                  </Show>
                )}
              </For>
            }>
              <div class="pt-16 text-center space-y-6">
                <Sparkles class="w-8 h-8 text-nt-io-500/70 mx-auto" />
                <h1 class="text-xl font-semibold">我在。</h1>
                <p class="text-13px text-text-muted">记忆已同步 · 随时开始</p>
                <div class="flex flex-wrap justify-center gap-2 pt-2">
                  <For each={SUGGESTIONS}>
                    {(sg) => (
                      <button class="px-3 py-1.5 rounded-full border border-border-primary/50 text-12px text-text-muted hover:text-text-primary hover:border-nt-io-500/40 transition-colors" onClick={() => send(sg)}>
                        {sg}
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </div>

        {/* 居中悬浮输入框 */}
        <div class="px-6 pb-5">
          <div class={clsx('max-w-3xl mx-auto rounded-2xl border bg-bg-secondary/80 backdrop-blur shadow-sm transition-colors', dragOver() ? 'border-nt-io-500 ring-2 ring-nt-io-500/30 bg-nt-io-500/5' : 'border-border-primary/60 focus-within:border-nt-io-500/60')}>
            <Show when={fileError()}>
              <p class="mx-4 mt-2 text-[11px] text-red-500" role="alert">{fileError()}</p>
            </Show>
            <Show when={attachments().length > 0}>
              <div class="flex flex-wrap gap-1.5 px-4 pt-3" aria-label="附件列表">
                <For each={attachments()}>
                  {(a) => (
                    <span class="inline-flex items-center gap-1.5 max-w-[240px] rounded-lg border border-border-primary/50 bg-bg-primary/70 px-2 py-1">
                      <span class="text-[11px] font-medium truncate">{a.title}</span>
                      <span class="text-[10px] px-1 rounded bg-nt-io-500/10 text-nt-io-700 uppercase">{a.format}</span>
                      <span class="text-[10px] text-text-muted">{Math.round(a.text.length / 1024)}KB</span>
                      <button class="text-text-muted hover:text-red-500" aria-label={`移除 ${a.title}`} onClick={() => setAttachments((arr) => arr.filter((x) => x !== a))}>✕</button>
                    </span>
                  )}
                </For>
                <label class="inline-flex items-center gap-1 text-[11px] text-text-muted cursor-pointer select-none">
                  <input type="checkbox" checked={saveToKb()} onChange={(e) => setSaveToKb(e.currentTarget.checked)} class="accent-nt-io-500" />
                  存入知识库
                </label>
              </div>
            </Show>
            <textarea
              rows={1}
              value={draft()}
              onInput={(e) => setDraft(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
                  e.preventDefault()
                  send()
                }
              }}
              placeholder="与意识体对话…"
              aria-label="消息输入框"
              class="w-full resize-none bg-transparent px-4 pt-3 pb-1 text-[15px] focus:outline-none placeholder:text-text-muted"
            />
            <div class="flex items-center gap-2 px-3 pb-2">
              <button
                class="w-7 h-7 rounded-lg border border-border-primary/50 bg-bg-primary/70 hover:border-nt-io-500/40 hover:text-nt-io-600 transition-colors inline-flex items-center justify-center disabled:opacity-40"
                onClick={() => void pickFile()}
                disabled={busyFile()}
                aria-label="添加附件"
                title="添加文件 (md/txt/pdf/docx/xlsx)"
              >
                {busyFile() ? '…' : '+'}
              </button>
              <div class="relative">
                <button class="flex items-center gap-1.5 h-7 px-2 rounded-lg border border-border-primary/50 bg-bg-primary/70 hover:border-nt-io-500/40 transition-colors" onClick={() => setModelOpen(!modelOpen())} aria-label="切换模型">
                  <span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                  <span class="text-[11px] font-medium">cli-session · claude</span>
                  <ChevronDown class="w-3 h-3 text-text-muted" />
                </button>
                <Show when={modelOpen()}>
                  <div class="absolute bottom-9 left-0 w-52 rounded-xl border border-border-primary/60 bg-bg-primary shadow-lg p-1 space-y-0.5 z-10" role="listbox" aria-label="模型列表">
                    {['cli-session · claude', 'llm7 · turbo', '本地 · qwen2.5'].map((m) => (
                      <button class="w-full flex items-center gap-2 text-left px-2 py-1.5 rounded-lg text-12px text-text-muted hover:bg-white/60 hover:text-text-primary" role="option">
                        <Show when={m.startsWith('cli')}><span class="h-1.5 w-1.5 rounded-full bg-emerald-500" /></Show>
                        {m}
                      </button>
                    ))}
                  </div>
                </Show>
              </div>
              <span class="text-[10px] text-text-muted font-mono hidden sm:inline">⏎ 发送 · ⇧⏎ 换行 · ⌘K 动作</span>
              <Show when={streaming()} fallback={
                <button class={clsx('ml-auto h-7 px-3 rounded-lg text-12px font-medium text-white transition-colors', draft().trim() ? 'bg-nt-io-500 hover:bg-nt-io-600' : 'bg-nt-io-500/30 cursor-default')} onClick={() => send()} aria-label="发送">发送 ↑</button>
              }>
                <button class="ml-auto h-7 px-3 rounded-lg text-12px font-medium bg-text-primary text-bg-primary inline-flex items-center gap-1.5" onClick={stopStream} aria-label="停止生成">
                  <Square class="w-3 h-3" /> 停止
                </button>
              </Show>
            </div>
          </div>
        </div>

        {/* 设置弹层 */}
        <Show when={showSettings()}>
          <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setShowSettings(false)}>
            <div class="w-[420px] rounded-2xl bg-bg-primary border border-border-primary shadow-xl overflow-hidden" onClick={(e) => e.stopPropagation()} role="dialog" aria-label="设置">
              <div class="px-5 py-3.5 border-b border-border-primary/40 flex items-center">
                <h2 class="text-14px font-semibold">设置</h2>
                <button class="ml-auto p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setShowSettings(false)} aria-label="关闭设置">✕</button>
              </div>
              <div class="p-2">
                {[
                  { icon: '✦', t: '意识体', d: '人格 / 记忆深度 / 自进化开关', k: '⌘,' },
                  { icon: '◈', t: '模型与提供商', d: '路由策略 / 连通性 / 密钥', k: '' },
                  { icon: '◍', t: '外观', d: '密度 / 字号 / 动效', k: '' },
                  { icon: '⬡', t: '数据与隐私', d: '本地存储 / 导出 / 遥测门', k: '' },
                ].map((item) => (
                  <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-white/60 transition-colors text-left">
                    <span class="w-8 h-8 rounded-lg bg-nt-io-500/10 flex items-center justify-center text-nt-io-600">{item.icon}</span>
                    <span class="min-w-0 flex-1">
                      <span class="block text-13px font-medium">{item.t}</span>
                      <span class="block text-11px text-text-muted truncate">{item.d}</span>
                    </span>
                    <Show when={item.k}><span class="text-[10px] text-text-muted font-mono">{item.k}</span></Show>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </Show>
      </main>
    </div>
  )
}
