import { type JSX } from 'solid-js'
import { clsx } from 'clsx'

export interface GenUIViewProps {
  /** 待渲染的原始内容（通常为工具 result 字符串） */
  content: () => string
}

function tryJson(s: string): unknown | null {
  const t = s.trim()
  if (!(t.startsWith('{') || t.startsWith('['))) return null
  try {
    return JSON.parse(t)
  } catch {
    return null
  }
}

function isMarkdownTable(s: string): boolean {
  const lines = s.split('\n').map((l) => l.trim()).filter(Boolean)
  if (lines.length < 2) return false
  const sep = lines[1].replace(/\|/g, '').replace(/-/g, '').trim()
  return lines[1].includes('---') && sep === '' && lines[0].includes('|')
}

function parseTable(s: string): { head: string[]; rows: string[][] } {
  const lines = s.split('\n').map((l) => l.trim()).filter(Boolean)
  const split = (l: string) => l.replace(/^\||\|$/g, '').split('|').map((c) => c.trim())
  return { head: split(lines[0]), rows: lines.slice(2).map(split) }
}

/**
 * GenUIView — 工具结果富渲染（GenUI 自适应呈现，对标 2026 Agent UX：
 * 不要把原始 pre 甩给用户，按内容类型渲染 JSON / 表格 / 代码 / 文本）。
 * 纯前端，依据内容结构自适应选择呈现形态。
 */
export function GenUIView(props: GenUIViewProps): JSX.Element {
  const json = () => tryJson(props.content())
  const table = () => (isMarkdownTable(props.content()) ? parseTable(props.content()) : null)

  return (
    <div class="gen-ui">
      {(() => {
        const j = json()
        if (j !== null) {
          return (
            <pre class="gen-ui__json">{JSON.stringify(j, null, 2)}</pre>
          )
        }
        const t = table()
        if (t) {
          return (
            <table class="gen-ui__table">
              <thead>
                <tr>
                  {t.head.map((h) => (
                    <th>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {t.rows.map((r) => (
                  <tr>
                    {r.map((c) => (
                      <td>{c}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )
        }
        const raw = props.content()
        const looksCode = raw.includes('\n') && (/^\s{2,}\S|```|;|\{|\}|=>|fn |def |import /m.test(raw))
        return (
          <pre class={clsx('gen-ui__text', looksCode && 'gen-ui__code')}>{raw}</pre>
        )
      })()}
    </div>
  )
}
