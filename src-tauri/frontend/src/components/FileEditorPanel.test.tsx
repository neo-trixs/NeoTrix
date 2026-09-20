import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, fireEvent, waitFor } from '@solidjs/testing-library'
import { FileEditorPanel } from './FileEditorPanel'
import { mockCommand, mockInvokeImpl, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

describe('FileEditorPanel', () => {
  beforeEach(() => resetInvokeMock())
  afterEach(() => resetInvokeMock())

  it('载入文件填充编辑器，写回调用 write_file', async () => {
    const read = mockCommand('read_file', async () => 'fn main() {}')
    const write = mockCommand('write_file', async () => undefined)
    const { container, getByText, getByPlaceholderText } = render(() => <FileEditorPanel onClose={() => {}} />)

    const pathInput = container.querySelector('input') as HTMLInputElement
    fireEvent.input(pathInput, { target: { value: './src/main.rs' } })
    fireEvent.click(getByText('载入'))
    await waitFor(() => expect(read.calledTimes()).toBe(1))
    expect(read.lastArgs()).toMatchObject({ path: './src/main.rs' })

    const ta = getByPlaceholderText('载入文件后在此编辑…') as HTMLTextAreaElement
    await waitFor(() => expect(ta.value).toBe('fn main() {}'))

    fireEvent.input(ta, { target: { value: 'fn main() { println!(); }' } })
    fireEvent.click(getByText('写回文件'))
    await waitFor(() => expect(write.calledTimes()).toBe(1))
    expect(write.lastArgs()).toMatchObject({ path: './src/main.rs', content: 'fn main() { println!(); }' })
  })

  it('读取失败显示错误且不填充', async () => {
    mockCommand('read_file', async () => { throw new Error('ENOENT') })
    const { container, getByText, getByPlaceholderText } = render(() => <FileEditorPanel onClose={() => {}} />)
    const pathInput = container.querySelector('input') as HTMLInputElement
    fireEvent.input(pathInput, { target: { value: './missing.rs' } })
    fireEvent.click(getByText('载入'))
    await waitFor(() => expect(getByText(/读取失败/)).toBeTruthy())
    const ta = getByPlaceholderText('载入文件后在此编辑…') as HTMLTextAreaElement
    expect(ta.value).toBe('')
  })
})
