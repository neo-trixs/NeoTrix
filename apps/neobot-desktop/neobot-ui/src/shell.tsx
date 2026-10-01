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

/** 顶栏：字标 + 语言切换 + 日志 + 退出。 */
export function Shell({ children }: { children?: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>(getLang)
  const [logs, setLogs] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const dialogRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => onLangChange(setLangState), [])

  const choose = useCallback(async (next: Lang) => {
    if (next === lang) return
    setBusy(true)
    try {
      await setLang(next) // 失败会自行回滚并抛出
    } catch {
      // ⛔ 不静默：语言没切成功就是没切成功。
      // 但也不弹窗 —— 顶栏空间小，且聊天区已有错误位。保持按钮可再点即可。
    } finally {
      setBusy(false)
    }
  }, [lang])

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

      <main className="nb-main">{children}</main>

      {logs !== null && (
        <div
          className="nb-modal"
          role="dialog"
          aria-modal="true"
          aria-label={t('shell.logs.title')}
          ref={dialogRef}
          onClick={(e) => {
            if (e.target === dialogRef.current) setLogs(null) // 点遮罩关闭
          }}
        >
          <section className="nb-modal-box">
            <header>
              <strong>{t('shell.logs.title')}</strong>
              <button type="button" onClick={() => setLogs(null)}>
                {t('shell.logs.close')}
              </button>
            </header>
            <pre>{logs.trim() ? logs : t('shell.logs.empty')}</pre>
          </section>
        </div>
      )}
    </div>
  )
}
