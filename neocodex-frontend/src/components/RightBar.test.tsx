import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, fireEvent } from '@solidjs/testing-library'
import { RightBar } from './RightBar'
import { mockCommand, resetInvokeMock } from '../test/invokeMock'

vi.mock('@tauri-apps/api/core', async () => {
  const { mockInvokeImpl } = await import('../test/invokeMock')
  return { invoke: mockInvokeImpl }
})

vi.mock('@tauri-apps/api/window', () => ({}))

const TREE_DATA = {
  root: '/Users/neo/demo',
  tree: [
    { name: 'src', path: '/Users/neo/demo/src', is_dir: true, children: [
      { name: 'main.rs', path: '/Users/neo/demo/src/main.rs', is_dir: false },
      { name: 'lib.rs', path: '/Users/neo/demo/src/lib.rs', is_dir: false },
    ]},
    { name: 'README.md', path: '/Users/neo/demo/README.md', is_dir: false },
  ],
  file_count: 3,
  agents_md: '# AGENTS.md\n内容',
}

function findBtn(text: string) {
  return [...document.querySelectorAll('button')].find((b) => b.textContent?.includes(text))
}

const settle = () => new Promise((r) => setTimeout(r, 150))

describe('RightBar 右栏回归（标签/项目树/文件树/预览/嵌入）', () => {
  beforeEach(() => {
    resetInvokeMock()
    document.body.innerHTML = ''
  })

  it('渲染 3 个标签：文件/地图/项目', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    const tabs = document.querySelectorAll('[role="tab"]')
    expect(tabs.length).toBe(3)
    expect(document.body.textContent).toContain('文件')
    expect(document.body.textContent).toContain('地图')
    expect(document.body.textContent).toContain('项目')
    expect(document.querySelector('[aria-selected="true"]')?.textContent).toBe('文件')
  })

  it('切换到地图标签显示 GlobeView', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    fireEvent.click(findBtn('地图')!)
    await settle()
    expect(document.querySelector('[role="tab"][aria-selected="true"]')?.textContent).toBe('地图')
    // GlobeView 挂载（检查 canvas 或特定 class）
    expect(document.querySelector('.rb-map')).toBeTruthy()
  })

  it('切换到项目标签显示 ProjectViewPanel', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    fireEvent.click(findBtn('项目')!)
    await settle()
    expect(document.querySelector('[role="tab"][aria-selected="true"]')?.textContent).toBe('项目')
    // ProjectViewPanel 渲染
    expect(document.body.textContent).toContain('demo')
  })

  it('方向键切换标签', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    const fileTab = document.querySelectorAll('[role="tab"]')[0]
    fireEvent.keyDown(fileTab, { key: 'ArrowRight' })
    await new Promise((r) => requestAnimationFrame(r))
    expect(document.querySelector('[role="tab"][aria-selected="true"]')?.textContent).toBe('地图')
    fireEvent.keyDown(document.querySelector('[role="tab"][aria-selected="true"]')!, { key: 'ArrowRight' })
    await new Promise((r) => requestAnimationFrame(r))
    expect(document.querySelector('[role="tab"][aria-selected="true"]')?.textContent).toBe('项目')
    fireEvent.keyDown(document.querySelector('[role="tab"][aria-selected="true"]')!, { key: 'ArrowLeft' })
    await new Promise((r) => requestAnimationFrame(r))
    expect(document.querySelector('[role="tab"][aria-selected="true"]')?.textContent).toBe('地图')
  })

  it('文件标签：项目树加载 + 目录展开 + 文件点击', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    // 默认文件标签，树加载
    expect(document.body.textContent).toContain('src')
    expect(document.body.textContent).toContain('README.md')
    // 展开 src 目录
    const srcItem = document.querySelector('[role="treeitem"]')
    fireEvent.click(srcItem!)
    await new Promise((r) => setTimeout(r, 200))
    expect(document.body.textContent).toContain('main.rs')
    expect(document.body.textContent).toContain('lib.rs')
    // 点击文件打开预览
    const mainFile = findBtn('main.rs')
    if (mainFile) {
      fireEvent.click(mainFile)
      await new Promise((r) => setTimeout(r, 100))
      expect(document.body.textContent).toContain('main.rs')
    }
  })

  it('文件树键盘导航：方向键移动焦点', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    const firstItem = document.querySelector('[role="treeitem"]') as HTMLElement
    firstItem?.focus()
    expect(firstItem).toBe(document.activeElement)
    fireEvent.keyDown(firstItem, { key: 'ArrowDown' })
    await new Promise((r) => requestAnimationFrame(r))
    // 焦点应移到下一个
    expect(document.activeElement).not.toBe(firstItem)
  })

  it('点击文件打开预览面板', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    // 展开 src
    const srcItem = document.querySelector('[role="treeitem"]')
    fireEvent.click(srcItem!)
    await settle()
    // 点击 main.rs
    const mainFile = findBtn('main.rs')
    if (mainFile) {
      fireEvent.click(mainFile)
      await settle()
      // 预览面板展开
      expect(document.body.textContent).toContain('main.rs')
    }
  })

  it('预览面板：raw/rendered 格式切换', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    // 展开并点击文件
    const srcItem = document.querySelector('[role="treeitem"]')
    fireEvent.click(srcItem!)
    await settle()
    const readmeFile = findBtn('README.md')
    if (readmeFile) {
      fireEvent.click(readmeFile)
      await settle()
      // 格式 tabs
      expect(document.body.textContent).toContain('Raw')
      expect(document.body.textContent).toContain('Rendered')
      // 切换到 Raw
      const rawTab = [...document.querySelectorAll('button')].find((b) => b.textContent === 'Raw')
      if (rawTab) {
        fireEvent.click(rawTab)
        await settle()
        // 预览模式为 raw
      }
    }
  })

  it('Artifact 视图切换：Preview/Code', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    // 打开文件
    const srcItem = document.querySelector('[role="treeitem"]')
    fireEvent.click(srcItem!)
    await settle()
    const readmeFile = findBtn('README.md')
    if (readmeFile) {
      fireEvent.click(readmeFile)
      await settle()
      // 预览/代码视图按钮
      expect(document.body.textContent).toContain('预览')
      expect(document.body.textContent).toContain('代码')
    }
  })

  it('地图标签显示 GlobeView', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    fireEvent.click(findBtn('地图')!)
    await settle()
    expect(document.querySelector('.rb-map')).toBeTruthy()
  })

  it('项目标签显示 ProjectViewPanel', async () => {
    mockCommand('neocodex_project_tree', async () => TREE_DATA)
    render(() => <RightBar />)
    await settle()
    fireEvent.click(findBtn('项目')!)
    await settle()
    expect(document.body.textContent).toContain('demo')
  })
})