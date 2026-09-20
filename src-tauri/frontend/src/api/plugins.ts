import { tauriInvoke } from './tauri-bridge'
import type { PluginEvent, PluginStatus } from './types'

/* ════════════════════════════════════════════
   api/plugins.ts — 插件市场（list/install/uninstall/enable/disable/log）
   对应 plugin_cmds.rs
   ════════════════════════════════════════════ */

export function pluginList(): Promise<PluginStatus[]> {
  return tauriInvoke('plugin_list', {})
}

export function pluginInstall(path: string): Promise<PluginStatus> {
  return tauriInvoke('plugin_install', { path })
}

export function pluginUninstall(id: string): Promise<void> {
  return tauriInvoke('plugin_uninstall', { id })
}

export function pluginEnable(id: string): Promise<void> {
  return tauriInvoke('plugin_enable', { id })
}

export function pluginDisable(id: string): Promise<void> {
  return tauriInvoke('plugin_disable', { id })
}

export function pluginGet(id: string): Promise<PluginStatus> {
  return tauriInvoke('plugin_get', { id })
}

export function pluginEventLog(count: number): Promise<PluginEvent[]> {
  return tauriInvoke('plugin_event_log', { count })
}
