// Unified API Client for NeoTrix Frontend
// 
// 前端只通过这个统一客户端与后端交互，不再直接调用 100+ 个 Tauri 命令

import { listen, Event } from '@tauri-apps/api/event';
import { chatSend, chatHelp } from './chat';
import { call as domainCall } from './domain';

function emptyMetadata(): ResponseMetadata {
  return {
    duration_ms: 0,
    layers_involved: [],
    capabilities_used: [],
    consciousness_state: { phi: 0, coherence: 0, gwt_resonance: 0, emotion: '', attention_focus: [] },
    confidence: 0,
  };
}

// ========== 类型定义 ==========

export interface UnifiedRequest {
  session_id?: string;
  input: string;
  context?: RequestContext;
  mode?: 'chat' | 'code' | 'design' | 'diagnose' | 'explain' | 'execute' | 'knowledge' | 'introspect';
  stream?: boolean;
}

export interface RequestContext {
  project_path?: string;
  selected_files: string[];
  selected_code?: string;
  open_file?: string;
  git_status?: string;
  metadata?: Record<string, unknown>;
}

export interface UnifiedResponse {
  response_id: string;
  session_id: string;
  content: string;
  payload?: ResponsePayload;
  message_type: 'text' | 'data' | 'progress' | 'error' | 'approval_required' | 'system_event';
  metadata: ResponseMetadata;
  is_stream_chunk: boolean;
  stream_done: boolean;
}

export interface ResponsePayload {
  type: 'code_changes' | 'file_ops' | 'terminal_commands' | 'knowledge_graph' | 'capability_tree' | 'health_snapshot' | 'evolution_plan' | 'task_result' | 'sessions' | 'providers' | 'key_value';
  data: unknown;
}

export interface ResponseMetadata {
  duration_ms: number;
  layers_involved: string[];
  capabilities_used: string[];
  consciousness_state: ConsciousnessState;
  confidence: number;
}

export interface ConsciousnessState {
  phi: number;
  coherence: number;
  gwt_resonance: number;
  emotion: string;
  attention_focus: string[];
}

export interface SessionInfo {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  message_count: number;
  project_path?: string;
}

export interface UnifiedChatRequest {
  session_id?: string;
  input: string;
  context?: RequestContext;
  mode?: string;
  stream?: boolean;
}

export interface UnifiedChatResponse {
  response_id: string;
  session_id: string;
  content: string;
  payload?: unknown;
  message_type: string;
  metadata: ResponseMetadata;
  is_stream_chunk: boolean;
  stream_done: boolean;
}

// ========== 统一 API 客户端 ==========

class UnifiedApiClient {
  private pendingStreams: Map<string, UnifiedChatResponse> = new Map();

  /**
   * 初始化统一 API（应用启动时调用）
   */
  async init(): Promise<void> {
    await chatHelp();
    console.log('[UnifiedAPI] Initialized (chat-first)');
  }

  /**
   * 统一对话接口 - 主要入口（经由自然语言 chat）
   */
  async chat(request: UnifiedChatRequest): Promise<UnifiedChatResponse> {
    const res = await chatSend(request.input, request.session_id);
    const firstAction = res.actions?.[0];
    return {
      response_id: crypto.randomUUID(),
      session_id: request.session_id || '',
      content: res.message,
      payload: firstAction ? { type: 'task_result', data: firstAction.result } : undefined,
      message_type: 'text',
      metadata: emptyMetadata(),
      is_stream_chunk: false,
      stream_done: true,
    };
  }

  /**
   * 流式对话 - 返回流 ID，通过事件监听接收分片
   *
   * 后端不再提供原生流式命令；此处经由 chatSend 一次性获取完整回复，
   * 缓存后通过 onStreamChunk 以单分片形式投递，保持调用方兼容。
   */
  async chatStream(request: UnifiedChatRequest): Promise<string> {
    const response = await this.chat(request);
    const streamId = crypto.randomUUID();
    this.pendingStreams.set(streamId, { ...response, is_stream_chunk: true, stream_done: true });
    return streamId;
  }

  /**
   * 监听流式响应分片
   */
  onStreamChunk(streamId: string, callback: (chunk: UnifiedChatResponse) => void): () => void {
    const pending = this.pendingStreams.get(streamId);
    if (pending) {
      this.pendingStreams.delete(streamId);
      queueMicrotask(() => callback(pending));
      return () => {};
    }
    const eventName = `unified-stream-${streamId}`;
    let unlistenFn: (() => void) | null = null;

    listen<UnifiedChatResponse>(eventName, (event: Event<UnifiedChatResponse>) => {
      callback(event.payload);
    }).then((unlisten) => {
      unlistenFn = unlisten;
    });

    return () => {
      unlistenFn?.();
    };
  }

  /**
   * 获取系统状态（经由自然语言 chat）
   */
  async getSystemState(): Promise<UnifiedChatResponse> {
    const res = await chatSend('查看状态');
    const firstAction = res.actions?.[0];
    return {
      response_id: crypto.randomUUID(),
      session_id: '',
      content: res.message,
      payload: firstAction ? { type: 'health_snapshot', data: firstAction.result } : undefined,
      message_type: 'text',
      metadata: emptyMetadata(),
      is_stream_chunk: false,
      stream_done: true,
    };
  }

  /**
   * 创建新会话（经由 session 域）
   */
  async createSession(projectPath?: string): Promise<SessionInfo> {
    const session = await domainCall<{ id: string; name: string; created_at: string; message_count: number; project?: string }>('session', 'create', { name: projectPath });
    return { id: session.id, title: session.name, created_at: session.created_at, updated_at: session.created_at, message_count: session.message_count, project_path: session.project };
  }

  /**
   * 列出所有会话（经由 session 域）
   */
  async listSessions(): Promise<SessionInfo[]> {
    const sessions = await domainCall<Array<{ id: string; name: string; created_at: string; message_count: number; project?: string }>>('session', 'list');
    return sessions.map((s) => ({ id: s.id, title: s.name, created_at: s.created_at, updated_at: s.created_at, message_count: s.message_count, project_path: s.project }));
  }

  /**
   * 删除会话（经由 session 域）
   */
  async deleteSession(sessionId: string): Promise<void> {
    await domainCall('session', 'delete', { id: sessionId });
  }

  /**
   * 执行 CLI 命令（经由 cli 域）
   */
  async execCli(command: string, args?: string[]): Promise<{message: string; success: boolean}> {
    return await domainCall<{message: string; success: boolean}>('cli', 'exec', { command: args ? [command, ...args].join(' ') : command });
  }

  /**
   * 列出 CLI 命令（经由 cli 域）
   */
  async cliList(): Promise<{ name: string; description: string; aliases: string[] }[]> {
    return await domainCall<{ name: string; description: string; aliases: string[] }[]>('cli', 'list');
  }

  /**
   * 便捷方法：发送聊天消息（自动处理上下文）
   */
  async sendMessage(
    input: string,
    options: {
      sessionId?: string;
      mode?: UnifiedRequest['mode'];
      context?: Partial<RequestContext>;
      onStream?: (chunk: UnifiedChatResponse) => void;
    } = {}
  ): Promise<UnifiedChatResponse> {
    const { sessionId, mode = 'chat', context, onStream } = options;

    if (onStream) {
      // 流式模式（chat-first：单分片投递）
      const streamId = await this.chatStream({
        session_id: sessionId,
        input,
        context: context as RequestContext,
        mode,
        stream: true,
      });

      return new Promise<UnifiedChatResponse>((resolve) => {
        const unlisten = this.onStreamChunk(streamId, (chunk) => {
          onStream(chunk);
          if (chunk.stream_done) {
            unlisten();
            resolve(chunk);
          }
        });
      });
    } else {
      // 非流式模式
      return this.chat({
        session_id: sessionId,
        input,
        context: context as RequestContext,
        mode,
        stream: false,
      });
    }
  }
}

// 单例导出
export const unifiedApi = new UnifiedApiClient();

// ========== 独立函数（测试用） ==========

export async function fullCatalog(): Promise<{name: string; backend: string}[]> {
  const plugins = await domainCall<Array<{ id: string; name: string }>>('plugin', 'list');
  return plugins.map((p) => ({ name: p.name || p.id, backend: 'plugin' }));
}

export async function cliList(): Promise<{name: string; backend: string}[]> {
  const commands = await domainCall<Array<{ name: string; description: string; aliases: string[] }>>('cli', 'list');
  return commands.map((c) => ({ name: c.name, backend: 'cli' }));
}

export async function tauriList(): Promise<{name: string; backend: string}[]> {
  return cliList();
}

export async function unifiedCatalog(): Promise<{name: string; backend: string}[]> {
  return cliList();
}

export async function execCli(input: string): Promise<{success: boolean; message: string; exit_code: number; json: unknown}> {
  const result = await domainCall<{message: string; success: boolean}>('cli', 'exec', { command: input });
  return { success: result.success, message: result.message, exit_code: result.success ? 0 : 1, json: null };
}

export async function cliLookup(name: string): Promise<{name: string; backend: string} | null> {
  const commands = await cliList();
  return commands.find((c) => c.name === name) || null;
}

// ========== SolidJS 响应式封装 ==========

import { createSignal, createEffect, onCleanup, type Accessor, type Setter } from 'solid-js';

export function useUnifiedApi() {
  const [initialized, setInitialized] = createSignal(false);
  const [systemState, setSystemState] = createSignal<UnifiedChatResponse | null>(null);
  const [sessions, setSessions] = createSignal<SessionInfo[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // 初始化
  createEffect(async () => {
    try {
      setLoading(true);
      await unifiedApi.init();
      setInitialized(true);
      
      // 获取初始系统状态和会话列表
      const [state, sessionList] = await Promise.all([
        unifiedApi.getSystemState(),
        unifiedApi.listSessions(),
      ]);
      setSystemState(state);
      setSessions(sessionList);
    } catch (e) {
      setError(e instanceof Error ? e.message : '初始化失败');
    } finally {
      setLoading(false);
    }
  });

  // 发送消息
  const sendMessage = async (
    input: string,
    options: {
      sessionId?: string;
      mode?: UnifiedRequest['mode'];
      context?: Partial<RequestContext>;
      onStream?: (chunk: UnifiedChatResponse) => void;
    } = {}
  ) => {
    setError(null);
    try {
      const response = await unifiedApi.sendMessage(input, options);
      return response;
    } catch (e) {
      const msg = e instanceof Error ? e.message : '发送失败';
      setError(msg);
      throw e;
    }
  };

  // 刷新系统状态
  const refreshSystemState = async () => {
    try {
      const state = await unifiedApi.getSystemState();
      setSystemState(state);
    } catch (e) {
      setError(e instanceof Error ? e.message : '刷新状态失败');
    }
  };

  // 刷新会话列表
  const refreshSessions = async () => {
    try {
      const sessionList = await unifiedApi.listSessions();
      setSessions(sessionList);
    } catch (e) {
      setError(e instanceof Error ? e.message : '刷新会话失败');
    }
  };

  // 创建会话
  const createSession = async (projectPath?: string) => {
    try {
      const session = await unifiedApi.createSession(projectPath);
      setSessions((prev: SessionInfo[]) => [session, ...prev]);
      return session;
    } catch (e) {
      setError(e instanceof Error ? e.message : '创建会话失败');
      throw e;
    }
  };

  // 删除会话
  const deleteSession = async (sessionId: string) => {
    try {
      await unifiedApi.deleteSession(sessionId);
      setSessions((prev: SessionInfo[]) => prev.filter(s => s.id !== sessionId));
    } catch (e) {
      setError(e instanceof Error ? e.message : '删除会话失败');
      throw e;
    }
  };

  return {
    // 状态
    initialized,
    systemState,
    sessions,
    loading,
    error,
    // 方法
    sendMessage,
    refreshSystemState,
    refreshSessions,
    createSession,
    deleteSession,
  };
}

// ========== 消息历史管理 Hook ==========

export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  payload?: unknown;
  metadata?: ResponseMetadata;
}

export function useChatSession(sessionId?: string) {
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [streaming, setStreaming] = createSignal(false);

  const addMessage = (message: Omit<ChatMessage, 'id' | 'timestamp'>) => {
    const newMessage: ChatMessage = {
      ...message,
      id: crypto.randomUUID(),
      timestamp: new Date(),
    };
    setMessages((prev: ChatMessage[]) => [...prev, newMessage]);
    return newMessage;
  };

  const updateMessage = (id: string, updates: Partial<ChatMessage>) => {
    setMessages((prev: ChatMessage[]) => prev.map(m => m.id === id ? { ...m, ...updates } : m));
  };

  const sendMessage = async (
    input: string,
    api: UnifiedApiClient,
    options: {
      mode?: UnifiedRequest['mode'];
      context?: Partial<RequestContext>;
    } = {}
  ) => {
    const userMessage = addMessage({ role: 'user', content: input });
    setStreaming(true);

    try {
      // 创建助手消息占位
      const assistantMessage = addMessage({ role: 'assistant', content: '' });
      let fullContent = '';

      const response = await api.sendMessage(input, {
        sessionId,
        mode: options.mode,
        context: options.context,
        onStream: (chunk) => {
          fullContent += chunk.content;
          updateMessage(assistantMessage.id, { 
            content: fullContent,
            payload: chunk.payload,
            metadata: chunk.metadata,
          });
        },
      });

      // 非流式或流式结束后的最终更新
      if (response.content && !response.is_stream_chunk) {
        fullContent = response.content;
        updateMessage(assistantMessage.id, { 
          content: fullContent,
          payload: response.payload,
          metadata: response.metadata,
        });
      }

      setStreaming(false);
      return response;
    } catch (e) {
      setStreaming(false);
      addMessage({ 
        role: 'system', 
        content: `错误: ${e instanceof Error ? e.message : '未知错误'}` 
      });
      throw e;
    }
  };

  const clearMessages = () => {
    setMessages([]);
  };

  return {
    messages,
    streaming,
    sendMessage,
    clearMessages,
  };
}