/**
 * IM API — 即时通讯渠道管理
 * 
 * 通过 Tauri invoke 调用后端 IM 命令。
 * 基于 DSH-IM 的多渠道适配器模式。
 */
import { invoke } from '@tauri-apps/api/core'

/** 渠道类型 */
export type ChannelType = 'wechat' | 'feishu' | 'dingtalk' | 'wecom' | 'qq' | 'slack' | 'telegram' | 'discord' | 'whatsapp'

/** 渠道配置 */
export interface ChannelConfig {
  channel: ChannelType
  enabled: boolean
  bots: BotConfig[]
  context_enhancement: boolean
  proactive_delivery: boolean
}

/** 机器人配置 */
export interface BotConfig {
  id: string
  channel: ChannelType
  name: string
  credential_type: string
  workspace: string | null
  model: string | null
  enabled: boolean
  created_at: number
}

/** IM 系统状态 */
export interface ImStatus {
  channels: ChannelConfig[]
  total_bots: number
  connected_bots: number
  dsh_market_enabled: boolean
}

/** DSH 市场配置 */
export interface DshMarketConfig {
  enabled: boolean
  api_endpoint: string
  auth_token: string | null
  sync_enabled: boolean
  last_sync: number | null
}

/**
 * 获取 IM 系统状态
 */
export async function getImStatus(): Promise<ImStatus> {
  return invoke<ImStatus>('im_status')
}

/**
 * 获取所有渠道配置
 */
export async function listChannels(): Promise<ChannelConfig[]> {
  return invoke<ChannelConfig[]>('im_list_channels')
}

/**
 * 获取单个渠道配置
 */
export async function getChannel(channel: ChannelType): Promise<ChannelConfig> {
  return invoke<ChannelConfig>('im_get_channel', { channel })
}

/**
 * 启用/禁用渠道
 */
export async function toggleChannel(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  return invoke<ChannelConfig>('im_toggle_channel', { channel, enabled })
}

/**
 * 添加机器人
 */
export async function addBot(params: {
  channel: ChannelType
  name: string
  credential_type: string
  workspace?: string
  model?: string
}): Promise<BotConfig> {
  return invoke<BotConfig>('im_add_bot', {
    channel: params.channel,
    name: params.name,
    credential_type: params.credential_type,
    workspace: params.workspace || null,
    model: params.model || null,
  })
}

/**
 * 删除机器人
 */
export async function removeBot(channel: ChannelType, botId: string): Promise<boolean> {
  return invoke<boolean>('im_remove_bot', { channel, botId })
}

/**
 * 更新机器人配置
 */
export async function updateBot(params: {
  channel: ChannelType
  botId: string
  name?: string
  workspace?: string
  model?: string
}): Promise<BotConfig> {
  return invoke<BotConfig>('im_update_bot', {
    channel: params.channel,
    botId: params.botId,
    name: params.name || null,
    workspace: params.workspace || null,
    model: params.model || null,
  })
}

/**
 * 设置上下文增强
 */
export async function setContextEnhancement(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  return invoke<ChannelConfig>('im_set_context_enhancement', { channel, enabled })
}

/**
 * 设置主动投递
 */
export async function setProactiveDelivery(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  return invoke<ChannelConfig>('im_set_proactive_delivery', { channel, enabled })
}

// ═══════════════════════════════════════════════
// DSH 市场模式
// ═══════════════════════════════════════════════

/**
 * 获取 DSH 市场配置
 */
export async function getDshMarketStatus(): Promise<DshMarketConfig> {
  return invoke<DshMarketConfig>('im_dsh_market_status')
}

/**
 * 启用/禁用 DSH 市场
 */
export async function toggleDshMarket(enabled: boolean): Promise<DshMarketConfig> {
  return invoke<DshMarketConfig>('im_dsh_market_toggle', { enabled })
}

/**
 * 更新 DSH 市场配置
 */
export async function updateDshMarketConfig(params: {
  api_endpoint?: string
  auth_token?: string
  sync_enabled?: boolean
}): Promise<DshMarketConfig> {
  return invoke<DshMarketConfig>('im_dsh_market_config', {
    api_endpoint: params.api_endpoint || null,
    auth_token: params.auth_token || null,
    sync_enabled: params.sync_enabled ?? null,
  })
}

/**
 * 从 DSH 市场同步插件
 */
export async function syncDshMarket(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('im_dsh_market_sync')
}
