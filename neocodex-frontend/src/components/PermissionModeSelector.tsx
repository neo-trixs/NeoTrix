import { createSignal, onMount, onCleanup, For } from 'solid-js'
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

  const handleSelect = (mode: PermissionMode) => {
    if (!disabled) {
      onChange(mode)
      setIsOpen(false)
    }
  }

  if (compact) {
    return (
      <div class="relative">
        <button
          class={clsx(
            'flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg border border-border-primary bg-bg-secondary',
            'text-text-primary hover:bg-bg-tertiary transition-colors',
            'min-w-[120px] max-w-[180px]',
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
          <ChevronDown class={clsx('w-3.5 h-3.5 text-text-muted flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
        </button>

        <Show when={isOpen()}>
          <div class="absolute bottom-full left-0 mb-1.5 bg-bg-secondary border border-border-primary rounded-xl shadow-xl overflow-hidden z-50 animate-in min-w-[180px]">
            <For each={PERMISSION_MODES}>
              {(mode: PermissionModeOption) => (
                <button
                  class={clsx(
                    'w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors',
                    'hover:bg-bg-tertiary',
                    mode.value === value && 'bg-nt-io-500/10'
                  )}
                  onClick={() => handleSelect(mode.value)}
                  disabled={disabled}
                  role="option"
                  aria-selected={mode.value === value}
                >
                  <mode.icon class={clsx('w-4 h-4 flex-shrink-0', mode.color)} />
                  <div class="flex-1 min-w-0 flex flex-col gap-0.5">
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
          'w-full flex items-center gap-3 px-3 py-2.5 rounded-lg border border-border-primary bg-bg-secondary',
          'text-text-primary hover:bg-bg-tertiary transition-colors text-left',
          disabled && 'opacity-50 cursor-not-allowed'
        )}
        onClick={() => !disabled && setIsOpen(!isOpen())}
        disabled={disabled}
        aria-label="权限模式"
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
      >
        <currentMode.icon class={clsx('w-5 h-5 flex-shrink-0', currentMode.color)} />
        <div class="flex-1 min-w-0 flex flex-col gap-0.5">
          <span class="font-medium truncate">{currentMode.label}</span>
          <span class="text-xs text-text-muted truncate">{currentMode.description}</span>
        </div>
        <ChevronDown class={clsx('w-4 h-4 text-text-muted flex-shrink-0 transition-transform', isOpen() && 'rotate-180')} />
      </button>

      <Show when={isOpen()}>
        <div class="absolute top-full left-0 right-0 mt-1.5 bg-bg-secondary border border-border-primary rounded-xl shadow-xl overflow-hidden z-50 animate-in">
          <For each={PERMISSION_MODES}>
            {(mode: PermissionModeOption) => (
              <button
                class={clsx(
                  'w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors',
                  'hover:bg-bg-tertiary',
                  mode.value === value && 'bg-nt-io-500/10'
                )}
                onClick={() => handleSelect(mode.value)}
                disabled={disabled}
                role="option"
                aria-selected={mode.value === value}
              >
                <mode.icon class={clsx('w-5 h-5 flex-shrink-0', mode.color)} />
                <div class="flex-1 min-w-0 flex flex-col gap-0.5">
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

// Helper component for conditional rendering
function Show(props: { when: boolean; fallback?: any; children: any }) {
  return props.when ? props.children : (props.fallback || null)
}