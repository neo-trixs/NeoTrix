import { createSignal, onMount, onCleanup } from 'solid-js'
import { Minus, Square, X } from 'lucide-solid'
import { getCurrentWindow } from '@tauri-apps/api/window'

function detectMacOS() {
  return navigator.userAgent.includes('Macintosh')
}

const IS_MACOS = detectMacOS()

export default function Navbar() {
  const [isFullscreen, setIsFullscreen] = createSignal(false)

  let unlisten: (() => void) | undefined

  onMount(async () => {
    if (!IS_MACOS) return
    try {
      const appWindow = getCurrentWindow()
      const fullscreen = await appWindow.isFullscreen()
      setIsFullscreen(fullscreen)
      unlisten = await appWindow.onResized(async () => {
        const fs = await appWindow.isFullscreen()
        setIsFullscreen(fs)
      })
    } catch (e) {
      console.error('[Navbar] failed to sync fullscreen:', e)
    }
  })

  onCleanup(() => unlisten?.())

  const handleWindowAction = (action: 'minimize' | 'maximize' | 'background') => {
    const appWindow = getCurrentWindow()
    switch (action) {
      case 'minimize':
        appWindow.minimize()
        break
      case 'maximize':
        appWindow.toggleMaximize()
        break
      case 'background':
        appWindow.hide()
        break
    }
  }

  return (
    <div
      class="relative flex h-12 w-full flex-none select-none items-center border-b border-white/5 bg-[#0f0f17]/80 backdrop-blur-xl"
      style={{
        'padding-left': IS_MACOS && !isFullscreen() ? '78px' : '16px',
        'padding-right': '16px',
        display: IS_MACOS && isFullscreen() ? 'none' : 'flex',
      }}
    >
      {/* Drag region */}
      <div
        class="min-w-0 flex-1 self-stretch touch-none"
        data-tauri-drag-region
      />

      {/* Window controls (Windows/Linux) */}
      {!IS_MACOS && (
        <div class="flex items-center gap-2">
          <button
            class="w-7 h-7 rounded-lg flex items-center justify-center text-[#6b7280] hover:bg-white/10 hover:text-white transition-colors"
            aria-label="Minimize"
            onClick={() => handleWindowAction('minimize')}
          >
            <Minus size={14} />
          </button>
          <button
            class="w-7 h-7 rounded-lg flex items-center justify-center text-[#6b7280] hover:bg-white/10 hover:text-white transition-colors"
            aria-label="Maximize"
            onClick={() => handleWindowAction('maximize')}
          >
            <Square size={12} />
          </button>
          <button
            class="w-7 h-7 rounded-lg flex items-center justify-center text-[#6b7280] hover:bg-red-500/20 hover:text-red-400 transition-colors"
            aria-label="Hide to tray"
            onClick={() => handleWindowAction('background')}
          >
            <X size={14} />
          </button>
        </div>
      )}
    </div>
  )
}
