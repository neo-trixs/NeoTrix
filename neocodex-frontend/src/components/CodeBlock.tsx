// ══════════════════════════════════════════════════════════════════════════
//  CodeBlock — 代码块组件
//  语法高亮 / 语言标签 / 复制按钮 / 行号
// ══════════════════════════════════════════════════════════════════════════
import { createSignal, Show } from 'solid-js'
import { clsx } from 'clsx'

interface CodeBlockProps {
  language?: string
  code: string
  filename?: string
  showLineNumbers?: boolean
}

export function CodeBlock(props: CodeBlockProps) {
  const [copied, setCopied] = createSignal(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(props.code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const lines = () => props.code.split('\n')
  const lang = () => props.language ?? detectLanguage(props.code)

  return (
    <div class="code-block">
      {/* 头部：语言 + 文件名 + 复制 */}
      <div class="code-header">
        <div class="code-lang">
          {props.filename && <span class="code-filename">{props.filename}</span>}
          <Show when={!props.filename && lang()}>
            <span class="code-lang-tag">{lang()}</span>
          </Show>
        </div>
        <button class="code-copy-btn" onClick={handleCopy}>
          {copied() ? '✓ 已复制' : '📋 复制'}
        </button>
      </div>

      {/* 代码内容 */}
      <div class="code-content">
        <Show when={props.showLineNumbers}>
          <div class="code-line-numbers">
            {lines().map((_, i) => (
              <span class="code-line-num">{i + 1}</span>
            ))}
          </div>
        </Show>
        <pre class="code-pre">
          <code>{props.code}</code>
        </pre>
      </div>
    </div>
  )
}

/** 简单语言检测 */
function detectLanguage(code: string): string {
  if (code.includes('fn ') && code.includes('let mut')) return 'rust'
  if (code.includes('function ') && (code.includes('=>') || code.includes('const '))) return 'typescript'
  if (code.includes('def ') && code.includes('import ')) return 'python'
  if (code.includes('class ') && code.includes('public ')) return 'java'
  if (code.includes('#include')) return 'cpp'
  if (code.includes('SELECT ') || code.includes('FROM ')) return 'sql'
  if (code.includes('{') && code.includes('"') && code.includes(':')) return 'json'
  if (code.includes('---') || code.includes('apiVersion:')) return 'yaml'
  return 'text'
}
