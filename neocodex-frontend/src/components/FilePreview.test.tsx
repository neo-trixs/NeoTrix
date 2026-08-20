import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { FilePreview } from './FilePreview'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

// base64 helper（真实 UTF-8 中文验证解码路径）
function b64(text: string): string {
  const bytes = new TextEncoder().encode(text)
  let bin = ''
  for (const b of bytes) bin += String.fromCharCode(b)
  return btoa(bin)
}

function makeAtt(over: Record<string, unknown> = {}) {
  return {
    id: 'att-1',
    name: 'test.rs',
    mime_type: 'text/rust',
    size: 123,
    data: b64('fn main() { println!("hello"); }'),
    ...over,
  }
}

describe('FilePreview 附件预览回归（图像/代码/CSV/回退分支）', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('代码文件显示文件名 + 大小 + 折叠内容', () => {
    render(() => <FilePreview attachment={makeAtt()} />)
    expect(document.body.textContent).toContain('test.rs')
    expect(document.body.textContent).toContain('123 B')
    // 内容默认折叠，点击展开
    fireEvent.click(document.querySelector('button[aria-expanded]')!)
    expect(document.body.textContent).toContain('fn main()')
  })

  it('中文内容经 UTF-8 解码不乱码', () => {
    render(() => (
      <FilePreview attachment={makeAtt({ name: '中文.md', mime_type: 'text/markdown', data: b64('你好世界') })} />
    ))
    fireEvent.click(document.querySelector('button[aria-expanded]')!)
    expect(document.body.textContent).toContain('你好世界')
  })

  it('超长内容截断显示', () => {
    const long = 'x'.repeat(15000)
    render(() => (
      <FilePreview attachment={makeAtt({ name: 'long.txt', mime_type: 'text/plain', data: b64(long) })} />
    ))
    fireEvent.click(document.querySelector('button[aria-expanded]')!)
    expect(document.body.textContent).toContain('已截断')
  })

  it('图像附件内联渲染 img', () => {
    render(() => (
      <FilePreview attachment={makeAtt({ name: 'pic.png', mime_type: 'image/png', data: 'data:image/png;base64,AAAA' })} />
    ))
    const img = document.querySelector('img')
    expect(img).toBeTruthy()
    expect(img!.getAttribute('alt')).toBe('pic.png')
  })

  it('CSV 附件渲染为表格', () => {
    render(() => (
      <FilePreview attachment={makeAtt({ name: 'data.csv', mime_type: 'text/csv', data: b64('name,age\nalice,30\nbob,25') })} />
    ))
    expect(document.querySelector('table')).toBeTruthy()
    expect(document.body.textContent).toContain('alice')
  })

  it('未知类型显示无预览回退', () => {
    render(() => (
      <FilePreview attachment={makeAtt({ name: 'weird.xyz', mime_type: 'application/x-weird' })} />
    ))
    expect(document.body.textContent).toContain('无可用预览')
  })

  it('移除按钮触发 onRemove', () => {
    const onRemove = vi.fn()
    render(() => <FilePreview attachment={makeAtt()} onRemove={onRemove} />)
    fireEvent.click(document.querySelector('[aria-label="移除附件"]')!)
    expect(onRemove).toHaveBeenCalled()
  })

  it('图像附件带 onAnnotate 时显示标注按钮', () => {
    render(() => (
      <FilePreview attachment={makeAtt({ name: 'pic.png', mime_type: 'image/png', data: 'AAAA' })} onAnnotate={() => {}} />
    ))
    expect(document.querySelector('[aria-label="标注图片"]')).toBeTruthy()
  })

  it('非图像附件无标注按钮', () => {
    render(() => (
      <FilePreview attachment={makeAtt()} onAnnotate={() => {}} />
    ))
    expect(document.querySelector('[aria-label="标注图片"]')).toBeNull()
  })
})