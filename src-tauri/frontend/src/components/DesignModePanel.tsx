import { createSignal, For, Show, createEffect } from 'solid-js'
import { clsx } from 'clsx'
import { MousePointer2, Square, Type, Mic, Undo2, Redo2, Eye, EyeOff, Layers, Zap } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   DesignModePanel — Cursor 3 风格浏览器标注模式
   
   融合产品模式：
   - Cursor 3: Design Mode (⌘+Shift+D 浏览器标注)
   - Cursor 3.7: Multi-select + Voice in Design Mode
   - Copilot Vision: 屏幕理解+高亮
   
   NeoTrix 特有：
   - E8 意识状态感知标注（φ 高时自动标注高优先级元素）
   - GWT 注意力路由（标注权重经注意力路由）
   ════════════════════════════════════════════════════════════ */

export type DesignTool = 'select' | 'rectangle' | 'text' | 'voice'

export interface AnnotationElement {
  id: string
  tag: string
  text?: string
  selector: string
  boundingBox?: { x: number; y: number; width: number; height: number }
  selected: boolean
}

export interface DesignModePanelProps {
  isActive: () => boolean
  onToggle: () => void
  selectedElements: () => AnnotationElement[]
  onSelectElement: (el: AnnotationElement) => void
  onRemoveElement: (id: string) => void
  onClearSelection: () => void
  onSubmitPrompt: (prompt: string) => void
  phi?: () => number | null
}

const TOOLS: { id: DesignTool; icon: any; label: string; shortcut: string }[] = [
  { id: 'select',  icon: MousePointer2, label: '选择元素', shortcut: '⌘+L' },
  { id: 'rectangle', icon: Square,      label: '框选区域', shortcut: 'Shift+拖拽' },
  { id: 'text',    icon: Type,          label: '文字标注', shortcut: '' },
  { id: 'voice',   icon: Mic,           label: '语音输入', shortcut: '' },
]

export function DesignModePanel(props: DesignModePanelProps) {
  const [activeTool, setActiveTool] = createSignal<DesignTool>('select')
  const [prompt, setPrompt] = createSignal('')
  const [showOverlay, setShowOverlay] = createSignal(true)

  // ⌘+Shift+D 快捷键切换 Design Mode
  createEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'D') {
        e.preventDefault()
        props.onToggle()
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  })

  const handleSubmit = () => {
    const p = prompt().trim()
    if (!p) return
    props.onSubmitPrompt(p)
    setPrompt('')
  }

  return (
    <div class="flex flex-col h-full">
      {/* Header */}
      <div class="flex items-center justify-between px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <MousePointer2 class="w-4 h-4 text-violet-500" />
          <span class="text-[13px] font-medium text-text-primary">Design Mode</span>
          <Show when={props.isActive()}>
            <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-violet-100 text-violet-600 font-medium animate-pulse">
              活跃
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-1.5">
          <button
            class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
            onClick={() => setShowOverlay(!showOverlay())}
            title={showOverlay() ? '隐藏叠加层' : '显示叠加层'}
          >
            {showOverlay() ? <Eye class="w-3.5 h-3.5" /> : <EyeOff class="w-3.5 h-3.5" />}
          </button>
          <button
            class={clsx(
              'px-2.5 py-1 rounded-lg text-[11px] font-medium transition-colors',
              props.isActive()
                ? 'bg-violet-500/10 text-violet-600 hover:bg-violet-500/20'
                : 'bg-white/60 text-text-muted hover:bg-white/80'
            )}
            onClick={props.onToggle}
          >
            {props.isActive() ? '退出 Design Mode' : '⌘⇧D 启动'}
          </button>
        </div>
      </div>

      <Show when={props.isActive()}>
        {/* Tool bar */}
        <div class="flex items-center gap-1 px-4 py-2 border-b border-border-primary/20">
          <For each={TOOLS}>
            {(tool) => {
              const ToolIcon = tool.icon
              const active = () => activeTool() === tool.id
              return (
                <button
                  class={clsx(
                    'flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] transition-all',
                    active()
                      ? 'bg-violet-500/10 text-violet-600 shadow-sm'
                      : 'text-text-muted hover:text-text-primary hover:bg-white/60'
                  )}
                  onClick={() => setActiveTool(tool.id)}
                  title={`${tool.label} ${tool.shortcut ? `(${tool.shortcut})` : ''}`}
                >
                  <ToolIcon class="w-3.5 h-3.5" />
                  <span>{tool.label}</span>
                </button>
              )
            }}
          </For>
        </div>

        {/* Selected elements */}
        <div class="px-4 py-2 border-b border-border-primary/20">
          <div class="flex items-center justify-between mb-1.5">
            <span class="text-[10px] text-text-muted">
              已选择元素 ({props.selectedElements().length})
            </span>
            <Show when={props.selectedElements().length > 0}>
              <button
                class="text-[10px] text-red-500 hover:text-red-600 transition-colors"
                onClick={props.onClearSelection}
              >
                清除全部
              </button>
            </Show>
          </div>
          <Show
            when={props.selectedElements().length > 0}
            fallback={
              <div class="text-[10px] text-text-muted/50 py-2 text-center">
                在浏览器中点击元素或 Shift+拖拽框选
              </div>
            }
          >
            <div class="flex flex-wrap gap-1.5">
              <For each={props.selectedElements()}>
                {(el) => (
                  <div class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-violet-50 border border-violet-200/50 text-[10px]">
                    <Layers class="w-3 h-3 text-violet-500" />
                    <span class="font-mono text-violet-700">{el.tag}</span>
                    {el.text && <span class="text-violet-500 max-w-[80px] truncate">"{el.text}"</span>}
                    <button
                      class="p-0.5 rounded text-violet-400 hover:text-red-500 transition-colors"
                      onClick={() => props.onRemoveElement(el.id)}
                    >
                      ×
                    </button>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>

        {/* Prompt input */}
        <div class="px-4 py-3">
          <div class="relative">
            <textarea
              class="w-full px-3 py-2 pr-10 rounded-xl bg-white/50 border border-black/5 text-[12px] text-text-primary placeholder-text-muted/50 outline-none focus:border-violet-500/40 resize-none transition-colors"
              placeholder="描述你想改变的内容…（语音或文字）"
              rows={3}
              value={prompt()}
              onInput={(e) => setPrompt(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
                  e.preventDefault()
                  handleSubmit()
                }
              }}
            />
            <div class="absolute right-2 bottom-2 flex gap-1">
              <button
                class={clsx(
                  'p-1.5 rounded-lg transition-colors',
                  activeTool() === 'voice'
                    ? 'bg-red-100 text-red-500 animate-pulse'
                    : 'text-text-muted hover:text-text-primary hover:bg-white/60'
                )}
                onClick={() => setActiveTool(activeTool() === 'voice' ? 'select' : 'voice')}
                title="语音输入"
              >
                <Mic class="w-3.5 h-3.5" />
              </button>
              <button
                class={clsx(
                  'p-1.5 rounded-lg transition-colors',
                  prompt().trim()
                    ? 'bg-violet-500 text-white hover:bg-violet-600'
                    : 'text-text-muted/30 cursor-not-allowed'
                )}
                onClick={handleSubmit}
                disabled={!prompt().trim()}
                title="提交 (⌘+Enter)"
              >
                <Zap class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
          <div class="flex items-center justify-between mt-2">
            <div class="flex items-center gap-2 text-[9px] text-text-muted/50">
              <span>⌘+Enter 提交</span>
              <span>⌘+L 添加元素</span>
              <span>Shift+拖拽 框选</span>
            </div>
            <Show when={props.phi?.() !== null && props.phi?.() !== undefined}>
              <span class="text-[9px] text-violet-500/60">
                φ {((props.phi?.() ?? 0) * 100).toFixed(0)}%
              </span>
            </Show>
          </div>
        </div>

        {/* Browser overlay indicator */}
        <Show when={showOverlay()}>
          <div class="px-4 py-2 border-t border-border-primary/20">
            <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-violet-50/50 border border-violet-200/30">
              <div class="w-2 h-2 rounded-full bg-violet-400 animate-pulse" />
              <span class="text-[10px] text-violet-600">
                浏览器叠加层已激活 — 点击元素选择，Shift+拖拽框选区域
              </span>
            </div>
          </div>
        </Show>
      </Show>

      {/* Inactive state */}
      <Show when={!props.isActive()}>
        <div class="flex-1 flex flex-col items-center justify-center px-4 py-8">
          <div class="w-12 h-12 rounded-2xl bg-violet-100 flex items-center justify-center mb-3">
            <MousePointer2 class="w-6 h-6 text-violet-500" />
          </div>
          <span class="text-[12px] font-medium text-text-primary mb-1">Design Mode</span>
          <span class="text-[11px] text-text-muted text-center mb-4">
            在浏览器中点击、框选或语音标注 UI 元素，agent 自动修改代码
          </span>
          <div class="text-[10px] text-text-muted/50 space-y-1 text-center">
            <div>⌘+Shift+D 启动 Design Mode</div>
            <div>⌘+L 将元素添加到对话</div>
            <div>Shift+拖拽 框选区域</div>
          </div>
        </div>
      </Show>
    </div>
  )
}
