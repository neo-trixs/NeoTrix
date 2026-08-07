import { createSignal, For, Show } from 'solid-js'
import { clsx } from 'clsx'

/* ════════════════════════════════════════════
   CodeView — 代码视图（设计 v2）
   左栏：文件树（cd-tree）
   右栏：编辑器（tabs + toolbar + 语法高亮 viewer）
   ════════════════════════════════════════════ */

interface Sample {
  name: string
  lang: string
  code: string
}

const SAMPLES: Sample[] = [
  {
    name: 'main.rs', lang: 'Rust',
    code: 'fn main() {\n    println!("Hello, NeoTrix!");\n    let engine = ReasoningEngine::new();\n    engine.run();\n}',
  },
  {
    name: 'lib.rs', lang: 'Rust',
    code: 'pub fn compute(input: &str) -> Result<f64> {\n    let parsed: f64 = input.parse()?;\n    Ok(parsed * std::f64::consts::PI)\n}',
  },
  {
    name: 'config.rs', lang: 'TOML',
    code: '[package]\nname = "neotrix"\nversion = "0.19.0"\nedition = "2021"\n\n[dependencies]\nserde = { version = "1", features = ["derive"] }',
  },
]

const RUST_KEYWORDS = ['pub', 'struct', 'impl', 'fn', 'let', 'mut', 'const', 'Self', 'for', 'in', 'return', 'if', 'else', 'match', 'use', 'mod', 'trait', 'enum', 'type', 'where', 'as', 'async', 'await', 'move']

function escHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

/* 简单 Rust 语法着色 → 安全 HTML */
function highlight(code: string): string {
  let html = escHtml(code)
  RUST_KEYWORDS.forEach((k) => {
    html = html.replace(new RegExp(`\\b${k}\\b`, 'g'), `<span class="kw">${k}</span>`)
  })
  html = html.replace(/\/\/.*/g, (m) => `<span class="cm">${escHtml(m)}</span>`)
  html = html.replace(/"([^"]*)"/g, (m) => `<span class="hl">${m}</span>`)
  html = html.replace(/\b[A-Z]\w+(?=\s*(?:[({<]|::))/g, (m) => `<span class="fn">${m}</span>`)
  return html
}

interface FileTreeItem {
  name: string
  type: 'dir' | 'file'
  open?: boolean
  children?: FileTreeItem[]
}

const TREE: FileTreeItem[] = [
  {
    name: 'src', type: 'dir', open: true, children: [
      { name: 'main.rs', type: 'file' },
      { name: 'lib.rs', type: 'file' },
      { name: 'engine_core.rs', type: 'file' },
      { name: 'config.rs', type: 'file' },
    ],
  },
  {
    name: 'tests', type: 'dir', open: false, children: [
      { name: 'test_engine.rs', type: 'file' },
    ],
  },
  { name: 'Cargo.toml', type: 'file' },
]

function Tree(props: {
  items: FileTreeItem[]
  depth?: number
  active: string | null
  onOpen: (name: string) => void
  onToggle: (item: FileTreeItem) => void
}) {
  return (
    <For each={props.items}>
      {(item) => (
        <>
          {item.type === 'dir' ? (
            <>
              <div
                class="ft-item"
                style={{ 'padding-left': `${(props.depth ?? 0) * 14 + 4}px` }}
                onClick={() => props.onToggle(item)}
              >
                <svg class={clsx('chev', item.open && 'open')} viewBox="0 0 9 9">
                  <line x1="3" y1="2.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                  <line x1="3" y1="6.5" x2="6" y2="4.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                </svg>
                <svg class="fic" viewBox="0 0 14 14">
                  <path d="M1.5 4.5h3.5l1-1.5h6a1 1 0 011 1v6a1 1 0 01-1 1h-10a1 1 0 01-1-1v-5.5z" stroke="currentColor" stroke-width="1" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
                {item.name}
              </div>
              <div class={clsx('ft-children', item.open && 'open')}>
                <Show when={item.open}>
                  <Tree
                    items={item.children ?? []}
                    depth={(props.depth ?? 0) + 1}
                    active={props.active}
                    onOpen={props.onOpen}
                    onToggle={props.onToggle}
                  />
                </Show>
              </div>
            </>
          ) : (
            <div
              class={clsx('ft-item ft-file', props.active === item.name && 'ft-active')}
              style={{ 'padding-left': `${(props.depth ?? 0) * 14 + 4}px` }}
              onClick={() => props.onOpen(item.name)}
            >
              <svg class="fic" viewBox="0 0 14 14">
                <path d="M2 1.5h10v11H2z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
                <line x1="4.5" y1="4.5" x2="9.5" y2="4.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
              </svg>
              {item.name}
            </div>
          )}
        </>
      )}
    </For>
  )
}

export function CodeView() {
  const [tree, setTree] = createSignal<FileTreeItem[]>(TREE)
  const [activeTab, setActiveTab] = createSignal(0)
  const [openTabs, setOpenTabs] = createSignal<number[]>([0])

  const sample = () => SAMPLES[activeTab()] ?? SAMPLES[0]

  const openFile = (name: string) => {
    const idx = SAMPLES.findIndex((s) => s.name === name)
    if (idx === -1) return
    if (!openTabs().includes(idx)) {
      setOpenTabs([...openTabs(), idx])
    }
    setActiveTab(idx)
  }

  const toggleDir = (item: FileTreeItem) => {
    item.open = !item.open
    setTree([...tree()])
  }

  const lineCount = () => sample().code.split('\n').length

  return (
    <div class="vw-code">
      <div class="cd-layout">
        {/* 左栏：文件树 */}
        <div class="cd-tree">
          <div class="cd-thead">
            <span>文件</span>
            <button class="cd-tbtn" title="刷新文件树" aria-label="刷新文件树">
              <svg viewBox="0 0 14 14">
                <polyline points="2,7 4,9 7,4" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
                <path d="M12 7a5 5 0 11-5-5 5 5 0 014.5 3" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linecap="round" />
              </svg>
            </button>
          </div>
          <div class="flex-1 overflow-y-auto p-2">
            <Tree
              items={tree()}
              active={sample().name}
              onOpen={openFile}
              onToggle={toggleDir}
            />
          </div>
        </div>

        {/* 右栏：编辑器 */}
        <div class="cd-editor">
          <div class="cd-tabs">
            <For each={openTabs()}>
              {(i) => (
                <div
                  class={clsx('cd-tab', i === activeTab() && 'on')}
                  onClick={() => setActiveTab(i)}
                >
                  {SAMPLES[i].name}
                </div>
              )}
            </For>
          </div>
          <div class="cd-toolbar">
            <button class="cd-tb-btn" title="格式化" aria-label="格式化">
              <svg viewBox="0 0 14 14">
                <line x1="2" y1="3" x2="12" y2="3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                <line x1="4" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
                <line x1="2" y1="11" x2="12" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              </svg>
            </button>
            <button class="cd-tb-btn" title="保存" aria-label="保存">
              <svg viewBox="0 0 14 14">
                <path d="M3 1.5h8l1.5 1.5v9a1 1 0 01-1 1h-9a1 1 0 01-1-1V3z" stroke="currentColor" stroke-width="1.2" fill="none" stroke-linejoin="round" />
                <line x1="4" y1="1.5" x2="4" y2="5" stroke="currentColor" stroke-width="1.2" />
                <circle cx="7" cy="9" r="1.5" stroke="currentColor" stroke-width="1.2" fill="none" />
              </svg>
            </button>
            <button class="cd-tb-btn" title="复制" aria-label="复制">
              <svg viewBox="0 0 14 14">
                <rect x="3.5" y="2.5" width="8" height="10" rx="1" stroke="currentColor" stroke-width="1.2" fill="none" />
                <path d="M2.5 10.5h-1a1 1 0 01-1-1v-7a1 1 0 011-1h6a1 1 0 011 1v1" stroke="currentColor" stroke-width="1.2" fill="none" />
              </svg>
            </button>
            <span class="cd-lang">{sample().lang}</span>
          </div>
          <div class="cd-view">
            <span class="cd-ln">
              <For each={Array.from({ length: lineCount() }, (_, i) => i + 1)}>
                {(n) => <span>{n}<br /></span>}
              </For>
            </span>
            <code innerHTML={highlight(sample().code)} />
          </div>
        </div>
      </div>
    </div>
  )
}
