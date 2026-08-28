import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { ApprovalPanel } from './ApprovalPanel'
import type { HarnessApproval } from '../api/harness'

describe('ApprovalPanel', () => {
  it('空队列时提示无待审批', () => {
    const { container } = render(() => <ApprovalPanel approvals={[]} onClose={() => {}} />)
    expect(container.textContent).toContain('无待审批动作')
  })

  it('展示待审批动作 + 待处理计数', () => {
    const list: HarnessApproval[] = [
      { id: 'a1', action: 'shell:rm -rf', state: 'pending' },
      { id: 'a2', action: 'network:curl', state: 'approved' },
    ]
    const { container } = render(() => <ApprovalPanel approvals={list} onClose={() => {}} />)
    expect(container.textContent).toContain('待审批动作')
    expect(container.textContent).toContain('shell:rm -rf')
    expect(container.textContent).toContain('1 待处理')
  })
})
