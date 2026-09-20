import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AgentActivityLog, type ActivityStep } from './AgentActivityLog'

const base = (over: Partial<ActivityStep> = {}): ActivityStep => ({
  kind: 'phase',
  label: '开始思考',
  ts: Date.now(),
  ...over,
})

describe('AgentActivityLog', () => {
  it('空日志显示占位', () => {
    const { container } = render(() => <AgentActivityLog steps={() => []} />)
    expect(container.querySelector('.agent-log__empty')).toBeTruthy()
  })

  it('渲染多类步骤并着色', () => {
    const steps = [
      base({ kind: 'phase', label: '开始思考' }),
      base({ kind: 'tool', label: 'search', detail: '成功', domain: 'NT-MEMORY' }),
      base({ kind: 'error', label: '超时' }),
      base({ kind: 'done', label: '完成' }),
    ]
    const { container } = render(() => <AgentActivityLog steps={() => steps} />)
    expect(container.querySelectorAll('.agent-log__row').length).toBe(4)
    expect(container.querySelector('.agent-log__dot--tool')).toBeTruthy()
    expect(container.querySelector('.agent-log__dot--error')).toBeTruthy()
    expect(container.textContent).toContain('NT-MEMORY')
  })

  it('推理步骤使用 reasoning 色点', () => {
    const { container } = render(() => (
      <AgentActivityLog steps={() => [base({ kind: 'reasoning', label: '检索知识库…' })]} />
    ))
    expect(container.querySelector('.agent-log__dot--reasoning')).toBeTruthy()
  })
})
