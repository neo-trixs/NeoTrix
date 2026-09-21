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
export async function chatSend(
  message: string,
  sessionId?: string,
): Promise<ChatResponse> {
  return tauriInvoke<ChatResponse>('chat_send', {
    message,
    session_id: sessionId,
  });
}

/**
 * Get help text for available commands.
 */
export async function chatHelp(): Promise<string> {
  return tauriInvoke<string>('chat_help');
}
