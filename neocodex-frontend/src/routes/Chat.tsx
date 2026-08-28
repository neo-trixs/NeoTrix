import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import {
  Square, RotateCcw, Edit2, Copy, Check, AlertCircle, AlertTriangle, Highlighter, X, Info,
  FolderTree, Bug, FlaskConical,
  Search, Cpu, Zap, FileText, AtSign,
} from 'lucide-solid'
import { NeoSend, NeoChevronRight } from '../components/neo-icons'
import { AutonomyMeter } from '../components/AutonomyMeter'
import { rootCause } from '../lib/errorRootCause'
import { chatStore, Message, ToolCallRecord, NeoCodexAttachmentDto } from '../stores/chat'
import { Sidebar } from '../components/Sidebar'
import { SettingsModal } from '../components/SettingsModal'
import { RightBar } from '../components/RightBar'
import { CoworkView } from '../components/CoworkView'
import { PERMISSION_MODES, type PermissionMode } from '../components/PermissionModeSelector'
import { ModelSwitcher } from '../components/ModelSwitcher'
import { ToolCallCard } from '../components/ToolCallCard'
import { FilePreview } from '../components/FilePreview'
import { Markdown } from '../components/Markdown'
import { GitPanel } from '../components/GitPanel'
import { ScheduledTasks } from '../components/ScheduledTasks'
import { CostDashboard } from '../components/CostDashboard'
import { CheckpointTimeline } from '../components/CheckpointTimeline'
import { SideChat } from '../components/SideChat'
import { ComputerUse } from '../components/ComputerUse'
import { TaskList } from '../components/TaskList'
import { LivePreview } from '../components/LivePreview'
import { SlashMenu, type SlashCommandDef } from '../components/SlashMenu'
import { runSlashDispatch, parseRunCommand, type SlashContext } from './chat/slashCommands'
import { HeroMark, UserIcon, BotIcon } from './chat/avatars'
import { foldPreview, guessMime, formatSize, estimateTokens, greeting } from '../lib/text'
import { CommandPalette, type PaletteCommand } from '../components/CommandPalette'
import { clsx } from 'clsx'
import { neocodex, system, unified, errText, harness } from '../api'
import type { HarnessRunResponse, HarnessProgressEvent, HarnessStep, HarnessApproval } from '../api/harness'
import { listen } from '@tauri-apps/api/event'
import { HarnessReportCard } from '../components/HarnessReportCard'
import { ApprovalPanel } from '../components/ApprovalPanel'
import { FileEditorPanel } from '../components/FileEditorPanel'
import { AgentActivityBar, type AgentPhase } from '../components/AgentActivityBar'
import { AgentActivityLog, type ActivityStep } from '../components/AgentActivityLog'
import { query } from '../api/query'
import { usePolling } from '../lib/usePolling'
import { subscribeStream, subscribeMenuEvents, type UnlistenFn } from '../api/events'
import type { AgentStatus } from '../api/types'

const actionBtnClass =
  'action-btn p-1.5 rounded-lg text-text-muted/70 hover:text-text-primary hover:bg-white/70 hover:shadow-sm transition-all duration-150'

/* 长消息内容折叠阈值（任务4：超长 assistant 消息折叠；末条/流式消息始终全量渲染，保证流式安全） */
const LONG_MSG_FOLD_CHARS = 6000
const LONG_MSG_SNIPPET_CHARS = 4000

/* 权限模式徽章短标签（对标 Claude 顶栏 mode 徽章） */
const MODE_SHORT_LABEL: Record<PermissionMode, string> = {
  auto: '自动',
  manual: '手动',
  accept_edits: '接受编辑',
  plan: '规划',
}

/* —— 斜杠命令（对标 Claude Code / 命令菜单） —— */
const SLASH_COMMANDS: SlashCommandDef[] = [
  { id: 'clear', label: '清除会话', desc: '清空当前会话全部消息', keywords: ['clear'] },
  { id: 'new', label: '新建会话', desc: '开启一段新对话', keywords: ['new'] },
  { id: 'compact', label: '压缩会话', desc: '精简上下文继续对话', keywords: ['compact'] },
  { id: 'model', label: '切换模型', desc: '查看 / 切换当前模型', keywords: ['model', 'models'] },
  { id: 'status', label: '运行状态', desc: '查看模型/上下文/成本与用量', keywords: ['status', 'health'] },
  { id: 'cost', label: '成本统计', desc: '查看 token 用量与估算成本', keywords: ['cost', 'spend', 'usage'] },
  { id: 'export', label: '导出会话', desc: '导出当前会话为 Markdown', keywords: ['export', 'share'] },
  { id: 'run', label: '运行 Harness 任务', desc: '完整执行闭环：/run <任务>', keywords: ['run', 'harness', '执行'] },
  { id: 'help', label: '快捷键帮助', desc: '显示常用快捷键说明', keywords: ['help', '?'] },
]

/* —— 设计 v2 图标（HeroMark/UserIcon/BotIcon）见 chat/avatars.tsx —— */

export function Chat() {
  const [inputValue, setInputValue] = createSignal('')
  const [textareaRef, setTextareaRef] = createSignal<HTMLTextAreaElement | null>(null)
  const [editingMessageId, setEditingMessageId] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false)
  // 左栏可拖拽宽度（毫米级 UI 打磨，对标灵活布局）
  const [sidebarWidth, setSidebarWidth] = createSignal(280)
  let sbResizing = false
  const onSidebarResizeDown = (e: MouseEvent) => {
    e.preventDefault()
    sbResizing = true
    const onMove = (ev: MouseEvent) => {
      if (!sbResizing) return
      setSidebarWidth(Math.min(460, Math.max(200, ev.clientX)))
    }
    const onUp = () => {
      sbResizing = false
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }
  const [settingsOpen, setSettingsOpen] = createSignal(false)
  const [streamError, setStreamError] = createSignal<string | null>(null)
  // OS 活动透明度层（对标 2026 Agent UX：agent 操作必须可观测，anti black-box）
  // 纯前端聚合既有流式事件，后端无需改动
  const [agentPhase, setAgentPhase] = createSignal<AgentPhase>('idle')
  const [agentDomain, setAgentDomain] = createSignal<string | null>(null)
  const [agentToolCount, setAgentToolCount] = createSignal(0)
  const [agentLastActivity, setAgentLastActivity] = createSignal<string | null>(null)
  // 活动日志（审计层）：滚动记录阶段/推理/工具/错误/完成，独立于对话线程
  const [agentLog, setAgentLog] = createSignal<ActivityStep[]>([])
  const [logOpen, setLogOpen] = createSignal(false)
  // 结构化错误三段式（what/why/next）：后端可选填充，前端缺失时推导
  const [streamErrorDetail, setStreamErrorDetail] = createSignal<{ what: string; why: string; next: string } | null>(null)
  const pushLog = (step: Omit<ActivityStep, 'ts'>) => {
    setAgentLog((prev) => {
      const next = [...prev, { ...step, ts: Date.now() }]
      return next.length > 24 ? next.slice(next.length - 24) : next
    })
  }
  // 信息通知（区别于 streamError 错误通道：中性色 / InfoIcon，非故障）
  const [infoNotice, setInfoNotice] = createSignal<string | null>(null)
  // 信息通知计时器：新通知接管旧计时器，避免快速触发（如连按 Shift+Tab）时旧计时器误清新通知
  let infoNoticeTimer: ReturnType<typeof setTimeout> | undefined
  // F10: 流式看门狗——onDone 永不到达（传输丢失/后端异常退出）时强制复位，
  // 防止 isGenerating 永久卡死输入。启动于 sendMessage，清除于 onDone/stop/失败。
  let streamWatchdogTimer: ReturnType<typeof setTimeout> | undefined
  const showInfo = (msg: string, ms = 3000) => {
    if (infoNoticeTimer) clearTimeout(infoNoticeTimer)
    setInfoNotice(msg)
    infoNoticeTimer = setTimeout(() => setInfoNotice(null), ms)
  }
  const showError = (msg: string) => {
    setStreamError(msg)
    setTimeout(() => setStreamError(null), 3000)
  }
  // Harness 能力路由（对话即OS）— 每次发送后非阻塞解析，透明展示命中域
  const [harnessRoute, setHarnessRoute] = createSignal<{ tag: string; domain: string; specialist: string } | null>(null)
  const [harnessReport, setHarnessReport] = createSignal<HarnessRunResponse | null>(null)
  const [harnessRunning, setHarnessRunning] = createSignal(false)
  const [harnessSteps, setHarnessSteps] = createSignal<HarnessStep[]>([])
  // 左栏 bot 审批流可见性：挂载时拉取待审批队列（可 approve/reject 交互）
  const [approvals, setApprovals] = createSignal<HarnessApproval[]>([])
  const refreshApprovals = () => {
    harness.harnessApprovalList().then(setApprovals).catch(() => setApprovals([]))
  }
  onMount(() => {
    refreshApprovals()
  })
  // 左栏 bot 文件内联编辑面板开关
  const [showFileEditor, setShowFileEditor] = createSignal(false)
  // Harness 能力地图（⌘K 面板发现用，挂载时懒加载）
  const [harnessCaps, setHarnessCaps] = createSignal<{ capability_tag: string; domain: string; specialist: string; keywords: string[]; description: string }[]>([])
  const loadHarnessCaps = async () => {
    try {
      const caps = await harness.harnessApiMap()
      setHarnessCaps(caps)
    } catch {
      /* 发现性能力，失败静默 */
    }
  }
  onMount(() => { void loadHarnessCaps() })
  const [copiedId, setCopiedId] = createSignal<string | null>(null)
  const [permissionMode, setPermissionMode] = createSignal<PermissionMode>('auto')
  const [annotationHint, setAnnotationHint] = createSignal<string | null>(null)
  // 批次1：Plan Mode 批准流 — plan 权限模式下完成的助理回复列为待批准规划（对标 Claude Code plan 审阅）
  const [planPending, setPlanPending] = createSignal<{ msgId: string } | null>(null)
  const [activeModel, setActiveModel] = createSignal<string | null>(null)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)
  // ⌘K 命令面板（对标 Claude Code / Osaurus 命令菜单）：全局唤起，动作复用既有 handler
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  // ⌘K 最近使用（对标 Raycast：置顶高频命令）
  const [recentPaletteIds, setRecentPaletteIds] = createSignal<string[]>([])
  const pushRecentCmd = (id: string) =>
    setRecentPaletteIds((ids) => [id, ...ids.filter((x) => x !== id)].slice(0, 5))
  // 统一命令桥：CLI 命令目录（懒加载，供 ⌘K 面板执行 /help /config /stats 等）
  const [unifiedCliCmds, setUnifiedCliCmds] = createSignal<PaletteCommand[]>([])
  const execUnifiedCli = async (input: string) => {
    try {
      const r = await unified.execCli(input)
      showInfo(r.message.length > 200 ? `${r.message.slice(0, 200)}…` : r.message, r.success ? 4000 : 5000)
    } catch (e) {
      showInfo(`CLI 命令执行失败：${errText(e)}`, 5000)
    }
  }
  // 长消息展开状态（任务4：内容折叠；message.id → 是否展开）
  const [expandedMsgIds, setExpandedMsgIds] = createSignal<Record<string, boolean>>({})
  // 上下文占用（任务3：/compact 自动提示数据源，只读轮询 agentStatus，不写入任何 store）
  const [contextPct, setContextPct] = createSignal<number | null>(null)
  const [compactHintDismissed, setCompactHintDismissed] = createSignal(false)
  // 待发送附件（dialog 选择后暂存，随下一条消息发送）
  const [pendingAttachments, setPendingAttachments] = createSignal<NeoCodexAttachmentDto[]>([])

  // 批次6：@-mention 上下文预算 —— 插入 @path 时读取文件估算 token，chip 展示 + 总预算
  const [mentionRefs, setMentionRefs] = createSignal<{ path: string; lines: number; tokens: number }[]>([])

  // 斜杠命令：当前输入以 / 开头时激活菜单
  const [slashIdx, setSlashIdx] = createSignal(0)
  const slashQuery = () => {
    const v = inputValue()
    if (!v.startsWith('/')) return null
    const rest = v.slice(1)
    // 含空格视为普通文本（命令通常无空格参数）
    if (rest.includes(' ')) return null
    return rest
  }
  const slashFiltered = () => {
    const q = slashQuery()
    if (q === null) return []
    const lq = q.trim().toLowerCase()
    if (!lq) return SLASH_COMMANDS
    return SLASH_COMMANDS.filter(
      (c) => c.keywords.some((k) => k.includes(lq)) || c.label.includes(lq)
    )
  }
  // 菜单是否激活：输入以 / 开头且有匹配命令，且未被 Esc 关闭（Esc 仅关菜单、保留输入草稿）
  const [slashDismissed, setSlashDismissed] = createSignal(false)
  const slashActive = () =>
    slashQuery() !== null && slashFiltered().length > 0 && !slashDismissed()
  // /compact 命令：调用后端 neocodex.compactSession 真实压缩（keep 8 条）
  // 防并发：压缩进行中禁止再次触发；成功后经 getSessionMessages 消费路径（loadSessionMessages）重读截断后的消息
  const [compacting, setCompacting] = createSignal(false)
  const runCompact = async () => {
    if (compacting()) return
    // F9: 流式进行中禁压缩——后端 compact 阻塞在 agent 互斥锁上直至流结束，
    // 期间正流式的消息可能被 keep=8 裁掉，onDone 随后 no-op 造成三方不一致
    if (isGenerating()) return
    const sessionId = currentSession()?.id ?? ''
    if (!sessionId) {
      setStreamError('当前没有激活会话，无法压缩')
      setTimeout(() => setStreamError(null), 3000)
      return
    }
    setCompacting(true)
    try {
      await neocodex.compactSession(sessionId, 8)
      // 后端消息已截断，重拉当前会话消息（与 neocodex_get_session_messages 消费路径一致）
      await chatStore.loadSessionMessages(sessionId)
      // 压缩成功后关闭自动压缩提示，避免残留
      setCompactHintDismissed(true)
      showInfo('上下文已压缩，更早的对话被截断', 3000)
    } catch (error) {
      console.error('[Chat] Compact session failed:', error)
      const errorMsg = errText(error) || '压缩会话失败，请重试'
      setStreamError(errorMsg)
      setTimeout(() => setStreamError(null), 3000)
    } finally {
      setCompacting(false)
    }
  }
  // 斜杠命令执行上下文（依赖注入到独立逻辑模块 routes/chat/slashCommands）
  const slashCtx: SlashContext = {
    store: chatStore,
    currentSessionId: () => currentSession()?.id ?? '',
    clearInput: () => { setInputValue(''); adjustTextarea() },
    showInfo,
    showError: (msg: string) => {
      setStreamError(msg)
      setTimeout(() => setStreamError(null), 3000)
    },
  }
  const runSlash = (cmd: SlashCommandDef) => {
    // 「运行 Harness 任务」：从当前输入提取 /run 参数，先于清空执行（保留参数）
    if (cmd.id === 'run') {
      const { isRun, instruction } = parseRunCommand(inputValue())
      setInputValue('')
      adjustTextarea()
      if (isRun && instruction) void runHarness(instruction)
      else showInfo('用法：/run <要执行的任务>', 3000)
      return
    }
    setInputValue('')
    setSlashDismissed(false)
    adjustTextarea()
    if (cmd.id === 'clear') {
      chatStore.clearMessages()
      setMentionRefs([])
    } else if (cmd.id === 'compact') {
      runCompact()
    } else if (cmd.id === 'new') {
      chatStore.addSession()
    } else {
      runSlashDispatch(slashCtx, cmd)
    }
  }

  /* 批次1：Plan Mode 批准流 —— 批准 / 拒绝 / 继续规划。
   * 批准：切换到 accept_edits 并携带计划原文继续，让 agent 按批准的计划实际执行
   * （对标 Claude Code plan 批准进入执行；同轮延续而非新开会话）。
   */
  const approvePlan = async () => {
    const pending = planPending()
    if (!pending) return
    const planText = chatStore.messageContent(pending.msgId) ?? ''
    setPlanPending(null)
    const targetMode: PermissionMode = 'accept_edits'
    setPermissionMode(targetMode)
    showInfo(`计划已批准，切换至「${PERMISSION_MODES.find(m => m.value === targetMode)?.label}」执行`, 3000)
    // 携带计划继续（同轮延续），确保 agent 上下文包含被批准的规划明细
    const body = planText.trim()
      ? `已批准以下计划，请按计划执行：\n\n${planText}`
      : '已批准计划，请执行。'
    await sendMessage(body, { userMessageAdded: false })
    setApprovalAccepted(approvalAccepted() + 1)
  }

  const rejectPlan = () => {
    const pending = planPending()
    if (!pending) return
    setPlanPending(null)
    setApprovalRejected(approvalRejected() + 1)
    showInfo('计划已拒绝，可继续规划或补充需求', 3000)
  }

  const cancelPlan = () => {
    const pending = planPending()
    if (!pending) return
    setPlanPending(null)
    showInfo('计划已取消，保持在规划模式', 3000)
  }

  // 视图切换：chat / cowork / computer（对应侧栏 segmented tabs）
  const [activeView, setActiveView] = createSignal<'chat' | 'cowork' | 'computer'>('chat')

  // 标签筛选（对标 Obsidian Tag Pane 多选过滤）
  const [activeTags, setActiveTags] = createSignal<string[]>([])
  const toggleTag = (name: string) => {
    setActiveTags((prev) =>
      prev.includes(name) ? prev.filter((t) => t !== name) : [...prev, name]
    )
  }
  const clearTags = () => setActiveTags([])

  // 顶部工具栏面板：一次只开一个
  type PanelId = 'git' | 'tasks' | 'cost' | 'timeline' | 'sidechat' | 'preview'
  // 面板快捷键顺序（⌘1-⌘6）与侧栏入口一一对齐
  const PANEL_ORDER: PanelId[] = ['git', 'cost', 'tasks', 'timeline', 'sidechat', 'preview']
  const [activePanel, setActivePanel] = createSignal<PanelId | null>(null)
  const togglePanel = (id: PanelId) => {
    setActivePanel(activePanel() === id ? null : id)
  }

  // 用函数访问 store，保证 store 变更时响应式重渲染
  const messages = () => chatStore.currentMessages
  const isGenerating = () => chatStore.isGenerating
  const currentSession = () => chatStore.currentSession

  // 权限模式徽章信息：短标签 + 色点（对标 Claude 顶栏 mode 徽章）
  const permissionModeInfo = () => {
    const m = PERMISSION_MODES.find((x) => x.value === permissionMode()) ?? PERMISSION_MODES[0]
    return { ...m, shortLabel: MODE_SHORT_LABEL[m.value] }
  }
  // 权限模式循环切换（Shift+Tab / 徽章点击共用；生成中禁止改动，与 PermissionModeSelector disabled 一致）
  const cyclePermissionMode = () => {
    if (isGenerating()) return
    const idx = PERMISSION_MODES.findIndex((m) => m.value === permissionMode())
    const next = PERMISSION_MODES[(idx + 1) % PERMISSION_MODES.length]
    setPermissionMode(next.value)
    showInfo(`权限模式：${next.label}`, 2500)
  }

  /* 任务3：上下文占用只读轮询（与 CostDashboard 同源 agentStatus，只读不污染）
     经 api/query 共享 3s TTL 缓存，避免两个轮询各自发 IPC */
  usePolling({
    intervalMs: 15000,
    immediate: true,
    run: async () => {
      try {
        const s = await query<AgentStatus>('agent_status', () => neocodex.agentStatus(), { ttlMs: 3000 })
        if (s && typeof s.context_usage === 'number') {
          setContextPct(s.context_usage * 100)
        }
      } catch {
        /* 只读轮询，失败静默 */
      }
    },
  })
  // 上下文回落到阈值以下后，重新允许 /compact 提示再次出现
  createEffect(() => {
    const p = contextPct()
    if (p !== null && p < 80) setCompactHintDismissed(false)
  })
  // OS 活动透明度层：能力路由命中域同步到活动条（让「OS 在想什么域」实时可见）
  createEffect(() => {
    const r = harnessRoute()
    setAgentDomain(r?.domain ?? null)
  })

  // 统一命令桥懒加载：⌘K 面板注入 CLI 命令 (单一真源 unified_cli_list)
  createEffect(() => {
    if (!paletteOpen()) return
    void (async () => {
      try {
        const cli = await unified.cliList()
        setUnifiedCliCmds(
          cli.map((s) => ({
            id: `cli:${s.name}`,
            label: s.name,
            desc: s.description,
            keywords: [s.name.replace(/^\//, ''), ...s.aliases.map((a) => a.replace(/^\//, ''))],
            run: () => void execUnifiedCli(s.name),
          })),
        )
      } catch {
        setUnifiedCliCmds([])
      }
    })()
  })
  const compactHintVisible = () => {
    const p = contextPct()
    if (p === null || p < 80) return false
    return !compactHintDismissed()
  }

  // Event listener cleanup functions
  const [unlistenStream, setUnlistenStream] = createSignal<UnlistenFn | null>(null)
  const [unlistenMenu, setUnlistenMenu] = createSignal<UnlistenFn | null>(null)
  // 提供商切换事件 handler（普通变量，避免 signal setter 函数式更新歧义）
  let providerChangedHandler: (() => void) | null = null
  // 全局 Esc handler（面板/菜单/设置关闭层级统一在此）
  let globalKeydownHandler: ((e: KeyboardEvent) => void) | null = null

  // Current assistant message being streamed
  const [currentAssistantMsgId, setCurrentAssistantMsgId] = createSignal<string | null>(null)

  // 流式"代次"（epoch）：事件回调闭包捕获当前代次，仅当事件代次与当前代次一致才处理。
  // onStart 起流时推进一次代次；停止/切换会话/发送失败再推进，使陈旧事件的迟到 token/done/tool 全部被丢弃（防跨会话/跨轮污染）。
  let generation = 0
  let activeGen = 0

  let scrollRef: HTMLDivElement | undefined
  let prevSessionId: string | null = null

  // Auto-resize textarea
  const adjustTextarea = () => {
    const textarea = textareaRef()
    if (textarea) {
      textarea.style.height = 'auto'
      textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`
    }
  }

  // Set up event listeners for streaming
  onMount(async () => {
    // 启动时加载会话历史（原代码从未调用 loadSessions，侧边栏恒为空）
    chatStore.loadSessions()

    // 注册流式事件监听（经统一事件层；单个失败不阻断其余）
    const unlistenStream = await subscribeStream({
      onStart: () => {
        // 记录代次：每次后端真正起流开启一个新代次，token/done/tool 仅接受同代次事件
        activeGen = ++generation
        // OS 活动：进入思考阶段，重置工具计数
        setAgentPhase('thinking')
        setAgentToolCount(0)
        setAgentLastActivity(null)
        pushLog({ kind: 'phase', label: '开始思考' })
      },
      onToken: (delta) => {
        if (activeGen !== generation) return
        const msgId = currentAssistantMsgId()
        if (msgId) {
          chatStore.appendMessageContent(msgId, delta)
        }
        // OS 活动：首个 token 起由「思考」转入「生成」
        if (agentPhase() === 'thinking') {
          setAgentPhase('generating')
          pushLog({ kind: 'phase', label: '生成回复' })
        }
      },
      onEnd: (content) => {
        if (activeGen !== generation) return
        const msgId = currentAssistantMsgId()
        if (msgId) {
          chatStore.updateMessage(msgId, content, false)
        }
      },
      onDone: (payload) => {
        if (activeGen !== generation) return
        const msgId = currentAssistantMsgId()
        const wasCancelled = payload.cancelled

        if (msgId) {
          if (wasCancelled) {
            // Message already has partial content from tokens, just mark as not streaming
            chatStore.updateMessage(msgId, payload.content, false)
            setStreamError('生成已停止')
            setTimeout(() => setStreamError(null), 3000)
          } else {
            chatStore.updateMessage(msgId, payload.content, false)
          }
          // 批次1：plan 模式下非取消完成的回复 → 进入批准待定态（仅当该回复确为规划产出）。
          // 依据：Valmeekam et al. 指出单步计划生成弱、需执行期验证，故此处在执行前显式设置审阅关卡。
          if (!wasCancelled && permissionMode() === 'plan') {
            setPlanPending({ msgId })
          }
        }

        chatStore.setGenerating(false)
        setCurrentAssistantMsgId(null)
        // OS 活动：完成阶段，1.5s 后回落空闲（短暂可见成功态）
        setAgentPhase('done')
        pushLog({ kind: wasCancelled ? 'error' : 'done', label: wasCancelled ? '生成已停止' : '完成' })
        setTimeout(() => setAgentPhase('idle'), 1500)
        if (streamWatchdogTimer) { clearTimeout(streamWatchdogTimer); streamWatchdogTimer = undefined }
      },
      onTool: (payload) => {
        if (activeGen !== generation) return
        const msgId = currentAssistantMsgId()
        const domain = harnessRoute()?.domain
        if (msgId) {
          const toolCall: ToolCallRecord = {
            id: `tool-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
            name: payload.name,
            args: payload.args,
            result: payload.result,
            duration_ms: payload.duration_ms,
            success: payload.success,
            domain,
          }
          chatStore.appendToolCall(msgId, toolCall)
        }
        // OS 活动：工具调用阶段 + 计数 + 最近活动描述
        setAgentPhase('tooling')
        setAgentToolCount((c) => c + 1)
        setAgentLastActivity(`工具调用：${payload.name}${payload.success ? '' : '（失败）'}`)
        pushLog({
          kind: 'tool',
          label: payload.name,
          detail: payload.success ? '成功' : '失败',
          domain,
        })
      },
      onError: (payload) => {
        // F1: provider 阶段错误——保留已累积 partial，标记消息完成并提示
        if (activeGen !== generation) return
        const msgId = currentAssistantMsgId()
        if (msgId) {
          const text = payload.partial || `⚠️ ${payload.message}`
          chatStore.updateMessage(msgId, text, false)
        }
        chatStore.setGenerating(false)
        setCurrentAssistantMsgId(null)
        setStreamError(payload.message || '生成失败')
        setTimeout(() => setStreamError(null), 5000)
        // 结构化错误三段式（后端可选填充，缺失则按根因规则推导 WHY/NEXT）
        const rc = rootCause(payload.message || payload.what || '生成失败')
        setStreamErrorDetail({
          what: payload.what ?? payload.message ?? '生成失败',
          why: payload.why ?? rc.why,
          next: payload.next ?? rc.next,
        })
        setTimeout(() => setStreamErrorDetail(null), 6000)
        // OS 活动：错误阶段 + 最近活动描述
        setAgentPhase('error')
        setAgentLastActivity(payload.message || '生成失败')
        pushLog({ kind: 'error', label: payload.message || '生成失败' })
        // 作废旧代次：错误后迟到的 token/done 一律丢弃
        generation++
        if (streamWatchdogTimer) { clearTimeout(streamWatchdogTimer); streamWatchdogTimer = undefined }
      },
      onReasoning: (payload) => {
        // OS 推理流（后端可选 emit）：把意识核心推理步骤实时透出，对抗 black-box
        if (activeGen !== generation) return
        pushLog({ kind: 'reasoning', label: payload.text.slice(0, 80) })
      },
      onSubscribeError: (event) => {
        setStreamError(`流式事件 ${event} 订阅失败，回复可能不完整`)
        setTimeout(() => setStreamError(null), 5000)
      },
    })
    setUnlistenStream(() => unlistenStream)

    // 读取当前激活模型（仅用于状态栏展示，只读命令）
    try {
      const cfg = await neocodex.providerConfig()
      setActiveModel(cfg.active_model || null)
    } catch {
      /* 展示字段，静默失败 */
    }

    // 读取应用版本（底部状态条展示，避免硬编码漂移）
    try {
      setAppVersion(await neocodex.appVersion())
    } catch {
      /* 版本非关键 */
    }

    // 上下文占用只读轮询由 usePolling 管理（15s；同源 agentStatus 共享缓存）

    // 监听提供商切换事件（SettingsModal / ProviderSelector 广播），同步状态栏模型
    const onProviderChanged = () => {
      neocodex
        .providerConfig()
        .then((cfg) => setActiveModel(cfg.active_model || null))
        .catch(() => {})
    }
    window.addEventListener('neotrix:provider-changed', onProviderChanged)
    providerChangedHandler = onProviderChanged

    // 全局 Esc：按层级关闭最深层（命令面板 → 斜杠菜单 → 功能面板 → 设置弹窗）。
    // 面板 Esc 关闭不一致（此前仅 textarea 局部处理且部分面板自带 Esc）；onCleanup 移除。
    // ⌘K 全局唤起命令面板（不依赖输入区焦点，对标 Claude Code ⌘K 全局命令菜单）。
    const onGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        setPaletteOpen((open) => !open)
        return
      }
      if (e.key !== 'Escape') return
      if (paletteOpen()) {
        setPaletteOpen(false)
        return
      }
      if (slashActive()) {
        setSlashDismissed(true)
        return
      }
      if (activePanel()) {
        setActivePanel(null)
        return
      }
      if (settingsOpen()) {
        setSettingsOpen(false)
        return
      }
    }
    window.addEventListener('keydown', onGlobalKeyDown)
    globalKeydownHandler = onGlobalKeyDown

    // 桌面菜单事件桥（Rust 菜单 emit → 前端动作；此前零订阅，菜单项全部无效）
    // new-session → chatStore.addSession()；open-settings / palette → 打开对应 UI
    const unlistenMenu = await subscribeMenuEvents({
      onNewSession: () => {
        chatStore.addSession()
      },
      onOpenSettings: () => setSettingsOpen(true),
      onOpenPalette: () => setPaletteOpen(true),
      onCheckUpdates: () => setSettingsOpen(true),
    })
    setUnlistenMenu(() => unlistenMenu)
  })

  // Clean up event listeners
  onCleanup(() => {
    unlistenStream()?.()
    unlistenMenu()?.()
    if (providerChangedHandler) {
      window.removeEventListener('neotrix:provider-changed', providerChangedHandler)
    }
    if (globalKeydownHandler) {
      window.removeEventListener('keydown', globalKeydownHandler)
    }
    // F5: 流式进行中卸载（路由切换 / → /chat 或 /globe）时复位 store——
    // 否则重挂后 isGenerating 恒 true 锁死发送守卫，且流式消息红色光标永久残留
    if (isGenerating()) {
      chatStore.abortGeneration()
      const msgId = currentAssistantMsgId()
      if (msgId) chatStore.finishMessage(msgId)
    }
  })

  // 流式滚动锁：用户上滚则暂停自动跟随，显示「回到底部」
  const [stickToBottom, setStickToBottom] = createSignal(true)
  // 渐进授权可视层：bot 计划审批计数（对标 2026 自治度可见性）
  const [approvalAccepted, setApprovalAccepted] = createSignal(0)
  const [approvalRejected, setApprovalRejected] = createSignal(0)
  const autonomyLevel = () => {
    const total = approvalAccepted() + approvalRejected()
    if (total === 0) return '待校准' as const
    const rate = approvalAccepted() / total
    if (rate >= 0.8) return '高信任' as const
    if (rate >= 0.5) return '协作' as const
    return '审慎' as const
  }
  const autonomyRate = () => {
    const total = approvalAccepted() + approvalRejected()
    return total === 0 ? 0 : approvalAccepted() / total
  }
  const autonomyLevelNum = () => {
    const m = permissionMode()
    if (m === 'manual') return 0
    if (m === 'plan') return 1
    if (m === 'auto') return 2
    if (m === 'accept_edits') return 3
    return 1
  }
  const onScrollMsg = () => {
    const el = scrollRef
    if (!el) return
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80
    setStickToBottom(nearBottom)
  }
  const scrollToBottom = () => {
    const el = scrollRef
    if (!el) return
    el.scrollTop = el.scrollHeight
    setStickToBottom(true)
  }

  // 消息区自动滚动：新消息/会话切换强制到底，流式期间若在底部则跟随
  createEffect(() => {
    const sid = currentSession()?.id ?? null
    const sessionChanged = sid !== prevSessionId
    // 会话切换：作废旧代次并清空组件级流式游标（防止旧会话迟到事件污染新会话/新轮）
    if (sessionChanged && prevSessionId !== null) {
      generation++
      setCurrentAssistantMsgId(null)
    }
    prevSessionId = sid
    messages()
    isGenerating()
    const el = scrollRef
    if (!el) return
    if (sessionChanged || stickToBottom()) {
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight
      })
    }
  })

  // 首次发送后焦点丢失修复：空态 ↔ 消息流切换（messages 长度 0 ↔ 非0）时，
  // 旧 textarea 卸载、新 textarea 挂载后重新 focus，保证键盘连续输入
  let prevMsgLen = 0
  createEffect(() => {
    const len = messages().length
    const crossed = (prevMsgLen === 0) !== (len === 0)
    prevMsgLen = len
    if (crossed) {
      requestAnimationFrame(() => {
        textareaRef()?.focus()
      })
    }
  })

  const handleInput = (e: Event) => {
    const target = e.target as HTMLTextAreaElement
    setInputValue(target.value)
    // 用户继续输入时恢复 slash 菜单（Esc 关闭仅是临时收起）
    setSlashDismissed(false)
    adjustTextarea()
  }

  /* @ 文件引用：光标处 @ 触发文件选择，插入 @路径 标记（对标 Claude @ 提及）。
   * 批次6：选择后读取文件内容估算 token/行数，记录为 mention chip（上下文预算可视化）。 */
  const handleAtMention = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({ multiple: false, directory: false })
      if (!selected) return
      const path = Array.isArray(selected) ? selected[0] : selected
      const textarea = textareaRef()
      const cur = inputValue()
      const pos = textarea?.selectionStart ?? cur.length
      const before = cur.slice(0, pos)
      const after = cur.slice(pos)
      const stripped = before.replace(/@\s*$/, '')
      const next = `${stripped}@${path} ${after}`.trimStart()
      setInputValue(next)
      requestAnimationFrame(() => {
        textarea?.focus()
        const caret = stripped.length + path.length + 2
        textarea?.setSelectionRange(caret, caret)
      })
      adjustTextarea()
      // 预算记录（只读失败不影响插入，仅少一个 chip）
      try {
        const content = await system.readFile(path)
        const lines = content.split('\n').length
        setMentionRefs(prev => {
          const existing = prev.find(r => r.path === path)
          if (existing) return prev.map(r => r.path === path ? { ...r, lines, tokens: estimateTokens(content) } : r)
          return [...prev, { path, lines, tokens: estimateTokens(content) }]
        })
      } catch {
        /* 读取失败静默：@ 引用仍插入，预算未知 */
      }
    } catch (e) {
      console.error('[Chat] @mention failed:', e)
    }
  }

  /* 移除 mention chip（@ 引用仍在输入文本中，仅解除预算显示；/clear 时清空） */
  const removeMentionRef = (path: string) => {
    setMentionRefs(prev => prev.filter(r => r.path !== path))
  }

  const handleKeyDown = (e: KeyboardEvent) => {
    // @ 文件引用：输入 @ 触发文件选择
    if (e.key === '@' && !e.shiftKey && !e.metaKey && !e.ctrlKey) {
      const textarea = textareaRef()
      const cur = inputValue()
      const pos = textarea?.selectionStart ?? cur.length
      // 仅当 @ 位于行首或空格后（非邮箱/路径中途）
      const charBefore = cur[pos - 1]
      if (pos === 0 || charBefore === ' ' || charBefore === '\n') {
        e.preventDefault()
        handleAtMention()
        return
      }
    }
    // 斜杠命令导航：菜单激活时优先处理方向键 / Enter（Esc 统一走全局 onGlobalKeyDown）
    const slashList = slashFiltered()
    if (slashActive()) {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault()
        const dir = e.key === 'ArrowDown' ? 1 : -1
        setSlashIdx((prev) => (prev + dir + slashList.length) % slashList.length)
        return
      }
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        runSlash(slashList[Math.min(slashIdx(), slashList.length - 1)])
        return
      }
    } else {
      setSlashIdx(0)
    }
    // Shift+Tab：权限模式循环切换（对标 Claude Code，按 PERMISSION_MODES 顺序循环）。
    // 仅输入区有焦点时劫持；Slash 菜单激活时不劫持（放行默认 Tab 焦点导航）；生成中禁止改动。
    if (e.key === 'Tab' && e.shiftKey && !slashActive()) {
      if (isGenerating()) return
      const textarea = textareaRef()
      if (textarea && document.activeElement === textarea) {
        e.preventDefault()
        cyclePermissionMode()
      }
    }
    // 面板快捷键：⌘1-⌘6 切换 6 个功能面板（顺序对齐侧栏），⌘7 切换电脑控制视图；
    // 面板仅 chat 视图可渲染，非 chat 视图按下自动先切回 chat
    if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '7') {
      const idx = Number(e.key) - 1
      if (idx === 6) {
        // ⌘7：电脑控制 → 侧栏内嵌视图
        e.preventDefault()
        if (activeView() === 'chat') setActivePanel(null)
        setActiveView(activeView() === 'computer' ? 'chat' : 'computer')
        return
      }
      const target: PanelId = PANEL_ORDER[idx]
      if (target) {
        e.preventDefault()
        // 非 chat 视图点按面板：先回 chat 再开面板（面板区被 activeView 门禁，避免无响应）
        if (activeView() !== 'chat') setActiveView('chat')
        togglePanel(target)
      }
      return
    }
    if (e.key === 'n' && e.metaKey && !e.shiftKey) {
      // ⌘N：新建对话（对标 Claude Code / Osaurus）
      e.preventDefault()
      chatStore.addSession()
      return
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  // 核心发送：addMessage(user) → addMessage(assistant placeholder) → invoke
  // opts.userMessageAdded = true 表示用户消息已由编辑/重生成逻辑写入，跳过重复添加
  // opts.regenerate = true 表示 re-发被截断后保留的用户消息（wire 已含该轮，
  //   后端跳过重复 record/context.push，修复 wire 双写）；编辑路径保持 false
  //   （被编辑的用户消息已随 truncate 移除，需重新落盘）
  const sendMessage = async (content: string, opts?: { userMessageAdded?: boolean; regenerate?: boolean }) => {
    if (!content || isGenerating()) return

    if (!currentSession()) {
      await chatStore.addSession()
    }

    // 新消息开始即清除旧批准条（含批准后同轮延续的二次发送，避免残留）
    setPlanPending(null)

    let userMsgId: string | null = null
    if (!opts?.userMessageAdded) {
      userMsgId = chatStore.addMessage({ role: 'user', content })
    }
    const atts = pendingAttachments()
    setInputValue('')
    setAnnotationHint(null)
    setPendingAttachments([])
    setMentionRefs([])
    adjustTextarea()
    setStreamError(null)

    // Create assistant message placeholder
    const assistantMsgId = chatStore.addMessage({
      role: 'assistant',
      content: '',
      isStreaming: true,
    })
    setCurrentAssistantMsgId(assistantMsgId)
    chatStore.setGenerating(true)

    // F10: 启动看门狗。上限 600s（超长任务仍保活）；onDone/stop/失败路径清除
    if (streamWatchdogTimer) clearTimeout(streamWatchdogTimer)
    streamWatchdogTimer = setTimeout(() => {
      if (!isGenerating()) return
      console.error('[Chat] Stream watchdog fired: no done event, force resetting')
      setStreamError('回复超时，请重试')
      setTimeout(() => setStreamError(null), 3000)
      const msgId = currentAssistantMsgId()
      if (msgId) chatStore.finishMessage(msgId)
      chatStore.setGenerating(false)
      setCurrentAssistantMsgId(null)
      generation++
    }, 600_000)

    try {
      // 流式生成经统一 IPC 层；实际 token 由 neocodex_stream_* 事件推送
      await neocodex.sendMessageStream({
        content,
        attachments: atts.length > 0 ? atts : undefined,
        regenerate: opts?.regenerate ?? false,
        permission_mode: permissionMode(),
        temperature: 0.7,
        max_tokens: 4096,
      })
      // The actual streaming happens via events (neocodex_stream_token, etc.)
    } catch (error) {
      console.error('[Chat] Send message failed:', error)
      const errorMsg = errText(error) || '发送失败，请重试'
      setStreamError(errorMsg)
      // 3s 自动清除（对标 runCompact 自清模式，避免错误 toast 永不消失）
      setTimeout(() => setStreamError(null), 3000)

      // F8: 失败时移除本地占位消息（user + error 两者 wire 中均不存在）——
      // 残留会让后续 regenerate/edit 的可见索引与 wire 错位，截断到错误位置。
      // 错误反馈由上方 streamError toast（3s 自动清）承担，不需持久化占位。
      if (userMsgId) chatStore.deleteMessage(userMsgId)
      if (assistantMsgId) chatStore.deleteMessage(assistantMsgId)
      chatStore.setGenerating(false)
      // 作废旧代次：丢弃本次失败发送可能触发的迟到事件
      generation++
      setCurrentAssistantMsgId(null)
      if (streamWatchdogTimer) { clearTimeout(streamWatchdogTimer); streamWatchdogTimer = undefined }
    }
  }

  const handleSend = async () => {
    let content = inputValue().trim()
    if (!content && !annotationHint() && pendingAttachments().length === 0) return
    // 对话即OS「运行」入口：/run <任务> 显式触发重路径真实执行闭环（对标 Claude Code /命令带参）
    const runCmd = parseRunCommand(content)
    if (runCmd.isRun) {
      setInputValue('')
      adjustTextarea()
      if (runCmd.instruction) await runHarness(runCmd.instruction)
      else showInfo('用法：/run <要执行的任务>', 3000)
      return
    }
    if (!content) content = annotationHint()! // send the annotation even with empty text
    if (annotationHint() && content !== annotationHint()!) {
      content = `${content}\n\n${annotationHint()}`
    }
    // 对话即OS：非阻塞解析能力路由，透明展示命中域（不替换 LLM 主链路）
    setHarnessRoute(null)
    void harness
      .harnessExecute({ instruction: content })
      .then((r) => {
        if (r.capability_tag && r.capability_tag !== 'orchestration') {
          setHarnessRoute({ tag: r.capability_tag, domain: r.domain, specialist: r.specialist })
        }
      })
      .catch(() => {})
    await sendMessage(content)
  }

  /* Harness 重路径真实执行（⌘K「运行 Harness 任务」/ /run 触发）— 完整闭环，结果落报告面板
     真·流式: 订阅 harness-progress 事件, 阶段1(allocated) 先渲染分配视图, 阶段2(done) 落全量报告 */
  const runHarness = async (instruction: string) => {
    const text = instruction.trim()
    if (!text) {
      showInfo('先在输入框写下要执行的任务', 3000)
      return
    }
    const runId = (globalThis.crypto?.randomUUID?.() ?? `run-${Date.now()}`)
    setHarnessRunning(true)
    setHarnessReport(null)
    setHarnessSteps([])
    let unlisten: (() => void) | undefined
    try {
      unlisten = await listen<HarnessProgressEvent>('harness-progress', (e) => {
        const p = e.payload
        if (p.run_id && p.run_id !== runId) return
        if (p.phase === 'allocated') {
          // 阶段1: 拆解+分配已就绪 → 即时渲染 (执行中)
          setHarnessReport(p.report ?? null)
        } else if (p.phase === 'step' && p.step) {
          // 阶段2: 子任务级实时进度 → 追加/覆写该 index 的步骤
          const s = p.step
          setHarnessSteps((prev) => {
            const next = prev.slice()
            next[s.index] = s
            return next
          })
        } else if (p.phase === 'done') {
          // 阶段3: 完整闭环报告 → 落盘并停止脉冲
          setHarnessReport(p.report ?? null)
          setHarnessRunning(false)
          unlisten?.()
        }
      })
      const r = await harness.harnessRun({ instruction: text, run_id: runId })
      // 兜底: 若事件未到达 (非 tauri 环境等), 以返回值填充
      setHarnessReport(r)
      const solvedInternal = r.internal_results.filter((x) => x.executed).length
      const solvedExternal = r.external_closures.filter((x) => x.solved).length
      showInfo(`Harness 完成 · 拆解 ${r.allocations.length} · 内置 ${solvedInternal}/${r.internal_count} · 外部 ${solvedExternal}/${r.external_gap_count}`, 3000)
    } catch (e) {
      showError(`Harness 执行失败：${errText(e)}`)
    } finally {
      setHarnessRunning(false)
      unlisten?.()
    }
  }

  /* 附件选择：dialog 选文件 → read_file 读取 → 暂存待发送 */
  const handlePickAttachment = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({ multiple: true })
      if (!selected) return
      const paths = Array.isArray(selected) ? selected : [selected]
      const atts: NeoCodexAttachmentDto[] = []
      for (const p of paths) {
        try {
          const content = await system.readFile(p)
          const name = p.split('/').pop() || p
          const mime = guessMime(name)
          atts.push({ name, size: content.length, mime_type: mime, data: content })
        } catch (e) {
          console.error('[Chat] Read attachment failed:', p, e)
        }
      }
      if (atts.length > 0) {
        setPendingAttachments([...pendingAttachments(), ...atts])
      }
    } catch (e) {
      console.error('[Chat] Open dialog failed:', e)
    }
  }

  const removeAttachment = (idx: number) => {
    setPendingAttachments(pendingAttachments().filter((_, i) => i !== idx))
  }

  /* 粘贴图片：剪贴板内若有图片文件 → 转 base64 → 暂存为附件（对标 Claude 图片粘贴） */
  const handlePasteImage = (e: ClipboardEvent) => {
    const items = e.clipboardData?.items
    if (!items) return
    for (const item of Array.from(items)) {
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile()
        if (!file) continue
        e.preventDefault()
        const reader = new FileReader()
        reader.onload = () => {
          const data = typeof reader.result === 'string' ? reader.result.split(',')[1] ?? '' : ''
          if (!data) return
          const att: NeoCodexAttachmentDto = {
            name: file.name || `pasted-image-${Date.now()}.png`,
            size: file.size,
            mime_type: item.type,
            data,
          }
          setPendingAttachments([...pendingAttachments(), att])
        }
        reader.readAsDataURL(file)
        break
      }
    }
  }

  const handleStop = async () => {
    // 立即作废旧代次：停止后迟到的事件（token/done/tool）一律丢弃，防止污染下一轮
    generation++
    const msgId = currentAssistantMsgId()
    try {
      await neocodex.stopStream()
    } catch (error) {
      console.error('[Chat] Stop stream failed:', error)
    }
    chatStore.abortGeneration()
    // 复位该消息的流式标记（onDone 已被代次失效丢弃，须本地兜底复位，否则消息永久卡流式态）
    if (msgId) {
      chatStore.finishMessage(msgId)
    }
    setCurrentAssistantMsgId(null)
    if (streamWatchdogTimer) { clearTimeout(streamWatchdogTimer); streamWatchdogTimer = undefined }
  }

  const handleRegenerate = (message: Message) => {
    // 流式进行中禁止 regenerate：本地截断 + 重发若被 isGenerating 守卫吞掉，
    // 会出现"会话已截断但无新回复"的三方不一致（审计 F2）
    if (isGenerating()) return
    // 持久化对齐：先计算可见索引再截断——regenerateFrom 会先移除该 assistant 消息，
    // 若在其后读 currentMessages 已找不到 message.id。可见索引 = 前端数组中
    // user/assistant 消息计数（tool/system 不计数，与后端 visible_message_indices 对齐）。
    const sid = chatStore.state.currentSessionId
    let visibleIdx = -1
    if (sid) {
      const msgs = chatStore.currentMessages
      const idx = msgs.findIndex(m => m.id === message.id)
      if (idx >= 0) {
        visibleIdx = msgs
          .slice(0, idx + 1)
          .filter(m => m.role === 'user' || m.role === 'assistant').length - 1
      }
    }
    const userContent = chatStore.regenerateFrom(message.id)
    if (userContent) {
      // 🟡 修复：regenerateFrom 仅截断本地 store——wire 中旧回复仍在，重载会话后
      // 复活且 agent 上下文未重建。此处同步调后端 neocodex_regenerate 截断 wire
      // 并重建上下文（R-P79：功能接线到生产路径，不留死代码）。
      if (sid && visibleIdx >= 0) {
        neocodex.regenerate(sid, visibleIdx).catch((e: Error) => {
          console.error('[Chat] 持久化重新生成失败（本地已截断，重载后可能回退）:', e)
          setStreamError(errText(e) || '重新生成失败')
          setTimeout(() => setStreamError(null), 3000)
        })
      }
      // regenerateFrom 已截断被点消息所在轮（及之后），用户消息保留，跳过重复添加
      // regenerate: true — wire 已含该用户轮（neocodex_regenerate 的 truncate 保留之），
      // 后端跳过重复 record/context.push，修复双写（审计 F3）
      sendMessage(userContent, { userMessageAdded: true, regenerate: true })
    }
  }

  const handleEditMessage = (message: Message) => {
    setEditingMessageId(message.id)
    setEditContent(message.content)
  }

  const handleSaveEdit = () => {
    // 流式进行中禁止编辑重发（审计 F2：本地截断 + 重发被守卫吞 → 会话截断无响应）
    if (isGenerating()) return
    const content = editContent().trim()
    const msgId = editingMessageId()
    if (msgId && content) {
      // 🟡 修复：与 handleRegenerate 同款持久化对齐——editAndResend 仅截断本地 store，
      // wire 中旧消息仍在（重载复活）。先算被编辑消息的可见索引，同步截断后端 wire。
      const sid = chatStore.state.currentSessionId
      let editVisibleIdx = -1
      if (sid) {
        const msgs = chatStore.currentMessages
        const idx = msgs.findIndex(m => m.id === msgId)
        if (idx >= 0) {
          // 截断点 = 被编辑消息自身（本地 slice(0, msgIndex) 将其一并移除）
          editVisibleIdx = msgs
            .slice(0, idx)
            .filter(m => m.role === 'user' || m.role === 'assistant').length
        }
      }
      // editAndResend 已截断并添加新用户消息，跳过重复添加
      chatStore.editAndResend(msgId, content)
      setEditingMessageId(null)
      setEditContent('')
      if (sid && editVisibleIdx >= 0) {
        neocodex.regenerate(sid, editVisibleIdx).catch((e: Error) => {
          console.error('[Chat] 持久化编辑失败（本地已截断，重载后可能回退）:', e)
        })
      }
      sendMessage(content, { userMessageAdded: true })
    }
  }

  const handleCancelEdit = () => {
    setEditingMessageId(null)
    setEditContent('')
  }

  const handleCopy = async (content: string, id: string) => {
    try {
      await navigator.clipboard.writeText(content)
      setCopiedId(id)
      setTimeout(() => setCopiedId(null), 1500)
    } catch {
      /* ignore */
    }
  }

  const formatTime = (date: Date) => {
    return new Date(date).toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  // ⌘K 命令面板动作（复用既有 handler，单一事实源）
  const paletteCommands: PaletteCommand[] = [
    { id: 'new', label: '新建对话', desc: '开启一段新对话', keywords: ['new', '新建', '对话'], run: () => chatStore.addSession() },
    { id: 'clear', label: '清除会话', desc: '清空当前会话全部消息', keywords: ['clear', '清除', '清空'], run: () => { chatStore.clearMessages(); setMentionRefs([]) } },
    { id: 'compact', label: '压缩会话', desc: '精简上下文继续对话', keywords: ['compact', '压缩'], run: () => runCompact() },
    { id: 'view-chat', label: '切换到对话视图', desc: '聊天主界面', keywords: ['chat', '对话', '视图'], run: () => setActiveView('chat') },
    { id: 'view-cowork', label: '切换到协同视图', desc: '协同会话与任务看板', keywords: ['cowork', '协同'], run: () => setActiveView('cowork') },
    { id: 'view-computer', label: '切换到电脑视图', desc: '屏幕操作与窗口管理', keywords: ['computer', '电脑'], run: () => setActiveView('computer') },
    { id: 'mode', label: '切换权限模式', desc: '自动 / 手动 / 接受编辑 / 规划', keywords: ['mode', '权限', '模式'], run: () => cyclePermissionMode() },
    { id: 'help', label: '快捷键帮助', desc: '显示常用快捷键说明', keywords: ['help', '帮助', '快捷键'], run: () => runSlash(SLASH_COMMANDS[3]) },
    { id: 'settings', label: '打开设置', desc: '提供商配置与应用设置', keywords: ['settings', '设置', '配置'], run: () => setSettingsOpen(true) },
    // Harness 融合入口：重路径真实执行（用当前输入框内容触发完整闭环）
    { id: 'harness-run', label: '运行 Harness 任务', desc: '用输入框内容触发完整执行闭环（内置+外部试错）', keywords: ['harness', '运行', '执行', 'run', '能力'], run: () => runHarness(inputValue()) },
    // 能力地图发现（Raycast 风格）：选中后把能力提示填入输入框，用户补充具体内容后发送
    ...harnessCaps().map((c) => ({
      id: `harness-${c.capability_tag}`,
      label: `能力：${c.description}`,
      desc: `${c.domain} · ${c.specialist}`,
      keywords: [c.capability_tag, ...c.keywords],
      run: () => {
        setInputValue(`/run ${c.description} `)
        adjustTextarea()
        textareaRef()?.focus()
      },
    })),
    ...unifiedCliCmds(),
  ]

  // 空状态引导：首屏建议提示（点击填入输入框，用户可增删后发送）
  const SUGGESTED_PROMPTS: string[] = [
    '帮我规划一个新功能的实现方案',
    '审查当前会话的代码改动',
    '解释这段报错日志的根因',
    '搜索并总结相关开源仓库',
    '压缩会话上下文继续对话',
    '运行 Harness 任务：抓取并吸收一个仓库',
  ]

  return (
    <div class="flex h-screen bg-transparent overflow-hidden">
        <Sidebar
          collapsed={sidebarCollapsed()}
          onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed())}
          onOpenSettings={() => setSettingsOpen(true)}
          activeView={activeView()}
          onSwitchView={setActiveView}
          activePanel={activePanel()}
          onTogglePanel={(id) => togglePanel(id as PanelId)}
          activeTags={activeTags()}
          onToggleTag={toggleTag}
          onClearTags={clearTags}
          width={sidebarWidth()}
        />
        {/* 左栏拖拽手柄（仅在展开态） */}
        <Show when={!sidebarCollapsed()}>
          <div
            class="sb-resize"
            role="separator"
            aria-orientation="vertical"
            aria-label="拖拽调整侧边栏宽度"
            onMouseDown={onSidebarResizeDown}
          />
        </Show>

      <main class="flex-1 flex flex-col min-w-0 overflow-hidden glass-L1 relative">
        {/* ===== 头部 ch-top：极简顶栏（对标 Claude Code 桌面，仅作窗口拖拽区） ===== */}
        <Show when={activeView() === 'chat'}>
          <header class="ch-top" data-tauri-drag-region>
            <div class="flex items-center gap-2 flex-shrink-0 min-w-0" data-tauri-drag-region>
              <AgentActivityBar
                phase={agentPhase}
                activeDomain={agentDomain}
                toolCount={agentToolCount}
                lastActivity={agentLastActivity}
              />
              {/* 渐进授权可视层：自治等级 + 审批通过率（对标 2026 自治度可见性） */}
              <AutonomyMeter
                level={autonomyLevelNum}
                mode={permissionMode}
                rate={() => Math.round(autonomyRate() * 100)}
              />
              {/* 活动日志审计层：展开查看 OS 完整活动时间线 */}
              <span class="relative flex-shrink-0">
                <button
                  class="agent-log-toggle"
                  onClick={() => setLogOpen((o) => !o)}
                  title="活动日志（审计层）"
                  aria-expanded={logOpen()}
                >
                  活动 ⌄
                </button>
                <Show when={logOpen()}>
                  <div class="agent-log-pop">
                    <div class="agent-log-pop__head">
                      <span>活动日志</span>
                      <button class="agent-log-pop__close" onClick={() => setLogOpen(false)} aria-label="收起活动日志">×</button>
                    </div>
                    <AgentActivityLog steps={agentLog} />
                  </div>
                </Show>
              </span>
            </div>
            <Show when={harnessRoute()}>
              <span
                class="ml-auto flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium bg-nt-io-500/10 text-nt-io-600 border border-nt-io-500/20 flex-shrink-0"
                title={`已路由到 ${harnessRoute()!.domain} · ${harnessRoute()!.specialist}`}
              >
                <span class="w-1.5 h-1.5 rounded-full bg-nt-io-500" />
                {harnessRoute()!.domain} · {harnessRoute()!.specialist}
              </span>
            </Show>
          </header>
          {/* 批次2：上下文占用 gauge 条（可交互，只读轮询数据源；>80% 自动亮起 /compact 一键） */}
          <Show when={contextPct() !== null}>
            <div class="ch-ctx" role="status" aria-label="上下文占用">
              <div
                class={clsx('ch-ctx-track', (contextPct() ?? 0) >= 80 && 'ch-ctx-track-danger')}
                title={`上下文占用 ${Math.round(contextPct() ?? 0)}%`}
              >
                <div
                  class={clsx('ch-ctx-fill', (contextPct() ?? 0) >= 80 && 'ch-ctx-fill-danger')}
                  style={{ width: `${Math.min(contextPct() ?? 0, 100)}%` }}
                />
              </div>
              <span class={clsx('ch-ctx-label', (contextPct() ?? 0) >= 80 && 'text-red-600')}>
                {Math.round(contextPct() ?? 0)}%
              </span>
              <Show when={(contextPct() ?? 0) >= 80 && !compacting()}>
                <button
                  class="ch-ctx-compact"
                  onClick={runCompact}
                  aria-label="压缩会话"
                  title="上下文即将用尽，点击压缩"
                >
                  <AlertTriangle class="w-3 h-3" />
                  压缩
                </button>
              </Show>
            </div>
          </Show>
        </Show>

        {/* ===== 顶部工具栏面板（一次一个，右侧滑出 + 遮罩点击关闭） ===== */}
        <Show when={activeView() === 'chat' && activePanel()}>
          <div
            class="fixed inset-0 z-30 bg-black/10 animate-fade-in"
            onClick={() => setActivePanel(null)}
            aria-hidden="true"
          />
          <Show when={activePanel() === 'git'}>
            <GitPanel open onClose={() => setActivePanel(null)} />
          </Show>
          <Show when={activePanel() === 'tasks'}>
            <ScheduledTasks open onClose={() => setActivePanel(null)} />
          </Show>
          <Show when={activePanel() === 'cost'}>
            <CostDashboard open onClose={() => setActivePanel(null)} />
          </Show>
          <Show when={activePanel() === 'timeline'}>
            <CheckpointTimeline
              open
              sessionId={currentSession()?.id ?? null}
              onClose={() => setActivePanel(null)}
              onRestored={async () => {
                const sid = currentSession()?.id ?? null
                if (sid) await chatStore.loadSessionMessages(sid)
                await chatStore.loadSessions()
              }}
            />
          </Show>
          <Show when={activePanel() === 'sidechat'}>
            <SideChat
              open
              sessionId={currentSession()?.id ?? null}
              onClose={() => setActivePanel(null)}
            />
          </Show>
          <Show when={activePanel() === 'preview'}>
            <LivePreview open onClose={() => setActivePanel(null)} />
          </Show>
        </Show>

        {/* ===== 消息流：气泡式 msg.r / msg.l（chat 视图） ===== */}
        <Show when={activeView() === 'chat'}>
        <div ref={scrollRef} class="flex-1 overflow-y-auto" role="log" aria-live="polite" onScroll={onScrollMsg}>
          <Show
            when={messages().length > 0}
            fallback={
              /* ===== 空状态：hero + cic 输入区（设计 v2） ===== */
              <div class="wc-inner h-full max-w-[800px] w-full mx-auto flex flex-col items-center justify-center gap-6 px-6 py-10 select-none">
                <div class="hero">
                  <div class="hero-svg">
                    <HeroMark />
                  </div>
                  <div>
                    <h1>{greeting()}</h1>
                    <p class="hero-sub">我是 NeoTrix，你的 AI 原生开发伙伴</p>
                  </div>
                </div>

                {/* 空状态引导：建议提示 chips（对标 2026 agent UX 首屏 onboarding） */}
                <div class="flex flex-wrap justify-center gap-2 max-w-[640px]">
                  <For each={SUGGESTED_PROMPTS}>
                    {(p) => (
                      <button
                        class="px-3 py-1.5 rounded-full text-[12px] text-text-secondary bg-white/60 border border-border-primary/40 hover:border-nt-io-500/40 hover:text-nt-io-700 hover:bg-white/80 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                        onClick={() => {
                          setInputValue(p)
                          adjustTextarea()
                          textareaRef()?.focus()
                        }}
                        title={p}
                      >
                        {p}
                      </button>
                    )}
                  </For>
                </div>

                {/* Harness 执行报告面板（/run 或 ⌘K 运行后展示，不污染会话历史） */}
                <Show when={harnessRunning() || harnessReport()}>
                  <div class="px-2 pb-2">
                    <HarnessReportCard running={harnessRunning()} report={harnessReport()} steps={harnessSteps()} onClose={() => setHarnessReport(null)} />
                  </div>
                </Show>

                {/* 左栏 bot 审批流可见性（挂载即拉取待审批队列，可 approve/reject） */}
                <Show when={approvals().length > 0}>
                  <div class="px-2 pb-2">
                    <ApprovalPanel approvals={approvals()} onClose={() => setApprovals([])} onResolved={refreshApprovals} />
                  </div>
                </Show>

                {/* 左栏 bot 文件内联编辑（read_file + write_file 闭环） */}
                <div class="px-2 pb-2">
                  <button
                    class="w-full px-2 py-1 rounded bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 text-11px"
                    onClick={() => setShowFileEditor(!showFileEditor())}
                  >
                    {showFileEditor() ? '收起文件编辑' : '文件内联编辑'}
                  </button>
                  <Show when={showFileEditor()}>
                    <FileEditorPanel onClose={() => setShowFileEditor(false)} />
                  </Show>
                </div>

                {/* cic 输入区 */}
                <div class="cic w-full">
                  {/* 斜杠命令菜单：空态同样渲染，保证 / 命令在任何输入态都有可见菜单 */}
                  <Show when={slashActive()}>
                    <div class="relative">
                      <SlashMenu
                        query={slashQuery() ?? ''}
                        commands={slashFiltered()}
                        selectedIdx={Math.min(slashIdx(), slashFiltered().length - 1)}
                        onSelect={runSlash}
                        onHover={(idx) => setSlashIdx(idx)}
                      />
                    </div>
                  </Show>
                  <textarea
                    id="chatInput"
                    rows={2}
                    placeholder="输入消息… (Enter 发送, Shift+Enter 换行)"
                    value={inputValue()}
                    onInput={handleInput}
                    onKeyDown={handleKeyDown}
                    onPaste={handlePasteImage}
                    ref={setTextareaRef}
                    aria-label="对话输入框"
                    aria-busy={isGenerating()}
                    aria-disabled={isGenerating()}
                  />
                  <div class="cic-actions">
                    <div class="cic-left">
                      <button class="cic-attach" onClick={handlePickAttachment} aria-label="附加文件" title="附加文件">
                        <svg viewBox="0 0 16 16">
                          <line x1="8" y1="3" x2="8" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                          <line x1="4" y1="8" x2="12" y2="8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                        </svg>
                      </button>
                      <ModelSwitcher disabled={isGenerating()} />
                    </div>
                    <div class="cic-right">
                      <Show when={inputValue().trim() || pendingAttachments().length > 0}>
                        <span class="text-10px text-text-muted/70 font-mono mr-2">
                          ≈{estimateTokens(inputValue())} tok
                          <Show when={pendingAttachments().length > 0}> · {pendingAttachments().length} 附件</Show>
                        </span>
                      </Show>
                      <button
                        class="vc-btn vc-send"
                        disabled={!inputValue().trim() && pendingAttachments().length === 0 && !annotationHint() && !isGenerating()}
                        onClick={isGenerating() ? handleStop : handleSend}
                        aria-label={isGenerating() ? '停止生成' : '发送消息'}
                        title={isGenerating() ? '停止生成' : '发送消息'}
                      >
                        {isGenerating() ? <Square class="w-4 h-4" /> : <NeoSend class="w-4 h-4" />}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            }
          >
            <div class="cs max-w-[800px] mx-auto">
              <For each={messages()}>
                {(message: Message, i) => {
                  const isUser = message.role === 'user'
                  const isEditing = editingMessageId() === message.id
                  const isTool = message.role === 'tool'
                  // 长消息折叠（任务4）：仅超长 assistant 消息，且非流式、非末条（末条全量在 DOM，流式安全）
                  const isLast = i() === messages().length - 1
                  const isLong = message.role === 'assistant' && message.content.length > LONG_MSG_FOLD_CHARS
                  const expanded = !!expandedMsgIds()[message.id]
                  const collapsible = isLong && !message.isStreaming && !isLast
                  const collapsed = collapsible && !expanded
                  return (
                    <div class={clsx('group msg', isUser ? 'r' : 'l')}>
                      {/* 头像 */}
                      <div class="ma2">
                        {isUser ? <UserIcon /> : <BotIcon />}
                      </div>

                      <div class="flex-1 min-w-0">
                        {isEditing ? (
                          <div class="p-2 rounded-xl bg-white/40 border border-nt-io-500/40 backdrop-blur-sm glass-edit">
                            <textarea
                              class="glass-edit-area w-full min-h-[90px] px-3 py-2 bg-white/50 border border-white/40 rounded-lg text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-io-500 font-mono text-[13px] resize-y"
                              value={editContent()}
                              onInput={(e) => setEditContent(e.target.value)}
                              onKeyDown={(e) => {
                                if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                                  handleSaveEdit()
                                } else if (e.key === 'Escape') {
                                  handleCancelEdit()
                                }
                              }}
                              ref={(el) => el?.focus()}
                            />
                            <div class="flex justify-end gap-2 mt-2">
                              <button
                                class="px-3 py-2 rounded-lg text-xs font-medium bg-bg-tertiary text-text-primary hover:bg-border-primary transition-colors"
                                onClick={handleCancelEdit}
                              >
                                取消
                              </button>
                              <button
                                class="px-3 py-2 rounded-lg text-xs font-medium bg-nt-io-500 text-text-primary hover:bg-nt-io-600 transition-colors"
                                onClick={handleSaveEdit}
                              >
                                保存并重发
                              </button>
                            </div>
                          </div>
                        ) : (
                          <>
                            {isTool && message.toolCalls && message.toolCalls.length > 0 ? (
                              <div class="space-y-1">
                                <For each={message.toolCalls}>
                                  {(call) => <ToolCallCard call={call} />}
                                </For>
                              </div>
                            ) : (
                              <div class="mb" style={{ 'contain': 'content' }}>
                                {isUser ? (
                                  <p class="whitespace-pre-wrap">{message.content}</p>
                                ) : (
                                  <>
                                    <div
                                      class="relative"
                                      style={collapsed ? { 'max-height': '360px', overflow: 'hidden' } : undefined}
                                    >
                                      <Markdown
                                        content={collapsed
                                          ? foldPreview(message.content, LONG_MSG_SNIPPET_CHARS)
                                          : message.content}
                                      />
                                      {collapsed && (
                                        <div
                                          class="absolute inset-x-0 bottom-0 h-16 pointer-events-none"
                                          style={{
                                            background: 'linear-gradient(180deg, rgba(255,255,255,0) 0%, rgba(255,255,255,0.92) 100%)',
                                          }}
                                        />
                                      )}
                                    </div>
                                    {collapsible && (
                                      <button
                                        class="mt-1 flex items-center gap-1 text-[11px] font-medium text-nt-io-700 hover:text-nt-io-800 transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none rounded"
                                        onClick={() =>
                                          setExpandedMsgIds((prev) => ({ ...prev, [message.id]: !prev[message.id] }))
                                        }
                                        aria-expanded={expanded}
                                      >
                                        {expanded ? '收起' : '展开全文'}
                                      </button>
                                    )}
                                    {/* 批次4：assistant 回复内 checklist 任务组（可勾选、本地持久化） */}
                                    <Show when={message.role === 'assistant' && !message.isStreaming}>
                                      <TaskList content={collapsed ? foldPreview(message.content, LONG_MSG_SNIPPET_CHARS) : message.content} messageId={message.id} />
                                    </Show>
                                  </>
                                )}
                                <div class="mb-t">{formatTime(message.timestamp)}</div>
                              </div>
                            )}

                            <Show when={message.toolCalls && message.toolCalls.length > 0 && message.role !== 'tool'}>
                              <div class="mt-2 space-y-1">
                                <For each={message.toolCalls}>
                                  {(call) => <ToolCallCard call={call} />}
                                </For>
                              </div>
                            </Show>

                            <Show when={message.attachments && message.attachments.length > 0}>
                              <div class="mt-2 space-y-2">
                                <For each={message.attachments}>
                                  {(att) => (
                                    <FilePreview
                                      attachment={att}
                                      onAnnotate={(hint) => setAnnotationHint(hint || null)}
                                    />
                                  )}
                                </For>
                              </div>
                            </Show>

                            {/* 流式光标：红色块状闪烁 */}
                            {message.isStreaming && <span class="stream-cursor">▍</span>}
                          </>
                        )}

                        {/* hover 操作行：opacity-0→100，150ms 过渡（iPolloWork duration-150 语言）；用户消息右对齐 */}
                        <div class={clsx(
                          'flex items-center gap-1 mt-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity duration-150',
                          isUser && 'justify-end',
                          isEditing && 'opacity-0'
                        )}>
                          <div class="flex items-center gap-1">
                            {message.role === 'assistant' && !message.isStreaming && !isGenerating() && (
                              <>
                                <button
                                  class={actionBtnClass}
                                  onClick={() => handleRegenerate(message)}
                                  aria-label="重新生成"
                                  title="重新生成"
                                >
                                  <RotateCcw class="w-3.5 h-3.5" />
                                </button>
                                <button
                                  class={actionBtnClass}
                                  onClick={() => handleEditMessage(message)}
                                  aria-label="编辑并重发"
                                  title="编辑并重发"
                                >
                                  <Edit2 class="w-3.5 h-3.5" />
                                </button>
                              </>
                            )}
                            {message.role === 'user' && !message.isStreaming && !isGenerating() && (
                              <button
                                class={actionBtnClass}
                                onClick={() => handleEditMessage(message)}
                                aria-label="编辑并重发"
                                title="编辑并重发"
                              >
                                <Edit2 class="w-3.5 h-3.5" />
                              </button>
                            )}
                            {!message.isStreaming && (
                              <button
                                class={actionBtnClass}
                                onClick={() => handleCopy(message.content, message.id)}
                                aria-label="复制"
                                title="复制"
                              >
                                <Show when={copiedId() === message.id} fallback={<Copy class="w-3.5 h-3.5" />}>
                                  <Check class="w-3.5 h-3.5 text-emerald-600" />
                                </Show>
                              </button>
                            )}
                          </div>
                        </div>
                      </div>
                    </div>
                  )
                }}
              </For>
            </div>
          </Show>
        </div>

        {/* Stream Error Toast（结构化错误：what / why / next） */}
        <Show when={streamError()}>
          <div role="alert" class="mx-4 mb-2 p-3 bg-red-50/80 border border-red-600/25 rounded-xl flex items-start gap-2 animate-in flex-shrink-0 shadow-sm backdrop-blur-md">
            <AlertCircle class="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
            <div class="min-w-0 flex-1">
              <span class="text-sm text-red-700">{streamError()}</span>
              <Show when={streamErrorDetail()}>
                {(d) => (
                  <div class="agent-err-steps mt-2 space-y-1">
                    <div><span class="agent-err-k">WHAT</span>{d().what}</div>
                    <div><span class="agent-err-k">WHY</span>{d().why}</div>
                    <div><span class="agent-err-k">NEXT</span>{d().next}</div>
                  </div>
                )}
              </Show>
            </div>
            <button
              class="ml-auto p-1 text-red-600 hover:text-red-800 flex-shrink-0"
              onClick={() => { setStreamError(null); setStreamErrorDetail(null) }}
              aria-label="关闭错误提示"
            >
              <Square class="w-4 h-4" />
            </button>
          </div>
        </Show>

        {/* Info Toast（信息通知：中性色 / InfoIcon，区别于错误提示，如 /help /compact） */}
        <Show when={infoNotice()}>
          <div class="mx-4 mb-2 p-3 bg-white/70 border border-border-primary/60 rounded-xl flex items-center gap-2 animate-in flex-shrink-0 shadow-sm backdrop-blur-md">
            <Info class="w-5 h-5 text-text-muted flex-shrink-0" />
            <span class="text-sm text-text-primary">{infoNotice()}</span>
            <button
              class="ml-auto p-1 text-text-muted hover:text-text-primary"
              onClick={() => setInfoNotice(null)}
              aria-label="关闭提示"
            >
              <Square class="w-4 h-4" />
            </button>
          </div>
        </Show>

        {/* Annotation pending hint */}
        <Show when={annotationHint()}>
          <div class="mx-4 mb-2 p-3 bg-nt-core-500/10 border border-nt-core-500/30 rounded-lg flex items-center gap-2 flex-shrink-0">
            <Highlighter class="w-4 h-4 text-nt-core-600 flex-shrink-0" />
            <span class="text-xs text-nt-core-700 truncate font-mono">{annotationHint()}</span>
            <button
              class="ml-auto p-1 text-text-muted hover:text-text-primary flex-shrink-0"
              onClick={() => setAnnotationHint(null)}
              aria-label="移除标注"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </Show>

        {/* 批次6：@-mention 上下文预算 chips（引用文件 → 行数 + 估算 token；总预算显式展示） */}
        <Show when={mentionRefs().length > 0}>
          <div class="mx-4 mb-2 flex flex-wrap items-center gap-2 flex-shrink-0">
            <For each={mentionRefs()}>
              {(ref) => (
                <div class="glass-chip flex items-center gap-2 p-2 bg-nt-io-500/5 border border-nt-io-500/30 rounded-lg text-xs text-text-primary backdrop-blur-sm" title={`${ref.path} · ${ref.lines} 行 · ≈${ref.tokens} tok`}>
                  <AtSign class="w-3.5 h-3.5 text-nt-io-600 flex-shrink-0" />
                  <span class="max-w-[180px] truncate font-mono">{ref.path}</span>
                  <span class="text-text-muted">{ref.lines}行 · ≈{ref.tokens} tok</span>
                  <button
                    class="p-0.5 text-text-muted hover:text-nt-io-600"
                    onClick={() => removeMentionRef(ref.path)}
                    aria-label={`移除引用 ${ref.path}`}
                    title="移除引用预算"
                  >
                    <X class="w-3 h-3" />
                  </button>
                </div>
              )}
            </For>
            <span class="text-10px font-mono text-text-muted">
              引用预算 ≈{mentionRefs().reduce((s, r) => s + r.tokens, 0)} tok
            </span>
          </div>
        </Show>

        {/* Pending attachments */}
        <Show when={pendingAttachments().length > 0}>
          <div class="mx-4 mb-2 flex flex-wrap gap-2 flex-shrink-0">
            <For each={pendingAttachments()}>
              {(att, i) => (
                <div class="glass-chip flex items-center gap-2 p-2 bg-white/40 border border-white/40 rounded-lg text-xs text-text-primary backdrop-blur-sm">
                  <span class="max-w-[160px] truncate font-mono">{att.name}</span>
                  <span class="text-text-muted">{formatSize(att.size)}</span>
                  <button
                    class="p-1 text-text-muted hover:text-nt-core-600"
                    onClick={() => removeAttachment(i())}
                    aria-label={`移除附件 ${att.name}`}
                    title="移除附件"
                  >
                    <X class="w-3.5 h-3.5" />
                  </button>
                </div>
              )}
            </For>
          </div>
        </Show>

        {/* ===== 底部输入区：cic 玻璃（消息流模式下） ===== */}
        <Show when={messages().length > 0}>
          {/* 上下文即将用尽提示（任务3，对标 Claude /compact 建议；只读数据源，非侵入可关闭） */}
          <Show when={compactHintVisible()}>
            <div class="mx-auto w-full max-w-[800px] px-6 pt-3">
              <div
                class="flex items-center gap-2 px-3 py-2 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-700 text-xs animate-in shadow-sm"
                role="status"
              >
                <AlertTriangle class="w-4 h-4 flex-shrink-0" />
                <span class="flex-1 min-w-0">
                  上下文即将用尽（{Math.round(contextPct() ?? 0)}%），输入 <span class="font-mono">/compact</span> 压缩
                </span>
                <button
                  class="flex-shrink-0 px-2 py-1 rounded-md font-medium bg-amber-500/15 hover:bg-amber-500/25 transition-colors focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
                  onClick={runCompact}
                >
                  压缩会话
                </button>
                <button
                  class="flex-shrink-0 p-1 rounded hover:bg-amber-500/15 transition-colors focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
                  onClick={() => setCompactHintDismissed(true)}
                  aria-label="关闭上下文提示"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </Show>

          {/* 批次1：Plan 模式激活提示（只读横幅，对标 Claude Code plan 只读阶段的视觉区分） */}
          <Show when={permissionMode() === 'plan' && !planPending() && !isGenerating()}>
            <div class="mx-auto w-full max-w-[800px] px-6 pt-3">
              <div
                class="flex items-center gap-2 px-3 py-2 rounded-xl bg-nt-core-500/10 border border-nt-core-500/30 text-nt-core-700 text-xs animate-in shadow-sm"
                role="status"
              >
                <FileText class="w-4 h-4 flex-shrink-0" />
                <span class="flex-1 min-w-0">规划模式：AI 只分析不执行。完成后请审阅并批准计划。</span>
                <button
                  class="flex-shrink-0 p-1 rounded hover:bg-nt-core-500/15 transition-colors focus-visible:ring-2 focus-visible:ring-nt-core-500 focus-visible:outline-none"
                  onClick={() => cyclePermissionMode()}
                  aria-label="退出规划模式"
                  title="退出规划模式"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </Show>

          {/* 批次1：Plan 批准条（规划产出待批准；批准→切换 accept_edits 并同轮执行，拒绝/取消→留规划态） */}
          <Show when={planPending() && !isGenerating()}>
            <div class="mx-auto w-full max-w-[800px] px-6 pt-3">
              <div
                class="flex items-center gap-2 px-3 py-2.5 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-900 text-xs animate-in shadow-sm"
                role="dialog"
                aria-label="计划批准"
              >
                <FileText class="w-4 h-4 flex-shrink-0" />
                <span class="flex-1 min-w-0 font-medium">计划已生成，请审阅后决定下一步</span>
                <div class="flex items-center gap-1.5 flex-shrink-0">
                  <button
                    class="px-2.5 py-1 rounded-md font-medium bg-emerald-600 text-white hover:bg-emerald-700 transition-colors focus-visible:ring-2 focus-visible:ring-emerald-500 focus-visible:outline-none"
                    onClick={approvePlan}
                    aria-label="批准并执行计划"
                  >
                    批准并执行
                  </button>
                  <button
                    class="px-2.5 py-1 rounded-md font-medium bg-amber-500/15 hover:bg-amber-500/25 transition-colors focus-visible:ring-2 focus-visible:ring-amber-500 focus-visible:outline-none"
                    onClick={rejectPlan}
                    aria-label="拒绝计划"
                  >
                    拒绝
                  </button>
                  <button
                    class="px-2.5 py-1 rounded-md font-medium bg-white/40 hover:bg-white/60 transition-colors text-text-muted hover:text-text-primary focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                    onClick={cancelPlan}
                    aria-label="取消计划"
                  >
                    取消
                  </button>
                </div>
              </div>
            </div>
          </Show>

          {/* 回到底部按钮（流式滚动锁：用户上滚暂停跟随后出现） */}
          <Show when={!stickToBottom()}>
            <button
              class="sb-tobottom"
              onClick={scrollToBottom}
              title="回到底部"
            >
              <NeoChevronRight class="w-4 h-4 rotate-90" />
              回到底部
            </button>
          </Show>

          <div class="flex-shrink-0 border-t border-border-primary/40 bg-white/10 backdrop-blur-xl">
            {/* 斜杠命令菜单（输入以 / 开头时浮层；Esc 可临时收起） */}
            <Show when={slashActive()}>
              <div class="relative max-w-[800px] mx-auto px-6 pt-2">
                <SlashMenu
                  query={slashQuery() ?? ''}
                  commands={slashFiltered()}
                  selectedIdx={Math.min(slashIdx(), slashFiltered().length - 1)}
                  onSelect={runSlash}
                  onHover={(idx) => setSlashIdx(idx)}
                />
              </div>
            </Show>
            <div class="max-w-[800px] mx-auto w-full px-6 pt-3 pb-2">
              {/* Harness 执行报告面板（/run 或 ⌘K 运行后展示，不污染会话历史） */}
              <Show when={harnessRunning() || harnessReport()}>
                <div class="pb-2">
                  <HarnessReportCard running={harnessRunning()} report={harnessReport()} steps={harnessSteps()} onClose={() => setHarnessReport(null)} />
                </div>
              </Show>

              {/* 左栏 bot 审批流可见性（可 approve/reject） */}
              <Show when={approvals().length > 0}>
                <div class="pb-2">
                  <ApprovalPanel approvals={approvals()} onClose={() => setApprovals([])} onResolved={refreshApprovals} />
                </div>
              </Show>

              {/* 左栏 bot 文件内联编辑（read_file + write_file 闭环） */}
              <div class="pb-2">
                <button
                  class="w-full px-2 py-1 rounded bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 text-11px"
                  onClick={() => setShowFileEditor(!showFileEditor())}
                >
                  {showFileEditor() ? '收起文件编辑' : '文件内联编辑'}
                </button>
                <Show when={showFileEditor()}>
                  <FileEditorPanel onClose={() => setShowFileEditor(false)} />
                </Show>
              </div>

              <div class="cic">
                <textarea
                  ref={setTextareaRef}
                  class="flex-1 bg-transparent border-none resize-none min-h-[52px] max-h-[240px] py-2 text-[14px] leading-relaxed text-text-primary placeholder-text-muted/70 focus:outline-none focus:ring-0 focus:border-none"
                  placeholder={isGenerating() ? '生成中仍可输入，下一条稍后发送…' : '输入消息… (Enter 发送, Shift+Enter 换行)'}
                  value={inputValue()}
                  onInput={handleInput}
                  onKeyDown={handleKeyDown}
                  onPaste={handlePasteImage}
                  rows={1}
                />
                <div class="cic-actions">
                  <div class="cic-left">
                    <button class="cic-attach" onClick={handlePickAttachment} aria-label="附加文件" title="附加文件">
                      <svg viewBox="0 0 16 16">
                        <line x1="8" y1="3" x2="8" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                        <line x1="4" y1="8" x2="12" y2="8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                      </svg>
                    </button>
                      <ModelSwitcher disabled={isGenerating()} />
                    </div>
                    <div class="cic-right">
                      <Show when={inputValue().trim() || pendingAttachments().length > 0}>
                      <span class="text-10px text-text-muted/70 font-mono mr-2">
                        ≈{estimateTokens(inputValue())} tok
                        <Show when={pendingAttachments().length > 0}> · {pendingAttachments().length} 附件</Show>
                      </span>
                    </Show>
                    <button
                      class="vc-btn vc-send"
                      disabled={!inputValue().trim() && pendingAttachments().length === 0 && !annotationHint() && !isGenerating()}
                      onClick={isGenerating() ? handleStop : handleSend}
                      aria-label={isGenerating() ? '停止生成' : '发送消息'}
                      title={isGenerating() ? '停止生成' : '发送消息'}
                    >
                      {isGenerating() ? <Square class="w-4 h-4" /> : <NeoSend class="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>

              {/* 底部状态条 */}
              <div class="flex items-center justify-between mt-2 px-1 pb-1">
                <div class="flex items-center gap-3 text-10px text-text-muted/80">
                  {/* 权限模式徽章（对标 Claude 顶栏 mode 徽章）：短标签 + 色点，点击循环切换 */}
                  <button
                    class="flex items-center gap-1.5 px-2 py-0.5 rounded-md border border-white/30 bg-white/40 hover:bg-white/60 transition-colors text-10px font-medium text-text-primary focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
                    onClick={cyclePermissionMode}
                    disabled={isGenerating()}
                    aria-label={`权限模式：${permissionModeInfo().label}，点击切换`}
                    title={`权限模式：${permissionModeInfo().label}（点击切换）`}
                  >
                    <span class={clsx('w-1.5 h-1.5 rounded-full bg-current', permissionModeInfo().color)} />
                    <span class="font-medium">{permissionModeInfo().shortLabel}</span>
                  </button>
                  <Show when={activeModel()}>
                    <span class="font-mono text-nt-io-700">{activeModel()}</span>
                  </Show>
                  <span>NeoTrix v{appVersion() ?? '0.18.0'}</span>
                  <span class="hidden md:inline">Enter 发送 · Shift+Enter 换行</span>
                </div>
              </div>
            </div>
          </div>
        </Show>
        </Show>

        {/* ===== 协同视图（cowork） ===== */}
        <Show when={activeView() === 'cowork'}>
          <CoworkView />
        </Show>

        {/* ===== 电脑控制：侧栏内嵌标签页（对标 Claude 侧栏） ===== */}
        <Show when={activeView() === 'computer'}>
          <ComputerUse embedded open onClose={() => setActiveView('chat')} />
        </Show>
      </main>

      {/* ===== 右栏：Artifact Pane + 文件树（设计 v2） ===== */}
      <RightBar />

      {/* ===== 设置弹窗（根级渲染，避免被侧栏 overflow 裁剪 ===== */}
      <SettingsModal open={settingsOpen()} onClose={() => setSettingsOpen(false)} />

      {/* ===== ⌘K 命令面板（根级渲染） ===== */}
      <CommandPalette
        open={paletteOpen()}
        commands={paletteCommands.map((c) => ({ ...c, run: () => { pushRecentCmd(c.id); c.run() } }))}
        recent={recentPaletteIds()
          .map((id) => paletteCommands.find((c) => c.id === id))
          .filter(Boolean)
          .map((c) => ({ ...c!, run: () => { pushRecentCmd(c!.id); c!.run() } })) as PaletteCommand[]}
        onClose={() => setPaletteOpen(false)}
      />
    </div>
  )
}
