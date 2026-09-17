/**
 * widgets/chat-panel/model/useChatState.ts — Chat 状态管理
 *
 * 从 Chat.tsx 提取所有状态声明（signals, effects）
 * Chat.tsx 从 2311 行 → ~800 行（渲染 + 薄胶水）
 */
import { createSignal, createEffect, onMount, onCleanup } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { chatStore, type Message, type NeoCodexAttachmentDto } from '../../../stores/chat'
import { tagsStore } from '../../../stores/tags'
import { PERMISSION_MODES, type PermissionMode } from '../../../components/PermissionModeSelector'
import { subscribeStream, subscribeMenuEvents, type UnlistenFn } from '../../../api/events'
import type { HarnessApproval, HarnessStep, HarnessRunResponse } from '../../../api/harness'
import type { ActivityStep } from '../../../components/AgentActivityLog'
import type { AgentPhase } from '../../../components/AgentActivityBar'
import { neocodex, harness, unified, errText } from '../../../api'
import { usePolling } from '../../../lib/usePolling'
import { query } from '../../../api/query'
import type { AgentStatus } from '../../../api/types'
import { rootCause } from '../../../lib/errorRootCause'

export interface ChatStateReturn {
  // Input
  inputValue: () => string
  setInputValue: (v: string) => void
  textareaRef: () => HTMLTextAreaElement | null
  setTextareaRef: (el: HTMLTextAreaElement | null) => void
  adjustTextarea: () => void

  // Edit
  editingMessageId: () => string | null
  setEditingMessageId: (id: string | null) => void
  editContent: () => string
  setEditContent: (v: string) => void
  showEditOrig: () => boolean
  setShowEditOrig: (v: boolean) => void

  // Sidebar
  sidebarCollapsed: () => boolean
  setSidebarCollapsed: (v: boolean) => void
  sidebarWidth: () => number
  setSidebarWidth: (w: number) => void
  onSidebarResizeDown: (e: MouseEvent) => void

  // Settings
  settingsOpen: () => boolean
  setSettingsOpen: (v: boolean) => void

  // Stream
  streamError: () => string | null
  setStreamError: (msg: string | null) => void
  streamErrorDetail: () => { what: string; why: string; next: string } | null
  setStreamErrorDetail: (d: { what: string; why: string; next: string } | null) => void
  showErrRaw: () => boolean
  setShowErrRaw: (v: boolean) => void

  // Agent activity
  agentPhase: () => AgentPhase
  setAgentPhase: (p: AgentPhase) => void
  agentDomain: () => string | null
  setAgentDomain: (d: string | null) => void
  agentToolCount: () => number
  setAgentToolCount: (fn: (n: number) => number) => void
  agentLastActivity: () => string | null
  setAgentLastActivity: (a: string | null) => void
  agentLog: () => ActivityStep[]
  pushLog: (step: Omit<ActivityStep, 'ts'>) => void
  logOpen: () => boolean
  setLogOpen: (v: boolean | ((prev: boolean) => boolean)) => void

  // Info notice
  infoNotice: () => string | null
  setInfoNotice: (v: string | null) => void
  showInfo: (msg: string, ms?: number) => void
  showError: (msg: string) => void

  // Harness
  harnessRoute: () => { tag: string; domain: string; specialist: string } | null
  setHarnessRoute: (r: { tag: string; domain: string; specialist: string } | null) => void
  harnessReport: () => HarnessRunResponse | null
  setHarnessReport: (r: HarnessRunResponse | null) => void
  harnessRunning: () => boolean
  setHarnessRunning: (v: boolean) => void
  harnessSteps: () => HarnessStep[]
  setHarnessSteps: (fn: (prev: HarnessStep[]) => HarnessStep[]) => void

  // Approvals
  approvals: () => HarnessApproval[]
  setApprovals: (a: HarnessApproval[]) => void
  refreshApprovals: () => void

  // File editor
  showFileEditor: () => boolean
  setShowFileEditor: (v: boolean) => void

  // Copied
  copiedId: () => string | null
  setCopiedId: (id: string | null) => void

  // Permission
  permissionMode: () => PermissionMode
  setPermissionMode: (m: PermissionMode) => void
  cyclePermissionMode: () => void

  // Annotation
  annotationHint: () => string | null
  setAnnotationHint: (h: string | null) => void

  // Plan
  planPending: () => { msgId: string } | null
  setPlanPending: (p: { msgId: string } | null) => void

  // Model/Version
  activeModel: () => string | null
  setActiveModel: (m: string | null) => void
  appVersion: () => string | null
  setAppVersion: (v: string | null) => void

  // Palette
  paletteOpen: () => boolean
  setPaletteOpen: (v: boolean | ((prev: boolean) => boolean)) => void
  shortcutHelpOpen: () => boolean
  setShortcutHelpOpen: (v: boolean | ((prev: boolean) => boolean)) => void
  recentPaletteIds: () => string[]
  pushRecentCmd: (id: string) => void

  // Search
  msgSearch: () => string
  setMsgSearch: (q: string) => void
  msgSearchOpen: () => boolean
  setMsgSearchOpen: (v: boolean) => void
  msgSearchInput: () => HTMLInputElement | null
  setMsgSearchInput: (el: HTMLInputElement | null) => void
  matchCount: () => number
  matchCursor: () => number
  setMatchCursor: (n: number) => void
  matchedIds: () => string[]
  jumpMatch: (dir: 1 | -1) => void

  // Theme
  theme: () => 'gold' | 'lilac' | 'mint'
  cycleTheme: () => void

  // Export
  exportMenuOpen: () => boolean
  setExportMenuOpen: (v: boolean) => void

  // Panel
  activePanel: () => string | null
  setActivePanel: (p: string | null) => void
  togglePanel: (id: string) => void

  // View
  activeView: () => 'chat' | 'cowork' | 'computer'
  setActiveView: (v: 'chat' | 'cowork' | 'computer') => void

  // Tags
  activeTags: () => string[]
  toggleTag: (name: string) => void
  clearTags: () => void

  // Context
  contextPct: () => number | null
  compactHintDismissed: () => boolean
  setCompactHintDismissed: (v: boolean) => void
  compactHintVisible: () => boolean
  compacting: () => boolean
  runCompact: () => Promise<void>

  // Attachments
  pendingAttachments: () => NeoCodexAttachmentDto[]
  setPendingAttachments: (fn: (prev: NeoCodexAttachmentDto[]) => NeoCodexAttachmentDto[]) => void
  removeAttachment: (idx: number) => void

  // Mentions
  mentionRefs: () => { path: string; lines: number; tokens: number }[]
  setMentionRefs: (fn: (prev: { path: string; lines: number; tokens: number }[]) => { path: string; lines: number; tokens: number }[]) => void
  removeMentionRef: (path: string) => void

  // Expanded messages
  expandedMsgIds: () => Record<string, boolean>
  setExpandedMsgIds: (fn: (prev: Record<string, boolean>) => Record<string, boolean>) => void

  // Slash
  slashIdx: () => number
  setSlashIdx: (fn: (prev: number) => number) => void
  slashDismissed: () => boolean
  setSlashDismissed: (v: boolean) => void

  // Streaming
  currentAssistantMsgId: () => string | null
  setCurrentAssistantMsgId: (id: string | null) => void
  generation: { value: number }
  activeGen: { value: number }

  // Scroll
  scrollRef: HTMLDivElement | undefined
  setScrollRef: (el: HTMLDivElement) => void
  stickToBottom: () => boolean
  setStickToBottom: (v: boolean) => void
  scrollToBottom: () => void

  // Approval counts
  approvalAccepted: () => number
  setApprovalAccepted: (fn: (n: number) => number) => void
  approvalRejected: () => number
  setApprovalRejected: (fn: (n: number) => number) => void

  // Autonomy
  autonomyLevel: () => '待校准' | '高信任' | '协作' | '审慎'
  autonomyRate: () => number
  autonomyLevelNum: () => number

  // Live tokens
  liveGenTokens: () => number

  // Stream watchdog
  streamWatchdogTimer: { value: ReturnType<typeof setTimeout> | undefined }

  // CLI
  unifiedCliCmds: () => any[]
  setUnifiedCliCmds: (cmds: any[]) => void
  harnessCaps: () => any[]
  loadHarnessCaps: () => Promise<void>

  // Messages
  messages: () => Message[]
  isGenerating: () => boolean
  currentSession: () => any

  // Event cleanup
  unlistenStream: () => UnlistenFn | null
  setUnlistenStream: (fn: UnlistenFn | null) => void
  unlistenMenu: () => UnlistenFn | null
  setUnlistenMenu: (fn: UnlistenFn | null) => void
}

export function useChatState(): ChatStateReturn {
  const navigate = useNavigate()

  // ── Input ──
  const [inputValue, setInputValue] = createSignal('')
  const DRAFT_KEY = 'nt_session_drafts'
  const readDrafts = (): Record<string, string> => {
    try { return JSON.parse(localStorage.getItem(DRAFT_KEY) || '{}') } catch { return {} }
  }
  const saveDraft = (id: string, text: string) => {
    const d = readDrafts()
    if (text) d[id] = text
    else delete d[id]
    try { localStorage.setItem(DRAFT_KEY, JSON.stringify(d)) } catch { /* 忽略 */ }
  }
  createEffect(() => {
    const id = chatStore.state.currentSessionId
    setInputValue(id ? (readDrafts()[id] ?? '') : '')
  })
  createEffect(() => {
    const id = chatStore.state.currentSessionId
    if (id) saveDraft(id, inputValue())
  })
  const [textareaRef, setTextareaRef] = createSignal<HTMLTextAreaElement | null>(null)
  const adjustTextarea = () => {
    const textarea = textareaRef()
    if (textarea) {
      textarea.style.height = 'auto'
      textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`
    }
  }

  // ── Edit ──
  const [editingMessageId, setEditingMessageId] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')
  const [showEditOrig, setShowEditOrig] = createSignal(false)

  // ── Sidebar ──
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false)
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

  // ── Settings ──
  const [settingsOpen, setSettingsOpen] = createSignal(false)

  // ── Stream ──
  const [streamError, setStreamError] = createSignal<string | null>(null)
  const [showErrRaw, setShowErrRaw] = createSignal(false)
  const [streamErrorDetail, setStreamErrorDetail] = createSignal<{ what: string; why: string; next: string } | null>(null)

  // ── Agent activity ──
  const [agentPhase, setAgentPhase] = createSignal<AgentPhase>('idle')
  const [agentDomain, setAgentDomain] = createSignal<string | null>(null)
  const [agentToolCount, setAgentToolCount] = createSignal(0)
  const [agentLastActivity, setAgentLastActivity] = createSignal<string | null>(null)
  const [agentLog, setAgentLog] = createSignal<ActivityStep[]>([])
  const [logOpen, setLogOpen] = createSignal(false)
  const pushLog = (step: Omit<ActivityStep, 'ts'>) => {
    setAgentLog((prev) => {
      const next = [...prev, { ...step, ts: Date.now() }]
      return next.length > 24 ? next.slice(next.length - 24) : next
    })
  }

  // ── Info notice ──
  const [infoNotice, setInfoNotice] = createSignal<string | null>(null)
  let infoNoticeTimer: ReturnType<typeof setTimeout> | undefined
  const showInfo = (msg: string, ms = 3000) => {
    if (infoNoticeTimer) clearTimeout(infoNoticeTimer)
    setInfoNotice(msg)
    infoNoticeTimer = setTimeout(() => setInfoNotice(null), ms)
  }
  const showError = (msg: string) => {
    setStreamError(msg)
    setTimeout(() => setStreamError(null), 3000)
  }

  // ── Harness ──
  const [harnessRoute, setHarnessRoute] = createSignal<{ tag: string; domain: string; specialist: string } | null>(null)
  const [harnessReport, setHarnessReport] = createSignal<HarnessRunResponse | null>(null)
  const [harnessRunning, setHarnessRunning] = createSignal(false)
  const [harnessSteps, setHarnessSteps] = createSignal<HarnessStep[]>([])

  // ── Approvals ──
  const [approvals, setApprovals] = createSignal<HarnessApproval[]>([])
  const refreshApprovals = () => {
    harness.harnessApprovalList().then(setApprovals).catch(() => setApprovals([]))
  }
  onMount(() => { refreshApprovals() })

  // ── File editor ──
  const [showFileEditor, setShowFileEditor] = createSignal(false)

  // ── Copied ──
  const [copiedId, setCopiedId] = createSignal<string | null>(null)

  // ── Permission ──
  const [permissionMode, setPermissionMode] = createSignal<PermissionMode>(
    (typeof localStorage !== 'undefined' && (localStorage.getItem('nt_perm_mode') as PermissionMode)) || 'auto',
  )
  const persistPermissionMode = (m: PermissionMode) => {
    try { localStorage.setItem('nt_perm_mode', m) } catch { /* 隐私模式忽略 */ }
  }
  const cyclePermissionMode = () => {
    if (isGenerating()) return
    const idx = PERMISSION_MODES.findIndex((m) => m.value === permissionMode())
    const next = PERMISSION_MODES[(idx + 1) % PERMISSION_MODES.length]
    setPermissionMode(next.value)
    persistPermissionMode(next.value)
    showInfo(`权限模式：${next.label}`, 2500)
  }

  // ── Annotation ──
  const [annotationHint, setAnnotationHint] = createSignal<string | null>(null)

  // ── Plan ──
  const [planPending, setPlanPending] = createSignal<{ msgId: string } | null>(null)

  // ── Model/Version ──
  const [activeModel, setActiveModel] = createSignal<string | null>(null)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)

  // ── Palette ──
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  const [shortcutHelpOpen, setShortcutHelpOpen] = createSignal(false)
  const [recentPaletteIds, setRecentPaletteIds] = createSignal<string[]>([])
  const pushRecentCmd = (id: string) =>
    setRecentPaletteIds((ids) => [id, ...ids.filter((x) => x !== id)].slice(0, 5))

  // ── Search ──
  const [msgSearch, setMsgSearch] = createSignal('')
  const [msgSearchOpen, setMsgSearchOpen] = createSignal(false)
  const [msgSearchInput, setMsgSearchInput] = createSignal<HTMLInputElement | null>(null)
  const matchCount = () => {
    const q = msgSearch().trim().toLowerCase()
    if (!q) return 0
    return messages().filter((m) => m.content.toLowerCase().includes(q)).length
  }
  const [matchCursor, setMatchCursor] = createSignal(0)
  const matchedIds = () => {
    const q = msgSearch().trim().toLowerCase()
    if (!q) return []
    return messages().filter((m) => m.content.toLowerCase().includes(q)).map((m) => m.id)
  }
  const messageEls = new Map<string, HTMLElement>()
  const jumpMatch = (dir: 1 | -1) => {
    const ids = matchedIds()
    if (!ids.length) return
    const i = (matchCursor() + dir + ids.length) % ids.length
    setMatchCursor(i)
    messageEls.get(ids[i])?.scrollIntoView({ block: 'center', behavior: 'smooth' })
  }

  // ── Theme ──
  const THEMES = ['gold', 'lilac', 'mint'] as const
  type Theme = (typeof THEMES)[number]
  const THEME_LABEL: Record<Theme, string> = { gold: '浅金', lilac: '浅紫', mint: '浅青' }
  const [theme, setTheme] = createSignal<Theme>(
    (typeof localStorage !== 'undefined' && (localStorage.getItem('nt-theme') as Theme)) ||
      (typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: light)').matches ? 'lilac' : 'gold'),
  )
  createEffect(() => {
    document.documentElement.dataset.theme = theme()
    try { localStorage.setItem('nt-theme', theme()) } catch { /* 隐私模式忽略 */ }
  })
  const cycleTheme = () => setTheme((t) => THEMES[(THEMES.indexOf(t) + 1) % THEMES.length])

  // ── Export ──
  const [exportMenuOpen, setExportMenuOpen] = createSignal(false)

  // ── Panel ──
  const [activePanel, setActivePanel] = createSignal<string | null>(null)
  const togglePanel = (id: string) => setActivePanel(activePanel() === id ? null : id)

  // ── View ──
  const [activeView, setActiveView] = createSignal<'chat' | 'cowork' | 'computer'>('chat')

  // ── Tags ──
  const [activeTags, setActiveTags] = createSignal<string[]>([])
  const toggleTag = (name: string) => {
    setActiveTags((prev) => prev.includes(name) ? prev.filter((t) => t !== name) : [...prev, name])
  }
  const clearTags = () => setActiveTags([])

  // ── Context ──
  const [contextPct, setContextPct] = createSignal<number | null>(null)
  const [compactHintDismissed, setCompactHintDismissed] = createSignal(false)
  const compactHintVisible = () => {
    const p = contextPct()
    if (p === null || p < 80) return false
    return !compactHintDismissed()
  }
  const [compacting, setCompacting] = createSignal(false)
  const runCompact = async () => {
    if (compacting() || isGenerating()) return
    const sessionId = currentSession()?.id ?? ''
    if (!sessionId) { setStreamError('当前没有激活会话，无法压缩'); setTimeout(() => setStreamError(null), 3000); return }
    setCompacting(true)
    try {
      await neocodex.compactSession(sessionId, 8)
      await chatStore.loadSessionMessages(sessionId)
      setCompactHintDismissed(true)
      showInfo('上下文已压缩，更早的对话被截断', 3000)
    } catch (error) {
      console.error('[Chat] Compact session failed:', error)
      setStreamError(errText(error) || '压缩会话失败，请重试')
      setTimeout(() => setStreamError(null), 3000)
    } finally { setCompacting(false) }
  }

  // ── Attachments ──
  const [pendingAttachments, setPendingAttachments] = createSignal<NeoCodexAttachmentDto[]>([])
  const removeAttachment = (idx: number) => setPendingAttachments(p => p.filter((_, i) => i !== idx))

  // ── Mentions ──
  const [mentionRefs, setMentionRefs] = createSignal<{ path: string; lines: number; tokens: number }[]>([])
  const removeMentionRef = (path: string) => setMentionRefs(prev => prev.filter(r => r.path !== path))

  // ── Expanded ──
  const [expandedMsgIds, setExpandedMsgIds] = createSignal<Record<string, boolean>>({})

  // ── Slash ──
  const [slashIdx, setSlashIdx] = createSignal(0)
  const [slashDismissed, setSlashDismissed] = createSignal(false)

  // ── Streaming ──
  const [currentAssistantMsgId, setCurrentAssistantMsgId] = createSignal<string | null>(null)
  const generation = { value: 0 }
  const activeGen = { value: 0 }

  // ── Scroll ──
  let scrollRef: HTMLDivElement | undefined
  const setScrollRef = (el: HTMLDivElement) => { scrollRef = el }
  const [stickToBottom, setStickToBottom] = createSignal(true)
  const scrollToBottom = () => {
    if (!scrollRef) return
    scrollRef.scrollTop = scrollRef.scrollHeight
    setStickToBottom(true)
  }

  // ── Approval counts ──
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

  // ── Live tokens ──
  const liveGenTokens = () => {
    const ms = messages()
    const last = ms[ms.length - 1]
    return last && last.role === 'assistant' ? Math.ceil(last.content.length / 4) : 0
  }

  // ── Stream watchdog ──
  const streamWatchdogTimer = { value: undefined as ReturnType<typeof setTimeout> | undefined }

  // ── CLI ──
  const [unifiedCliCmds, setUnifiedCliCmds] = createSignal<any[]>([])
  const [harnessCaps, setHarnessCaps] = createSignal<any[]>([])
  const loadHarnessCaps = async () => {
    try { setHarnessCaps(await harness.harnessApiMap()) } catch { /* 静默 */ }
  }
  onMount(() => { void loadHarnessCaps() })

  // ── Messages (reactive store access) ──
  const messages = () => chatStore.currentMessages
  const isGenerating = () => chatStore.isGenerating
  const currentSession = () => chatStore.currentSession

  // ── Event cleanup ──
  const [unlistenStream, setUnlistenStream] = createSignal<UnlistenFn | null>(null)
  const [unlistenMenu, setUnlistenMenu] = createSignal<UnlistenFn | null>(null)

  // ── Context polling ──
  usePolling({
    intervalMs: 15000,
    immediate: true,
    run: async () => {
      try {
        const s = await query<AgentStatus>('agent_status', () => neocodex.agentStatus(), { ttlMs: 3000 })
        if (s && typeof s.context_usage === 'number') setContextPct(s.context_usage * 100)
      } catch { /* 只读轮询，失败静默 */ }
    },
  })
  createEffect(() => {
    const p = contextPct()
    if (p !== null && p < 80) setCompactHintDismissed(false)
  })
  createEffect(() => {
    const r = harnessRoute()
    setAgentDomain(r?.domain ?? null)
  })

  // ── Panel cost redirect ──
  createEffect(() => {
    if (activePanel() === 'cost') { setActivePanel(null); navigate('/insights') }
  })

  return {
    inputValue, setInputValue, textareaRef, setTextareaRef, adjustTextarea,
    editingMessageId, setEditingMessageId, editContent, setEditContent, showEditOrig, setShowEditOrig,
    sidebarCollapsed, setSidebarCollapsed, sidebarWidth, setSidebarWidth, onSidebarResizeDown,
    settingsOpen, setSettingsOpen,
    streamError, setStreamError, streamErrorDetail, setStreamErrorDetail, showErrRaw, setShowErrRaw,
    agentPhase, setAgentPhase, agentDomain, setAgentDomain, agentToolCount, setAgentToolCount,
    agentLastActivity, setAgentLastActivity, agentLog, pushLog, logOpen, setLogOpen,
    infoNotice, setInfoNotice, showInfo, showError,
    harnessRoute, setHarnessRoute, harnessReport, setHarnessReport, harnessRunning, setHarnessRunning,
    harnessSteps, setHarnessSteps,
    approvals, setApprovals, refreshApprovals,
    showFileEditor, setShowFileEditor,
    copiedId, setCopiedId,
    permissionMode, setPermissionMode, cyclePermissionMode,
    annotationHint, setAnnotationHint,
    planPending, setPlanPending,
    activeModel, setActiveModel, appVersion, setAppVersion,
    paletteOpen, setPaletteOpen, shortcutHelpOpen, setShortcutHelpOpen,
    recentPaletteIds, pushRecentCmd,
    msgSearch, setMsgSearch, msgSearchOpen, setMsgSearchOpen, msgSearchInput, setMsgSearchInput,
    matchCount, matchCursor, setMatchCursor, matchedIds, jumpMatch,
    theme, cycleTheme,
    exportMenuOpen, setExportMenuOpen,
    activePanel, setActivePanel, togglePanel,
    activeView, setActiveView,
    activeTags, toggleTag, clearTags,
    contextPct, compactHintDismissed, setCompactHintDismissed, compactHintVisible, compacting, runCompact,
    pendingAttachments, setPendingAttachments, removeAttachment,
    mentionRefs, setMentionRefs, removeMentionRef,
    expandedMsgIds, setExpandedMsgIds,
    slashIdx, setSlashIdx, slashDismissed, setSlashDismissed,
    currentAssistantMsgId, setCurrentAssistantMsgId, generation, activeGen,
    scrollRef: undefined as HTMLDivElement | undefined, setScrollRef, stickToBottom, setStickToBottom, scrollToBottom,
    approvalAccepted, setApprovalAccepted, approvalRejected, setApprovalRejected,
    autonomyLevel, autonomyRate, autonomyLevelNum,
    liveGenTokens, streamWatchdogTimer,
    unifiedCliCmds, setUnifiedCliCmds, harnessCaps, loadHarnessCaps,
    messages, isGenerating, currentSession,
    unlistenStream, setUnlistenStream, unlistenMenu, setUnlistenMenu,
    messageEls,
  } as any
}
