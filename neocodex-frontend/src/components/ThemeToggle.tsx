import { createSignal, onMount } from 'solid-js'

type ThemeMode = 'light' | 'dark' | 'auto'

export function ThemeToggle() {
  const [mode, setMode] = createSignal<ThemeMode>('auto')

  onMount(() => {
    const saved = localStorage.getItem('theme-mode') as ThemeMode
    if (saved) setMode(saved)
    applyTheme(mode())
  })

  const applyTheme = (m: ThemeMode) => {
    const isDark = m === 'dark' || (m === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    document.documentElement.setAttribute('data-theme-mode', isDark ? 'dark' : 'light')
    localStorage.setItem('theme-mode', m)
  }

  const cycle = () => {
    const next = mode() === 'light' ? 'dark' : mode() === 'dark' ? 'auto' : 'light'
    setMode(next)
    applyTheme(next)
  }

  return (
    <button onClick={cycle} class="p-2 rounded-md hover:bg-bg-tertiary transition-colors">
      {mode() === 'light' ? '☀️' : mode() === 'dark' ? '🌙' : '💻'}
    </button>
  )
}
