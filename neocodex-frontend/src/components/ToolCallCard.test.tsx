import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@solidjs/testing-library'
import { ToolCallCard } from './ToolCallCard'
import type { ToolCallRecord } from '../stores/chat'

const successCall: ToolCallRecord = {
  id: 't1',
  name: 'read_file',
  args: '{"path":"main.rs"}',
  result: 'pub fn main() {}',
  duration_ms: 120,
  success: true,
}

const failedCall: ToolCallRecord = {
  id: 't2',
  name: 'grep',
  args: '{"pattern":"foo"}',
  result: 'no matches found',
  duration_ms: 3000,
  success: false,
}

/* 0 时长完成事件：审批拒绝/exit_code=0 等瞬时完成，后端仍发完成事件。
   此前按 duration_ms===0 判"执行中"，会永久显示转圈。 */
const zeroDurationSuccess: ToolCallRecord = {
  id: 't3',
  name: 'shell',
  args: '{"cmd":"true"}',
  result: '',
  duration_ms: 0,
  success: true,
}

const zeroDurationFailure: ToolCallRecord = {
  id: 't4',
  name: 'write_file',
  args: '{"path":"x"}',
  result: 'TOOL_ERROR: 审批被拒绝',
  duration_ms: 0,
  success: false,
}

describe('ToolCallCard', () => {
  beforeEach(() => {
    // jsdom lacks clipboard by default
    Object.assign(navigator, {
      clipboard: { writeText: () => Promise.resolve() },
    })
  })

  it('shows tool name and success check', () => {
    render(() => <ToolCallCard call={successCall} />)
    expect(screen.getByText('read_file')).toBeTruthy()
    expect(screen.getByText('120ms')).toBeTruthy()
  })

  it('shows failure state with red styling and seconds duration', () => {
    render(() => <ToolCallCard call={failedCall} />)
    expect(screen.getByText('grep')).toBeTruthy()
    expect(screen.getByText('3.0s')).toBeTruthy()
  })

  it('treats 0-duration success as completed (not running)', () => {
    render(() => <ToolCallCard call={zeroDurationSuccess} />)
    expect(screen.queryByText('执行中…')).toBeNull()
    expect(screen.getByRole('status', { name: '工具调用成功' })).toBeTruthy()
  })

  it('treats 0-duration failure as completed and surfaces reason', () => {
    render(() => <ToolCallCard call={zeroDurationFailure} />)
    expect(screen.queryByText('执行中…')).toBeNull()
    expect(screen.getByRole('status', { name: '工具调用失败' })).toBeTruthy()
    expect(screen.getByText('TOOL_ERROR: 审批被拒绝')).toBeTruthy()
  })

  it('reveals args and result after expanding', async () => {
    render(() => <ToolCallCard call={successCall} />)
    // args/result hidden initially
    expect(screen.queryByText('{"path":"main.rs"}')).toBeNull()
    const toggle = screen.getByRole('button')
    await fireEvent.click(toggle)
    expect(screen.getByText('{"path":"main.rs"}')).toBeTruthy()
    expect(screen.getByText('pub fn main() {}')).toBeTruthy()
  })
})
