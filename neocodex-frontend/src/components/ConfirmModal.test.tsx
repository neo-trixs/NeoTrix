import { describe, it, expect, vi } from 'vitest'
import { cleanup, render, fireEvent } from '@solidjs/testing-library'
import { afterEach } from 'vitest'
import { ConfirmModal, type ModalReq } from './ConfirmModal'

const req: ModalReq = { title: '确认删除', message: '此操作不可撤销', danger: true }

afterEach(() => cleanup())

describe('ConfirmModal 确认/输入模态回归（替换原生 confirm/prompt）', () => {
  it('纯确认模式：渲染标题/消息/两键', () => {
    render(() => <ConfirmModal req={req} onConfirm={() => {}} onClose={() => {}} />)
    expect(document.body.textContent).toContain('确认删除')
    expect(document.body.textContent).toContain('此操作不可撤销')
    expect(document.body.textContent).toContain('取消')
    expect(document.body.textContent).toContain('确定')
  })

  it('danger 模式确认键为红色', () => {
    render(() => <ConfirmModal req={req} onConfirm={() => {}} onClose={() => {}} />)
    const confirmBtn = [...document.querySelectorAll('button')].find((b) =>
      b.textContent === '确定'
    )
    expect(confirmBtn!.className).toContain('bg-red-600')
  })

  it('普通模式确认键为品牌色', () => {
    render(() => (
      <ConfirmModal
        req={{ title: '确认', danger: false }}
        onConfirm={() => {}}
        onClose={() => {}}
      />
    ))
    const confirmBtn = [...document.querySelectorAll('button')].find((b) =>
      b.textContent === '确定'
    )
    expect(confirmBtn!.className).toContain('bg-nt-io-600')
  })

  it('确认键回传输入值', () => {
    const onConfirm = vi.fn()
    render(() => (
      <ConfirmModal
        req={{ title: '重命名', inputLabel: '新名称', initialValue: 'old' }}
        onConfirm={onConfirm}
        onClose={() => {}}
      />
    ))
    const input = document.querySelector('input') as HTMLInputElement
    fireEvent.input(input, { target: { value: 'new-name' } })
    const confirmBtn = [...document.querySelectorAll('button')].find((b) =>
      b.textContent === '确定'
    )
    fireEvent.click(confirmBtn!)
    expect(onConfirm).toHaveBeenCalledWith('new-name')
  })

  it('无输入模式确认回传空字符串（inputVal 初值）', () => {
    const onConfirm = vi.fn()
    render(() => <ConfirmModal req={req} onConfirm={onConfirm} onClose={() => {}} />)
    const confirmBtn = [...document.querySelectorAll('button')].find((b) =>
      b.textContent === '确定'
    )
    fireEvent.click(confirmBtn!)
    expect(onConfirm).toHaveBeenCalledTimes(1)
    expect(onConfirm.mock.calls[0][0]).toBe('')
  })

  it('输入框 Enter 触发确认', () => {
    const onConfirm = vi.fn()
    render(() => (
      <ConfirmModal
        req={{ title: '重命名', inputLabel: '新名称', initialValue: 'old' }}
        onConfirm={onConfirm}
        onClose={() => {}}
      />
    ))
    const input = document.querySelector('input') as HTMLInputElement
    fireEvent.input(input, { target: { value: 'renamed' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(onConfirm).toHaveBeenCalledWith('renamed')
  })

  it('Esc 关闭且阻止事件外泄（stopImmediatePropagation）', async () => {
    const onClose = vi.fn()
    // 外层容器也监听 Esc（模拟双层弹窗场景），确认框必须独占
    const outer = vi.fn()
    window.addEventListener('keydown', outer)
    render(() => <ConfirmModal req={req} onConfirm={() => {}} onClose={onClose} />)
    // Solid createEffect 在 render 后 flush：等一帧让监听器挂载
    await new Promise((r) => setTimeout(r, 0))
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(onClose).toHaveBeenCalled()
    expect(outer).not.toHaveBeenCalled()
    window.removeEventListener('keydown', outer)
  })

  it('req 为 null 时渲染空（隐藏）', () => {
    render(() => <ConfirmModal req={null} onConfirm={() => {}} onClose={() => {}} />)
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('自定义按钮文案', () => {
    render(() => (
      <ConfirmModal
        req={{ title: 'T', confirmLabel: '保存', cancelLabel: '返回' }}
        onConfirm={() => {}}
        onClose={() => {}}
      />
    ))
    expect(document.body.textContent).toContain('保存')
    expect(document.body.textContent).toContain('返回')
  })

  it('inputLabel 时聚焦输入框（rAF 后）', async () => {
    render(() => (
      <ConfirmModal
        req={{ title: '重命名', inputLabel: '新名称', initialValue: 'x' }}
        onConfirm={() => {}}
        onClose={() => {}}
      />
    ))
    await new Promise((r) => requestAnimationFrame(r))
    const input = document.querySelector('input') as HTMLInputElement
    expect(document.activeElement).toBe(input)
  })
})