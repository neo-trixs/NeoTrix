/* ════════════════════════════════════════════
   components/settings/AppearanceSection.tsx — 外观：主题/字号/动效/密度
   纯偏好 UI：值 + setter 回调由父组件注入（父组件负责持久化）。
   ════════════════════════════════════════════ */
import { Show } from 'solid-js'
import { clsx } from 'clsx'
import { PaletteIcon, ExpandIcon, DataIcon, InfoIcon } from './settingsIcons'

export type FontSize = 'sm' | 'md' | 'lg'
export type MotionPref = 'full' | 'reduced'
export type DensityPref = 'comfortable' | 'compact'

interface Props {
  fontSizePref: () => FontSize
  motionPref: () => MotionPref
  densityPref: () => DensityPref
  setFontSize: (v: FontSize) => void
  setMotion: (v: MotionPref) => void
  setDensity: (v: DensityPref) => void
}

export function AppearanceSection(props: Props) {
  return (
    <div class="space-y-4">
      <div class="ss-card">
        <div class="ss-card-header">
          <PaletteIcon />
          主题
        </div>
        <div class="ss-card-body">
          <div class="ss-row">
            <div>
              <div class="ss-row-label">雪域白 · 浅橙</div>
              <div class="ss-row-desc">唯一主题 · 极简 Mac 圆角</div>
            </div>
            <span class="text-[10px] text-nt-io-600">✓ 当前</span>
          </div>
        </div>
      </div>

      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          界面字号
        </div>
        <div class="ss-card-body space-y-2">
          {(['sm', 'md', 'lg'] as FontSize[]).map((size) => (
            <button
              class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', props.fontSizePref() === size ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
              onClick={() => props.setFontSize(size)}
              role="radio"
              aria-checked={props.fontSizePref() === size}
            >
              <div class="text-[12.5px] text-text-primary">{size === 'sm' ? '小' : size === 'md' ? '中' : '大'}</div>
              <Show when={props.fontSizePref() === size}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
            </button>
          ))}
        </div>
      </div>

      <div class="ss-card">
        <div class="ss-card-header">
          <DataIcon />
          动效强度
        </div>
        <div class="ss-card-body space-y-2">
          <button
            class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', props.motionPref() === 'full' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
            onClick={() => props.setMotion('full')}
            role="radio"
            aria-checked={props.motionPref() === 'full'}
          >
            <div class="text-[12.5px] text-text-primary">完整动效</div>
            <Show when={props.motionPref() === 'full'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
          </button>
          <button
            class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', props.motionPref() === 'reduced' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
            onClick={() => props.setMotion('reduced')}
            role="radio"
            aria-checked={props.motionPref() === 'reduced'}
          >
            <div class="text-[12.5px] text-text-primary">减弱动效</div>
            <Show when={props.motionPref() === 'reduced'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
          </button>
        </div>
        <div class="px-4 pb-3 -mt-1">
          <p class="text-[10.5px] text-text-muted">减弱后移除无限循环动画，减少视觉干扰</p>
        </div>
      </div>

      <div class="ss-card">
        <div class="ss-card-header">
          <InfoIcon />
          界面密度
        </div>
        <div class="ss-card-body space-y-2">
          <button
            class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', props.densityPref() === 'comfortable' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
            onClick={() => props.setDensity('comfortable')}
            role="radio"
            aria-checked={props.densityPref() === 'comfortable'}
          >
            <div class="text-[12.5px] text-text-primary">舒适</div>
            <Show when={props.densityPref() === 'comfortable'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
          </button>
          <button
            class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', props.densityPref() === 'compact' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
            onClick={() => props.setDensity('compact')}
            role="radio"
            aria-checked={props.densityPref() === 'compact'}
          >
            <div class="text-[12.5px] text-text-primary">紧凑</div>
            <Show when={props.densityPref() === 'compact'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
          </button>
        </div>
        <div class="px-4 pb-3 -mt-1">
          <p class="text-[10.5px] text-text-muted">紧凑模式缩小消息间距与面板内边距，单屏承载更多信息</p>
        </div>
      </div>
    </div>
  )
}