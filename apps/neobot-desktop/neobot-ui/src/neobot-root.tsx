/**
 * NeoBot 自持根组件 —— **对话区由本仓实现**。
 *
 * # 为什么要自己写
 *
 * 上游 99% 的屏幕时间在 `<iframe>` 里，内容是 `source/deepseek-harness`
 * （空 submodule）起的那个 Web UI。本仓没有那个运行时，所以：
 * 上游的外壳（导航、设置、工作台、恢复页）1:1 复用，**中间的对话区自己实现**。
 *
 * 这是分工，不是缺口 —— 见 `frontend/VENDOR.md`。
 *
 * # 数据一律走 API 平台，不自己 invent 端点
 *
 * 每个能力都能在**设置 → API** 页签里查到它的存活状态。
 * 写一个界面上有、契约里没有的调用，就是制造隐性接口 ——
 * 而「隐性接口」正是本轮花大力气要消灭的那类东西。
 *
 * # 主题：chrome 走语义 token，气泡固定
 *
 * 列表/顶栏/边框/输入框用壳语义类（text-ink/text-muted/border-line/
 * bg-panel/bg-panel-hover/bg-btn-fill/text-btn-ink，随 `html[data-theme]` 翻色）。
 * 消息气泡是内容色（浅蓝/浅灰+深字），深浅模式下都可读，故意固定 ——
 * `dark:` 变体在此无用（tailwind `darkMode:'class'` 要 `.dark` 类，
 * 而应用只写 `dataset.theme`），裸 hex 只许出现在气泡/徽标两处。
 */

import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './vendor/openghost/tex.js'
import './vendor/openghost/markdown.js'
import './vendor/openghost/highlight.js'
import { installOpenghostShim } from './vendor/openghost/shim.ts'
import './ui/nb-markdown.css'

interface ConvoView {
  id: string
  kind: string
  title: string
  members: string[]
  task_count: number
  last_active: string
  muted: boolean
  unread: number
}

type Msg = { who: 'me' | 'bot'; text: string; ts: string; failed?: boolean }

/** 库侧记忆视图（`neobot_memory_list` 返回值，蛇形同名）。 */
interface MemoryView {
  lines: string[]
  bytes: number
  cap: number
  revisions: number
}

/** 库侧汇总（`neobot_usage_summary` 返回值，蛇形同名；行明细暂不展示）。 */
interface UsageSummary {
  days: number
  input: number
  cached: number
  written: number
  output: number
  requests: number
  tokens: number
}
/** 库侧成员（`neobot_member_list` 返回值，蛇形同名）。 */
interface MemberView {
  id: string
  kind: string
  display: string
}

/** 库侧 `CapabilitySnapshot`（`neobot_core_capabilities` 返回值，蛇形同名）。 */
interface CapabilitySnapshot {
  crystal_version: string
  tool_count: number
  model: string
  model_source: string
}
/** 库侧 `ChatMessage`（`neobot_convo_messages` 返回值，蛇形同名）。 */
interface ChatMessage {
  id: string
  convo_id: string
  role: string
  text: string
  created_at: string
}

/**
 * bot 消息转 HTML（OpenGhost 渲染引擎，见 vendor/VENDOR-OPENGHOST.md）。
 *
 * ⛔ 渲染器自转义（`<`/`>`/`&`/`"` 全转，链接 escaped + noopener），
 * 模型输出可直接进 innerHTML —— 与上游同信任等级。
 * vendor 缺席（加载失败）时回纯文本，界面不白屏。
 */
function renderBot(text: string): string | null {
  try {
    const html = window.Markdown?.render(text)
    return typeof html === 'string' && html ? html : null
  } catch {
    return null
  }
}

async function copyCode(button: HTMLButtonElement): Promise<void> {
  const pre = button.closest('.md-code')?.querySelector('pre code')
  const text = pre?.textContent ?? ''
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // 剪贴板被拒（非安全上下文等）：静默收手，不弹错。
    // 报错会让用户以为「复制功能坏了」，而实际只是这次不许写。
    return
  }
  button.classList.add('is-copied')
  window.setTimeout(() => button.classList.remove('is-copied'), 1200)
}

function relTime(iso: string): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return ''
  const mins = Math.floor((Date.now() - t) / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins} 分钟前`
  if (mins < 1440) return `${Math.floor(mins / 60)} 小时前`
  if (mins < 2880) return '昨天'
  return `${Math.floor(mins / 1440)} 天前`
}

/** 气泡上的时刻（`HH:MM`；跨天补日期）。 */
function clockOf(iso: string): string {
  const t = new Date(iso)
  if (Number.isNaN(t.getTime())) return ''
  const hm = `${String(t.getHours()).padStart(2, '0')}:${String(t.getMinutes()).padStart(2, '0')}`
  const today = new Date()
  const sameDay = t.toDateString() === today.toDateString()
  return sameDay ? hm : `${t.getMonth() + 1}/${t.getDate()} ${hm}`
}

const nowIso = (): string => new Date().toISOString()

export function NeoBotRoot() {
  const [convos, setConvos] = useState<ConvoView[] | null>(null)
  const [err, setErr] = useState('')
  const [sel, setSel] = useState<string | null>(null)
  const [msgs, setMsgs] = useState<Msg[]>([])
  const [draft, setDraft] = useState('')
  const [busy, setBusy] = useState(false)
  const [histLoading, setHistLoading] = useState(false)
  // 能力快照：调不到就 null，顶栏直接不渲染 —— 「不可用就不渲染」。
  // ⛔ 不设静态默认值：两处默认值各自猜，正是本缺口的病因。
  const [caps, setCaps] = useState<CapabilitySnapshot | null>(null)
  // 今日用量：同上，缺席不渲染。发送完成后重拉（send 落库在后）。
  const [usageToday, setUsageToday] = useState<number | null>(null)
  const reloadUsage = useCallback(() => {
    void invoke<UsageSummary>('neobot_usage_summary', { days: 1 })
      .then(s => setUsageToday(s.tokens))
      .catch(() => setUsageToday(null))
  }, [])
  useEffect(reloadUsage, [reloadUsage])

  // 记忆面板：默认收起（不占首屏），展开才拉。
  // ⛔ 拉不到就 null 且面板内明说「读不到」，不渲染成「还没有记忆」——
  //    那两种是两种处境（§4 教训 12 的同一个坑）。
  const [memOpen, setMemOpen] = useState(false)
  const [mem, setMem] = useState<MemoryView | null>(null)
  const [memErr, setMemErr] = useState('')
  const [memDraft, setMemDraft] = useState('')
  const [memBusy, setMemBusy] = useState(false)
  const reloadMem = useCallback(() => {
    void invoke<MemoryView>('neobot_memory_list')
      .then((v) => {
        setMem(v)
        setMemErr('')
      })
      .catch((e) => {
        setMem(null)
        setMemErr(String(e).slice(0, 160))
      })
  }, [])
  useEffect(() => {
    if (memOpen) reloadMem()
  }, [memOpen, reloadMem])
  const addMemory = useCallback(async () => {
    const text = memDraft.trim()
    if (!text || memBusy) return
    setMemBusy(true)
    try {
      await invoke<boolean>('neobot_memory_add', { text })
      setMemDraft('')
      reloadMem()
    } catch (e) {
      setMemErr(String(e).slice(0, 160))
    } finally {
      setMemBusy(false)
    }
  }, [memDraft, memBusy, reloadMem])
  const undoMemory = useCallback(async () => {
    if (memBusy) return
    setMemBusy(true)
    try {
      const done = await invoke<boolean>('neobot_memory_undo')
      // ⛔ false 不是错：本来就没什么可撤的，别弹错误吓人。
      if (!done) setMemErr('')
      reloadMem()
    } catch (e) {
      setMemErr(String(e).slice(0, 160))
    } finally {
      setMemBusy(false)
    }
  }, [memBusy, reloadMem])

  const reload = useCallback(() => {
    void invoke<ConvoView[]>('neobot_convo_list')
      .then((v) => {
        setConvos(v)
        setErr('')
        setSel((s) => s ?? v[0]?.id ?? null)
      })
      .catch((e) => {
        // ⛔ 读失败就**说读失败**，不留空列表假装「没有会话」——
        //    那两种情况对用户是两种处境，混起来会让人白折腾。
        setConvos([])
        setErr(String(e))
      })
  }, [])

  // 侧栏筛选：标题/成员/会话 id 任一命中即保留（大小写不敏感）。
  // ⛔ 只筛不重排：排序是后端的 last_active 语义，前端另排会盖掉它。
  const [q, setQ] = useState('')
  const shown = useMemo(() => {
    const list = convos ?? []
    const needle = q.trim().toLowerCase()
    if (!needle) return list
    return list.filter(
      c =>
        c.title.toLowerCase().includes(needle) ||
        c.id.toLowerCase().includes(needle) ||
        c.members.some(m => m.toLowerCase().includes(needle)),
    )
  }, [convos, q])

  // 新建对话：⛔ 成员必须已登记（库侧 `create_conversation` 会校验），
  // 而建会话前得先有成员 —— 所以表单里能就地登记，不必去设置面板绕一圈。
  const [newOpen, setNewOpen] = useState(false)
  const [newTitle, setNewTitle] = useState('')
  const [newMember, setNewMember] = useState('')
  const [members, setMembers] = useState<MemberView[]>([])
  const [newErr, setNewErr] = useState('')
  const [creating, setCreating] = useState(false)
  const reloadMembers = useCallback(() => {
    void invoke<MemberView[]>('neobot_member_list')
      .then(setMembers)
      .catch(() => setMembers([]))
  }, [])
  const createConvo = useCallback(async () => {
    const title = newTitle.trim()
    const member = newMember.trim()
    if (!title || creating) return
    setCreating(true)
    setNewErr('')
    try {
      // 填了成员就先登记（幂等），再建会话 —— 否则后端会以
      // 「成员不存在」拒绝，而那句话对用户毫无可操作性。
      if (member) {
        await invoke('neobot_member_add', { id: member, kind: 'human' })
        reloadMembers()
      }
      const id = await invoke<string>('neobot_convo_group', { title, members: member ? [member] : [] })
      setNewTitle('')
      setNewMember('')
      setNewOpen(false)
      reload()
      setSel(id)
    } catch (e) {
      setNewErr(String(e).slice(0, 160))
    } finally {
      setCreating(false)
    }
  }, [newTitle, newMember, creating, reload, reloadMembers])

  useEffect(reload, [reload])
  // 切会话即换历史：只换标题不换消息流的是串台。
  // ⛔ 加载完成前禁发：乐观气泡与迟到的历史会拼出重复，拦住比事后去重诚实。
  useEffect(() => {
    if (!sel) {
      setMsgs([])
      return
    }
    let alive = true
    setHistLoading(true)
    void invoke<ChatMessage[]>('neobot_convo_messages', { convo_id: sel })
      .then((rows) => {
        if (!alive) return
        setMsgs(rows.map((r): Msg => ({
          who: r.role === 'user' ? 'me' : 'bot',
          text: r.text,
          ts: r.created_at,
        })))
      })
      .catch(() => {
        if (alive) setMsgs([])
      })
      .finally(() => {
        if (alive) setHistLoading(false)
      })
    return () => {
      alive = false
    }
  }, [sel])
  // 挂载即记一条：这是「对话区确实渲染了」的唯一**运行时**证据。
  // 之前只能靠「日志里没有启动错误」反推 —— 那是**否定式**证据，
  // 而「没有报错」和「渲染了」之间没有必然关系。
  useEffect(() => {
    installOpenghostShim()
    void invoke('log_frontend', { level: 'info', target: 'neobot-root', message: 'mounted' }).catch(() => {})
    // 能力矩阵以此为准（旧自研 UI 的静态默认矩阵已随旧 UI 删除）。
    void invoke<CapabilitySnapshot>('neobot_core_capabilities')
      .then(setCaps)
      .catch(() => setCaps(null))
  }, [])

  async function send() {
    const text = draft.trim()
    if (!text || busy || histLoading) return
    setDraft('')
    setMsgs(m => [...m, { who: 'me', text, ts: nowIso() }])
    setBusy(true)
    try {
      // convo_id 缺席（无会话时）= 脱离会话手动跑，后端不落库。
      const r = await invoke<{ output?: string, text?: string }>('neobot_send', { convo_id: sel ?? undefined, text })
      setMsgs(m => [...m, { who: 'bot', text: r?.output ?? r?.text ?? '（无输出）', ts: nowIso() }])
    } catch (e) {
      // ⛔ 失败气泡标成 failed：那样才能给「重发」，也不至于和正常回复混淆。
      setMsgs(m => [...m, { who: 'bot', text: `发送失败：${String(e).slice(0, 200)}`, ts: nowIso(), failed: true }])
    } finally {
      setBusy(false)
      reloadUsage()
    }
  }

  /** 重发：拿失败气泡的原文再发一次（失败那条替换掉，不留残骸）。 */
  const retry = useCallback(
    async (idx: number) => {
      const bad = msgs[idx]
      if (!bad?.failed || busy) return
      setMsgs(m => m.filter((_, i) => i !== idx))
      setBusy(true)
      try {
        const r = await invoke<{ output?: string, text?: string }>('neobot_send', {
          convo_id: sel ?? undefined,
          text: bad.text.replace(/^发送失败：/, ''),
        })
        setMsgs(m => [...m, { who: 'bot', text: r?.output ?? r?.text ?? '（无输出）', ts: nowIso() }])
      } catch (e) {
        setMsgs(m => [...m, { who: 'bot', text: `发送失败：${String(e).slice(0, 200)}`, ts: nowIso(), failed: true }])
      } finally {
        setBusy(false)
        reloadUsage()
      }
    },
    [msgs, busy, sel, reloadUsage],
  )

  const current = convos?.find(c => c.id === sel) ?? null
  // 代码块复制按钮是渲染器吐的（vendor），事件委托在这收 ——
  // 逐个绑定的话，60 条消息就是 60 个监听器，且新气泡还要补绑。
  const msgsRef = useRef<HTMLDivElement | null>(null)
  useEffect(() => {
    const host = msgsRef.current
    if (!host) return
    const onClick = (e: MouseEvent) => {
      const btn = (e.target as HTMLElement).closest?.('.md-copy')
      if (btn instanceof HTMLButtonElement) void copyCode(btn)
    }
    host.addEventListener('click', onClick)
    return () => host.removeEventListener('click', onClick)
  }, [])

  // 自动滚到底：**只在用户本来就在底部时**才滚。
  // ⛔ 无条件 scrollTop = 高度会把人从正在读的历史里硬拽走 —— 那是「自作聪明」。
  const [pinned, setPinned] = useState(true)
  const scrollToEnd = useCallback(() => {
    const host = msgsRef.current
    if (host) host.scrollTop = host.scrollHeight
    setPinned(true)
  }, [])
  useEffect(() => {
    const host = msgsRef.current
    if (!host) return
    const onScroll = () => {
      // 阈值 48px：触底时 scrollTop 与 scrollHeight 之差常有几像素抖动。
      setPinned(host.scrollHeight - host.scrollTop - host.clientHeight < 48)
    }
    host.addEventListener('scroll', onScroll, { passive: true })
    return () => host.removeEventListener('scroll', onScroll)
  }, [])
  useEffect(() => {
    if (!pinned) return
    const host = msgsRef.current
    if (host) host.scrollTop = host.scrollHeight
  }, [msgs, pinned])

  // 输入框随内容增高（上限 5 行后转为滚动）。
  // ⛔ 占位符承诺了「Shift+Enter 换行」，框却永远一行 ⇒ 用户看不见自己打的第二行。
  const taRef = useRef<HTMLTextAreaElement | null>(null)
  useEffect(() => {
    const ta = taRef.current
    if (!ta) return
    ta.style.height = 'auto'
    ta.style.height = `${Math.min(ta.scrollHeight, 160)}px`
  }, [draft])

  return (
    <div className="flex min-h-0 flex-1" data-testid="neobot-root">
      {/* 侧栏：筛选 + 新建在上，列表独立滚动，记忆面板钉在底部。
          ⛔ 三段必须拆开：以前整栏一个 overflow-y-auto + `mt-auto`，
          列表一长记忆面板就被一起滚走，`mt-auto` 在滚动容器里本就无效。 */}
      <aside className="flex min-h-0 w-64 shrink-0 flex-col border-r border-line">
        <div className="shrink-0 space-y-1.5 p-2">
          <div className="flex gap-1.5">
            <input
              value={q}
              onChange={e => setQ(e.target.value)}
              placeholder="搜索会话…"
              aria-label="搜索会话"
              className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1 text-[12px] text-ink outline-none focus:border-info-hover"
            />
            <button
              type="button"
              onClick={() => {
                setNewOpen(o => !o)
                if (!newOpen) reloadMembers()
              }}
              aria-expanded={newOpen}
              title="新建对话"
              className="h-[26px] w-[26px] shrink-0 rounded-lg border border-line text-[15px] leading-none text-muted hover:bg-panel-hover"
            >
              {newOpen ? '×' : '+'}
            </button>
          </div>
          {newOpen && (
            <div className="space-y-1.5 rounded-lg bg-panel p-2">
              <input
                value={newTitle}
                onChange={e => setNewTitle(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    void createConvo()
                  }
                }}
                placeholder="标题（如：海豚调试）"
                aria-label="会话标题"
                className="w-full rounded-lg border border-line bg-canvas px-2 py-1 text-[12px] text-ink outline-none focus:border-info-hover"
              />
              <input
                value={newMember}
                onChange={e => setNewMember(e.target.value)}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    e.preventDefault()
                    void createConvo()
                  }
                }}
                list="nb-members"
                placeholder="成员 id（可空）"
                aria-label="成员 id"
                className="w-full rounded-lg border border-line bg-canvas px-2 py-1 text-[12px] text-ink outline-none focus:border-info-hover"
              />
              <datalist id="nb-members">
                {members.map(m => (
                  <option key={m.id} value={m.id}>{m.display}</option>
                ))}
              </datalist>
              {newErr && <div className="break-all text-[11px] text-[#c33b38]">{newErr}</div>}
              <button
                type="button"
                onClick={() => void createConvo()}
                disabled={creating || !newTitle.trim()}
                className="w-full rounded-lg bg-btn-fill py-1 text-[12px] text-btn-ink disabled:opacity-40"
              >
                {creating ? '建着…' : '建会话'}
              </button>
            </div>
          )}
        </div>

        <div
          data-testid="nb-convo-list"
          className="min-h-0 flex-1 overflow-y-auto px-2 pb-2"
        >
          {err && (
            <div className="m-1 rounded-lg bg-btn-danger-hover p-2 text-xs text-[#c33b38]">
              <div className="font-semibold">读取会话失败</div>
              <div className="mt-1 break-all">{err.slice(0, 160)}</div>
              <button
                type="button"
                className="mt-2 rounded-full border border-[#c33b38]/30 px-2 py-0.5 text-[11px]"
                onClick={reload}
              >
                重试
              </button>
            </div>
          )}
          {!err && convos?.length === 0 && (
            <p className="p-3 text-center text-xs text-muted">
              还没有会话。
              <br />
              点右上角 + 建一个。
            </p>
          )}
          {!err && (convos?.length ?? 0) > 0 && shown.length === 0 && (
            <p className="p-3 text-center text-xs text-muted">没有匹配「{q}」的会话。</p>
          )}
          {shown.map(c => {
            // ⛔ 选中态用 `bg-nav-active` 而不是 `bg-panel-hover` ——
            //    两者同色时，指针划过和选中**看起来一模一样**，等于没有选中态。
            const active = sel === c.id
            return (
              <button
                key={c.id}
                type="button"
                onClick={() => setSel(c.id)}
                aria-current={active ? 'true' : undefined}
                className={`mb-0.5 block w-full rounded-lg px-2 py-1.5 text-left hover:bg-btn-hover ${
                  active ? 'bg-btn-active' : ''
                }`}
              >
                <div className="flex items-center gap-1">
                  <span className={`truncate text-[13px] ${active ? 'font-semibold text-ink' : 'text-ink'}`}>
                    {c.title || c.id}
                  </span>
                  {c.muted && (
                    <span className="shrink-0 text-[10px] text-muted" title="已静音">
                      🔇
                    </span>
                  )}
                  {c.unread > 0 && (
                    <span className="ml-auto shrink-0 rounded-full bg-info px-1.5 text-[10px] text-btn-ink">
                      {c.unread}
                    </span>
                  )}
                </div>
                <div className="truncate text-[11px] text-muted">
                  {c.kind === 'group' ? `${c.members.length} 人群聊` : '私聊'} · {relTime(c.last_active)}
                </div>
              </button>
            )
          })}
        </div>

        {/* 记忆：只放自己写下的事实（库侧拒密钥行）；默认收起。
            ⛔ 这块在滚动列表**外面** —— 放里面就会被长列表一起滚走。 */}
        <div className="shrink-0 border-t border-line p-2 pt-1.5">
          <button
            type="button"
            onClick={() => setMemOpen(o => !o)}
            aria-expanded={memOpen}
            className="flex w-full items-center gap-1 rounded-lg px-2 py-1.5 text-left text-[11px] text-muted hover:bg-panel-hover"
          >
            <span>{memOpen ? '▾' : '▸'}</span>
            <span>记忆{mem && mem.lines.length > 0 ? ` ${mem.lines.length}` : ''}</span>
            {mem && mem.bytes > 0 && (
              <span className="ml-auto tabular-nums">
                {mem.bytes}/{mem.cap}
              </span>
            )}
          </button>
          {memOpen && (
            <div className="px-2 pb-2">
              {memErr && <div className="mb-1 break-all text-[11px] text-[#c33b38]">{memErr}</div>}
              {mem && mem.lines.length === 0 && !memErr && (
                <p className="mb-1 text-[11px] text-muted">
                  还没有记忆。记下的事实会逐轮注入对话（只存你写的，不自动抓取）。
                </p>
              )}
              {mem && mem.lines.length > 0 && (
                <ul className="mb-1 max-h-32 space-y-0.5 overflow-y-auto">
                  {mem.lines.map((l, i) => (
                    <li key={i} className="break-words text-[11px] leading-snug text-ink">
                      {l}
                    </li>
                  ))}
                </ul>
              )}
              <div className="flex gap-1">
                <input
                  value={memDraft}
                  onChange={e => setMemDraft(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') {
                      e.preventDefault()
                      void addMemory()
                    }
                  }}
                  placeholder="记一条…"
                  aria-label="新增记忆"
                  className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1 text-[11px] text-ink outline-none focus:border-[#2468f2]"
                />
                <button
                  type="button"
                  onClick={() => void addMemory()}
                  disabled={memBusy || !memDraft.trim()}
                  className="shrink-0 rounded-lg bg-btn-fill px-2 py-1 text-[11px] text-btn-ink disabled:opacity-40"
                >
                  记下
                </button>
              </div>
              <button
                type="button"
                onClick={() => void undoMemory()}
                disabled={memBusy || !mem || mem.revisions === 0}
                title="回到上一版记忆（可再撤）"
                className="mt-1 w-full rounded-lg border border-line px-2 py-1 text-[11px] text-muted hover:bg-panel-hover disabled:opacity-40"
              >
                撤一版（{mem?.revisions ?? 0}）
              </button>
            </div>
          )}
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 shrink-0 items-center border-b border-line px-4">
          <span className="text-sm font-semibold text-ink">
            {current?.title ?? 'NeoBot'}
          </span>
          {current && (
            <span className="ml-2 text-xs text-muted">
              {current.task_count} 个任务
            </span>
          )}
          {caps && (
            <span className="ml-2 truncate text-xs text-muted" title={caps.model_source}>
              {caps.model} · {caps.tool_count} 工具
            </span>
          )}
          {usageToday !== null && usageToday > 0 && (
            <span className="ml-2 truncate text-xs text-muted" title="今日 tokens（输入+输出）">
              今日 {usageToday.toLocaleString()} tokens
            </span>
          )}
        </header>

        <div ref={msgsRef} className="relative min-h-0 flex-1 overflow-y-auto px-4 py-3">
          {msgs.length === 0 ? (
            <p className="pt-16 text-center text-xs text-muted">
              {histLoading ? '正在读历史…' : '发一句话开始。Enter 发送，Shift+Enter 换行。'}
            </p>
          ) : (
            <div className="mx-auto max-w-[760px] space-y-2">
              {msgs.map((m, i) => (
                <div key={i} className={m.who === 'me' ? 'flex justify-end' : 'flex'}>
                  <div className="flex max-w-[76%] flex-col">
                    <div
                      className={`rounded-2xl px-3 py-2 text-[13px] leading-relaxed ${
                        m.who === 'me'
                          ? 'rounded-br-md bg-[#d6e4fb] text-[#12325c]'
                          : 'rounded-bl-md bg-[#ededef] text-[#18181b]'
                      }`}
                    >
                      {m.who === 'me' || renderBot(m.text) === null ? (
                        <span className="whitespace-pre-wrap break-words">{m.text}</span>
                      ) : (
                        <div
                          className="nb-md"
                          dangerouslySetInnerHTML={{ __html: renderBot(m.text) as string }}
                        />
                      )}
                    </div>
                    <div
                      className={`mt-0.5 flex items-center gap-2 px-1 text-[10px] text-muted ${
                        m.who === 'me' ? 'justify-end' : ''
                      }`}
                    >
                      <span>{clockOf(m.ts)}</span>
                      {m.failed && (
                        <button
                          type="button"
                          onClick={() => void retry(i)}
                          disabled={busy}
                          className="rounded border border-line px-1 hover:bg-panel-hover disabled:opacity-40"
                        >
                          重发
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              ))}
              {busy && (
                <div className="flex">
                  <div className="rounded-2xl rounded-bl-md bg-[#ededef] px-3 py-2 text-[13px] text-[#61666b]">
                    <span className="inline-flex items-center gap-1">
                      <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-[#8a8f98]" />
                      正在想…
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}
          {/* 回到底部：只在用户已经滚上去时才出现（贴底时它是噪声）。 */}
          {!pinned && msgs.length > 0 && (
            <button
              type="button"
              onClick={scrollToEnd}
              className="sticky bottom-2 mx-auto block rounded-full border border-line bg-panel px-3 py-1 text-[11px] text-muted shadow-sm hover:bg-panel-hover"
            >
              回到底部 ↓
            </button>
          )}
        </div>

        <div className="shrink-0 border-t border-line p-3">
          <div className="mx-auto flex max-w-[760px] items-end gap-2">
            <textarea
              ref={taRef}
              value={draft}
              onChange={e => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  void send()
                }
              }}
              rows={1}
              placeholder="消息…（Enter 发送，Shift+Enter 换行）"
              aria-label="消息输入框"
              className="max-h-40 min-h-[36px] flex-1 resize-none overflow-y-auto rounded-2xl border border-line bg-panel px-3 py-2 text-[13px] text-ink outline-none focus:border-info-hover"
            />
            <button
              type="button"
              onClick={() => void send()}
              disabled={busy || histLoading || !draft.trim()}
              className="h-9 shrink-0 rounded-full bg-btn-fill px-4 text-[13px] text-btn-ink transition-colors hover:bg-btn-fill-hover disabled:opacity-40"
            >
              {busy ? '…' : '发送'}
            </button>
          </div>
          {histLoading && (
            <p className="mx-auto mt-1 max-w-[760px] text-[11px] text-muted">历史加载中，先不让你发（免得与迟到消息重排）。</p>
          )}
        </div>
      </section>
    </div>
  )
}
