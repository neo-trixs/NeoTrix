import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { Monitor, X, RefreshCw, Loader2, MousePointerClick, Keyboard, AppWindow, Cpu } from 'lucide-solid'
import { computer as computerApi } from '../api'
import type { DisplayInfo, FrontmostApp, MousePosition, ScreenCapture, WindowInfo } from '../api/types'
import { clsx } from 'clsx'

// 截图+窗口枚举节流：时间戳提升到模块级，跨组件重挂载（切视图重挂载触发 onMount）仍生效，
// 5s 内重复进入复用上次结果，避免完整截图/枚举 IPC 风暴；手动「重新捕获」/头部刷新不受限。
let lastSnapshotAt = 0
const SNAPSHOT_THROTTLE_MS = 5000
// 🟡 修复：模块级缓存最近一次截图。切走再切回（<5s）触发重挂载，onMount 自动加载被
// 节流跳过时恢复缓存，避免面板显示「点击重新捕获获取截图」空态误导；头部刷新按钮
// 传 force 直接绕过节流（与「手动不受限」注释一致）。
let lastSnapshotDataUrl: string | null = null

interface Props {
  open: boolean
  onClose: () => void
  /** 内嵌模式：作为侧栏标签页渲染在主区域（无浮层/无滑入动画/无关闭按钮） */
  embedded?: boolean
}

export function ComputerUse(props: Props) {
  const [screenshotDataUrl, setScreenshotDataUrl] = createSignal<string | null>(null)
  const [windows, setWindows] = createSignal<WindowInfo[]>([])
  const [frontmost, setFrontmost] = createSignal<FrontmostApp | null>(null)
  const [mousePos, setMousePos] = createSignal<MousePosition | null>(null)
  const [displays, setDisplays] = createSignal<DisplayInfo[]>([])
  const [loading, setLoading] = createSignal(false)
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  const [keyText, setKeyText] = createSignal('')
  const [keyCode, setKeyCode] = createSignal('')
  const [mods, setMods] = createSignal<string[]>([])
  let firstBtnRef: HTMLButtonElement | undefined
  let closeBtnRef: HTMLButtonElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板时的触发元素（浮层模式还原焦点用；embedded 模式由 ⌘6/侧栏触发，可能为 body）
  let lastFocusedEl: HTMLElement | null = null

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范），并记录触发元素；
  // 关闭时（Esc/关闭按钮/视图切换触发卸载）经 effect 清理还原焦点
  createEffect(() => {
    if (!props.open) return
    lastFocusedEl = document.activeElement as HTMLElement | null
    const raf = requestAnimationFrame(() => {
      if (firstBtnRef) firstBtnRef.focus()
      else panelRef?.focus()
    })
    return () => {
      cancelAnimationFrame(raf)
      if (lastFocusedEl?.isConnected) lastFocusedEl.focus()
    }
  })

  const close = () => {
    // 焦点还原：优先触发元素（工具栏按钮仍存活），拿不到则还原到面板关闭按钮
    if (lastFocusedEl && lastFocusedEl.isConnected) {
      lastFocusedEl.focus()
    } else if (!props.embedded && closeBtnRef) {
      closeBtnRef.focus()
    }
    props.onClose()
  }

  const load = async (force = false) => {
    const now = Date.now()
    if (now - lastSnapshotAt < SNAPSHOT_THROTTLE_MS && !force) {
      // 节流命中：恢复最近截图（若有），不打断已有内容
      if (lastSnapshotDataUrl) setScreenshotDataUrl(lastSnapshotDataUrl)
      return
    }
    lastSnapshotAt = now
    setLoading(true)
    setError(null)
    try {
      const [disp, front] = await Promise.all([
        computerApi.screenList(),
        computerApi.getFrontmostApp().catch(() => null),
      ])
      setDisplays(disp)
      setFrontmost(front)
      await capture()
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const capture = async () => {
    setBusy('capture')
    setError(null)
    try {
      // 内存内联截图：后端捕获→base64→返回并自清理临时文件，前端不再 readFile/remove
      const shot = await computerApi.screenshotAndSave()
      if (!shot.data_base64) throw new Error('截图未返回内存数据')
      lastSnapshotDataUrl = `data:image/png;base64,${shot.data_base64}`
      setScreenshotDataUrl(lastSnapshotDataUrl)
      // Refresh window list + mouse position
      const [wl, mp] = await Promise.all([
        computerApi.getWindowList().catch(() => [] as WindowInfo[]),
        computerApi.mousePosition().catch(() => null),
      ])
      setWindows(wl)
      setMousePos(mp)
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const moveMouse = async (x: number, y: number) => {
    setBusy('move')
    setError(null)
    try {
      await computerApi.mouseMove(x, y)
      const mp = await computerApi.mousePosition()
      setMousePos(mp)
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const click = async () => {
    setBusy('click')
    setError(null)
    try {
      await computerApi.mouseClick(null)
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const typeText = async () => {
    if (!keyText()) return
    setBusy('type')
    setError(null)
    try {
      await computerApi.keyboardType(keyText())
      setKeyText('')
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const pressKey = async () => {
    if (!keyCode()) return
    setBusy('press')
    setError(null)
    try {
      await computerApi.keyboardPress(keyCode(), mods())
    } catch (e) {
      setError(String(e))
    } finally {
      setBusy(null)
    }
  }

  const toggleMod = (m: string) => {
    setMods(prev => (prev.includes(m) ? prev.filter(x => x !== m) : [...prev, m]))
  }

  onMount(load)

  return (
    <Show when={props.open}>
      <div
        ref={panelRef}
        class={props.embedded ? 'flex-1 h-full flex flex-col min-h-0' : 'panel w-[30rem]'}
        role="dialog"
        aria-label="电脑控制"
        aria-modal={props.embedded ? undefined : 'true'}
        tabIndex={-1}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault()
            close()
            return
          }
          if (e.key === 'Tab' && panelRef) {
            const focusables = panelRef.querySelectorAll<HTMLElement>(
              'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
            )
            if (focusables.length === 0) return
            const first = focusables[0]
            const last = focusables[focusables.length - 1]
            const active = document.activeElement
            if (e.shiftKey && (active === first || active === panelRef)) {
              e.preventDefault()
              last.focus()
            } else if (!e.shiftKey && active === last) {
              e.preventDefault()
              first.focus()
            }
          }
        }}
      >
        {/* Header */}
        <div class="panel-head">
          <Monitor class="panel-head-icon text-nt-core-300" />
          <span class="panel-title">电脑控制</span>
          <span class="panel-sub">macOS 控制</span>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={() => load(true)}
            aria-label="刷新"
            title="刷新（绕过 5s 节流）"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <Show when={!props.embedded}>
            <button
              ref={closeBtnRef}
              class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
              onClick={close}
              aria-label="关闭"
            >
              <X class="w-4 h-4" />
            </button>
          </Show>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg">{error()}</div>
          </Show>

          {/* Frontmost app */}
          <Show when={frontmost()}>
            <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3 flex items-center gap-3">
              <AppWindow class="w-5 h-5 text-nt-core-300 flex-shrink-0" />
              <div class="min-w-0">
                <div class="text-sm font-medium text-text-primary truncate">{frontmost()!.app_name}</div>
                <div class="text-xs text-text-muted truncate">{frontmost()!.title || '—'}</div>
              </div>
            </div>
          </Show>

          {/* Screenshot */}
          <div>
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2 text-xs text-text-muted">
                <Cpu class="w-3.5 h-3.5" />
                屏幕捕获
              </div>
              <button
                class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-nt-core-300 hover:bg-nt-core-500/10 border border-nt-core-500/30 transition-colors"
                onClick={capture}
                disabled={busy() !== null}
              >
                {busy() === 'capture' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <RefreshCw class="w-3.5 h-3.5" />}
                重新捕获
              </button>
            </div>
            <Show when={loading()} fallback={
              <Show when={screenshotDataUrl()} fallback={
                <div class="rounded-xl border border-dashed border-border-primary p-8 text-center text-xs text-text-muted">
                  点击「重新捕获」获取屏幕截图
                </div>
              }>
                <div class="rounded-xl overflow-hidden border border-border-primary bg-bg-primary/40">
                  <img src={screenshotDataUrl()!} alt="屏幕截图" class="w-full h-auto max-h-64 object-contain" />
                </div>
              </Show>
            }>
              <div class="rounded-xl border border-dashed border-border-primary p-8 text-center text-xs text-text-muted flex items-center justify-center gap-2">
                <Loader2 class="w-4 h-4 animate-spin" />
                正在捕获屏幕…
              </div>
            </Show>
          </div>

          {/* Mouse control */}
          <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3">
            <div class="flex items-center gap-2 text-xs text-text-muted mb-2">
              <MousePointerClick class="w-3.5 h-3.5" />
              鼠标控制
            </div>
            <Show when={mousePos()}>
              <div class="text-xs font-mono text-text-primary mb-2">
                位置: ({mousePos()!.x}, {mousePos()!.y})
              </div>
            </Show>
            <div class="grid grid-cols-3 gap-2">
              <button class="col-span-3 flex items-center justify-center gap-2 px-2 py-2 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium" onClick={click} disabled={busy() !== null}>
                {busy() === 'click' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <MousePointerClick class="w-3.5 h-3.5" />}
                点击 (当前位置)
              </button>
              <Show when={mousePos()}>
                <button class="px-2 py-2 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(Math.max(0, mousePos()!.x - 40), mousePos()!.y)} disabled={busy() !== null}>← 左移</button>
                <button class="px-2 py-2 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(mousePos()!.x, Math.max(0, mousePos()!.y - 40))} disabled={busy() !== null}>↑ 上移</button>
                <button class="px-2 py-2 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(Math.min(displays()[0]?.width ?? 1920, mousePos()!.x + 40), mousePos()!.y)} disabled={busy() !== null}>→ 右移</button>
                <button class="px-2 py-2 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(mousePos()!.x, Math.min(displays()[0]?.height ?? 1080, mousePos()!.y + 40))} disabled={busy() !== null}>↓ 下移</button>
              </Show>
            </div>
          </div>

          {/* Keyboard control */}
          <div class="rounded-xl border border-border-primary bg-bg-primary/40 p-3 space-y-3">
            <div class="flex items-center gap-2 text-xs text-text-muted">
              <Keyboard class="w-3.5 h-3.5" />
              键盘控制
            </div>
            <div>
              <label class="text-[10px] text-text-muted uppercase tracking-wider">输入文本</label>
              <div class="flex gap-2 mt-1">
                <input
                  value={keyText()}
                  onInput={(e) => setKeyText(e.currentTarget.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); typeText() } }}
                  placeholder="要输入的文本..."
                  class="flex-1 bg-bg-primary border border-border-primary rounded-lg px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-core-400/50"
                />
                <button
                  class="flex items-center gap-1 px-3 py-2 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium"
                  onClick={typeText}
                  disabled={busy() !== null || !keyText()}
                >
                  {busy() === 'type' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Keyboard class="w-3.5 h-3.5" />}
                  输入
                </button>
              </div>
            </div>
            <div>
              <label class="text-[10px] text-text-muted uppercase tracking-wider">按键 (key code) + 修饰键</label>
              <div class="flex gap-2 mt-1">
                <input
                  value={keyCode()}
                  onInput={(e) => setKeyCode(e.currentTarget.value.replace(/\D/g, ''))}
                  onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); pressKey() } }}
                  placeholder="如 36 (回车) / 49 (空格)"
                  class="flex-1 bg-bg-primary border border-border-primary rounded-lg px-3 py-2 text-sm font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-core-400/50"
                />
                <button
                  class="flex items-center gap-1 px-3 py-2 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium"
                  onClick={pressKey}
                  disabled={busy() !== null || !keyCode()}
                >
                  {busy() === 'press' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Keyboard class="w-3.5 h-3.5" />}
                  按下
                </button>
              </div>
              <div class="flex gap-2 mt-2 flex-wrap">
                {['command', 'option', 'control', 'shift'].map((m) => (
                  <button
                    class={clsx(
                      'px-2 py-1 rounded-lg text-xs border transition-colors',
                      mods().includes(m)
                        ? 'border-nt-core-400/50 bg-nt-core-500/15 text-nt-core-300'
                        : 'border-border-primary text-text-muted hover:text-text-primary'
                    )}
                    onClick={() => toggleMod(m)}
                    aria-pressed={mods().includes(m)}
                  >
                    {m}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Windows */}
          <div>
            <div class="flex items-center gap-2 text-xs text-text-muted mb-2">
              <AppWindow class="w-3.5 h-3.5" />
              可见应用 ({windows().length})
            </div>
            <Show when={windows().length === 0}>
              <div class="text-xs text-text-muted px-2">未获取到窗口列表</div>
            </Show>
            <div class="space-y-1">
              <For each={windows()}>
                {(w) => (
                  <div class="flex items-center gap-2 px-2 py-2 rounded-lg bg-bg-primary/40 text-xs">
                    <span class="text-text-muted w-6 h-6 rounded bg-bg-tertiary flex items-center justify-center font-mono text-[10px] flex-shrink-0">
                      {w.pid || '—'}
                    </span>
                    <span class="text-text-primary truncate flex-1">{w.app_name}</span>
                    <span class="text-text-muted truncate max-w-[40%]">{w.title}</span>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      </div>
    </Show>
  )
}
