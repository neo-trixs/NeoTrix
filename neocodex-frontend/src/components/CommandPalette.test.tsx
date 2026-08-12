import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@solidjs/testing-library'
import { CommandPalette, type PaletteCommand } from './CommandPalette'

const commands: PaletteCommand[] = [
  { id: 'new', label: '新建对话', desc: '开启一段新对话', keywords: ['new', '新建'], run: () => {} },
  { id: 'compact', label: '压缩会话', desc: '精简上下文', keywords: ['compact', '压缩'], run: () => {} },
  { id: 'mode', label: '切换权限模式', desc: '自动/手动', keywords: ['mode', '权限'], run: () => {} },
]

describe('CommandPalette', () => {
  it('renders nothing when closed', () => {
    render(() => <CommandPalette open={false} commands={commands} onClose={() => {}} />)
    expect(screen.queryByRole('dialog')).toBeNull()
  })

  it('renders all commands when open', () => {
    render(() => <CommandPalette open commands={commands} onClose={() => {}} />)
    expect(screen.getByText('新建对话')).toBeTruthy()
    expect(screen.getByText('压缩会话')).toBeTruthy()
    expect(screen.getByText('切换权限模式')).toBeTruthy()
  })

  it('filters by keyboard input and runs selected command', async () => {
    let ran = ''
    const onClose = () => {}
    const { unmount } = render(() => (
      <CommandPalette
        open
        commands={commands}
        onClose={onClose}
      />
    ))
    const input = screen.getByLabelText('搜索命令')
    await fireEvent.input(input, { target: { value: 'new' } })
    expect(screen.getByText('新建对话')).toBeTruthy()
    expect(screen.queryByText('切换权限模式')).toBeNull()

    // 直接点击选项执行
    ran = ''
    const changed = commands.map((c) => (c.id === 'new' ? { ...c, run: () => (ran = 'new') } : c))
    unmount()
    render(() => <CommandPalette open commands={changed} onClose={onClose} />)
    await fireEvent.input(screen.getByLabelText('搜索命令'), { target: { value: 'new' } })
    await fireEvent.click(screen.getByText('新建对话'))
    expect(ran).toBe('new')
  })

  it('closes on backdrop click', async () => {
    let closed = false
    render(() => <CommandPalette open commands={commands} onClose={() => (closed = true)} />)
    const backdrop = document.querySelector('.cmd-palette-backdrop')
    expect(backdrop).toBeTruthy()
    await fireEvent.click(backdrop!)
    expect(closed).toBe(true)
  })
})