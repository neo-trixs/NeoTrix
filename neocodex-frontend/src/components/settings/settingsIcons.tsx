/* ═══════════════════════════════════════════
   components/settings/settingsIcons.tsx — 设置页共享浅橙电流细线图标
   主题：轻灵电流（light-current）——细线轮廓 + 浅橙渐变发光节点 + 流动动画。
   - 主线 stroke=currentColor（导航灰→激活橙语义随父级 color 变化）
   - 端点/关键节点 fill=浅橙渐变（#fbd9b8 → #f5a862 → #e07f2b），带发光
   - 部分主线加 .ic-flow 电流流动（stroke-dasharray 循环）
   每个 svg 内嵌独立 defs（自包含；重复 id 浏览器取首个定义，内容一致无碍）。
   ══════════════════════════════════════════════ */
 
/* 每个图标内嵌的浅橙电流渐变（浅金→浅橙→中橙） */
const CURRENT_GRAD = (
  <defs>
    <linearGradient id="nt-current-grad" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#fbd9b8" />
      <stop offset="55%" stop-color="#f5a862" />
      <stop offset="100%" stop-color="#e07f2b" />
    </linearGradient>
  </defs>
)
 
/* ── 外扩线条图标（open/expand 语义，非内敛） ── */
export function ExpandIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 中央电流节点（渐变发光） + 四条外射线（细线流动） */}
      <circle cx="8" cy="8" r="1.4" fill="url(#nt-current-grad)" />
      <line x1="8" y1="4" x2="8" y2="0.8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="8" y1="12" x2="8" y2="15.2" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="4" y1="8" x2="0.8" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="12" y1="8" x2="15.2" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <circle cx="8" cy="0.8" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="8" cy="15.2" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="0.8" cy="8" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="15.2" cy="8" r="0.7" fill="url(#nt-current-grad)" />
    </svg>
  )
}
 
export function PaletteIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 色板细环 + 外扩扇区；三个取样点渐变发光 */}
      <path d="M8 2.5a5.5 5.5 0 100 11c1.5 0 2-1 1-2-.7-.7-.3-1.5 1-1.5h1.5c1.1 0 2-.9 2-2A5.5 5.5 0 008 2.5z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
      <circle cx="5.5" cy="6.5" r="0.85" fill="url(#nt-current-grad)" />
      <circle cx="8" cy="5" r="0.85" fill="url(#nt-current-grad)" />
      <circle cx="10.5" cy="6.5" r="0.85" fill="url(#nt-current-grad)" />
    </svg>
  )
}
 
export function InfoIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 信息细环 + 电流节点 */}
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1" />
      <line x1="8" y1="7.5" x2="8" y2="11" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <circle cx="8" cy="4.6" r="0.85" fill="url(#nt-current-grad)" />
    </svg>
  )
}

export function AlertCircleIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class={props.class ?? 'nt-ic'}>
      {CURRENT_GRAD}
      {/* 警告圆环 + 电流节点 */}
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1" />
      <line x1="8" y1="5" x2="8" y2="9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      <circle cx="8" cy="11.5" r="0.85" fill="url(#nt-current-grad)" />
    </svg>
  )
}

export function PluginsIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 拼图块细线 + 外扩射线（插件扩展语义） */}
      <path d="M5 3h4v2.5a1.5 1.5 0 010 3V11H5V8.5a1.5 1.5 0 010-3V3z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
      <line x1="8" y1="1.2" x2="8" y2="0.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="8" y1="15" x2="8" y2="15.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="1" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="15" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <circle cx="8" cy="0.5" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="8" cy="15.5" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="0.5" cy="8" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="15.5" cy="8" r="0.7" fill="url(#nt-current-grad)" />
    </svg>
  )
}
 
export function DataIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 数据库圆柱细线 + 外扩导出箭头 */}
      <ellipse cx="8" cy="4" rx="5" ry="2.2" stroke="currentColor" stroke-width="1" />
      <path d="M3 4v8c0 1.2 2.2 2.2 5 2.2s5-1 5-2.2V4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="3" y1="8" x2="8" y2="8" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <circle cx="13" cy="4" r="0.8" fill="url(#nt-current-grad)" />
      <line x1="13" y1="8" x2="13" y2="3.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="13" y1="3.5" x2="11.5" y2="5" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" />
<line x1="13" y1="3.5" x2="14.5" y2="5" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  )
}

export function XIcon() {
  return (
    <svg viewBox="0 0 12 12" width="12" height="12" fill="none" class="nt-ic">
      <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
      <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
    </svg>
  )
}
 
export function TagIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 标签签低细线 + 斜杆孔发光 + 外扩射线（标签集合语义） */}
      <path d="M2 3.5h8l4 4.5-6 6L2 9V3.5z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
      <circle cx="6.4" cy="6.4" r="1.05" fill="url(#nt-current-grad)" />
      <line x1="8" y1="14.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" opacity="0.4" />
      <line x1="15" y1="7" x2="15.5" y2="7" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

/* ── 界面语言：地球（国际化语义） ── */
export function GlobeIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      <circle cx="8" cy="8" r="6.2" stroke="currentColor" stroke-width="1" />
      <path d="M1.8 8h12.4" stroke="currentColor" stroke-width="1" />
      <path d="M8 1.8c2.1 1.7 3.2 3.9 3.2 6.2S10.1 12.5 8 14.2C5.9 12.5 4.8 10.3 4.8 8S5.9 3.5 8 1.8z" stroke="currentColor" stroke-width="1" />
    </svg>
  )
}

/* ── 动效/性能：闪电电流（性能活跃语义，契合 light-current 主题） ── */
export function BoltIcon() {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 闪电细线 + 两端渐变节点（电流动能） */}
      <path d="M8.8 1.5L3.5 9h3.4l-1.4 5.5L11 6.8H7.6l1.2-5.3z" stroke="currentColor" stroke-width="1" stroke-linejoin="round" />
      <circle cx="6.2" cy="3.5" r="0.7" fill="url(#nt-current-grad)" />
      <circle cx="10.4" cy="11.2" r="0.7" fill="url(#nt-current-grad)" />
    </svg>
  )
}
 
/* ── 模型：代理池语义（层叠立方体 + 顶部电流节点） ── */
export function ModelIcon() {  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class="nt-ic">
      {CURRENT_GRAD}
      {/* 层叠模型层细线 + 顶部电流节点（模型池/代理池语义） */}
      <rect x="3.5" y="4" width="9" height="3" rx="1" stroke="currentColor" stroke-width="1" />
      <rect x="5" y="8" width="6" height="3" rx="1" stroke="currentColor" stroke-width="1" />
      <circle cx="8" cy="1.8" r="1.1" fill="url(#nt-current-grad)" />
      <line x1="8" y1="2.9" x2="8" y2="4" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
      <line x1="3.5" y1="11" x2="2" y2="11" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" opacity="0.4" />
      <line x1="12.5" y1="11" x2="14" y2="11" stroke="currentColor" stroke-width="0.9" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}
 
export function CheckIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 12 12" fill="none" class={props.class}>
      <path d="M2.5 6.2l2.5 2.5 4.5-5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  )
}

export function ActiveDotIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 8 8" fill="url(#nt-current-grad)" class={props.class}>
      {CURRENT_GRAD}
      <circle cx="4" cy="4" r="3.2" />
    </svg>
  )
}
 
/* ── 连接测试：试管图标（连接测试/验证语义） ── */
export function TestTubeIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class={props.class}>
      {CURRENT_GRAD}
      {/* 试管主体 */}
      <path d="M4 3v8a2 2 0 01-2 2h8a2 2 0 002-2V3a1 1 0 00-1-1H4a1 1 0 01-1-1V3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      {/* 液面 */}
      <path d="M4 5h8" stroke="currentColor" stroke-width="0.8" stroke-linecap="round" />
      {/* 液面气泡 */}
      <circle cx="10" cy="7" r="0.8" fill="url(#nt-current-grad)" opacity="0.6" />
      <circle cx="10" cy="9" r="0.6" fill="url(#nt-current-grad)" opacity="0.4" />
      {/* 颈部高光 */}
      <path d="M5 3h8" stroke="currentColor" stroke-width="0.6" opacity="0.3" stroke-linecap="round" />
    </svg>
  )
}

/* ── 新增外部模型：加号（智能配置入口） ── */
export function PlusIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 16 16" width="16" height="16" fill="none" class={props.class}>
      {CURRENT_GRAD}
      <line x1="8" y1="3.5" x2="8" y2="12.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="3.5" y1="8" x2="12.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  )
}