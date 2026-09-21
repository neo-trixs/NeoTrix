import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, fireEvent, waitFor } from '@solidjs/testing-library'
import { FileEditorPanel } from './FileEditorPanel'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

describe('FileEditorPanel', () => {
  beforeEach(() => resetInvokeMock())
  afterEach(() => resetInvokeMock())

  /** mock domain_call（file 域），记录每次调用的 action/args */
  function mockFileDomain(handlers: Record<string, (args: any) => unknown>) {
    const calls: { action: string; args: any }[] = []
    mockCommand('domain_call', async (req: any) => {
      calls.push({ action: req.action, args: req.args })
      const fn = handlers[req.action]
      if (!fn) throw new Error(`unexpected domain action: ${req.action}`)
      return { ok: true, data: await fn(req.args) }
    })
    return calls
  }

  it('载入文件填充编辑器，写回调用 file/write', async () => {
    const calls = mockFileDomain({
      read: async () => 'fn main() {}',
      write: async () => null,
    })
    const { container, getByText, getByPlaceholderText } = render(() => <FileEditorPanel onClose={() => {}} />)

    const pathInput = container.querySelector('input') as HTMLInputElement
    fireEvent.input(pathInput, { target: { value: './src/main.rs' } })
    fireEvent.click(getByText('载入'))
    await waitFor(() => expect(calls.filter((c) => c.action === 'read')).toHaveLength(1))
    expect(calls.find((c) => c.action === 'read')!.args).toMatchObject({ path: './src/main.rs' })

    const ta = getByPlaceholderText('载入文件后在此编辑…') as HTMLTextAreaElement
    await waitFor(() => expect(ta.value).toBe('fn main() {}'))

    fireEvent.input(ta, { target: { value: 'fn main() { println!(); }' } })
    fireEvent.click(getByText('写回文件'))
    await waitFor(() => expect(calls.filter((c) => c.action === 'write')).toHaveLength(1))
    expect(calls.find((c) => c.action === 'write')!.args).toMatchObject({ path: './src/main.rs', content: 'fn main() { println!(); }' })
  })

  it('读取失败显示错误且不填充', async () => {
    mockFileDomain({
      read: async () => {
        throw new Error('ENOENT')
      },
    })
    const { container, getByText, getByPlaceholderText } = render(() => <FileEditorPanel onClose={() => {}} />)
    const pathInput = container.querySelector('input') as HTMLInputElement
    fireEvent.input(pathInput, { target: { value: './missing.rs' } })
    fireEvent.click(getByText('载入'))
    await waitFor(() => expect(getByText(/读取失败/)).toBeTruthy())
    const ta = getByPlaceholderText('载入文件后在此编辑…') as HTMLTextAreaElement
    expect(ta.value).toBe('')
  })
})
