import { createSignal, createEffect, onMount, onCleanup, For, Show } from 'solid-js'
import { Shield, MousePointer2, Edit3, FileText, ChevronDown, Check } from 'lucide-solid'
import { clsx } from 'clsx'

export type PermissionMode = 'auto' | 'manual' | 'accept_edits' | 'plan'

export interface PermissionModeOption {
  value: PermissionMode
  label: string
  description: string
  icon: any
  color: string
}

export const PERMISSION_MODES: PermissionModeOption[] = [
  {
    value: 'auto',
    label: '自动',
    description: 'AI 自动决定何时需要确认',
    icon: Shield,
    color: 'text-nt-io-600'
  },
  {
    value: 'manual',
    label: '手动',
    description: '所有工具调用都需要用户确认',
    icon: MousePointer2,
    color: 'text-blue-400'
  },
  {
    value: 'accept_edits',
    label: '接受编辑',
    description: '自动接受文件编辑，其余需确认',
    icon: Edit3,
    color: 'text-emerald-600'
  },
  {
    value: 'plan',
    label: '规划模式',
    description: '只读模式，仅生成计划不执行',
    icon: FileText,
    color: 'text-amber-600'
  }
]

export interface PermissionModeSelectorProps {
  value: PermissionMode
  onChange: (mode: PermissionMode) => void
  disabled?: boolean
  compact?: boolean
}

export function PermissionModeSelector(props: PermissionModeSelectorProps) {
  const { value, onChange, disabled = false, compact = false } = props
  const [isOpen, setIsOpen] = createSignal(false)

  // Esc 关闭下拉（对标 Codex 下拉规范）
  onMount(() => window.addEventListener('keydown', handleEsc))
  onCleanup(() => window.removeEventListener('keydown', handleEsc))
  const handleEsc = (e: KeyboardEvent) => {
    if (e.key === 'Escape') setIsOpen(false)
  }

  const currentMode = PERMISSION_MODES.find(m => m.value === value) || PERMISSION_MODES[0]

  // 下拉面板 ref：焦点管理 / 方向键导航均限定在本面板内（避免多 listbox 全局串扰）
  let panelRef: HTMLDivElement | undefined

  // 打开后焦点移入列表并高亮当前项（对标 Codex 下拉规范）
  createEffect(() => {
    if (!isOpen()) return
    requestAnimationFrame(() => {
      const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
      if (opts.length === 0) return
      const active = opts.find(o => o.getAttribute('aria-selected') === 'true')
      const target = active && !active.hasAttribute('disabled')
        ? active
        : opts.find(o => !o.hasAttribute('disabled'))
      target?.focus()
    })
  })

  const handleSelect = (mode: PermissionMode) => {
    if (!disabled) {
      onChange(mode)
      setIsOpen(false)
    }
  }

  // 方向键 roving 导航（compact / 完整版共用；限定本面板内）
  const handleOptionKeyDown = (e: KeyboardEvent, i: number) => {
    if (e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return
    e.preventDefault()
    const dir = e.key === 'ArrowDown' ? 1 : -1
    // roving tabindex：聚焦下一选项（高亮跟随焦点）
    requestAnimationFrame(() => {
      const opts = Array.from(panelRef?.querySelectorAll<HTMLElement>('[role="option"]') ?? [])
      opts[(i + dir + PERMISSION_MODES.length) % PERMISSION_MODES.length]?.focus?.()
    })
  }

  if (compact) {
    return (
      <div class="relative">
        <button
          class={clsx(
            'flex items-center gap-2 px-3 py-2 rounded-lg border border-white/40 bg-white/40',
            'text-text-primary hover:bg-white/60 transition-colors backdrop-blur-sm',
            'min-w-[120px] max-w-[180px] focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
            disabled && 'opacity-50 cursor-not-allowed'
          )}
          onClick={() => !disabled && setIsOpen(!isOpen())}
          disabled={disabled}
          aria-label="权限模式"
          aria-expanded={isOpen()}
          aria-haspopup="listbox"
        >
          <currentMode.icon class={clsx('w-4 h-4 flex-shrink-0', currentMode.color)} />
          <span class="text-sm font-medium truncate">{currentMode.label}</span>
          <ChevronDown class={clsx('w-4 h-4 text-text-muted flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
        </button>

        <Show when={isOpen()}>
          <div ref={panelRef} class="absolute bottom-full left-0 mb-2 glass-pop border border-white/50 rounded-xl shadow-xl overflow-hidden z-50 animate-in min-w-[180px]" role="listbox" aria-label="权限模式选择">
            <For each={PERMISSION_MODES}>
              {(mode: PermissionModeOption, i) => (
                <button
                  class={clsx(
                    'w-full flex items-center gap-3 px-3 py-3 text-left transition-colors',
                    'hover:bg-bg-tertiary focus-visible:bg-bg-tertiary focus-visible:outline-none',
                    mode.value === value && 'bg-nt-io-500/10 text-nt-io-600'
                  )}
                  onClick={() => handleSelect(mode.value)}
                  disabled={disabled}
                  role="option"
                  aria-selected={mode.value === value}
                  onKeyDown={(e) => handleOptionKeyDown(e, i())}
                >
                  <mode.icon class={clsx('w-4 h-4 flex-shrink-0', mode.color)} />
                  <div class="flex-1 min-w-0 flex flex-col gap-1">
                    <span class="font-medium truncate">{mode.label}</span>
                    <span class="text-xs text-text-muted truncate">{mode.description}</span>
                  </div>
                  {mode.value === value && <Check class="w-4 h-4 text-nt-io-500 flex-shrink-0" />}
                </button>
              )}
            </For>
          </div>
        </Show>

        <div 
          class={isOpen() ? 'fixed inset-0 z-40' : 'hidden'} 
          onClick={() => setIsOpen(false)}
          aria-hidden="true"
        />
      </div>
    )
  }

  // Full width version (for settings drawer, etc.)
  return (
    <div class="relative w-full">
      <button
        class={clsx(
          'w-full flex items-center gap-3 px-3 py-3 rounded-lg border border-white/40 bg-white/40',
          'text-text-primary hover:bg-white/60 transition-colors text-left backdrop-blur-sm',
          'focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none',
          disabled && 'opacity-50 cursor-not-allowed'
        )}
        onClick={() => !disabled && setIsOpen(!isOpen())}
        disabled={disabled}
        aria-label="权限模式"
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
      >
        <currentMode.icon class={clsx('w-4 h-4 flex-shrink-0', currentMode.color)} />
        <div class="flex-1 min-w-0 flex flex-col gap-1">
          <span class="font-medium truncate">{currentMode.label}</span>
          <span class="text-xs text-text-muted truncate">{currentMode.description}</span>
        </div>
        <ChevronDown class={clsx('w-4 h-4 text-text-muted flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
      </button>

      <Show when={isOpen()}>
        <div ref={panelRef} class="absolute top-full left-0 right-0 mt-2 glass-pop border border-white/50 rounded-xl shadow-xl overflow-hidden z-50 animate-in" role="listbox" aria-label="权限模式选择">
          <For each={PERMISSION_MODES}>
            {(mode: PermissionModeOption, i) => (
              <button
                class={clsx(
                  'w-full flex items-center gap-3 px-3 py-3 text-left transition-colors',
                  'hover:bg-bg-tertiary focus-visible:bg-bg-tertiary focus-visible:outline-none',
                  mode.value === value && 'bg-nt-io-500/10 text-nt-io-600'
                )}
                onClick={() => handleSelect(mode.value)}
                disabled={disabled}
                role="option"
                aria-selected={mode.value === value}
                onKeyDown={(e) => handleOptionKeyDown(e, i())}
              >
                <mode.icon class={clsx('w-4 h-4 flex-shrink-0', mode.color)} />
                <div class="flex-1 min-w-0 flex flex-col gap-1">
                  <span class="font-medium truncate">{mode.label}</span>
                  <span class="text-xs text-text-muted truncate">{mode.description}</span>
                </div>
                {mode.value === value && <Check class="w-4 h-4 text-nt-io-500 flex-shrink-0" />}
              </button>
            )}
          </For>
        </div>
      </Show>

      <div 
        class={isOpen() ? 'fixed inset-0 z-40' : 'hidden'} 
        onClick={() => setIsOpen(false)}
        aria-hidden="true"
      />
    </div>
  )
}
