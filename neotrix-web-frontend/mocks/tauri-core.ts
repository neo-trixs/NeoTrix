export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "brain_stats":
    case "get_brain_stats": {
      try {
        const res = await fetch("/api/stats");
        const data = await res.json();
        return {
          iteration: data.stage ?? 0,
          absorb_count: 0,
          capability_sum: data.energy ?? 0,
          memory_count: 0,
          engine_active: false,
          capability_vector: [],
          dimension_names: [],
        } as T;
      } catch {
        return {
          iteration: 0,
          absorb_count: 0,
          capability_sum: 0,
          memory_count: 0,
          engine_active: false,
          capability_vector: [],
          dimension_names: [],
        } as T;
      }
    }

    case "agent_reason": {
      try {
        const res = await fetch("/api/reason", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ prompt: ((args as any)?.req?.prompt as string) ?? "" }),
        });
        const data = await res.json();
        return { output: data.response ?? "", success: true } as T;
      } catch {
        return { output: "⚠ neotrix-web backend not reachable", success: false } as T;
      }
    }

    case "get_pending_permissions":
      return [] as T;

    case "session_list":
      return [] as T;

    case "read_dir_recursive":
      return [] as T;

    case "read_file":
      return "" as T;

    case "search_knowledge":
      return "[]" as T;

    case "get_current_provider":
    case "save_provider_config":
    case "test_provider":
    case "tool_search":
    case "tool_execute":
    case "browser_open":
    case "browser_close":
    case "get_chain_stats":
    case "get_user_avatar":
    case "get_distillation_flow":
    case "distill_message":
    case "get_identity":
    case "set_user_identity":
    case "brain_write_back":
    case "read_consciousness_response":
    case "proxy_status":
    case "proxy_set_mode":
    case "proxy_start_daemon":
    case "proxy_stop_daemon":
    case "plugin_list":
    case "plugin_load":
    case "plugin_unload":
    case "plugin_uninstall":
    case "plugin_install_from_zip":
    case "plugin_get_info":
    case "plugin_write_data":
    case "plugin_read_data":
      return null as T;

    default: {
      console.warn(`[web-invoke] unhandled command: ${cmd}`, args);
      return null as T;
    }
  }
}

export async function convertFileSrc(path: string): Promise<string> {
  return path;
}
