import { describe, it, expect } from 'vitest'
import { parseRunCommand } from '../routes/chat/slashCommands'
import { render } from '@solidjs/testing-library'
import { HarnessReportCard } from './HarnessReportCard'
import type { HarnessRunResponse } from '../api/harness'

function makeReport(overrides: Partial<HarnessRunResponse> = {}): HarnessRunResponse {
  return {
    instruction: '测试任务',
    allocations: [
      { task: { capability_tag: 'xlsx_consolidation', domain: 'NT-IO', specialist: '数据', summary: '合并价格表' }, provider: { Internal: { path: ['nt_file_ability'] } } },
      { task: { capability_tag: 'github_research', domain: 'NT-WORLD', specialist: '探索', summary: '检索资料' }, provider: { External: { reason: '能力网无内置' } } },
    ],
    internal_count: 1,
    external_gap_count: 1,
    strengthening_actions: 0,
    external_gaps: [],
    internal_results: [{ task_id: 't1', summary: '合并价格表', provider_path: ['nt_file_ability'], executed: true, output: 'ok' }],
    external_closures: [{ solved: true, solution: '选型建议', knowledge_acquired: true }],
    ...overrides,
  }
}

describe('parseRunCommand', () => {
  it('匹配 /run <任务> 提取指令', () => {
    expect(parseRunCommand('/run 合并价格表')).toEqual({ isRun: true, instruction: '合并价格表' })
    expect(parseRunCommand('  /run   检索 GitHub 资料 ')).toEqual({ isRun: true, instruction: '检索 GitHub 资料' })
  })
  it('无参 /run 视为未提供指令（调用方给用法提示）', () => {
    expect(parseRunCommand('/run')).toEqual({ isRun: false, instruction: '' })
    expect(parseRunCommand('/run   ')).toEqual({ isRun: false, instruction: '' })
  })
  it('非 /run 前缀不匹配', () => {
    expect(parseRunCommand('运行价格表')).toEqual({ isRun: false, instruction: '' })
    expect(parseRunCommand('/model')).toEqual({ isRun: false, instruction: '' })
  })
})

describe('HarnessReportCard 真·流式阶段', () => {
  it('阶段1 allocated: running=true 显示「执行中」+ 分配视图（无完成计数）', () => {
    const r = makeReport({ internal_results: [], external_closures: [] })
    const { container } = render(() => <HarnessReportCard running={true} report={r} onClose={() => {}} />)
    expect(container.textContent).toContain('执行中')
    expect(container.textContent).toContain('xlsx_consolidation')
    expect(container.textContent).toContain('子任务 2')
    // 仅分配阶段，不应出现「已解决」外部求解行
    expect(container.textContent).not.toContain('已解决')
  })

  it('阶段2 done: running=false 显示完成概览计数 + 外部已解决', () => {
    const { container } = render(() => <HarnessReportCard running={false} report={makeReport()} onClose={() => {}} />)
    expect(container.textContent).not.toContain('执行中')
    expect(container.textContent).toContain('拆解 2')
    expect(container.textContent).toContain('内置执行 1/1')
    expect(container.textContent).toContain('外部求解 1/1')
    expect(container.textContent).toContain('已解决')
  })

  it('未解缺口高亮', () => {
    const r = makeReport({ external_closures: [{ solved: false, solution: '', knowledge_acquired: false }], external_gaps: ['github_research [能力网无内置]'] })
    const { container } = render(() => <HarnessReportCard running={false} report={r} onClose={() => {}} />)
    expect(container.textContent).toContain('未解缺口 1')
    expect(container.textContent).toContain('未解决')
  })
})
