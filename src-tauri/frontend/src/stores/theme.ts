import { createSignal, createEffect, onMount } from 'solid-js'

export type ThemeMode = 'light' | 'dark' | 'system'

const STORAGE_KEY = 'neotrix-theme-mode'

const getSystemMode = (): 'light' | 'dark' => {
  if (typeof window !== 'undefined' && window.matchMedia?.('(prefers-color-scheme: dark)').matches) {
    return 'dark'
  }
  return 'light'
}

const resolveMode = (mode: ThemeMode): 'light' | 'dark' =>
  mode === 'system' ? getSystemMode() : mode

const [themeMode, _setThemeMode] = createSignal<ThemeMode>('system')
const [resolvedMode, setResolvedMode] = createSignal<'light' | 'dark'>('light')

const applyTheme = () => {
  const resolved = resolveMode(themeMode())
  setResolvedMode(resolved)
  document.documentElement.setAttribute('data-theme-mode', resolved)
}

export const setThemeMode = (mode: ThemeMode) => {
  _setThemeMode(mode)
  applyTheme()
  try { localStorage.setItem(STORAGE_KEY, mode) } catch { /* privacy mode */ }
}

export const cycleThemeMode = () => {
  const modes: ThemeMode[] = ['light', 'dark', 'system']
  const next = modes[(modes.indexOf(themeMode()) + 1) % modes.length]
  setThemeMode(next)
}

const MODE_LABEL: Record<ThemeMode, string> = { light: '浅色', dark: '深色', system: '跟随系统' }
export const themeModeLabel = () => MODE_LABEL[themeMode()]

onMount(() => {
  try {
    const saved = localStorage.getItem(STORAGE_KEY) as ThemeMode | null
    if (saved && ['light', 'dark', 'system'].includes(saved)) {
      _setThemeMode(saved)
    }
  } catch { /* privacy mode */ }

  applyTheme()

  if (typeof window !== 'undefined' && window.matchMedia) {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    mq.addEventListener('change', () => {
      if (themeMode() === 'system') applyTheme()
    })
  }
})

export { themeMode, resolvedMode }
