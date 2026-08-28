// ══════════════════════════════════════════════════════════════════════════
//  Smart Canvas — 会话桥 (Conversation Bridge)
//  观察 chatStore 的对话结果（文本 / 附件 / 工具结果 / URL），
//  自动 spawn 为画板节点。对话中产生的"图文视频 / 流程 / 网页"即实时上画板。
//  去重：seen 集合按 msg/attachment/tool id 标记，避免重复与无限循环。
// ══════════════════════════════════════════════════════════════════════════
import { createRoot, createEffect } from 'solid-js'
import { chatStore } from '../stores/chat'
import type { Message, NeoCodexAttachmentDto } from '../stores/chat'
import { canvasStore } from '../stores/canvas'

let started = false
const seen = new Set<string>()

function dataUrlOf(att: NeoCodexAttachmentDto): string | null {
  if (!att.data) return null
  return `data:${att.mime_type};base64,${att.data}`
}

/** 工具结果 → 表格 / JSON / diff / 代码 节点 */
function spawnTool(m: Message, idx: number) {
  const tc = m.toolCalls![idx]
  const key = `tool:${tc.id}`
  if (seen.has(key)) return
  seen.add(key)
  const r = tc.result ?? ''
  try {
    const parsed = JSON.parse(r)
    if (Array.isArray(parsed) && parsed.length > 0 && Array.isArray(parsed[0])) {
      canvasStore.spawn({ kind: 'table', title: tc.name, data: parsed, salience: 0.7, source: tc.name })
      return
    }
    if (typeof parsed === 'object' && parsed !== null) {
      canvasStore.spawn({ kind: 'json', title: tc.name, data: parsed, salience: 0.6, source: tc.name })
      return
    }
  } catch {
    /* 非 JSON → 走文本类判定 */
  }
  if (/^\s*[-+]\s|^[+-].*\n[+-]/m.test(r)) {
    canvasStore.spawn({ kind: 'diff', title: tc.name, data: r, salience: 0.6, source: tc.name })
    return
  }
  if (r.trim()) {
    canvasStore.spawn({ kind: 'code', title: tc.name, data: r, salience: 0.55, source: tc.name })
  }
}

/** 文本消息 → 流程图(mermaid) / 网页(URL) / 长文(markdown) 节点 */
function spawnText(m: Message) {
  const c = m.content
  // 流程图：```mermaid 围栏
  const mer = c.match(/```mermaid\s*\n([\s\S]*?)```/)
  if (mer) {
    const k = `mer:${m.id}`
    if (!seen.has(k)) {
      seen.add(k)
      canvasStore.spawn({ kind: 'mermaid', title: '流程图', data: mer[1].trim(), salience: 0.7, source: m.role })
    }
  }
  // 网页：消息中的 URL
  const url = c.match(/https?:\/\/[^\s)]+/)
  if (url) {
    const k = `url:${m.id}`
    if (!seen.has(k)) {
      seen.add(k)
      canvasStore.spawn({ kind: 'webpage', title: '网页', data: { url: url[0] }, salience: 0.6, source: m.role })
    }
  }
  // 长文：助手生成的较长说明/文档（避免每条聊天气泡都上画板，重复对话区）
  if (m.role === 'assistant' && c.length > 600) {
    const k = `doc:${m.id}`
    if (!seen.has(k)) {
      seen.add(k)
      canvasStore.spawn({ kind: 'markdown', title: '长文', data: c, salience: 0.5, source: 'assistant' })
    }
  }
}

function ingest() {
  const msgs = chatStore.currentMessages
  for (const m of msgs) {
    // 附件：图 / 视频
    const atts = m.attachments
    if (atts) {
      for (let i = 0; i < atts.length; i++) {
        const akey = `att:${m.id}:${i}`
        if (seen.has(akey)) continue
        seen.add(akey)
        const url = dataUrlOf(atts[i])
        if (!url) continue
        const mt = atts[i].mime_type
        if (mt.startsWith('image/')) {
          canvasStore.spawn({
            kind: 'image', title: atts[i].name,
            data: { src: url, meta: `${mt} · ${Math.round(atts[i].size / 1024)}KB` },
            salience: 0.8, source: 'attachment',
          })
        } else if (mt.startsWith('video/')) {
          canvasStore.spawn({ kind: 'video', title: atts[i].name, data: { src: url }, salience: 0.8, source: 'attachment' })
        }
      }
    }
    // 流式中的消息暂不上画板（避免抖动），完成后由下次变更触发
    if (m.isStreaming) continue
    if ((m.role === 'assistant' || m.role === 'user') && m.content.trim()) spawnText(m)
    if (m.toolCalls?.length) for (let i = 0; i < m.toolCalls.length; i++) spawnTool(m, i)
  }
}

/** 启动桥（幂等）。在 RightBar onMount 调用一次即可全局生效。 */
export function startCanvasBridge(): void {
  if (started) return
  started = true
  createRoot(() => {
    createEffect(ingest)
  })
}
