import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'

vi.mock('../../../../components/ModelSwitcher', () => ({
  ModelSwitcher: (props: any) => <div data-testid="model-switcher" data-disabled={String(props.disabled)} />,
}))

vi.mock('../../../../lib/text', async () => {
  const actual: any = await vi.importActual('../../../../lib/text')
  return {
    ...actual,
    estimateTokens: vi.fn(() => 42),
  }
})

import { InputArea } from '../InputArea'

describe('InputArea', () => {
  const baseProps = () => ({
    inputValue: () => 'hello world',
    onInput: vi.fn(),
    onKeyDown: vi.fn(),
    onPaste: vi.fn(),
    textareaRef: vi.fn(),
    isGenerating: () => false,
    pendingAttachments: () => [],
    annotationHint: () => null,
    onPickAttachment: vi.fn(),
    onSend: vi.fn(),
    onStop: vi.fn(),
  })

  it('渲染 textarea 与占位文案（非生成中）', () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const ta = container.querySelector('textarea') as HTMLTextAreaElement
    expect(ta).toBeTruthy()
    expect(ta.placeholder).toContain('输入消息')
    expect(ta.value).toBe('hello world')
  })

  it('生成中占位文案切换', () => {
    const props = { ...baseProps(), isGenerating: () => true }
    const { container } = render(() => <InputArea {...props} />)
    const ta = container.querySelector('textarea') as HTMLTextAreaElement
    expect(ta.placeholder).toContain('生成中仍可输入')
  })

  it('显示 token 估算与附件计数', () => {
    const props = {
      ...baseProps(),
      pendingAttachments: () => [{ name: 'a.txt', size: 10, mime_type: 'text/plain', data: 'hi' } as any],
    }
    const { container } = render(() => <InputArea {...props} />)
    expect(container.textContent).toContain('42 tok')
    expect(container.textContent).toContain('1 附件')
  })

  it('空输入且无附件时 token 不显示', () => {
    const props = { ...baseProps(), inputValue: () => '   ', pendingAttachments: () => [] }
    const { container } = render(() => <InputArea {...props} />)
    expect(container.textContent).not.toContain('tok')
  })

  it('触发 onInput', async () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const ta = container.querySelector('textarea') as HTMLTextAreaElement
    await fireEvent.input(ta, { target: { value: 'new text' } })
    expect(props.onInput).toHaveBeenCalled()
  })

  it('触发 onKeyDown', async () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const ta = container.querySelector('textarea') as HTMLTextAreaElement
    await fireEvent.keyDown(ta, { key: 'Enter' })
    expect(props.onKeyDown).toHaveBeenCalled()
  })

  it('触发 onPaste', async () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const ta = container.querySelector('textarea') as HTMLTextAreaElement
    await fireEvent.paste(ta, { clipboardData: { items: [] } } as any)
    expect(props.onPaste).toHaveBeenCalled()
  })

  it('点击附件按钮触发 onPickAttachment', async () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const btn = container.querySelector('.cic-attach') as HTMLButtonElement
    await fireEvent.click(btn)
    expect(props.onPickAttachment).toHaveBeenCalledTimes(1)
  })

  it('非生成中点击发送触发 onSend', async () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    const btn = container.querySelector('.vc-send') as HTMLButtonElement
    await fireEvent.click(btn)
    expect(props.onSend).toHaveBeenCalledTimes(1)
  })

  it('生成中点击触发 onStop', async () => {
    const props = { ...baseProps(), isGenerating: () => true }
    const { container } = render(() => <InputArea {...props} />)
    const btn = container.querySelector('.vc-send') as HTMLButtonElement
    await fireEvent.click(btn)
    expect(props.onStop).toHaveBeenCalledTimes(1)
  })

  it('渲染 ModelSwitcher 占位', () => {
    const props = baseProps()
    const { container } = render(() => <InputArea {...props} />)
    expect(container.querySelector('[data-testid="model-switcher"]')).toBeTruthy()
  })
})
