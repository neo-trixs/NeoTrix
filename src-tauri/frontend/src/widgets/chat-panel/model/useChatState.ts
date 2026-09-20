/**
 * widgets/chat-panel/model/useChatState.ts — Chat 状态管理 (组合层)
 *
 * 从 Chat.tsx 提取所有状态声明（signals, effects）
 * Chat.tsx 从 2311 行 → ~800 行（渲染 + 薄胶水）
 *
 * v2: 按领域拆分为子 Hook（hooks/），本文件为组合层，保持 ChatStateReturn 接口不变。
 */
import { createSignal, createEffect, onMount, onCleanup } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { chatStore, type Message, type NeoCodexAttachmentDto } from '../../../stores/chat'
import { tagsStore } from '../../../stores/tags'
import { subscribeStream, subscribeMenuEvents, type UnlistenFn } from '../../../api/events'
import { harness, unified, errText } from '../../../api'
import type { AgentStatus } from '../../../api/types'
import { rootCause } from '../../../lib/errorRootCause'

// ── 子 Hook 导入 ──
import { useInputState } from './hooks/useInputState'
import { useSidebarState } from './hooks/useSidebarState'
import { useAgentActivity } from './hooks/useAgentActivity'
import { useStreamError } from './hooks/useStreamError'
import { useHarnessState } from './hooks/useHarnessState'
import { useMsgSearch } from './hooks/useMsgSearch'
import { useThemeState } from './hooks/useThemeState'
import { useContextState } from './hooks/useContextState'
import { usePermissionState } from './hooks/usePermissionState'

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
  agentPhase: () => any
  setAgentPhase: (p: any) => void
  agentDomain: () => string | null
  setAgentDomain: (d: string | null) => void
  agentToolCount: () => number
  setAgentToolCount: (fn: (n: number) => number) => void
  agentLastActivity: () => string | null
  setAgentLastActivity: (a: string | null) => void
  agentLog: () => any[]
  pushLog: (step: Omit<any, 'ts'>) => void
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
  harnessReport: () => any | null
  setHarnessReport: (r: any | null) => void
  harnessRunning: () => boolean
  setHarnessRunning: (v: boolean) => void
  harnessSteps: () => any[]
  setHarnessSteps: (fn: (prev: any[]) => any[]) => void

  // Approvals
  approvals: () => any[]
  setApprovals: (a: any[]) => void
  refreshApprovals: () => void

  // File editor
  showFileEditor: () => boolean
  setShowFileEditor: (v: boolean) => void

  // Copied
  copiedId: () => string | null
  setCopiedId: (id: string | null) => void

  // Permission
  permissionMode: () => any
  setPermissionMode: (m: any) => void
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

  // ── 组合子 Hook ──
  const input = useInputState()
  const sidebar = useSidebarState()
  const agent = useAgentActivity()
  const streamErr = useStreamError()
  const harnessState = useHarnessState()
  const search = useMsgSearch()
  const themeState = useThemeState()

  // ── 独立 signals (轻量，不值得单独提取) ──
  const [settingsOpen, setSettingsOpen] = createSignal(false)
  const [showFileEditor, setShowFileEditor] = createSignal(false)
  const [copiedId, setCopiedId] = createSignal<string | null>(null)
  const [annotationHint, setAnnotationHint] = createSignal<string | null>(null)
  const [planPending, setPlanPending] = createSignal<{ msgId: string } | null>(null)
  const [activeModel, setActiveModel] = createSignal<string | null>(null)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)
  const [exportMenuOpen, setExportMenuOpen] = createSignal(false)
  const [activePanel, setActivePanel] = createSignal<string | null>(null)
  const [activeView, setActiveView] = createSignal<'chat' | 'cowork' | 'computer'>('chat')
  const [activeTags, setActiveTags] = createSignal<string[]>([])
  const toggleTag = (name: string) => {
    setActiveTags((prev) => prev.includes(name) ? prev.filter((t) => t !== name) : [...prev, name])
  }
  const clearTags = () => setActiveTags([])

  // ── Edit ──
  const [editingMessageId, setEditingMessageId] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')
  const [showEditOrig, setShowEditOrig] = createSignal(false)

  // ── Palette ──
  const [paletteOpen, setPaletteOpen] = createSignal(false)
  const [shortcutHelpOpen, setShortcutHelpOpen] = createSignal(false)
  const [recentPaletteIds, setRecentPaletteIds] = createSignal<string[]>([])
  const pushRecentCmd = (id: string) =>
    setRecentPaletteIds((ids) => [id, ...ids.filter((x) => x !== id)].slice(0, 5))

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

  // ── Attachments ──
  const [pendingAttachments, setPendingAttachments] = createSignal<NeoCodexAttachmentDto[]>([])
  const removeAttachment = (idx: number) => setPendingAttachments(p => p.filter((_, i) => i !== idx))

  // ── Mentions ──
  const [mentionRefs, setMentionRefs] = createSignal<{ path: string; lines: number; tokens: number }[]>([])
  const removeMentionRef = (path: string) => setMentionRefs(prev => prev.filter(r => r.path !== path))

  // ── Live tokens ──
  const liveGenTokens = () => {
    const ms = chatStore.currentMessages
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

  // ── Info notice ──
  const [infoNotice, setInfoNotice] = createSignal<string | null>(null)
  let infoNoticeTimer: ReturnType<typeof setTimeout> | undefined
  const showInfo = (msg: string, ms = 3000) => {
    if (infoNoticeTimer) clearTimeout(infoNoticeTimer)
    setInfoNotice(msg)
    infoNoticeTimer = setTimeout(() => setInfoNotice(null), ms)
  }
  const showError = (msg: string) => {
    streamErr.setStreamError(msg)
    setTimeout(() => streamErr.setStreamError(null), 3000)
  }

  // ── 需要跨 Hook 依赖的子 Hook ──
  const context = useContextState({
    isGenerating,
    showInfo,
    setStreamError: streamErr.setStreamError,
  })
  const permission = usePermissionState({ isGenerating, showInfo })

  // ── Event cleanup ──
  const [unlistenStream, setUnlistenStream] = createSignal<UnlistenFn | null>(null)
  const [unlistenMenu, setUnlistenMenu] = createSignal<UnlistenFn | null>(null)

  // ── Effects ──
  createEffect(() => {
    const r = harnessState.harnessRoute()
    agent.setAgentDomain(r?.domain ?? null)
  })
  createEffect(() => {
    if (activePanel() === 'cost') { setActivePanel(null); navigate('/insights') }
  })

  return {
    // Input (from sub-hook)
    inputValue: input.inputValue, setInputValue: input.setInputValue,
    textareaRef: input.textareaRef, setTextareaRef: input.setTextareaRef,
    adjustTextarea: input.adjustTextarea,

    // Edit
    editingMessageId, setEditingMessageId, editContent, setEditContent, showEditOrig, setShowEditOrig,

    // Sidebar (from sub-hook)
    sidebarCollapsed: sidebar.sidebarCollapsed, setSidebarCollapsed: sidebar.setSidebarCollapsed,
    sidebarWidth: sidebar.sidebarWidth, setSidebarWidth: sidebar.setSidebarWidth,
    onSidebarResizeDown: sidebar.onSidebarResizeDown,

    // Settings
    settingsOpen, setSettingsOpen,

    // Stream error (from sub-hook)
    streamError: streamErr.streamError, setStreamError: streamErr.setStreamError,
    streamErrorDetail: streamErr.streamErrorDetail, setStreamErrorDetail: streamErr.setStreamErrorDetail,
    showErrRaw: streamErr.showErrRaw, setShowErrRaw: streamErr.setShowErrRaw,

    // Agent activity (from sub-hook)
    agentPhase: agent.agentPhase, setAgentPhase: agent.setAgentPhase,
    agentDomain: agent.agentDomain, setAgentDomain: agent.setAgentDomain,
    agentToolCount: agent.agentToolCount, setAgentToolCount: agent.setAgentToolCount,
    agentLastActivity: agent.agentLastActivity, setAgentLastActivity: agent.setAgentLastActivity,
    agentLog: agent.agentLog, pushLog: agent.pushLog,
    logOpen: agent.logOpen, setLogOpen: agent.setLogOpen,

    // Info notice
    infoNotice, setInfoNotice, showInfo, showError,

    // Harness (from sub-hook)
    harnessRoute: harnessState.harnessRoute, setHarnessRoute: harnessState.setHarnessRoute,
    harnessReport: harnessState.harnessReport, setHarnessReport: harnessState.setHarnessReport,
    harnessRunning: harnessState.harnessRunning, setHarnessRunning: harnessState.setHarnessRunning,
    harnessSteps: harnessState.harnessSteps, setHarnessSteps: harnessState.setHarnessSteps,
    approvals: harnessState.approvals, setApprovals: harnessState.setApprovals,
    refreshApprovals: harnessState.refreshApprovals,

    // File editor
    showFileEditor, setShowFileEditor,

    // Copied
    copiedId, setCopiedId,

    // Permission (from sub-hook)
    permissionMode: permission.permissionMode, setPermissionMode: permission.setPermissionMode,
    cyclePermissionMode: permission.cyclePermissionMode,

    // Annotation
    annotationHint, setAnnotationHint,

    // Plan
    planPending, setPlanPending,

    // Model/Version
    activeModel, setActiveModel, appVersion, setAppVersion,

    // Palette
    paletteOpen, setPaletteOpen, shortcutHelpOpen, setShortcutHelpOpen,
    recentPaletteIds, pushRecentCmd,

    // Search (from sub-hook)
    msgSearch: search.msgSearch, setMsgSearch: search.setMsgSearch,
    msgSearchOpen: search.msgSearchOpen, setMsgSearchOpen: search.setMsgSearchOpen,
    msgSearchInput: search.msgSearchInput, setMsgSearchInput: search.setMsgSearchInput,
    matchCount: search.matchCount, matchCursor: search.matchCursor,
    setMatchCursor: search.setMatchCursor, matchedIds: search.matchedIds,
    jumpMatch: search.jumpMatch,

    // Theme (from sub-hook)
    theme: themeState.theme, cycleTheme: themeState.cycleTheme,

    // Export
    exportMenuOpen, setExportMenuOpen,

    // Panel
    activePanel, setActivePanel,
    togglePanel: (id: string) => setActivePanel(activePanel() === id ? null : id),

    // View
    activeView, setActiveView,

    // Tags
    activeTags, toggleTag, clearTags,

    // Context (from sub-hook)
    contextPct: context.contextPct,
    compactHintDismissed: context.compactHintDismissed,
    setCompactHintDismissed: context.setCompactHintDismissed,
    compactHintVisible: context.compactHintVisible,
    compacting: context.compacting,
    runCompact: context.runCompact,

    // Attachments
    pendingAttachments, setPendingAttachments, removeAttachment,

    // Mentions
    mentionRefs, setMentionRefs, removeMentionRef,

    // Expanded
    expandedMsgIds, setExpandedMsgIds,

    // Slash
    slashIdx, setSlashIdx, slashDismissed, setSlashDismissed,

    // Streaming
    currentAssistantMsgId, setCurrentAssistantMsgId, generation, activeGen,

    // Scroll
    scrollRef: undefined as HTMLDivElement | undefined, setScrollRef,
    stickToBottom, setStickToBottom, scrollToBottom,

    // Approval counts (from sub-hook)
    approvalAccepted: permission.approvalAccepted, setApprovalAccepted: permission.setApprovalAccepted,
    approvalRejected: permission.approvalRejected, setApprovalRejected: permission.setApprovalRejected,
    autonomyLevel: permission.autonomyLevel, autonomyRate: permission.autonomyRate,
    autonomyLevelNum: permission.autonomyLevelNum,

    // Live tokens
    liveGenTokens, streamWatchdogTimer,

    // CLI
    unifiedCliCmds, setUnifiedCliCmds, harnessCaps, loadHarnessCaps,

    // Messages
    messages, isGenerating, currentSession,

    // Event cleanup
    unlistenStream, setUnlistenStream, unlistenMenu, setUnlistenMenu,

    // Search element refs (from sub-hook)
    messageEls: search.messageEls,
  } as any
}
