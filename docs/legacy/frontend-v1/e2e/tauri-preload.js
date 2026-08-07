(() => {
  console.log('[preload] starting injection');
  const CALLBACKS = new Map();
  const LISTENERS = new Map();
  let cbSeq = 0;

  const DEFAULTS = {
    neocodex_app_version: () => "0.18.0-test",
    neocodex_list_sessions: () => {
      console.log('[preload] neocodex_list_sessions called, returning defaults');
      return [];
    },
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
    console.log('[preload] transformCallback registered id:', id);
    return id;
  };
  I.runCallback = (id, d) => {
    console.log('[preload] runCallback id:', id);
    return CALLBACKS.get(id)?.(d);
  };
  I.unregisterCallback = (id) => CALLBACKS.delete(id);
  I.convertFileSrc = (p) => p;
  I.metadata = { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } };

  I.invoke = (cmd, args) => {
    console.log('[preload] invoke called:', cmd, args);
    window.__TAURI_INVOKE_CALLS__ = window.__TAURI_INVOKE_CALLS__ || [];
    window.__TAURI_INVOKE_CALLS__.push({ cmd, args });

    if (cmd === "plugin:event|listen") {
      const { event, handler } = args;
      console.log('[preload] listen for:', event, 'handler:', handler);
      if (!LISTENERS.has(event)) LISTENERS.set(event, new Map());
      LISTENERS.get(event).set(handler, CALLBACKS.get(handler));
      return Promise.resolve(handler);
    }
    if (cmd === "plugin:event|unlisten") {
      const { event, handler } = args;
      console.log('[preload] unlisten:', event, handler);
      LISTENERS.get(event)?.delete(handler);
      return Promise.resolve(null);
    }
    if (cmd === "plugin:event|emit" || cmd === "plugin:event|emit_to") {
      const { event, payload } = args;
      console.log('[preload] emit:', event, payload);
      for (const fn of LISTENERS.get(event)?.values() ?? []) fn({ event, payload });
      return Promise.resolve(null);
    }

    const mock = (window.__TAURI_MOCKS__ || DEFAULTS)[cmd];
    if (!mock) {
      console.warn('[preload] no mock for:', cmd);
      return Promise.reject(new Error(`[preload] no mock for "${cmd}"`));
    }
    console.log('[preload] using mock for:', cmd);
    return Promise.resolve(mock(args));
  };
  E.unregisterListener = (event, id) => {
    console.log('[preload] unregisterListener:', event, id);
    LISTENERS.get(event)?.delete(id);
  };

  window.__TAURI_MOCKS__ = window.__TAURI_MOCKS__ || {};
  window.__TAURI_DEFAULTS__ = DEFAULTS;
  window.__TAURI_EMIT__ = (event, payload) => {
    console.log('[preload] EMIT:', event, payload);
    for (const fn of LISTENERS.get(event)?.values() ?? []) fn({ event, payload });
  };
  console.log('[preload] injection complete');
})();
