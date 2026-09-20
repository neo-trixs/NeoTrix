import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { FolderOpen, FileText, Plus, Trash2, RefreshCw, AlertCircle, Check, ChevronRight } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   FolderInstructionsPanel — 项目文件夹上下文注入面板
   
   融合产品模式：
   - Claude Code: CLAUDE.md per-project instructions
   - Cursor: .cursorrules project context
   - Windsurf: .windsurfrules project context
   
   NeoTrix 特有：
   - .neotrix/instructions.md 自动发现
   - AGENTS.md 层级继承（根 → 子目录）
   - 意识状态感知（φ 高时自动注入）
   ════════════════════════════════════════════════════════════ */

export type InstructionLevel = 'root' | 'project' | 'session'

export interface InstructionFile {
  level: InstructionLevel
  path: string
  content: string
  exists: boolean
  lastModified?: Date
}

export interface FolderInstructionsPanelProps {
  instructions: () => InstructionFile[]
  onAdd: (path: string) => void
  onRemove: (path: string) => void
  onRefresh: () => void
  onEdit: (path: string, content: string) => void
  projectRoot?: () => string | null
  loading?: () => boolean
}

const LEVEL_CONFIG: Record<InstructionLevel, { label: string; description: string; color: string; icon: any }> = {
  root: {
    label: '根指令',
    description: '全局 AGENTS.md，所有项目生效',
    color: 'text-purple-500 bg-purple-50 border-purple-200/50',
    icon: FileText,
  },
  project: {
    label: '项目指令',
    description: '.neotrix/instructions.md，当前项目生效',
    color: 'text-blue-500 bg-blue-50 border-blue-200/50',
    icon: FolderOpen,
  },
  session: {
    label: '会话指令',
    description: '会话内临时上下文，仅当前会话生效',
    color: 'text-amber-500 bg-amber-50 border-amber-200/50',
    icon: FileText,
  },
}

export function FolderInstructionsPanel(props: FolderInstructionsPanelProps) {
  const [newPath, setNewPath] = createSignal('')
  const [editingPath, setEditingPath] = createSignal<string | null>(null)
  const [editContent, setEditContent] = createSignal('')

  const startEdit = (inst: InstructionFile) => {
    setEditingPath(inst.path)
    setEditContent(inst.content)
  }

  const saveEdit = () => {
    const path = editingPath()
    if (path && props.onEdit) {
      props.onEdit(path, editContent())
      setEditingPath(null)
    }
  }

  const handleAdd = () => {
    const path = newPath().trim()
    if (path && props.onAdd) {
      props.onAdd(path)
      setNewPath('')
    }
  }

  // 按层级分组
  const grouped = () => {
    const map = new Map<InstructionLevel, InstructionFile[]>()
    for (const inst of props.instructions()) {
      const list = map.get(inst.level) ?? []
      list.push(inst)
      map.set(inst.level, list)
    }
    return map
  }

  return (
    <div class="flex flex-col h-full">
      {/* 头部 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <FileText class="w-4 h-4 text-text-muted" />
            <span class="text-[13px] font-medium text-text-primary">项目指令</span>
          </div>
          <button
            class="p-1.5 rounded-lg text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
            onClick={props.onRefresh}
            title="刷新"
          >
            <RefreshCw class={clsx('w-3.5 h-3.5', props.loading?.() && 'animate-spin')} />
          </button>
        </div>
        <div class="text-[11px] text-text-muted">
          管理 AGENTS.md / .neotrix/instructions.md 项目上下文
        </div>
      </div>

      {/* 项目根路径 */}
      <Show when={props.projectRoot?.()}>
        <div class="px-4 py-2 bg-white/30 border-b border-border-primary/40">
          <div class="flex items-center gap-2 text-[10px] text-text-muted">
            <FolderOpen class="w-3 h-3" />
            <span class="truncate">{props.projectRoot?.()}</span>
          </div>
        </div>
      </Show>

      {/* 新增指令文件 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center gap-2">
          <input
            class="flex-1 min-w-0 px-2.5 py-1.5 rounded-lg text-[11px] bg-white/40 border border-border-primary/40 outline-none focus:border-nt-io-500/40 transition-colors"
            placeholder="添加指令文件路径..."
            value={newPath()}
            onInput={(e) => setNewPath(e.currentTarget.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleAdd() }}
          />
          <button
            class="p-1.5 rounded-lg bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors"
            onClick={handleAdd}
            title="添加"
          >
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* 指令文件列表 */}
      <div class="flex-1 overflow-y-auto p-3 space-y-3">
        <Show
          when={props.instructions().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-32 text-text-muted text-[12px]">
              <FileText class="w-6 h-6 mb-2 opacity-30" />
              <span>暂无指令文件</span>
              <span class="text-[11px] mt-1 opacity-60">添加 AGENTS.md 或 .neotrix/instructions.md</span>
            </div>
          }
        >
          <For each={Array.from(grouped().entries())}>
            {([level, files]) => {
              const config = LEVEL_CONFIG[level]
              return (
                <div>
                  {/* 层级标题 */}
                  <div class="flex items-center gap-2 mb-2">
                    <span class={clsx(
                      'px-1.5 py-0.5 rounded text-[9px] border',
                      config.color
                    )}>
                      {config.label}
                    </span>
                    <span class="text-[10px] text-text-muted">{config.description}</span>
                  </div>

                  {/* 文件列表 */}
                  <div class="space-y-1.5">
                    <For each={files}>
                      {(inst) => (
                        <div class={clsx(
                          'p-2.5 rounded-lg border transition-all',
                          inst.exists
                            ? 'border-border-primary/40 bg-white/30'
                            : 'border-dashed border-red-200 bg-red-50/20'
                        )}>
                          {/* 文件路径行 */}
                          <div class="flex items-center gap-2 mb-1">
                            {inst.exists ? (
                              <Check class="w-3 h-3 text-emerald-500 flex-shrink-0" />
                            ) : (
                              <AlertCircle class="w-3 h-3 text-red-400 flex-shrink-0" />
                            )}
                            <span class="flex-1 min-w-0 text-[11px] font-mono text-text-primary truncate">
                              {inst.path}
                            </span>
                            <button
                              class="p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors"
                              onClick={() => startEdit(inst)}
                              title="编辑"
                            >
                              <FileText class="w-3 h-3" />
                            </button>
                            <button
                              class="p-1 rounded text-text-muted hover:text-red-500 hover:bg-red-50 transition-colors"
                              onClick={() => props.onRemove && props.onRemove(inst.path)}
                              title="移除"
                            >
                              <Trash2 class="w-3 h-3" />
                            </button>
                          </div>

                          {/* 文件内容预览 */}
                          <Show when={inst.exists && inst.content && editingPath() !== inst.path}>
                            <div class="mt-1.5 p-2 rounded bg-white/30 text-[10px] text-text-muted font-mono max-h-20 overflow-y-auto leading-relaxed">
                              {inst.content.slice(0, 300)}{inst.content.length > 300 ? '...' : ''}
                            </div>
                          </Show>

                          {/* 编辑模式 */}
                          <Show when={editingPath() === inst.path}>
                            <div class="mt-2">
                              <textarea
                                class="w-full h-32 px-2.5 py-2 rounded-lg text-[11px] font-mono bg-white/40 border border-nt-io-500/40 outline-none resize-y"
                                value={editContent()}
                                onInput={(e) => setEditContent(e.currentTarget.value)}
                                autofocus
                              />
                              <div class="flex gap-2 mt-2">
                                <button
                                  class="flex-1 px-2 py-1 rounded-lg text-[10px] font-medium text-white bg-nt-io-500 hover:bg-nt-io-600 transition-colors"
                                  onClick={saveEdit}
                                >
                                  保存
                                </button>
                                <button
                                  class="px-2 py-1 rounded-lg text-[10px] font-medium text-text-muted bg-white/60 hover:bg-white/80 transition-colors"
                                  onClick={() => setEditingPath(null)}
                                >
                                  取消
                                </button>
                              </div>
                            </div>
                          </Show>

                          {/* 最后修改时间 */}
                          <Show when={inst.lastModified}>
                            <div class="mt-1 text-[9px] text-text-muted/50">
                              修改于 {inst.lastModified!.toLocaleString('zh-CN')}
                            </div>
                          </Show>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              )
            }}
          </For>
        </Show>
      </div>

      {/* 底部说明 */}
      <div class="px-4 py-2 border-t border-border-primary/40 bg-white/20">
        <div class="text-[10px] text-text-muted/60 leading-relaxed">
          指令文件按层级继承：根 → 项目 → 会话。内容会自动注入到 AI 上下文中。
        </div>
      </div>
    </div>
  )
}
