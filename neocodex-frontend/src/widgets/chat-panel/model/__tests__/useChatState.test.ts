/**
 * widgets/chat-panel/model/__tests__/useChatState.test.ts — useChatState 单元测试
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock solid-js router BEFORE importing useChatState
vi.mock('@solidjs/router', () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: () => '/chat' }),
}))

vi.mock('../../../../stores/chat', () => ({
  chatStore: {
    state: { currentSessionId: 'test-session' },
    currentMessages: [],
    isGenerating: false,
    currentSession: { id: 'test-session' },
    loadSessions: vi.fn(),
    loadSessionMessages: vi.fn().mockResolvedValue(undefined),
  },
}))

vi.mock('../../../../stores/tags', () => ({
  tagsStore: { state: { tags: [] } },
}))

vi.mock('../../../../components/PermissionModeSelector', () => ({
  PERMISSION_MODES: [
    { value: 'auto', label: '自动' },
    { value: 'manual', label: '手动' },
    { value: 'accept_edits', label: '接受编辑' },
    { value: 'plan', label: '规划' },
  ],
}))

vi.mock('../../../../api/events', () => ({
  subscribeStream: vi.fn().mockResolvedValue(() => {}),
  subscribeMenuEvents: vi.fn().mockResolvedValue(() => {}),
}))

vi.mock('../../../../api', () => ({
  neocodex: {
    agentStatus: vi.fn().mockResolvedValue({ context_usage: 0.5 }),
    compactSession: vi.fn().mockResolvedValue(undefined),
  },
  harness: {
    harnessApprovalList: vi.fn().mockResolvedValue([]),
    harnessApiMap: vi.fn().mockResolvedValue([]),
  },
  unified: {
    cliList: vi.fn().mockResolvedValue([]),
  },
  errText: vi.fn((e: any) => e?.message || String(e)),
}))

vi.mock('../../../../lib/usePolling', () => ({
  usePolling: vi.fn(),
}))

vi.mock('../../../../api/query', () => ({
  query: vi.fn().mockResolvedValue(null),
}))

vi.mock('../../../../components/AgentActivityLog', () => ({
  // ActivityStep type is just a type, no mock needed
}))

vi.mock('../../../../components/AgentActivityBar', () => ({
  // AgentPhase type is just a type, no mock needed
}))

vi.mock('../../../../lib/errorRootCause', () => ({
  rootCause: vi.fn(() => null),
}))

describe('useChatState', () => {
  let useChatState: typeof import('../useChatState').useChatState

  beforeEach(async () => {
    vi.clearAllMocks()
    // Dynamic import so mocks are applied
    const mod = await import('../useChatState')
    useChatState = mod.useChatState
  })

  it('should return all state signals', () => {
    const state = useChatState()

    // Input
    expect(typeof state.inputValue).toBe('function')
    expect(typeof state.setInputValue).toBe('function')
    expect(typeof state.textareaRef).toBe('function')
    expect(typeof state.setTextareaRef).toBe('function')
    expect(typeof state.adjustTextarea).toBe('function')

    // Edit
    expect(typeof state.editingMessageId).toBe('function')
    expect(typeof state.setEditingMessageId).toBe('function')
    expect(typeof state.editContent).toBe('function')
    expect(typeof state.setEditContent).toBe('function')

    // Sidebar
    expect(typeof state.sidebarCollapsed).toBe('function')
    expect(typeof state.setSidebarCollapsed).toBe('function')
    expect(typeof state.sidebarWidth).toBe('function')
    expect(typeof state.setSidebarWidth).toBe('function')

    // Settings
    expect(typeof state.settingsOpen).toBe('function')
    expect(typeof state.setSettingsOpen).toBe('function')

    // Stream
    expect(typeof state.streamError).toBe('function')
    expect(typeof state.setStreamError).toBe('function')

    // Agent activity
    expect(typeof state.agentPhase).toBe('function')
    expect(typeof state.setAgentPhase).toBe('function')
    expect(typeof state.agentToolCount).toBe('function')
    expect(typeof state.setAgentToolCount).toBe('function')

    // Info notice
    expect(typeof state.infoNotice).toBe('function')
    expect(typeof state.showInfo).toBe('function')
    expect(typeof state.showError).toBe('function')

    // Harness
    expect(typeof state.harnessRoute).toBe('function')
    expect(typeof state.setHarnessRoute).toBe('function')
    expect(typeof state.harnessReport).toBe('function')
    expect(typeof state.setHarnessReport).toBe('function')
    expect(typeof state.harnessRunning).toBe('function')
    expect(typeof state.setHarnessRunning).toBe('function')

    // Permission
    expect(typeof state.permissionMode).toBe('function')
    expect(typeof state.setPermissionMode).toBe('function')
    expect(typeof state.cyclePermissionMode).toBe('function')

    // Theme
    expect(typeof state.theme).toBe('function')
    expect(typeof state.cycleTheme).toBe('function')

    // Panel
    expect(typeof state.activePanel).toBe('function')
    expect(typeof state.setActivePanel).toBe('function')
    expect(typeof state.togglePanel).toBe('function')

    // View
    expect(typeof state.activeView).toBe('function')
    expect(typeof state.setActiveView).toBe('function')

    // Tags
    expect(typeof state.activeTags).toBe('function')
    expect(typeof state.toggleTag).toBe('function')
    expect(typeof state.clearTags).toBe('function')

    // Context
    expect(typeof state.contextPct).toBe('function')
    expect(typeof state.compactHintDismissed).toBe('function')
    expect(typeof state.compacting).toBe('function')
    expect(typeof state.runCompact).toBe('function')

    // Attachments
    expect(typeof state.pendingAttachments).toBe('function')
    expect(typeof state.setPendingAttachments).toBe('function')
    expect(typeof state.removeAttachment).toBe('function')

    // Streaming
    expect(typeof state.currentAssistantMsgId).toBe('function')
    expect(typeof state.setCurrentAssistantMsgId).toBe('function')

    // Scroll
    expect(typeof state.setScrollRef).toBe('function')
    expect(typeof state.stickToBottom).toBe('function')
    expect(typeof state.scrollToBottom).toBe('function')

    // Approval counts
    expect(typeof state.approvalAccepted).toBe('function')
    expect(typeof state.approvalRejected).toBe('function')

    // Autonomy
    expect(typeof state.autonomyLevel).toBe('function')
    expect(typeof state.autonomyRate).toBe('function')
    expect(typeof state.autonomyLevelNum).toBe('function')

    // Messages
    expect(typeof state.messages).toBe('function')
    expect(typeof state.isGenerating).toBe('function')
    expect(typeof state.currentSession).toBe('function')
  })

  it('should have correct initial values', () => {
    const state = useChatState()

    expect(state.inputValue()).toBe('')
    expect(state.editingMessageId()).toBeNull()
    expect(state.editContent()).toBe('')
    expect(state.showEditOrig()).toBe(false)
    expect(state.sidebarCollapsed()).toBe(false)
    expect(state.sidebarWidth()).toBe(280)
    expect(state.settingsOpen()).toBe(false)
    expect(state.streamError()).toBeNull()
    expect(state.showErrRaw()).toBe(false)
    expect(state.agentPhase()).toBe('idle')
    expect(state.agentToolCount()).toBe(0)
    expect(state.infoNotice()).toBeNull()
    expect(state.harnessRoute()).toBeNull()
    expect(state.harnessRunning()).toBe(false)
    expect(state.showFileEditor()).toBe(false)
    expect(state.copiedId()).toBeNull()
    expect(state.planPending()).toBeNull()
    expect(state.activeModel()).toBeNull()
    expect(state.paletteOpen()).toBe(false)
    expect(state.msgSearch()).toBe('')
    expect(state.activePanel()).toBeNull()
    expect(state.activeView()).toBe('chat')
    expect(state.activeTags()).toEqual([])
    expect(state.contextPct()).toBeNull()
    expect(state.compactHintDismissed()).toBe(false)
    expect(state.compacting()).toBe(false)
    expect(state.pendingAttachments()).toEqual([])
    expect(state.currentAssistantMsgId()).toBeNull()
    expect(state.stickToBottom()).toBe(true)
    expect(state.approvalAccepted()).toBe(0)
    expect(state.approvalRejected()).toBe(0)
  })

  it('should update inputValue', () => {
    const state = useChatState()
    expect(state.inputValue()).toBe('')
    state.setInputValue('hello')
    expect(state.inputValue()).toBe('hello')
  })

  it('should update editingMessageId', () => {
    const state = useChatState()
    expect(state.editingMessageId()).toBeNull()
    state.setEditingMessageId('msg-123')
    expect(state.editingMessageId()).toBe('msg-123')
  })

  it('should update editContent', () => {
    const state = useChatState()
    expect(state.editContent()).toBe('')
    state.setEditContent('edited content')
    expect(state.editContent()).toBe('edited content')
  })

  it('should update sidebarCollapsed', () => {
    const state = useChatState()
    expect(state.sidebarCollapsed()).toBe(false)
    state.setSidebarCollapsed(true)
    expect(state.sidebarCollapsed()).toBe(true)
  })

  it('should update sidebarWidth', () => {
    const state = useChatState()
    expect(state.sidebarWidth()).toBe(280)
    state.setSidebarWidth(320)
    expect(state.sidebarWidth()).toBe(320)
  })

  it('should update settingsOpen', () => {
    const state = useChatState()
    expect(state.settingsOpen()).toBe(false)
    state.setSettingsOpen(true)
    expect(state.settingsOpen()).toBe(true)
  })

  it('should update streamError', () => {
    const state = useChatState()
    expect(state.streamError()).toBeNull()
    state.setStreamError('test error')
    expect(state.streamError()).toBe('test error')
  })

  it('should update agentPhase', () => {
    const state = useChatState()
    expect(state.agentPhase()).toBe('idle')
    state.setAgentPhase('thinking')
    expect(state.agentPhase()).toBe('thinking')
  })

  it('should update permissionMode', () => {
    const state = useChatState()
    // Default depends on localStorage mock
    expect(typeof state.permissionMode()).toBe('string')
    state.setPermissionMode('manual')
    expect(state.permissionMode()).toBe('manual')
  })

  it('should update activePanel and togglePanel', () => {
    const state = useChatState()
    expect(state.activePanel()).toBeNull()
    state.togglePanel('git')
    expect(state.activePanel()).toBe('git')
    state.togglePanel('git')
    expect(state.activePanel()).toBeNull()
  })

  it('should update activeView', () => {
    const state = useChatState()
    expect(state.activeView()).toBe('chat')
    state.setActiveView('cowork')
    expect(state.activeView()).toBe('cowork')
  })

  it('should toggleTag work correctly', () => {
    const state = useChatState()
    expect(state.activeTags()).toEqual([])
    state.toggleTag('rust')
    expect(state.activeTags()).toEqual(['rust'])
    state.toggleTag('ts')
    expect(state.activeTags()).toEqual(['rust', 'ts'])
    state.toggleTag('rust')
    expect(state.activeTags()).toEqual(['ts'])
  })

  it('should clearTags work correctly', () => {
    const state = useChatState()
    state.toggleTag('rust')
    state.toggleTag('ts')
    expect(state.activeTags()).toEqual(['rust', 'ts'])
    state.clearTags()
    expect(state.activeTags()).toEqual([])
  })

  it('should removeAttachment work correctly', () => {
    const state = useChatState()
    state.setPendingAttachments(() => [{ name: 'file1.txt' } as any, { name: 'file2.txt' } as any])
    expect(state.pendingAttachments()).toHaveLength(2)
    state.removeAttachment(0)
    expect(state.pendingAttachments()).toHaveLength(1)
    expect(state.pendingAttachments()[0].name).toBe('file2.txt')
  })

  it('should autonomyLevel compute correctly', () => {
    const state = useChatState()
    expect(state.autonomyLevel()).toBe('待校准')
    state.setApprovalAccepted(() => 8)
    state.setApprovalRejected(() => 2)
    expect(state.autonomyLevel()).toBe('高信任')
    state.setApprovalAccepted(() => 5)
    state.setApprovalRejected(() => 5)
    expect(state.autonomyLevel()).toBe('协作')
    state.setApprovalAccepted(() => 2)
    state.setApprovalRejected(() => 8)
    expect(state.autonomyLevel()).toBe('审慎')
  })

  it('should autonomyRate compute correctly', () => {
    const state = useChatState()
    expect(state.autonomyRate()).toBe(0)
    state.setApprovalAccepted(() => 8)
    state.setApprovalRejected(() => 2)
    expect(state.autonomyRate()).toBe(0.8)
  })

  it('should autonomyLevelNum compute correctly', () => {
    const state = useChatState()
    state.setPermissionMode('auto')
    expect(state.autonomyLevelNum()).toBe(2)
    state.setPermissionMode('manual')
    expect(state.autonomyLevelNum()).toBe(0)
    state.setPermissionMode('plan')
    expect(state.autonomyLevelNum()).toBe(1)
    state.setPermissionMode('accept_edits')
    expect(state.autonomyLevelNum()).toBe(3)
  })
})
