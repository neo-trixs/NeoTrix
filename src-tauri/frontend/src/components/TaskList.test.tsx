import { describe, it, expect } from 'vitest'
import { parseTodoItems, TaskList } from './TaskList'
import { render } from '@solidjs/testing-library'

describe('TaskList parseTodoItems', () => {
  it('解析 - [ ] 待办与 - [x] 已完成', () => {
    const items = parseTodoItems('- [ ] 修复 bug\n- [x] 写测试\n- [ ] 发布')
    expect(items).toHaveLength(3)
    expect(items[0]).toMatchObject({ text: '修复 bug', done: false })
    expect(items[1].done).toBe(true)
  })

  it('忽略非 checklist 列表与空内容', () => {
    expect(parseTodoItems('- 普通列表项')).toHaveLength(0)
    expect(parseTodoItems('')).toHaveLength(0)
    expect(parseTodoItems('  无列表')).toHaveLength(0)
  })

  it('支持缩进与 * 前缀', () => {
    const items = parseTodoItems('  * [ ] 缩进任务\n\n* [X] 大写已完成')
    expect(items).toHaveLength(2)
    expect(items[0].text).toBe('缩进任务')
    expect(items[1].done).toBe(true)
  })
})

describe('TaskList render', () => {
  it('无 checklist 时不渲染', () => {
    const { container } = render(() => <TaskList content="普通回复" messageId="m1" />)
    expect(container.querySelector('[class*="任务清单"]')).toBeNull()
  })

  it('渲染任务清单与进度', () => {
    const { container } = render(() => (
      <TaskList content={'- [ ] 做 A\n- [x] 做 B'} messageId="m2" />
    ))
    expect(container.textContent).toContain('任务清单')
    expect(container.textContent).toContain('做 A')
    expect(container.textContent).toContain('做 B')
    // 进度: JSX 相邻文本节点可能被空白折叠，单独断言数字存在
    expect(container.textContent).toContain('1')
    expect(container.textContent).toContain('2')
  })
})