export type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
export type UnlistenFn = () => void;

export function getInvoke(): InvokeFn | null {
  const g = globalThis as Record<string, unknown>;
  const internals = g.__TAURI_INTERNALS__ as { invoke?: InvokeFn } | undefined;
  if (internals && typeof internals.invoke === "function") {
    return internals.invoke.bind(internals);
  }
  return null;
}

export async function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const fn = getInvoke();
  if (!fn) {
    throw new Error(`IPC unavailable (cmd=${cmd}); running outside Tauri`);
  }
  return (await fn(cmd, args)) as T;
}

export const isTauri = (): boolean => getInvoke() !== null;

/**
 * Generic Tauri event subscription. Uses the event plugin IPC. In a real
 * Tauri webview this registers the handler; under the Playwright IPC mock it
 * is supplied by the injected shim. Returns an unlisten function.
 */
export function listen<T = unknown>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  const fn = getInvoke();
  if (!fn) {
    throw new Error(`listen unavailable (event=${event}); running outside Tauri`);
  }
  return fn("plugin:event|listen", {
    event,
    handler,
    target: { kind: "Any" },
  }) as Promise<UnlistenFn>;
}