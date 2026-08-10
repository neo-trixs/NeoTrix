import { clsx } from 'clsx'
import { Show } from 'solid-js'
import type { ProviderMeta } from '../api/types'

/* ════════════════════════════════════════════
   ProviderIcon — LLM 第三方统一品牌标识（对标 Slack/Linear brand avatar）
   视觉语言：品牌色 monogram（厂商字色块）+ 可选分类徽章
   - 分类徽章：本地(绿) / 代理(琥珀) / 云端(蓝)
   - 免费徽章：keyless / free tier 提供商
   全站 ProviderSelector / SettingsModal 共用，保证同一提供商呈现一致
   ProviderMeta 契约见 api/types.ts（单一事实源）
   ════════════════════════════════════════════ */

export type { ProviderMeta } from '../api/types'

/** 品牌色 + 首字 monogram（无版权风险的自绘标识） */
const BRAND: Record<string, { color: string; glyph: string }> = {
  openai: { color: '#10a37f', glyph: 'O' },
  anthropic: { color: '#d97757', glyph: 'C' },
  gemini: { color: '#4285f4', glyph: 'G' },
  deepseek: { color: '#4d6bfe', glyph: 'D' },
  'deepseek-free': { color: '#4d6bfe', glyph: 'D' },
  groq: { color: '#f55036', glyph: 'G' },
  openrouter: { color: '#6d5ce6', glyph: 'OR' },
  cerebras: { color: '#1f9d7c', glyph: 'C' },
  sambanova: { color: '#7c3aed', glyph: 'S' },
  mistral: { color: '#f7a600', glyph: 'M' },
  cohere: { color: '#39594d', glyph: 'Co' },
  together: { color: '#4d5de0', glyph: 'T' },
  'together-free': { color: '#4d5de0', glyph: 'T' },
  'github-models': { color: '#24292e', glyph: 'GH' },
  huggingface: { color: '#ff9d00', glyph: 'HF' },
  siliconflow: { color: '#3b82f6', glyph: 'S' },
  nvidia: { color: '#76b900', glyph: 'NV' },
  cloudflare: { color: '#f6821f', glyph: 'CF' },
  zai: { color: '#e85454', glyph: 'Z' },
  pollinations: { color: '#db2777', glyph: 'P' },
  bazaarlink: { color: '#0d9488', glyph: 'B' },
  'opencode-zen': { color: '#f0913a', glyph: 'Z' },
  modelscope: { color: '#2563eb', glyph: 'MS' },
  ovh: { color: '#1a3fbf', glyph: 'OVH' },
  freetheai: { color: '#9333ea', glyph: 'F' },
  zerolimit: { color: '#16a34a', glyph: 'Z' },
  llm7: { color: '#64748b', glyph: '7' },
  kilo: { color: '#e85454', glyph: 'K' },
  // 本地推理 — 统一绿
  ollama: { color: '#16a34a', glyph: 'O' },
  'lm-studio': { color: '#0d9488', glyph: 'L' },
  llamacpp: { color: '#3b82f6', glyph: 'LL' },
  'vllm-local': { color: '#2563eb', glyph: 'V' },
  'sglang-local': { color: '#9333ea', glyph: 'S' },
}

const FALLBACK_COLORS = ['#f0913a', '#e85454', '#16a34a', '#2563eb', '#9333ea', '#0d9488']

function fallbackColor(name: string): string {
  let h = 0
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0
  return FALLBACK_COLORS[h % FALLBACK_COLORS.length]
}

function glyphOf(name: string): string {
  const meta = BRAND[name.toLowerCase()]
  if (meta) return meta.glyph
  // 兜底：取名字前两个字符大写
  const clean = name.replace(/[-_]/g, '')
  return (clean[0] ?? '?').toUpperCase() + (clean[1] !== undefined ? clean[1].toLowerCase() : '')
}

export function ProviderIcon(props: { name: string; size?: 'sm' | 'md'; className?: string }) {
  const size = () => props.size ?? 'md'
  const key = () => props.name.toLowerCase()
  return (
    <span
      class={clsx(
        'rounded-lg flex items-center justify-center font-semibold text-white flex-shrink-0 select-none',
        size() === 'md' ? 'w-8 h-8 text-[12px]' : 'w-6 h-6 text-[10px]',
        props.className
      )}
      style={{
        background: BRAND[key()]?.color ?? fallbackColor(props.name),
        'box-shadow': 'inset 0 1px 0 rgba(255,255,255,0.25)',
      }}
      aria-hidden="true"
    >
      {glyphOf(props.name)}
    </span>
  )
}

/** 分类徽章：本地(自我主体)/代理(自定义)/云端(第三方) — 对标目录三分类语义 */
export function CategoryBadge(props: { category: string; className?: string }) {
  const cat = () => props.category ?? 'unknown'
  return (
    <span
      class={clsx(
        'inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-[9px] font-medium whitespace-nowrap',
        cat() === 'local' && 'bg-nt-core-500/10 text-nt-core-700',
        cat() === 'proxy' && 'bg-nt-act-500/12 text-nt-act-700',
        cat() === 'cloud' && 'bg-nt-memory-500/10 text-nt-memory-700',
        cat() === 'unknown' && 'bg-bg-tertiary text-text-muted',
        props.className
      )}
      title={
        cat() === 'local' ? '本地推理 · 数据不出设备'
        : cat() === 'proxy' ? '自定义代理 · OpenAI 兼容中转'
        : cat() === 'cloud' ? '云端 API · 数据发送至第三方'
        : '未知分类'
      }
    >
      {cat() === 'local' ? '本地'
        : cat() === 'proxy' ? '代理'
        : cat() === 'cloud' ? '云端'
        : '未知'}
    </span>
  )
}

/** 免费徽章：keyless / free tier 提供商 */
export function FreeBadge(props: { free: boolean; className?: string }) {
  return (
    <Show when={props.free}>
      <span
        class={clsx(
          'inline-flex items-center px-1.5 py-0.5 rounded-full text-[9px] font-medium whitespace-nowrap bg-nt-repair-500/10 text-nt-repair-700',
          props.className
        )}
        title="免费 / keyless 提供商"
      >
        免费
      </span>
    </Show>
  )
}
