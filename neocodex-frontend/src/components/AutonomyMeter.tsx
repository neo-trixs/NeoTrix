import { For, type JSX } from 'solid-js'
import { clsx } from 'clsx'

export interface AutonomyMeterProps {
  /** 自治等级 0-3（0 手动 / 1 规划 / 2 自动 / 3 接受编辑） */
  level: () => number
  /** 当前权限模式短标签 */
  mode: () => string
  /** 审批通过率 0-100（渐进授权：依据历史批准率衡量可信度） */
  rate: () => number
}

const SEG_COUNT = 4

/**
 * AutonomyMeter — 渐进授权可视层（对标 2026 Agent UX：自治等级需可见、可审计）。
 * 展示当前自治等级 + 审批通过率，让用户随时看清「OS 现在有多少自主权」。
 */
export function AutonomyMeter(props: AutonomyMeterProps): JSX.Element {
  return (
    <div
      class="autonomy-meter"
      title={`自治等级 L${props.level()} · 审批通过率 ${props.rate()}%`}
      role="status"
      aria-label={`自治等级 ${props.level()}，审批通过率 ${props.rate()}%`}
    >
      <span class="autonomy-meter__label">{props.mode()}</span>
      <div class="autonomy-meter__bars" aria-hidden="true">
        <For each={Array.from({ length: SEG_COUNT })}>
          {(_, i) => (
            <span class={clsx('autonomy-meter__seg', i() < props.level() && 'autonomy-meter__seg--on')} />
          )}
        </For>
      </div>
      <span class="autonomy-meter__rate">{props.rate()}%</span>
    </div>
  )
}
