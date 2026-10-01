/**
 * 自持 UI 的 i18n —— R4。
 *
 * # 为什么不用 vendored 的那份词条
 *
 * `neobot-ui/src/i18n/` 原先逐字复制自 `frontend/src/i18n/locales/`，
 * 实测那两份是**上游 DSH 的 452 个扁平键**（`app.wordmark` / `status.loading` …），
 * 而**我方自持 UI 一个 chat 词条都没有** —— `neobot-root.tsx` 里是
 * 24 处硬编码中文字面量。
 *
 * ⇒ 复用它等于「词条齐全但一条都用不上」。故本目录改为**我方自有词条集**，
 *   键随自持 UI 的真实文案走。旧的两份 452 键文件已移出（见 git 历史）。
 *
 * # 契约
 *
 * `set_language` 是契约表里 `Status::Implemented` 且已注册的持久化命令
 * （api.rs:277，参数 `lang`）。语言选择**必须经它落到后端**，
 * 否则下次启动会回退 —— 只存 localStorage 是不够的。
 *
 * # 与 openghost 的联动（这里的切换是**已接线**的，非空壳）
 *
 * `vendor/openghost/shim.ts` 的 `lang()` 读
 * `document.documentElement.lang` 决定代码块复制按钮的 aria-label
 * （`code.copy`）。`setLang()` 同步写该属性 ⇒ 切语言会**立刻**改变
 * 复制按钮的可访问名。这是切换器当前**可观测**的效果。
 */
import { invokeCmd } from '../ipc'
import { useEffect, useReducer } from 'react'
import zhCN from './locales/zh-CN.json'
import enUS from './locales/en-US.json'

export type Lang = 'zh-CN' | 'en-US'

const TABLES: Record<Lang, Record<string, string>> = {
  'zh-CN': zhCN as Record<string, string>,
  'en-US': enUS as Record<string, string>,
}

const STORAGE_KEY = 'neobot.lang'

/** 后端契约里的语言取值。保持与 `set_language` 实现一致。 */
function normalize(raw: string | null | undefined): Lang {
  return raw && raw.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

function detectInitial(): Lang {
  // 优先级：用户显式选择 > 浏览器语言。
  // ⛔ 不用 `navigator.language` 作唯一来源：Tauri 窗口环境下它可能与
  //    用户实际选择不一致，而 `set_language` 已把选择持久化到后端。
  const stored = readStored()
  if (stored) return normalize(stored)
  return normalize(typeof navigator !== 'undefined' ? navigator.language : null)
}

function readStored(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY)
  } catch {
    // localStorage 在受限上下文可能抛（隐私模式 / file:// 某些实现）
    return null
  }
}

let current: Lang = detectInitial()

/** 当前语言。 */
export function getLang(): Lang {
  return current
}

/** 可选语言列表（供切换器渲染）。 */
export function availableLangs(): { value: Lang; label: string }[] {
  return [
    { value: 'zh-CN', label: TABLES['zh-CN']['lang.name'] ?? '中文' },
    { value: 'en-US', label: TABLES['en-US']['lang.name'] ?? 'English' },
  ]
}

/**
 * 取词条。**诚实回退**：缺键时返回键名本身，不返回空串 ——
 * 空串会让按钮/标签**失名**，而键名至少可被搜索定位。
 *
 * @param vars 插值变量，替换 `{name}` 占位符。
 *
 * ⚠️ 缺变量时**保留占位符原样**（输出 `撤一版（{n}）`）而**不是**塞空串或
 * `undefined`：前者一眼看得出是词条/调用不匹配，后者会渲染成
 * 「撤一版（）」这种看不出错的坏 UI。这与「缺键回键名」是同一条原则。
 */
export function t(key: string, vars?: Record<string, string | number>): string {
  const raw = TABLES[current][key] ?? key
  if (!vars) return raw
  return raw.replace(/\{(\w+)\}/g, (whole, name: string) =>
    Object.prototype.hasOwnProperty.call(vars, name) ? String(vars[name]) : whole,
  )
}

/**
 * 切换语言。
 *
 * 三处同步，缺一不可：
 *   ① 后端 `set_language` —— 持久化的**唯一真源**（跨启动）
 *   ② localStorage —— 同会话快速渲染，避免等 IPC 往返
 *   ③ `document.documentElement.lang` —— openghost 垫片读它决定
 *      代码块复制按钮的 aria-label
 *
 * 后端失败**不静默**：回滚到原语言并抛出，让调用方能提示用户。
 */
export async function setLang(next: Lang): Promise<void> {
  const target = normalize(next)
  if (target === current) return
  const prev = current
  current = target
  applyDocumentLang(target)
  try {
    localStorage.setItem(STORAGE_KEY, target)
  } catch {
    /* 存不下不影响本次会话，后端已持久化 */
  }
  try {
    await invokeCmd('set_language', { lang: target })
  } catch (e) {
    current = prev // 回滚：不让 UI 显示一个后端没记住的语言
    applyDocumentLang(prev)
    throw e
  }
  // 通知订阅者重渲染（切换器高亮、词条刷新）
  for (const fn of listeners) fn(target)
}

function applyDocumentLang(lang: Lang): void {
  if (typeof document !== 'undefined') document.documentElement.lang = lang
}

const listeners = new Set<(l: Lang) => void>()

/** 订阅语言变化（返回取消订阅函数）。 */
export function onLangChange(fn: (l: Lang) => void): () => void {
  listeners.add(fn)
  return () => listeners.delete(fn)
}

/**
 * 订阅语言变化并**强制重渲染**的 `t`。
 *
 * ⛔ 只导出模块级 `t` 是不够的 —— 那是「导出 ≠ 接入」的又一形态：
 *    `setLang` 确实会通知订阅者，但若**只有外壳**订阅，聊天区组件
 *    永不重渲染，它的 `t()` 调用就不会重新求值 ⇒ 语言切换器**只换外壳、
 *    不换正文**，是个半成品（实测：切 en-US 后聊天区仍是中文）。
 *
 * 用法：组件顶部 `const t = useT()`。它返回的 `t` 读的是**实时** `current`，
 * 而模块级函数（如 `relTime`）继续用导入的 `t` 即可 —— 二者读同一个 `current`，
 * 只要组件重渲染，渲染期内的模块级调用同样拿到新语言。
 */
export function useT(): typeof t {
  const [, force] = useReducer((x: number) => x + 1, 0)
  useEffect(() => onLangChange(() => force()), [])
  return t
}

// 模块加载即同步 document lang —— 与 main.tsx 的 applyDocumentLang 同义，
// 但放在这里可保证**任何** import 本模块的路径都得到一致的语言属性。
applyDocumentLang(current)
