import { createSignal, onMount, createEffect, onCleanup, Show, For } from 'solid-js'
import { GitBranch, FileCode2, Check, X, Loader2, RefreshCw, ChevronDown, ChevronRight, GitCommitHorizontal, Upload, AlertTriangle } from 'lucide-solid'
import { neocodex, errText } from '../api'
import type { GitStatus } from '../api/types'
import { clsx } from 'clsx'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

interface DiffLine {
  t: 'add' | 'del' | 'ctx'
  o: number | null
  n: number | null
  s: string
}

interface Hunk {
  lines: DiffLine[]
}

interface DiffFile {
  path: string
  hunks: Hunk[]
}

interface DiffResponse {
  files: DiffFile[]
}

interface Props {
  open: boolean
  onClose: () => void
}

export function GitPanel(props: Props) {
  const [status, setStatus] = createSignal<GitStatus | null>(null)
  const [diff, setDiff] = createSignal<DiffResponse | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [expanded, setExpanded] = createSignal<Set<string>>(new Set())
  // 已暂存文件来自后端真实状态（git diff --cached --name-only）：
  // 面板打开时初始化，accept/reject 后本地即时更新 + 后台刷新对齐，非会话级记忆
  const [staged, setStaged] = createSignal<Set<string>>(new Set())
  const [branches, setBranches] = createSignal<string[]>([])
  const [selectedBranch, setSelectedBranch] = createSignal('')
  const [commitMsg, setCommitMsg] = createSignal('')
  const [toast, setToast] = createSignal<string | null>(null)
  let toastTimer: number | undefined
  // 面板卸载时清理 toast 定时器，避免泄漏
  onCleanup(() => window.clearTimeout(toastTimer))
  const [busy, setBusy] = createSignal<string | null>(null)
  const [error, setError] = createSignal<string | null>(null)
  // 统一确认模态（替换原生 confirm）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingRejectPath, setPendingRejectPath] = createSignal<string | null>(null)
  // 批次3：提交审阅关卡状态——待确认的 commit message + 已确认跳过关卡标志（本次会话内一次）
  const [pendingCommitMsg, setPendingCommitMsg] = createSignal<string | null>(null)
  const [commitGateBypassed, setCommitGateBypassed] = createSignal(false)
  let firstBtnRef: HTMLButtonElement | undefined
  let panelRef: HTMLDivElement | undefined
  // 打开面板前的触发元素，关闭后还原焦点
  let lastFocusedEl: HTMLElement | null = null

  // 面板打开时聚焦首个按钮（对标 Codex 面板聚焦规范），并记录触发元素；
  // 关闭时（Esc/关闭按钮/遮罩点击触发卸载）经 effect 清理还原焦点
  createEffect(() => {
    if (!props.open) return
    lastFocusedEl = document.activeElement as HTMLElement | null
    const raf = requestAnimationFrame(() => {
      if (firstBtnRef) firstBtnRef.focus()
      else panelRef?.focus()
    })
    return () => {
      cancelAnimationFrame(raf)
      if (lastFocusedEl?.isConnected) lastFocusedEl.focus()
    }
  })

  // 仅首次加载自动展开第一个文件；刷新/操作后重载不重置用户当前的展开状态
  let firstLoad = true

  // 🟡 修复：silent 静默刷新 —— accept/reject 后刷新若再置 loading，整个 body 被
  //「加载变更...」spinner 替换重渲染，diff 展开/滚动位置全部丢失。操作后刷新不闪
  // loading（后台对齐），仅首次加载/手动刷新按钮显示 spinner。
  const load = async (silent = false) => {
    if (!silent) setLoading(true)
    setError(null)
    try {
      const [st, df, stagedFiles, branches] = await Promise.all([
        neocodex.gitStatus(),
        neocodex.getDiff(),
        neocodex.gitStagedFiles(),
        neocodex.listBranches(),
      ])
      setStatus(st)
      setDiff(df)
      setStaged(new Set(stagedFiles))
      setBranches(branches)
      // 默认对齐当前分支；用户手动选择后不再覆盖
      if (!selectedBranch()) {
        setSelectedBranch(st?.branch ?? branches[0] ?? '')
      }
      if (firstLoad) {
        firstLoad = false
        if (df.files.length > 0) setExpanded(new Set([df.files[0].path]))
      }
    } catch (e) {
      setError(errText(e))
    } finally {
      if (!silent) setLoading(false)
    }
  }

  onMount(load)

  // 后台对齐后端真实暂存状态（accept/reject 后调用，失败时保留本地标记不打扰）
  const refreshStaged = async () => {
    try {
      setStaged(new Set(await neocodex.gitStagedFiles()))
    } catch {
      // 忽略：本地即时标记已足够，下次 load 会重新对齐
    }
  }

  const showToast = (msg: string) => {
    setToast(msg)
    window.clearTimeout(toastTimer)
    toastTimer = window.setTimeout(() => setToast(null), 4000)
  }

  const toggle = (path: string) => {
    setExpanded(prev => {
      const next = new Set(prev)
      if (next.has(path)) next.delete(path)
      else next.add(path)
      return next
    })
  }

  const doApply = async (path: string, action: 'accept' | 'reject') => {
    setBusy(`${path}:${action}`)
    setError(null)
    try {
      await neocodex.applyDiff(path, action)
      // accept = 已暂存；reject = 变更已丢弃。本地即时更新标记，后台刷新对齐后端
      setStaged(prev => {
        const next = new Set(prev)
        if (action === 'accept') next.add(path)
        else next.delete(path)
        return next
      })
      void refreshStaged()
      // 静默刷新：避免全面板 spinner 闪烁 + diff 展开/滚动丢失
      await load(true)
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const apply = async (path: string, action: 'accept' | 'reject') => {
    // 拒绝 = git restore 丢弃变更，破坏性操作需确认（对标 Codex）
    if (action === 'reject') {
      setPendingRejectPath(path)
      setModalReq({
        title: '丢弃变更',
        message: `确定丢弃 ${path} 的变更？此操作不可撤销。`,
        danger: true,
        confirmLabel: '丢弃',
      })
      return
    }
    await doApply(path, action)
  }

  // 提交前审阅关卡：未暂存（未审阅）文件强制提示（对标审阅前置流）。
  // git commit 仅包含已暂存内容，故关卡不阻塞功能，只在"存在未审阅变更"时显式要求用户知情确认。
  const unreviewedFiles = () => {
    const stagedSet = staged()
    // 状态中已修改/新增但未暂存的条目即未审阅变更
    return diff()?.files.filter(f => !stagedSet.has(f.path)).length ?? 0
  }

  // 提交已暂存内容（git commit -m）。若有未审阅变更，先经确认模态（知情关卡），确认后提交。
  const doCommit = async () => {
    if (busy() !== null) return
    const msg = commitMsg().trim()
    if (!msg) {
      setError('请输入 commit message')
      return
    }
    // 审阅关卡：未审阅文件数 > 0 时，先弹确认识别（唯二入口：已审阅再提交 / 显式跳过）
    const pending = unreviewedFiles()
    if (pending > 0 && !commitGateBypassed()) {
      setPendingCommitMsg(msg)
      setModalReq({
        title: '存在未审阅变更',
        message: `还有 ${pending} 个文件的变更未审阅（未暂存）。git commit 将仅包含已暂存内容。是否继续提交？`,
        danger: false,
        confirmLabel: '继续提交',
        cancelLabel: '回去审阅',
      })
      return
    }
    await execCommit(msg)
  }

  const execCommit = async (msg: string) => {
    if (busy() !== null) return
    setBusy('commit')
    setError(null)
    try {
      await neocodex.gitCommit(msg)
      setCommitMsg('')
      setCommitGateBypassed(false)
      showToast('提交成功')
      await load()
    } catch (e) {
      // 🟡 修复：提交失败（hook 拒绝/冲突等）后必须复位审阅关卡，否则本会话内
      // 后续提交会静默跳过「未审阅变更」知情关卡
      setCommitGateBypassed(false)
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  // 推送当前分支到远程，成功反馈远程输出摘要。
  const doPush = async () => {
    if (busy() !== null) return
    setBusy('push')
    setError(null)
    try {
      const summary = await neocodex.gitPush()
      showToast(summary || '推送成功')
    } catch (e) {
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  // 切换分支（git checkout）：busy 防并发；成功后以后端返回的分支名对齐下拉并整体刷新，
  // 失败回滚到原分支（load 内 `if (!selectedBranch())` 守卫不会覆盖显式 set 的值）
  const doCheckout = async (branch: string) => {
    if (busy() !== null || branch === selectedBranch()) return
    const prev = selectedBranch()
    setBusy('checkout')
    setError(null)
    try {
      const checkedOut = await neocodex.gitCheckout(branch)
      setSelectedBranch(checkedOut)
      showToast(`已切换到分支 ${checkedOut}`)
      await load()
    } catch (e) {
      setSelectedBranch(prev)
      setError(errText(e))
    } finally {
      setBusy(null)
    }
  }

  const renderLine = (line: DiffLine) => {
    const num = line.t === 'add' ? line.n : line.t === 'del' ? line.o : line.n
    return (
      <div
        class={clsx(
          'flex items-start gap-2 px-2 py-1 text-xs font-mono leading-relaxed',
          line.t === 'add' && 'bg-emerald-500/10 text-emerald-300',
          line.t === 'del' && 'bg-red-500/10 text-red-300',
          line.t === 'ctx' && 'text-text-secondary'
        )}
      >
        <span class="w-8 flex-shrink-0 text-right text-text-muted/50 select-none">
          {line.t === 'add' ? '+' : line.t === 'del' ? '-' : ' '}
          {num ?? ''}
        </span>
        <span class="whitespace-pre-wrap break-all">{line.s}</span>
      </div>
    )
  }

  const totalChanges = () => {
    const d = diff()
    if (!d) return 0
    // 已暂存文件不计入待处理变更数，accept 后徽章随之下跌
    return d.files
      .filter(f => !staged().has(f.path))
      .reduce((acc, f) => acc + f.hunks.reduce((a, h) => a + h.lines.filter(l => l.t !== 'ctx').length, 0), 0)
  }

  return (
    <Show when={props.open}>
      <div
        ref={panelRef}
        class="panel w-[30rem]"
        role="dialog"
        aria-modal="true"
        aria-label="Git 面板"
        tabIndex={-1}
        onKeyDown={(e) => {
          if (e.key === 'Escape') {
            e.preventDefault()
            props.onClose()
            return
          }
          if (e.key === 'Tab' && panelRef) {
            const focusables = panelRef.querySelectorAll<HTMLElement>(
              'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
            )
            if (focusables.length === 0) return
            const first = focusables[0]
            const last = focusables[focusables.length - 1]
            const active = document.activeElement
            if (e.shiftKey && (active === first || active === panelRef)) {
              e.preventDefault()
              last.focus()
            } else if (!e.shiftKey && active === last) {
              e.preventDefault()
              first.focus()
            }
          }
        }}
      >
        {/* Header */}
        <div class="panel-head">
          <GitBranch class="panel-head-icon text-nt-act-600" />
          <span class="panel-title">Git 变更</span>
          <Show when={status()}>
            <span class="panel-sub font-mono">{status()!.branch}</span>
            <span class={clsx(status()!.dirty ? 'badge-warn' : 'badge-success')}>
              {status()!.dirty ? `${totalChanges()} 处变更` : '干净'}
            </span>
          </Show>
          <button
            ref={firstBtnRef}
            class="panel-close"
            onClick={() => void load()}
            aria-label="刷新"
            title="刷新"
          >
            <RefreshCw class={clsx('w-4 h-4', loading() && 'animate-spin')} />
          </button>
          <button
            class="p-2 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
            onClick={props.onClose}
            aria-label="关闭"
          >
            <X class="w-4 h-4" />
          </button>
        </div>

        {/* Git 操作区：分支选择 / commit message / commit / push */}
        <div class="border-b border-border-primary p-3 space-y-2">
          <div class="flex items-center gap-2">
            {busy() === 'checkout' ? (
              <Loader2 class="w-3.5 h-3.5 animate-spin text-text-muted flex-shrink-0" />
            ) : (
              <GitBranch class="w-3.5 h-3.5 text-text-muted flex-shrink-0" />
            )}
            <select
              class="flex-1 min-w-0 bg-bg-secondary border border-border-primary rounded-lg px-2 py-1.5 text-xs font-mono text-text-primary focus:outline-none focus:ring-1 focus:ring-nt-io-500 disabled:opacity-50"
              value={selectedBranch()}
              onChange={(e) => void doCheckout(e.currentTarget.value)}
              disabled={busy() !== null}
              aria-label="分支"
            >
              <For each={branches()}>
                {(b) => <option value={b}>{b}</option>}
              </For>
            </select>
          </div>
<Show when={unreviewedFiles() > 0}>
              <div
                class="flex items-center gap-1.5 px-2 py-1 rounded-md text-[10px] text-amber-700 bg-amber-500/10 border border-amber-500/30"
                role="status"
                title="这些变更尚未在面板中接受（git add 暂存），提交前将提示确认"
              >
                <AlertTriangle class="w-3 h-3 flex-shrink-0" />
                {unreviewedFiles()} 个文件未审阅
              </div>
            </Show>
            <div class="flex items-center gap-2">
              <input
              class="flex-1 min-w-0 bg-bg-secondary border border-border-primary rounded-lg px-2 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-nt-io-500 disabled:opacity-50"
              placeholder="Commit message"
              value={commitMsg()}
              onInput={(e) => setCommitMsg(e.currentTarget.value)}
              onKeyDown={(e) => { if (e.key === 'Enter' && busy() === null) void doCommit() }}
              disabled={busy() !== null}
              aria-label="Commit message"
            />
            <button
              class="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium text-white bg-nt-act-600 hover:bg-nt-act-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex-shrink-0"
              onClick={() => void doCommit()}
              disabled={busy() !== null || !commitMsg().trim()}
              aria-label="提交暂存变更"
              title="git commit（提交已暂存内容）"
            >
              {busy() === 'commit' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <GitCommitHorizontal class="w-3.5 h-3.5" />}
              Commit
            </button>
            <button
              class="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium text-white bg-nt-io-600 hover:bg-nt-io-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex-shrink-0"
              onClick={() => void doPush()}
              disabled={busy() !== null}
              aria-label="推送到远程"
              title="git push"
            >
              {busy() === 'push' ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Upload class="w-3.5 h-3.5" />}
              Push
            </button>
          </div>
        </div>

        {/* Body */}
        <div class="flex-1 overflow-y-auto p-3">
          <Show when={loading()}>
            <div class="flex items-center justify-center gap-2 py-8 text-text-muted text-sm">
              <Loader2 class="w-4 h-4 animate-spin" />
              加载变更...
            </div>
          </Show>
          <Show when={toast()}>
            <div class="flex items-center gap-2 p-3 mb-2 text-xs text-emerald-500 bg-emerald-500/10 rounded-lg">
              <Check class="w-3.5 h-3.5 flex-shrink-0" />
              <span class="break-all">{toast()}</span>
              <button
                onClick={() => setToast(null)}
                class="ml-auto p-1 hover:bg-emerald-500/20 rounded"
                aria-label="关闭提示"
              >
                ×
              </button>
            </div>
          </Show>
          <Show when={error()}>
            <div class="p-3 text-xs text-red-500 bg-red-500/10 rounded-lg mb-2">{error()}</div>
          </Show>
          <Show when={!loading() && diff() && diff()!.files?.length === 0 && !error()}>
            <div class="py-10 text-center text-xs text-text-muted">工作区干净，没有未提交的变更</div>
          </Show>
          <Show when={!loading() && diff() && (diff()!.files?.length ?? 0) > 0}>
            <div class="space-y-2">
              <For each={diff()!.files}>
                {(file) => {
                  const isOpen = () => expanded().has(file.path)
                  const isStaged = () => staged().has(file.path)
                  const addCount = file.hunks.reduce((a, h) => a + h.lines.filter(l => l.t === 'add').length, 0)
                  const delCount = file.hunks.reduce((a, h) => a + h.lines.filter(l => l.t === 'del').length, 0)
                  return (
                    <div class={clsx(
                      'rounded-lg border overflow-hidden bg-bg-primary/40',
                      isStaged() ? 'border-emerald-500/40 bg-emerald-500/[0.04]' : 'border-border-primary'
                    )}>
                      {/* File header */}
                      <div class="flex items-center gap-2 px-3 py-2 bg-bg-secondary/60">
                        <button
                          class="flex items-center gap-2 flex-1 min-w-0 text-left"
                          onClick={() => toggle(file.path)}
                          aria-expanded={isOpen()}
                        >
                          {isOpen() ? (
                            <ChevronDown class="w-4 h-4 text-text-muted flex-shrink-0" />
                          ) : (
                            <ChevronRight class="w-4 h-4 text-text-muted flex-shrink-0" />
                          )}
                          <FileCode2 class="w-4 h-4 text-nt-act-600 flex-shrink-0" />
                          <span class="text-sm text-text-primary truncate font-mono">{file.path}</span>
                        </button>
                        <Show when={isStaged()}>
                          <span
                            class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] text-emerald-600 bg-emerald-500/15 border border-emerald-500/30 flex-shrink-0"
                            title="已通过 git add 暂存"
                          >
                            <Check class="w-3 h-3" />
                            已暂存
                          </span>
                        </Show>
                        <span class="text-xs text-emerald-600 flex-shrink-0">+{addCount}</span>
                        <span class="text-xs text-red-500 flex-shrink-0">-{delCount}</span>
                        <button
                          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-emerald-600 hover:bg-emerald-500/10 border border-emerald-500/30 flex-shrink-0"
                          onClick={() => apply(file.path, 'accept')}
                          disabled={busy() !== null}
                          aria-label="接受变更"
                          title="git add"
                        >
                          {busy() === `${file.path}:accept` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <Check class="w-3.5 h-3.5" />}
                          接受
                        </button>
                        <button
                          class="flex items-center gap-1 px-2 py-1 rounded-lg text-xs text-red-500 hover:bg-red-500/10 border border-red-500/30 flex-shrink-0"
                          onClick={() => apply(file.path, 'reject')}
                          disabled={busy() !== null}
                          aria-label="拒绝此变更"
                          title="git restore"
                        >
                          {busy() === `${file.path}:reject` ? <Loader2 class="w-3.5 h-3.5 animate-spin" /> : <X class="w-3.5 h-3.5" />}
                          拒绝
                        </button>
                      </div>
                      {/* Diff body */}
                      <Show when={isOpen()}>
                        <div class="border-t border-border-primary">
                          <For each={file.hunks}>
                            {(hunk) => (
                              <div class="py-1">
                                <For each={hunk.lines}>
                                  {(line) => renderLine(line)}
                                </For>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                    </div>
                  )
                }}
              </For>
            </div>
          </Show>
        </div>
      </div>

      <ConfirmModal
        req={modalReq()}
        onConfirm={() => {
          if (pendingRejectPath()) {
            void doApply(pendingRejectPath()!, 'reject')
          }
          if (pendingCommitMsg()) {
            // 返回为 null 再提取，避免借用冲突；确认后允许本次会话内无条件提交倒未审阅变更清零
            setCommitGateBypassed(true)
            void execCommit(pendingCommitMsg()!)
          }
          setPendingRejectPath(null)
          setPendingCommitMsg(null)
          setModalReq(null)
        }}
        onClose={() => {
          setPendingRejectPath(null)
          setPendingCommitMsg(null)
          setModalReq(null)
        }}
      />
    </Show>
  )
}