/**
 * 自持应用外壳 —— R2。
 *
 * # 范围（刻意最小）
 *
 * 只做**已实现且已注册**的后端命令所支撑的事，不画大饼：
 *   · 语言切换 → `set_language`（api.rs:277，持久化到后端）
 *   · 查看日志 → `read_run_logs`（api.rs:288）
 *   · 退出     → `quit_app`（api.rs:263）
 *   · 外链     → `open_external_url`（api.rs:159）
 *
 * ⛔ **不做**多窗口（`create_app_window` / `remote_open_window`）与
 *    `read_clipboard_image`（arboard 未开 image-data feature，后端**诚实报错**）。
 *    画一个点不动的按钮比不画更糟：那正是本仓「导出 ≠ 接入」的翻版。
 *
 * # 为什么 `read_clipboard_image` 不做
 *
 * 契约表里它标 `Status::Implemented` 但附注「⛔ arboard 未暴露像素读取
 * （image-data feature 未开）⇒ **诚实报错**而非返空 data URL」。
 * ⇒ 命令存在但**必然失败**。接它等于交付一个点了就报错的按钮。
 */
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'

import { getLang, onLangChange, setLang, t, availableLangs, type Lang } from './i18n'
import './shell.css'

/**
 * 拦截外链点击 → 交给后端 `open_external_url`。
 *
 * ⛔ 不用 `window.open`：在 Tauri 里那会开一个**应用外**的浏览器窗口，
 *    绕过应用生命周期与「外链须经审计」的约束。`open_external_url` 才是
 *    契约表登记的那条路。
 *
 * 导出成函数而非副作用注册：让 `main.tsx` 决定挂载时机（可测）。
 */
export function useExternalLinks(): void {
  useEffect(() => {
    const onClick = (e: MouseEvent) => {
      const a = (e.target as HTMLElement | null)?.closest?.('a[href]')
      if (!(a instanceof HTMLAnchorElement)) return
      const href = a.getAttribute('href') ?? ''
      if (!/^https?:\/\//i.test(href)) return
      e.preventDefault()
      void invoke('open_external_url', { url: href }).catch(() => {
        /* 外链被系统策略拒绝：不做二次提示，避免与聊天区提示打架 */
      })
    }
    document.addEventListener('click', onClick, true)
    return () => document.removeEventListener('click', onClick, true)
  }, [])
}

/**
 * 模态对话框的**真模态行为**。
 *
 * ⛔ 上一版只写了 `role="dialog" aria-modal="true"` 与一个用于点遮罩判断的
 *    ref，却**没有实现任何模态行为** —— 无 Esc 关闭、无初始焦点、无焦点陷阱、
 *    关闭后不还焦。那不是「细节没做完」，而是 **`aria-modal` 是一句谎话**：
 *    向辅助技术宣告模态，却不实现模态语义。
 *    后果具体：键盘用户打开日志后**按 Esc 出不来**，Tab 会跑到背后的
 *    顶栏与聊天区，焦点落到看不见的地方。
 *
 * 这里补齐四件：Esc 关闭 / 初始焦点 / Tab 陷阱 / 关闭后归还焦点给触发元素。
 */
function useModalBehaviour(open: boolean, onClose: () => void) {
  const boxRef = useRef<HTMLDivElement | null>(null)
  const restoreTo = useRef<HTMLElement | null>(null)

  useEffect(() => {
    if (!open) return
    // 记住打开前的焦点元素，关闭后归还（否则焦点掉回 body，键盘用户失位）
    restoreTo.current = document.activeElement as HTMLElement | null

    const box = boxRef.current
    // 初始焦点：优先取标记了 data-autofocus 的控件，否则退到关闭按钮。
    // ⛔ 不 autofocus 到日志正文：它是 <pre>，可聚焦但无操作价值。
    const first = (box?.querySelector('[data-autofocus]') as HTMLElement | null) ??
      (box?.querySelector('button') as HTMLElement | null)
    first?.focus()

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation() // 别让 Esc 同时冒泡去关掉别的层
        onClose()
        return
      }
      if (e.key !== 'Tab') return
      // Tab 陷阱：只在对话框内可用的元素间循环
      const items = box
        ? [...box.querySelectorAll<HTMLElement>(
            'button,select,input,textarea,a[href],[tabindex]:not([tabindex="-1"])',
          )].filter((el) => !el.hasAttribute('disabled') && el.offsetParent !== null)
        : []
      if (items.length === 0) {
        e.preventDefault()
        return
      }
      const firstEl = items[0]
      const lastEl = items[items.length - 1]
      const active = document.activeElement
      if (e.shiftKey && (active === firstEl || !box?.contains(active))) {
        e.preventDefault()
        lastEl.focus()
      } else if (!e.shiftKey && (active === lastEl || !box?.contains(active))) {
        e.preventDefault()
        firstEl.focus()
      }
    }
    document.addEventListener('keydown', onKey, true)
    return () => {
      document.removeEventListener('keydown', onKey, true)
      restoreTo.current?.focus?.()
    }
  }, [open, onClose])

  return boxRef
}


/**
 * macOS 原生菜单动作 —— 自研壳的接线。
 *
 * # 为什么必须有这段
 *
 * Rust 侧 `menu::install`（上游 `install_macos_menu` 移植）装的是**原生菜单栏**。
 * ⛔ 上游壳把「文件/运行/帮助」整组隐藏并假定由原生菜单承载，而原生菜单只发
 *    `macos-menu-action` 事件 —— **前端不 `listen` 就一个都收不到**。
 *    neobot-ui 落地时没有这段，于是 macOS 上：菜单在、点了没反应。
 *
 * # 诚实边界
 *
 * ⛔ 只接**本壳真有实现**的动作；其余一律回一句「尚未接入」，
 *    而不是静默 return —— 静默 return 与「菜单项不存在」在用户眼里一样，
 *    但后者他能看见这个功能存在（那是产品缺口，不是 bug）。
 *    已接：`desktop-copy-run-logs`（顶栏已有日志面板，直接复用）。
 */
const MENU_NOT_WIRED: Record<string, string> = {
  'desktop-config': '设置面板',
  'desktop-about': '关于',
  'desktop-check-update': '检查更新',
  'desktop-restart': '重启',
  'desktop-new-window': '新建窗口',
  'desktop-new-chat': '新聊天',
  'desktop-open-folder': '打开文件夹',
  'desktop-documentation': '文档',
}

function useMacosMenu(onLogs: () => void, onNote: (msg: string) => void) {
  useEffect(() => {
    if (!navigator.userAgent.includes('Macintosh')) return
    let stop: (() => void) | undefined
    void listen<string>('macos-menu-action', (event) => {
      const id = event.payload
      if (id === 'desktop-copy-run-logs') {
        onLogs()
        return
      }
      const name = MENU_NOT_WIRED[id]
      // 未登记的 id：也说一句。静默是最差的一种。
      onNote(name ? `「${name}」在自研版尚未接入` : `菜单动作 ${id} 尚未接入`)
    }).then((un) => {
      stop = un
    }).catch((e) => {
      onNote(`原生菜单接线失败：${String(e).slice(0, 80)}`)
    })
    return () => stop?.()
  }, [onLogs, onNote])
}

/** 顶栏：字标 + 语言切换 + 日志 + 退出。 */
export function Shell({ children }: { children?: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>(getLang)
  const [logs, setLogs] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  // 顶栏内联错误位：⛔ 不能没有它。
  //   原实现在 setLang 失败时 `catch {}` 空处理，注释还写着
  //   「不静默：语言没切成功就是没切成功」—— **注释与代码自相矛盾**：
  //   用户点了语言、界面静默回退、零解释，只能反复点。
  const [barErr, setBarErr] = useState<string | null>(null)

  const [menuNote, setMenuNote] = useState<string | null>(null)
  const noteMenu = useCallback((msg: string) => {
    setMenuNote(msg)
    window.setTimeout(() => setMenuNote(null), 6000)
  }, [])
  useEffect(() => onLangChange(setLangState), [])

  const choose = useCallback(async (next: Lang) => {
    if (next === lang) return
    setBusy(true)
    try {
      setBarErr(null)
      await setLang(next) // 失败会自行回滚并抛出
    } catch (e) {
      // 说清「没切成」+ 为什么，**但用内联一行**而不是弹窗（顶栏空间小）。
      // `setLang` 已回滚，故这里只需如实报告，语言显示仍是当前生效的那个。
      setBarErr(t('shell.langFailed', { msg: String(e).slice(0, 80) }))
      window.setTimeout(() => setBarErr(null), 6000) // 自动消失，不长期占位
    } finally {
      setBusy(false)
    }
  }, [lang])

  // macOS 原生菜单 → 复用既有实现（日志面板）；其余动作给一句「尚未接入」。
  useMacosMenu(() => void openLogs(), noteMenu)

  const closeLogs = useCallback(() => setLogs(null), [])
  const dialogRef = useModalBehaviour(logs !== null, closeLogs)

  const openLogs = useCallback(async () => {
    setBusy(true)
    try {
      setLogs(await invoke<string>('read_run_logs'))
    } catch (e) {
      // 诚实显示失败，而不是显示空日志让人以为「没日志」
      setLogs(`⚠ ${String(e).slice(0, 300)}`)
    } finally {
      setBusy(false)
    }
  }, [])

  return (
    <div className="nb-shell">
      <header className="nb-bar">
        <div className="nb-brand">
          <span className="nb-wordmark">{t('shell.wordmark')}</span>
          <span className="nb-tagline">{t('shell.tagline')}</span>
        </div>

        <div className="nb-actions">
          <label className="nb-lang">
            <span className="nb-lang-label">{t('shell.language')}</span>
            <select
              value={lang}
              disabled={busy}
              onChange={(e) => void choose(e.target.value as Lang)}
            >
              {availableLangs().map((l) => (
                <option key={l.value} value={l.value}>
                  {l.label}
                </option>
              ))}
            </select>
          </label>

          <button type="button" disabled={busy} onClick={() => void openLogs()}>
            {t('shell.openLogs')}
          </button>
          <button type="button" disabled={busy} onClick={() => void invoke('quit_app')}>
            {t('shell.quit')}
          </button>
        </div>
      </header>

      {(barErr ?? menuNote) !== null && (
        <p className="nb-bar-err" role="alert">
          {barErr ?? menuNote}
        </p>
      )}

      <main className="nb-main">{children}</main>

      {/* 遮罩只是 backdrop，**不是** dialog —— role/aria 必须落在 box 上，
          否则辅助技术会把整块遮罩当成对话框，读屏体验是错的。
          （注释必须在 `{cond && (` **外面**：括号内 `&&` 只接受一个表达式，
            写成「注释 + 元素」是语法错误 —— 我第一版就这么写错了。） */}
      {logs !== null && (
        <div
          className="nb-modal"
          onClick={(e) => {
            if (e.target === e.currentTarget) closeLogs() // 点遮罩关闭
          }}
        >
          <section
            className="nb-modal-box"
            role="dialog"
            aria-modal="true"
            aria-label={t('shell.logs.title')}
            ref={dialogRef}
          >
            <header>
              <strong>{t('shell.logs.title')}</strong>
              <button type="button" data-autofocus onClick={closeLogs}>
                {t('shell.logs.close')}
              </button>
            </header>
            {/* 日志正文可滚：让它可聚焦，键盘用户才能滚动读日志 */}
            <pre tabIndex={0}>{logs.trim() ? logs : t('shell.logs.empty')}</pre>
          </section>
        </div>
      )}
    </div>
  )
}
