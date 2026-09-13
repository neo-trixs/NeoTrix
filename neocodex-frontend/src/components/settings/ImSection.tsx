/**
 * IM Section — 即时通讯渠道管理
 * 
 * 基于 DSH-IM 的多渠道适配器模式，为 NeoTrix 提供 IM 渠道管理。
 * 支持 9 个内置渠道：微信、飞书、钉钉、企业微信、QQ、Slack、Telegram、Discord、WhatsApp
 */
import { createSignal, For, Show, onMount } from 'solid-js'
import { clsx } from 'clsx'
import { im } from '../../api'
import type { ChannelConfig } from '../../api/im'

/** 渠道类型 */
type ChannelType = 'wechat' | 'feishu' | 'dingtalk' | 'wecom' | 'qq' | 'slack' | 'telegram' | 'discord' | 'whatsapp'

/** 渠道元数据 */
const CHANNEL_META: Record<ChannelType, { name: string; icon: string; color: string }> = {
  wechat:    { name: '微信',      icon: '💬', color: '#07C160' },
  feishu:    { name: '飞书',      icon: '🐦', color: '#3370FF' },
  dingtalk:  { name: '钉钉',      icon: '📌', color: '#1677FF' },
  wecom:     { name: '企业微信',   icon: '🏢', color: '#3370FF' },
  qq:        { name: 'QQ',        icon: '🐧', color: '#1EBAFC' },
  slack:     { name: 'Slack',     icon: '💼', color: '#4A154B' },
  telegram:  { name: 'Telegram',  icon: '✈️', color: '#26A5E4' },
  discord:   { name: 'Discord',   icon: '🎮', color: '#5865F2' },
  whatsapp:  { name: 'WhatsApp',  icon: '📱', color: '#25D366' },
}

const ALL_CHANNELS: ChannelType[] = ['wechat', 'feishu', 'dingtalk', 'wecom', 'qq', 'slack', 'telegram', 'discord', 'whatsapp']

export function ImSection() {
  const [channels, setChannels] = createSignal<ChannelConfig[]>([])
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal<string | null>(null)

  const loadChannels = async () => {
    try {
      setLoading(true)
      const data = await im.listChannels()
      setChannels(data)
      setError(null)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(loadChannels)

  const toggleChannel = async (channel: ChannelType, enabled: boolean) => {
    try {
      await im.toggleChannel(channel, enabled)
      await loadChannels()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    }
  }

  const getChannelStatus = (channel: ChannelType): ChannelConfig | undefined => {
    return channels().find(c => c.channel === channel)
  }

  return (
    <div class="space-y-4">
      {/* IM 渠道列表 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <span class="text-base mr-2">💬</span>
          即时通讯渠道
        </div>
        <div class="ss-card-body">
          <Show when={!loading()}>
            <Show when={error()}>
              <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg mb-4">{error()}</div>
            </Show>
            <div class="grid grid-cols-3 gap-3">
              <For each={ALL_CHANNELS}>
                {(channel) => {
                  const meta = CHANNEL_META[channel]
                  const config = getChannelStatus(channel)
                  const enabled = config?.enabled ?? false
                  return (
                    <div class={clsx(
                      'p-3 rounded-xl border transition-all cursor-pointer',
                      enabled
                        ? 'border-emerald-200 bg-emerald-50/50 hover:bg-emerald-50'
                        : 'border-zinc-200 bg-zinc-50/50 opacity-60'
                    )} onClick={() => toggleChannel(channel, !enabled)}>
                      <div class="flex items-center gap-2 mb-1">
                        <span class="text-xl">{meta.icon}</span>
                        <span class="text-sm font-medium text-text-primary">{meta.name}</span>
                      </div>
                      <div class="text-[10px] text-text-muted">
                        {enabled ? '已启用' : '未启用'}
                      </div>
                      <div class="mt-2 flex items-center gap-1">
                        <span class={clsx(
                          'w-2 h-2 rounded-full',
                          enabled ? 'bg-emerald-500' : 'bg-zinc-300'
                        )} />
                        <span class="text-[10px] text-text-muted">
                          {config?.bots?.length ?? 0} 机器人
                        </span>
                      </div>
                    </div>
                  )
                }}
              </For>
            </div>
          </Show>
          <Show when={loading()}>
            <div class="py-8 text-center text-text-muted text-sm">加载中...</div>
          </Show>
        </div>
      </div>

      {/* DSH 市场模式 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <span class="text-base mr-2">🏪</span>
          DSH 市场模式
        </div>
        <div class="ss-card-body space-y-3">
          <div class="flex items-center justify-between">
            <div>
              <div class="text-sm font-medium text-text-primary">启用 DSH 市场</div>
              <div class="text-[11px] text-text-secondary">连接 DSH-IM 插件市场</div>
            </div>
            <button class="ss-toggle" onClick={() => {/* TODO: toggle */}}>
              <div class="ss-toggle-dot" />
            </button>
          </div>
          <div class="text-[10px] text-text-muted">
            DSH 市场提供多渠道机器人模板和插件，支持一键部署到各 IM 平台。
          </div>
        </div>
      </div>
    </div>
  )
}
