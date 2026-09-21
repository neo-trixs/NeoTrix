import { tauriInvoke } from './tauri-bridge';

/**
 * Chat response from the backend.
 */
export interface ChatResponse {
  /** Natural language reply */
  message: string;
  /** Actions that were performed */
  actions: ChatAction[];
  /** Updated state (optional) */
  state?: Record<string, unknown>;
}

/**
 * Record of an action performed.
 */
export interface ChatAction {
  /** Domain that was called */
  domain: string;
  /** Action that was performed */
  action: string;
  /** Action result */
  result: unknown;
}

/**
 * Send a natural language message to NeoTrix.
 * This is the single entry point for all user interactions.
 *
 * @example
 * ```ts
 * const response = await chatSend("添加代理 http://1.2.3.4:8080");
 * console.log(response.message); // "已添加代理: http://1.2.3.4:8080"
 * ```
 */
/**
 * Backend envelope for chat_send (mirrors Rust IpcResponse<ChatResponse>).
 * Unwrapped here so callers keep working with plain ChatResponse.
 */
interface ChatEnvelope {
  ok: boolean;
  error?: { code: string; message: string };
  data?: ChatResponse;
}

export async function chatSend(
  message: string,
  sessionId?: string,
): Promise<ChatResponse> {
  const env = await tauriInvoke<ChatEnvelope>('chat_send', {
    message,
    session_id: sessionId,
  });
  if (!env.ok || !env.data) {
    const err = env.error ?? { code: 'UNKNOWN', message: 'Unknown chat error' };
    throw new Error(`[${err.code}] ${err.message}`);
  }
  return env.data;
}

/**
 * Get help text for available commands.
 */
export async function chatHelp(): Promise<string> {
  return tauriInvoke<string>('chat_help');
}

/**
 * Extract typed result from a ChatResponse.
 *
 * Backend contract: human-readable text lives in `message`,
 * structured data lives in `actions[0].result`.
 * Falls back to `fallback` when no action result is present
 * (e.g. unparseable intent) instead of throwing.
 */
export function extractResult<T>(response: ChatResponse, fallback: T): T {
  const result = response.actions?.[0]?.result as T | undefined;
  if (result === undefined || result === null) return fallback;
  return result;
}
