import { call } from './domain'
import type { PluginEvent, PluginStatus } from './types'

/* ════════════════════════════════════════════
   api/plugins.ts — 插件市场（list/install/uninstall/enable/disable/log）
   对应 plugin domain plugin (domain_call)
   ════════════════════════════════════════════ */

export function pluginList(): Promise<PluginStatus[]> {
  return call<PluginStatus[]>('plugin', 'list')
}

export function pluginInstall(path: string): Promise<PluginStatus> {
  return call<PluginStatus>('plugin', 'install', { path })
}

export function pluginUninstall(id: string): Promise<void> {
  return call<void>('plugin', 'uninstall', { id })
}

export function pluginEnable(id: string): Promise<void> {
  return call<void>('plugin', 'enable', { id })
}

export function pluginDisable(id: string): Promise<void> {
  return call<void>('plugin', 'disable', { id })
}

export function pluginGet(id: string): Promise<PluginStatus> {
  return call<PluginStatus>('plugin', 'get', { id })
}

export function pluginEventLog(count: number): Promise<PluginEvent[]> {
  return call<PluginEvent[]>('plugin', 'event_log', { count })
}
