/**
 * useThemeState — 主题切换 (gold/lilac/mint)
 */
import { createSignal, createEffect } from 'solid-js'

const THEMES = ['gold', 'lilac', 'mint'] as const
type Theme = (typeof THEMES)[number]

export function useThemeState() {
  const [theme, setTheme] = createSignal<Theme>(
    (typeof localStorage !== 'undefined' && (localStorage.getItem('nt-theme') as Theme)) ||
      (typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: light)').matches ? 'lilac' : 'gold'),
  )

  createEffect(() => {
    document.documentElement.dataset.theme = theme()
    try { localStorage.setItem('nt-theme', theme()) } catch { /* 隐私模式忽略 */ }
  })

  const cycleTheme = () => setTheme((t) => THEMES[(THEMES.indexOf(t) + 1) % THEMES.length])

  return { theme, cycleTheme }
}
