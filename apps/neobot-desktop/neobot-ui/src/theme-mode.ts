/**
 * 深/浅主题切换 —— 把**已移植但够不着**的浅色盘接活。
 *
 * # 发现的缺陷
 *
 * `theme.css` 里有一整块 `html[data-theme="light"]`（38 行浅色取值，
 * 随上游一并移植过来），但**全仓没有任何代码设置 `data-theme`** ——
 * 连注释都在说「随 `html[data-theme]` 翻色」，而那个属性**永远是空的**
 * ⇒ 浅色盘是**死代码**。契约表里也只有 `get_dsh_theme`（**能报、不能改**），
 * 没有 `set_theme` ⇒ 后端亦无写入口。
 *
 * # 取舍（诚实说明）
 *
 * · 初始化**读** `get_dsh_theme`：它已 `Status::Implemented` 且已注册
 *   （`main.rs`），是我方 Rust 函数，不是上游代码。
 *   ⛔ 但名字里的 `dsh` 是**概念泄漏**（DSH = DeepSeek Harness）——
 *   自持 UI 不该依赖一个 DSH 命名的 API。这是**记在案的命名债**，
 *   改名涉及 Rust 侧与契约表，**不在前端单方面改**（那会造成两套真源）。
 * · 切换本身**只写 `data-theme` + localStorage**：真源在前端，
 *   不绕后端。因为后端**没有**写入口，硬造一个会是假接口。
 *
 * 设计取舍：`system` 档用 `matchMedia('(prefers-color-scheme: light)')`
 * 跟随系统，并**监听变化**——桌面应用该跟随系统，用户不该被迫手选。
 */
import { useEffect, useState } from 'react'
import { invokeCmd } from './ipc'

export type ThemeMode = 'dark' | 'light' | 'system'

const KEY = 'neobot.theme'
const listeners = new Set<(m: ThemeMode) => void>()

function read(): ThemeMode {
  try {
    const v = localStorage.getItem(KEY)
    if (v === 'dark' || v === 'light' || v === 'system') return v
  } catch { /* 存不下就用默认 */ }
  return 'system'
}

/** 把 mode 落到 DOM：`system` 需先问系统。 */
function apply(mode: ThemeMode): 'dark' | 'light' {
  const eff = mode === 'system'
    ? (typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: light)').matches
        ? 'light' : 'dark')
    : mode
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', eff)
    // ⛔ 同时设 color-scheme：否则滚动条/表单控件仍是系统浅色，
    //    与深色内容并排出现（macOS WKWebView 尤其明显）。
    document.documentElement.style.colorScheme = eff
  }
  return eff
}

let current: ThemeMode = read()
apply(current)

export function getThemeMode(): ThemeMode {
  return current
}

export function onThemeChange(fn: (m: ThemeMode) => void): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}

export function setThemeMode(next: ThemeMode): void {
  if (next === current) return
  current = next
  apply(next)
  try { localStorage.setItem(KEY, next) } catch { /* 存不下不影响本次会话 */ }
  for (const fn of listeners) fn(next)
}

/**
 * 挂载时用后端的 `get_dsh_theme` 校准一次。
 * ⛔ **失败不阻塞**：主题不是关键路径，取不到就用前端默认（system）。
 *   「主题读不到」不该让整个界面起不来。
 */
export function useThemeBootstrap(): ThemeMode {
  const [mode, setMode] = useState<ThemeMode>(current)
  useEffect(() => onThemeChange(setMode), [])
  useEffect(() => {
    let alive = true
    void invokeCmd<string | null>('get_dsh_theme')
      .then((v: string | null) => {
        if (!alive) return
        // 后端只认 dark/light/system；'system' 时保留前端的跟随逻辑
        if (v === 'dark' || v === 'light' || v === 'system') setThemeMode(v)
      })
      .catch(() => { /* 保持前端默认，不报错 */ })
    return () => { alive = false }
  }, [])
  // 跟随系统：系统切换时重算（仅 system 档需要）
  useEffect(() => {
    if (mode !== 'system' || typeof matchMedia === 'undefined') return
    const mq = matchMedia('(prefers-color-scheme: light)')
    const onChg = () => apply('system')
    mq.addEventListener('change', onChg)
    return () => mq.removeEventListener('change', onChg)
  }, [mode])
  return mode
}
