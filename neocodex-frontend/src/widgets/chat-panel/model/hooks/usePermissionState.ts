/**
 * usePermissionState — 权限模式切换 + 自治度评估
 */
import { createSignal } from 'solid-js'
import { PERMISSION_MODES, type PermissionMode } from '../../../../components/PermissionModeSelector'

export function usePermissionState(deps: {
  isGenerating: () => boolean
  showInfo: (msg: string, ms?: number) => void
}) {
  const [permissionMode, setPermissionMode] = createSignal<PermissionMode>(
    (typeof localStorage !== 'undefined' && (localStorage.getItem('nt_perm_mode') as PermissionMode)) || 'auto',
  )

  const persistPermissionMode = (m: PermissionMode) => {
    try { localStorage.setItem('nt_perm_mode', m) } catch { /* 隐私模式忽略 */ }
  }

  const cyclePermissionMode = () => {
    if (deps.isGenerating()) return
    const idx = PERMISSION_MODES.findIndex((m) => m.value === permissionMode())
    const next = PERMISSION_MODES[(idx + 1) % PERMISSION_MODES.length]
    setPermissionMode(next.value)
    persistPermissionMode(next.value)
    deps.showInfo(`权限模式：${next.label}`, 2500)
  }

  const [approvalAccepted, setApprovalAccepted] = createSignal(0)
  const [approvalRejected, setApprovalRejected] = createSignal(0)

  const autonomyLevel = () => {
    const total = approvalAccepted() + approvalRejected()
    if (total === 0) return '待校准' as const
    const rate = approvalAccepted() / total
    if (rate >= 0.8) return '高信任' as const
    if (rate >= 0.5) return '协作' as const
    return '审慎' as const
  }

  const autonomyRate = () => {
    const total = approvalAccepted() + approvalRejected()
    return total === 0 ? 0 : approvalAccepted() / total
  }

  const autonomyLevelNum = () => {
    const m = permissionMode()
    if (m === 'manual') return 0
    if (m === 'plan') return 1
    if (m === 'auto') return 2
    if (m === 'accept_edits') return 3
    return 1
  }

  return {
    permissionMode, setPermissionMode, cyclePermissionMode,
    approvalAccepted, setApprovalAccepted,
    approvalRejected, setApprovalRejected,
    autonomyLevel, autonomyRate, autonomyLevelNum,
  }
}
