import { invoke } from '@tauri-apps/api/core';

// ─── Types ───────────────────────────────────────────────
export interface RequestState {
  pendingCount: number;
  lastError: string | null;
  isOnline: boolean;
}

export interface RequestPolicy {
  timeoutMs: number;
  maxRetries: number;
  baseBackoffMs: number;
  maxBackoffMs: number;
}

const DEFAULT_POLICY: RequestPolicy = {
  timeoutMs: 30_000,
  maxRetries: 3,
  baseBackoffMs: 1_000,
  maxBackoffMs: 10_000,
};

// ─── Reactive State ──────────────────────────────────────
class RequestCenter {
  private state: RequestState = {
    pendingCount: 0,
    lastError: null,
    isOnline: navigator.onLine,
  };
  private listeners: Set<(s: RequestState) => void> = new Set();

  constructor() {
    window.addEventListener('online', () => this.updateOnline(true));
    window.addEventListener('offline', () => this.updateOnline(false));
  }

  private updateOnline(online: boolean) {
    this.state = { ...this.state, isOnline: online };
    this.notify();
  }

  subscribe(listener: (s: RequestState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify() {
    this.listeners.forEach((l) => l({ ...this.state }));
  }

  getState(): RequestState {
    return { ...this.state };
  }

  // ─── Core Request Method ─────────────────────────────
  async invoke<T>(
    command: string,
    args?: Record<string, unknown>,
    policy: Partial<RequestPolicy> = {},
  ): Promise<T> {
    const p = { ...DEFAULT_POLICY, ...policy };
    this.state = { ...this.state, pendingCount: this.state.pendingCount + 1 };
    this.notify();

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= p.maxRetries; attempt++) {
      try {
        const controller = new AbortController();
        const timer = setTimeout(() => controller.abort(), p.timeoutMs);

        const result = await Promise.race([
          invoke<T>(command, args),
          new Promise<never>((_, reject) => {
            controller.signal.addEventListener('abort', () =>
              reject(new Error(`Request timeout after ${p.timeoutMs}ms`))
            );
          }),
        ]);

        clearTimeout(timer);
        this.state = {
          ...this.state,
          pendingCount: Math.max(0, this.state.pendingCount - 1),
          lastError: null,
        };
        this.notify();
        return result;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));

        // Don't retry on non-retryable errors
        if (this.isNonRetryable(lastError)) break;

        // Exponential backoff with jitter
        if (attempt < p.maxRetries) {
          const backoff = Math.min(
            p.baseBackoffMs * 2 ** attempt + Math.random() * 100,
            p.maxBackoffMs,
          );
          await new Promise((r) => setTimeout(r, backoff));
        }
      }
    }

    this.state = {
      ...this.state,
      pendingCount: Math.max(0, this.state.pendingCount - 1),
      lastError: lastError?.message ?? 'Unknown error',
    };
    this.notify();
    throw lastError;
  }

  private isNonRetryable(err: Error): boolean {
    const msg = err.message.toLowerCase();
    return (
      msg.includes('permission denied') ||
      msg.includes('not found') ||
      msg.includes('validation') ||
      msg.includes('already exists')
    );
  }
}

export const requestCenter = new RequestCenter();

// ─── Convenience Wrapper ─────────────────────────────────
export async function invokeWithRetry<T>(
  command: string,
  args?: Record<string, unknown>,
  policy?: Partial<RequestPolicy>,
): Promise<T> {
  return requestCenter.invoke<T>(command, args, policy);
}

// ─── Classified Error Types ──────────────────────────────
export type ErrorType =
  | 'permission_denied'
  | 'not_found'
  | 'validation_error'
  | 'rate_limited'
  | 'network_error'
  | 'timeout'
  | 'unknown';

export interface ClassifiedError {
  type: ErrorType;
  message: string;
  original?: Error;
}

export function classifyError(err: Error): ClassifiedError {
  const msg = err.message.toLowerCase();
  if (msg.includes('permission denied')) return { type: 'permission_denied', message: err.message, original: err };
  if (msg.includes('not found') || msg.includes('404')) return { type: 'not_found', message: err.message, original: err };
  if (msg.includes('validation') || msg.includes('invalid')) return { type: 'validation_error', message: err.message, original: err };
  if (msg.includes('429') || msg.includes('rate limit')) return { type: 'rate_limited', message: err.message, original: err };
  if (msg.includes('timeout') || msg.includes('abort')) return { type: 'timeout', message: err.message, original: err };
  if (msg.includes('network') || msg.includes('fetch') || msg.includes('connection')) return { type: 'network_error', message: err.message, original: err };
  return { type: 'unknown', message: err.message, original: err };
}
