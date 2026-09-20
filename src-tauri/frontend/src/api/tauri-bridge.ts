import { invoke, type InvokeArgs } from "@tauri-apps/api/core";

/**
 * Centralized Tauri IPC bridge.
 * All frontend → backend calls should go through this function.
 * Provides typed responses, error logging, and future API migration path.
 */
export async function tauriInvoke<T>(
  cmd: string,
  args?: InvokeArgs,
): Promise<T> {
  if (import.meta.env.DEV) {
    console.debug(`[IPC] ${cmd}`, args);
  }

  try {
    const result = await invoke<T>(cmd, args);

    if (import.meta.env.DEV) {
      console.debug(`[IPC] ${cmd} → OK`);
    }

    return result;
  } catch (error) {
    const err = error as { code?: string; message?: string; recoverable?: boolean };

    console.error(`[IPC] ${cmd} → ERROR`, {
      code: err?.code ?? "UNKNOWN",
      message: err?.message ?? String(error),
      recoverable: err?.recoverable ?? false,
    });

    throw error;
  }
}

/**
 * Type-safe IPC call with error recovery.
 * Returns [result, null] on success, [null, error] on failure.
 */
export async function tauriInvokeSafe<T>(
  cmd: string,
  args?: InvokeArgs,
): Promise<[T, null] | [null, { code: string; message: string }]> {
  try {
    const result = await tauriInvoke<T>(cmd, args);
    return [result, null];
  } catch (error) {
    const err = error as { code?: string; message?: string };
    return [null, { code: err?.code ?? "UNKNOWN", message: err?.message ?? String(error) }];
  }
}
