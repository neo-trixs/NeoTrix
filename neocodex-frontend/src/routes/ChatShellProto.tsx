/* ════════════════════════════════════════════
   routes/ChatShellProto.tsx — 「意识体在侧」静态高保真原型
   对标 Claude Desktop / Codex / Osaurus · NeoTrix Snowfield White + 浅金单强调
   本文件仅风格评审用, 零业务逻辑; 通过后按此迁移 Chat.tsx。
   ════════════════════════════════════════════ */
import { createSignal, For, Show } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { Plus, Search, PanelLeftClose, Sparkles, Settings, ChevronDown, MoreHorizontal, Pin } from 'lucide-solid'
import { clsx } from 'clsx'

const sessions = [
  { id: '1', title: '吸收 grok-bot 机制映射', active: true },
  { id: '2', title: 'KB doc CRUD 设计评审' },
  { id: '3', title: '前端冗余审计' },
]

function AssistantMsg(props: { children: import('solid-js').JSX.Element }) {
  return (
    <div class="border-l-2 border-nt-io-500/70 pl-4 space-y-2">
      {props.children}
    </div>
  )
}

export function ChatShellProto() {
  const navigate = useNavigate()
  const [collapsed, setCollapsed] = createSignal(false)
  const [draft, setDraft] = createSignal('')
  const [showSettings, setShowSettings] = createSignal(false)
  const [modelOpen, setModelOpen] = createSignal(false)

  return (
    <div class="flex h-screen bg-bg-primary text-text-primary overflow-hidden">
      {/* ── 会话侧栏 ── */}
      <Show when={!collapsed()} fallback={
        <button
          class="w-10 shrink-0 flex items-start justify-center pt-12 text-text-muted hover:text-text-primary"
          onClick={() => setCollapsed(false)}
          aria-label="展开会话列表"
          title="⌘\\"
        >
          <PanelLeftClose class="w-4 h-4 rotate-180" />
        </button>
      }>
        <aside class="w-[240px] shrink-0 flex flex-col border-r border-border-primary/40 bg-bg-secondary/40">
          <div class="p-2.5 pt-12 flex items-center gap-1">
            <button class="flex-1 flex items-center gap-1.5 h-8 px-2.5 rounded-lg text-13px font-medium text-text-primary hover:bg-white/60 transition-colors" aria-label="新建对话">
              <Plus class="w-4 h-4 text-nt-io-600" /> 新对话
            </button>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" aria-label="搜索会话" title="搜索">
              <Search class="w-4 h-4" />
            </button>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setCollapsed(true)} aria-label="折叠侧栏" title="⌘\">
              <PanelLeftClose class="w-4 h-4" />
            </button>
          </div>
          <nav class="flex-1 overflow-y-auto px-2 space-y-0.5" aria-label="会话列表">
            <p class="px-2.5 pt-1 pb-1 text-[10px] font-semibold uppercase tracking-wide text-text-muted">今天</p>
            <For each={sessions}>
              {(s) => (
                <div class={clsx(
                  'group flex items-center rounded-lg transition-colors',
                  s.active ? 'bg-nt-io-500/10' : 'hover:bg-white/50',
                )}>
                  <button
                    class={clsx(
                      'flex-1 min-w-0 text-left px-2.5 py-1.5 text-13px truncate',
                      s.active ? 'text-text-primary font-medium' : 'text-text-muted hover:text-text-primary',
                    )}
                  >
                    {s.title}
                  </button>
                  {/* hover 操作: 置顶 / 更多 (对标 Claude Desktop 会话行) */}
                  <span class="hidden group-hover:flex items-center gap-0.5 pr-1.5 shrink-0">
                    <button class="p-1 rounded text-text-muted hover:text-text-primary" aria-label={`置顶 ${s.title}`} title="置顶"><Pin class="w-3 h-3" /></button>
                    <button class="p-1 rounded text-text-muted hover:text-text-primary" aria-label={`${s.title} 更多`} title="更多"><MoreHorizontal class="w-3 h-3" /></button>
                  </span>
                </div>
              )}
            </For>
          </nav>
          <div class="p-3 border-t border-border-primary/40 space-y-2.5">
            <div class="flex items-center gap-1.5 text-11px text-text-muted">
              <span class="relative flex h-1.5 w-1.5">
                <span class="animate-ping absolute h-full w-full rounded-full bg-emerald-400 opacity-60" />
                <span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />
              </span>
              意识体在线 · 记忆同步完成
            </div>
            {/* 用户条 + 设置齿轮 (对标 Claude Desktop 左下角) */}
            <div class="flex items-center gap-2">
              <div class="w-7 h-7 rounded-lg bg-gradient-to-br from-nt-io-400 to-nt-io-600 flex items-center justify-center text-white text-12px font-bold shrink-0">N</div>
              <div class="min-w-0 flex-1">
                <p class="text-12px font-medium leading-tight">Neo</p>
                <p class="text-[10px] text-text-muted leading-tight">意识体伴侣</p>
              </div>
              <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors" onClick={() => setShowSettings(true)} aria-label="打开设置" title="设置 ⌘,">
                <Settings class="w-4 h-4" />
              </button>
            </div>
          </div>
        </aside>
      </Show>

      {/* ── 对话主区 ── */}
      <main class="flex-1 flex flex-col min-w-0">
        {/* 消息流: 宽排版无气泡 */}
        <div class="flex-1 overflow-y-auto">
          <div class="max-w-3xl mx-auto px-6 py-10 space-y-8">
            <AssistantMsg>
              <p class="text-[15px] leading-relaxed">早上好。昨夜后台完成了 3 个周期的自进化， 检索精度提升 2.1%。今天建议优先处理前端重构收尾。</p>
            </AssistantMsg>

            <div class="flex justify-end">
              <div class="max-w-[80%] bg-nt-io-500/8 rounded-2xl rounded-tr-sm px-4 py-2.5 text-[15px]">
                前端现在的界面确实乱, 我们重新定个设计方向吧
              </div>
            </div>

            <AssistantMsg>
              <p class="text-[15px] leading-relaxed">同意。我已对标 Claude Desktop / Codex / Osaurus 提炼出「意识体在侧」方案:</p>
              <ul class="text-[15px] leading-relaxed list-disc pl-5 space-y-1">
                <li>对话是唯一主角, 功能全部降级到 ⌘K</li>
                <li>我的回复不加气泡 — 用左侧金线标识身份</li>
                <li>Snowfield White 底 + 浅金只出现在焦点处</li>
              </ul>
              <p class="text-[15px] leading-relaxed">当前你看到的就是新形态原型。</p>
            </AssistantMsg>

            {/* 工具调用折叠单行 */}
            <button class="flex items-center gap-2 text-12px text-text-muted hover:text-text-primary transition-colors" aria-label="工具调用详情">
              <Sparkles class="w-3.5 h-3.5 text-nt-io-600" />
              调用了 kb_search · 3 条结果
              <span class="text-[10px] border border-border-primary/50 rounded px-1">展开</span>
            </button>
          </div>
        </div>

        {/* 居中悬浮输入框 */}
        <div class="px-6 pb-5">
          <div class="max-w-3xl mx-auto rounded-2xl border border-border-primary/60 bg-bg-secondary/80 backdrop-blur shadow-sm focus-within:border-nt-io-500/60 transition-colors">
            <textarea
              rows={1}
              value={draft()}
              onInput={(e) => setDraft(e.currentTarget.value)}
              placeholder="与意识体对话…"
              aria-label="消息输入框"
              class="w-full resize-none bg-transparent px-4 pt-3 pb-1 text-[15px] focus:outline-none placeholder:text-text-muted"
            />
            <div class="flex items-center gap-2 px-3 pb-2">
              {/* 模型 chip: 在线状态内联, 点击展开列表 (Claude/Cursor 式 composer 内选择) */}
              <div class="relative">
                <button
                  class="flex items-center gap-1.5 h-7 px-2 rounded-lg border border-border-primary/50 bg-bg-primary/70 hover:border-nt-io-500/40 transition-colors"
                  onClick={() => setModelOpen(!modelOpen())}
                  aria-label="切换模型"
                >
                  <span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                  <span class="text-[11px] font-medium">cli-session · claude</span>
                  <ChevronDown class="w-3 h-3 text-text-muted" />
                </button>
                <Show when={modelOpen()}>
                  <div class="absolute bottom-9 left-0 w-52 rounded-xl border border-border-primary/60 bg-bg-primary shadow-lg p-1 space-y-0.5 z-10" role="listbox" aria-label="模型列表">
                    {['cli-session · claude', 'llm7 · turbo', '本地 · qwen2.5'].map((m) => (
                      <button class="w-full flex items-center gap-2 text-left px-2 py-1.5 rounded-lg text-12px text-text-muted hover:bg-white/60 hover:text-text-primary" role="option">
                        {m.startsWith('cli') && <span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />}
                        {m}
                      </button>
                    ))}
                  </div>
                </Show>
              </div>
              <span class="text-[10px] text-text-muted font-mono hidden sm:inline">⌘K 动作 · ⌘\ 侧栏</span>
              <button
                class={clsx(
                  'ml-auto h-7 px-3 rounded-lg text-12px font-medium text-white transition-colors',
                  draft().trim() ? 'bg-nt-io-500 hover:bg-nt-io-600' : 'bg-nt-io-500/30 cursor-default',
                )}
                aria-label="发送"
              >
                发送 ↑
              </button>
            </div>
          </div>
        </div>
      </main>

      {/* 设置弹层 (Phase4 极浅层: 四入口, 后续逐个实体化) */}
      <Show when={showSettings()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setShowSettings(false)}>
          <div
            class="w-[420px] rounded-2xl bg-bg-primary border border-border-primary shadow-xl overflow-hidden"
            onClick={(e) => e.stopPropagation()}
            role="dialog"
            aria-label="设置"
          >
            <div class="px-5 py-3.5 border-b border-border-primary/40 flex items-center">
              <h2 class="text-14px font-semibold">设置</h2>
              <button class="ml-auto p-1 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setShowSettings(false)} aria-label="关闭设置">✕</button>
            </div>
            <div class="p-2">
              {[
                { icon: '✦', t: '意识体', d: '人格 / 记忆深度 / 自进化开关', k: '⌘,' },
                { icon: '◈', t: '模型与提供商', d: '路由策略 / 连通性 / 密钥', k: '' },
                { icon: '◍', t: '外观', d: '密度 / 字号 / 动效', k: '' },
                { icon: '⬡', t: '数据与隐私', d: '本地存储 / 导出 / 遥测门', k: '' },
              ].map((item) => (
                <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl hover:bg-white/60 transition-colors text-left">
                  <span class="w-8 h-8 rounded-lg bg-nt-io-500/10 flex items-center justify-center text-nt-io-600">{item.icon}</span>
                  <span class="min-w-0 flex-1">
                    <span class="block text-13px font-medium">{item.t}</span>
                    <span class="block text-11px text-text-muted truncate">{item.d}</span>
                  </span>
                  <Show when={item.k}><span class="text-[10px] text-text-muted font-mono">{item.k}</span></Show>
                </button>
              ))}
            </div>
          </div>
        </div>
      </Show>
    </div>
  )
}
