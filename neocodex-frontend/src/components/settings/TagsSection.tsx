/* ════════════════════════════════════════════
   components/settings/TagsSection.tsx — 标签：层级/自动打标/计数/推荐
   消费全局 tagsStore；破坏性删除经 onRequestDelete(tagName) 回调父组件确认模态。
   ════════════════════════════════════════════ */
import { For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { tagsStore, TAG_PALETTE, RECOMMENDED_TAGS, TAG_KEYWORDS } from '../../stores/tags'
import { TagRow } from './TagRow'
import { TagIcon, ExpandIcon, PaletteIcon, AutoTagIcon } from './settingsIcons'

interface Props {
  newTagInput: () => string
  setNewTagInput: (v: string) => void
  onAdd: () => void
  onSeed: () => void
  missingRecommended: () => { name: string; color: string }[]
  onRequestDelete: (name: string) => void
  showNotice: (msg: string) => void
}

export function TagsSection(props: Props) {
  // 标签树统计
  const tagTree = () => tagsStore.tagTree()
  const tagCounts = () => tagsStore.tagCounts()
  const totalSessions = () => Object.keys(tagsStore.state.sessionTags).length

  return (
    <div class="space-y-4">
      {/* 标签概览统计 */}
      <div class="ss-card rounded-2xl border-black/5 shadow-sm overflow-hidden">
        <div class="ss-card-body bg-white">
          <div class="flex items-center gap-4">
            <div class="text-center">
              <div class="text-[18px] font-semibold text-zinc-900 leading-none">{Object.keys(tagsStore.state.tags).length}</div>
              <div class="text-10px text-zinc-500 mt-1">标签总数</div>
            </div>
            <div class="w-px h-8 bg-black/5" />
            <div class="text-center">
              <div class="text-[18px] font-semibold text-emerald-600 leading-none">{totalSessions()}</div>
              <div class="text-10px text-zinc-500 mt-1">已打标会话</div>
            </div>
            <div class="w-px h-8 bg-black/5" />
            <div class="text-center">
              <div class="text-[18px] font-semibold text-zinc-900 leading-none">{tagTree().length}</div>
              <div class="text-10px text-zinc-500 mt-1">根分组</div>
            </div>
          </div>
        </div>
      </div>

      {/* 标签管理 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <TagIcon />
          标签管理
        </div>
        <div class="ss-card-body">
          <div class="text-[11px] text-text-muted leading-relaxed pb-3">
            标签用于组织会话，支持层级嵌套（如 <span class="nt-tag-hint-inline">工作/功能</span>）。
            点击色块可单独设色；重命名与删除全局生效；计数徽章显示使用该标签的会话数。
          </div>

          {/* 快速新建 */}
          <div class="flex items-center gap-2 mb-3">
            <input
              class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
              placeholder="新建标签，如 工作/功能 或 领域/前端…"
              value={props.newTagInput()}
              onInput={(e) => props.setNewTagInput(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === 'Enter') props.onAdd() }}
              aria-label="新建标签"
            />
            <button
              class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
              onClick={props.onAdd}
              disabled={!props.newTagInput().trim()}
            >
              添加
            </button>
          </div>

          <Show
            when={Object.keys(tagsStore.state.tags).length > 0}
            fallback={
              <div class="px-3 py-6 text-center text-[11px] text-text-muted border border-dashed border-border-primary/60 rounded-lg">
                暂无标签。输入上方名称新建，或从下方<b>推荐标签</b>一键套用。
              </div>
            }
          >
            <ul class="divide-y divide-border-primary/30 rounded-lg border border-border-primary/40 overflow-hidden">
              <For each={Object.entries(tagsStore.state.tags).sort((a, b) => a[0].localeCompare(b[0]))}>
                {([name, color]) => (
                  <TagRow
                    name={name}
                    color={color}
                    count={tagCounts()[name] ?? 0}
                    onColorChange={(c) => tagsStore.setTagColor(name, c)}
                    onRename={(next) => {
                      const err = tagsStore.renameTag(name, next)
                      if (err) props.showNotice(err)
                      else props.showNotice(`已重命名标签 #${next}`)
                    }}
                    onDelete={() => props.onRequestDelete(name)}
                  />
                )}
              </For>
            </ul>
          </Show>
        </div>
      </div>

      {/* 标签树 */}
      <Show when={tagTree().length > 0}>
        <div class="ss-card">
          <div class="ss-card-header">
            <ExpandIcon />
            标签树
          </div>
          <div class="ss-card-body">
            <p class="text-[11px] text-text-muted leading-relaxed pb-3">
              层级视图：根标签聚合子标签计数，点击展开查看子标签详情。
            </p>
            <div class="space-y-1.5">
              <For each={tagTree()}>
                {(root) => (
                  <div class="rounded-lg border border-border-primary/40 overflow-hidden">
                    <div class="flex items-center gap-2 px-3 py-2 bg-zinc-50/60">
                      <span class="w-3 h-3 rounded-full flex-shrink-0" style={{ background: root.color }} />
                      <span class="text-[12px] font-medium text-text-primary">#{root.name}</span>
                      <span class="text-10px text-zinc-400 font-mono ml-auto">{root.count} 会话</span>
                    </div>
                    <Show when={root.children.length > 0}>
                      <div class="px-3 py-1.5 bg-white/40 space-y-1">
                        <For each={root.children}>
                          {(child) => (
                            <div class="flex items-center gap-2 text-[11px]">
                              <span class="w-2 h-2 rounded-full flex-shrink-0" style={{ background: child.color }} />
                              <span class="text-text-secondary">#{child.name}</span>
                              <span class="text-10px text-zinc-400 font-mono ml-auto">{child.count}</span>
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>
        </div>
      </Show>

      {/* 推荐标签 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          推荐标签
          <Show when={props.missingRecommended().length === 0}>
            <span class="ml-auto text-10px font-medium text-nt-core-700 bg-nt-core-500/10 px-2 py-0.5 rounded-full">已全部添加</span>
          </Show>
        </div>
        <div class="ss-card-body">
          <p class="text-[11px] text-text-muted leading-relaxed pb-3">
            一套面向 AI 开发工作流的预置标签：<b>工作</b> 归类任务类型，<b>领域</b> 归类技术栈，<b>星域</b> 归类 NeoTrix 七域。
            仅添加缺失项，不会覆盖你已有的标签。
          </p>
          <div class="flex items-center gap-2 flex-wrap">
            <For each={RECOMMENDED_TAGS}>
              {(r) => {
                const exists = () => !!tagsStore.state.tags[r.name]
                return (
                  <button
                    class={clsx(
                      'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[11px] font-medium transition-all',
                      exists()
                        ? 'border-border-primary/40 bg-white/30 text-text-muted cursor-default'
                        : 'border-white/70 bg-white/55 text-text-primary hover:bg-white/85 hover:shadow-sm cursor-pointer'
                    )}
                    style={exists() ? undefined : { 'border-color': r.color + '55' }}
                    onClick={() => { if (!exists()) { tagsStore.setTagColor(r.name, r.color); props.showNotice(`已添加推荐标签 #${r.name}`) } }}
                    disabled={exists()}
                    aria-label={exists() ? `${r.name} 已添加` : `添加推荐标签 ${r.name}`}
                  >
                    <span class="w-2 h-2 rounded-full flex-shrink-0" style={{ background: r.color }} />
                    <span class="font-mono">#{r.name}</span>
                    {exists() && <span class="text-9px text-nt-core-700">✓</span>}
                  </button>
                )
              }}
            </For>
          </div>
          <Show when={props.missingRecommended().length > 0}>
            <button
              class="mt-3 px-3 py-1.5 rounded-lg bg-nt-io-500 text-white text-[11px] font-medium hover:bg-nt-io-600 transition-colors"
              onClick={props.onSeed}
            >
              一键添加全部推荐（{props.missingRecommended().length}）
            </button>
          </Show>
        </div>
      </div>

      {/* 自动打标配置 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <AutoTagIcon />
          自动打标
        </div>
        <div class="ss-card-body">
          <p class="text-[11px] text-text-muted leading-relaxed pb-3">
            首条用户消息将自动匹配关键词，为会话打上推荐标签。每会话至多触发一次，最多匹配 2 个标签。
          </p>
          <div class="space-y-2">
            <For each={Object.entries(TAG_KEYWORDS)}>
              {([tag, keywords]) => (
                <div class="flex items-start gap-2 px-2.5 py-1.5 rounded-lg bg-zinc-50/60 text-[11px]">
                  <span class="font-mono text-nt-io-600 whitespace-nowrap">#{tag}</span>
                  <span class="text-zinc-400">→</span>
                  <span class="text-zinc-500 flex-1">{keywords.slice(0, 6).join('、')}{keywords.length > 6 ? '…' : ''}</span>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>

      {/* 标签色板 */}
      <div class="ss-card">
        <div class="ss-card-header">
          <PaletteIcon />
          标签色板
        </div>
        <div class="ss-card-body">
          <div class="flex items-center gap-2 flex-wrap">
            <For each={TAG_PALETTE}>
              {(c) => (
                <span
                  class="w-6 h-6 rounded-full border border-white/70 shadow-sm"
                  style={{ background: c }}
                  role="img"
                  aria-label={`色板 ${c}`}
                />
              )}
            </For>
          </div>
          <p class="text-[10.5px] text-text-muted mt-2">
            新标签自动按名称哈希分配色板颜色，可在上方标签列表手动覆盖。
          </p>
        </div>
      </div>
    </div>
  )
}
