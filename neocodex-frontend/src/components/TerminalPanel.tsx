/* ════════════════════════════════════════════
   TerminalPanel — 内嵌 PTY 终端（吸收 Minke Terminal Tab）
   xterm.js 生命周期:
     mount → pty_spawn → 监听 output → term.write
     term.onData → pty_write (用户键入)
     resize → pty_resize (FitAddon)
     cleanup → unlisten × 2 + pty_close
   ════════════════════════════════════════════ */
import { createSignal, onCleanup, onMount, Show } from 'solid-js'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import type { UnlistenFn } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'
import {
  ptySpawn,
  ptyWrite,
  ptyResize,
  ptyClose,
  onPtyOutput,
  onPtyExit,
} from '../api/pty'

export function TerminalPanel() {
  let containerRef: HTMLDivElement | undefined
  const [status, setStatus] = createSignal<'connecting' | 'live' | 'exited' | 'error'>('connecting')
  const [errMsg, setErrMsg] = createSignal('')

  onMount(async () => {
    if (!containerRef) return

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: "'JetBrains Mono', 'SF Mono', Menlo, monospace",
      theme: {
        background: '#1a1a2e',
        foreground: '#e0e0e0',
        cursor: '#f0913a',
        selectionBackground: '#2563eb44',
      },
      scrollback: 5000,
    })
    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(containerRef)

    // 首帧布局后 fit（容器需已有尺寸）
    requestAnimationFrame(() => {
      try { fit.fit() } catch { /* 容器未可见时忽略 */ }
    })

    const unlisteners: UnlistenFn[] = []
    let sessionId = ''
    let exited = false

    try {
      const cols = Math.max(20, Math.floor(containerRef.clientWidth / 9))
      const rows = Math.max(6, Math.floor(containerRef.clientHeight / 18))
      sessionId = await ptySpawn(cols, rows)

      // PTY 输出 → 终端
      unlisteners.push(
        await onPtyOutput(sessionId, (data) => term.write(data)),
        await onPtyExit(sessionId, (code) => {
          exited = true
          setStatus('exited')
          term.write(`\r\n\x1b[90m[进程已退出，code ${code}]\x1b[0m\r\n`)
        }),
      )

      // 用户键入 → PTY stdin
      term.onData((data) => {
        if (!exited) void ptyWrite(sessionId, data)
      })

      // 尺寸变化 → 同步 PTY
      const doFit = () => {
        try {
          fit.fit()
          void ptyResize(sessionId, term.cols, term.rows).catch(() => {})
        } catch { /* 不可见时跳过 */ }
      }
      const ro = new ResizeObserver(doFit)
      ro.observe(containerRef)
      window.addEventListener('resize', doFit)
      onCleanup(() => {
        ro.disconnect()
        window.removeEventListener('resize', doFit)
      })

      setStatus('live')
      term.focus()
    } catch (e) {
      setErrMsg(String(e))
      setStatus('error')
    }

    onCleanup(() => {
      for (const fn of unlisteners) fn()
      if (sessionId) void ptyClose(sessionId).catch(() => {})
      term.dispose()
    })
  })

  return (
    <div class="fixed right-0 top-7 bottom-0 w-[480px] z-40 bg-[#1a1a2e] border-l border-white/10 flex flex-col shadow-2xl">
      {/* 标题栏 */}
      <div class="flex items-center gap-2 px-3 py-1.5 border-b border-white/10 flex-shrink-0">
        <span class="text-11px font-medium text-zinc-300">终端</span>
        <span
          classList={{
            'w-1.5 h-1.5 rounded-full': true,
            'bg-emerald-400 animate-pulse': status() === 'live',
            'bg-amber-400 animate-pulse': status() === 'connecting',
            'bg-zinc-500': status() === 'exited',
            'bg-red-500': status() === 'error',
          }}
        />
        <span class="text-10px text-zinc-500">
          {status() === 'live' && 'PTY 已连接'}
          {status() === 'connecting' && '连接中…'}
          {status() === 'exited' && '进程退出'}
          {status() === 'error' && `错误: ${errMsg()}`}
        </span>
      </div>
      {/* xterm 挂载点 */}
      <div ref={containerRef} class="flex-1 min-h-0 px-1 py-1" />
      <Show when={status() !== 'live' && status() !== 'connecting'}>
        <button
          class="absolute inset-0 flex items-center justify-center bg-black/50 text-sm text-zinc-300 hover:text-white"
          onClick={() => window.location.reload()}
          title="重启应用以恢复终端"
        >
          终端已断开 — 重启应用恢复
        </button>
      </Show>
    </div>
  )
}
