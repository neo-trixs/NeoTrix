/**
 * NeoBot 自持根组件 —— **对话区由本仓实现**。
 *
 * # 为什么要自己写
 *
 * 上游 99% 的屏幕时间在 `<iframe>` 里，内容是 `source/deepseek-harness`
 * （空 submodule）起的那个 Web UI。本仓没有那个运行时，所以：
 * 上游的外壳（导航、设置、工作台、恢复页）1:1 复用，**中间的对话区自己实现**。
 *
 * 这是分工，不是缺口 —— 见 `frontend/VENDOR.md`。
 *
 * # 数据一律走 API 平台，不自己 invent 端点
 *
 * 每个能力都能在**设置 → API** 页签里查到它的存活状态。
 * 写一个界面上有、契约里没有的调用，就是制造隐性接口 ——
 * 而「隐性接口」正是本轮花大力气要消灭的那类东西。
 */

import { invoke } from '@tauri-apps/api/core'
import { useCallback, useEffect, useState } from 'react'

interface ConvoView {
  id: string
  kind: string
  title: string
  members: string[]
  task_count: number
  last_active: string
  muted: boolean
  unread: number
}

type Msg = { who: 'me' | 'bot'; text: string }

function relTime(iso: string): string {
  const t = Date.parse(iso)
  if (Number.isNaN(t)) return ''
  const mins = Math.floor((Date.now() - t) / 60000)
  if (mins < 1) return '刚刚'
  if (mins < 60) return `${mins} 分钟前`
  if (mins < 1440) return `${Math.floor(mins / 60)} 小时前`
  if (mins < 2880) return '昨天'
  return `${Math.floor(mins / 1440)} 天前`
}

export function NeoBotRoot() {
  const [convos, setConvos] = useState<ConvoView[] | null>(null)
  const [err, setErr] = useState('')
  const [sel, setSel] = useState<string | null>(null)
  const [msgs, setMsgs] = useState<Msg[]>([])
  const [draft, setDraft] = useState('')
  const [busy, setBusy] = useState(false)

  const reload = useCallback(() => {
    void invoke<ConvoView[]>('neobot_convo_list')
      .then((v) => {
        setConvos(v)
        setErr('')
        setSel((s) => s ?? v[0]?.id ?? null)
      })
      .catch((e) => {
        // ⛔ 读失败就**说读失败**，不留空列表假装「没有会话」——
        //    那两种情况对用户是两种处境，混起来会让人白折腾。
        setConvos([])
        setErr(String(e))
      })
  }, [])

  useEffect(reload, [reload])
  // 挂载即记一条：这是「对话区确实渲染了」的唯一**运行时**证据。
  // 之前只能靠「日志里没有启动错误」反推 —— 那是**否定式**证据，
  // 而「没有报错」和「渲染了」之间没有必然关系。
  useEffect(() => {
    void invoke('log_frontend', { level: 'info', target: 'neobot-root', message: 'mounted' }).catch(() => {})
  }, [])

  async function send() {
    const text = draft.trim()
    if (!text || busy) return
    setDraft('')
    setMsgs(m => [...m, { who: 'me', text }])
    setBusy(true)
    try {
      const r = await invoke<{ output?: string, text?: string }>('neobot_send', { text })
      setMsgs(m => [...m, { who: 'bot', text: r?.output ?? r?.text ?? '（无输出）' }])
    } catch (e) {
      setMsgs(m => [...m, { who: 'bot', text: `发送失败：${String(e).slice(0, 200)}` }])
    } finally {
      setBusy(false)
    }
  }

  const current = convos?.find(c => c.id === sel) ?? null

  return (
    <div className="flex min-h-0 flex-1" data-testid="neobot-root">
      <aside className="flex w-64 shrink-0 flex-col overflow-y-auto border-r border-[#e4e4e7] p-2">
        {err && (
          <div className="m-1 rounded-lg bg-red-50 p-2 text-xs text-[#c33b38]">
            <div className="font-semibold">读取会话失败</div>
            <div className="mt-1 break-all">{err.slice(0, 160)}</div>
            <button
              type="button"
              className="mt-2 rounded-full border border-[#c33b38]/30 px-2 py-0.5 text-[11px]"
              onClick={reload}
            >
              重试
            </button>
          </div>
        )}
        {!err && convos?.length === 0 && (
          <p className="p-3 text-center text-xs text-[#a1a1aa]">
            store 里还没有会话。
            <br />
            在「设置 → 应用」里建一个。
          </p>
        )}
        {convos?.map(c => (
          <button
            key={c.id}
            type="button"
            onClick={() => setSel(c.id)}
            className={`mb-0.5 rounded-lg px-2 py-1.5 text-left hover:bg-[#f5f6f7] ${
              sel === c.id ? 'bg-[#f1f3f5]' : ''
            }`}
          >
            <div className="flex items-center gap-1">
              <span className="truncate text-[13px] text-[#18181b]">{c.title || c.id}</span>
              {c.unread > 0 && (
                <span className="ml-auto shrink-0 rounded-full bg-[#2468f2] px-1.5 text-[10px] text-white">
                  {c.unread}
                </span>
              )}
            </div>
            <div className="truncate text-[11px] text-[#a1a1aa]">
              {c.kind === 'group' ? `${c.members.length} 人群聊` : '私聊'} · {relTime(c.last_active)}
            </div>
          </button>
        ))}
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 shrink-0 items-center border-b border-[#e4e4e7] px-4">
          <span className="text-sm font-semibold text-[#18181b]">
            {current?.title ?? 'NeoBot'}
          </span>
          {current && (
            <span className="ml-2 text-xs text-[#a1a1aa]">
              {current.task_count} 个任务
            </span>
          )}
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
          {msgs.length === 0 ? (
            <p className="pt-16 text-center text-xs text-[#a1a1aa]">
              对话区由 NeoBot 自己实现（上游那部分是 DSH 运行时起的 Web UI，本仓没有）。
            </p>
          ) : (
            <div className="mx-auto max-w-[760px] space-y-2">
              {msgs.map((m, i) => (
                <div key={i} className={m.who === 'me' ? 'flex justify-end' : 'flex'}>
                  <div
                    className={`max-w-[72%] rounded-2xl px-3 py-2 text-[13px] leading-relaxed ${
                      m.who === 'me'
                        ? 'rounded-br-md bg-[#d6e4fb] text-[#12325c]'
                        : 'rounded-bl-md bg-[#ededef] text-[#18181b]'
                    }`}
                  >
                    {m.text}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="shrink-0 border-t border-[#e4e4e7] p-3">
          <div className="mx-auto flex max-w-[760px] gap-2">
            <textarea
              value={draft}
              onChange={e => setDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  void send()
                }
              }}
              rows={1}
              placeholder="消息…（Enter 发送，Shift+Enter 换行）"
              className="min-h-[36px] flex-1 resize-none rounded-2xl border border-[#e4e4e7] bg-[#f9fafb] px-3 py-2 text-[13px] outline-none focus:border-[#2468f2]"
            />
            <button
              type="button"
              onClick={() => void send()}
              disabled={busy || !draft.trim()}
              className="h-9 shrink-0 rounded-full bg-[#0f1115] px-4 text-[13px] text-white disabled:opacity-40"
            >
              {busy ? '…' : '发送'}
            </button>
          </div>
        </div>
      </section>
    </div>
  )
}
