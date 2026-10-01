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
 * # 与 openghost 的联动（这��切换是**已接线**的，非空壳）
 *
 * `vendor/openghost/shim.ts` 的 `lang()` 读
 * `document.documentElement.lang` 决定代码块复制按钮的 aria-label
 * （`code.copy`）。`setLang()` 同步写该属性 ⇒ 切语言会**立刻**改变
 * 复制按钮的可访问名。这是切换器当前**可观测**的效果。
 */
import { invoke } from '@tauri-apps/api/core'
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
 */
export function t(key: string): string {
  return TABLES[current][key] ?? key
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
    await invoke('set_language', { lang: target })
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

// 模块加载即同步 document lang —— 与 main.tsx 的 applyDocumentLang 同义，
// 但放在这里可保证**任何** import 本模块的路径都得到一致的语言属性。
applyDocumentLang(current)
