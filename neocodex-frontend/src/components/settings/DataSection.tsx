/* ════════════════════════════════════════════
   components/settings/DataSection.tsx — 数据：记忆统计 + 导出/清空
   破坏性操作（清空）触发父组件确认模态（onRequestClear 回调）。
   ════════════════════════════════════════════ */
import { Show } from 'solid-js'
import type { MemoryStats } from '../../api/types'
import { DataIcon, ExpandIcon } from './settingsIcons'

interface Props {
  memStats: () => MemoryStats | null
  dataBusy: () => boolean
  onExport: () => void
  onRequestClear: () => void
}

export function DataSection(props: Props) {
  return (
    <div class="space-y-4">
      <div class="ss-card">
        <div class="ss-card-header">
          <DataIcon />
          记忆统计
        </div>
        <div class="ss-card-body">
          <Show when={props.memStats()} fallback={<div class="text-xs text-text-muted py-2">加载记忆统计…</div>}>
            {(ms) => (
              <div class="grid grid-cols-2 gap-2">
                <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                  <div class="text-[10px] text-text-muted mb-1">记忆条目</div>
                  <div class="text-[13px] text-text-primary font-medium">{ms().total_entries}</div>
                </div>
                <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                  <div class="text-[10px] text-text-muted mb-1">分类</div>
                  <div class="text-[13px] text-text-primary font-medium">{ms().total_categories}</div>
                </div>
                <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                  <div class="text-[10px] text-text-muted mb-1">平均置信度</div>
                  <div class="text-[13px] text-text-primary font-medium">{(ms().avg_confidence * 100).toFixed(0)}%</div>
                </div>
                <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                  <div class="text-[10px] text-text-muted mb-1">占用空间</div>
                  <div class="text-[13px] text-text-primary font-medium">{(ms().memory_usage_bytes / 1024).toFixed(1)} KB</div>
                </div>
              </div>
            )}
          </Show>
        </div>
      </div>

      <div class="ss-card">
        <div class="ss-card-header">
          <ExpandIcon />
          数据操作
        </div>
        <div class="ss-card-body space-y-2">
          <button
            class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors"
            onClick={props.onExport}
            disabled={props.dataBusy()}
          >
            <span class="text-[12.5px] text-text-primary">导出记忆（JSON）</span>
            <span class="text-[10px] text-text-muted">→ 文件</span>
          </button>
          <button
            class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-red-500/30 bg-red-500/5 hover:bg-red-500/10 transition-colors"
            onClick={props.onRequestClear}
            disabled={props.dataBusy()}
          >
            <span class="text-[12.5px] text-red-500">清空全部记忆</span>
            <span class="text-[10px] text-red-400">不可恢复</span>
          </button>
        </div>
      </div>
    </div>
  )
}