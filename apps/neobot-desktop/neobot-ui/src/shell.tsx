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
import { invokeCmd as invoke } from './ipc'
import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useRef, useState } from 'react'

import { loadApiPanel } from './api-panel'
import { getLang, onLangChange, setLang, t, availableLangs, type Lang } from './i18n'
import { clearActivity, getActivity, onActivity, type ActivityEntry } from './ipc'
import { setThemeMode, useThemeBootstrap, type ThemeMode } from './theme-mode'
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
  'desktop-about': '关于',
  'desktop-check-update': '检查更新',
  'desktop-restart': '重启',
  'desktop-new-window': '新建窗口',
  'desktop-new-chat': '新聊天',
  'desktop-open-folder': '打开文件夹',
  'desktop-documentation': '文档',
}

function useMacosMenu(onLogs: () => void, onNote: (msg: string) => void, onSettings: () => void) {
  useEffect(() => {
    if (!navigator.userAgent.includes('Macintosh')) return
    let stop: (() => void) | undefined
    void listen<string>('macos-menu-action', (event) => {
      const id = event.payload
      if (id === 'desktop-copy-run-logs') {
        onLogs()
        return
      }
      if (id === 'desktop-config') {
        onSettings()
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
  }, [onLogs, onNote, onSettings])
}

/** 顶栏：字标 + 语言切换 + 日志 + 退出。 */
export function Shell({ children }: { children?: React.ReactNode }) {
  const [lang, setLangState] = useState<Lang>(getLang)
  const [logs, setLogs] = useState<string | null>(null)
  // 活动面板：吸收 UI-TARS 的 Event Stream Viewer / 工具调用耗时统计。
  // 订阅式更新 —— 新调用进来时面板**自己**刷新，不靠轮询。
  const [activity, setActivity] = useState<ActivityEntry[]>(() => getActivity())
  useEffect(() => {
    setActivity(getActivity())
    return onActivity(() => setActivity(getActivity()))
  }, [])
  const [busy, setBusy] = useState(false)
  // 深/浅切换：把 theme.css 里已移植但够不着的浅色盘接活
  const theme = useThemeBootstrap()
  // 顶栏内联错误位：⛔ 不能没有它。
  //   原实现在 setLang 失败时 `catch {}` 空处理，注释还写着
  //   「不静默：语言没切成功就是没切成功」—— **注释与代码自相矛盾**：
  //   用户点了语言、界面静默回退、零解释，只能反复点。
  const [barErr, setBarErr] = useState<string | null>(null)

  // 设置：数据源是后端契约面板（`api-panel.ts` 此前是**孤儿文件** ——
  // 没有任何入口引用它，于是 macOS「设置…」只能回一句「尚未接入」）。
  // 把它接成真正的设置面：既给了顶栏一个设置入口，也让那份清单有归宿。
  const [settings, setSettings] = useState(false)
  const settingsHost = useRef<HTMLDivElement | null>(null)
  const openSettings = useCallback(() => setSettings(true), [])
  const closeSettings = useCallback(() => setSettings(false), [])
  const settingsModal = useModalBehaviour(settings, closeSettings)
  useEffect(() => {
    if (settingsHost.current) void loadApiPanel(settingsHost.current)
  }, [settings])

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
  useMacosMenu(() => void openLogs(), noteMenu, openSettings)

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

  // ⭐⭐⭐ 平台标记：macOS 的**交通灯**（红黄绿）占标题栏左侧，
  // 而 `tauri.conf.json` 用的是 `titleBarStyle: "Overlay"` + `hiddenTitle: true`
  // ⇒ ⭐⭐ **webview 内容会延伸到交通灯之下** ⇒ 不让位就会**视觉重叠**。
  //
  // ⭐⭐ 判定口径**刻意与本文件既有的 `useMacosMenu` 完全一致**
  //（`navigator.userAgent.includes('Macintosh')`）——
  // ⭐⭐ **同一份平台判定只留一处口径**，⛔ 不引入第二套（否则两处会漂移）。
  const isMac = typeof navigator !== 'undefined'
    && navigator.userAgent.includes('Macintosh')

  return (
    // ⭐⭐ `data-platform` 供 CSS 定向让位；⛔ 不用 JS 改 style
    //（⭐ 那样每次语言/主题重渲染都要重算，⭐ 且无法被 CSS 媒体查询覆盖）
    <div className="nb-shell" data-platform={isMac ? 'macos' : 'other'}>
      <header className="nb-bar">
        <div className="nb-brand">
          <span className="nb-wordmark">{t('shell.wordmark')}</span>
          <span className="nb-tagline">{t('shell.tagline')}</span>
        </div>

        <div className="nb-actions">

          {/* ⭐ 2026-10-07 补 testid：`neobot-ui-smoke.mjs` 此前用
              `.nb-actions button` 的**位置**点它（第一个），而同一按钮组里
              后面又加了设置/退出 ⇒ 位置会漂；且门在 act 里可能**已打开设置弹窗**，
              位置点击会被遮罩吞掉 ⇒ 活动面板读不到失败行。
              ⇒ 按邻里的 `nb-settings-open` 同一办法给稳定钩子。*/}
          <button type="button" data-testid="nb-logs-open" disabled={busy} onClick={() => void openLogs()}>
            {t('shell.openLogs')}
          </button>
          <span className="nb-actions-sep" aria-hidden="true" />
          <button type="button" className="nb-primary" data-testid="nb-settings-open" onClick={openSettings}>
            {t('shell.settings')}
          </button>
          <button type="button" className="nb-danger" disabled={busy} onClick={() => void invoke('quit_app')}>
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

      {/* 设置：内容是**后端契约面板**（读 `neobot_api_specs`，不硬编码清单）。
          此前 `api-panel.ts` 是孤儿文件 —— 没有任何入口引用它，
          于是 macOS「设置…」只能回「尚未接入」，顶栏也没有设置入口。 */}
      {settings && (
        <div
          className="nb-modal"
          onClick={(e) => {
            if (e.target === e.currentTarget) closeSettings()
          }}
        >
          <section
            className="nb-modal-box nb-modal-wide"
            role="dialog"
            aria-modal="true"
            aria-label={t('shell.settings')}
            ref={settingsModal}
          >
            <header>
              <strong>{t('shell.settings')}</strong>
              <button type="button" data-testid="nb-settings-close" data-autofocus onClick={closeSettings}>
                {t('shell.logs.close')}
              </button>
            </header>
            {/* ⭐⭐⭐ 必须**放在 `settingsHost` 外面**（实测依据，非猜）：
             * `api-panel.ts:64` / `:74` 的 `loadApiPanel` 用
             * `host.replaceChildren(...)` ⇒ ⭐⭐ **每次打开设置都清空整个 host**
             * ⇒ 塞进 host 的控件会**被静默抹掉且不报错**。
             * ⭐⭐ 控件**原文搬移**（⛔ 不是重写）：`data-testid` 与
             * `availableLangs()` 的动态选项都原样保留 —— 3 个 smoke 脚本
             * 9 处靠 testid 驱动，其中 6 处带 `.catch(() => {})`，
             * ⭐⭐ **改名/丢失 ⇒ 测试静默 no-op 而非变红**。*/}
            <section className="nb-prefs" aria-label={t('shell.theme')}>
              <label className="nb-pref">
              <span className="nb-pref-label">{t('shell.theme')}</span>
              <select
                data-testid="nb-theme-select"
                value={theme}
                  onChange={(e) => setThemeMode(e.target.value as ThemeMode)}
              >
                <option value="system">{t('shell.themeSystem')}</option>
                <option value="dark">{t('shell.themeDark')}</option>
                <option value="light">{t('shell.themeLight')}</option>
              </select>
            </label>
              <label className="nb-pref">
              <span className="nb-pref-label">{t('shell.language')}</span>
              <select
                data-testid="nb-lang-select"
                value={lang}
                  onChange={(e) => void choose(e.target.value as Lang)}
              >
                {availableLangs().map((l) => (
                  <option key={l.value} value={l.value}>
                    {l.label}
                  </option>
                ))}
              </select>
            </label>
            </section>
            <div className="nb-scroll nb-settings-body" ref={settingsHost} />
          </section>
        </div>
      )}

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
            {/* ── 活动面板（先于原始日志）：失败时**先看这里** ──
                它能回答两个日志文本回答不了的问题：
                「是哪条命令失败」与「它耗时多久」。 */}
            <section className="nb-act">
              <header className="nb-act-head">
                <strong>{t('shell.activity')}</strong>
                <span className="nb-act-count">{activity.length}</span>
                <button type="button" onClick={() => { clearActivity(); setActivity([]) }}>
                  {t('shell.activityClear')}
                </button>
              </header>
              {activity.length === 0 ? (
                <p className="nb-act-empty">{t('shell.activityEmpty')}</p>
              ) : (
                <ol className="nb-act-list" tabIndex={0}>
                  {[...activity].reverse().map((e, i) => (
                    <li key={`${e.at}-${i}`} className={e.ok ? 'ok' : 'bad'}>
                      <code>{e.cmd}</code>
                      <span className="nb-act-ms">{e.ms} ms</span>
                      <span className="nb-act-st">{e.ok ? '✓' : '✗'}</span>
                      {e.error !== undefined && (
                        <span className="nb-act-err">{e.error}</span>
                      )}
                    </li>
                  ))}
                </ol>
              )}
            </section>
            <h4 className="nb-logs-title">{t('shell.logs.title')}</h4>
            {/* 日志正文可滚：让它可聚焦，键盘用户才能滚动读日志 */}
            <pre tabIndex={0}>{logs.trim() ? logs : t('shell.logs.empty')}</pre>
          </section>
        </div>
      )}
    </div>
  )
}
