// features/approve-tool/model — 审批工具执行
export type ApproveAction = 'approve' | 'reject' | 'defer'

export interface ApproveDecision {
  toolCallId: string
  action: ApproveAction
  reason?: string
  decidedAt: number
}

export function createDecision(toolCallId: string, action: ApproveAction, reason?: string): ApproveDecision {
  if (action === 'reject' && !reason?.trim()) throw new Error('Reject requires reason')
  return { toolCallId, action, reason: reason?.trim(), decidedAt: Date.now() }
}

export function isValidDecision(d: ApproveDecision): boolean {
  if (!d.toolCallId) return false
  if (d.action === 'reject' && !d.reason) return false
  return true
}

export function labelForAction(action: ApproveAction): string {
  return { approve: '批准', reject: '拒绝', defer: '稍后' }[action]
}
