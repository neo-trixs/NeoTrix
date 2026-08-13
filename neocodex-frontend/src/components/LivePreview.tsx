import { createSignal, createEffect, Show } from 'solid-js'
import { RefreshCw, ExternalLink, MonitorPlay, Loader2, X } from 'lucide-solid'
import { neocodex } from '../api'
import { clsx } from 'clsx'

/* 批次5：Live Preview 面板 —— 内嵌本地 dev server 预览（对标 Spedy/UI-Inspector/Nimbalyst live preview）。
 * 从当前 workspace 推断常见 dev 端口，iframe 内嵌渲染；提供 URL 编辑/刷新/外部打开。
 * 地址可达性探测用受限 fetch（无 CORS 时 catch 仅代表探测失败，不阻塞手动预览）。 */

interface Props {
  open: boolean
  onClose: () => void
}

const COMMON_PORTS = [1421, 5173, 3000, 8080, 8000, 4173, 4321]

const PORT_HINTS: Record<string, string> = {
  vite: 'Vite 默认 5173',
  next: 'Next.js 默认 3000',
  react: 'CRA 默认 3000',
  svelte: 'SvelteKit 默认 5173',
  astro: 'Astro 默认 4321',
  'nuxt@3': 'Nuxt 默认 3000',
}

export function LivePreview(props: Props) {
  const [url, setUrl] = createSignal('')
  const [probing, setProbing] = createSignal(false)
  const [detected, setDetected] = createSignal<string | null>(null)
  const [busy, setBusy] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [frameKey, setFrameKey] = createSignal(0)

  // 推断 dev server：扫描常见端口，首个可达的作为默认预览地址
  const detect = async () => {
    setProbing(true)
    setError(null)
    setDetected(null)
    try {
      const proj = await neocodex.getProject()
      const workspace = proj || ''
      for (const port of COMMON_PORTS) {
        try {
          const ctrl = new AbortController()
          const t = setTimeout(() => ctrl.abort(), 1200)
          const resp = await fetch(`http://localhost:${port}`, {
            method: 'HEAD',
            signal: ctrl.signal,
            cache: 'no-store',
            mode: 'no-cors',
          })
          clearTimeout(t)
          // mode:no-cors 无法读 status，能触发请求视为在线
          const found = `http://localhost:${port}`
          setDetected(found)
          setUrl(found)
          return
        } catch {
          /* 端口不可达，继续 */
        }
      }
      setError('未探测到本地 dev server，请手动输入预览地址')
      if (workspace) setUrl(`http://localhost:${COMMON_PORTS[0]}`)
    } finally {
      setProbing(false)
    }
  }

  const refresh = () => {
    if (!url()) return
    setFrameKey((k) => k + 1)
  }

  const openBrowser = async () => {
    if (!url()) return
    setBusy(true)
    try {
      await neocodex.openExternal(url())
    } catch (e) {
      setError(e instanceof Error ? e.message : '打开外部浏览器失败')
    } finally {
      setBusy(false)
    }
  }

  // 刷新用 src 追加时间戳破坏缓存（避免 Solid key 类型限制）
  const frameSrc = () => {
    const base = url()
    if (!base) return ''
    const sep = base.includes('?') ? '&' : '?'
    return `${base}${sep}preview_t=${frameKey()}`
  }

  const pickPort = (port: number) => setUrl(`http://localhost:${port}`)

  // Esc 关闭（对标其他面板浮层模式：window keydown + 打开时挂载）
  createEffect(() => {
    if (!props.open) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Escape') return
      e.preventDefault()
      props.onClose()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  return (
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/20 animate-fade-in"
      role="dialog"
      aria-label="Live Preview"
      ref={(el) => (props.open ? el?.focus?.() : undefined)}
      tabIndex={-1}
    >
      <div class="w-[min(1000px,92vw)] h-[min(720px,88vh)] glass-pop border border-white/50 rounded-2xl shadow-2xl overflow-hidden flex flex-col">
        {/* 头部：URL 编辑 / 端口 / 刷新 / 外部打开 / 关闭 */}
        <div class="flex items-center gap-2 px-4 py-2.5 border-b border-white/30 bg-white/40 backdrop-blur-sm flex-shrink-0">
          <MonitorPlay class="w-4 h-4 text-nt-io-600 flex-shrink-0" />
          <input
            class="flex-1 min-w-0 bg-white/50 border border-white/40 rounded-lg px-2 py-1.5 text-xs font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-io-500"
            placeholder="http://localhost:5173"
            value={url()}
            onInput={(e) => setUrl(e.currentTarget.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') refresh() }}
            aria-label="预览地址"
          />
          <button
            class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-xs font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
            onClick={refresh}
            disabled={probing() || busy()}
            aria-label="加载预览"
            title="加载预览"
          >
            {probing() || busy() ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <RefreshCw class="w-3.5 h-3.5" />}
            加载
          </button>
          <button
            class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors flex-shrink-0"
            onClick={openBrowser}
            disabled={!url() || busy()}
            aria-label="在浏览器中打开"
            title="在浏览器中打开"
          >
            <ExternalLink class="w-4 h-4" />
          </button>
          <button
            class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors flex-shrink-0"
            onClick={props.onClose}
            aria-label="关闭"
            title="关闭 (Esc)"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* 端口快速选择 */}
        <div class="flex items-center gap-1.5 px-4 py-1.5 border-b border-white/20 flex-shrink-0 overflow-x-auto">
          <span class="text-[10px] text-text-muted flex-shrink-0">端口:</span>
          <For0 ports={COMMON_PORTS} onPick={pickPort} />
          <span class="text-[10px] text-text-muted flex-shrink-0">
            {PORT_HINTS.vite} · {PORT_HINTS.next} · {PORT_HINTS.astro}
          </span>
        </div>

        {/* 探测提示区 */}
        <Show when={detected()}>
          <div class="px-4 py-1.5 text-[11px] text-emerald-700 bg-emerald-500/10 border-b border-emerald-500/20 flex-shrink-0" role="status">
            已探测到本地服务: {detected()}
          </div>
        </Show>
        <Show when={error()}>
          <div class="px-4 py-1.5 text-[11px] text-red-700 bg-red-500/10 border-b border-red-500/20 flex-shrink-0" role="alert">
            {error()}
          </div>
        </Show>

        {/* iframe 内核 */}
        <div class="flex-1 min-h-0 bg-[#0f0f14] relative">
          <Show when={url()} fallback={<EmptyPreview onDetect={detect} />}>
            <iframe
              src={frameSrc()}
              class="w-full h-full border-0"
              // 🟡 修复：移除 allow-same-origin —— 与 allow-scripts 组合是已知沙箱逃逸
              //（iframe 内脚本可移除自身 sandbox 并访问宿主文档/API Key）。预览站点
              // 以 opaque origin 运行：脚本受限但页面渲染/表单/弹窗不受影响，预览目的达标。
              sandbox="allow-scripts allow-forms allow-popups"
              title="Live Preview"
            />
          </Show>
        </div>
      </div>
    </div>
  )
}

function EmptyPreview(props: { onDetect: () => void }) {
  return (
    <div class={clsx('absolute inset-0 flex flex-col items-center justify-center gap-3 text-center')}>
      <MonitorPlay class="w-8 h-8 text-text-muted/60" />
      <p class="text-sm text-text-muted">输入预览地址，或自动探测本地 dev server</p>
      <button
        class="px-3 py-1.5 rounded-lg text-xs font-medium bg-white/10 hover:bg-white/20 text-text-primary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
        onClick={props.onDetect}
      >
        自动探测
      </button>
    </div>
  )
}

/* Solid 移除 For 泛型传递，用轻量映射 */
function For0(props: { ports: number[]; onPick: (p: number) => void }) {
  return (
    <>{props.ports.map((p, i) => (
      <button
        data-port={i}
        class="px-1.5 py-0.5 rounded text-[10px] font-mono text-text-muted hover:text-nt-io-700 hover:bg-nt-io-500/10 transition-colors flex-shrink-0"
        onClick={() => props.onPick(p)}
        title={`使用 localhost:${p}`}
      >
        {p}
      </button>
    ))}</>
  )
}