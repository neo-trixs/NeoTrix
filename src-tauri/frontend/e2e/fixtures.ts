import { test as base, expect } from "@playwright/test";

const PRELOAD_SCRIPT = `(function() {
  const CALLBACKS = new Map();
  const LISTENERS = new Map();
  let cbSeq = 0;

  const DEFAULTS = {
    neocodex_app_version: () => "0.18.0-test",
    neocodex_list_sessions: () => [],
    neocodex_list_archived: () => [],
    neocodex_get_session_messages: () => [],
    neocodex_get_side_chat: () => [],
    neocodex_check_update: () => ({ current: "0.18.0", available: false, latest: "", error: null }),
    neocodex_search_files: () => [],
    cmd_diff_changed_files: () => ({ staged: [], unstaged: [], untracked: [] }),
    cmd_diff_unstaged: () => [],
    cmd_diff_staged: () => [],
    cmd_diff_base: () => [],
    cmd_diff_review: () => ({ score: 100, summary: "", critical: 0, warning: 0, info: 0, files: [] }),
  };

  const I = (window.__TAURI_INTERNALS__ = window.__TAURI_INTERNALS__ || {});
  const E = (window.__TAURI_EVENT_PLUGIN_INTERNALS__ = window.__TAURI_EVENT_PLUGIN_INTERNALS__ || {});

  I.transformCallback = (cb, once) => {
    const id = ++cbSeq;
    CALLBACKS.set(id, (d) => { if (once) CALLBACKS.delete(id); cb(d); });
    return id;
  };
  I.runCallback = (id, d) => CALLBACKS.get(id)?.(d);
  I.unregisterCallback = (id) => CALLBACKS.delete(id);
  I.convertFileSrc = (p) => p;
  I.metadata = { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } };

  I.invoke = (cmd, args) => {
    window.__TAURI_INVOKE_CALLS__ = window.__TAURI_INVOKE_CALLS__ || [];
    window.__TAURI_INVOKE_CALLS__.push({ cmd, args });

    if (cmd === "plugin:event|listen") {
      const { event, handler } = args;
      if (!LISTENERS.has(event)) LISTENERS.set(event, new Map());
      LISTENERS.get(event).set(handler, CALLBACKS.get(handler));
      return Promise.resolve(handler);
    }
    if (cmd === "plugin:event|unlisten") {
      const { event, handler } = args;
      LISTENERS.get(event)?.delete(handler);
      return Promise.resolve(null);
    }
    if (cmd === "plugin:event|emit" || cmd === "plugin:event|emit_to") {
      const { event, payload } = args;
      for (const fn of LISTENERS.get(event)?.values() ?? []) fn({ event, payload });
      return Promise.resolve(null);
    }

    const mock = (window.__TAURI_MOCKS__ || {})[cmd] || DEFAULTS[cmd];
    if (!mock) return Promise.reject(new Error('[preload] no mock for "' + cmd + '"'));
    return Promise.resolve(mock(args));
  };
  E.unregisterListener = (event, id) => LISTENERS.get(event)?.delete(id);

  window.__TAURI_MOCKS__ = window.__TAURI_MOCKS__ || {};
  window.__TAURI_DEFAULTS__ = DEFAULTS;
  window.__TAURI_EMIT__ = (event, payload) => {
    for (const fn of LISTENERS.get(event)?.values() ?? []) fn({ event, payload });
  };
})();`

export const test = base.extend({
  page: async ({ page }, use) => {
    await page.addInitScript(PRELOAD_SCRIPT);
    await use(page);
  },
});

export { expect };

/** Inspect the list of (cmd, args) pairs the app invoked through mocked IPC. */
export function invokeCalls(page) {
  return page.evaluate(() => (window as any).__TAURI_INVOKE_CALLS__ ?? []);
}

/**
 * Register a mock handler for a command. The handler's source is embedded into
 * an init script so it runs before the app loads (addInitScript args do NOT
 * preserve functions).
 */
export function mockCommand(page, cmd, handler) {
  const src = handler.toString();
  return page.addInitScript(
    ({ name, src }) => {
      const fn = Function('"use strict"; return (' + src + ');')();
      (window as any).__TAURI_MOCKS__ = (window as any).__TAURI_MOCKS__ || {};
      (window as any).__TAURI_MOCKS__[name] = fn;
    },
    { name: cmd, src }
  );
}

/** Emit a Tauri event the app is listening for (e.g. stream tokens). */
export function emitEvent(page, event, payload) {
  return page.evaluate(
    ([ev, pl]) => (window as any).__TAURI_EMIT__(ev, pl),
    [event, payload]
  );
}
