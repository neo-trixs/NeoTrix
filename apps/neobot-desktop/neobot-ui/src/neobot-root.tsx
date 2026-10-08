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
import { listen } from '@tauri-apps/api/event'

import { t, useT } from './i18n'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import './vendor/openghost/tex.js'
import './vendor/openghost/markdown.js'
import './vendor/openghost/highlight.js'
import { installOpenghostShim } from './vendor/openghost/shim.ts'
import './ui/nb-markdown.css'
import './nb-reduced-motion.css'

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

type Msg = {
  who: 'me' | 'bot'
  text: string
  ts: string
  failed?: boolean
  /** ⭐ 稳定标识。⛔ 旧实现用**数组下标**寻址重发，而 `msgs` 是可变列表 ⇒
   *  「点击 → promise resolve」之间任何 `setMsgs` 都会让下标错位
   *  ⇒ **可能重发另一条消息的内容**。 */
  id?: string
  /** ⭐ 发送时的**原始正文**（失败时才有）。
   *  ⛔⛔ 旧实现把「本地化错误前缀 + 正文」塞进同一个 `text`，`重发` 再用
   *    **当前语言的前缀正则**剥回去 ⇒ **切换语言后重发会把前缀一起当正文发出去**；
   *    且 `String(e).slice(0, 200)` 截断会让前缀正则失配。
   *  ⇒ 根因是**一个显示字段承载了两份信息**。存一份原文即可根除。 */
  prompt?: string
}

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

/** 库侧轨迹行（`neobot_run_list` 的 `runs[]`，蛇形同名）。 */
interface RunRow {
  id: string
  title: string
  /** **原样**状态串（库侧不解析；可能是本仓还不认识的新值）。 */
  status: string
  created_at: string
  updated_at: string
  error: string | null
  steps: number
  /** 其中失败步数（≠ 失败轮数）。 */
  failed_steps: number
}

/** 库侧轨迹页的一步（`neobot_run_trace` 的 `steps[]`）。 */
interface StepView {
  id: number
  n: number
  tool: string
  ok: boolean
  output: string
  /** `null` = 该行没有配平键（**不是**空串）。 */
  tool_call_id: string | null
}

/** 库侧文件改动（`neobot_run_trace` 的 `changes[]`）。 */
interface ChangeView {
  id: string
  at: string
  path: string
  /** `read` | `write` | `edit`。 */
  kind: string
  bytes: number
  content_omitted: boolean
}

/** 库侧单轮轨迹（`neobot_run_trace` 返回值）。 */
interface RunTrace {
  run: RunRow
  steps: StepView[]
  changes: ChangeView[]
}

/**
 * 跑轮生命周期信号（Rust `RUN_EVENT = "neobot:run"` 的载荷）。
 *
 * ⛔ **刻意没有 `delta`/`token` 字段**，前端的类型里也不许加。
 *    发送路径走的是配对核心的 `POST /v1/agents/run`，那个端点是否支持
 *    SSE **本仓无法验证** ⇒ 凭空造一条 token 流就是「对不存在能力的
 *    投机实现」。这里只收**能证实**的三段信号。
 */
interface RunSignal {
  phase: 'started' | 'finished' | 'failed'
  convo_id: string | null
  elapsed_ms: number
  trace: Array<{ kind: string; detail: string }>
  error: string | null
}

/** 跑轮信号事件名（必须与 `commands.rs::RUN_EVENT` 逐字一致）。 */
const RUN_EVENT = 'neobot:run'

/**
 * 卡住看门狗的阈值（秒）。
 *
 * ⭐ 吸收 `outsourc-e/hermes-workspace`（MIT）的 `use-streaming-message.ts`：
 *   它把「服务端已受理但还没动静」与「服务端在跑但忽然安静」分成**两个**预算
 *   （120s / 300s），理由是这两种情况**需要用户做不同的事**。
 *   本仓的发送是**一次同步 HTTP**（`nt_core` 里 `post_with_auth(.., 120)`），
 *   所以只有一个阈值，且它必须**略大于**服务端的 120s ——
 *   否则看门狗会先于后端超时判死，用户看到「可能卡住了」而实际后端马上就要
 *   返回一个**说得出原因**的真错误。用「猜的超时」盖住「已知的超时」
 *   只会把一个可解释的失败换成一句不可解释的猜测。
 */
const RUN_STALL_SECS = 135

/**
 * 把一次失败分成人能据以行动的几类。
 *
 * ⭐ 吸收两处，两处都强调**不要误分类**：
 *   · `hermes-dojo`（MIT）`_classify_error_root_cause`：把 infra / auth /
 *     rate_limit 明确标成**不是技能能修的** ⇒ 不该给「重试就好」的暗示。
 *   · `hermes-workspace`（MIT）`connection-errors.ts`：泛化的 `token`
 *     字样**不得**路由到「重新登录」，因为 `"failed to fetch token from /api/x"`
 *     是网络噪声。
 * ⇒ 判据全部落在**成对出现**的标记上；单个泛化词不构成证据。
 */
type FailKind = 'unpaired' | 'auth' | 'rate' | 'network' | 'timeout' | 'empty' | 'other'

function classifyFailure(raw: string): FailKind {
  const s = raw.toLowerCase()
  // ⚠️ 每条都要**成对**证据：单看一个泛化词会把网络抖动说成要重新登录。
  if (s.includes('unpaired') || s.includes('pair first')) return 'unpaired'
  if (s.includes('rate limit') || s.includes('429') || s.includes('too many requests')) {
    return 'rate'
  }
  if (
    (s.includes('401') || s.includes('403') || s.includes('unauthorized')) &&
    (s.includes('auth') || s.includes('key') || s.includes('token') || s.includes('bearer'))
  ) {
    return 'auth'
  }
  if (s.includes('timeout') || s.includes('timed out') || s.includes('deadline')) return 'timeout'
  if (
    s.includes('connection') ||
    s.includes('connect') ||
    s.includes('network') ||
    s.includes('dns') ||
    s.includes('refused') ||
    s.includes('resolve')
  ) {
    return 'network'
  }
  return 'other'
}

/**
 * 失败类 → 一句**可据以行动**的话；`other` 如实回显原文（不猜）。
 *
 * ⛔ 用**显式映射表**而非 `t(`run.err.${kind}`)` 模板字面量：
 *    4d 门的正则只认 `'x'` 与 `"x"` ⇒ 模板字面量的键**不在它的视野内**
 *    ⇒ 键拼错不会被门发现，且无法 grep（`neobot-root.tsx` 的组标签
 *    已为同一理由走过这条路，见 `GROUP_LABEL`）。
 */
const FAIL_HINT_KEY: Record<FailKind, string> = {
  unpaired: 'run.errUnpaired',
  auth: 'run.errAuth',
  rate: 'run.errRate',
  network: 'run.errNetwork',
  timeout: 'run.errTimeout',
  empty: 'run.emptyOutput',
  other: 'run.errOther',
}

function failHint(kind: FailKind, raw: string): string {
  // `other` 没有可指的动作 ⇒ **如实回显原文**，不套一句通用废话
  // （「未知错误」比原文更没用：原文里可能有用户能据此行动的线索）。
  if (kind === 'other') return raw.slice(0, 200)
  return t(FAIL_HINT_KEY[kind])
}

/**
 * 人类可读的一步标签：只留**末段**文件名，命令截断。
 *
 * ⭐ 吸收 `hermes-workspace`（MIT）`streaming-activity-ui.ts`：它把工具名翻译成
 *   `read foo.ts`（**只取 basename**，完整路径不进标签）、
 *   `exec <cmd 截到 27 字符 + …>`。理由是转录该说**发生了什么**，
 *   不是**参数原文** —— 完整路径属于展开后的一层。
 */
function stepLabel(tool: string): string {
  if (tool === 'reply') return t('trace.stepReply')
  if (tool.startsWith('side-effect:')) return t('trace.stepSideEffect')
  if (tool === 'cancelled:tool_calls') return t('trace.stepCancelled')
  const parts = tool.split(/[\\/:]/)
  return parts[parts.length - 1] || tool
}

/** 文件改动一行的 kind → 词条键（⛔ 显式映射；理由同 `FAIL_HINT_KEY`）。 */
const CHANGE_KIND_KEY: Record<string, string> = {
  read: 'trace.kindRead',
  write: 'trace.kindWrite',
  edit: 'trace.kindEdit',
}

/** 文件改动一行的人类标签（kind + 末段路径）。 */
function changeLabel(kind: string, path: string): string {
  const parts = path.split('/')
  const tail = parts[parts.length - 1] || path
  const key = CHANGE_KIND_KEY[kind]
  // 未知 kind 不编词：直接原样显示（库侧认不出的值也可能是新 kind）。
  return key ? `${t(key)} ${tail}` : tail
}

/** 字节数的人类标签（读=读到的长度；写/改=改后长度）。 */
function bytesLabel(kind: string, bytes: number): string {
  if (kind === 'read') return t('trace.bytesRead', { n: bytes.toLocaleString() })
  return t('trace.bytesAfter', { n: bytes.toLocaleString() })
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
  // ⭐⭐ 2026-10-03：草稿**按会话分区**（此前是**单个全局字符串**）
  // ⛔ 旧形态的后果：切会话时草稿**丢失或串台**（在 A 打的字，切到 B 还在）。
  //    ⭐ 用 Record 而非数组下标：会话 id 是字符串，Record 可直接按 id 取，
  //    且 ⭐ 删除键用 `delete`（`undefined` 会留下 `'undefined'` 键的坑）。
  const [drafts, setDrafts] = useState<Record<string, string>>({})
  // ⭐ 当前会话的草稿（渲染/发送都读它，避免各处重复写 `drafts[sel ?? '']`）
  const draft = sel ? (drafts[sel] ?? '') : ''
  // ⭐⭐ 2026-10-03：busy **按会话隔离**（此前是**单个全局 bool**）
  // ⛔ 旧形态的后果：在 A 会话跑长任务时，**B 会话也发不出去**
  //    （`:581` 的 `if (!text || busy) return` 一刀切整个界面）。
  //    ⭐ 这是**串台**的一种：不是内容串，是**可用性**串。
  const [busyByConvo, setBusyByConvo] = useState<Record<string, boolean>>({})
  // computer 控制面：租约倒计时 + 最近回执（只读后端真相，失败静默不打断对话）
  const [lease, setLease] = useState<{ active: boolean; owner: string; remaining_ms: number }>({
    active: false,
    owner: '',
    remaining_ms: 0,
  })
  const [receipts, setReceipts] = useState<
    Array<{ at: string; action: string; status: string; advice: string; sent: boolean }>
  >([])
  const busy = sel ? (busyByConvo[sel] ?? false) : false
  const [histLoading, setHistLoading] = useState(false)
  // ⭐⭐ 2026-10-03：历史**分页**（接上 `neobot_convo_messages_page`）
  // ⛔ 旧形态：切会话就 `invoke<ChatMessage[]>('neobot_convo_messages')`
  //    **一次性全量拉取** ⇒ 长会话把整段历史一次塞进 DOM。
  const [hasMore, setHasMore] = useState(false)
  // ⭐ 下一页游标 = 当前**最小** seq（⛔ 不是最大，见 store 层论证）
  const [oldestSeq, setOldestSeq] = useState<number | null>(null)
  const [loadingMore, setLoadingMore] = useState(false)
  /** 首页条数：⭐ 有界（⛔ 不用「全部」）。中英各一条，不新增文案键。 */
  const PAGE = 60
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

  // ── 对话 / 轨迹 双页签 ──────────────────────────────────────────
  //
  // ⭐⭐ 吸收的形态：DSH 的会话视图是**两个页签**（对话 / 轨迹），
  //    而本仓此前只有「对话」—— **跑过什么完全看不见**。
  //    现在数据源接上（`neobot_run_list` / `neobot_run_trace`），
  //    两页共用**同一个选中会话**（不是各自选一个：那会造出「对话看 A、
  //    轨迹看 B」这种无法解释的错位）。
  const [view, setView] = useState<'chat' | 'trace'>('chat')

  // 轨迹列表：⛔ 与会话列表同纪律 —— `null` = 还没读；`[]` = **真的没有**；
  // `runsErr` 非空 = **读不到**。三者渲染成三种不同的话。
  const [runs, setRuns] = useState<RunRow[] | null>(null)
  const [runsErr, setRunsErr] = useState('')
  const [runsLoading, setRunsLoading] = useState(false)
  const reloadRuns = useCallback(() => {
    setRunsLoading(true)
    // ⛔ 缺席 convoId = 全部会话（库侧 `run_list` 允许）；给了就只看那个会话。
    //    「全量」是有界的：库侧钳在 200 条，且 `null` 是「还没读」不是「没有」。
    void invoke<{ runs: RunRow[] }>('neobot_run_list', { convoId: sel ?? undefined, limit: 100 })
      .then((v) => {
        setRuns(v.runs)
        setRunsErr('')
      })
      .catch((e) => {
        setRuns(null)
        setRunsErr(String(e).slice(0, 200))
      })
      .finally(() => setRunsLoading(false))
  }, [sel])
  // 轨迹**按需**拉：不在轨迹页时不花钱（对照：会话列表是首屏就要的）。
  useEffect(() => {
    if (view === 'trace') reloadRuns()
  }, [view, reloadRuns])
  // 跑完一轮后轨迹会变 ⇒ 重拉（`runPhase` 进 finished/failed 时触发）。
  const [runPhase, setRunPhase] = useState<'started' | 'finished' | 'failed' | null>(null)
  useEffect(() => {
    if (view === 'trace' && (runPhase === 'finished' || runPhase === 'failed')) reloadRuns()
    // ⛔ 依赖只到 `runPhase`：**不把 reloadRuns 列进去** —— 切会话也会换它，
    //    那样这条 effect 会在每次切会话时多跑一次（读一整页轨迹只为了立刻扔掉）。
  }, [runPhase, view]) // eslint-disable-line react-hooks/exhaustive-deps

  // 单轮详情：点开才拉。⛔ `detailFor` 是 id ⇒ 拉回的是哪一轮由**答**决定，
  //    不是由「谁点的」决定（后者在连点两行时会串台）。
  const [detailFor, setDetailFor] = useState<string | null>(null)
  const [detail, setDetail] = useState<RunTrace | null>(null)
  const [detailErr, setDetailErr] = useState('')
  const toggleRun = useCallback(async (id: string) => {
    if (detailFor === id) {
      setDetailFor(null) // 收起：不需要清 detail（下次展开会重拉）
      return
    }
    setDetailFor(id)
    setDetail(null)
    setDetailErr('')
    try {
      const got = await invoke<RunTrace>('neobot_run_trace', { taskId: id })
      // ⛔ 回来时用户可能已展开别的轮次 ⇒ 只在**仍然是这一轮**时才落状态。
      setDetailFor((cur) => {
        if (cur === id) setDetail(got)
        return cur
      })
    } catch (e) {
      setDetailFor((cur) => {
        if (cur === id) setDetailErr(String(e).slice(0, 200))
        return cur
      })
    }
  }, [detailFor])

  // ── 跑轮生命周期信号 + 卡住看门狗 ─────────────────────────────────
  //
  // ⭐ 有了 `neobot:run` 三段信号，「正在想…」才第一次是**真的**在描述
  //    后端状态，而不是一个凭 `busy` 猜出来的脉冲。
  // ⛔ `runTrace` 只存**最后一条 finished 的服务端 trace 摘要**：它是当次的
  //    补充，**不冒充**轨迹页（轨迹页读库、覆盖历史轮）。
  const [lastRunTrace, setLastRunTrace] = useState<RunSignal['trace'] | null>(null)
  // ⛔ 看门狗只在「已 started 且还没结束」时跑；秒数进一个 state 让文案跟着走。
  const [stallSecs, setStallSecs] = useState(0)
  useEffect(() => {
    let alive = true
    let un: (() => void) | undefined
    void listen<RunSignal>(RUN_EVENT, (ev) => {
      const s = ev.payload
      if (!alive) return
      // ⛔ 只认**本会话**的信号：busy 是按会话隔离的，信号也必须
      //    （否则 A 会话在跑，B 会话顶部跟着显示「正在跑」= 又一处串台）。
      const mine = (s.convo_id ?? null) === (sel ?? null)
      if (!mine) return
      setRunPhase(s.phase)
      if (s.phase === 'finished') setStallSecs(0)
      if (s.phase === 'finished') setLastRunTrace(s.trace.length ? s.trace : null)
    })
      .then((fn) => {
        if (alive) un = fn
        else fn() // ⛔ 挂载已撤 ⇒ 立刻退订（否则往已死的组件发事件）
      })
      .catch(() => {
        // ⛔ listen 失败**不让界面炸**：轨迹页与信号是增强，不是主流程。
        //    非静默：把它写成一条可查的前端日志（ipc.ts 的活动面板看不到这里）。
        void invoke('log_frontend', {
          level: 'warn', target: 'neobot-root',
          message: `listen(${RUN_EVENT}) failed: trace signal unavailable`,
        }).catch(() => {})
      })
    return () => {
      alive = false
      un?.()
    }
  }, [sel])

  // 看门狗计时：只有「在跑」且没超阈值时才有意义。
  useEffect(() => {
    if (runPhase !== 'started' || !busy) {
      setStallSecs(0)
      return
    }
    const t0 = Date.now()
    const id = window.setInterval(() => {
      const secs = Math.floor((Date.now() - t0) / 1000)
      // ⛔ 超过阈值就**停表**而不是继续涨：一个无界的数字会诱使人盯着它数，
      //    而它此时不提供新信息（真结果只会来自后端超时或成功）。
      if (secs <= RUN_STALL_SECS) setStallSecs(secs)
    }, 1000)
    return () => window.clearInterval(id)
  }, [runPhase, busy])
  const stalled = runPhase === 'started' && busy && stallSecs >= RUN_STALL_SECS

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
    setHasMore(false)
    setOldestSeq(null)
    // ⭐⭐ 改走**分页**命令：`beforeSeq` 缺席 = 取**最新**一页
    // （store 层是 `seq > after_seq` 正序取，故「最新一页」= 不设游标）
    void invoke<{ messages: ChatMessage[], hasMore: boolean, nextSeq: number | null }>(
      'neobot_convo_messages_page', { convoId: sel, beforeSeq: null, limit: PAGE },
    )
      .then((page) => {
        if (!alive) return
        setHasMore(page.hasMore)
        setOldestSeq(page.nextSeq)
        setMsgs(page.messages.map((r): Msg => ({
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
    // 租约倒计时 + 回执：1s 轮询（只读；失败静默 —— 它是辅助信息，不该刷错误）
    const tickComputer = () => {
      void invoke<{ active: boolean; owner: string; remaining_ms: number }>('neobot_computer_lease_status')
        .then(setLease)
        .catch(() => {})
      void invoke<
        Array<{ at: string; action: string; status: string; advice: string; sent: boolean }>
      >('neobot_computer_receipts', { limit: 5 })
        .then(setReceipts)
        .catch(() => {})
    }
    tickComputer()
    const timer = window.setInterval(tickComputer, 1000)
    return () => window.clearInterval(timer)
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

  /** ⭐⭐ 加载更早一页（把更早的历史**前置**到现有列表，不整体替换）。 */
  const loadOlder = useCallback(async () => {
    if (!sel || loadingMore || !hasMore || oldestSeq == null) return
    setLoadingMore(true)
    const convo = sel
    try {
      const page = await invoke<{ messages: ChatMessage[], hasMore: boolean, nextSeq: number | null }>(
        'neobot_convo_messages_page', { convoId: convo, beforeSeq: oldestSeq, limit: PAGE },
      )
      if (convo !== sel) return   // ⭐ 会话已切 ⇒ 丢弃（与 send 同一套守卫）
      const older = page.messages.map((r): Msg => ({
        who: r.role === 'user' ? 'me' : 'bot',
        text: r.text,
        ts: r.created_at,
      }))
      // ⭐⭐ 前置而非替换 ⇒ ⭐ 已有滚动位置与阅读进度**不跳**
      setMsgs(m => [...older, ...m])
      setHasMore(page.hasMore)
      setOldestSeq(page.nextSeq)
    } catch (e) {
      // ⛔ 非静默：失败要让用户知道，否则按钮一直亮着像还能点
      setHasMore(false)
      void invoke('log_frontend', { level: 'error', target: 'neobot-root', message: `loadOlder: ${String(e).slice(0, 160)}` }).catch(() => {})
    } finally {
      setLoadingMore(false)
    }
  }, [sel, loadingMore, hasMore, oldestSeq, PAGE])

  async function send() {
    const text = draft.trim()
    if (!text || busy || histLoading) return
    // ⛔⛔ **抓住发起时的会话**，别用 resolve 时的 `sel`。
    //    `msgs` 是**当前会话**的列表；而 `setMsgs(m => [...m, …])` 在 resolve
    //    那一刻作用在**当时**的列表上。若用户在 `await` 期间切了会话
    //    （`:540` 那条路径正是切会话时 `setMsgs([])` + 重载历史），
    //    ⇒ 迟到的回复/失败气泡会**被追加进新会话的消息流**。
    //    ⓘ 历史加载有 `alive` 守卫（`:540`），**但它只护历史加载那几处**，
    //    `send()` 的 then/catch 此前**无任何守卫** ⇒ 这条路径是裸的。
    //    ⇒ 解法：记下发起时的 convoId，回来后比对，不符就**丢弃**（不串台）。
    const sendConvo = sel ?? null
    // ⭐⭐ 草稿清空按**发起时的会话**（⛔ 不是 `sel` —— await 期间可能已切走）
    if (sendConvo) setDrafts(d => ({ ...d, [sendConvo]: '' }))
    setMsgs(m => [...m, { who: 'me', text, ts: nowIso() }])
    // ⭐⭐ busy 按**发起时的会话**置位 ⇒ ⭐ **别的会话此刻仍可发**
    if (sendConvo) setBusyByConvo(b => ({ ...b, [sendConvo]: true }))
    try {
      // convo_id 缺席（无会话时）= 脱离会话手动跑，后端不落库。
      const r = await invoke<{ output?: string, text?: string }>('neobot_send', { convoId: sendConvo ?? undefined, text })
      // ⛔ 串台守卫：会话已变 ⇒ 这条回复属于**旧会话**，追加到新会话就是错的。
      if ((sel ?? null) !== sendConvo) return
      // ⭐⭐ **空白回复按失败处理**，不是成功（吸收 `42-evey/hermes-plugins`（MIT）
      //    `evey-delegate-model/_call_model` 的「空内容计为失败并重试」）。
      //    ⛔ 改前是 `r?.output ?? r?.text ?? t('chat.noOutput')` ⇒ 跑完了、
      //    返回体里没有正文时，界面上出现一条**看起来正常**的
      //    「（无输出）」气泡，既不能重发、也不标红。
      //    ⇒ 那是在把「没答上来」说成「答上来了但是空的」。
      const body = (r?.output ?? r?.text ?? '').trim()
      if (!body) {
        setMsgs(m => [...m, {
          who: 'bot',
          text: failHint('empty', ''),
          ts: nowIso(), failed: true,
          prompt: text,
          id: `f${nowIso()}-${m.length}`,
        }])
        return
      }
      setMsgs(m => [...m, { who: 'bot', text: body, ts: nowIso() }])
    } catch (e) {
      // ⛔ 同样要守：失败气泡串到别的会话同样是错的。
      if ((sel ?? null) !== sendConvo) return
      // ⛔ 失败气泡标成 failed：那样才能给「重发」，也不至于和正常回复混淆。
      setMsgs(m => [...m, {
        who: 'bot',
        // ⭐ 失败气泡**先给一句可据以行动的话**，原文放 `title` 里备查。
        //    判据是**成对标记**（见 classifyFailure），单个泛化词不算证据 ——
        //    否则网络抖动会被说成「去改密钥」，而密钥没问题。
        text: `${t('chat.sendFailedPrefix')}${failHint(classifyFailure(String(e)), String(e))}`,
        ts: nowIso(), failed: true,
        // ⭐ 存下**原始正文**：`重发` 直接用它，不必剥本地化前缀
        //    （切语言后剥前缀会把前缀一起发出去）。
        prompt: text,
        id: `f${nowIso()}-${m.length}`,
      }])
    } finally {
      // ⭐⭐ 按**发起时的会话**复位 busy —— ⛔ 若用 `sel`，切走后会把
      //    **新会话**误置/误清（那正是我们刚隔离掉的那类串台）
      if (sendConvo) setBusyByConvo(b => ({ ...b, [sendConvo]: false }))
      reloadUsage()
    }
  }

  /** 重发：拿失败气泡的原文再发一次（失败那条替换掉，不留残骸）。 */
  const retry = useCallback(
    async (msgId: string) => {
      // ⭐⭐ **按 id 寻址，不用数组下标**（理由见 `Msg.id` 的注释）。
      const bad = msgs.find(x => x.id === msgId)
      if (!bad?.failed || busy) return
      // ⭐⭐ 重发也按**发起时的会话**记 busy（否则同样会串到新会话）
      const retryConvo = sel ?? null
      setMsgs(m => m.filter(x => x.id !== msgId))
      if (retryConvo) setBusyByConvo(b => ({ ...b, [retryConvo]: true }))
      try {
        const r = await invoke<{ output?: string, text?: string }>('neobot_send', {
          convoId: retryConvo ?? undefined,
          // ⛔ 不再「剥本地化前缀」还原正文：改用发送时存下的原文。
          //    旧写法在切换语言后必然把前缀一起发出去。
          text: bad.prompt ?? bad.text,
        })
        // ⭐ 与首发同口径：空白 = 失败（可再重发），不是一条「（无输出）」。
        const body = (r?.output ?? r?.text ?? '').trim()
        if (!body) {
          setMsgs(m => [...m, {
            who: 'bot', text: failHint('empty', ''), ts: nowIso(), failed: true,
            prompt: bad.prompt ?? bad.text, id: `f${nowIso()}-${m.length}`,
          }])
          return
        }
        setMsgs(m => [...m, { who: 'bot', text: body, ts: nowIso() }])
      } catch (e) {
        setMsgs(m => [...m, {
          who: 'bot',
          text: `${t('chat.sendFailedPrefix')}${failHint(classifyFailure(String(e)), String(e))}`,
          ts: nowIso(), failed: true,
          // ⭐ 重发仍要能再重发 ⇒ 沿用**同一份原文**。
          prompt: bad.prompt ?? bad.text,
          id: `f${nowIso()}-${m.length}`,
        }])
      } finally {
        if (retryConvo) setBusyByConvo(b => ({ ...b, [retryConvo]: false }))
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
              {/* ⭐⭐ 2026-10-03 修「`truncate` 形同虚设」
                  ⛔ 改前：`<div class="flex">` 里直接放 `<span class="truncate">`
                  ⇒ ⭐⭐ **flex 子项的 `min-width` 默认是 `auto`**，
                  ⇒ ⭐⭐ **内容不换行时它不肯收缩** ⇒ `truncate` 的
                  `overflow:hidden; text-overflow:ellipsis` **永远不生效**
                  ⇒ 长会话标题**直接撑破侧栏**（这正是「不能被遮挡」的根因之一）。
                  ⇒ ⭐⭐ 修法就是 dsh `conversation-bar.tsx` 用的那一行：
                  **`flex-1 min-w-0`** —— `min-w-0` 才是让 `truncate` 生效的前提。*/}
              <div className="flex min-w-0 items-center gap-1">
                <span className={`min-w-0 flex-1 truncate text-[13px] ${active ? 'font-semibold text-ink' : 'text-ink'}`}>
                  {c.title || c.id}
                </span>
                {c.muted && (
                  <span
                    className="shrink-0 text-[12px] leading-none text-muted"
                    title={t('chat.muted')}
                    aria-label={t('chat.muted')}
                  >
                    {/* ⛔ 不用 emoji：彩色字形在深色侧栏里是唯一的彩色噪点，
                        且各平台字形不一致。短横杠即「静音条」，与文字同色。 */}
                    ▬
                  </span>
                )}
                {c.unread > 0 && (
                  <span className="ml-auto shrink-0 rounded-full bg-info px-1.5 text-[12px] tabular-nums text-btn-ink">
                    {c.unread}
                  </span>
                )}
              </div>
              <div className="truncate text-[12px] text-muted">
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
      {/* ⭐⭐ 侧栏宽：⭐ 此前用 `w-64`（Tailwind = 256px），
          而 `theme.css` 声明了 `--nb-side-w: 260px` 却**零消费**
          ⇒ ⭐⭐ **同一个语义有两个数字**，⭐ 这正是 douchat 栽过的坑
          （它的侧栏宽同时存在 324 / 322 / 244 三个值）。
          ⇒ ⭐⭐ 收敛到**单一真源**；⭐ `var()` 的 fallback 兜住
          ⭐ 万一 token 未定义（⭐ 与 `--nb-traffic-inset` 同一纪律）。*/}
      <aside className="flex min-h-0 w-[var(--nb-side-w,260px)] shrink-0 flex-col border-r border-line">
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
              className="h-[var(--nb-row-h)] w-[26px] shrink-0 rounded-lg border border-line text-[16px] leading-none text-muted hover:bg-panel-hover"
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
              {newErr && <div className="break-all text-[12px] text-[#c33b38]">{newErr}</div>}
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
        <div className="shrink-0 px-2.5 pb-1 pt-1 text-[12px] uppercase tracking-wide text-muted">
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
                className="mt-2 rounded-full border border-[#c33b38]/30 px-2 py-0.5 text-[12px]"
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
                        className="w-full px-2 py-1 text-left text-[12px] font-medium text-muted"
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
            className="flex w-full items-center gap-1 rounded-lg px-2 py-1.5 text-left text-[12px] text-muted hover:bg-panel-hover"
          >
            {/* ⭐⭐ 2026-10-03 遮挡根治：三段式收缩契约
                （形态照 dsh `conversation-bar.tsx` 的 slots 思路）。
                ⛔ 改前：`<span>标题</span>` + `<span class="ml-auto">计数</span>`
                ⇒ 两段**都没有收缩约束**，`ml-auto` 只推右边、不防挤压
                ⇒ ⭐⭐ 「记忆 12」这类标题一长，**右侧计数就被挤出/重叠**。 */}
            <span className="shrink-0" aria-hidden="true">{memOpen ? '▾' : '▸'}</span>
            {/* ⭐ `min-w-0` 是 flex 子项**能收缩的前提**（缺它则 `truncate` 无效）*/}
            <span className="min-w-0 flex-1 truncate">{t('chat.memory')}{mem && mem.lines.length > 0 ? ` ${mem.lines.length}` : ''}</span>
            {mem && mem.bytes > 0 && (
              // ⭐ `shrink-0` ⇒ 计数是**最后被牺牲**的，标题先省略
              <span className="ml-2 shrink-0 tabular-nums text-[12px]">
                {mem.bytes}/{mem.cap}
              </span>
            )}
          </button>
          {memOpen && (
            <div className="px-2 pb-2">
              {memErr && <div className="mb-1 break-all text-[12px] text-[#c33b38]">{memErr}</div>}
              {mem && mem.lines.length === 0 && !memErr && (
                <p className="mb-1 text-[12px] text-muted">
                  {t('chat.noMemory')}
                </p>
              )}
              {mem && mem.lines.length > 0 && (
                <ul className="mb-1 max-h-32 space-y-0.5 overflow-y-auto">
                  {mem.lines.map((l, i) => (
                    <li key={i} className="break-words text-[12px] leading-snug text-ink">
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
                  className="min-w-0 flex-1 rounded-lg border border-line bg-panel px-2 py-1 text-[12px] text-ink outline-none focus:border-[#2468f2]"
                />
                <button
                  type="button"
                  onClick={() => void addMemory()}
                  disabled={memBusy || !memDraft.trim()}
                  className="shrink-0 rounded-lg bg-btn-fill px-2 py-1 text-[12px] text-btn-ink disabled:opacity-40"
                >
                  {t('chat.memorySave')}
                </button>
              </div>
              <button
                type="button"
                onClick={() => void undoMemory()}
                disabled={memBusy || !mem || mem.revisions === 0}
                title={t('chat.memoryUndoTitle')}
                className="mt-1 w-full rounded-lg border border-line px-2 py-1 text-[12px] text-muted hover:bg-panel-hover disabled:opacity-40"
              >
                {t('chat.memoryUndo', { n: mem?.revisions ?? 0 })}
              </button>
            </div>
          )}
        </div>
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        {/* ⭐⭐⭐ 会话头**挂上栏族**（2026-10-04）。
         * ⛔ 改前：`h-12`（=48px）+ `px-4` + `border-b` ⇒ ⭐⭐ **没走栏族**，
         *   ⭐⭐ 48px 恰好等于顶栏的 `--nb-bar-h` ⇒ ⭐⭐ 于是**会话头与顶栏等高**，
         *   ⭐⭐ 而会话头里没有品牌也没有主按钮 ⇒ ⭐⭐ 视觉上「白白多出一条 48px 的带子」。
         * ✅ 现在：走 `.nb-band`（`--nb-bar-h-sub` = 44px）
         *   ⇒ ⭐⭐ **顶栏 48 / 次级栏 44** 的两级栏族第一次真正生效。
         * ⭐⭐ 对标 douchat 的教训：它顶栏与会话头**都是 46px**、纯人工约定，
         *   且 `inset:48px` 已漂 2px ⇒ ⭐⭐ **「靠约定统一」必然漂**，必须落到类。*/}
        <header className="nb-band nb-convo-head">
          {/* ⭐⭐ 对话 / 轨迹 双页签（2026-10-07）。
           * ⛔ 刻意放在**同一根栏**里而不是另起一根：另起一根会让
           *    「顶栏 48 / 会话头 44 / 页签 44」三根等高栏叠在一起，
           *    而这四行里只有一行在承载**位置信息**（页签 = 你在看哪一半）。
           *    ⭐ 栏族纪律（`theme.css` 的 `--nb-bar-h-sub`）说的就是这件事。
           * ⛔ 形态用 `role="tablist"` + `aria-selected` 而不是纯按钮：
           *    读屏用户需要知道「这两个是同一个东西的两半」，
           *    而两个独立按钮读起来就是两个独立功能。*/}
          <div role="tablist" aria-label={t('chat.tabHint')} className="flex shrink-0 items-center gap-0.5">
            {(['chat', 'trace'] as const).map((v) => (
              <button
                key={v}
                type="button"
                role="tab"
                aria-selected={view === v}
                data-testid={`nb-tab-${v}`}
                onClick={() => setView(v)}
                className={`rounded-md px-2 py-1 text-[12px] transition-colors ${
                  view === v
                    ? 'bg-btn-active font-semibold text-ink'
                    : 'text-muted hover:bg-panel-hover'
                }`}
              >
                {v === 'chat' ? t('chat.tabChat') : t('chat.tabTrace')}
              </button>
            ))}
          </div>
          {/* ⭐⭐ 同型缺陷**第 5 处**：⭐ 会话标题可很长（⭐ 用户可自命名）
           *    ⇒ 无 `min-width:0` ⇒ ⭐⭐ **撑破栏、把右侧操作挤出**。
           *    ⭐ `min-w-0` 是 ellipsis 的硬前提（本项目已栽 5 次）。
           * ⭐⭐ 2026-10-07：span → <h2>，给会话区一个真实标题层级
           *    （实测整个对话视图 `h1~h6` = 0 个 ⇒ 读屏用户拿不到任何
           *    文档结构，与视觉一样是「一层东西」。Tailwind preflight 已重置
           *    标题的 UA 外边距 ⇒ 同 className 视觉等价，不会撑乱栏）。*/}
          <h2 className="ml-2 min-w-0 flex-1 truncate text-sm font-semibold text-ink">
            {current?.title ?? 'NeoBot'}
          </h2>
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
          {/* ⛔ 今日用量**只在对话页**出现：轨迹页刻意不给任何用量数字
           *    （每轮没有可信的费用可摊，见 `nt_run_trace` 不变量 2）。
           *    在轨迹页显示一个**会话级**数字紧挨着**逐轮**行，
           *    会诱导用户拿它去除 —— 那正是一个凭空捏造出来的「每轮费用」。*/}
          {view === 'chat' && usageToday !== null && usageToday > 0 && (
            <span className="ml-2 truncate text-xs text-muted" title={t('chat.tokensTitle')}>
              {t('chat.tokensToday', { n: usageToday.toLocaleString() })}
            </span>
          )}
        </header>

        {view === 'trace' ? (
          <TraceView
            runs={runs}
            runsErr={runsErr}
            loading={runsLoading}
            reload={reloadRuns}
            detailFor={detailFor}
            detail={detail}
            detailErr={detailErr}
            toggleRun={toggleRun}
            lastRunTrace={lastRunTrace}
          />
        ) : (
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
<div className="mx-auto max-w-[var(--nb-col-w)] space-y-2">
              {/* ⭐⭐ 2026-10-03「加载更早」（接上 `neobot_convo_messages_page`）。
                  ⛔ 长会话此前一次性全量拉取 ⇒ 整段历史进 DOM。
                  ⭐ 放在消息列表**上方** ⇒ 更早的内容出现在顶部，符合阅读直觉。
                  ⛔ `hasMore` 为假时**完全不渲染** ⇒ ⛔ 不占位、不引视觉噪声。 */}
              {hasMore && (
                <button
                  type="button"
                  className="mx-auto block rounded-full border border-line px-3 py-1 text-[12px] text-muted hover:text-ink disabled:opacity-60"
                  disabled={loadingMore}
                  onClick={() => { void loadOlder() }}
                >
                  {loadingMore ? t('chat.loadingOlder') : t('chat.loadOlder')}
                </button>
              )}
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
                      className={`flex items-center gap-2 px-1 text-[12px] text-muted ${
                        groupEnd ? 'mt-0.5' : 'mt-0'
                      } ${m.who === 'me' ? 'justify-end' : ''} ${
                        !groupEnd && !m.failed ? 'invisible' : ''
                      }`}
                    >
                      {clockOf(m.ts)}
                      {m.failed && (
                        <button
                          type="button"
                          onClick={() => void retry(m.id ?? String(vi.index))}
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
              {/* ⭐⭐⭐ 2026-10-07 加 `aria-live="polite"`（此前**全树零 live region**）。
               *
               * # 为什么加
               *
               * 实测 `grep -rn aria-live neobot-ui/src/` **零命中**（唯一的
               * `role="alert"` 在 `shell.tsx:292` 的顶栏错误位）⇒
               * ⭐⭐ 「已受理 → 思考中 → 已停滞 Ns」这条**诚实增量链读屏完全听不到**。
               * 而这条链正是本轮专门做的（`run.started` 来自**后端信号**而非 `busy` 猜）。
               * ⓘ `nt_check_layout` 的 a11y 段只查 alt / 可读名 / label，
               *   ⛔ 不查 live region ⇒ 此前无人发现。
               *
               * # 为什么容器**常驻**而内容条件渲染
               *
               * ⛔ 若把 `aria-live` 放在 `{busy && ...}` **里面**，多数读屏
               *   **不会播报**：live region 与其内容同时插入 DOM 时，
               *   辅助技术常认为「没有变化发生」。
               *   ✅ 可靠形态 = 容器自页面加载就在 DOM 里，**只有内容变**。
               *   ⓘ 空闲时容器是**空的 flex**，高度 0 ⇒ 视觉零影响。
               *
               * # 为什么是 `polite` 而不是 `assertive`
               *
               * ⛔ `assertive` 会**打断**读屏当前朗读；「正在思考」「已停滞」
               *   是背景进展提示，不是打断级事件 ⇒ `polite` 排队播报。
               * ⭐ `aria-atomic` 让「已受理」整句读出，而不是只读变化的两个字。
               *
               * ⭐ 本块**不在**虚拟化容器内（是 `msgVirtualizer` 的兄弟节点）
               *   ⇒ ⛔ 不会有「虚拟化反复挂载 ⇒ 重复播报」的风险。*/}
              <div className="flex" aria-live="polite" aria-atomic="true">
                {busy && (
                  <div className="rounded-2xl rounded-bl-md bg-[#ededef] px-3 py-2 text-[13px] text-[#61666b]">
                    {/* ⭐ 两条文案由**后端信号**决定，不由 `busy` 猜：
                     *    `busy` 只说「本地有个 await 没回来」。
                     *    `runPhase === 'started'` 才说「后端确实接了」，
                     *    两者在「invoke 发出但后端还没受理」的窗口里是不同的。*/}
                    <span className="inline-flex items-center gap-1">
                      {/* ⛔ 脉冲点挂 `nb-run-pulse` ⇒ `prefers-reduced-motion`
                       *    下落成**静态**可见的点（见 `nb-reduced-motion.css`：
                       *    那里刻意不写成 `animation: none` 了事，因为起始帧是透明的）。*/}
                      <span className="nb-run-pulse inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-[#8a8f98]" />
                      {runPhase === 'started' ? t('run.started') : t('chat.thinking')}
                    </span>
                    {/* ⭐⭐ 卡住**有界**且**说清下一步**（吸收 hermes-workspace 的
                     *    两档 stall 预算；本仓只有一档，因为它只有一次同步 HTTP）。
                     *    ⛔ 改前是一个**无界**的脉冲：真卡住时它会一直跳，
                     *    用户既不知道是「慢」还是「死了」，也等不到任何提示。*/}
                    {stalled && (
                      <p className="mt-1 max-w-[var(--nb-col-w)] text-[12px] leading-snug text-[#8a5a12]">
                        {t('run.stalled', { n: stallSecs })}
                      </p>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
          {/* 回到底部：只在用户已经滚上去时才出现（贴底时它是噪声）。 */}
          {!pinned && msgs.length > 0 && (
            <button
              type="button"
              onClick={scrollToEnd}
              className="sticky bottom-2 mx-auto block rounded-full border border-line bg-panel px-3 py-1 text-[12px] text-muted shadow-sm hover:bg-panel-hover"
            >
              {t('chat.backToBottom')}
            </button>
          )}
        </div>
        )}

        <div className="shrink-0 border-t border-line p-3">
          <div className="mx-auto flex max-w-[var(--nb-col-w)] items-end gap-2">
            <textarea
              ref={taRef}
              value={draft}
              onChange={e => {
                // ⭐⭐ 草稿写回**所属会话**，⛔ 不是当前选中的
                //    （输入期间切会话，草稿必须跟着**它本来属于的**会话走）
                const id = sel
                if (!id) return
                setDrafts(d => ({ ...d, [id]: e.target.value }))
              }}
              onKeyDown={(e) => {
                // ⛔⛔⛔ **必须有 IME 守卫**：`isComposing` 为真时，回车是
                //    **输入法选词确认**，不是发送。
                //    ⓘ 后果不是「多发一次」而是**灾难性的**：选词确认被当发送 ⇒
                //    拼音串（还在 preedit、未上屏）被当成正文发出去。
                //    ⓘ 而 `zh-CN` 是本仓的**默认语言** ⇒ 中文用户必遇。
                //    ⓘ `nativeEvent.isComposing` 与 `e.isComposing` 在 React
                //    的合成事件里等价，但用 native 更明确（不被池化影响）。
                //    ⛔ 也**不能**只判 `e.keyCode === 229`（老 hack，部分 IME 不触发）。
                if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
                  e.preventDefault()
                  void send()
                }
              }}
              rows={1}
              placeholder={t('chat.inputPlaceholder')}
              aria-label={t('chat.inputLabel')}
              className="max-h-40 min-h-[var(--nb-row-2-h)] flex-1 resize-none overflow-y-auto rounded-2xl border border-line bg-panel px-3 py-2 text-[13px] text-ink outline-none focus:border-info-hover"
            />
            {/* computer 控制租约 + 回执三态（吸收 computer-use P6/P7）。
             *  租约 = 「此刻谁持有控制权、到什么时候」；回执 = 「这个动作发出去没有」。
             *  两者都只读后端真相（lease registry / ledger），前端不做本地猜测。 */}
            {(lease.active || receipts.length > 0) && (
              <div className="mx-auto mb-1 flex w-full max-w-[var(--nb-col-w)] flex-wrap items-center gap-2 text-[12px]">
                {lease.active && (
                  <span
                    className="rounded-full border px-2 py-0.5"
                    style={{ borderColor: 'var(--nb-color-line-strong)' }}
                    title={t('chat.leaseHint')}
                  >
                    {t('chat.lease')} {Math.ceil(lease.remaining_ms / 1000)}s · {lease.owner}
                  </span>
                )}
                {receipts.slice(0, 3).map((r, i) => (
                  <span
                    key={`${r.at}-${i}`}
                    className="rounded-full border px-2 py-0.5"
                    style={{
                      borderColor:
                        r.status === 'applied'
                          ? 'var(--nb-color-line-strong)'
                          : r.status === 'outcome_unknown'
                            ? 'var(--nb-color-btn-danger-hover)'
                            : 'var(--nb-color-line)',
                      opacity: r.status === 'failed' ? 0.7 : 1,
                    }}
                    title={`${r.action} · advice=${r.advice || '-'} · sent=${r.sent}`}
                  >
                    {r.action || '-'} · {r.status}
                    {r.advice ? ` · ${r.advice}` : ''}
                  </span>
                ))}
              </div>
            )}
            {/* 全局急停（kill switch）。吸收 computer-use P7：急停属于**协议层**，
             * UI 只是它的第二个入口；0 轮可停时如实说 0，不假装成功。 */}
            {busy && (
              <button
                type="button"
                onClick={() => {
                  void invoke<number>('neobot_stop_all')
                    .then(n =>
                      void invoke('log_frontend', {
                        level: 'info',
                        target: 'neobot-root',
                        message: `stop_all flipped ${String(n)}`,
                      })
                    )
                    .catch(e =>
                      void invoke('log_frontend', {
                        level: 'error',
                        target: 'neobot-root',
                        message: `stop_all failed: ${String(e).slice(0, 160)}`,
                      })
                    )
                }}
                aria-label={t('chat.stopAllLabel')}
                title={t('chat.stopAllHint')}
                className="nb-danger h-9 shrink-0 rounded-full border px-3 text-[13px]"
                style={{
                  borderColor: 'var(--nb-color-line-strong)',
                  color: 'var(--nb-color-ink)',
                }}
              >
                {t('chat.stopAll')}
              </button>
            )}
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
            <p className="mx-auto mt-1 max-w-[var(--nb-col-w)] text-[12px] text-muted">{t('chat.historyPaused')}</p>
          )}
        </div>
        {/* ⛔ 输入区**始终可见**（不随页签切走）：轨迹页是「去看跑过什么」，
         *    不是「去另一个地方发消息」。把它藏起来会让「看完回去发」多一步，
         *    而那一页切回来草稿还在（草稿按会话分区，本来就防串台）。 */}
      </section>
    </div>
  )
}

/** 单步输出的行数上限：超过就折叠成「还有 N 行」。 */
const STEP_LINES = 6

/**
 * 轨迹页（对话页的另一半）。
 *
 * # 为什么单独一个组件而不是内联在 `NeoBotRoot` 里
 *
 * ⛔ 它有自己的**拉取生命周期**（按需拉、切会话重拉、展开才拉详情），
 *    塞进根组件会让根组件的状态面多出 8 个只服务这一页的 state，
 *    而根组件的任何一次 setState 都会**重渲染整个侧栏 + 对话流 + 轨迹页**。
 *    拆开之后，轨迹页内部的 tick（看门狗那类）不再牵动侧栏。
 *
 * # 三态是真的三态（吸收 taste-skill §4.5「LLM 只写成功态」）
 *
 *   · `null`   = 还没读 ⇒ 读骨架（不是「没有」）
 *   · `''`     = **读不到** ⇒ 说读不到 + 给重读按钮
 *   · `[]`     = **真的没有** ⇒ 说清是「没跑过」并指向对话页
 *
 * ⭐ 吸收 `super-hermes`（MIT）的做法：长输出**按行数省略**为
 *   `(+N lines)` 而不是截断字符 —— 「有多大」是信息，「前 200 个字符」
 *   是噪音。
 */
function TraceView(props: {
  runs: RunRow[] | null
  runsErr: string
  loading: boolean
  reload: () => void
  detailFor: string | null
  detail: RunTrace | null
  detailErr: string
  toggleRun: (id: string) => void | Promise<void>
  lastRunTrace: RunSignal['trace'] | null
}) {
  const { runs, runsErr, loading, reload, detailFor, detail, detailErr, toggleRun, lastRunTrace } = props
  // 展开的行数：每一步独立（⛔ 不共用一个计数 —— 共用会让「展开第 3 步」
  // 连带把第 1 步也展开，而用户只点了第 3 步）。
  const [openSteps, setOpenSteps] = useState<Record<number, boolean>>({})

  return (
    <div data-testid="nb-trace" className="relative min-h-0 flex-1 overflow-y-auto px-4 py-3">
      <div className="mx-auto max-w-[var(--nb-col-w)] space-y-2">
        {/* ⭐ 重读按钮只在**正在读**时可用 —— ⛔ 此前 `loading` 只进了 deps、
         *    界面上完全不可见，于是「读了一次」与「读了很多次」长得一模一样。
         *    （这类「变量算了但没渲染」是本仓 §4 教训 1 的 UI 版：没人看见，
         *    就等于它不存在。）*/}
        {runsErr && (
          <button
            type="button"
            className="rounded-full border border-line px-2 py-0.5 text-[12px] text-muted hover:bg-panel-hover disabled:opacity-40"
            onClick={reload}
            disabled={loading}
          >
            {loading ? t('trace.loading') : t('trace.retry')}
          </button>
        )}
        {/* ⛔ 当次的服务端 trace 摘要：它是**补充**，不是轨迹页的主体，
         *    所以排在最上面并**标清来源**，免得被当成「这就是全部」。*/}
        {lastRunTrace && lastRunTrace.length > 0 && (
          <section className="rounded-lg border border-line bg-panel p-2" data-testid="nb-trace-live">
            <h3 className="text-[12px] font-medium text-muted">{t('trace.liveTitle')}</h3>
            <ul className="mt-1 space-y-0.5">
              {lastRunTrace.map((r, i) => (
                <li key={i} className="break-words text-[12px] text-ink">
                  <span className="text-muted">{r.kind}</span> {r.detail}
                </li>
              ))}
            </ul>
          </section>
        )}

        {runsErr && (
          <div className="m-1 rounded-lg bg-btn-danger-hover p-2 text-xs" style={{ color: '#a3272b' }}>
            <div className="font-semibold">{t('trace.loadFailed')}</div>
            <div className="mt-1 break-all">{runsErr}</div>
            <button
              type="button"
              className="mt-2 rounded-full border border-line px-2 py-0.5 text-[12px]"
              onClick={reload}
            >
              {t('trace.retry')}
            </button>
          </div>
        )}

        {/* ⛔ 骨架屏而不是「暂无」：taste-skill §4.5「避免泛用转圈」。
         *    这里用**与行同高**的条 ⇒ 读的时候版式不跳。*/}
        {!runsErr && runs === null && (
          <div className="space-y-1.5" aria-busy="true" data-testid="nb-trace-skeleton">
            {[0, 1, 2].map((i) => (
              <div key={i} className="h-[var(--nb-row-2-h)] rounded-lg bg-panel-2" />
            ))}
            <p className="pt-1 text-center text-[12px] text-muted">{t('trace.loading')}</p>
          </div>
        )}

        {!runsErr && runs !== null && runs.length === 0 && (
          <div className="flex flex-col items-center justify-center gap-2 py-10 text-center">
            <div className="text-[13px] font-medium text-ink">{t('trace.empty')}</div>
            <div className="max-w-[320px] text-[12px] leading-relaxed text-muted">
              {t('trace.emptyHint')}
            </div>
          </div>
        )}

        {!runsErr && runs !== null && runs.length > 0 && (
          <ul className="space-y-1.5" data-testid="nb-trace-list">
            {runs.map((r) => {
              const open = detailFor === r.id
              // ⛔ 详情**只认正在展开的那一轮**：拿 `detail.run.id !== r.id`
              //    来判会漏掉「刚展开、还没回来」的那一瞬（此时 detail 是旧的）。
              const mine = open && detail?.run.id === r.id
              return (
                <li key={r.id} className="rounded-lg border border-line bg-panel">
                  <button
                    type="button"
                    data-testid="nb-trace-run"
                    aria-expanded={open}
                    onClick={() => void toggleRun(r.id)}
                    className="flex w-full items-center gap-2 px-2.5 py-2 text-left transition-colors hover:bg-panel-hover"
                  >
                    <span aria-hidden="true" className="shrink-0 text-muted">
                      {open ? '▾' : '▸'}
                    </span>
                    {/* ⭐ `min-w-0` 在 `truncate` 之前（flex 子项不收缩则
                     *    ellipsis 永不生效 —— 本仓已栽 5 次，见 renderConvoRow）。*/}
                    <span className="min-w-0 flex-1 truncate text-[13px] text-ink">
                      {r.title || r.id}
                    </span>
                    {/* ⛔ 状态**原样显示**库侧给的串：库侧不解析（可能是不认识的新
                     *    状态），这里也不映射成已知集合 —— 那会把「新的」显示成
                     *    「没有」而丢信息（`nt_run_trace` 不变量 1）。*/}
                    <span className="shrink-0 text-[12px] text-muted" title={r.status}>
                      {r.status}
                    </span>
                    <span className="shrink-0 text-[12px] tabular-nums text-muted">
                      {t('trace.steps', { n: r.steps })}
                    </span>
                    {/* ⛔ 失败步数 > 0 才出现，且用**危险色**：0 个失败步时
                    //    显示「0 失败」是纯噪声。*/}
                    {r.failed_steps > 0 && (
                      <span
                        className="shrink-0 text-[12px] tabular-nums"
                        style={{ color: '#a3272b' }}
                      >
                        {t('trace.failedSteps', { n: r.failed_steps })}
                      </span>
                    )}
                  </button>

                  {open && (
                    <div className="border-t border-line px-2.5 pb-2 pt-1.5">
                      {detailErr ? (
                        <p className="break-all text-[12px]" style={{ color: '#a3272b' }}>
                          {t('trace.readFailed')}: {detailErr}
                        </p>
                      ) : !detail ? (
                        // ⛔ 「正在读」与「读了但没有」必须分开（对照 `noSteps`）。
                        <p className="text-[12px] text-muted">{t('trace.loading')}</p>
                      ) : (
                        <>
                          {mine && detail.steps.length === 0 && (
                            <p className="mb-1 text-[12px] text-muted">{t('trace.noSteps')}</p>
                          )}
                          {mine && detail.steps.map((s) => {
                            const full = openSteps[s.id] ?? false
                            const lines = s.output.split('\n')
                            const clipped = !full && lines.length > STEP_LINES
                            return (
                              <div key={s.id} className="border-b border-line py-1 last:border-b-0">
                                <div className="flex items-center gap-2 text-[12px]">
                                  <span className="shrink-0 tabular-nums text-muted">#{s.n}</span>
                                  {/* ⭐ 人化标签：只留末段，不铺完整路径（吸收
                                      hermes-workspace 的 tool 标签做法）。*/}
                                  <span className="min-w-0 flex-1 truncate text-ink">
                                    {stepLabel(s.tool)}
                                  </span>
                                  <span
                                    className="shrink-0"
                                    style={{ color: s.ok ? undefined : '#a3272b' }}
                                  >
                                    {s.ok ? t('trace.stepOk') : t('trace.stepFailed')}
                                  </span>
                                </div>
                                {/* ⭐ 长输出按**行**省略（`(+N 行)`），不按字符截断：
                                    「多大」是信息，「前 200 字」是噪音。
                                    吸收 super-hermes 的 `(+27 lines)` 形态。*/}
                                {s.output && (
                                  <pre
                                    className={`mt-0.5 whitespace-pre-wrap break-words text-[12px] leading-snug text-muted ${
                                      clipped ? 'max-h-[7.5em] overflow-hidden' : ''
                                    }`}
                                  >
                                    {clipped ? lines.slice(0, STEP_LINES).join('\n') : s.output}
                                  </pre>
                                )}
                                {(lines.length > STEP_LINES || full) && (
                                  <button
                                    type="button"
                                    onClick={() =>
                                      setOpenSteps((p) => ({ ...p, [s.id]: !full }))
                                    }
                                    className="mt-0.5 text-[12px] text-muted underline hover:text-ink"
                                  >
                                    {clipped
                                      ? t('trace.moreLines', { n: lines.length - STEP_LINES })
                                      : t('trace.showLess')}
                                  </button>
                                )}
                              </div>
                            )
                          })}
                          {mine && (
                            <div className="mt-1.5">
                              <h4 className="text-[12px] font-medium text-muted">
                                {t('trace.changes')}
                              </h4>
                              {detail.changes.length === 0 ? (
                                <p className="text-[12px] text-muted">{t('trace.noChanges')}</p>
                              ) : (
                                <ul className="mt-0.5 space-y-0.5">
                                  {detail.changes.map((c) => (
                                    <li key={c.id} className="flex items-baseline gap-2 text-[12px]">
                                      <span className="min-w-0 flex-1 truncate text-ink" title={c.path}>
                                        {changeLabel(c.kind, c.path)}
                                      </span>
                                      <span className="shrink-0 tabular-nums text-[12px] text-muted">
                                        {bytesLabel(c.kind, c.bytes)}
                                      </span>
                                      {/* ⭐⭐ 2026-10-07：渲染 `content_omitted`。
                                       * ⛔ 此前它在 TS 接口里声明着、却**从不显示** ——
                                       *   而 store 段头写明的意图正是「UI 据此说
                                       *   「内容已略去」」。⇒ 链路是「写 → 传到界面 →
                                       *   **丢在最后一格**」，与本轮开头修的
                                       *   `AgentRunResult.trace` **完全同型**。
                                       *   ⛔ 而且它是被 `check-dead-config-flag`
                                       *   当成**新增死开关**抓出来的 —— 门抓的是真缺陷。
                                       *   诚实边界：字节数**是真的**（见 store 注释
                                       *   「写/改=改后内容长度」），略去的是**内容本体**；
                                       *   而本轨迹页**本来就不展示 diff**，所以这句话
                                       *   说明的是「库里没存内容」，不是「这里少显示了」。*/}
                                      {c.content_omitted && (
                                        <span className="shrink-0 text-[12px] text-muted">
                                          {t('trace.contentOmitted')}
                                        </span>
                                      )}
                                    </li>
                                  ))}
                                </ul>
                              )}
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  )}
                </li>
              )
            })}
          </ul>
        )}

        {/* ⛔ 用途说明**常驻**（不是 tooltip）：`trace.usageHidden` 说的是
         *    「为什么这里没有费用」—— 一个**需要解释的缺席**，
         *    藏在 hover 里就等于没有。*/}
        <p className="pt-1 text-center text-[12px] text-muted">{t('trace.usageHidden')}</p>
      </div>
    </div>
  )
}
