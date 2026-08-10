import { Show, For } from 'solid-js'
import { clsx } from 'clsx'
import { tagDepth } from '../stores/tags'

/* ════════════════════════════════════════════
   NeoTag — 标签胶囊（对标 Obsidian tag pill）
   视觉语言：色点 + #name + 可选计数徽章
   - 层级：parent/child 显示斜杠分隔（深层缩进由容器处理）
   - 状态：默认 / hover（色点 → 填充淡化）/ active（选中过滤）/ removable（× 移除）
   ════════════════════════════════════════════ */

export interface NeoTagProps {
  name: string
  color: string
  count?: number
  active?: boolean
  /** 显示移除按钮（打标输入内） */
  removable?: boolean
  onRemove?: (name: string) => void
  onClick?: (name: string) => void
  /** 组件尺寸：sm 行内 / md 侧栏 */
  size?: 'sm' | 'md'
  /** 嵌套标签是否显示分层缩进指示（父/子） */
  showHierarchy?: boolean
}

export function NeoTag(props: NeoTagProps) {
  const parts = () => props.name.split('/')
  const isNested = () => parts().length > 1
  const last = () => parts()[parts().length - 1]
  const depth = () => tagDepth(props.name)

  const color = () => props.color || '#909098'

  return (
    <span
      class={clsx(
        'nt-tag inline-flex items-center gap-1 select-none',
        props.size === 'md' ? 'nt-tag-md' : 'nt-tag-sm',
        props.active && 'nt-tag-active'
      )}
      style={{ '--tag-color': color(), '--tag-indent': `${(depth() - 1) * 8}px` }}
      role={props.onClick ? 'button' : undefined}
      tabIndex={props.onClick ? 0 : undefined}
      onClick={(e) => { e.stopPropagation(); props.onClick?.(props.name) }}
      onKeyDown={(e) => { if (props.onClick && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); e.stopPropagation(); props.onClick(props.name) } }}
      title={`#${props.name}`}
    >
      <span class="nt-tag-dot" aria-hidden="true" />
      <Show when={props.showHierarchy && isNested()}>
        <span class="nt-tag-hint">
          <For each={parts().slice(0, -1)}>
            {(p) => (
              <>
                <span class="nt-tag-part">{p}</span>
                <span class="nt-tag-slash">/</span>
              </>
            )}
          </For>
        </span>
      </Show>
      <span class="nt-tag-label">{last()}</span>
      <Show when={props.count !== undefined}>
        <span class={clsx('nt-tag-count', props.active && 'nt-tag-count-active')}>{props.count}</span>
      </Show>
      <Show when={props.removable}>
        <button
          class="nt-tag-x"
          aria-label={`移除标签 #${props.name}`}
          title={`移除 #${props.name}`}
          onClick={(e) => { e.stopPropagation(); props.onRemove?.(props.name) }}
        >
          <svg viewBox="0 0 10 10" fill="none">
            <line x1="2.5" y1="2.5" x2="7.5" y2="7.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            <line x1="7.5" y1="2.5" x2="2.5" y2="7.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
        </button>
      </Show>
    </span>
  )
}

/** 会话标题下的标签行（渲染该会话全部标签，可点击筛选/移除） */
export function NeoTagRow(props: {
  tags: { name: string; color: string }[]
  activeTag?: string | null
  onTagClick?: (name: string) => void
  onTagRemove?: (name: string) => void
  className?: string
}) {
  return (
    <div class={clsx('flex flex-wrap gap-1', props.className)}>
      <For each={props.tags}>
        {(t) => (
          <NeoTag
            name={t.name}
            color={t.color}
            size="sm"
            active={props.activeTag === t.name}
            removable={!!props.onTagRemove}
            onClick={props.onTagClick}
            onRemove={props.onTagRemove}
            showHierarchy
          />
        )}
      </For>
    </div>
  )
}

