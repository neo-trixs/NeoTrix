import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render } from '@solidjs/testing-library'
import { RightSidebar } from '../RightSidebar'

vi.mock('../../../../stores/chat', () => ({
  chatStore: {
    get currentMessages() {
      return [
        { id: 'u1', role: 'user', content: '帮我拆分模块', timestamp: new Date(), toolCalls: [] },
        { id: 'a1', role: 'assistant', content: '已按方案拆分', timestamp: new Date(), isStreaming: false, toolCalls: [{ id: 't1', name: 'write_file', args: '{}', result: 'ok', duration_ms: 100, success: true }] },
      ]
    },
  },
}))

vi.mock('../../../../components/CausalMap', () => ({
  CausalMap: (props: any) => <div data-testid="causal-map" data-nodes={props.nodes.length}>{props.nodes.map((n: any) => n.title).join(',')}</div>,
}))

describe('RightSidebar — 因果链画板', () => {
  it('从 chatStore 提取节点并渲染 CausalMap', () => {
    const { container } = render(() => <RightSidebar />)
    const map = container.querySelector('[data-testid="causal-map"]') as HTMLElement
    expect(map).toBeTruthy()
    expect(Number(map.getAttribute('data-nodes'))).toBe(3) // user + assistant + tool
    expect(map.textContent).toContain('你的请求')
    expect(map.textContent).toContain('AI 回复')
    expect(map.textContent).toContain('工具调用')
  })

  it('渲染侧栏容器与切换按钮', () => {
    const { container } = render(() => <RightSidebar />)
    expect(container.querySelector('aside.rb')).toBeTruthy()
    expect(container.querySelector('button.rb-float')).toBeTruthy()
  })
})
