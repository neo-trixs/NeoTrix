import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, waitFor } from '@solidjs/testing-library'
import { GitPanel } from './GitPanel'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

/** mock domain_call（git 域），返回后端真实形状 */
function mockGitDomain(handlers: Record<string, (args: any) => unknown>) {
  return mockCommand('domain_call', async (req: any) => {
    const fn = handlers[`${req.domain}/${req.action}`]
    if (!fn) throw new Error(`unexpected domain call: ${req.domain}/${req.action}`)
    return { ok: true, data: await fn(req.args) }
  })
}

function diffFiles() {
  return [
    {
      path: 'src/foo.rs',
      hunks: [
        {
          lines: [
            { t: 'add', o: null, n: 1, s: '+fn new() {}' },
            { t: 'del', o: 1, n: null, s: '-fn old() {}' },
          ],
        },
      ],
    },
  ]
}

function gitStubs(over: Record<string, (args: any) => unknown> = {}) {
  return mockGitDomain({
    'git/status': async () => ({ branch: 'main', clean: false, files: [{ status: 'M', path: 'src/foo.rs' }], count: 1 }),
    'git/diff': async () => ({ diff: '', lines: 2, files: diffFiles(), hunks: [] }),
    'git/staged_files': async () => [],
    'git/branches': async () => ({ current: 'main', branches: ['main'], count: 1 }),
    ...over,
  })
}

describe('GitPanel 渲染回归（P1-1：diff 列表必须渲染）', () => {
  beforeEach(() => {
    resetInvokeMock()
  })

  it('加载完成后显示 diff 文件列表而非永久 spinner', async () => {
    gitStubs()
    render(() => <GitPanel open onClose={() => {}} />)
    // 初始加载中：显示 spinner
    expect(document.querySelector('.animate-spin')).toBeTruthy()
    // 加载完成：diff 文件路径出现，spinner 消失
    await waitFor(() => {
      expect(document.body.textContent).toContain('src/foo.rs')
    })
    expect(document.querySelector('.animate-spin')).toBeNull()
    // 空态文案不应出现
    expect(document.body.textContent).not.toContain('工作区干净')
  })

  it('工作区干净时显示空态而非 spinner', async () => {
    gitStubs({
      'git/diff': async () => ({ diff: '', lines: 0, files: [], hunks: [] }),
      'git/status': async () => ({ branch: 'main', clean: true, files: [], count: 0 }),
    })
    render(() => <GitPanel open onClose={() => {}} />)
    await waitFor(() => {
      expect(document.body.textContent).toContain('工作区干净')
    })
  })
})
