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
