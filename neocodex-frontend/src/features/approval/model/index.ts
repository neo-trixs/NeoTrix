// features/approval/model — 审批状态机
export type ApprovalStatus = 'pending' | 'approved' | 'rejected' | 'expired'

export interface ApprovalRequest {
  id: string
  title: string
  description: string
  status: ApprovalStatus
  createdAt: number
  expiresAt?: number
}

export function isExpired(req: ApprovalRequest, now = Date.now()): boolean {
  return req.expiresAt !== undefined && now > req.expiresAt
}

export function canApprove(req: ApprovalRequest, now = Date.now()): boolean {
  return req.status === 'pending' && !isExpired(req, now)
}

export function transition(req: ApprovalRequest, to: ApprovalStatus, now = Date.now()): ApprovalRequest {
  if (req.status !== 'pending') throw new Error(`Cannot transition from ${req.status}`)
  if (isExpired(req, now) && to !== 'expired') throw new Error('Request expired')
  return { ...req, status: to }
}

export function autoExpire(req: ApprovalRequest, now = Date.now()): ApprovalRequest {
  if (req.status === 'pending' && isExpired(req, now)) return { ...req, status: 'expired' }
  return req
}
