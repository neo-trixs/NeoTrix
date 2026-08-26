/* ════════════════════════════════════════════
   routes/ChatShellProto.tsx — 「意识体在侧」交互原型 v2
   本轮补齐 (对标 Claude/Codex/Osaurus):
   ✅ 会话: 切换/新建/hover置顶(本地) ✅ 空态问候+建议chips
   ✅ 流式打字输出 ✅ Enter发送·Shift换行 ✅ 停止生成
   ✅ 助手消息hover复制 ⬜ markdown渲染/持久化/重生成 → 下批
   ════════════════════════════════════════════ */
import { createSignal, For, Show, onCleanup, onMount, createEffect } from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { Plus, Search, PanelLeftClose, Sparkles, Settings, ChevronDown, Pin, Copy, Check, Square, Pencil, MoreVertical, FileText, Trash2 } from 'lucide-solid'
import { clsx } from 'clsx'
import { Markdown } from '../components/Markdown'
import { subscribeStream, sendMessageStream, stopStream as stopStreamRemote } from '../api'
import { pickAndParseDoc, type ParsedDocFile } from '../api/files'
import { openDirectoryDialog } from '../api/fs'
import { isTauriRuntime } from '../lib/env'
import { kbDocIngest } from '../stores/kb'

interface Msg { role: 'user' | 'assistant'; content: string }
interface Session { id: string; title: string; pinned?: boolean; tags?: string[]; project: string; messages: Msg[] }

const seedSessions = (): Session[] => [
  { id: 's1', title: '吸收 grok-bot 机制映射', pinned: true, tags: ['吸收','研发'], project: 'NeoTrix', messages: [{ role: 'assistant' as const, content: '上一轮的机制映射已归档。随时开新话题。' }] },
  { id: 's2', title: '新对话', tags: [], project: '默认项目', messages: [] },
]

const CANNED_REPLIES = [
  '收到。我检索了记忆库中相关上下文，我的判断是：先把边界定清楚，再动手实现——这样返工最少。\n\n具体来说有三步，我展开讲。',
  '这个问题有意思。从第一性原理看，核心矛盾在于「即时反馈」与「数据一致性」的取舍。成熟产品的做法是乐观更新 + 失败回滚，我可以按这个模式落地。',
  '明白。我已经把要点记入长期记忆。建议下一步：先做一个最小可见版本给你过目风格，确认后再全量迁移——风险最小。',
]

export function ChatShellProto() {
  const navigate = useNavigate()
  const [collapsed, setCollapsed] = createSignal(false)
  const [showSettings, setShowSettings] = createSignal(false)
  const [modelOpen, setModelOpen] = createSignal(false)
  // 意识体自有对话通道置顶, 其余为 gateway 运行时注册的真实后端
  const SELF_MODELS = ['cli-session']
  const [gatewayModels, setGatewayModels] = createSignal<string[]>(['llm7', 'pollinations'])
  const modelOptions = () => [...SELF_MODELS, ...gatewayModels().filter((m) => !SELF_MODELS.includes(m))]
  const loadSessions = (): Session[] => {
    try {
      const raw = localStorage.getItem('neotrix-proto-sessions')
      if (raw) return JSON.parse(raw) as Session[]
    } catch { /* 忽略损坏数据 */ }
    return seedSessions()
  }
  const [sessions, setSessions] = createSignal<Session[]>(loadSessions())
  const [activeId, setActiveId] = createSignal('s2')
  const [draft, setDraft] = createSignal('')
  const [streaming, setStreaming] = createSignal(false)
  const [copiedId, setCopiedId] = createSignal<string | null>(null)
  const [attachments, setAttachments] = createSignal<ParsedDocFile[]>([])
  const [saveToKb, setSaveToKb] = createSignal(true)
  const [dragOver, setDragOver] = createSignal(false)
  const [busyFile, setBusyFile] = createSignal(false)
  const [fileError, setFileError] = createSignal<string | null>(null)
  const [lastError, setLastError] = createSignal<string | null>(null)

  // ── 统一偏好: 强调色/字号/密度/圆角/语言 (持久化, 全原型生效) ──
  type Prefs = { accent: string; fontSize: number; density: 'cozy' | 'compact'; radius: number; lang: 'zh' | 'en'; enterSends: boolean; motion: boolean; msgWidth: 'md' | 'lg'; defaultModel: string; autoTag: boolean }
  const loadPrefs = (): Prefs => {
    try {
      const raw = localStorage.getItem('neotrix-proto-prefs')
      if (raw) return { accent: '#f0913a', fontSize: 15, density: 'cozy', radius: 20, lang: 'zh', enterSends: true, motion: true, msgWidth: 'md', defaultModel: 'cli-session · claude', autoTag: true, ...JSON.parse(raw) }
    } catch { /* ignore */ }
    return { accent: '#f0913a', fontSize: 15, density: 'cozy', radius: 20, lang: 'zh', enterSends: true, motion: true, msgWidth: 'md', defaultModel: 'cli-session · claude', autoTag: true }
  }
  const [prefs, setPrefs] = createSignal<Prefs>(loadPrefs())
  createEffect(() => {
    try { localStorage.setItem('neotrix-proto-prefs', JSON.stringify(prefs())) } catch { /* ignore */ }
  })
  const setPref = <K extends keyof Prefs>(k: K, v: Prefs[K]) => setPrefs((p) => ({ ...p, [k]: v }))
  const [activeTag, setActiveTag] = createSignal<string | null>(null)

  // ── 项目实体化 (对标 Claude Projects / Cursor 工作区) ──
  interface FileNode { path: string; name: string }
  interface Project { id: string; name: string; path?: string }
  const DEFAULT_PROJECT = '默认项目'
  const loadProjects = (): Project[] => {
    try {
      const raw = localStorage.getItem('neotrix-proto-projects')
      if (raw) return JSON.parse(raw)
    } catch { /* ignore */ }
    return [{ id: 'p-neotrix', name: 'NeoTrix' }, { id: 'p-default', name: DEFAULT_PROJECT }]
  }
  const [projects, setProjects] = createSignal<Project[]>(loadProjects())
  const loadFiles = (): Record<string, FileNode[]> => {
    try {
      const raw = localStorage.getItem('neotrix-proto-files')
      if (raw) return JSON.parse(raw)
    } catch { /* ignore */ }
    return { 'p-neotrix': [
      { path: 'docs/DESIGN.md', name: 'DESIGN.md' },
      { path: 'src/lib.rs', name: 'lib.rs' },
    ] }
  }
  const [projectFiles, setProjectFiles] = createSignal<Record<string, FileNode[]>>(loadFiles())
  createEffect(() => {
    try {
      localStorage.setItem('neotrix-proto-projects', JSON.stringify(projects()))
      localStorage.setItem('neotrix-proto-files', JSON.stringify(projectFiles()))
    } catch { /* ignore */ }
  })
  const ensureProject = (name: string): Project => {
    const found = projects().find((pj) => pj.name === name)
    if (found) return found
    const pj = { id: `p-${name}`, name }
    setProjects((all) => [...all, pj])
    return pj
  }
  const [collapsedProjects, setCollapsedProjects] = createSignal<string[]>([])
  const toggleProject = (name: string) =>
    setCollapsedProjects((arr) => (arr.includes(name) ? arr.filter((x) => x !== name) : [...arr, name]))
  const [creatingProject, setCreatingProject] = createSignal(false)
  const [newProjectName, setNewProjectName] = createSignal('')
  function commitNewProject() {
    const name = newProjectName().trim()
    if (!name) { setCreatingProject(false); return }
    ensureProject(name)
    setNewProjectName('')
    setCreatingProject(false)
    setCollapsedProjects((arr) => arr.filter((x) => x !== name))
  }

  /** Tauri 宿主: 弹出系统目录选择器, 以目录名为项目名并记录真实路径 */
  async function handleNewProjectClick() {
    if (!isTauriRuntime()) { setCreatingProject(true); return }
    const dir = await openDirectoryDialog()
    if (!dir) return
    const name = dir.split('/').filter(Boolean).pop() ?? dir
    if (!projects().find((pj) => pj.path === dir)) {
      setProjects((all) => [...all, { id: `p-${name}`, name, path: dir }])
    }
    setCollapsedProjects((arr) => arr.filter((x) => x !== name))
  }

  /** 项目重命名: 同步迁移其下会话归属 */
  function renameProject(id: string, newName: string) {
    const pj = projects().find((x) => x.id === id)
    if (!pj || !newName.trim() || pj.name === newName) return
    setProjects((all) => all.map((x) => (x.id === id ? { ...x, name: newName } : x)))
    setSessions((all) => all.map((sx) => (sx.project === pj.name ? { ...sx, project: newName } : sx)))
  }

  const activeProjectName = () => active()?.project ?? '默认项目'
  const [projMenuOpen, setProjMenuOpen] = createSignal(false)
  const [projRenamingId, setProjRenamingId] = createSignal<string | null>(null)
  const [projRenameVal, setProjRenameVal] = createSignal('')

  function switchProject(name: string) {
    setSessions((all) => all.map((sx) => (sx.id === activeId() ? { ...sx, project: name } : sx)))
    setProjMenuOpen(false)
  }
  const [renamingId, setRenamingId] = createSignal<string | null>(null)
  const [renameVal, setRenameVal] = createSignal('')
  const T = () => prefs().lang === 'zh'
    ? { newChat: '新对话', today: '今天', send: '发送 ↑', settings: '设置', pin: '置顶', more: '更多' }
    : { newChat: 'New Chat', today: 'Today', send: 'Send ↑', settings: 'Settings', pin: 'Pin', more: 'More' }


  const [settingsSection, setSettingsSection] = createSignal('appearance')
  function SettingRow(props: { label: string; children?: import('solid-js').JSX.Element }) {
    return (
      <div class="flex items-center justify-between gap-4 py-2">
        <span class="text-[13px] font-medium">{props.label}</span>
        <div class="flex items-center gap-1 p-0.5 rounded-lg bg-black/5">{props.children}</div>
      </div>
    )
  }
  const segCls = (active: boolean) =>
    clsx('h-6 px-2.5 rounded-md text-[11px] font-medium transition-colors', active ? 'bg-white shadow-sm text-text-primary' : 'text-text-muted hover:text-text-primary')

  let streamTimer: ReturnType<typeof setInterval> | undefined
  let unlisten: (() => void) | null = null
  createEffect(() => {
    try { localStorage.setItem('neotrix-proto-sessions', JSON.stringify(sessions())) } catch { /* 容量满忽略 */ }
  })

  // ── 标签自完善引擎 (无需人工提示的后台自省): 关键词推导+去重规范化 ──
  const TAG_RULES: [RegExp, string][] = [
    [/吸收|absorb/i, '吸收'],
    [/重构|refactor|前端|frontend|UI/i, '前端'],
    [/kb|知识库|knowledge/i, '知识库'],
    [/项目|project/i, '项目'],
    [/测试|test/i, '测试'],
    [/设计|design/i, '设计'],
    [/流式|stream/i, '流式'],
  ]
  const deriveTags = (title: string): string[] => TAG_RULES.filter(([re]) => re.test(title)).map(([, t]) => t)
  const [autoTagCount, setAutoTagCount] = createSignal(0)
  createEffect(() => {
    if (!prefs().autoTag) return
    const all = sessions()
    let changed = false
    let tagged = 0
    const next = all.map((sx) => {
      const existing = sx.tags ?? []
      const merged = [...new Set([...existing, ...(existing.length ? [] : deriveTags(sx.title))])]
      if (merged.length !== existing.length || merged.some((t, i) => t !== existing[i])) {
        changed = true
        if (!existing.length && merged.length) tagged++
        return { ...sx, tags: merged }
      }
      return sx
    })
    if (changed) {
      setSessions(next)
      if (tagged > 0) setAutoTagCount((c) => c + tagged)
    }
  })
  onMount(() => {
    const close = () => setOpenKebab(null)
    window.addEventListener('click', close)
    window.addEventListener('keydown', (e) => { if (e.key === 'Escape') close() })
    onCleanup(() => {
      window.removeEventListener('click', close)
    })
  })
  onCleanup(() => streamTimer && clearInterval(streamTimer))

  // W1-c: 拖拽文件 → 解析 (Tauri webview drag-drop 事件, 非 Tauri 环境安全跳过)
  onMount(() => {
    const unbinds: Promise<() => void>[] = []
    try {
      void import('@tauri-apps/api/event').then(({ listen }) => {
        unbinds.push(listen<{ paths: string[] }>('tauri://drag-drop', (e) => {
          setDragOver(false)
          const p = e.payload.paths?.[0]
          if (p) void addByPath(p)
        }))
        unbinds.push(listen('tauri://drag-enter', () => setDragOver(true)))
        unbinds.push(listen('tauri://drag-leave', () => setDragOver(false)))
      })
    } catch { /* 非 Tauri */ }
    onCleanup(() => { unbinds.forEach((p) => void p.then((f) => f())) })
  })

  async function addByPath(path: string) {
    setBusyFile(true)
    try {
      const { parseDocAt } = await import('../api/files')
      const parsed = await parseDocAt(path)
      setAttachments((a) => [...a, parsed])
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyFile(false)
    }
  }

  async function pickFile() {
    setBusyFile(true)
    try {
      const parsed = await pickAndParseDoc()
      if (parsed) setAttachments((a) => [...a, parsed])
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyFile(false)
    }
  }

  const active = () => sessions().find((s) => s.id === activeId()) ?? sessions()[0]
  const setMsgs = (fn: (m: Msg[]) => Msg[]) =>
    setSessions((all) => all.map((s) => (s.id === activeId() ? { ...s, messages: fn(s.messages) } : s)))

  const allTags = () => {
    const set = new Set<string>()
    sessions().forEach((sx) => sx.tags?.forEach((t) => set.add(t)))
    return [...set]
  }
  const visibleSessions = () =>
    [...sessions()]
      .filter((sx) => { const t = activeTag(); return !t || sx.tags?.includes(t) })
      .sort((a, b) => Number(b.pinned ?? false) - Number(a.pinned ?? false))

  const PROJECT_COLORS = ['#f0913a', '#3b82f6', '#8b5cf6', '#10b981', '#ef4444']
  const projectColor = (name: string) => {
    let h = 0
    for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0
    return PROJECT_COLORS[h % PROJECT_COLORS.length]
  }
  const [collapsedDirs, setCollapsedDirs] = createSignal<string[]>([])
  const [openKebab, setOpenKebab] = createSignal<string | null>(null)
  const kebab = (key: string) => (e: MouseEvent) => { e.stopPropagation(); setOpenKebab(openKebab() === key ? null : key) }
  const menuStop = (e: MouseEvent) => e.stopPropagation()
  // 对标 Notion/Linear: 点击任意处/Esc 关闭弹出菜单
  function deleteProject(id: string) {
    const pj = projects().find((x) => x.id === id)
    if (!pj || pj.name === DEFAULT_PROJECT) return
    setSessions((all) => all.map((sx) => (sx.project === pj.name ? { ...sx, project: DEFAULT_PROJECT } : sx)))
    setProjects((all) => all.filter((x) => x.id !== id))
    setOpenKebab(null)
  }
  const toggleDir = (path: string) =>
    setCollapsedDirs((arr) => (arr.includes(path) ? arr.filter((x) => x !== path) : [...arr, path]))
  function flattenVisible(files: FileNode[]): { name: string; path: string; isDir: boolean; depth: number }[] {
    const out: { name: string; path: string; isDir: boolean; depth: number }[] = []
    const walk = (nodes: TreeNode[], depth: number) => {
      for (const n of nodes) {
        out.push({ name: n.name, path: n.path, isDir: n.isDir, depth })
        if (n.isDir && !collapsedDirs().includes(n.path)) walk(n.children, depth + 1)
      }
    }
    walk(buildTree(files), 0)
    return out
  }
  async function attachFromTree(path: string) {
    setBusyFile(true)
    setFileError(null)
    try {
      const { parseDocAt } = await import('../api/files')
      try {
        const parsed = await parseDocAt(path)
        setAttachments((a) => [...a, parsed])
      } catch {
        // 非 doc-parse 类型 (代码/配置) → read_file 文本回退
        const { readTextFileAt } = await import('../api/files')
        const text = await readTextFileAt(path)
        const name = path.split('/').pop() ?? path
        const ext = name.includes('.') ? name.split('.').pop()! : 'txt'
        setAttachments((a) => [...a, { path, title: name, format: ext, text, tables: null }])
      }
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    } finally {
      setBusyFile(false)
    }
  }
  async function addFileToProject(pid: string) {
    try {
      const parsed = await import('../api/files').then((m) => m.pickAndParseDoc())
      if (!parsed) return
      setProjectFiles((map) => {
        const arr = map[pid] ?? []
        if (arr.some((f) => f.path === parsed.path)) return map
        return { ...map, [pid]: [...arr, { path: parsed.path, name: parsed.title }] }
      })
    } catch (e) {
      setFileError(e instanceof Error ? e.message : String(e))
    }
  }
  const projectGroups = () =>
    projects().map((pj) => ({
      id: pj.id,
      name: pj.name,
      sessions: visibleSessions().filter((sx) => (sx.project || '默认项目') === pj.name),
    }))

  interface TreeNode { name: string; path: string; isDir: boolean; children: TreeNode[] }
  function buildTree(files: FileNode[]): TreeNode[] {
    const root: TreeNode = { name: '', path: '', isDir: true, children: [] }
    for (const f of files) {
      const parts = f.path.split('/')
      let cur = root
      for (let i = 0; i < parts.length; i++) {
        const isLeaf = i === parts.length - 1
        const name = parts[i]
        let next = cur.children.find((c) => c.name === name && c.isDir === !isLeaf)
        if (!next) {
          next = { name, path: parts.slice(0, i + 1).join('/'), isDir: !isLeaf, children: [] }
          cur.children.push(next)
        }
        cur = next
      }
    }
    const sortRec = (n: TreeNode) => {
      n.children.sort((a, b) => (a.isDir === b.isDir ? a.name.localeCompare(b.name) : a.isDir ? -1 : 1))
      n.children.forEach(sortRec)
    }
    sortRec(root)
    return root.children
  }

  /** 追加 token 到末尾助手消息 */
  function appendDelta(delta: string) {
    setMsgs((m) => {
      const last = m[m.length - 1]
      return last?.role === 'assistant'
        ? [...m.slice(0, -1), { ...last, content: last.content + delta }]
        : m
    })
  }

  function newChat(project?: string) {
    const s: Session = { id: `s${Date.now()}`, title: '新对话', project: project ?? DEFAULT_PROJECT, messages: [] }
    setSessions((all) => [s, ...all])
    setCollapsedProjects((arr) => arr.filter((x) => x !== s.project))
    setActiveId(s.id)
    setDraft('')
  }

  function commitRename(id: string) {
    const v = renameVal().trim()
    if (v) setSessions((all) => all.map((x) => (x.id === id ? { ...x, title: v } : x)))
    setRenamingId(null)
  }

  function deleteSession(id: string) {
    setSessions((all) => {
      if (all.length <= 1) return all
      const rest = all.filter((x) => x.id !== id)
      if (activeId() === id) setActiveId(rest[0].id)
      return rest
    })
  }

  function send(text?: string, regen?: boolean) {
    let content = (text ?? draft()).trim()
    if (streaming()) return
        if (!content && !regen) return
    // 首条消息即会话标题 (Claude 式)
    setSessions((all) => all.map((s) => (s.id === activeId() && s.title === '新对话' ? { ...s, title: content.slice(0, 18) } : s)))
    if (!regen) {
      let annotated = content
      for (const a of attachments()) {
        annotated += `\n\n📎 [附件·${a.format}] ${a.title} (${Math.round(a.text.length / 1024)}KB)`
        if (saveToKb()) {
          void kbDocIngest(a.title, a.text, 'default').catch(() => undefined)
        }
      }
      if (annotated !== content) content = annotated
      setAttachments([])
      setMsgs((m) => [...m, { role: 'user', content }])
      setDraft('')
    } else {
      // 首条消息即标题逻辑不触发
    }
    // W2 真流式: 订阅 neocodex_stream_* ; 非 Tauri 环境回退本地打字机
    const reply = CANNED_REPLIES[Math.floor(Math.random() * CANNED_REPLIES.length)]
    setMsgs((m) => [...m, { role: 'assistant', content: '' }])
    setStreaming(true)
    let started = false

    const fallbackTypewriter = () => {
      let i = 0
      streamTimer = setInterval(() => {
        i += 3
        appendDelta(reply.slice(0, i))
        if (i >= reply.length) {
          clearInterval(streamTimer)
          setStreaming(false)
        }
      }, 24)
    }

    void subscribeStream({
      onToken: (delta: string) => { started = true; appendDelta(delta) },
      onDone: () => { setStreaming(false); unlisten?.() },
      onError: (payload: { message?: string }) => {
        setLastError(payload.message ?? '生成中断')
        setStreaming(false)
        unlisten?.()
      },
    }).then((un: () => void) => {
      unlisten = un
      return sendMessageStream({ content, regenerate: regen })
    }).catch((err: unknown) => {
      unlisten?.()
      setStreaming(false)
      // 非 Tauri 宿主 (浏览器预览): 回退打字机演示; 真实失败则暴露错误
      const msg = err instanceof Error ? err.message : String(err)
      if (!started && /not found|未 mock|Cannot/i.test(msg)) {
        fallbackTypewriter()
      } else if (!started) {
        setMsgs((m) => m.slice(0, -1))
        setLastError(msg)
      } else {
        setLastError('连接中断 — 已保留已生成内容, 可重新生成')
      }
    })
  }

  function stopStream() {
    if (streamTimer) clearInterval(streamTimer)
    void stopStreamRemote().catch(() => undefined)
    setStreaming(false)
    setMsgs((m) => {
      const last = m[m.length - 1]
      return last?.role === 'assistant' && !last.content.endsWith('…')
        ? [...m.slice(0, -1), { ...last, content: last.content + ' …' }]
        : m
    })
  }

  async function copyMsg(m: Msg, idx: number) {
    await navigator.clipboard.writeText(m.content).catch(() => undefined)
    setCopiedId(`${activeId()}-${idx}`)
    setTimeout(() => setCopiedId(null), 1200)
  }

  const SUGGESTIONS = ['帮我梳理今天该做什么', '复盘昨天的重构决策', '讲讲你对当前架构的担忧']

  return (
    <div
    class="flex h-screen bg-bg-primary text-text-primary overflow-hidden"
    style={{ fontSize: `${prefs().fontSize}px`, '--ac': prefs().accent } as import('solid-js').JSX.CSSProperties}
  >
      {/* ── 会话侧栏 ── */}
      <Show when={!collapsed()} fallback={
        <button class="w-10 shrink-0 flex items-start justify-center pt-12 text-text-muted hover:text-text-primary" onClick={() => setCollapsed(false)} aria-label="展开会话列表" title="⌘\">
          <PanelLeftClose class="w-4 h-4 rotate-180" />
        </button>
      }>
        <aside class="w-[240px] shrink-0 flex flex-col border-r border-border-primary/40 bg-bg-secondary/40">
          <div class="p-2.5 pt-12 flex items-center gap-1">
<Show when={!creatingProject()} fallback={
              <input class="flex-1 h-8 px-2.5 rounded-lg text-[13px] bg-bg-primary border border-nt-io-500/60 focus:outline-none" value={newProjectName()}
                onInput={(e) => setNewProjectName(e.currentTarget.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') commitNewProject(); if (e.key === 'Escape') { setCreatingProject(false); setNewProjectName('') } }}
                onBlur={() => commitNewProject()} placeholder="项目名称…" autofocus aria-label="新项目名称" />
            }>
              <button class="flex-1 flex items-center gap-1.5 h-8 px-2.5 rounded-lg text-13px font-medium text-text-primary hover:bg-white/60 transition-colors" onClick={() => void handleNewProjectClick()} aria-label="新建项目">
                <Plus class="w-4 h-4" style={{ color: prefs().accent }} /> {prefs().lang === 'zh' ? '新项目' : 'New Project'}
              </button>
            </Show>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" aria-label="搜索会话" title="搜索">
              <Search class="w-4 h-4" />
            </button>
            <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60" onClick={() => setCollapsed(true)} aria-label="折叠侧栏" title="⌘\">
              <PanelLeftClose class="w-4 h-4" />
            </button>
          </div>
          <nav class="flex-1 overflow-y-auto px-2 space-y-0.5" aria-label="会话列表">
            {/* 标签过滤 pills */}
            <Show when={allTags().length > 0}>
              <div class="flex flex-wrap gap-1 px-2 pb-1.5" role="group" aria-label="标签过滤">
                <button
                  class={clsx('px-1.5 py-0.5 rounded-full text-[10px] border transition-colors',
                    activeTag() === null ? 'bg-nt-io-500/15 text-text-primary border-transparent' : 'border-border-primary/50 text-text-muted hover:text-text-primary')}
                  onClick={() => setActiveTag(null)}
                >全部</button>
                <For each={allTags()}>
                  {(tag) => (
                    <button
                      class={clsx('px-1.5 py-0.5 rounded-full text-[10px] border transition-colors',
                        activeTag() === tag ? 'text-white border-transparent' : 'border-border-primary/50 text-text-muted hover:text-text-primary')}
                      style={activeTag() === tag ? { background: prefs().accent } : undefined}
                      onClick={() => setActiveTag(activeTag() === tag ? null : tag)}
                    >#{tag}</button>
                  )}
                </For>
              </div>
            </Show>

            {/* 项目分组 (对标 Claude Projects) */}
            <For each={projectGroups()}>
              {(g) => (
                <div class="mb-1">
                  <div class="group/p flex items-center gap-1 px-2 py-1 rounded-lg hover:bg-white/40 transition-colors">
                    <button class="flex items-center gap-1.5 min-w-0 flex-1 text-left" onClick={() => toggleProject(g.name)} aria-label={`展开/折叠项目 ${g.name}`}>
                      <ChevronDown class={clsx('w-3 h-3 text-text-muted transition-transform shrink-0', collapsedProjects().includes(g.name) && '-rotate-90')} />
                      <span class="w-2 h-2 rounded-full shrink-0" style={{ background: projectColor(g.name) }} />
                      <span class="text-[11px] font-semibold uppercase tracking-wide truncate">{g.name}</span>
                      <span class="text-[10px] text-text-muted">{g.sessions.length}</span>

                    </button>
                    <div class="relative shrink-0">
                      <button
                        class="opacity-0 group-hover/p:opacity-100 p-1 rounded text-text-muted hover:text-text-primary transition-colors"
                        onClick={kebab('proj:' + g.id)}
                        aria-label={`项目 ${g.name} 操作`}
                      >
                        <MoreVertical class="w-3.5 h-3.5" />
                      </button>
                      <Show when={openKebab() === 'proj:' + g.id}>
                        <div class="absolute right-0 top-full mt-1 w-44 rounded-xl border border-border-primary/60 bg-bg-primary shadow-xl p-1 z-30" role="menu" onClick={menuStop}>
                          <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-text-muted hover:bg-white/60 hover:text-text-primary text-left"
                            onClick={() => { newChat(g.name); setOpenKebab(null) }}>
                            <Plus class="w-3.5 h-3.5" /> 新建对话
                          </button>
                          <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-text-muted hover:bg-white/60 hover:text-text-primary text-left"
                            onClick={() => { void addFileToProject(g.id); setOpenKebab(null) }}>
                            <FileText class="w-3.5 h-3.5" /> 添加文件
                          </button>
                          <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-text-muted hover:bg-white/60 hover:text-text-primary text-left"
                            onClick={() => { setProjRenamingId(g.id); setProjRenameVal(g.name); setOpenKebab(null) }}>
                            <Pencil class="w-3.5 h-3.5" /> 重命名项目
                          </button>
                          <Show when={g.name !== DEFAULT_PROJECT}>
                            <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-red-500 hover:bg-red-50 text-left"
                              onClick={() => deleteProject(g.id)}>
                              <Trash2 class="w-3.5 h-3.5" /> 删除项目
                            </button>
                          </Show>
                        </div>
                      </Show>
                    </div>
                  </div>
                  <Show when={!collapsedProjects().includes(g.name)}>
                    <div class="space-y-0.5 pl-1.5">
                      <For each={g.sessions}>
                        {(s) => (
                          <div class={clsx('group flex items-center rounded-lg transition-colors', s.id === activeId() ? 'bg-nt-io-500/10' : 'hover:bg-white/50')}>
                            <Show when={renamingId() === s.id} fallback={
                              <button
                                class={clsx(
                                  'flex-1 min-w-0 text-left px-2.5 py-1.5 text-[13px] truncate flex items-center gap-1.5',
                                  s.id === activeId() ? 'text-text-primary font-medium' : 'text-text-muted hover:text-text-primary',
                                )}
                                onClick={() => { if (renamingId() !== s.id) setActiveId(s.id) }}
                                onDblClick={() => { setRenamingId(s.id); setRenameVal(s.title) }}
                                title={`${s.title} · ${s.project}`}
                              >
                                <Show when={s.pinned}><Pin class="w-3 h-3 shrink-0 rotate-45 text-nt-io-600" /></Show>
                                <span class="truncate">{s.title}</span>
                                <Show when={s.tags?.length}>
                                  <span class="ml-auto hidden group-hover:flex gap-0.5 shrink-0">
                                    <For each={s.tags}>{(t) => <span class="text-[9px] px-1 rounded bg-black/5 text-text-muted">#{t}</span>}</For>
                                  </span>
                                </Show>
                              </button>
                            }>
                              <input
                                class="flex-1 min-w-0 mx-1 my-1 px-2 py-1 rounded-md text-[13px] bg-bg-primary border border-nt-io-500/60 focus:outline-none"
                                value={renameVal()}
                                onInput={(e) => setRenameVal(e.currentTarget.value)}
                                onKeyDown={(e) => {
                                  if (e.key === 'Enter') commitRename(s.id)
                                  if (e.key === 'Escape') setRenamingId(null)
                                }}
                                onBlur={() => commitRename(s.id)}
                                autofocus
                                aria-label={`重命名 ${s.title}`}
                              />
                            </Show>
                  <div class="relative shrink-0">
                    <button
                      class="opacity-0 group-hover:opacity-100 p-1 rounded text-text-muted hover:text-text-primary transition-colors"
                      onClick={kebab('sess:' + s.id)}
                      aria-label={`会话 ${s.title} 操作`}
                    >
                      <MoreVertical class="w-3.5 h-3.5" />
                    </button>
                    <Show when={openKebab() === 'sess:' + s.id}>
                      <div class="absolute right-0 top-full mt-1 w-40 rounded-xl border border-border-primary/60 bg-bg-primary shadow-xl p-1 z-30" role="menu" onClick={menuStop}>
                        <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-text-muted hover:bg-white/60 hover:text-text-primary text-left"
                          onClick={() => { setSessions((all) => all.map((x) => (x.id === s.id ? { ...x, pinned: !x.pinned } : x))); setOpenKebab(null) }}>
                          <Pin class="w-3.5 h-3.5" /> {s.pinned ? '取消置顶' : '置顶'}
                        </button>
                        <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-text-muted hover:bg-white/60 hover:text-text-primary text-left"
                          onClick={() => { setRenamingId(s.id); setRenameVal(s.title); setOpenKebab(null) }}>
                          <Pencil class="w-3.5 h-3.5" /> 重命名
                        </button>
                        <button class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] text-red-500 hover:bg-red-50 text-left"
                          onClick={() => { deleteSession(s.id); setOpenKebab(null) }}>
                          <Trash2 class="w-3.5 h-3.5" /> 删除会话
                        </button>
                      </div>
                    </Show>
                  </div>
                          </div>
                        )}
                      </For>
                    </div>

                    {/* 项目文件目录树 (全类型可入对话) — 空项目整区隐藏 */}
                    <Show when={(projectFiles()[g.id]?.length ?? 0) > 0}>
                    <div class="pl-1.5 pt-1 border-t border-border-primary/30 mt-1">
                      <For each={flattenVisible(projectFiles()[g.id] ?? [])}>
                        {(row) => (
                          <div class="group/f w-full flex items-center gap-1 pr-1 rounded hover:bg-white/50 transition-colors text-[12px]" style={{ 'padding-left': `${6 + row.depth * 12}px` }}>
                            <button
                              class="flex-1 min-w-0 flex items-center gap-1 text-left py-0.5"
                              onClick={() => { if (!row.isDir) { void attachFromTree(row.path) } else toggleDir(row.path) }}
                              aria-label={`${row.isDir ? '目录' : '文件'} ${row.name}`}
                              title={row.isDir ? undefined : `点击附加到对话 · ${row.path}`}
                            >
                              <span class={clsx('text-text-muted text-[9px] w-3 shrink-0', !row.isDir && 'opacity-0')}>{collapsedDirs().includes(row.path) ? '▸' : '▾'}</span>
                              <span class="truncate">{row.name}</span>
                            </button>
                            <Show when={!row.isDir}>
                              <button class="opacity-0 group-hover/f:opacity-100 p-0.5 text-text-muted hover:text-red-500 shrink-0"
                                aria-label={`从项目中移除 ${row.name}`} title="移除"
                                onClick={() => setProjectFiles((map) => {
                                  const arr = (map[g.id] ?? []).filter((f) => f.path !== row.path)
                                  return { ...map, [g.id]: arr }
                                })}>
                                ✕
                              </button>
                            </Show>
                          </div>
                        )}
                      </For>
                    </div>
                    </Show>
                  </Show>
                </div>
              )}
            </For>
          </nav>
          <div class="p-3 border-t border-border-primary/40 space-y-2.5">
            <div class="flex items-center gap-2">
              <div class="w-7 h-7 rounded-lg bg-gradient-to-br from-nt-io-400 to-nt-io-600 flex items-center justify-center text-white text-12px font-bold shrink-0">N</div>
              <div class="min-w-0 flex-1"><p class="text-12px font-medium leading-tight">Neo</p><p class="text-[10px] text-text-muted leading-tight">意识体伴侣</p></div>
              <button class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-white/60 transition-colors" onClick={() => setShowSettings(true)} aria-label="打开设置" title="设置 ⌘,">
                <span class="sr-only">.</span><Settings class="w-4 h-4" />
              </button>
              <Show when={autoTagCount() > 0}>
                <span class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded-full bg-nt-io-500/10 text-nt-io-700 text-[10px]" title="本次会话自动完善标签数">✦ {autoTagCount()}</span>
              </Show>
            </div>
          </div>
        </aside>
      </Show>

      {/* ── 对话主区 ── */}
      <main class="flex-1 flex flex-col min-w-0">
        <div class="flex-1 overflow-y-auto">
          <div class={clsx('mx-auto px-6 py-10', prefs().msgWidth === 'lg' ? 'max-w-4xl' : 'max-w-3xl', prefs().density === 'compact' ? 'space-y-5' : 'space-y-8')}>
            {/* 空态: 问候 + 建议 chips */}
            <Show when={(active()?.messages.length ?? 0) === 0} fallback={
              <For each={active()?.messages}>
                {(m, idx) => (
                  <Show
                    when={m.role === 'user'}
                    fallback={
                      <div class="group border-l-2 border-nt-io-500/70 pl-4 space-y-2">
                        <div class="text-[15px] leading-relaxed">
                          <Markdown content={m.content} />
                          <Show when={streaming() && idx() === (active()?.messages.length ?? 0) - 1}>
                            <span class={clsx('inline-block w-[2px] h-[14px] bg-nt-io-500 ml-0.5 align-middle', prefs().motion && 'animate-pulse')} />
                          </Show>
                        </div>
                        <div class="flex items-center gap-3 opacity-0 group-hover:opacity-100 transition-opacity">
                        <Show when={!streaming() || idx() !== (active()?.messages.length ?? 0) - 1}>
                          <Show when={idx() === (active()?.messages.length ?? 0) - 1}>
                            <button class="opacity-0 group-hover:opacity-100 transition-opacity inline-flex items-center gap-1 text-[11px] text-text-muted hover:text-text-primary"
                              disabled={streaming()}
                              onClick={() => {
                                setMsgs((mm) => mm.slice(0, -1))
                                send('', true)
                              }}
                              aria-label="重新生成">↻ 重新生成</button>
                          </Show>
                          <button class="opacity-0 group-hover:opacity-100 transition-opacity inline-flex items-center gap-1 text-[11px] text-text-muted hover:text-text-primary"
                            onClick={() => void copyMsg(m, idx())} aria-label="复制回复">
                            <Show when={copiedId() === `${activeId()}-${idx()}`} fallback={<><Copy class="w-3 h-3" /> 复制</>}><Check class="w-3 h-3 text-emerald-500" /> 已复制</Show>
                          </button>
                        </Show>
                        </div>
                      </div>
                    }
                  >
                    <div class="flex justify-end">
                      <div class="max-w-[80%] rounded-2xl rounded-tr-sm px-4 py-2.5 text-[15px] whitespace-pre-wrap" style={{ background: `${prefs().accent}14` }}>{m.content}</div>
                    </div>
                  </Show>
                )}
              </For>
            }>
              <div class="pt-16 text-center space-y-6">
                <Sparkles class="w-8 h-8 mx-auto opacity-70" style={{ color: prefs().accent }} />
                <h1 class="text-xl font-semibold">我在。</h1>
                <p class="text-13px text-text-muted">记忆已同步 · 随时开始</p>
                <div class="flex flex-wrap justify-center gap-2 pt-2">
                  <For each={SUGGESTIONS}>
                    {(sg) => (
                      <button class="px-3 py-1.5 rounded-full border border-border-primary/50 text-12px text-text-muted hover:text-text-primary hover:border-nt-io-500/40 transition-colors" onClick={() => send(sg)}>
                        {sg}
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </div>

        {/* 居中悬浮输入框 */}
        <div class="px-6 pb-5">
          <div class={clsx('max-w-3xl mx-auto border bg-bg-secondary/80 backdrop-blur shadow-sm transition-colors', dragOver() ? 'border-nt-io-500 ring-2 ring-nt-io-500/30 bg-nt-io-500/5' : 'border-border-primary/60 focus-within:border-nt-io-500/60')}
            style={{ 'border-radius': `${prefs().radius}px` }}>
            <Show when={fileError()}>
              <p class="mx-4 mt-2 text-[11px] text-red-500" role="alert">{fileError()}</p>
            </Show>
            <Show when={attachments().length > 0}>
              <div class="flex flex-wrap gap-1.5 px-4 pt-3" aria-label="附件列表">
                <For each={attachments()}>
                  {(a) => (
                    <span class="inline-flex items-center gap-1.5 max-w-[240px] rounded-lg border border-border-primary/50 bg-bg-primary/70 px-2 py-1">
                      <span class="text-[11px] font-medium truncate">{a.title}</span>
                      <span class="text-[10px] px-1 rounded bg-nt-io-500/10 text-nt-io-700 uppercase">{a.format}</span>
                      <span class="text-[10px] text-text-muted">{Math.round(a.text.length / 1024)}KB</span>
                      <button class="text-text-muted hover:text-red-500" aria-label={`移除 ${a.title}`} onClick={() => setAttachments((arr) => arr.filter((x) => x !== a))}>✕</button>
                    </span>
                  )}
                </For>
                <label class="inline-flex items-center gap-1 text-[11px] text-text-muted cursor-pointer select-none">
                  <input type="checkbox" checked={saveToKb()} onChange={(e) => setSaveToKb(e.currentTarget.checked)} class="accent-nt-io-500" />
                  存入知识库
                </label>
              </div>
            </Show>
            <textarea
              rows={1}
              value={draft()}
              onInput={(e) => setDraft(e.currentTarget.value)}
              onKeyDown={(e) => {
                const wantSend = prefs().enterSends
                  ? e.key === 'Enter' && !e.shiftKey
                  : e.key === 'Enter' && (e.metaKey || e.ctrlKey)
                if (wantSend && !e.isComposing) {
                  e.preventDefault()
                  send()
                }
              }}
              placeholder="与意识体对话…"
              aria-label="消息输入框"
              class="w-full resize-none bg-transparent px-4 pt-3 pb-1 text-[15px] focus:outline-none placeholder:text-text-muted"
            />
            <div class="flex items-center gap-2 px-3 pb-2">
              <button
                class="w-7 h-7 rounded-lg border border-border-primary/50 bg-bg-primary/70 hover:border-nt-io-500/40 hover:text-nt-io-600 transition-colors inline-flex items-center justify-center disabled:opacity-40"
                onClick={() => void pickFile()}
                disabled={busyFile()}
                aria-label="添加附件"
                title="添加文件 (md/txt/pdf/docx/xlsx)"
              >
                {busyFile() ? '…' : '+'}
              </button>
              <div class="relative">
              <button
                class="flex items-center gap-1.5 h-7 px-2 rounded-lg border border-border-primary/50 bg-bg-primary/70 hover:border-nt-io-500/40 transition-colors"
                onClick={() => setModelOpen(!modelOpen())}
                aria-label="切换模型"
                title="切换对话模型"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                <span class="text-[11px] font-medium">{prefs().defaultModel}</span>
                <ChevronDown
                  class={clsx('w-3 h-3 text-text-muted transition-transform', modelOpen() && 'rotate-180')}
                />
              </button>
              <Show when={modelOpen()}>
                <div
                  class="absolute bottom-full mb-2 left-0 w-60 rounded-xl border border-border-primary/60 bg-bg-primary shadow-xl p-1 z-20"
                  role="listbox"
                  aria-label="对话模型"
                >
                  <p class="px-2 py-1 text-[10px] text-text-muted uppercase tracking-wide">对话模型</p>
                  <For each={modelOptions()}>
                    {(m) => (
                      <button
                        class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg text-[12px] transition-colors"
                        classList={{
                          'bg-nt-io-500/10 font-medium text-text-primary': prefs().defaultModel === m,
                          'text-text-muted hover:bg-white/60 hover:text-text-primary': prefs().defaultModel !== m,
                        }}
                        role="option"
                        aria-selected={prefs().defaultModel === m}
                        onClick={() => { setPref('defaultModel', m); setModelOpen(false) }}
                      >
                        <Show when={SELF_MODELS.includes(m)}>
                          <span class="text-[9px] px-1 rounded bg-nt-io-500/15 text-nt-io-700 font-semibold">自有</span>
                        </Show>
                        <span class="truncate flex-1 text-left">{m}</span>
                        <Show when={prefs().defaultModel === m}>
                          <Check class="w-3 h-3 text-nt-io-600" />
                        </Show>
                      </button>
                    )}
                  </For>
                  <button
                    class="w-full mt-0.5 px-2 py-1.5 rounded-lg text-[11px] text-text-muted hover:text-text-primary hover:bg-white/60 text-left border-t border-border-primary/40"
                    onClick={() => { setModelOpen(false); setShowSettings(true); setSettingsSection('models') }}
                  >
                    ⚙ 管理提供商…
                  </button>
                </div>
              </Show>
              </div>
              <span class="text-[10px] text-text-muted font-mono hidden sm:inline">⏎ 发送 · ⇧⏎ 换行 · ⌘K 动作</span>
              <Show when={streaming()} fallback={
                <button
                  class={clsx('ml-auto h-7 px-3 rounded-lg text-12px font-medium text-white transition-opacity', draft().trim() ? '' : 'opacity-40 cursor-default')}
                  style={{ background: draft().trim() ? prefs().accent : `${prefs().accent}4D` }}
                  onClick={() => send()}
                  aria-label="发送"
                >
                  {T().send}
                </button>
              }>
                <button
                  class="ml-auto h-7 px-3 rounded-lg text-12px font-medium bg-text-primary text-bg-primary inline-flex items-center gap-1.5"
                  onClick={stopStream}
                  aria-label="停止生成"
                >
                  <Square class="w-3 h-3" /> 停止
                </button>
              </Show>
          </div>
          </div>
          {/* duck.ai 吸收: 隐私声明 */}
          <p class="max-w-3xl mx-auto text-center text-[10px] text-text-muted pt-1.5">
            隐私优先 · 会话仅存本机 · 遥测默认关闭
          </p>
        </div>

        {/* 设置 — macOS 系统设置式双栏, 控件真实生效 */}
        <Show when={showSettings()}>
          <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/30" onClick={() => setShowSettings(false)}>
            <div
              class="w-[680px] max-w-[92vw] h-[460px] bg-bg-primary border border-border-primary shadow-2xl flex overflow-hidden"
              style={{ 'border-radius': '20px' }}
              onClick={(e) => e.stopPropagation()}
              role="dialog"
              aria-label="设置"
            >
              {/* 左分类导航 */}
              <nav class="w-[176px] shrink-0 bg-bg-secondary/70 border-r border-border-primary/40 p-3 space-y-1" aria-label="设置分类">
                <p class="px-2 pb-2 text-[15px] font-semibold">{T().settings}</p>
                <For each={[
                  { id: 'general', icon: '✦', label: prefs().lang === 'zh' ? '通用' : 'General' },
                  { id: 'appearance', icon: '◍', label: prefs().lang === 'zh' ? '外观' : 'Appearance' },
                  { id: 'models', icon: '◈', label: prefs().lang === 'zh' ? '模型' : 'Models' },
                  { id: 'data', icon: '⬡', label: prefs().lang === 'zh' ? '数据' : 'Data' },
                  { id: 'about', icon: '✧', label: prefs().lang === 'zh' ? '关于' : 'About' },
                ]}>
                  {(sec) => (
                    <button
                      class={clsx(
                        'w-full flex items-center gap-2 px-2.5 py-1.5 rounded-lg text-[13px] transition-colors',
                        settingsSection() === sec.id ? 'font-medium text-text-primary' : 'text-text-muted hover:bg-white/40',
                      )}
                      style={settingsSection() === sec.id ? { background: `${prefs().accent}1A` } : undefined}
                      onClick={() => setSettingsSection(sec.id)}
                    >
                      <span class="text-nt-io-600">{sec.icon}</span> {sec.label}
                    </button>
                  )}
                </For>
              </nav>

              {/* 右内容面板 */}
              <div class="flex-1 overflow-y-auto p-5">
                <Show when={settingsSection() === 'appearance'}>
                  <div class="space-y-5">
                    <div>
                      <p class="text-[11px] text-text-muted mb-2">{prefs().lang === 'zh' ? '强调色' : 'Accent'}</p>
                      <div class="flex gap-2">
                        <For each={[{ c: '#f0913a', n: '浅金' }, { c: '#3b82f6', n: '蓝' }, { c: '#8b5cf6', n: '紫' }, { c: '#10b981', n: '绿' }]}>
                          {(sw) => (
                            <button
                              class="w-8 h-8 rounded-full border-2 transition-transform hover:scale-110"
                              style={{ background: sw.c, 'border-color': prefs().accent === sw.c ? prefs().accent : 'transparent', outline: prefs().accent === sw.c ? `2px solid ${sw.c}55` : 'none' }}
                              onClick={() => setPref('accent', sw.c)}
                              aria-label={`强调色 ${sw.n}`}
                              title={sw.n}
                            />
                          )}
                        </For>
                      </div>
                    </div>
                    <SettingRow label={prefs().lang === 'zh' ? '字号' : 'Font Size'}>
                      <For each={[{ v: 14, l: '小' }, { v: 15, l: '中' }, { v: 16, l: '大' }]}>
                        {(o) => (
                          <button class={segCls(prefs().fontSize === o.v)} onClick={() => setPref('fontSize', o.v)}>{o.l}</button>
                        )}
                      </For>
                    </SettingRow>
                    <SettingRow label={prefs().lang === 'zh' ? '密度' : 'Density'}>
                      <For each={[{ v: 'compact', l: prefs().lang === 'zh' ? '紧凑' : 'Compact' }, { v: 'cozy', l: prefs().lang === 'zh' ? '舒适' : 'Cozy' }]}>
                        {(o) => (
                          <button class={segCls(prefs().density === o.v)} onClick={() => setPref('density', o.v as 'cozy' | 'compact')}>{o.l}</button>
                        )}
                      </For>
                    </SettingRow>
                    <SettingRow label={prefs().lang === 'zh' ? '圆角风格' : 'Corner Style'}>
                      <For each={[{ v: 12, l: '标准' }, { v: 20, l: 'Mac 大圆角' }]}>
                        {(o) => (
                          <button class={segCls(prefs().radius === o.v)} onClick={() => setPref('radius', o.v)}>{o.l}</button>
                        )}
                      </For>
                    </SettingRow>
                  </div>
                </Show>

                <Show when={settingsSection() === 'general'}>
                  <SettingRow label="语言 / Language">
                    <For each={[{ v: 'zh', l: '中文' }, { v: 'en', l: 'English' }]}>
                      {(o) => (
                        <button class={segCls(prefs().lang === o.v)} onClick={() => setPref('lang', o.v as 'zh' | 'en')}>{o.l}</button>
                      )}
                    </For>
                  </SettingRow>
                  <SettingRow label={prefs().lang === 'zh' ? '清空演示数据' : 'Clear Demo Data'}>
                    <button class="h-7 px-3 rounded-lg text-[12px] border border-red-200 text-red-500 hover:bg-red-50"
                      onClick={() => {
                        ;['neotrix-proto-sessions','neotrix-proto-prefs','neotrix-proto-projects','neotrix-proto-files'].forEach((k)=>localStorage.removeItem(k))
                        location.reload()
                      }}>
                      {prefs().lang === 'zh' ? '清除并重启' : 'Reset'}
                    </button>
                  </SettingRow>
                  <p class="mt-4 text-[12px] text-text-muted leading-relaxed">
                    {prefs().lang === 'zh'
                      ? '偏好即时保存于本机。'
                      : 'Preferences persist locally.'}
                  </p>
                </Show>

                <Show when={settingsSection() === 'models'}>
                  <div class="space-y-3">
                    <p class="text-[11px] text-text-muted">{prefs().lang === 'zh' ? '点击设为默认对话模型 · ● 为网关可用性' : 'Click to set default · ● gateway availability'}</p>
                    <For each={[{ name: 'cli-session', self: true }, ...gatewayModels().filter((m) => m !== 'cli-session').map((m) => ({ name: m, self: false }))]}>
                      {(m) => (
                        <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl border transition-colors text-left"
                          style={{ 'border-color': prefs().defaultModel === m.name ? `${prefs().accent}80` : undefined, background: prefs().defaultModel === m.name ? `${prefs().accent}0D` : undefined }}
                          onClick={() => setPref('defaultModel', m.name)}
                          role="radio" aria-checked={prefs().defaultModel === m.name}
                        >
                          <span class="w-2 h-2 rounded-full shrink-0" style={{ background: m.self ? '#10b981' : '#9ca3af' }} />
                          <span class="text-[13px] font-medium flex-1">{m.name}</span>
                          <Show when={m.self}><span class="text-[9px] px-1.5 py-0.5 rounded bg-nt-io-500/15 text-nt-io-700 font-semibold">自有核</span></Show>
                          <span class={clsx('w-4 h-4 rounded-full border-2 flex items-center justify-center', prefs().defaultModel === m.name ? '' : 'border-border-primary/60')} style={prefs().defaultModel === m.name ? { 'border-color': prefs().accent } : undefined}>
                            <Show when={prefs().defaultModel === m.name}><span class="w-2 h-2 rounded-full" style={{ background: prefs().accent }} /></Show>
                          </span>
                        </button>
                      )}
                    </For>
                    <SettingRow label={prefs().lang === 'zh' ? '提供商广场与连通测试' : 'Providers & Test'}>
                      <button class="h-7 px-3 rounded-lg text-[12px] font-medium text-white" style={{ background: prefs().accent }} onClick={() => { setShowSettings(false); navigate('/plugins') }}>
                        {prefs().lang === 'zh' ? '打开' : 'Open'}
                      </button>
                    </SettingRow>
                  </div>
                </Show>

                <Show when={settingsSection() === 'data'}>
                  <div class="space-y-3">
                    <SettingRow label={prefs().lang === 'zh' ? '导出记忆' : 'Export Memory'}>
                      <button class="h-7 px-3 rounded-lg text-[12px] border border-border-primary/50 hover:bg-white/60" onClick={() => void import('../api/memory').then((m) => m.memoryExport()).then((txt) => {
                        const blob = new Blob([txt], { type: 'application/json' })
                        const a = document.createElement('a')
                        a.href = URL.createObjectURL(blob)
                        a.download = 'neotrix-memory.json'
                        a.click()
                      })}>
                        {prefs().lang === 'zh' ? '导出 JSON' : 'Export'}
                      </button>
                    </SettingRow>
                    <p class="text-[11px] text-text-muted leading-relaxed">
                      {prefs().lang === 'zh'
                        ? '遥测默认关闭 (打包边界)。会话与偏好仅存本机 localStorage / knowledge.db。'
                        : 'Telemetry off by default. Sessions & prefs stay local.'}
                    </p>
                  </div>
                </Show>

                <Show when={settingsSection() === 'about'}>
                  <div class="space-y-4 text-center pt-6">
                    <div class="w-14 h-14 mx-auto rounded-2xl bg-gradient-to-br from-nt-io-400 to-nt-io-600 flex items-center justify-center text-white text-xl font-bold shadow-lg">N</div>
                    <div>
                      <p class="text-[15px] font-semibold">NeoTrix</p>
                      <p class="text-[11px] text-text-muted font-mono mt-0.5">v0.19.0-rc1 · aarch64</p>
                    </div>
                    <p class="text-[12px] text-text-muted leading-relaxed max-w-xs mx-auto">
                      {prefs().lang === 'zh'
                        ? 'AI 原生开发者工具箱 — 与意识体对话的工作台。自进化推理 · VSA 知识表示 · GWT 注意力路由。'
                        : 'AI-native developer toolkit — a workbench to talk with your consciousness entity.'}
                    </p>
                    <div class="flex justify-center gap-4 text-[11px] text-text-muted">
                      <span>隐私优先</span><span>·</span><span>本地优先</span><span>·</span><span>开源精神</span>
                    </div>
                    <p class="text-[10px] text-text-muted/70">© 2026 NeoTrix · Built with Rust + SolidJS</p>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </Show>
      </main>
    </div>
  )
}
