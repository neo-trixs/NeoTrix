/**
 * useChatActions.test.ts — useChatActions 单元测试
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock router
vi.mock('@solidjs/router', () => ({
  useNavigate: () => vi.fn(),
}))

// Mock stores
const mockAddMessage = vi.fn(() => 'msg-123')
const mockAddSession = vi.fn(() => Promise.resolve('sess-123'))
const mockSetGenerating = vi.fn()
const mockDeleteMessage = vi.fn()
const mockRegenerateFrom = vi.fn(() => 'user content')
const mockEditAndResend = vi.fn()
const mockUpdateMessage = vi.fn()

vi.mock('../../../../stores/chat', () => ({
  chatStore: {
    state: { currentSessionId: 'sess-1' },
    currentMessages: [{ id: 'm1', role: 'user', content: 'hi', timestamp: new Date() }],
    isGenerating: false,
    currentSession: { id: 'sess-1' },
    addMessage: mockAddMessage,
    addSession: mockAddSession,
    setGenerating: mockSetGenerating,
    deleteMessage: mockDeleteMessage,
    regenerateFrom: mockRegenerateFrom,
    editAndResend: mockEditAndResend,
    messageContent: vi.fn(() => 'plan text'),
    updateMessage: mockUpdateMessage,
    clearMessages: vi.fn(),
    loadSessionMessages: vi.fn(),
  },
}))

vi.mock('../../../../stores/tags', () => ({
  tagsStore: { autoTagFromText: vi.fn() },
}))

vi.mock('../../../../components/PermissionModeSelector', () => ({
  PERMISSION_MODES: [
    { value: 'auto', label: '自动' },
    { value: 'manual', label: '手动' },
    { value: 'accept_edits', label: '接受编辑' },
    { value: 'plan', label: '规划' },
  ],
}))

vi.mock('../../../../api', () => ({
  neocodex: {
    sendMessageStream: vi.fn(() => Promise.resolve()),
    stopStream: vi.fn(() => Promise.resolve()),
    regenerate: vi.fn(() => Promise.resolve()),
    compactSession: vi.fn(() => Promise.resolve()),
    providerConfig: vi.fn(() => Promise.resolve({ active_model: 'test' })),
    appVersion: vi.fn(() => Promise.resolve('0.1.0')),
  },
  system: { readFile: vi.fn(() => Promise.resolve('file content')) },
  errText: vi.fn((e: any) => e?.message || String(e)),
  harness: {
    harnessExecute: vi.fn(() => Promise.resolve({ capability_tag: 'test', domain: 'nt_core', specialist: 'spec' })),
    harnessApprovalList: vi.fn(() => Promise.resolve([])),
    harnessApiMap: vi.fn(() => Promise.resolve([])),
    harnessRun: vi.fn(() => Promise.resolve({ allocations: [], internal_results: [], external_closures: [], internal_count: 0, external_gap_count: 0 })),
  },
  unified: { execCli: vi.fn(() => Promise.resolve({ message: 'ok', success: true })), cliList: vi.fn(() => Promise.resolve([])) },
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(() => Promise.resolve(null)),
}))

vi.mock('../../../../lib/text', () => ({
  guessMime: vi.fn(() => 'text/plain'),
  estimateTokens: vi.fn(() => 10),
}))

vi.mock('../../../../lib/usePolling', () => ({ usePolling: vi.fn() }))
vi.mock('../../../../api/query', () => ({ query: vi.fn(() => Promise.resolve(null)) }))
vi.mock('../../../../api/events', () => ({
  subscribeStream: vi.fn(() => Promise.resolve(() => {})),
  subscribeMenuEvents: vi.fn(() => Promise.resolve(() => {})),
}))

// useChatActions 经 domain.chat.send 发送（chat-first）；桩化实际调用路径
const mockChatSend = vi.fn(() => Promise.resolve({ id: 'r1', content: 'hi', role: 'assistant' }))
vi.mock('../../../../api/domain', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../../../api/domain')>()
  return { ...actual, chat: { ...actual.chat, send: mockChatSend } }
})

describe('useChatActions', () => {
  let state: any
  let actions: any

  beforeEach(async () => {
    vi.clearAllMocks()
    const { useChatState } = await import('../useChatState')
    const { useChatActions } = await import('../useChatActions')
    // create state with mocked router context
    state = useChatState()
    actions = useChatActions(state)
    // reset generating flag
    const { chatStore } = await import('../../../../stores/chat')
    ;(chatStore as any).isGenerating = false
  })

  it('should expose all actions', () => {
    expect(typeof actions.sendMessage).toBe('function')
    expect(typeof actions.handleSend).toBe('function')
    expect(typeof actions.handleStop).toBe('function')
    expect(typeof actions.handleRegenerate).toBe('function')
    expect(typeof actions.handleEditMessage).toBe('function')
    expect(typeof actions.handleSaveEdit).toBe('function')
    expect(typeof actions.handleCancelEdit).toBe('function')
    expect(typeof actions.handleBranch).toBe('function')
    expect(typeof actions.handleQuote).toBe('function')
    expect(typeof actions.copyMessage).toBe('function')
    expect(typeof actions.handleCopy).toBe('function')
    expect(typeof actions.approvePlan).toBe('function')
    expect(typeof actions.rejectPlan).toBe('function')
    expect(typeof actions.cancelPlan).toBe('function')
    expect(typeof actions.handlePickAttachment).toBe('function')
    expect(typeof actions.handlePasteImage).toBe('function')
    expect(typeof actions.exportConversation).toBe('function')
    expect(typeof actions.buildExport).toBe('function')
    expect(typeof actions.copyExport).toBe('function')
    expect(typeof actions.downloadMd).toBe('function')
  })

  it('sendMessage should add messages and call stream', async () => {
    await actions.sendMessage('hello')
    expect(mockAddMessage).toHaveBeenCalled()
    expect(mockChatSend).toHaveBeenCalledWith('hello')
  })

  it('handleEditMessage should set editing id and content', () => {
    const msg = { id: 'm1', content: 'original', role: 'user' } as any
    actions.handleEditMessage(msg)
    expect(state.editingMessageId()).toBe('m1')
    expect(state.editContent()).toBe('original')
  })

  it('handleCancelEdit should clear editing', () => {
    state.setEditingMessageId('m1')
    state.setEditContent('foo')
    actions.handleCancelEdit()
    expect(state.editingMessageId()).toBeNull()
    expect(state.editContent()).toBe('')
  })

  it('handleQuote should prepend quote', () => {
    const msg = { id: 'm1', content: 'quoted content here', role: 'assistant' } as any
    state.setInputValue('my reply')
    actions.handleQuote(msg)
    expect(state.inputValue()).toContain('quoted content here')
  })

  it('handleBranch should create new session and set input', async () => {
    const msg = { id: 'm1', content: 'branch content', role: 'user' } as any
    await actions.handleBranch(msg)
    expect(mockAddSession).toHaveBeenCalled()
    expect(state.inputValue()).toBe('branch content')
  })

  it('approvePlan should handle no pending', async () => {
    state.setPlanPending(null)
    await actions.approvePlan()
    // should not throw, no call
    expect(mockAddMessage).not.toHaveBeenCalledWith(expect.objectContaining({ role: 'user' }))
  })

  it('rejectPlan should clear pending', () => {
    state.setPlanPending({ msgId: 'm1' })
    state.setApprovalRejected(() => 0)
    actions.rejectPlan()
    expect(state.planPending()).toBeNull()
    expect(state.approvalRejected()).toBe(1)
  })

  it('cancelPlan should clear pending', () => {
    state.setPlanPending({ msgId: 'm1' })
    actions.cancelPlan()
    expect(state.planPending()).toBeNull()
  })

  it('buildExport md should contain messages', () => {
    const md = actions.buildExport('md')
    expect(md).toContain('hi')
  })

  it('buildExport json should be valid json', () => {
    const json = actions.buildExport('json')
    expect(() => JSON.parse(json)).not.toThrow()
  })

  it('handleSaveEdit should handle empty', () => {
    state.setEditingMessageId('m1')
    state.setEditContent('   ')
    actions.handleSaveEdit()
    // should not call editAndResend due to empty
    expect(mockEditAndResend).not.toHaveBeenCalled()
  })

  it('handleCopy should set copied id', async () => {
    // mock clipboard
    Object.assign(navigator, { clipboard: { writeText: vi.fn(() => Promise.resolve()) } })
    await actions.handleCopy('hello', 'id1')
    expect(state.copiedId()).toBe('id1')
  })
})
