import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { GenUIView } from './GenUIView'

describe('GenUIView', () => {
  it('JSON 内容渲染为格式化 pre', () => {
    const json = JSON.stringify({ a: 1, b: [2, 3] })
    const { container } = render(() => <GenUIView content={() => json} />)
    expect(container.querySelector('.gen-ui__json')).toBeTruthy()
    expect(container.textContent).toContain('"a"')
  })

  it('markdown 表格渲染为 table 元素', () => {
    const md = '| name | val |\n| --- | --- |\n| x | 1 |\n| y | 2 |'
    const { container } = render(() => <GenUIView content={() => md} />)
    const table = container.querySelector('.gen-ui__table')
    expect(table).toBeTruthy()
    expect(table!.querySelectorAll('tbody tr').length).toBe(2)
    expect(table!.querySelectorAll('thead th').length).toBe(2)
  })

  it('普通文本渲染为文本块', () => {
    const { container } = render(() => <GenUIView content={() => 'just some plain text'} />)
    expect(container.querySelector('.gen-ui__text')).toBeTruthy()
    expect(container.querySelector('.gen-ui__json')).toBeNull()
  })
})
