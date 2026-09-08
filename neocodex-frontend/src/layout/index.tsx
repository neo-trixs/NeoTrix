import { createSignal, onMount, onCleanup, For, Show } from 'solid-js'
import { invoke } from '@tauri-apps/api/core'
import Navbar from './components/navbar'

interface Message {
  role: 'user' | 'assistant'
  content: string
  timestamp: number
}

interface LlamacppHealth {
  running: boolean
  port: number
  pid: number | null
  uptime_secs: number
  model_loaded: string | null
  binary_found: boolean
}

interface LocalModel {
  name: string
  path: string
  size_bytes: number
  quantization: string
}

export default function App() {
  const [messages, setMessages] = createSignal<Message[]>([])
  const [input, setInput] = createSignal('')
  const [loading, setLoading] = createSignal(false)
  const [health, setHealth] = createSignal<LlamacppHealth | null>(null)
  const [models, setModels] = createSignal<LocalModel[]>([])
  const [selectedModel, setSelectedModel] = createSignal<string | null>(null)
  const [conversations, setConversations] = createSignal<{ id: string; title: string; messages: Message[] }[]>([])
  const [currentConversationId, setCurrentConversationId] = createSignal<string | null>(null)

  let messagesEndRef: HTMLDivElement | undefined

  const createNewConversation = () => {
    const newId = Date.now().toString()
    const newConv = { id: newId, title: 'New Chat', messages: [] }
    setConversations(prev => [newConv, ...prev])
    setCurrentConversationId(newId)
    setMessages([])
  }

  const selectConversation = (id: string) => {
    const conv = conversations().find(c => c.id === id)
    if (conv) {
      setCurrentConversationId(id)
      setMessages(conv.messages)
    }
  }

  const updateConversationTitle = (id: string, title: string) => {
    setConversations(prev => prev.map(c => c.id === id ? { ...c, title } : c))
  }

  const loadHealth = async () => {
    try {
      const result = await invoke<{ ok: boolean; data: LlamacppHealth }>('domain_call', {
        domain: 'llamacpp',
        action: 'health',
        args: {},
      })
      if (result.ok) setHealth(result.data)
    } catch (e) {
      console.error('Health check failed:', e)
    }
  }

  const loadModels = async () => {
    try {
      const result = await invoke<{ ok: boolean; data: { models: LocalModel[] } }>('domain_call', {
        domain: 'llamacpp',
        action: 'models',
        args: {},
      })
      if (result.ok) {
        setModels(result.data.models)
        if (result.data.models.length > 0 && !selectedModel()) {
          setSelectedModel(result.data.models[0].name)
        }
      }
    } catch (e) {
      console.error('Load models failed:', e)
    }
  }

  const startServer = async () => {
    try {
      setLoading(true)
      await invoke('domain_call', {
        domain: 'llamacpp',
        action: 'start',
        args: {},
      })
      await loadHealth()
    } catch (e) {
      console.error('Start server failed:', e)
    } finally {
      setLoading(false)
    }
  }

  const stopServer = async () => {
    try {
      await invoke('domain_call', {
        domain: 'llamacpp',
        action: 'stop',
        args: {},
      })
      await loadHealth()
    } catch (e) {
      console.error('Stop server failed:', e)
    }
  }

  const sendMessage = async () => {
    if (!input().trim() || loading()) return

    const userMsg: Message = {
      role: 'user',
      content: input().trim(),
      timestamp: Date.now(),
    }
    setMessages(prev => [...prev, userMsg])
    setInput('')
    setLoading(true)

    // Update conversation title if it's the first message
    const currentId = currentConversationId()
    if (currentId) {
      const conv = conversations().find(c => c.id === currentId)
      if (conv && conv.messages.length === 0) {
        updateConversationTitle(currentId, userMsg.content.slice(0, 30) + (userMsg.content.length > 30 ? '...' : ''))
      }
    }

    try {
      const apiMessages = [...messages(), userMsg].map(m => ({
        role: m.role,
        content: m.content,
      }))

      const result = await invoke<{ ok: boolean; data: any }>('domain_call', {
        domain: 'llamacpp',
        action: 'send',
        args: {
          messages: apiMessages,
          temperature: 0.7,
          max_tokens: 2048,
        },
      })

      if (result.ok) {
        const content = result.data?.choices?.[0]?.message?.content || 'No response'
        const assistantMsg: Message = {
          role: 'assistant',
          content,
          timestamp: Date.now(),
        }
        setMessages(prev => [...prev, assistantMsg])

        // Save messages to conversation
        if (currentId) {
          setConversations(prev => prev.map(c => 
            c.id === currentId 
              ? { ...c, messages: [...c.messages, userMsg, assistantMsg] }
              : c
          ))
        }
      }
    } catch (e) {
      console.error('Send message failed:', e)
      const errorMsg: Message = {
        role: 'assistant',
        content: `Error: ${e}`,
        timestamp: Date.now(),
      }
      setMessages(prev => [...prev, errorMsg])
    } finally {
      setLoading(false)
    }
  }

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }

  const formatSize = (bytes: number) => {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
  }

  const formatUptime = (secs: number) => {
    const h = Math.floor(secs / 3600)
    const m = Math.floor((secs % 3600) / 60)
    if (h > 0) return `${h}h ${m}m`
    return `${m}m`
  }

  onMount(() => {
    loadHealth()
    loadModels()
    const interval = setInterval(loadHealth, 10000)
    onCleanup(() => clearInterval(interval))
  })

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-[#0f0f17] text-[#e4e4e7]">
      {/* Sidebar */}
      <aside class="w-[280px] flex flex-col bg-[#0a0a12] border-r border-white/5 rounded-l-[14px]">
        {/* Logo */}
        <div class="px-5 py-4 border-b border-white/5">
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded-xl bg-gradient-to-br from-[#FF8C42] to-[#E06B20] flex items-center justify-center shadow-lg shadow-orange-500/20">
              <span class="text-white font-bold text-sm">NT</span>
            </div>
            <div>
              <h1 class="text-sm font-semibold text-white">NeoTrix</h1>
              <p class="text-[10px] text-[#6b7280]">AI-Native Desktop</p>
            </div>
          </div>
        </div>

        {/* Server Status */}
        <div class="px-4 py-3 border-b border-white/5">
          <div class="flex items-center justify-between mb-2">
            <span class="text-xs font-medium text-[#9ca3af]">Server</span>
            <span
              class={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                health()?.running 
                  ? 'bg-emerald-500/15 text-emerald-400' 
                  : 'bg-red-500/15 text-red-400'
              }`}
            >
              {health()?.running ? '● Running' : '○ Stopped'}
            </span>
          </div>
          <Show when={health()?.running}>
            <div class="text-[10px] text-[#6b7280] space-y-0.5 mb-2">
              <div>Port: {health()?.port}</div>
              <div>Uptime: {formatUptime(health()?.uptime_secs ?? 0)}</div>
              <Show when={health()?.model_loaded}>
                <div class="truncate">Model: {health()?.model_loaded?.split('-').slice(0, 3).join('-')}</div>
              </Show>
            </div>
          </Show>
          <div class="flex gap-2">
            <Show
              when={!health()?.running}
              fallback={
                <button
                  onClick={stopServer}
                  class="flex-1 px-3 py-1.5 text-[11px] font-medium bg-white/5 text-[#9ca3af] rounded-lg hover:bg-white/10 transition-colors"
                >
                  Stop
                </button>
              }
            >
              <button
                onClick={startServer}
                disabled={loading()}
                class="flex-1 px-3 py-1.5 text-[11px] font-medium bg-gradient-to-r from-[#FF8C42] to-[#E06B20] text-white rounded-lg hover:opacity-90 disabled:opacity-50 transition-opacity shadow-lg shadow-orange-500/20"
              >
                {loading() ? 'Starting...' : 'Start'}
              </button>
            </Show>
          </div>
        </div>

        {/* Conversation List */}
        <div class="flex-1 overflow-hidden flex flex-col px-4 py-3 border-b border-white/5">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-[10px] font-semibold text-[#6b7280] uppercase tracking-wider">
              Chats
            </h3>
            <button
              onClick={createNewConversation}
              class="text-[10px] text-[#FF8C42] hover:text-[#E06B20] transition-colors"
            >
              + New
            </button>
          </div>
          <div class="flex-1 overflow-y-auto space-y-1 pr-1 custom-scrollbar">
            <For each={conversations()}>
              {conv => (
                <button
                  onClick={() => selectConversation(conv.id)}
                  class={`w-full text-left px-3 py-2 text-xs rounded-lg transition-colors ${
                    currentConversationId() === conv.id
                      ? 'bg-[#FF8C42]/10 text-[#FF8C42] border border-[#FF8C42]/20'
                      : 'text-[#9ca3af] hover:bg-white/5 border border-transparent'
                  }`}
                >
                  <div class="truncate font-medium">{conv.title}</div>
                  <div class="text-[10px] text-[#6b7280] mt-0.5">
                    {conv.messages.length} messages
                  </div>
                </button>
              )}
            </For>
            <Show when={conversations().length === 0}>
              <p class="text-xs text-[#6b7280] text-center py-4">No conversations yet</p>
            </Show>
          </div>
        </div>

        {/* Model Selector */}
        <div class="flex-1 overflow-hidden flex flex-col px-4 py-3">
          <h3 class="text-[10px] font-semibold text-[#6b7280] uppercase tracking-wider mb-2">
            Models ({models().length})
          </h3>
          <div class="flex-1 overflow-y-auto space-y-1 pr-1 custom-scrollbar">
            <For each={models()}>
              {model => (
                <button
                  onClick={() => setSelectedModel(model.name)}
                  class={`w-full text-left px-3 py-2 text-xs rounded-lg transition-colors ${
                    selectedModel() === model.name
                      ? 'bg-[#FF8C42]/10 text-[#FF8C42] border border-[#FF8C42]/20'
                      : 'text-[#9ca3af] hover:bg-white/5 border border-transparent'
                  }`}
                >
                  <div class="truncate font-medium">{model.name.split('-').slice(0, 3).join('-')}</div>
                  <div class="text-[10px] text-[#6b7280] mt-0.5">
                    {formatSize(model.size_bytes)} · {model.quantization}
                  </div>
                </button>
              )}
            </For>
            <Show when={models().length === 0}>
              <p class="text-xs text-[#6b7280] text-center py-4">No models found</p>
            </Show>
          </div>
        </div>

        {/* New Chat */}
        <div class="px-4 pb-4 pt-2">
          <button
            onClick={createNewConversation}
            class="w-full px-4 py-2.5 text-xs font-medium bg-white/5 text-[#e4e4e7] rounded-xl hover:bg-white/10 transition-colors border border-white/5 whitespace-nowrap overflow-hidden text-ellipsis"
          >
            + New Chat
          </button>
        </div>
      </aside>

      {/* Main Area */}
      <main class="flex-1 flex flex-col min-w-0 bg-[#0f0f17] rounded-r-[14px]">
        <Navbar />

        {/* Messages */}
        <div class="flex-1 overflow-y-auto px-6 py-4 custom-scrollbar">
          <Show when={messages().length === 0}>
            <div class="flex items-center justify-center h-full">
              <div class="text-center">
                <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-[#FF8C42] to-[#E06B20] flex items-center justify-center shadow-xl shadow-orange-500/30">
                  <span class="text-white font-bold text-xl">NT</span>
                </div>
                <h2 class="text-xl font-semibold text-white mb-2">NeoTrix Desktop</h2>
                <p class="text-sm text-[#6b7280]">
                  {health()?.running ? 'Ready to assist' : 'Start the server to begin'}
                </p>
              </div>
            </div>
          </Show>
          <div class="space-y-4 max-w-3xl mx-auto">
            <For each={messages()}>
              {msg => (
                <div
                  class={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    class={`max-w-[85%] px-4 py-3 rounded-2xl text-sm leading-relaxed ${
                      msg.role === 'user'
                        ? 'bg-gradient-to-r from-[#FF8C42] to-[#E06B20] text-white rounded-br-md'
                        : 'bg-white/5 text-[#e4e4e7] rounded-bl-md border border-white/5'
                    }`}
                  >
                    {msg.content}
                  </div>
                </div>
              )}
            </For>
            <Show when={loading()}>
              <div class="flex justify-start">
                <div class="bg-white/5 px-4 py-3 rounded-2xl rounded-bl-md text-sm text-[#6b7280] border border-white/5">
                  <span class="animate-pulse">Thinking...</span>
                </div>
              </div>
            </Show>
          </div>
          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div class="px-6 pb-6 pt-2">
          <div class="max-w-3xl mx-auto flex gap-3">
            <textarea
              value={input()}
              onInput={e => setInput(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder={health()?.running ? 'Type a message...' : 'Start the server first'}
              disabled={!health()?.running || loading()}
              class="flex-1 resize-none rounded-2xl bg-white/5 border border-white/10 px-4 py-3 text-sm text-[#e4e4e7] placeholder-[#6b7280] focus:outline-none focus:border-[#FF8C42]/50 focus:ring-1 focus:ring-[#FF8C42]/30 disabled:opacity-50 transition-all"
              rows={1}
            />
            <button
              onClick={sendMessage}
              disabled={!health()?.running || loading() || !input().trim()}
              class="px-5 py-3 bg-gradient-to-r from-[#FF8C42] to-[#E06B20] text-white rounded-2xl text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-opacity shadow-lg shadow-orange-500/20"
            >
              Send
            </button>
          </div>
        </div>
      </main>
    </div>
  )
}
