import { describe, it, expect } from 'vitest'
import { createSignal } from 'solid-js'
import { render, screen } from '@solidjs/testing-library'
import { Markdown } from './Markdown'

describe('Markdown', () => {
  it('renders plain paragraph text', () => {
    render(() => <Markdown content="hello world" />)
    expect(screen.getByText('hello world')).toBeTruthy()
  })

  it('renders headings with correct hierarchy', () => {
    render(() => <Markdown content={'# Title\n## Sub\n### Sub3'} />)
    expect(screen.getByRole('heading', { level: 1, name: 'Title' })).toBeTruthy()
    expect(screen.getByRole('heading', { level: 2, name: 'Sub' })).toBeTruthy()
    expect(screen.getByRole('heading', { level: 3, name: 'Sub3' })).toBeTruthy()
  })

  it('renders fenced code block with language label', () => {
    render(() => <Markdown content={'```rust\nfn main() {}\n```'} />)
    expect(screen.getByText('rust')).toBeTruthy()
    expect(screen.getByText('fn main() {}')).toBeTruthy()
  })

  it('renders unclosed fence as code block (no infinite loop)', () => {
    render(() => <Markdown content={'```js\nconst x = 1'} />)
    expect(screen.getByText('js')).toBeTruthy()
    expect(screen.getByText('const x = 1')).toBeTruthy()
  })

  it('renders unordered and ordered lists', () => {
    render(() => <Markdown content={'- a\n- b\n- c\n\n1. one\n2. two'} />)
    expect(screen.getByText('a')).toBeTruthy()
    expect(screen.getByText('b')).toBeTruthy()
    expect(screen.getByText('one')).toBeTruthy()
    expect(screen.getByText('two')).toBeTruthy()
  })

  it('renders blockquote merging consecutive lines', () => {
    render(() => <Markdown content={'> line1\n> line2'} />)
    expect(screen.getByText('line1')).toBeTruthy()
    expect(screen.getByText('line2')).toBeTruthy()
  })

  it('renders horizontal rule', () => {
    const { container } = render(() => <Markdown content={'---'} />)
    const hr = container.querySelector('div[class*="h-px"]')
    expect(hr).toBeTruthy()
  })

  it('renders table header and rows', () => {
    render(() => (
      <Markdown content={'| name | value |\n|------|-------|\n| a    | 1     |\n| b    | 2     |'} />
    ))
    expect(screen.getByText('name')).toBeTruthy()
    expect(screen.getByText('value')).toBeTruthy()
    expect(screen.getByText('a')).toBeTruthy()
    expect(screen.getByText('1')).toBeTruthy()
    expect(screen.getByText('b')).toBeTruthy()
    expect(screen.getByText('2')).toBeTruthy()
  })

  it('renders bold, italic, and inline code', () => {
    render(() => <Markdown content={'**bold** *italic* `code`'} />)
    expect(screen.getByText('bold')).toBeTruthy()
    expect(screen.getByText('italic')).toBeTruthy()
    expect(screen.getByText('code')).toBeTruthy()
    const bold = screen.getByText('bold')
    expect(bold.className).toContain('font-semibold')
  })

  it('renders safe http link with target blank', () => {
    render(() => <Markdown content={'[docs](https://example.com/x)'} />)
    const link = screen.getByRole('link', { name: 'docs' }) as HTMLAnchorElement
    expect(link.href).toBe('https://example.com/x')
    expect(link.target).toBe('_blank')
    expect(link.rel).toContain('noopener')
  })

  it('sanitizes javascript: links to # (XSS guard)', () => {
    render(() => <Markdown content={'[bad](javascript:alert(1))'} />)
    const link = screen.getByRole('link', { name: 'bad' })
    // jsdom 将相对 "#" 解析为绝对 URL，用 href 属性校验实际写入值
    expect(link.getAttribute('href')).toBe('#')
  })

  it('sanitizes data: links to # (XSS guard)', () => {
    const payload = 'data:text/html,' + '<' + 'script' + '>1<' + '/script' + '>'
    render(() => <Markdown content={'[bad](' + payload + ')'} />)
    const link = screen.getByRole('link', { name: 'bad' })
    expect(link.getAttribute('href')).toBe('#')
  })

  it('renders multiple paragraphs separated by blank lines', () => {
    render(() => <Markdown content={'para one\n\npara two'} />)
    expect(screen.getByText('para one')).toBeTruthy()
    expect(screen.getByText('para two')).toBeTruthy()
  })

  it('re-renders with new content via signal', () => {
    const [content, setContent] = createSignal('# head\n\nsome')
    render(() => <Markdown content={content()} />)
    expect(screen.getByText('head')).toBeTruthy()
    setContent('# head\n\nsome more **text**')
    expect(screen.getByText('text')).toBeTruthy()
  })
})