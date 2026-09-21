/**
 * IM API — 即时通讯渠道管理
 * 
 * 迁移到新架构：状态查询和简单操作走 chatSend。
 */
import { chatSend, extractResult } from './chat'

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
  const response = await chatSend("查看IM状态")
  return extractResult<ImStatus>(response, { channels: [], total_bots: 0, connected_bots: 0, dsh_market_enabled: false })
}

/**
 * 获取所有渠道配置
 */
export async function listChannels(): Promise<ChannelConfig[]> {
  const response = await chatSend("查看IM渠道列表")
  return extractResult<ChannelConfig[]>(response, [])
}

/**
 * 获取单个渠道配置
 */
export async function getChannel(channel: ChannelType): Promise<ChannelConfig> {
  const response = await chatSend(`查看${channel}渠道配置`)
  return extractResult<ChannelConfig>(response, { channel, enabled: false, bots: [], context_enhancement: false, proactive_delivery: false })
}

/**
 * 启用/禁用渠道
 */
export async function toggleChannel(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  const action = enabled ? '启用' : '禁用'
  await chatSend(`${action}${channel}渠道`)
  return { channel, enabled, bots: [], context_enhancement: false, proactive_delivery: false }
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
  const response = await chatSend(`在${params.channel}添加机器人${params.name}`)
  return extractResult<BotConfig>(response, {
    id: `bot-${Date.now()}`,
    channel: params.channel,
    name: params.name,
    credential_type: params.credential_type,
    workspace: params.workspace || null,
    model: params.model || null,
    enabled: true,
    created_at: Date.now(),
  })
}

/**
 * 删除机器人
 */
export async function removeBot(channel: ChannelType, botId: string): Promise<boolean> {
  const response = await chatSend(`从${channel}删除机器人 ${botId}`)
  return response.actions.length > 0
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
  const response = await chatSend(`更新${params.channel}机器人${params.botId}配置`)
  return extractResult<BotConfig>(response, {
    id: params.botId,
    channel: params.channel,
    name: params.name || '',
    credential_type: '',
    workspace: params.workspace || null,
    model: params.model || null,
    enabled: true,
    created_at: Date.now(),
  })
}

/**
 * 设置上下文增强
 */
export async function setContextEnhancement(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  const action = enabled ? '启用' : '禁用'
  await chatSend(`${action}${channel}上下文增强`)
  return { channel, enabled, bots: [], context_enhancement: enabled, proactive_delivery: false }
}

/**
 * 设置主动投递
 */
export async function setProactiveDelivery(channel: ChannelType, enabled: boolean): Promise<ChannelConfig> {
  const action = enabled ? '启用' : '禁用'
  await chatSend(`${action}${channel}主动投递`)
  return { channel, enabled, bots: [], context_enhancement: false, proactive_delivery: enabled }
}

// ═══════════════════════════════════════════════
// DSH 市场模式
// ═══════════════════════════════════════════════

/**
 * 获取 DSH 市场配置
 */
export async function getDshMarketStatus(): Promise<DshMarketConfig> {
  const response = await chatSend("查看DSH市场配置")
  return extractResult<DshMarketConfig>(response, { enabled: false, api_endpoint: '', auth_token: null, sync_enabled: false, last_sync: null })
}

/**
 * 启用/禁用 DSH 市场
 */
export async function toggleDshMarket(enabled: boolean): Promise<DshMarketConfig> {
  const action = enabled ? '启用' : '禁用'
  await chatSend(`${action}DSH市场`)
  return { enabled, api_endpoint: '', auth_token: null, sync_enabled: false, last_sync: null }
}

/**
 * 更新 DSH 市场配置
 */
export async function updateDshMarketConfig(params: {
  api_endpoint?: string
  auth_token?: string
  sync_enabled?: boolean
}): Promise<DshMarketConfig> {
  await chatSend("更新DSH市场配置")
  return { enabled: true, api_endpoint: params.api_endpoint || '', auth_token: params.auth_token || null, sync_enabled: params.sync_enabled ?? false, last_sync: null }
}

/**
 * 从 DSH 市场同步插件
 */
export async function syncDshMarket(): Promise<Record<string, string>> {
  await chatSend("同步DSH市场插件")
  return {}
}
