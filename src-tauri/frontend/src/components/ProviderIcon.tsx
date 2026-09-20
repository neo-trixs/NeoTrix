import { clsx } from 'clsx'
import { Show, For } from 'solid-js'

/* ════════════════════════════════════════════
   ProviderIcon — NeoTrix 自主几何标识系统
   视觉语言：E8 Hexagram 六边形基座 + 分类几何纹理
   - 本地(自我主体)：实心六边形 + 内嵌菱形
   - 代理(自定义中转)：六边形 + 同心环
   - 云端(第三方)：六边形 + 点阵网络
   色彩统一走 NeoTrix 设计 token（非厂商品牌色）
   全站 ProviderSelector / SettingsModal / ModelSwitcher 共用
   ════════════════════════════════════════════ */

/* 六边形路径（pointy-top，外接圆半径 r，中心 cx,cy） */
function hexPath(cx: number, cy: number, r: number): string {
  const pts: string[] = []
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30)
    pts.push(`${cx + r * Math.cos(angle)},${cy + r * Math.sin(angle)}`)
  }
  return `M${pts.join('L')}Z`
}

/* 部署分类 → 几何纹理 + 色彩语义 */
const CAT_STYLE: Record<string, { fill: string; accent: string; label: string; desc: string }> = {
  local:   { fill: 'var(--nt-color-core-500, #8b5cf6)', accent: 'var(--nt-color-core-400, #a78bfa)', label: '本地', desc: '数据不出设备' },
  proxy:   { fill: 'var(--nt-color-act-500, #f59e0b)',    accent: 'var(--nt-color-act-400, #fbbf24)',    label: '代理', desc: '自定义中转' },
  cloud:   { fill: 'var(--nt-color-io-500, #3b82f6)',     accent: 'var(--nt-color-io-400, #60a5fa)',     label: '云端', desc: '第三方 API' },
  unknown: { fill: 'var(--nt-color-text-muted, #9ca3af)', accent: 'var(--nt-color-text-muted, #6b7280)', label: '未知', desc: '' },
}

/* 型号尺寸映射 */
const SIZE = {
  sm: { svg: 24, hexR: 9, inner: 5, stroke: 1.5, fontSize: 7 },
  md: { svg: 32, hexR: 13, inner: 7, stroke: 2,   fontSize: 10 },
} as const

function HexagonInner(props: { category: string; cx: number; cy: number; r: number }) {
  const cat = () => props.category ?? 'unknown'
  const cx = () => props.cx
  const cy = () => props.cy
  const r = () => props.r

  return (
    <Show
      when={cat() === 'local'}
      fallback={
        <Show
          when={cat() === 'proxy'}
          fallback={
            /* cloud / unknown: 点阵网络 */
            <g opacity="0.7">
              <For each={[-1, 0, 1]}>
                {(dx) => (
                  <For each={[-1, 0, 1]}>
                    {(dy) => {
                      const dist = Math.sqrt(dx * dx + dy * dy)
                      if (dist > 1.2) return null
                      return (
                        <circle
                          cx={cx() + dx * r() * 0.5}
                          cy={cy() + dy * r() * 0.5}
                          r={r() * 0.12}
                          fill="white"
                          opacity={0.9 - dist * 0.3}
                        />
                      )
                    }}
                  </For>
                )}
              </For>
            </g>
          }
        >
          {/* proxy: 同心环（中继/转发语义） */}
          <g opacity="0.6">
            <circle cx={cx()} cy={cy()} r={r() * 0.7} fill="none" stroke="white" stroke-width="1" opacity="0.5" />
            <circle cx={cx()} cy={cy()} r={r() * 0.4} fill="none" stroke="white" stroke-width="1" opacity="0.7" />
            <circle cx={cx()} cy={cy()} r={r() * 0.15} fill="white" opacity="0.9" />
          </g>
        </Show>
      }
    >
      {/* local: 内嵌菱形（自我主体/自包含语义） */}
      <g opacity="0.7">
        <path
          d={`M${cx()} ${cy() - r() * 0.6} L${cx() + r() * 0.6} ${cy()} L${cx()} ${cy() + r() * 0.6} L${cx() - r() * 0.6} ${cy()} Z`}
          fill="white"
          opacity="0.5"
        />
        <path
          d={`M${cx()} ${cy() - r() * 0.35} L${cx() + r() * 0.35} ${cy()} L${cx()} ${cy() + r() * 0.35} L${cx() - r() * 0.35} ${cy()} Z`}
          fill="white"
          opacity="0.8"
        />
      </g>
    </Show>
  )
}

export function ProviderIcon(props: { name: string; size?: 'sm' | 'md'; className?: string; category?: string }) {
  const s = () => SIZE[props.size ?? 'md']
  const cat = () => CAT_STYLE[props.category ?? 'unknown'] ?? CAT_STYLE.unknown
  const cx = () => s().svg / 2
  const cy = () => s().svg / 2

  return (
    <span
      class={clsx(
        'flex items-center justify-center flex-shrink-0 select-none',
        props.className,
      )}
      aria-hidden="true"
      title={cat().desc ? `${cat().label} · ${cat().desc}` : cat().label}
    >
      <svg
        width={s().svg}
        height={s().svg}
        viewBox={`0 0 ${s().svg} ${s().svg}`}
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* 外层六边形 */}
        <path
          d={hexPath(cx(), cy(), s().hexR)}
          fill={cat().fill}
          stroke={cat().accent}
          stroke-width={s().stroke}
        />
        {/* 内层纹理（分类几何） */}
        <HexagonInner category={props.category ?? 'unknown'} cx={cx()} cy={cy()} r={s().hexR * 0.85} />
      </svg>
    </span>
  )
}

/** 分类徽章：本地(自我主体)/代理(自定义)/云端(第三方) — 对标目录三分类语义 */
export function CategoryBadge(props: { category: string; className?: string }) {
  const cat = () => CAT_STYLE[props.category ?? 'unknown'] ?? CAT_STYLE.unknown
  return (
    <span
      class={clsx(
        'inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-9px font-medium whitespace-nowrap',
        props.category === 'local' && 'bg-nt-core-500/10 text-nt-core-700',
        props.category === 'proxy' && 'bg-nt-act-500/12 text-nt-act-700',
        props.category === 'cloud' && 'bg-nt-memory-500/10 text-nt-memory-700',
        props.category !== 'local' && props.category !== 'proxy' && props.category !== 'cloud' && 'bg-bg-tertiary text-text-muted',
        props.className,
      )}
      title={cat().desc ? `${cat().label} · ${cat().desc}` : cat().label}
    >
      {cat().label}
    </span>
  )
}

/** 免费徽章：keyless / free tier 提供商 */
export function FreeBadge(props: { free: boolean; className?: string }) {
  return (
    <Show when={props.free}>
      <span
        class={clsx(
          'inline-flex items-center px-1.5 py-0.5 rounded-full text-9px font-medium whitespace-nowrap bg-nt-repair-500/10 text-nt-repair-700',
          props.className,
        )}
        title="免费 / keyless 提供商"
      >
        免费
      </span>
    </Show>
  )
}
