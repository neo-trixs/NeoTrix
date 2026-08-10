import { createSignal, For, Show, type JSX } from 'solid-js'
import { Copy, Check } from 'lucide-solid'
import { clsx } from 'clsx'

/**
 * Markdown — 轻量 markdown 渲染器（对标 Claude Code 完整渲染）。
 * 无第三方依赖：行级块解析 + 行内 token 解析为 SolidJS 元素。
 * 支持：标题(#~####) / 粗体 / 斜体 / 行内代码 / 代码块(```lang) /
 *       无序列表 / 有序列表 / 引用 / 分隔线 / 链接 / 表格。
 */

type Block =
  | { type: 'paragraph'; text: string }
  | { type: 'heading'; level: number; text: string }
  | { type: 'code'; language: string; code: string }
  | { type: 'hr' }
  | { type: 'blockquote'; lines: string[] }
  | { type: 'list'; ordered: boolean; items: string[] }
  | { type: 'table'; header: string[]; rows: string[][] }

/* ---------- 行内解析 ---------- */

const INLINE_RE = /(`[^`]+`)|(\*\*[^*]+?\*\*)|(\*[^*\n]+?\*)|(\[([^\]]+)\]\(([^)\s]+)(?:\s+"[^"]*")?\))/g

function renderInline(text: string): JSX.Element[] {
  const nodes: JSX.Element[] = []
  INLINE_RE.lastIndex = 0
  let lastIndex = 0
  let m: RegExpExecArray | null
  while ((m = INLINE_RE.exec(text)) !== null) {
    if (m.index > lastIndex) nodes.push(text.slice(lastIndex, m.index))
    if (m[1] !== undefined) {
      nodes.push(<code class="inline-code">{m[1].slice(1, -1)}</code>)
    } else if (m[2] !== undefined) {
      nodes.push(<strong class="font-semibold text-text-primary">{renderInline(m[2].slice(2, -2))}</strong>)
    } else if (m[3] !== undefined) {
      nodes.push(<em class="italic">{renderInline(m[3].slice(1, -1))}</em>)
    } else if (m[4] !== undefined) {
      nodes.push(
        <a
          class="text-nt-io-700 underline decoration-nt-io-500/50 underline-offset-2 hover:text-nt-io-800 transition-colors"
          href={m[6]}
          target="_blank"
          rel="noopener noreferrer"
        >
          {renderInline(m[5])}
        </a>,
      )
    }
    lastIndex = INLINE_RE.lastIndex
  }
  if (lastIndex < text.length) nodes.push(text.slice(lastIndex))
  return nodes
}

/* ---------- 块级解析 ---------- */

function parseBlocks(content: string): Block[] {
  const lines = content.split('\n')
  const blocks: Block[] = []
  let i = 0

  while (i < lines.length) {
    const line = lines[i]

    // 围栏代码块 ```lang ... ```
    const fence = line.match(/^```(\w*)\s*$/)
    if (fence) {
      const language = fence[1]
      const codeLines: string[] = []
      i++
      while (i < lines.length && !/^```\s*$/.test(lines[i])) {
        codeLines.push(lines[i])
        i++
      }
      i++ // 跳过闭合围栏
      blocks.push({ type: 'code', language, code: codeLines.join('\n') })
      continue
    }

    if (line.trim() === '') {
      i++
      continue
    }

    // 标题 # ~ ####
    const heading = line.match(/^(#{1,4})\s+(.*)$/)
    if (heading) {
      blocks.push({ type: 'heading', level: heading[1].length, text: heading[2] })
      i++
      continue
    }

    // 分隔线 --- / *** / ___
    if (/^(-{3,}|\*{3,}|_{3,})\s*$/.test(line.trim())) {
      blocks.push({ type: 'hr' })
      i++
      continue
    }

    // 引用 > line（连续引用合并）
    if (line.trim().startsWith('>')) {
      const quoteLines: string[] = []
      while (i < lines.length && lines[i].trim().startsWith('>')) {
        quoteLines.push(lines[i].trim().replace(/^>\s?/, ''))
        i++
      }
      blocks.push({ type: 'blockquote', lines: quoteLines })
      continue
    }

    // 无序列表
    if (/^\s*[-*+]\s+/.test(line)) {
      const items: string[] = []
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*[-*+]\s+/, ''))
        i++
      }
      blocks.push({ type: 'list', ordered: false, items })
      continue
    }

    // 有序列表
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const items: string[] = []
      while (i < lines.length && /^\s*\d+[.)]\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\s*\d+[.)]\s+/, ''))
        i++
      }
      blocks.push({ type: 'list', ordered: true, items })
      continue
    }

    // 表格：| a | b |  + 分隔行 + 数据行
    if (
      line.trim().startsWith('|') &&
      i + 1 < lines.length &&
      /^\s*\|[\s:|-]+\|\s*$/.test(lines[i + 1])
    ) {
      const splitRow = (row: string) =>
        row.trim().replace(/^\||\|$/g, '').split('|').map((c) => c.trim())
      const header = splitRow(line)
      i += 2 // 跳过表头与分隔行
      const rows: string[][] = []
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        rows.push(splitRow(lines[i]))
        i++
      }
      blocks.push({ type: 'table', header, rows })
      continue
    }

    // 普通段落：聚合到下一个特殊块为止
    const paraLines: string[] = []
    while (i < lines.length) {
      const l = lines[i]
      const t = l.trim()
      if (
        t === '' ||
        /^```/.test(l) ||
        /^#{1,4}\s+/.test(l) ||
        /^\s*[-*+]\s+/.test(l) ||
        /^\s*\d+[.)]\s+/.test(l) ||
        t.startsWith('|') ||
        /^(-{3,}|\*{3,}|_{3,})\s*$/.test(t) ||
        t.startsWith('>')
      ) {
        break
      }
      paraLines.push(l)
      i++
    }
    blocks.push({ type: 'paragraph', text: paraLines.join('\n') })
  }

  return blocks
}

/* ---------- 代码块（带 header 栏 + 复制） ---------- */

function CodeBlock(props: { language: string; code: string }) {
  const [copied, setCopied] = createSignal(false)

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(props.code)
      setCopied(true)
      setTimeout(() => setCopied(false), 1600)
    } catch {
      /* clipboard 不可用时静默 */
    }
  }

  return (
    <div class="group/code relative rounded-md overflow-hidden my-2">
      <div class="code-block-header">
        <span class="text-[10.5px] text-text-muted font-medium uppercase tracking-wider font-mono">
          {props.language || 'plaintext'}
        </span>
        <button
          class="opacity-0 group-hover/code:opacity-100 group-focus-within/code:opacity-100 p-1 rounded text-text-muted hover:text-text-primary hover:bg-white/70 transition-all focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
          onClick={copy}
          aria-label="复制代码"
          title="复制代码"
        >
          <Show when={copied()} fallback={<Copy class="w-3.5 h-3.5" />}>
            <Check class="w-3.5 h-3.5 text-emerald-600" />
          </Show>
        </button>
      </div>
      <pre class="code-block">
        <code class={props.language ? `language-${props.language}` : ''}>{props.code}</code>
      </pre>
    </div>
  )
}

/* ---------- 块渲染 ---------- */

const HEADING_CLASS: Record<number, string> = {
  1: 'md-heading text-xl font-semibold text-text-primary leading-snug',
  2: 'md-heading text-lg font-semibold text-text-primary leading-snug',
  3: 'md-heading text-base font-semibold text-text-primary leading-snug',
  4: 'md-heading text-sm font-semibold text-text-secondary leading-snug',
}

function renderBlock(block: Block): JSX.Element {
  switch (block.type) {
    case 'paragraph':
      return (
        <p class="whitespace-pre-wrap text-[13.5px] leading-[1.7] text-text-secondary">
          {renderInline(block.text)}
        </p>
      )

    case 'heading': {
      const Tag = `h${block.level}` as 'h1' | 'h2' | 'h3' | 'h4'
      return (
        <Tag class={HEADING_CLASS[block.level]}>
          {renderInline(block.text)}
        </Tag>
      )
    }

    case 'code':
      return <CodeBlock language={block.language} code={block.code} />

    case 'hr':
      return <div class="my-3 h-px bg-border-primary/60" />

    case 'blockquote':
      return (
        <blockquote class="border-l-2 border-nt-io-500/50 pl-3 py-1 text-text-secondary bg-nt-io-500/5 rounded-r-md">
          <For each={block.lines}>
            {(l) => <p class="whitespace-pre-wrap text-[13px] leading-relaxed">{renderInline(l)}</p>}
          </For>
        </blockquote>
      )

    case 'list':
      return block.ordered ? (
        <ol class="list-decimal pl-5 space-y-1 my-2 text-[13.5px] text-text-secondary leading-relaxed">
          <For each={block.items}>
            {(item) => <li class="pl-1 marker:text-text-muted">{renderInline(item)}</li>}
          </For>
        </ol>
      ) : (
        <ul class="list-disc pl-5 space-y-1 my-2 text-[13.5px] text-text-secondary leading-relaxed">
          <For each={block.items}>
            {(item) => <li class="pl-1 marker:text-text-muted">{renderInline(item)}</li>}
          </For>
        </ul>
      )

    case 'table':
      return (
        <div class="my-2 overflow-x-auto rounded-md border border-border-primary/80 bg-white/40">
          <table class="w-full text-xs border-collapse">
            <thead>
              <tr>
                <For each={block.header}>
                  {(h) => (
                    <th class="border-b border-r border-border-primary/60 px-3 py-2 text-left font-semibold text-text-primary bg-white/60 last:border-r-0">
                      {renderInline(h)}
                    </th>
                  )}
                </For>
              </tr>
            </thead>
            <tbody>
              <For each={block.rows}>
                {(row) => (
                  <tr class="odd:bg-white/40">
                    <For each={row}>
                      {(cell) => (
                        <td class="border-b border-r border-border-primary/40 px-3 py-2 text-text-secondary last:border-r-0 last:border-b-0">
                          {renderInline(cell)}
                        </td>
                      )}
                    </For>
                  </tr>
                )}
              </For>
            </tbody>
          </table>
        </div>
      )

    default:
      return <span />
  }
}

/* ---------- 入口 ---------- */

export function Markdown(props: { content: string; class?: string }) {
  return (
    <div class={clsx('markdown-body min-w-0', props.class)}>
      <For each={parseBlocks(props.content)}>
        {(block) => renderBlock(block)}
      </For>
    </div>
  )
}
