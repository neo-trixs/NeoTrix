import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, fireEvent, waitFor } from '@solidjs/testing-library'
import { ApprovalPanel } from './ApprovalPanel'
import type { HarnessApproval } from '../api/harness'
import { mockInvokeImpl, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

describe('ApprovalPanel', () => {
  beforeEach(() => resetInvokeMock())
  afterEach(() => resetInvokeMock())

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

  // 后端无审批队列：harnessApprovalResolve 明确抛出 NOT_IMPLEMENTED，
  // 组件 catch 打日志且不触发 onResolved，按钮恢复可用。
  it('点击「通过」后端未实现时不触发 onResolved 且按钮恢复', async () => {
    let resolved = 0
    const list: HarnessApproval[] = [{ id: 'a1', action: 'shell:rm -rf', state: 'pending' }]
    const { getByText } = render(() => (
      <ApprovalPanel approvals={list} onClose={() => {}} onResolved={() => { resolved++ }} />
    ))
    fireEvent.click(getByText('通过'))
    await waitFor(() => expect(resolved).toBe(0))
    expect(getByText('通过')).toBeTruthy()
  })

  it('点击「拒绝」后端未实现时不触发 onResolved', async () => {
    let resolved = 0
    const list: HarnessApproval[] = [{ id: 'a1', action: 'shell:rm -rf', state: 'pending' }]
    const { getByText } = render(() => (
      <ApprovalPanel approvals={list} onClose={() => {}} onResolved={() => { resolved++ }} />
    ))
    fireEvent.click(getByText('拒绝'))
    await waitFor(() => expect(resolved).toBe(0))
    expect(getByText('拒绝')).toBeTruthy()
  })
})
