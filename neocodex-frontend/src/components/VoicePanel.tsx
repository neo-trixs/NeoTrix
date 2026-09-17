import { createSignal, createEffect, onMount, onCleanup, Show, For, Switch, Match } from 'solid-js'
import { clsx } from 'clsx'
import { Mic, MicOff, Volume2, VolumeX, Settings, Loader2, Circle, Trash2 } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   VoicePanel — 语音输入/输出面板
   
   融合产品模式：
   - ChatGPT Desktop: Whisper + TTS 全语音交互
   - Claude Desktop: 语音输入支持
   - Cursor: 语音命令（规划中）
   
   NeoTrix 特有：
   - 与意识状态联动（φ 高时自动启用语音反馈）
   - SEAL 情感驱动 TTS 语调
   ════════════════════════════════════════════════════════════ */

export type VoiceState = 'idle' | 'listening' | 'processing' | 'speaking' | 'error'

export interface VoicePanelProps {
  state: () => VoiceState
  onToggleListen: () => void
  onToggleTts: () => void
  ttsEnabled: () => boolean
  onClearHistory: () => void
  transcript?: () => string | null
  error?: () => string | null
  phi?: () => number | null
  compact?: boolean
}

const STATE_LABELS: Record<VoiceState, string> = {
  idle: '待命',
  listening: '聆听中',
  processing: '处理中',
  speaking: '播放中',
  error: '错误',
}

const STATE_COLORS: Record<VoiceState, string> = {
  idle: 'text-text-muted',
  listening: 'text-red-500',
  processing: 'text-amber-500',
  speaking: 'text-blue-500',
  error: 'text-red-500',
}

export function VoicePanel(props: VoicePanelProps) {
  const state = () => props.state()

  // 脉冲动画（listening 时）
  let pulseRef: HTMLDivElement | undefined

  if (props.compact) {
    return (
      <div class="flex items-center gap-2">
        {/* 主麦克风按钮 */}
        <button
          class={clsx(
            'relative p-2 rounded-full transition-all duration-200',
            state() === 'listening'
              ? 'bg-red-500/15 text-red-500 ring-2 ring-red-400/30'
              : state() === 'speaking'
              ? 'bg-blue-500/15 text-blue-500'
              : 'bg-white/40 text-text-muted hover:bg-white/60 hover:text-text-primary'
          )}
          onClick={props.onToggleListen}
          title={state() === 'listening' ? '停止聆听' : '开始语音'}
        >
          {/* 脉冲环 */}
          <Show when={state() === 'listening'}>
            <div class="absolute inset-0 rounded-full bg-red-400/20 animate-ping" />
          </Show>
          <Switch>
            <Match when={state() === 'listening'}><Circle class={clsx('w-4 h-4', STATE_COLORS[state()])} /></Match>
            <Match when={state() === 'processing'}><Loader2 class={clsx('w-4 h-4 animate-spin', STATE_COLORS[state()])} /></Match>
            <Match when={state() === 'speaking'}><Volume2 class={clsx('w-4 h-4', STATE_COLORS[state()])} /></Match>
            <Match when={state() === 'error'}><MicOff class={clsx('w-4 h-4', STATE_COLORS[state()])} /></Match>
            <Match when={true}><Mic class={clsx('w-4 h-4', STATE_COLORS[state()])} /></Match>
          </Switch>
        </button>

        {/* TTS 开关 */}
        <button
          class={clsx(
            'p-1.5 rounded-lg transition-colors',
            props.ttsEnabled()
              ? 'text-blue-500 bg-blue-500/10 hover:bg-blue-500/20'
              : 'text-text-muted hover:text-text-primary hover:bg-white/60'
          )}
          onClick={props.onToggleTts}
          title={props.ttsEnabled() ? '关闭语音播报' : '开启语音播报'}
        >
          {props.ttsEnabled() ? <Volume2 class="w-3.5 h-3.5" /> : <VolumeX class="w-3.5 h-3.5" />}
        </button>
      </div>
    )
  }

  return (
    <div class="p-3 rounded-xl bg-white/30 border border-border-primary/40">
      {/* 头部 */}
      <div class="flex items-center justify-between mb-3">
        <div class="flex items-center gap-2">
          <Mic class="w-4 h-4 text-text-muted" />
          <span class="text-[12px] font-medium text-text-primary">语音控制</span>
        </div>
        <button
          class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
          onClick={props.onClearHistory}
          title="清除语音记录"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
      </div>

      {/* 主麦克风按钮（大） */}
      <div class="flex justify-center mb-4">
        <button
          class={clsx(
            'relative w-16 h-16 rounded-full flex items-center justify-center transition-all duration-300',
            state() === 'listening'
              ? 'bg-red-500 text-white shadow-lg shadow-red-500/25'
              : state() === 'speaking'
              ? 'bg-blue-500 text-white shadow-lg shadow-blue-500/25'
              : state() === 'processing'
              ? 'bg-amber-500 text-white shadow-lg shadow-amber-500/25'
              : 'bg-white/60 text-text-muted hover:bg-white/80 hover:text-text-primary border border-black/5'
          )}
          onClick={props.onToggleListen}
          title={state() === 'listening' ? '停止聆听' : '开始语音'}
        >
          {/* 脉冲环（listening 时） */}
          <Show when={state() === 'listening'}>
            <div class="absolute inset-0 rounded-full bg-red-400/30 animate-ping" style={{ 'animation-duration': '1.5s' }} />
            <div class="absolute inset-0 rounded-full bg-red-400/20 animate-ping" style={{ 'animation-duration': '2s', 'animation-delay': '0.5s' }} />
          </Show>
          <Switch>
            <Match when={state() === 'listening'}><Circle class="w-6 h-6" /></Match>
            <Match when={state() === 'processing'}><Loader2 class="w-6 h-6 animate-spin" /></Match>
            <Match when={state() === 'speaking'}><Volume2 class="w-6 h-6" /></Match>
            <Match when={state() === 'error'}><MicOff class="w-6 h-6" /></Match>
            <Match when={true}><Mic class="w-6 h-6" /></Match>
          </Switch>
        </button>
      </div>

      {/* 状态标签 */}
      <div class="text-center mb-3">
        <span class={clsx('text-[11px] font-medium', STATE_COLORS[state()])}>
          {STATE_LABELS[state()]}
        </span>
      </div>

      {/* 实时转写 */}
      <Show when={props.transcript?.()}>
        <div class="p-2 rounded-lg bg-white/40 border border-black/5 mb-3">
          <div class="text-[10px] text-text-muted mb-1">实时转写</div>
          <div class="text-[12px] text-text-primary">{props.transcript?.()}</div>
        </div>
      </Show>

      {/* 错误信息 */}
      <Show when={props.error?.()}>
        <div class="p-2 rounded-lg bg-red-50 border border-red-200/50 mb-3">
          <div class="text-[11px] text-red-600">{props.error?.()}</div>
        </div>
      </Show>

      {/* TTS 开关 */}
      <div class="flex items-center justify-between">
        <span class="text-[11px] text-text-muted">语音播报</span>
        <button
          class={clsx(
            'w-10 h-5 rounded-full transition-colors relative',
            props.ttsEnabled() ? 'bg-blue-500' : 'bg-zinc-200'
          )}
          onClick={props.onToggleTts}
          role="switch"
          aria-checked={props.ttsEnabled()}
        >
          <div class={clsx(
            'absolute top-0.5 w-4 h-4 rounded-full bg-white shadow-sm transition-transform',
            props.ttsEnabled() ? 'translate-x-5' : 'translate-x-0.5'
          )} />
        </button>
      </div>

      {/* φ 意识状态联动 */}
      <Show when={props.phi?.() !== null && props.phi?.() !== undefined && props.phi()! > 0.7}>
        <div class="mt-3 p-2 rounded-lg bg-purple-50/50 border border-purple-200/30">
          <div class="text-[10px] text-purple-600">
            ✦ 意识连贯度高，建议启用语音反馈
          </div>
        </div>
      </Show>
    </div>
  )
}
