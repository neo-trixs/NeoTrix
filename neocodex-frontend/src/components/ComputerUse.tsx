import { createSignal, onMount, createEffect, Show, For } from 'solid-js'
import { Monitor, X, RefreshCw, Loader2, MousePointerClick, Keyboard, AppWindow, Cpu } from 'lucide-solid'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'

interface ScreenCapture {
  path: string
  width: number
  height: number
  format: string
  timestamp: number
}

interface WindowInfo {
  title: string
  pid: number
  app_name: string
}

interface FrontmostApp {
  app_name: string
  title: string
}

interface MousePosition {
  x: number
  y: number
}

interface DisplayInfo {
  id: number
  name: string
  width: number
  height: number
  is_primary: boolean
  scale_factor: number
}

interface Props {
  open: boolean
  onClose: () => void
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

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范）
  createEffect(() => {
    if (props.open && firstBtnRef) firstBtnRef.focus()
  })

  const load = async () => {
    setLoading(true)
    setError(null)
    try {
      const [disp, front] = await Promise.all([
        invoke<DisplayInfo[]>('computer_screen_list'),
        invoke<FrontmostApp>('computer_get_frontmost_app').catch(() => null),
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
      // Capture to temp path, then read as data URL
      const ts = Date.now()
      const filePath = `/tmp/neotrix_screen_${ts}.png`
      await invoke<ScreenCapture>('computer_screenshot_and_save', { path: filePath })
      // Read file content via tauri fs plugin
      const { readFile } = await import('@tauri-apps/plugin-fs')
      const bytes = await readFile(filePath)
      const blob = new Blob([bytes], { type: 'image/png' })
      const dataUrl = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader()
        reader.onload = () => resolve(String(reader.result))
        reader.onerror = () => reject(new Error('read blob failed'))
        reader.readAsDataURL(blob)
      })
      setScreenshotDataUrl(dataUrl)
      // Refresh window list + mouse position
      const [wl, mp] = await Promise.all([
        invoke<WindowInfo[]>('computer_get_window_list').catch(() => [] as WindowInfo[]),
        invoke<MousePosition>('computer_mouse_position').catch(() => null),
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
      await invoke('computer_mouse_move', { x, y })
      const mp = await invoke<MousePosition>('computer_mouse_position')
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
      await invoke('computer_mouse_click', { button: null })
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
      await invoke('computer_keyboard_type', { text: keyText() })
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
      await invoke('computer_keyboard_press', { key: keyCode(), modifiers: mods() })
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
      <div class="panel w-[30rem]">
        {/* Header */}
        <div class="panel-head">
          <Monitor class="panel-head-icon text-nt-core-300" />
          <span class="panel-title">电脑控制</span>
          <span class="panel-sub">macOS 控制</span>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={load}
            aria-label="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="p-1.5 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
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
            <Show when={screenshotDataUrl()} fallback={
              <div class="rounded-xl border border-dashed border-border-primary p-8 text-center text-xs text-text-muted">
                点击「重新捕获」获取屏幕截图
              </div>
            }>
              <div class="rounded-xl overflow-hidden border border-border-primary bg-bg-primary/40">
                <img src={screenshotDataUrl()!} alt="屏幕截图" class="w-full h-auto max-h-64 object-contain" />
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
            <div class="grid grid-cols-3 gap-1.5">
              <button class="col-span-3 flex items-center justify-center gap-1.5 px-2 py-1.5 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium" onClick={click} disabled={busy() !== null}>
                {busy() === 'click' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <MousePointerClick class="w-3.5 h-3.5" />}
                点击 (当前位置)
              </button>
              <Show when={mousePos()}>
                <button class="px-2 py-1.5 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(Math.max(0, mousePos()!.x - 40), mousePos()!.y)} disabled={busy() !== null}>← 左移</button>
                <button class="px-2 py-1.5 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(mousePos()!.x, Math.max(0, mousePos()!.y - 40))} disabled={busy() !== null}>↑ 上移</button>
                <button class="px-2 py-1.5 rounded-lg bg-bg-tertiary text-xs text-text-secondary hover:text-text-primary transition-colors" onClick={() => moveMouse(Math.min(displays()[0]?.width ?? 1920, mousePos()!.x + 40), mousePos()!.y)} disabled={busy() !== null}>→ 右移</button>
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
              <div class="flex gap-1.5 mt-1">
                <input
                  value={keyText()}
                  onInput={(e) => setKeyText(e.currentTarget.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); typeText() } }}
                  placeholder="要输入的文本..."
                  class="flex-1 bg-bg-primary border border-border-primary rounded-lg px-2.5 py-1.5 text-sm text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-core-400/50"
                />
                <button
                  class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium"
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
              <div class="flex gap-1.5 mt-1">
                <input
                  value={keyCode()}
                  onInput={(e) => setKeyCode(e.currentTarget.value.replace(/\D/g, ''))}
                  onKeyDown={(e) => { if (e.key === 'Enter') { e.preventDefault(); pressKey() } }}
                  placeholder="如 36 (回车) / 49 (空格)"
                  class="flex-1 bg-bg-primary border border-border-primary rounded-lg px-2.5 py-1.5 text-sm font-mono text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-core-400/50"
                />
                <button
                  class="flex items-center gap-1 px-2.5 py-1.5 rounded-lg bg-nt-core-500/15 text-nt-core-300 hover:bg-nt-core-500/25 transition-colors text-xs font-medium"
                  onClick={pressKey}
                  disabled={busy() !== null || !keyCode()}
                >
                  {busy() === 'press' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Keyboard class="w-3.5 h-3.5" />}
                  按下
                </button>
              </div>
              <div class="flex gap-1.5 mt-1.5 flex-wrap">
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
                  <div class="flex items-center gap-2 px-2 py-1.5 rounded-lg bg-bg-primary/40 text-xs">
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
