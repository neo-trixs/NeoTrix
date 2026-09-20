import { createSignal, For, Show, createEffect } from 'solid-js'
import { clsx } from 'clsx'
import { Shield, FileCheck, AlertTriangle, CheckCircle, XCircle, RefreshCw, Eye, EyeOff, GitBranch, Layers } from 'lucide-solid'

/* ════════════════════════════════════════════════════════════
   SparseCheckoutWizard — Context Integrity Guardian (CIG)
   
   融合产品模式：
   - Devin: Context Integrity Guardian (CIG) — 自动裁剪无关文件
   - Cursor: 代码库索引 + 选择性上下文
   - Claude Code: CLAUDE.md 按项目上下文注入
   
   NeoTrix 特有：
   - .neotrix/sparse-config.json 持久化
   - 意识状态感知（φ 低时自动扩大上下文窗口）
   - 五级 Context Level 自动裁剪策略
   ════════════════════════════════════════════════════════════ */

export type ContextLevel = 'minimal' | 'focused' | 'standard' | 'expanded' | 'full'

export interface SparseRule {
  pattern: string       // glob pattern: "src/**/*.rs", "!target/**"
  include: boolean
  priority: number      // higher = more important
  description?: string
}

export interface ContextIntegrityReport {
  totalFiles: number
  includedFiles: number
  excludedFiles: number
  estimatedTokens: number
  maxTokens: number
  integrityScore: number  // 0-1
  violations: ContextViolation[]
  lastChecked: Date
}

export interface ContextViolation {
  type: 'unmatched_inclusion' | 'missing_exclusion' | 'token_overflow' | 'stale_reference'
  file?: string
  message: string
  severity: 'warning' | 'error'
}

export interface SparseCheckoutWizardProps {
  level: () => ContextLevel
  onLevelChange: (level: ContextLevel) => void
  rules: () => SparseRule[]
  onAddRule: (rule: SparseRule) => void
  onRemoveRule: (index: number) => void
  report: () => ContextIntegrityReport | null
  onRefresh: () => void
  phi?: () => number | null
  loading?: () => boolean
}

const LEVEL_CONFIG: Record<ContextLevel, { label: string; description: string; color: string; icon: any; tokenBudget: string }> = {
  minimal: {
    label: '极简',
    description: '仅当前文件 + 直接依赖',
    color: 'text-emerald-500 bg-emerald-50 border-emerald-200/50',
    icon: Layers,
    tokenBudget: '~2K tokens',
  },
  focused: {
    label: '聚焦',
    description: '当前任务相关文件',
    color: 'text-blue-500 bg-blue-50 border-blue-200/50',
    icon: Eye,
    tokenBudget: '~8K tokens',
  },
  standard: {
    label: '标准',
    description: '项目主要源码',
    color: 'text-amber-500 bg-amber-50 border-amber-200/50',
    icon: FileCheck,
    tokenBudget: '~32K tokens',
  },
  expanded: {
    label: '扩展',
    description: '包含测试和文档',
    color: 'text-orange-500 bg-orange-50 border-orange-200/50',
    icon: GitBranch,
    tokenBudget: '~100K tokens',
  },
  full: {
    label: '全量',
    description: '不裁剪，全量上下文',
    color: 'text-red-500 bg-red-50 border-red-200/50',
    icon: EyeOff,
    tokenBudget: '无限制',
  },
}

export function SparseCheckoutWizard(props: SparseCheckoutWizardProps) {
  const [newPattern, setNewPattern] = createSignal('')
  const [newInclude, setNewInclude] = createSignal(true)
  const [newDescription, setNewDescription] = createSignal('')
  const report = () => props.report()

  const handleAddRule = () => {
    const pattern = newPattern().trim()
    if (!pattern) return
    props.onAddRule({
      pattern,
      include: newInclude(),
      priority: 50,
      description: newDescription() || undefined,
    })
    setNewPattern('')
    setNewDescription('')
  }

  // 自动根据 φ 值建议 Context Level
  const suggestedLevel = (): ContextLevel | null => {
    const phi = props.phi?.()
    if (phi === null || phi === undefined) return null
    if (phi < 0.3) return 'expanded'  // 意识低 → 扩大上下文
    if (phi < 0.6) return 'standard'
    if (phi < 0.8) return 'focused'
    return 'minimal'  // 意识高 → 极简上下文
  }

  return (
    <div class="flex flex-col h-full">
      {/* 头部 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2">
            <Shield class="w-4 h-4 text-text-muted" />
            <span class="text-[13px] font-medium text-text-primary">上下文完整性守护</span>
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
          Sparse Checkout 策略配置 · 自动裁剪无关文件
        </div>
      </div>

      {/* Context Level 选择器 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="flex items-center justify-between mb-2">
          <span class="text-[11px] font-medium text-text-primary">上下文级别</span>
          <Show when={suggestedLevel() && suggestedLevel() !== props.level()}>
            <button
              class="text-[10px] text-nt-io-600 hover:text-nt-io-700 transition-colors"
              onClick={() => props.onLevelChange(suggestedLevel()!)}
            >
              建议: {LEVEL_CONFIG[suggestedLevel()!].label}
            </button>
          </Show>
        </div>
        <div class="grid grid-cols-5 gap-1">
          <For each={(['minimal', 'focused', 'standard', 'expanded', 'full'] as ContextLevel[])}>
            {(level) => {
              const config = LEVEL_CONFIG[level]
              const active = () => props.level() === level
              return (
                <button
                  class={clsx(
                    'flex flex-col items-center gap-1 p-1.5 rounded-lg text-[9px] transition-all',
                    active()
                      ? `${config.color} border`
                      : 'border border-transparent text-text-muted hover:bg-white/60'
                  )}
                  onClick={() => props.onLevelChange(level)}
                >
                  <config.icon class="w-3.5 h-3.5" />
                  <span>{config.label}</span>
                </button>
              )
            }}
          </For>
        </div>
        <div class="mt-2 text-[10px] text-text-muted">
          {LEVEL_CONFIG[props.level()].description} · {LEVEL_CONFIG[props.level()].tokenBudget}
        </div>
      </div>

      {/* 完整性报告 */}
      <Show when={report()}>
        <div class="px-4 py-3 border-b border-border-primary/40">
          <div class="flex items-center gap-2 mb-2">
            <span class="text-[11px] font-medium text-text-primary">完整性报告</span>
            <span class={clsx(
              'px-1.5 py-0.5 rounded text-[9px]',
              report()!.integrityScore >= 0.8
                ? 'text-emerald-500 bg-emerald-50'
                : report()!.integrityScore >= 0.5
                ? 'text-amber-500 bg-amber-50'
                : 'text-red-500 bg-red-50'
            )}>
              {Math.round(report()!.integrityScore * 100)}%
            </span>
          </div>

          {/* 统计数据 */}
          <div class="grid grid-cols-4 gap-2 mb-2">
            <div class="text-center">
              <div class="text-[14px] font-medium text-text-primary">{report()!.totalFiles}</div>
              <div class="text-[9px] text-text-muted">总文件</div>
            </div>
            <div class="text-center">
              <div class="text-[14px] font-medium text-emerald-500">{report()!.includedFiles}</div>
              <div class="text-[9px] text-text-muted">包含</div>
            </div>
            <div class="text-center">
              <div class="text-[14px] font-medium text-red-400">{report()!.excludedFiles}</div>
              <div class="text-[9px] text-text-muted">排除</div>
            </div>
            <div class="text-center">
              <div class="text-[14px] font-medium text-text-primary">~{Math.round(report()!.estimatedTokens / 1000)}K</div>
              <div class="text-[9px] text-text-muted">估算 tokens</div>
            </div>
          </div>

          {/* 违规列表 */}
          <Show when={report()!.violations.length > 0}>
            <div class="space-y-1 mt-2">
              <For each={report()!.violations.slice(0, 3)}>
                {(v) => (
                  <div class={clsx(
                    'flex items-start gap-1.5 p-1.5 rounded text-[10px]',
                    v.severity === 'error' ? 'bg-red-50 text-red-600' : 'bg-amber-50 text-amber-600'
                  )}>
                    {v.severity === 'error' ? <XCircle class="w-3 h-3 flex-shrink-0 mt-0.5" /> : <AlertTriangle class="w-3 h-3 flex-shrink-0 mt-0.5" />}
                    <span class="flex-1">{v.message}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>

      {/* 裁剪规则 */}
      <div class="px-4 py-3 border-b border-border-primary/40">
        <div class="text-[11px] font-medium text-text-primary mb-2">裁剪规则</div>
        <div class="flex items-center gap-2 mb-2">
          <input
            class="flex-1 min-w-0 px-2.5 py-1.5 rounded-lg text-[11px] font-mono bg-white/40 border border-border-primary/40 outline-none focus:border-nt-io-500/40"
            placeholder="src/**/*.rs"
            value={newPattern()}
            onInput={(e) => setNewPattern(e.currentTarget.value)}
            onKeyDown={(e) => { if (e.key === 'Enter') handleAddRule() }}
          />
          <button
            class={clsx(
              'px-2 py-1.5 rounded-lg text-[10px] font-medium border transition-colors',
              newInclude()
                ? 'text-emerald-600 bg-emerald-50 border-emerald-200/50'
                : 'text-red-500 bg-red-50 border-red-200/50'
            )}
            onClick={() => setNewInclude(!newInclude())}
            title={newInclude() ? '包含' : '排除'}
          >
            {newInclude() ? '包含' : '排除'}
          </button>
          <button
            class="p-1.5 rounded-lg bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors"
            onClick={handleAddRule}
            title="添加规则"
          >
            <CheckCircle class="w-3.5 h-3.5" />
          </button>
        </div>

        {/* 规则列表 */}
        <div class="space-y-1 max-h-32 overflow-y-auto">
          <For each={props.rules()}>
            {(rule, i) => (
              <div class="flex items-center gap-2 px-2 py-1 rounded bg-white/30 text-[10px]">
                <span class={clsx(
                  'px-1 py-0.5 rounded text-[8px] font-medium',
                  rule.include ? 'text-emerald-600 bg-emerald-50' : 'text-red-500 bg-red-50'
                )}>
                  {rule.include ? 'IN' : 'OUT'}
                </span>
                <span class="flex-1 font-mono text-text-primary truncate">{rule.pattern}</span>
                <Show when={rule.description}>
                  <span class="text-text-muted/50 truncate">{rule.description}</span>
                </Show>
                <button
                  class="p-0.5 rounded text-text-muted/30 hover:text-red-500 transition-colors"
                  onClick={() => props.onRemoveRule(i())}
                >
                  <XCircle class="w-3 h-3" />
                </button>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* 底部说明 */}
      <div class="px-4 py-2 border-t border-border-primary/40 bg-white/20">
        <div class="text-[10px] text-text-muted/60 leading-relaxed">
          Sparse Checkout 自动裁剪无关文件，保持上下文精简。规则按优先级排序，高优先级规则覆盖低优先级。
        </div>
      </div>
    </div>
  )
}
