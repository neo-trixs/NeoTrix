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

  // 每次 req 切换（打开/换内容）同步输入初值
  createEffect(() => {
    const r = props.req
    setInputVal(r?.initialValue ?? '')
  })

  const submit = () => props.onConfirm(inputVal())

  return (
    <Show when={props.req}>
      {(r) => (
        <div class="fixed inset-0 z-[100] flex items-center justify-center p-4">
          <div class="absolute inset-0 bg-black/20" onClick={props.onClose} />
          <div
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
                autofocus
                class="w-full px-3 py-2 rounded-lg border border-border-primary bg-white/70 text-[13px] text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-2 focus:ring-nt-io-500"
                onKeyDown={(e) => {
                  if (e.key === 'Enter') submit()
                  if (e.key === 'Escape') props.onClose()
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