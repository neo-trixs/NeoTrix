import { describe, it, expect, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { SlashMenu, type SlashCommandDef } from './SlashMenu'

const cmds: SlashCommandDef[] = [
  { id: 'clear', label: '清除会话', desc: '清空当前对话', keywords: ['clear'] },
  { id: 'plan', label: '生成计划', desc: '拆解任务为计划', keywords: ['plan'] },
  { id: 'help', label: '帮助', desc: '查看可用命令', keywords: ['help'] },
]

describe('SlashMenu 渲染与过滤（纯逻辑回归）', () => {
  it('空 query 显示全部命令', () => {
    render(() => (
      <SlashMenu query="" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    const items = document.querySelectorAll('[role="option"]')
    expect(items.length).toBe(3)
  })

  it('query 匹配 keyword 过滤命令', () => {
    render(() => (
      <SlashMenu query="cle" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    const items = document.querySelectorAll('[role="option"]')
    expect(items.length).toBe(1)
    expect(items[0].textContent).toContain('清除会话')
  })

  it('query 匹配 label（中文）过滤命令', () => {
    render(() => (
      <SlashMenu query="计划" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    const items = document.querySelectorAll('[role="option"]')
    expect(items.length).toBe(1)
    expect(items[0].textContent).toContain('生成计划')
  })

  it('大小写不敏感匹配', () => {
    render(() => (
      <SlashMenu query="PLAN" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    expect(document.querySelectorAll('[role="option"]').length).toBe(1)
  })

  it('无匹配时隐藏整个菜单', () => {
    render(() => (
      <SlashMenu query="zzz" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    expect(document.querySelector('[role="listbox"]')).toBeNull()
  })

  it('选中项标记 aria-selected', () => {
    render(() => (
      <SlashMenu query="" commands={cmds} selectedIdx={1} onSelect={() => {}} />
    ))
    const items = document.querySelectorAll('[role="option"]')
    expect(items[1].getAttribute('aria-selected')).toBe('true')
    expect(items[0].getAttribute('aria-selected')).toBe('false')
  })

  it('点击命令触发 onSelect', () => {
    const onSelect = vi.fn()
    render(() => (
      <SlashMenu query="" commands={cmds} selectedIdx={0} onSelect={onSelect} />
    ))
    fireEvent.click(document.querySelectorAll('[role="option"]')[2])
    expect(onSelect).toHaveBeenCalledWith(cmds[2])
  })

  it('悬停命令触发 onHover（带索引）', () => {
    const onHover = vi.fn()
    render(() => (
      <SlashMenu query="" commands={cmds} selectedIdx={0} onSelect={() => {}} onHover={onHover} />
    ))
    fireEvent.mouseEnter(document.querySelectorAll('[role="option"]')[1])
    expect(onHover).toHaveBeenCalledWith(1)
  })

  it('不提供 onHover 时悬停不抛错', () => {
    render(() => (
      <SlashMenu query="" commands={cmds} selectedIdx={0} onSelect={() => {}} />
    ))
    fireEvent.mouseEnter(document.querySelectorAll('[role="option"]')[0])
    expect(document.body.textContent).toContain('清除会话')
  })
})