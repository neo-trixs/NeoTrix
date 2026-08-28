import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentActivityBar } from './AgentActivityBar'

describe('AgentActivityBar', () => {
  it('idle 阶段不渲染任何内容', () => {
    const { container } = render(() => (
      <AgentActivityBar phase={() => 'idle'} activeDomain={() => null} toolCount={() => 0} lastActivity={() => null} />
    ))
    expect(container.querySelector('.agent-activity')).toBeNull()
  })

  it('思考阶段显示标签与呼吸点', () => {
    const { container } = render(() => (
      <AgentActivityBar phase={() => 'thinking'} activeDomain={() => null} toolCount={() => 0} lastActivity={() => null} />
    ))
    const el = container.querySelector('.agent-activity--thinking')
    expect(el).toBeTruthy()
    expect(container.textContent).toContain('思考中')
  })

  it('工具调用阶段显示活跃域与工具计数', () => {
    const { container } = render(() => (
      <AgentActivityBar
        phase={() => 'tooling'}
        activeDomain={() => 'NT-MEMORY'}
        toolCount={() => 2}
        lastActivity={() => '工具调用：search'}
      />
    ))
    const el = container.querySelector('.agent-activity--tooling') as HTMLElement
    expect(el).toBeTruthy()
    expect(el.getAttribute('title')).toContain('工具调用：search')
    expect(container.textContent).toContain('NT-MEMORY')
    expect(container.textContent).toContain('2')
  })

  it('错误阶段应用错误态 class', () => {
    const { container } = render(() => (
      <AgentActivityBar phase={() => 'error'} activeDomain={() => null} toolCount={() => 0} lastActivity={() => '超时'} />
    ))
    expect(container.querySelector('.agent-activity--error')).toBeTruthy()
  })

  it('完成阶段应用成功态 class', () => {
    const { container } = render(() => (
      <AgentActivityBar phase={() => 'done'} activeDomain={() => null} toolCount={() => 1} lastActivity={() => null} />
    ))
    expect(container.querySelector('.agent-activity--done')).toBeTruthy()
  })
})
