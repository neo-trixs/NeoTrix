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

import { invokeCmd as invoke } from './ipc'

import { useVirtualizer } from '@tanstack/react-virtual'

import { t, useT } from './i18n'
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

/**
 * 失败消息的**本地化前缀**正则。`重发` 要剥掉自己加的前缀，
 * 而前缀随语言变 ⇒ 不能写死 `/^发送失败：/`。
 */
function sendFailedPrefixRe(): RegExp {
  return new RegExp('^' + t('chat.sendFailedPrefix').replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
}

/**
 * 复制文本到剪贴板，**带降级链**。
 *
 * 优先浏览器 Clipboard API；被拒时**回落**到 Tauri 命令
 * `write_clipboard_text`（`Status::Implemented` 且已注册）。
 * ⛔ 只用 `navigator.clipboard` 会在 Tauri 自定义协议（非安全上下文）下
 * **无声失效** —— 用户只看到「按了没反应」。
 *
 * @returns 是否复制成功（调用方据此给反馈，⛔ 不吞失败）
 */
async function copyText(text: string): Promise<boolean> {
  if (!text) return false
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    try {
      await invoke('write_clipboard_text', { text })
      return true
    } catch {
      return false // 两条路都被拒 ⇒ 如实返回 false，由调用方提示
    }
  }
}

/** 给按钮一个 1.2s 的「已复制」视觉反馈。 */
function flashCopied(button: HTMLElement): void {
  button.classList.add('is-copied')
  window.setTimeout(() => button.classList.remove('is-copied'), 1200)
}

async function copyCode(button: HTMLButtonElement): Promise<void> {
  const pre = button.closest('.md-code')?.querySelector('pre code')
  const text = pre?.textContent ?? ''
  // ⛔ 复制失败**不静默**：按钮不给反馈会让用户以为是自己按错了
  if (await copyText(text)) flashCopied(button)
}

function relTime(iso: string): string {
  // ⚠️ 局部变量**不可**再叫 `t`：本文件已 `import { t } from './i18n'`，
  //    同名局部会把它遮住 ⇒ t('x') 变成「对数字调用」，
  //    编译期报 `Type 'Number' has no call signatures`（tsc 抓到的，不是我看出来的）。
  const parsed = Date.parse(iso)
  if (Number.isNaN(parsed)) return ''
  const mins = Math.floor((Date.now() - parsed) / 60000)
  if (mins < 1) return t('time.justNow')
  if (mins < 60) return t('time.minutesAgo', { n: mins })
  if (mins < 1440) return t('time.hoursAgo', { n: Math.floor(mins / 60) })
  if (mins < 2880) return t('time.yesterday')
  return t('time.daysAgo', { n: Math.floor(mins / 1440) })
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
  // ⛔ 不可只用模块级 `t`：那样本组件**不订阅**语言变化 ⇒ 切换时
  //    外壳变了、聊天区不变（半成品切换器）。`useT()` 负责订阅并触发重渲染。
  const t = useT()

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
  // ⛔ 「读失败」与「本来就没有」是**两种处境**，混起来会让人白折腾 ——
  //    这正是本文件对会话列表已有的原则，此处曾**违反**它：
  //    `neobot_core_capabilities` 失败时 `setCaps(null)`，于是模型/工具数
  //    凭空消失、无任何解释，用户以为「就是没有」。失败路径实测抓到。
  const [capsErr, setCapsErr] = useState(false)
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

  // ── 侧栏：按最近活跃分组 + 可折叠（统一队列 #2）─────────────────────
  // 为什么分组：会话多起来后，「找不到上周那个对话」是最痛的点，而
  // 一列平铺的时间戳只有秒级信息量。
  // 为什么**可折叠**：分组解决了「找得到」，但**分组本身**会让长列表更长，
  // 所以必须能收起 —— 否则只是把问题挪了位置。
  //
  // ⚠️ 与虚拟化共存的关键：把「组头 + 组内条目」**拍平成一个数组**再交给
  //    virtualizer。若直接虚拟化 convos 再插组头，index→item 的映射就错位。
  type ConvoRow =
    | { kind: 'header'; key: string; label: string; count: number; collapsed: boolean }
    | { kind: 'item'; key: string; c: ConvoView }

  const [collapsed, setCollapsed] = useState<Record<string, boolean>>({})
  const GROUP_LABEL: Record<'today' | 'week' | 'older', () => string> = {
    today: () => t('convo.group.today'),
    week: () => t('convo.group.week'),
    older: () => t('convo.group.older'),
  }
  const convoRows = useMemo<ConvoRow[]>(() => {
    const now = Date.now()
    const bucketOf = (iso: string): string => {
      const t = Date.parse(iso)
      if (Number.isNaN(t)) return 'older'
      const age = now - t
      if (age < 864e5) return 'today'                 // 24h 内
      if (age < 7 * 864e5) return 'week'              // 一周内
      return 'older'
    }
    const order = ['today', 'week', 'older'] as const
    const groups = new Map<string, ConvoView[]>()
    for (const c of shown) {
      const k = bucketOf(c.last_active)
      const arr = groups.get(k)
      if (arr) arr.push(c)
      else groups.set(k, [c])
    }
    const out: ConvoRow[] = []
    for (const k of order) {
      const items = groups.get(k)
      if (!items || items.length === 0) continue
      // 组内按最近活跃在前 —— 分组后组内顺序才是主要的导航线索
      items.sort((a, b) => Date.parse(b.last_active) - Date.parse(a.last_active))
      const isCollapsed = collapsed[k] ?? false
      out.push({
        kind: 'header', key: k, count: items.length,
        // ⛔ 刻意用**显式映射**而非模板字面量 `t(\`convo.group.${k}\`)`：
        //    模板字面量的键**不在** 4d 门的正则视野内（它只认 'x' 与 "x"）
        //    ⇒ 键拼错不会被门发现，且无法 grep。
        //    显式映射可被门覆盖、可被搜索、拼错即编译期可见。
        label: GROUP_LABEL[k](), collapsed: isCollapsed,
      })
      if (!isCollapsed) {
        for (const c of items) out.push({ kind: 'item', key: c.id, c })
      }
    }
    return out
  }, [shown, collapsed, msgs.length, t])

  const toggleGroup = useCallback((k: string) => {
    setCollapsed((prev) => ({ ...prev, [k]: !(prev[k] ?? false) }))
  }, [])

  // ── 会话列表虚拟化 ──────────────────────────────────────────────────
  // 设计吸收自 @tanstack/react-virtual（MIT, 3.14.13）：**先读其 .d.ts 确认
  // API**（count / getScrollElement / estimateSize 必需，overscan 可选；
  // 实例提供 getVirtualItems / getTotalSize / measureElement）再动手。
  //
  // ⛔ 为什么现在做：此前 `shown.map(...)` **全量渲染** —— 会话上千则 DOM
  //    节点上千，打开侧栏就要建上千个按钮。
  //    而这一层**已**因 `display:flex` 修好高度约束、**真的在滚**
  //    （实测 clientH=578），所以窗口化才有可滚容器可用。
  // ⚠️ estimateSize 是**初估**：会话项两行、标题会换行 ⇒ 必须挂
  //    `measureElement` 实测校正，否则滚动条长度会跳。
  const listRef = useRef<HTMLDivElement | null>(null)
  const virtualizer = useVirtualizer({
    // ⛔ 吃**拍平后的 rows**（含组头），不是 convos —— 否则组头与条目
    //    在 index 空间里对不上，virtualizer 会把组头当条目渲染。
    count: convoRows.length,
    getScrollElement: () => listRef.current,
    estimateSize: () => 58,
    overscan: 8,
  })

  // 新建对话：⛔ 成员必须已登记（库侧 `create_conversation` 会校验），
  // 而建会话前得先有成员 —— 所以表单里能就地登记，不必去设置面板绕一圈。
  // ── 键盘快捷键：Cmd/Ctrl+K 聚焦搜索 · Esc 清空搜索 ──────────────────
  //
  // ⛔ **Esc 必须让路给对话框**：日志/记忆弹窗自己处理 Esc（`shell.tsx`
  //   用 capture + `stopPropagation`）。若这里无条件清空搜索，
  //   用户关弹窗的同一个 Esc 会**顺手把搜索也清掉** —— 那是两个无关状态
  //   被一次按键改掉，用户不会预期。
  //   ⛔ 判据用 `stopPropagation` 的**顺序**是脆的（依赖监听器注册次序），
  //   所以显式检查 `[role=dialog]` 是否存在，让路条件写进代码而不是靠时序。
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const dialogOpen = document.querySelector('[role="dialog"]') !== null
      // Cmd/Ctrl+K：聚焦搜索。⛔ 弹窗打开时**不抢焦点** ——
      //   把焦点从模态里抢走会让读屏用户丢失当前位置。
      if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K')) {
        if (dialogOpen) return
        e.preventDefault()
        const el = document.querySelector<HTMLInputElement>('[data-testid="nb-search"]')
        if (!el) return
        el.focus()
        el.select() // ⭐ 选中全部：直接打字即覆盖，符合「搜索框」惯例
        return
      }
      if (e.key !== 'Escape') return
      if (dialogOpen) return // ⛔ 弹窗的 Esc 归弹窗
      // Esc：只在**有内容**时清空，且保持焦点（用户还要继续输入）
      setQ((prev) => (prev ? '' : prev))
      const el = document.querySelector<HTMLInputElement>('[data-testid="nb-search"]')
      if (el && el.value) el.focus()
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [])

  // ── 会话列表方向键导航 ────────────────────────────────────────────
  //
  // ⛔⛔ **虚拟化是这个功能的全部难点**：DOM 里**永远只有约 20 个**
  // `nb-convo-item`（1000 个会话时实测 20），其余 980 个**不存在于 DOM**。
  // ⇒ 天真的「找下一个兄弟节点并 focus」在第 21 个就**走到尽头**：
  //   用户按 21 次 ↓ 之后什么都不会发生，而界面看起来完全正常。
  // ⇒ 到达**渲染窗口边缘**时必须**滚动列表**让虚拟化器渲染下一批，
  //   然后在下一帧再对焦。⛔ 滚动后 React 还没重渲染 ⇒ 不能同步 focus，
  //   必须 `requestAnimationFrame` 等一帧。
  //
  // ⛔ **不劫持 Tab**：Tab 仍是「离开这个列表」的标准方式。
  //   只处理 ↑/↓/Home/End —— 这是 listbox 的既有约定，不发明新键。
  const onConvoListKeyDown = useCallback((e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp' && e.key !== 'Home' && e.key !== 'End') return
    const list = e.currentTarget
    const items = () => [...list.querySelectorAll<HTMLElement>('[data-testid="nb-convo-item"]')]
    const rendered = items()
    if (rendered.length === 0) return
    e.preventDefault()

    const cur = rendered.indexOf(document.activeElement as HTMLElement)
    const atEnd = cur === rendered.length - 1
    const atStart = cur <= 0

    // Home / End：跳到**当前渲染窗口**的首/尾（ⓘ 不是整个列表 ——
    //   虚拟化下「整个列表的末尾」得先滚过去，Home/End 只处理可见区间，
    //   否则一次 End 要等 1000 项全部渲染，延迟不可接受）。
    // 同步 roving：方向键移到哪一项，Tab 的停靠点就跟到哪一项
    // （否则方向键移过、随后按 Tab 会跳回旧选中项，行为自相矛盾）。
    const focusAt = (el: HTMLElement | undefined) => {
      if (!el) return
      const id = el.dataset.convoId
      if (id) setRoving(id)
      el.focus()
    }
    if (e.key === 'Home') { focusAt(rendered[0]); return }
    if (e.key === 'End') { focusAt(rendered[rendered.length - 1]); return }

    // 越过边缘 ⇒ 滚动加载下一批，下一帧再对焦
    if ((e.key === 'ArrowDown' && atEnd) || (e.key === 'ArrowUp' && atStart)) {
      const before = list.scrollTop
      list.scrollTop += e.key === 'ArrowDown' ? list.clientHeight : -list.clientHeight
      // 已在顶部/底部且滚不动 ⇒ 没有下一批，如实什么都不做
      if (list.scrollTop === before) return
      requestAnimationFrame(() => {
        const now = items()
        if (now.length === 0) return
        const t = e.key === 'ArrowDown' ? now[0] : now[now.length - 1]
        if (t?.dataset.convoId) setRoving(t.dataset.convoId)
        t?.focus()
      })
      return
    }
    const next = e.key === 'ArrowDown' ? cur + 1 : cur - 1
    focusAt(rendered[next])
  }, [])

  // ── roving tabindex：解决虚拟化列表的**焦点陷阱** ────────────────
  //
  // ⛔⛔ 这是我在写方向键时**实测发现的既存缺陷**（与方向键无关）：
  //   会话项全是 `<button>`，而列表是**虚拟化**的（1000 项只渲染 ~20）。
  //   Tab 聚焦某项会把它**滚进视口** ⇒ 虚拟化器渲染**下一批** ⇒
  //   于是「Tab 的下一个可聚焦元素」**永远存在** ⇒
  //   连按 61 次 Tab 都**出不去列表**（门实测）⇒ **焦点陷阱**。
  //   ⇒ 标准解法 **roving tabindex**：整张列表**只允许一个** `tabIndex=0`，
  //   其余全 `-1`。Tab 于是「进入列表（落在当前项）→ 再按即离开」，
  //   而不是逐项爬 1000 次。方向键在列表**内部**移动焦点并更新 roving 项。
  const [roving, setRoving] = useState<string | null>(null)
  /**
   * 整张列表的**唯一** Tab 停靠点：roving 项 → 当前选中项 → 首个会话。
   * ⛔ 绝不能「每项都 0」：虚拟化会让 Tab 变成出不去的焦点陷阱（见上）。
   * ⛔ 也绝不能「全 -1」：那样列表**根本进不去**（键盘用户无法触达会话）。
   *    ⇒ 必须**恰好一个** 0。
   */
  const tabStopId = roving ?? sel ?? shown[0]?.id ?? null
  const convoTabIndex = useCallback(
    (id: string) => (tabStopId ? (id === tabStopId ? 0 : -1) : -1), [tabStopId])

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
      .then((v) => {
        setCaps(v)
        setCapsErr(false)
      })
      .catch(() => {
        setCaps(null)
        setCapsErr(true) // 说清「读失败」，而不是假装「没有」
      })
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
      setMsgs(m => [...m, { who: 'bot', text: r?.output ?? r?.text ?? t('chat.noOutput'), ts: nowIso() }])
    } catch (e) {
      // ⛔ 失败气泡标成 failed：那样才能给「重发」，也不至于和正常回复混淆。
      setMsgs(m => [...m, { who: 'bot', text: `${t('chat.sendFailedPrefix')}${String(e).slice(0, 200)}`, ts: nowIso(), failed: true }])
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
          text: bad.text.replace(sendFailedPrefixRe(), ''),
        })
        setMsgs(m => [...m, { who: 'bot', text: r?.output ?? r?.text ?? t('chat.noOutput'), ts: nowIso() }])
      } catch (e) {
        setMsgs(m => [...m, { who: 'bot', text: `${t('chat.sendFailedPrefix')}${String(e).slice(0, 200)}`, ts: nowIso(), failed: true }])
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

  // 消息列表窗口化（与会话列表同一模式）。
  // estimateSize 是**初估**：气泡高度随内容换行而变 ⇒ 必须 measureElement 校正。
  const msgVirtualizer = useVirtualizer({
    count: msgs.length,
    getScrollElement: () => msgsRef.current,
    estimateSize: () => 64,
    overscan: 6,
  })

  // 自动滚到底：**只在用户本来就在底部时**才滚。
  // ⛔ 无条件 scrollTop = 高度会把人从正在读的历史里硬拽走 —— 那是「自作聪明」。
  const [pinned, setPinned] = useState(true)
  const scrollToEnd = useCallback(() => {
    // 用 scrollToIndex 而非 scrollTop = scrollHeight。
    // ⛔ **但我先前把它说成「后者会跳到错误位置」是夸大的** —— 浏览器会把
    //    scrollTop 钳到 scrollHeight - clientHeight（即底部），所以它**也能到底**。
    //    负向测试：注入 scrollTop 做法后，本门判据 C **仍通过** ⇒ C 区分不出两者。
    //    保留 scrollToIndex 的理由是**语义正确**（由 virtualizer 定位到最后一项，
    //    不依赖「钳到边界」这个副作用），**不是**「否则会坏」。
    setPinned(true)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [msgs.length])
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
    // 同上：语义上应由 virtualizer 定位，而非依赖 scrollTop 钳到边界
    msgVirtualizer.scrollToIndex(msgs.length - 1, { align: 'end' })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [msgs.length, pinned])

  // 输入框随内容增高（上限 5 行后转为滚动）。
  // ⛔ 占位符承诺了「Shift+Enter 换行」，框却永远一行 ⇒ 用户看不见自己打的第二行。
  const taRef = useRef<HTMLTextAreaElement | null>(null)
  useEffect(() => {
    const ta = taRef.current
    if (!ta) return
    ta.style.height = 'auto'
    ta.style.height = `${Math.min(ta.scrollHeight, 160)}px`
  }, [draft])

  /** 会话行：抽成独立渲染函数，供 virtualizer 逐项调用。 */
  const renderConvoRow = (c: ConvoView) => {
    // ⛔ 选中态用 `bg-nav-active` 而不是 `bg-panel-hover` ——
    //    两者同色时，指针划过和选中**看起来一模一样**，等于没有选中态。
    const active = sel === c.id
    return (
      <button
              key={c.id}
              // ⛔ 稳定钩子：视觉门原先量 `[data-testid="nb-convo-list"] button` 的
              //    **全部** button ⇒ 侧栏加「按最近活跃分组」的组头（~25px）后，
              //    组头被当成「会话项」⇒ 门报「行高 25px < 40px」。
              //    门没错在阈值，是**量错了对象**（同 f38acb0f §7.2 同一纪律，
              //    只不过这次是**别人加功能**暴露了选择器的位置假设）。
              data-testid="nb-convo-item"
              type="button"
              // ⛔ roving tabindex：见上方注释。整张列表只有一个 0，
              //   否则虚拟化会让 Tab 变成出不去的焦点陷阱。
              tabIndex={convoTabIndex(c.id)}
              data-convo-id={c.id}
              onClick={() => { setRoving(c.id); setSel(c.id) }}
              aria-current={active ? 'true' : undefined}
              // 选中态：底色 + 左侧强调条。⛔ 只给 font-semibold 不够 ——
              // 字号权重的差别在 13px 下几乎看不出来，用户会以为没选中。
              style={active ? { boxShadow: 'inset 2px 0 0 var(--color-info)' } : undefined}
              className={`mb-1 block w-full rounded-lg px-2.5 py-2 text-left transition-colors hover:bg-btn-hover ${
                active ? 'bg-btn-active' : ''
              }`}
            >
              <div className="flex items-center gap-1">
                <span className={`truncate text-[13px] ${active ? 'font-semibold text-ink' : 'text-ink'}`}>
                  {c.title || c.id}
                </span>
                {c.muted && (
                  <span
                    className="shrink-0 text-[10px] leading-none text-muted"
                    title={t('chat.muted')}
                    aria-label={t('chat.muted')}
                  >
                    {/* ⛔ 不用 emoji：彩色字形在深色侧栏里是唯一的彩色噪点，
                        且各平台字形不一致。短横杠即「静音条」，与文字同色。 */}
                    ▬
                  </span>
                )}
                {c.unread > 0 && (
                  <span className="ml-auto shrink-0 rounded-full bg-info px-1.5 text-[10px] tabular-nums text-btn-ink">
                    {c.unread}
                  </span>
                )}
              </div>
              <div className="truncate text-[11px] text-muted">
                {c.kind === 'group' ? t('chat.kindGroup', { n: c.members.length }) : t('chat.kindPrivate')} · {relTime(c.last_active)}
              </div>
            </button>
    )
  }

  return (
    <div className="flex min-h-0 flex-1" data-testid="neobot-root">
      {/* 侧栏：筛选 + 新建在上，列表独立滚动，记忆面板钉在底部。
          ⛔ 三段必须拆开：以前整栏一个 overflow-y-auto + `mt-auto`，
          列表一长记忆面板就被一起滚走，`mt-auto` 在滚动容器里本就无效。 */}
      <aside className="flex min-h-0 w-64 shrink-0 flex-col border-r border-line">
        <div className="shrink-0 space-y-1.5 p-2">
          <div className="flex gap-1.5">
            <input
              // ⛔ 门需要稳定选择器：`aria-label` 是**随语言变**的
              //   （zh「搜索会话」/ en「Search conversations」），
              //   拿它当选择器会让门在切语言后失效。
              data-testid="nb-search"
              value={q}
              onChange={e => setQ(e.target.value)}
              placeholder={t('chat.searchPlaceholder')}
              aria-label={t('chat.searchLabel')}
              title={`${t('chat.searchLabel')} · ${t('chat.searchShortcut')}`}
              className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1 text-[12px] text-ink outline-none focus:border-info-hover"
            />
            <button
              type="button"
              onClick={() => {
                setNewOpen(o => !o)
                if (!newOpen) reloadMembers()
              }}
              aria-expanded={newOpen}
              title={t('chat.newConversation')}
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
                placeholder={t('chat.titlePlaceholder')}
                aria-label={t('chat.titleLabel')}
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
                placeholder={t('chat.memberIdOptional')}
                aria-label={t('chat.memberIdLabel')}
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
                {creating ? t('chat.creating') : t('chat.create')}
              </button>
            </div>
          )}
        </div>

        {/* 分组标题：⛔ 没有它，搜索框与第一条会话之间没有层级，
            列表看起来像「搜索框下面的东西」而不是「一组会话」。 */}
        <div className="shrink-0 px-2.5 pb-1 pt-1 text-[11px] uppercase tracking-wide text-muted">
          {t('chat.section')}
        </div>

        <div
          ref={listRef}
          data-testid="nb-convo-list"
          className="nb-scroll min-h-0 flex-1 overflow-y-auto px-2 pb-2"
          onKeyDown={onConvoListKeyDown}
        >
          {err && (
            <div className="m-1 rounded-lg bg-btn-danger-hover p-2 text-xs text-[#c33b38]">
              <div className="font-semibold">{t('chat.loadFailed')}</div>
              <div className="mt-1 break-all">{err.slice(0, 160)}</div>
              <button
                type="button"
                className="mt-2 rounded-full border border-[#c33b38]/30 px-2 py-0.5 text-[11px]"
                onClick={reload}
              >
                {t('chat.retry')}
              </button>
            </div>
          )}
          {!err && convos?.length === 0 && (
            <p className="p-3 text-center text-xs text-muted">
              {t('chat.noConvos')}
              <br />
              {t('chat.createHint')}
            </p>
          )}
          {!err && (convos?.length ?? 0) > 0 && shown.length === 0 && (
            <p className="p-3 text-center text-xs text-muted">{t('chat.noMatch', { q })}</p>
          )}
          {/* 窗口化渲染。
              ⛔ 外层**必须**撑出 `getTotalSize()`：省了它，滚动条会按
                 「可见项数」算高度 ⇒ 根本滚不动（这是窗口化最常见的错）。
              ⛔ 每项**必须** `ref={virtualizer.measureElement}`：会话项两行、
                 标题会换行，高度不等；只靠 estimateSize 滚动条会跳。 */}
          {shown.length > 0 && (
            <div
              data-testid="nb-convo-sizer"
              style={{ height: `${virtualizer.getTotalSize()}px`, position: 'relative' }}
            >
              {virtualizer.getVirtualItems().map((vi) => {
                const row = convoRows[vi.index]
                if (!row) return null
                if (row.kind === 'header') {
                  return (
                    <div
                      key={row.key}
                      data-index={vi.index}
                      ref={virtualizer.measureElement}
                      style={{
                        position: 'absolute', top: 0, left: 0, width: '100%',
                        transform: `translateY(${vi.start}px)`,
                      }}
                    >
                      <button
                        type="button"
                        data-testid={`nb-group-${row.key}`}
                        aria-expanded={!row.collapsed}
                        onClick={() => toggleGroup(row.key)}
                        className="w-full px-2 py-1 text-left text-[11px] font-medium text-muted"
                      >
                        <span aria-hidden="true">{row.collapsed ? '▸' : '▾'}</span>{' '}
                        {row.label}
                        <span className="ml-1 opacity-70">({row.count})</span>
                      </button>
                    </div>
                  )
                }
                const c = row.c
                return (
                  <div
                    key={c.id}
                    data-index={vi.index}
                    ref={virtualizer.measureElement}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      transform: `translateY(${vi.start}px)`,
                    }}
                  >
                    {renderConvoRow(c)}
                  </div>
                )
              })}
            </div>
          )}

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
            <span>{t('chat.memory')}{mem && mem.lines.length > 0 ? ` ${mem.lines.length}` : ''}</span>
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
                  {t('chat.noMemory')}
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
                  placeholder={t('chat.memoryPlaceholder')}
                  aria-label={t('chat.addMemory')}
                  className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1 text-[11px] text-ink outline-none focus:border-[#2468f2]"
                />
                <button
                  type="button"
                  onClick={() => void addMemory()}
                  disabled={memBusy || !memDraft.trim()}
                  className="shrink-0 rounded-lg bg-btn-fill px-2 py-1 text-[11px] text-btn-ink disabled:opacity-40"
                >
                  {t('chat.memorySave')}
                </button>
              </div>
              <button
                type="button"
                onClick={() => void undoMemory()}
                disabled={memBusy || !mem || mem.revisions === 0}
                title={t('chat.memoryUndoTitle')}
                className="mt-1 w-full rounded-lg border border-line px-2 py-1 text-[11px] text-muted hover:bg-panel-hover disabled:opacity-40"
              >
                {t('chat.memoryUndo', { n: mem?.revisions ?? 0 })}
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
              {t('chat.taskCount', { n: current.task_count })}
            </span>
          )}
          {caps && (
            <span className="ml-2 truncate text-xs text-muted" title={caps.model_source}>
              {caps.model} · {t('chat.toolCount', { n: caps.tool_count })}
            </span>
          )}
          {capsErr && (
            <span
              className="ml-2 truncate text-xs text-muted"
              style={{ color: '#a3272b' }}
              title={t('chat.capsFailedHint')}
            >
              {t('chat.capsFailed')}
            </span>
          )}
          {usageToday !== null && usageToday > 0 && (
            <span className="ml-2 truncate text-xs text-muted" title={t('chat.tokensTitle')}>
              {t('chat.tokensToday', { n: usageToday.toLocaleString() })}
            </span>
          )}
        </header>

        <div ref={msgsRef} className="relative min-h-0 flex-1 overflow-y-auto px-4 py-3">
          {/* ⛔ 空状态之前只有一行 12px 小字浮在上方：既没有视觉重心，
              也读不出「我现在能做什么」。空状态是唯一一次能告诉用户下一步
              的机会，占位不足等于浪费。
              ⛔ 本注释必须放在三元表达式**外面**：写在 `? (` 与 `)` 之间的
              裸注释不是合法 JSX。踩过一次。 */}
          {msgs.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center gap-2 px-6 text-center">
              <div
                className="mb-1 flex h-11 w-11 items-center justify-center rounded-full border border-line text-[18px] text-muted"
                aria-hidden="true"
              >
                💬
              </div>
              <div className="text-[13px] font-medium text-ink">
                {histLoading ? t('chat.historyLoading') : t('chat.emptyTitle')}
              </div>
              {!histLoading && (
                <div className="max-w-[320px] text-[12px] leading-relaxed text-muted">
                  {t('chat.emptyHint')}
                </div>
              )}
            </div>
          ) : (
            <div className="mx-auto max-w-[760px] space-y-2">
              {/* 消息窗口化。⛔ 外层**必须**撑出 getTotalSize()：省了它，
                  滚动条按「可见项数」算高度 ⇒ 根本滚不动。 */}
              {msgs.length > 0 && (
              <div
                data-testid="nb-msg-sizer"
                style={{ height: `${msgVirtualizer.getTotalSize()}px`, position: 'relative' }}
              >
              {msgVirtualizer.getVirtualItems().map((vi) => {
                const m = msgs[vi.index]
                if (!m) return null
                // ⛔ 连续同作者的消息**聚成一组**：组内只留 2px、组间留 10px。
                //    此前每条都等距（外层 space-y-2，且虚拟化后外层间距根本不生效），
                //    于是 6 条消息读起来像 6 个独立事件，而不是「一问一答」两段。
                //    间距必须写在**被测量的元素内**（paddingTop）——
                //    绝对定位 + measureElement 的布局下，外部 margin 不会被计入高度。
                const prev = msgs[vi.index - 1]
                const next = msgs[vi.index + 1]
                const groupStart = !prev || prev.who !== m.who
                const groupEnd = !next || next.who !== m.who
                return (
                <div
                  key={vi.index}
                  data-index={vi.index}
                  ref={msgVirtualizer.measureElement}
                  className={m.who === 'me' ? 'flex justify-end' : 'flex'}
                  style={{
                    position: 'absolute', top: 0, left: 0, width: '100%',
                    transform: `translateY(${vi.start}px)`,
                    paddingTop: `${groupStart ? 10 : 2}px`,
                  }}
                >
                  {/* ⛔ `group` 是悬停钩子：复制按钮默认**不可见**（不占视觉），
                      悬停/聚焦才出现。⛔ 刻意**不用 React state** 控制显隐 ——
                      纯视觉的东西交给 CSS，否则每次悬停都触发一次重渲染，
                      而消息列表是**虚拟化**的（重渲染代价随可见条数放大）。
                      键盘可达性用 `focus-within` 一并覆盖，不只 `hover`。 */}
                  <div className="group relative flex max-w-[76%] flex-col">
                    <div
                      className={`rounded-2xl px-3 py-2 text-[13px] leading-relaxed ${
                        m.who === 'me'
                          ? 'rounded-br-md bg-[#d6e4fb] text-[#12325c]'
                          : 'rounded-bl-md bg-[#ededef] text-[#18181b]'
                      }`}
                    >
                      {/* 悬停/聚焦复制：绝对定位到气泡右上角外侧，
                          ⛔ 不参与文档流 ⇒ 不改变气泡高度（measureElement
                          测到的仍是纯文本高度，虚拟化高度不受影响）。 */}
                      <button
                        type="button"
                        data-testid="nb-msg-copy"
                        aria-label={t('chat.copyMessage')}
                        title={t('chat.copyMessage')}
                        onClick={async (e) => {
                          // ⛔ 显式取事件对象：写 `event?.currentTarget` 会落到
                          //    DOM lib 的**全局 event**（遗留全局），tsc 不报错
                          //    但语义错、且未来 lib 收紧就会断。
                          const btn = e.currentTarget
                          if (await copyText(m.text)) flashCopied(btn)
                        }}
                        className="nb-msg-copy"
                      >
                        <span aria-hidden="true">⧉</span>
                      </button>
                      {m.who === 'me' || renderBot(m.text) === null ? (
                        <span className="whitespace-pre-wrap break-words">{m.text}</span>
                      ) : (
                        <div
                          className="nb-md"
                          dangerouslySetInnerHTML={{ __html: renderBot(m.text) as string }}
                        />
                      )}
                    </div>
                    {/* ⛔ 时刻**只在组末**显示：每条都挂一个 10px 时间戳时，
                        一组三条会出现三个几乎相同的时间，读起来是噪声而不是信息。
                        组末一条代表「这段话说完于何时」。失败重发按钮不受此限 ——
                        它必须跟着失败的那一条。 */}
                    <div
                      className={`flex items-center gap-2 px-1 text-[10px] text-muted ${
                        groupEnd ? 'mt-0.5' : 'mt-0'
                      } ${m.who === 'me' ? 'justify-end' : ''} ${
                        !groupEnd && !m.failed ? 'invisible' : ''
                      }`}
                    >
                      {clockOf(m.ts)}
                      {m.failed && (
                        <button
                          type="button"
                          onClick={() => void retry(vi.index)}
                          disabled={busy}
                          className="rounded border border-line px-1 hover:bg-panel-hover disabled:opacity-40"
                        >
                          {t('chat.resend')}
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              )
              })}
              </div>
              )}
              {busy && (
                <div className="flex">
                  <div className="rounded-2xl rounded-bl-md bg-[#ededef] px-3 py-2 text-[13px] text-[#61666b]">
                    <span className="inline-flex items-center gap-1">
                      <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-[#8a8f98]" />
                      {t('chat.thinking')}
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
              {t('chat.backToBottom')}
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
              placeholder={t('chat.inputPlaceholder')}
              aria-label={t('chat.inputLabel')}
              className="max-h-40 min-h-[36px] flex-1 resize-none overflow-y-auto rounded-2xl border border-line bg-panel px-3 py-2 text-[13px] text-ink outline-none focus:border-info-hover"
            />
            <button
              type="button"
              onClick={() => void send()}
              disabled={busy || histLoading || !draft.trim()}
              className="h-9 shrink-0 rounded-full bg-btn-fill px-4 text-[13px] text-btn-ink transition-colors hover:bg-btn-fill-hover disabled:opacity-40"
            >
              {busy ? t('chat.sendEllipsis') : t('chat.send')}
            </button>
          </div>
          {histLoading && (
            <p className="mx-auto mt-1 max-w-[760px] text-[11px] text-muted">{t('chat.historyPaused')}</p>
          )}
        </div>
      </section>
    </div>
  )
}
