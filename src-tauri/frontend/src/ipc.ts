export type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
export type UnlistenFn = () => void;

interface TauriInternals {
  invoke?: InvokeFn;
  transformCallback?: (handler: (payload: unknown) => void, once?: boolean) => number;
}

function internals(): TauriInternals | null {
  const g = globalThis as Record<string, unknown>;
  const i = g.__TAURI_INTERNALS__ as TauriInternals | undefined;
  if (i && typeof i.invoke === "function") return i;
  return null;
}

export function getInvoke(): InvokeFn | null {
  const i = internals();
  return i ? i.invoke!.bind(i) : null;
}

export async function invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const fn = getInvoke();
  if (!fn) {
    throw new Error(`IPC unavailable (cmd=${cmd}); running outside Tauri`);
  }
  return (await fn(cmd, args)) as T;
}

export const isTauri = (): boolean => internals() !== null;

/**
 * Generic Tauri event subscription. Real Tauri requires the handler to be a
 * callback id (obtained via `transformCallback`); passing the JS function
 * directly is dropped by JSON serialization and no event would arrive.
 * The Playwright IPC mocks pass either the raw payload or `{ event, payload }`,
 * while real Tauri delivers `{ event, id, payload }` — we normalize so the
 * handler always receives the payload value. Returns an unlisten function.
 */
export async function listen<T = unknown>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  const i = internals();
  if (!i || !i.invoke) {
    throw new Error(`listen unavailable (event=${event}); running outside Tauri`);
  }
  const normalize = (raw: unknown): T => {
    const o = raw as { event?: string; id?: number; payload?: unknown } | null;
    if (o && typeof o === "object" && "payload" in o) return o.payload as T;
    return raw as T;
  };
  const handlerArg: unknown = i.transformCallback
    ? i.transformCallback((raw) => handler(normalize(raw)))
    : (raw: unknown) => handler(normalize(raw));
  return (await i.invoke("plugin:event|listen", {
    event,
    handler: handlerArg,
    target: { kind: "Any" },
  })) as UnlistenFn;
}