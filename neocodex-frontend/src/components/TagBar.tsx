import { createSignal, Show, For } from 'solid-js'
import { clsx } from 'clsx'
import { tagsStore } from '../stores/tags'
import { NeoTag } from './NeoTag'

/* ════════════════════════════════════════════
   TagBar — 侧栏标签区（对标 Obsidian Tag Pane）
   - 层级树：根标签可折叠，子标签缩进显示
   - 多选过滤：点击根标签筛选全部含该根的会话；再点取消
   - 空态：无标签时显示引导
   ════════════════════════════════════════════ */

export interface TagBarProps {
  /** 当前激活的筛选标签（多选） */
  activeTags: string[]
  onToggleTag: (name: string) => void
  onClearTags: () => void
}

function HashIcon(props: { class?: string }) {
  return (
    <svg viewBox="0 0 16 16" fill="none" class={props.class}>
      <line x1="5.5" y1="2.5" x2="4" y2="13.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="10.5" y1="2.5" x2="9" y2="13.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="2.5" y1="6" x2="13.5" y2="6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="2.5" y1="10" x2="13.5" y2="10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

export function TagBar(props: TagBarProps) {
  const [collapsedRoots, setCollapsedRoots] = createSignal<Set<string>>(new Set())

  const tree = () => tagsStore.tagTree()
  const activeCount = () => props.activeTags.length

  const toggleRoot = (name: string) => {
    setCollapsedRoots((prev) => {
      const next = new Set(prev)
      if (next.has(name)) next.delete(name)
      else next.add(name)
      return next
    })
  }

  const rootActive = (name: string) => props.activeTags.includes(name)
  const childActive = (name: string) => props.activeTags.includes(name)

  return (
    <div class="px-3 pb-2">
      <Show
        when={tree().length > 0}
        fallback={
          <div class="px-3 py-2 rounded-lg border border-dashed border-border-primary/60 text-[11px] text-text-muted/70 leading-relaxed">
            暂无标签。在会话上悬停点 <span class="nt-tag-hint-inline">#</span> 即可打标。
          </div>
        }
      >
        {/* 标签区头部：标题 + 清除筛选 */}
        <div class="flex items-center justify-between px-1 pb-1 pt-2">
          <div class="flex items-center gap-1.5 text-[10px] uppercase tracking-widest text-text-muted/60 font-medium">
            <HashIcon class="w-3 h-3" />
            标签
          </div>
          <Show when={activeCount() > 0}>
            <button
              class="text-[10px] text-nt-io-600 hover:text-nt-io-700 font-medium px-1.5 py-0.5 rounded hover:bg-nt-io-500/10 transition-colors"
              onClick={props.onClearTags}
              aria-label="清除标签筛选"
            >
              清除 ({activeCount()})
            </button>
          </Show>
        </div>

        {/* 标签树 */}
        <ul class="space-y-0.5" role="list" aria-label="会话标签列表">
          <For each={tree()}>
            {(root) => {
              const isCollapsed = () => collapsedRoots().has(root.name)
              const isActive = () => rootActive(root.name)
              return (
                <li>
                  {/* 根标签行 */}
                  <div class="flex items-center gap-1">
                    <button
                      class={clsx(
                        'p-0.5 rounded text-text-muted/50 hover:text-text-primary transition-colors flex-shrink-0',
                        isCollapsed() && root.children.length > 0 && 'rotate-[-90deg]'
                      )}
                      onClick={() => toggleRoot(root.name)}
                      aria-label={isCollapsed() ? `展开 ${root.name}` : `折叠 ${root.name}`}
                      aria-expanded={!isCollapsed()}
                      style={{ visibility: root.children.length > 0 ? 'visible' : 'hidden' }}
                    >
                      <svg viewBox="0 0 10 10" fill="none" class="w-2.5 h-2.5 transition-transform">
                        <path d="M2.5 2l3.5 3-3.5 3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
                      </svg>
                    </button>
                    <NeoTag
                      name={root.name}
                      color={root.color}
                      count={root.count}
                      size="md"
                      active={isActive()}
                      onClick={() => props.onToggleTag(root.name)}
                    />
                  </div>

                  {/* 子标签（缩进） */}
                  <Show when={!isCollapsed() && root.children.length > 0}>
                    <ul class="ml-4 mt-0.5 space-y-0.5 border-l border-border-primary/50 pl-1.5">
                      <For each={root.children}>
                        {(child) => (
                          <li>
                            <NeoTag
                              name={child.name}
                              color={child.color}
                              count={child.count}
                              size="md"
                              active={childActive(child.name)}
                              onClick={() => props.onToggleTag(child.name)}
                              showHierarchy
                            />
                          </li>
                        )}
                      </For>
                    </ul>
                  </Show>
                </li>
              )
            }}
          </For>
        </ul>
      </Show>
    </div>
  )
}
