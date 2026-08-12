import { Show, createEffect, createSignal } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   ConfirmModal — 统一确认/输入模态（对标 Codex 桌面体验）
   替换原生 window.confirm / window.prompt：
   - 纯 confirm：req 只含 title/message → 确认/取消两键
   - 含输入：req 含 inputLabel/initialValue → 渲染文本输入框，onConfirm 回传值
   - danger 选项：确认键红色（破坏性操作）
   ════════════════════════════════════════════ */

export interface ModalReq {
  title: string
  message?: string
  confirmLabel?: string
  cancelLabel?: string
  danger?: boolean
  /** 提供则渲染文本输入（重命名等 prompt 场景） */
  inputLabel?: string
  initialValue?: string
  placeholder?: string
}

interface Props {
  req: ModalReq | null
  onConfirm: (input?: string) => void
  onClose: () => void
}

export function ConfirmModal(props: Props) {
  const [inputVal, setInputVal] = createSignal('')
  let dialogRef: HTMLDivElement | undefined
  let confirmBtnRef: HTMLButtonElement | undefined
  // 打开前触发元素：关闭后还原焦点（对标 Codex 弹窗焦点规范）
  let restoreFocusEl: HTMLElement | null = null

  // 每次 req 切换（打开/换内容）同步输入初值 + 打开时聚焦首可聚焦元素
  createEffect(() => {
    const r = props.req
    setInputVal(r?.initialValue ?? '')
    if (r) {
      restoreFocusEl = document.activeElement as HTMLElement | null
      requestAnimationFrame(() => {
        if (r.inputLabel) {
          dialogRef?.querySelector<HTMLElement>('input')?.focus()
        } else {
          confirmBtnRef?.focus()
        }
      })
    } else {
      // 关闭后焦点还原到触发元素
      if (restoreFocusEl?.isConnected) restoreFocusEl.focus()
      restoreFocusEl = null
    }
  })

  // 键盘：Esc 关闭 + Tab 轻量焦点循环（限制在弹窗内，不外泄到背景）
  createEffect(() => {
    if (!props.req) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        props.onClose()
        return
      }
      if (e.key !== 'Tab' || !dialogRef) return
      const focusables = dialogRef.querySelectorAll<HTMLElement>('button, input, [href], [tabindex]:not([tabindex="-1"])')
      if (focusables.length === 0) return
      const first = focusables[0]
      const last = focusables[focusables.length - 1]
      const active = document.activeElement as HTMLElement | null
      // 焦点在弹窗外（背景/body）：Tab 拉回弹窗内
      if (!active || !dialogRef.contains(active)) {
        e.preventDefault()
        if (e.shiftKey) last.focus()
        else first.focus()
        return
      }
      if (e.shiftKey && active === first) {
        e.preventDefault()
        last.focus()
      } else if (!e.shiftKey && active === last) {
        e.preventDefault()
        first.focus()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  })

  const submit = () => props.onConfirm(inputVal())

  return (
    <Show when={props.req}>
      {(r) => (
        <div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-black/20" onClick={props.onClose} />
          <div
            ref={dialogRef}
            class="glass-modal relative w-full max-w-sm rounded-2xl p-5"
            role="dialog"
            aria-modal="true"
            aria-label={r().title}
          >
            <h3 class="text-[15px] font-semibold text-text-primary mb-1.5">{r().title}</h3>
            <Show when={r().message}>
              <p class="text-[13px] text-text-muted leading-relaxed">{r().message}</p>
            </Show>

            <Show when={r().inputLabel}>
              <label class="block mt-3 text-xs font-medium text-text-secondary mb-1.5">
                {r().inputLabel}
              </label>
              <input
                value={inputVal()}
                onInput={(e) => setInputVal((e.target as HTMLInputElement).value)}
                placeholder={r().placeholder ?? ''}
                class="w-full px-3 py-2 rounded-lg border border-border-primary bg-white/70 text-[13px] text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-nt-io-500"
                onKeyDown={(e) => {
                  if (e.key === 'Enter') submit()
                }}
              />
            </Show>

            <div class="flex justify-end gap-2 mt-5">
              <button
                class="px-3 py-1.5 rounded-lg text-[13px] text-text-secondary hover:bg-white/60 transition-colors"
                onClick={props.onClose}
              >
                {r().cancelLabel ?? '取消'}
              </button>
              <button
                ref={confirmBtnRef}
                class={clsx(
                  'px-3 py-1.5 rounded-lg text-[13px] font-medium text-white transition-colors',
                  r().danger
                    ? 'bg-red-600 hover:bg-red-500'
                    : 'bg-nt-io-600 hover:bg-nt-io-500'
                )}
                onClick={submit}
              >
                {r().confirmLabel ?? '确定'}
              </button>
            </div>
          </div>
        </div>
      )}
    </Show>
  )
}