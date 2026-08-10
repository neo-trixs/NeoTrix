import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import {
  Send, Square, RotateCcw, Edit2, Copy, Check, AlertCircle, Highlighter, X,
  FolderTree, Bug, FlaskConical,
  Search, Cpu, Zap,
} from 'lucide-solid'
import { chatStore, Message, ToolCallRecord, NeoCodexAttachmentDto } from '../stores/chat'
import { Sidebar } from '../components/Sidebar'
import { RightBar } from '../components/RightBar'
import { CoworkView } from '../components/CoworkView'
import { ProviderSelector } from '../components/ProviderSelector'
import { PermissionModeSelector, type PermissionMode } from '../components/PermissionModeSelector'
import { ToolCallCard } from '../components/ToolCallCard'
import { FilePreview } from '../components/FilePreview'
import { Markdown } from '../components/Markdown'
import { GitPanel } from '../components/GitPanel'
import { ScheduledTasks } from '../components/ScheduledTasks'
import { CostDashboard } from '../components/CostDashboard'
import { CheckpointTimeline } from '../components/CheckpointTimeline'
import { SideChat } from '../components/SideChat'
import { ComputerUse } from '../components/ComputerUse'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

const SUGGESTIONS: { text: string; icon: typeof FolderTree }[] = [
  { text: '解释当前项目结构', icon: FolderTree },
  { text: '修复最近的编译错误', icon: Bug },
  { text: '生成测试用例', icon: FlaskConical },
  { text: '搜索代码中的符号', icon: Search },
  { text: '分析性能瓶颈', icon: Cpu },
  { text: '优化依赖与构建', icon: Zap },
]

const actionBtnClass =
  'p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors'

/* 根据扩展名猜测 MIME（附件预览用） */
function guessMime(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() || ''
  const map: Record<string, string> = {
    png: 'image/png', jpg: 'image/jpeg', jpeg: 'image/jpeg', gif: 'image/gif', webp: 'image/webp', svg: 'image/svg+xml',
    rs: 'text/rust', ts: 'text/typescript', tsx: 'text/typescript', js: 'text/javascript', jsx: 'text/javascript',
    py: 'text/python', go: 'text/plain', java: 'text/plain', c: 'text/plain', cpp: 'text/plain', h: 'text/plain',
    rb: 'text/plain', sh: 'text/plain', json: 'application/json', yaml: 'text/yaml', yml: 'text/yaml',
    toml: 'text/plain', md: 'text/markdown', sql: 'text/plain', html: 'text/html', css: 'text/css',
    csv: 'text/csv', txt: 'text/plain', pdf: 'application/pdf',
  }
  return map[ext] ?? 'application/octet-stream'
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
/* —— 设计 v2 图标：E8 六芒星（hero） —— */
function HeroMark() {
  return (
    <svg viewBox="0 0 32 32" fill="none">
      <path d="M16 2l4 8 8 4-8 4-4 8-4-8-8-4 8-4 4-8z" fill="#E85454" opacity="0.25" />
      <path d="M16 6l2.5 5 5.5 2.5-5.5 2.5-2.5 5-2.5-5L8 13.5l5.5-2.5 2.5-5z" fill="#E85454" />
      <circle cx="16" cy="13.5" r="2.5" fill="#E85454" stroke="none" />
      <circle cx="16" cy="13.5" r="1" fill="#fff" stroke="none" />
      <path d="M4 20q4-4 8 0t8-8 8 4" stroke="#D04040" stroke-width="0.8" stroke-linecap="round" opacity="0.4" fill="none" />
    </svg>
  )
}

/* 时间自适应问候语 */
function greeting(): string {
  const h = new Date().getHours()
  if (h < 6) return '夜深了'
  if (h < 12) return '上午好'
  if (h < 14) return '中午好'
  if (h < 18) return '下午好'
  return '晚上好'
}

/* —— 设计 v2 头像图标：用户（人形）/ 助手（方框·意识） —— */
function UserIcon() {
  return (
    <svg viewBox="0 0 14 14">
      <circle cx="7" cy="4.5" r="2.5" stroke="currentColor" stroke-width="1.2" fill="none" />
      <path d="M2 12.5a5 5 0 0110 0" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
    </svg>
  )
}

function BotIcon() {
  return (
    <svg viewBox="0 0 14 14">
      <rect x="2" y="3" width="10" height="8" rx="1.5" stroke="currentColor" stroke-width="1.2" fill="none" />
      <circle cx="7" cy="7" r="1.5" stroke="currentColor" stroke-width="1" fill="none" />
    </svg>
  )
}

export function Chat() {
  const [inputValue, setInputValue] = createSignal('')
  const [textareaRef, setTextareaRef] = createSignal<HTMLTextAreaElement | null>(null)
  const [editingMessageId, setEditingMessageId] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')
  const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false)
  const [streamError, setStreamError] = createSignal<string | null>(null)
  const [copiedId, setCopiedId] = createSignal<string | null>(null)
  const [permissionMode, setPermissionMode] = createSignal<PermissionMode>('auto')
  const [annotationHint, setAnnotationHint] = createSignal<string | null>(null)
  const [activeModel, setActiveModel] = createSignal<string | null>(null)
  // 待发送附件（dialog 选择后暂存，随下一条消息发送）
  const [pendingAttachments, setPendingAttachments] = createSignal<NeoCodexAttachmentDto[]>([])

  // 视图切换：chat / cowork / computer（对应侧栏 segmented tabs）
  const [activeView, setActiveView] = createSignal<'chat' | 'cowork' | 'computer'>('chat')

  // 顶部工具栏面板：一次只开一个
  type PanelId = 'git' | 'tasks' | 'cost' | 'timeline' | 'sidechat'
  const [activePanel, setActivePanel] = createSignal<PanelId | null>(null)
  const togglePanel = (id: PanelId) => {
    setActivePanel(activePanel() === id ? null : id)
  }

  // 用函数访问 store，保证 store 变更时响应式重渲染
  const messages = () => chatStore.currentMessages
  const isGenerating = () => chatStore.isGenerating
  const currentSession = () => chatStore.currentSession

  // Event listener cleanup functions
  const [unlistenStart, setUnlistenStart] = createSignal<UnlistenFn | null>(null)
  const [unlistenToken, setUnlistenToken] = createSignal<UnlistenFn | null>(null)
  const [unlistenEnd, setUnlistenEnd] = createSignal<UnlistenFn | null>(null)
  const [unlistenDone, setUnlistenDone] = createSignal<UnlistenFn | null>(null)
  const [unlistenTool, setUnlistenTool] = createSignal<UnlistenFn | null>(null)
  // 提供商切换事件 handler（普通变量，避免 signal setter 函数式更新歧义）
  let providerChangedHandler: (() => void) | null = null

  // Current assistant message being streamed
  const [currentAssistantMsgId, setCurrentAssistantMsgId] = createSignal<string | null>(null)

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

    const startUnlisten = await listen<string>('neocodex_stream_start', (event) => {
      // Stream started - could show user message echo if needed
      console.log('[Chat] Stream started:', event.payload)
    })
    setUnlistenStart(() => startUnlisten)

    const tokenUnlisten = await listen<string>('neocodex_stream_token', (event) => {
      const msgId = currentAssistantMsgId()
      if (msgId) {
        chatStore.appendMessageContent(msgId, event.payload)
      }
    })
    setUnlistenToken(() => tokenUnlisten)

    const endUnlisten = await listen<string>('neocodex_stream_end', (event) => {
      const msgId = currentAssistantMsgId()
      if (msgId) {
        chatStore.updateMessage(msgId, event.payload, false)
      }
    })
    setUnlistenEnd(() => endUnlisten)

    const doneUnlisten = await listen<{ cancelled: boolean; elapsed_ms: number; content: string }>('neocodex_stream_done', (event) => {
      const msgId = currentAssistantMsgId()
      const wasCancelled = event.payload.cancelled

      if (msgId) {
        if (wasCancelled) {
          // Message already has partial content from tokens, just mark as not streaming
          chatStore.updateMessage(msgId, event.payload.content, false)
          setStreamError('生成已停止')
          setTimeout(() => setStreamError(null), 3000)
        } else {
          chatStore.updateMessage(msgId, event.payload.content, false)
        }
      }

      chatStore.setGenerating(false)
      setCurrentAssistantMsgId(null)
    })
    setUnlistenDone(() => doneUnlisten)

    const toolUnlisten = await listen<{ name: string; args: string; result: string; duration_ms: number; success: boolean }>('neocodex_stream_tool', (event) => {
      const msgId = currentAssistantMsgId()
      if (msgId) {
        const toolCall: ToolCallRecord = {
          id: `tool-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
          name: event.payload.name,
          args: event.payload.args,
          result: event.payload.result,
          duration_ms: event.payload.duration_ms,
          success: event.payload.success,
        }
        chatStore.appendToolCall(msgId, toolCall)
      }
    })
    setUnlistenTool(() => toolUnlisten)

    // 读取当前激活模型（仅用于状态栏展示，只读命令）
    try {
      const cfg = await invoke<{ active_model: string }>('neocodex_provider_config')
      setActiveModel(cfg.active_model || null)
    } catch {
      /* 展示字段，静默失败 */
    }

    // 监听提供商切换事件（SettingsModal / ProviderSelector 广播），同步状态栏模型
    const onProviderChanged = () => {
      invoke<{ active_model: string }>('neocodex_provider_config')
        .then((cfg) => setActiveModel(cfg.active_model || null))
        .catch(() => {})
    }
    window.addEventListener('neotrix:provider-changed', onProviderChanged)
    providerChangedHandler = onProviderChanged
  })

  // Clean up event listeners
  onCleanup(() => {
    unlistenStart()?.()
    unlistenToken()?.()
    unlistenEnd()?.()
    unlistenDone()?.()
    unlistenTool()?.()
    if (providerChangedHandler) {
      window.removeEventListener('neotrix:provider-changed', providerChangedHandler)
    }
  })

  // 消息区自动滚动：新消息/会话切换强制到底，流式期间若在底部则跟随
  createEffect(() => {
    const sid = currentSession()?.id ?? null
    const sessionChanged = sid !== prevSessionId
    prevSessionId = sid
    messages()
    isGenerating()
    const el = scrollRef
    if (!el) return
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 240
    if (sessionChanged || nearBottom) {
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight
      })
    }
  })

  const handleInput = (e: Event) => {
    const target = e.target as HTMLTextAreaElement
    setInputValue(target.value)
    adjustTextarea()
  }

  const handleKeyDown = (e: KeyboardEvent) => {
    // 面板快捷键：⌘1-⌘5 切换 5 个功能面板，⌘6 切换电脑控制视图，Esc 关闭
    if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '8') {
      const idx = Number(e.key) - 1
      if (idx === 5) {
        // ⌘6：电脑控制 → 侧栏内嵌视图
        e.preventDefault()
        setActiveView(activeView() === 'computer' ? 'chat' : 'computer')
        return
      }
      const panels: PanelId[] = ['git', 'tasks', 'cost', 'timeline', 'sidechat']
      const target = panels[idx]
      if (target) {
        e.preventDefault()
        togglePanel(target)
      }
      return
    }
    if (e.key === 'Escape' && activePanel()) {
      setActivePanel(null)
      return
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  // 核心发送：addMessage(user) → addMessage(assistant placeholder) → invoke
  // opts.userMessageAdded = true 表示用户消息已由编辑/重生成逻辑写入，跳过重复添加
  const sendMessage = async (content: string, opts?: { userMessageAdded?: boolean }) => {
    if (!content || isGenerating()) return

    if (!currentSession()) {
      chatStore.addSession()
    }

    if (!opts?.userMessageAdded) {
      chatStore.addMessage({ role: 'user', content })
    }
    const atts = pendingAttachments()
    setInputValue('')
    setAnnotationHint(null)
    setPendingAttachments([])
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

    try {
      // Call the real Tauri IPC command for streaming
      await invoke('neocodex_send_message_stream', {
        content,
        attachments: atts.length > 0 ? atts : null,
        regenerate: false,
        permission_mode: permissionMode(),
        temperature: 0.7,
        max_tokens: 4096,
      })
      // The actual streaming happens via events (neocodex_stream_token, etc.)
    } catch (error) {
      console.error('[Chat] Send message failed:', error)
      const errorMsg = error instanceof Error ? error.message : '发送失败，请重试'
      setStreamError(errorMsg)

      // Update the assistant message with error
      if (assistantMsgId) {
        chatStore.updateMessage(assistantMsgId, `❌ ${errorMsg}`, false)
      }
      chatStore.setGenerating(false)
      setCurrentAssistantMsgId(null)
    }
  }

  const handleSend = async () => {
    let content = inputValue().trim()
    if (!content && !annotationHint() && pendingAttachments().length === 0) return
    if (!content) content = annotationHint()! // send the annotation even with empty text
    if (annotationHint() && content !== annotationHint()!) {
      content = `${content}\n\n${annotationHint()}`
    }
    await sendMessage(content)
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
          const content = await invoke<string>('read_file', { path: p })
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

  /* ── 语音输入：MediaRecorder 录音 → voice_get_transcription → 填入输入框 ── */
  const [recording, setRecording] = createSignal(false)
  let mediaRecorder: MediaRecorder | null = null
  let audioChunks: Blob[] = []

  const handleVoiceToggle = async () => {
    if (recording()) {
      // 停止录音并转写
      mediaRecorder?.stop()
      return
    }
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
      mediaRecorder = new MediaRecorder(stream)
      audioChunks = []
      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) audioChunks.push(e.data)
      }
      mediaRecorder.onstop = async () => {
        stream.getTracks().forEach((t) => t.stop())
        setRecording(false)
        const blob = new Blob(audioChunks, { type: 'audio/webm' })
        if (blob.size === 0) return
        try {
          const buf = await blob.arrayBuffer()
          const base64 = btoa(String.fromCharCode(...new Uint8Array(buf)))
          const tr = await invoke<{ text: string; confidence: number }>('voice_get_transcription', {
            audioData: base64,
            language: null,
            model: null,
          })
          if (tr.text) {
            setInputValue((prev) => (prev ? `${prev} ${tr.text}` : tr.text))
            adjustTextarea()
          }
        } catch (e) {
          console.error('[Chat] Transcription failed:', e)
        }
      }
      mediaRecorder.start()
      setRecording(true)
    } catch (e) {
      console.error('[Chat] Mic access denied:', e)
    }
  }

  const handleSuggestion = (text: string) => {
    setInputValue(text)
    handleSend()
  }

  const handleStop = async () => {
    try {
      await invoke('neocodex_stop_stream')
    } catch (error) {
      console.error('[Chat] Stop stream failed:', error)
    }
    chatStore.abortGeneration()
    setCurrentAssistantMsgId(null)
  }

  const handleRegenerate = () => {
    const userContent = chatStore.regenerateLast()
    if (userContent) {
      // regenerateLast 已删除最后一条助手消息，用户消息保留，跳过重复添加
      sendMessage(userContent, { userMessageAdded: true })
    }
  }

  const handleEditMessage = (message: Message) => {
    setEditingMessageId(message.id)
    setEditContent(message.content)
  }

  const handleSaveEdit = () => {
    const content = editContent().trim()
    const msgId = editingMessageId()
    if (msgId && content) {
      // editAndResend 已截断并添加新用户消息，跳过重复添加
      chatStore.editAndResend(msgId, content)
      setEditingMessageId(null)
      setEditContent('')
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

  return (
    <div class="flex h-screen bg-bg-primary">
      <Sidebar
        collapsed={sidebarCollapsed()}
        onToggleCollapse={() => setSidebarCollapsed(!sidebarCollapsed())}
        activeView={activeView()}
        onSwitchView={setActiveView}
        activePanel={activePanel()}
        onTogglePanel={(id) => togglePanel(id as PanelId)}
      />

      <main class="flex-1 flex flex-col min-w-0 overflow-hidden glass-L1 relative">
        {/* ===== 头部 ch-top：极简顶栏（对标 Claude Code 桌面，仅会话标题） ===== */}
        <Show when={activeView() === 'chat'}>
          <header class="ch-top" data-tauri-drag-region>
            <div class="flex items-center gap-2 flex-shrink-0 min-w-0">
              <Show when={currentSession()}>
                <span class="text-[11px] text-text-muted px-2 py-1 rounded bg-white/50 border border-border-primary/60 flex-shrink-0 truncate max-w-[220px]">
                  {currentSession()!.title} · {currentSession()!.messages.length} 条消息
                </span>
              </Show>
            </div>
          </header>
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
              onRestored={() => chatStore.loadSessions()}
            />
          </Show>
          <Show when={activePanel() === 'sidechat'}>
            <SideChat
              open
              sessionId={currentSession()?.id ?? null}
              onClose={() => setActivePanel(null)}
            />
          </Show>
        </Show>

        {/* ===== 消息流：气泡式 msg.r / msg.l（chat 视图） ===== */}
        <Show when={activeView() === 'chat'}>
        <div ref={scrollRef} class="flex-1 overflow-y-auto">
          <Show
            when={messages().length > 0}
            fallback={
              /* ===== 空状态：hero + cic 输入区（设计 v2） ===== */
              <div class="wc-inner h-full max-w-[640px] w-full mx-auto flex flex-col items-center justify-center gap-6 px-6 py-10 select-none">
                <div class="hero">
                  <div class="hero-svg">
                    <HeroMark />
                  </div>
                  <div>
                    <h1>{greeting()}</h1>
                    <p class="hero-sub">我是 NeoTrix，你的 AI 原生开发伙伴</p>
                  </div>
                </div>

                {/* cic 输入区 */}
                <div class="cic w-full">
                  <textarea
                    id="chatInput"
                    rows={2}
                    placeholder="输入消息… (Enter 发送, Shift+Enter 换行)"
                    value={inputValue()}
                    onInput={handleInput}
                    onKeyDown={handleKeyDown}
                    ref={setTextareaRef}
                  />
                  <div class="cic-actions">
                    <div class="cic-left">
                      <ProviderSelector iconOnly />
                      <button class="cic-attach" onClick={handlePickAttachment} aria-label="附加文件" title="附加文件">
                        <svg viewBox="0 0 16 16">
                          <line x1="8" y1="3" x2="8" y2="11" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                          <line x1="4" y1="8" x2="12" y2="8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
                        </svg>
                      </button>
                    </div>
                    <div class="cic-right">
                      <button
                        class={clsx('vc-btn vc-lang', recording() && 'recording')}
                        onClick={handleVoiceToggle}
                        aria-label={recording() ? '停止录音' : '语音输入'}
                        title={recording() ? '停止录音并转写' : '语音输入'}
                      >
                        <svg viewBox="0 0 16 16">
                          <rect x="5.5" y="2" width="5" height="7" rx="2.5" stroke="currentColor" stroke-width="1.2" fill="none" />
                          <path d="M3 7v.5a5 5 0 0010 0V7" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
                          <line x1="8" y1="12" x2="8" y2="14" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                          <line x1="5" y1="14" x2="11" y2="14" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                        </svg>
                      </button>
                      <button
                        class="vc-btn vc-send"
                        disabled={!inputValue().trim() || isGenerating()}
                        onClick={isGenerating() ? handleStop : handleSend}
                        aria-label={isGenerating() ? '停止生成' : '发送消息'}
                        title={isGenerating() ? '停止生成' : '发送消息'}
                      >
                        {isGenerating() ? <Square class="w-4 h-4" /> : <Send class="w-4 h-4" />}
                      </button>
                    </div>
                  </div>
                </div>

                {/* 快速问答 */}
                <div class="qa flex flex-wrap gap-2 justify-center">
                  <For each={SUGGESTIONS}>
                    {(s) => (
                      <button class="qa-btn" onClick={() => handleSuggestion(s.text)}>
                        <s.icon />
                        <span>{s.text}</span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            }
          >
            <div class="cs max-w-[640px] mx-auto">
              <For each={messages()}>
                {(message: Message) => {
                  const isUser = message.role === 'user'
                  const isEditing = editingMessageId() === message.id
                  const isTool = message.role === 'tool'
                  return (
                    <div class={clsx('group msg', isUser ? 'r' : 'l')}>
                      {/* 头像 */}
                      <div class="ma2">
                        {isUser ? <UserIcon /> : <BotIcon />}
                      </div>

                      <div class="flex-1 min-w-0">
                        {isEditing ? (
                          <div class="p-2 rounded-lg bg-white/60 border border-nt-io-500/40">
                            <textarea
                              class="w-full min-h-[90px] px-3 py-2 bg-white/70 border border-border-primary rounded-lg text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-io-500 font-mono text-[13px] resize-y"
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
                              <div class="mb">
                                {isUser ? (
                                  <p class="whitespace-pre-wrap">{message.content}</p>
                                ) : (
                                  <Markdown content={message.content} />
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

                        {/* hover 操作行 */}
                        <div class={clsx(
                          'flex items-center gap-1 mt-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity',
                          isEditing && 'opacity-0'
                        )}>
                          <div class="flex items-center gap-1">
                            {message.role === 'assistant' && !message.isStreaming && (
                              <>
                                <button
                                  class={actionBtnClass}
                                  onClick={handleRegenerate}
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
                            {message.role === 'user' && !message.isStreaming && (
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

        {/* Stream Error Toast */}
        <Show when={streamError()}>
          <div class="mx-4 mb-2 p-3 bg-red-50/90 border border-red-600/25 rounded-xl flex items-center gap-2 animate-in flex-shrink-0 shadow-sm backdrop-blur-md">
            <AlertCircle class="w-5 h-5 text-red-600 flex-shrink-0" />
            <span class="text-sm text-red-700">{streamError()}</span>
            <button
              class="ml-auto p-1 text-red-600 hover:text-red-800"
              onClick={() => setStreamError(null)}
              aria-label="关闭错误提示"
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

        {/* Pending attachments */}
        <Show when={pendingAttachments().length > 0}>
          <div class="mx-4 mb-2 flex flex-wrap gap-2 flex-shrink-0">
            <For each={pendingAttachments()}>
              {(att, i) => (
                <div class="flex items-center gap-2 p-2 bg-white/60 border border-border-primary/60 rounded-lg text-xs text-text-primary">
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
          <div class="flex-shrink-0 border-t border-border-primary/50 bg-white/25 backdrop-blur-xl">
            <div class="max-w-[640px] mx-auto w-full px-6 pt-3 pb-2">
              <div class="cic">
                <textarea
                  ref={setTextareaRef}
                  class="flex-1 bg-transparent border-none resize-none min-h-[26px] max-h-[160px] py-2 text-[13.5px] leading-relaxed text-text-primary placeholder-text-muted/70 focus:outline-none focus:ring-0 focus:border-none"
                  placeholder={isGenerating() ? '正在生成…' : '输入消息… (Enter 发送, Shift+Enter 换行)'}
                  value={inputValue()}
                  onInput={handleInput}
                  onKeyDown={handleKeyDown}
                  disabled={isGenerating()}
                  rows={1}
                />
                <div class="cic-actions">
                  <div class="cic-left">
                    <ProviderSelector iconOnly />
                    <PermissionModeSelector
                      value={permissionMode()}
                      onChange={setPermissionMode}
                      disabled={isGenerating()}
                      compact
                    />
                  </div>
                  <div class="cic-right">
                    <button
                      class="vc-btn vc-send"
                      disabled={!inputValue().trim() && !isGenerating()}
                      onClick={isGenerating() ? handleStop : handleSend}
                      aria-label={isGenerating() ? '停止生成' : '发送消息'}
                      title={isGenerating() ? '停止生成' : '发送消息'}
                    >
                      {isGenerating() ? <Square class="w-4 h-4" /> : <Send class="w-4 h-4" />}
                    </button>
                  </div>
                </div>
              </div>

              {/* 底部状态条 */}
              <div class="flex items-center justify-between mt-2 px-1 pb-1">
                <div class="flex items-center gap-3 text-[10px] text-text-muted/80">
                  <Show when={activeModel()}>
                    <span class="font-mono text-nt-io-700">{activeModel()}</span>
                  </Show>
                  <span>NeoTrix v0.18.0</span>
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
    </div>
  )
}
