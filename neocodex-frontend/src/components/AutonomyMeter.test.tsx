import { describe, it, expect } from 'vitest'
import { render } from '@solidjs/testing-library'
import { AutonomyMeter } from './AutonomyMeter'

describe('AutonomyMeter', () => {
  it('渲染模式标签与通过率', () => {
    const { container } = render(() => (
      <AutonomyMeter level={() => 2} mode={() => '自动'} rate={() => 100} />
    ))
    expect(container.textContent).toContain('自动')
    expect(container.textContent).toContain('100%')
  })
  it('等级点亮对应段数', () => {
    const { container } = render(() => (
      <AutonomyMeter level={() => 3} mode={() => '接受编辑'} rate={() => 92} />
    ))
    expect(container.querySelectorAll('.autonomy-meter__seg--on').length).toBe(3)
    expect(container.querySelectorAll('.autonomy-meter__seg').length).toBe(4)
  })
  it('等级 0 不点亮任何段', () => {
    const { container } = render(() => (
      <AutonomyMeter level={() => 0} mode={() => '手动'} rate={() => 100} />
    ))
    expect(container.querySelectorAll('.autonomy-meter__seg--on').length).toBe(0)
  })
})
