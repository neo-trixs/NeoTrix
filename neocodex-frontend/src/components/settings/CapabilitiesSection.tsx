/* ════════════════════════════════════════════
   components/settings/CapabilitiesSection.tsx — 能力展现
   展示 NeoTrix 的核心能力模块和运行状态
   ════════════════════════════════════════════ */
import { createSignal, onMount, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { domain, errText } from '../../api'
import { ExpandIcon, ModelIcon, NetworkIcon, InfoIcon } from './settingsIcons'

interface Capability {
  id: string
  name: string
  description: string
  status: 'active' | 'inactive' | 'error'
  layer: string
  icon: string
}

const CAPABILITIES: Capability[] = [
  { id: 'nt-core', name: 'NT-CORE', description: 'E8 引导者 — 纯逻辑与意识', status: 'active', layer: 'L5 认知层', icon: '🧬' },
  { id: 'nt-mind', name: 'NT-MIND', description: '进化工匠 — SEAL 流水线', status: 'active', layer: 'L5 认知层', icon: '🧠' },
  { id: 'nt-memory', name: 'NT-MEMORY', description: '知识守护者 — SQLite KB', status: 'active', layer: 'L1 行动层', icon: '💾' },
  { id: 'nt-world', name: 'NT-WORLD', description: '虚空探索者 — 爬虫与感知', status: 'active', layer: 'L2 感知层', icon: '🌍' },
  { id: 'nt-act', name: 'NT-ACT', description: '行动执行者 — 工具与编排', status: 'active', layer: 'L1 行动层', icon: '⚡' },
  { id: 'nt-io', name: 'NT-IO', description: '界面使徒 — LLM 与 CLI', status: 'active', layer: 'L1 行动层', icon: '🔌' },
  { id: 'nt-shield', name: 'NT-SHIELD', description: '影卫 — 安全与代理', status: 'active', layer: 'L3 具身层', icon: '🛡️' },
  { id: 'nt-physical', name: 'NT-PHYSICAL', description: '具身骨架 — 传感器与电机', status: 'inactive', layer: 'L3 具身层', icon: '🦾' },
  { id: 'nt-feel', name: 'NT-FEEL', description: '情感中枢 — 情感引擎', status: 'inactive', layer: 'L4 情感层', icon: '❤️' },
  { id: 'nt-meta', name: 'NT-META', description: '元吸收者 — 元认知协调', status: 'active', layer: 'L6 元认知层', icon: '🪞' },
  { id: 'nt-repair', name: 'NT-REPAIR', description: '自愈工程师 — 修复与恢复', status: 'active', layer: 'L6 元认知层', icon: '🔧' },
  { id: 'nt-nexus', name: 'NT-NEXUS', description: '枢纽 — 跨会话记忆', status: 'active', layer: 'L6 元认知层', icon: '🔗' },
]

interface DomainStatus {
  name: string
  actions: number
  description: string
}

export function CapabilitiesSection() {
  const [domains, setDomains] = createSignal<DomainStatus[]>([])
  const [loading, setLoading] = createSignal(true)
  const [error, setError] = createSignal<string | null>(null)

  onMount(async () => {
    try {
      const result = await domain.list()
      setDomains(result.map((d: any) => ({
        name: d.name,
        actions: d.actions?.length ?? 0,
        description: d.description ?? '',
      })))
    } catch (e) {
      setError(errText(e))
    } finally {
      setLoading(false)
    }
  })

  const statusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-emerald-600 bg-emerald-50 border-emerald-200'
      case 'inactive': return 'text-zinc-500 bg-zinc-50 border-zinc-200'
      case 'error': return 'text-red-600 bg-red-50 border-red-200'
      default: return 'text-zinc-500 bg-zinc-50 border-zinc-200'
    }
  }

  const statusLabel = (status: string) => {
    switch (status) {
      case 'active': return '运行中'
      case 'inactive': return '未激活'
      case 'error': return '异常'
      default: return status
    }
  }

  return (
    <div class="space-y-4">
      {/* 能力模块概览 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          能力模块
        </div>
        <div class="ss-card-body">
          <div class="grid grid-cols-2 gap-2">
            <For each={CAPABILITIES}>
              {(cap) => (
                <div class={clsx(
                  'p-3 rounded-xl border transition-colors',
                  cap.status === 'active'
                    ? 'border-emerald-200 bg-emerald-50/50 hover:bg-emerald-50'
                    : 'border-zinc-200 bg-zinc-50/50 opacity-60'
                )}>
                  <div class="flex items-center gap-2 mb-1">
                    <span class="text-base">{cap.icon}</span>
                    <span class="text-sm font-medium text-text-primary">{cap.name}</span>
                    <span class={clsx('text-[10px] px-1.5 py-0.5 rounded-full border ml-auto', statusColor(cap.status))}>
                      {statusLabel(cap.status)}
                    </span>
                  </div>
                  <p class="text-[11px] text-text-secondary leading-relaxed">{cap.description}</p>
                  <p class="text-[10px] text-text-muted mt-1">{cap.layer}</p>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>

      {/* 域插件状态 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <ModelIcon />
          已注册域插件
        </div>
        <div class="ss-card-body">
          <Show when={loading()}>
            <div class="flex items-center justify-center py-8 text-text-muted text-sm">
              加载中...
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>
          <Show when={!loading() && domains().length === 0 && !error()}>
            <div class="py-8 text-center text-xs text-text-muted">
              未检测到域插件
            </div>
          </Show>
          <Show when={domains().length > 0}>
            <div class="space-y-2">
              <For each={domains()}>
                {(d) => (
                  <div class="flex items-center gap-3 p-2 rounded-lg bg-bg-primary/40">
                    <div class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-xs font-medium flex-shrink-0">
                      {d.name.slice(0, 2).toUpperCase()}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="text-sm font-medium text-text-primary truncate">{d.name}</div>
                      <div class="text-[11px] text-text-secondary truncate">{d.description}</div>
                    </div>
                    <span className="text-[10px] text-text-muted font-mono">{d.actions} actions</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>

      {/* 六层架构 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          六层意识-具身架构
        </div>
        <div class="ss-card-body space-y-2">
          {[
            { layer: 'L6', name: '元认知层', modules: 'NT-META · NT-REPAIR · NT-NEXUS', color: 'purple' },
            { layer: 'L5', name: '认知层', modules: 'NT-CORE · NT-MIND', color: 'blue' },
            { layer: 'L4', name: '情感层', modules: 'NT-FEEL', color: 'pink' },
            { layer: 'L3', name: '具身层', modules: 'NT-PHYSICAL · NT-SHIELD · NT-FEEL', color: 'green' },
            { layer: 'L2', name: '感知层', modules: 'NT-WORLD · NT-SENSE', color: 'amber' },
            { layer: 'L1', name: '行动层', modules: 'NT-ACT · NT-IO · NT-MEMORY', color: 'orange' },
          ].map((l) => (
            <div class="flex items-center gap-3 p-2 rounded-lg bg-bg-primary/40">
              <span class={clsx(
                'w-10 h-6 rounded text-[10px] font-bold flex items-center justify-center text-white',
                l.color === 'purple' && 'bg-purple-500',
                l.color === 'blue' && 'bg-blue-500',
                l.color === 'pink' && 'bg-pink-500',
                l.color === 'green' && 'bg-emerald-500',
                l.color === 'amber' && 'bg-amber-500',
                l.color === 'orange' && 'bg-orange-500',
              )}>
                {l.layer}
              </span>
              <div class="flex-1">
                <div class="text-xs font-medium text-text-primary">{l.name}</div>
                <div class="text-[10px] text-text-muted">{l.modules}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
