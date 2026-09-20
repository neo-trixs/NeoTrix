import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { ProjectView } from './ProjectView'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

const TREE = {
  root: '/Users/neo/demo',
  file_count: 3,
  agents_md: '# AGENTS 指南\n遵守规范',
  tree: [
    {
      name: 'src', path: '/Users/neo/demo/src', is_dir: true,
      children: [
        { name: 'main.rs', path: '/Users/neo/demo/src/main.rs', is_dir: false },
      ],
    },
    { name: 'README.md', path: '/Users/neo/demo/README.md', is_dir: false },
  ],
}

const settle = () => new Promise((r) => setTimeout(r, 120))

describe('ProjectView 项目视图回归（目录树/tab/文件打开）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('open=false 不渲染', () => {
    render(() => <ProjectView open={false} onClose={() => {}} />)
    expect(document.body.textContent).toBe('')
  })

  it('加载后显示根目录名与文件数', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    expect(document.querySelector('.panel-title')?.textContent).toBe('demo')
    expect(document.body.textContent).toContain('3 文件')
  })

  it('首层目录自动展开，子文件可见', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('main.rs')
    expect(document.body.textContent).toContain('README.md')
  })

  it('点击已展开目录折叠子项', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    const srcBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('src'))!
    fireEvent.click(srcBtn)
    await settle()
    expect(document.body.textContent).not.toContain('main.rs')
    // 再点展开
    fireEvent.click(srcBtn)
    await settle()
    expect(document.body.textContent).toContain('main.rs')
  })

  it('点击文件回调 onOpenFile 传绝对路径', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    const onOpenFile = vi.fn()
    render(() => <ProjectView open onClose={() => {}} onOpenFile={onOpenFile} />)
    await settle()
    const fileBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('main.rs'))!
    fireEvent.click(fileBtn)
    expect(onOpenFile).toHaveBeenCalledWith('/Users/neo/demo/src/main.rs')
  })

  it('切换到 AGENTS.md tab 显示内容', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    fireEvent.click([...document.querySelectorAll('[role="tab"]')].find((t) => t.textContent === 'AGENTS.md')!)
    await settle()
    expect(document.body.textContent).toContain('# AGENTS 指南')
    const tabs = document.querySelectorAll('[role="tab"]')
    expect(tabs[1].getAttribute('aria-selected')).toBe('true')
    expect(tabs[0].getAttribute('aria-selected')).toBe('false')
  })

  it('方向键切换 tab', async () => {
    mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    const treeTab = document.querySelectorAll('[role="tab"]')[0]
    fireEvent.keyDown(treeTab, { key: 'ArrowRight' })
    await settle()
    expect(document.querySelectorAll('[role="tab"]')[1].getAttribute('aria-selected')).toBe('true')
  })

  it('刷新按钮重新加载', async () => {
    const stub = mockCommand('neocodex_project_tree', async () => TREE)
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    fireEvent.click(document.querySelector('[aria-label="刷新"]')!)
    await settle()
    expect(stub.calledTimes()).toBe(2)
  })

  it('加载失败显示错误', async () => {
    mockCommand('neocodex_project_tree', async () => {
      throw new Error('tree failed')
    })
    render(() => <ProjectView open onClose={() => {}} />)
    await settle()
    expect(document.body.textContent).toContain('tree failed')
  })
})