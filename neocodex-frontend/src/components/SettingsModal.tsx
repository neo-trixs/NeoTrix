import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { clsx } from 'clsx'
import { PluginMarketplace } from './PluginMarketplace'
import { TrafficLights } from './TrafficLights'
import { ConfirmModal, type ModalReq } from './ConfirmModal'
import { tagsStore, normalizeTagName, TAG_PALETTE, RECOMMENDED_TAGS, tagDepth } from '../stores/tags'
import { ProviderIcon, CategoryBadge, FreeBadge } from './ProviderIcon'
import { memory, neocodex } from '../api'
import { listenUpdateEvents } from '../api/system'
import type { MemoryStats, ProviderConfig, ProviderMeta, McpServerInfo, McpToolInfo } from '../api/types'

/* ════════════════════════════════════════════
   SettingsModal — 统一设置面板（设计 v3）
   对标主流产品（Claude/Cursor）设置结构：
   左侧分类导航（外扩线条图标） + 右侧内容分区
   图标语言：极简线条 · 外扩（open）而非内敛 —— 开阔心态
   - 通用：提供商统一视觉（ProviderIcon + 分类徽章 + 免费徽章）
   - 标签：推荐标签一键套用（对标 Linear/GitHub 默认 label）+ 计数 + 层级
   ════════════════════════════════════════════ */

/** 提供商分类分组（对标 Claude Desktop 分类设置） */
const CATEGORY_ORDER = ['local', 'proxy', 'cloud', 'unknown'] as const
const CATEGORY_TITLE: Record<string, string> = {
  local: '本地推理 · 数据不出设备',
  proxy: '自定义代理 · OpenAI 兼容中转',
  cloud: '云端 API · 需密钥',
  unknown: '其他',
}

type SectionId = 'general' | 'appearance' | 'plugins' | 'data' | 'tags' | 'about'

/* ── 外扩线条图标（open/expand 语义，非内敛） ── */
function ExpandIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 中央节点 + 四条外射线：向外打开 */}
      <circle cx="8" cy="8" r="1.2" stroke="currentColor" stroke-width="1.3" />
      <line x1="8" y1="3" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="8" y1="13" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="3" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
      <line x1="13" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  )
}

function PaletteIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 色板 + 外扩扇区 */}
      <path d="M8 2.5a5.5 5.5 0 100 11c1.5 0 2-1 1-2-.7-.7-.3-1.5 1-1.5h1.5c1.1 0 2-.9 2-2A5.5 5.5 0 008 2.5z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <circle cx="5.5" cy="6.5" r="0.7" fill="currentColor" />
      <circle cx="8" cy="5" r="0.7" fill="currentColor" />
      <circle cx="10.5" cy="6.5" r="0.7" fill="currentColor" />
    </svg>
  )
}

function InfoIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2" />
      <line x1="8" y1="7.5" x2="8" y2="11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <circle cx="8" cy="4.8" r="0.7" fill="currentColor" />
    </svg>
  )
}

function PluginsIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 拼图块 + 外扩射线（插件扩展语义） */}
      <path d="M5 3h4v2.5a1.5 1.5 0 010 3V11H5V8.5a1.5 1.5 0 010-3V3z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <line x1="8" y1="1" x2="8" y2="0.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="8" y1="15" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="1" y1="8" x2="0.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="15" y1="8" x2="15.5" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

function DataIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 数据库圆柱 + 外扩箭头（导出语义） */}
      <ellipse cx="8" cy="4" rx="5" ry="2.2" stroke="currentColor" stroke-width="1.2" />
      <path d="M3 4v8c0 1.2 2.2 2.2 5 2.2s5-1 5-2.2V4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="3" y1="8" x2="8" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="8" x2="13" y2="3.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="13" y1="3.5" x2="11.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
      <line x1="13" y1="3.5" x2="14.5" y2="5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  )
}

function XIcon() {
  return (
    <svg viewBox="0 0 12 12" fill="none">
      <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
    </svg>
  )
}

function TagIcon() {
  return (
    <svg viewBox="0 0 16 16" fill="none">
      {/* 标签签低 + 斜杠孔 + 外扩射线（标签集合语义） */}
      <path d="M2 3.5h8l4 4.5-6 6L2 9V3.5z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      <circle cx="6.4" cy="6.4" r="0.9" fill="currentColor" stroke="none" />
      <line x1="8" y1="14.5" x2="8" y2="15.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
      <line x1="15" y1="7" x2="15.5" y2="7" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" opacity="0.4" />
    </svg>
  )
}

const SECTIONS: { id: SectionId; label: string; icon: () => any }[] = [
  { id: 'general', label: '通用', icon: ExpandIcon },
  { id: 'appearance', label: '外观', icon: PaletteIcon },
  { id: 'plugins', label: '插件', icon: PluginsIcon },
  { id: 'data', label: '数据', icon: DataIcon },
  { id: 'tags', label: '标签', icon: TagIcon },
  { id: 'about', label: '关于', icon: InfoIcon },
]

/* 分组侧栏导航（对标 osaurus ManagementView 分组结构）：
   常规 General / 扩展 Extensions / 数据 Data / 系统 System */
const NAV_GROUPS: { title: string; ids: SectionId[] }[] = [
  { title: '常规', ids: ['general', 'appearance'] },
  { title: '扩展', ids: ['plugins'] },
  { title: '数据', ids: ['data', 'tags'] },
  { title: '系统', ids: ['about'] },
]

const sectionById = (id: SectionId) => SECTIONS.find((s) => s.id === id)!

export function SettingsModal(props: { open: boolean; onClose: () => void }) {
  const [section, setSection] = createSignal<SectionId>('general')
  const [config, setConfig] = createSignal<ProviderConfig | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [switching, setSwitching] = createSignal(false)
  const [notice, setNotice] = createSignal<string | null>(null)
  const [motionPref, setMotionPref] = createSignal<'full' | 'reduced'>('full')
  const [densityPref, setDensityPref] = createSignal<'comfortable' | 'compact'>('comfortable')
  const [themePref] = createSignal<'light'>('light')
  const [fontSizePref, setFontSizePref] = createSignal<'sm' | 'md' | 'lg'>('md')
  const [memStats, setMemStats] = createSignal<MemoryStats | null>(null)
  const [dataBusy, setDataBusy] = createSignal(false)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)
  // 热更新状态（对标 Cursor/Claude 更新流：检查 → 下载进度 → 重启安装）
  const [updateState, setUpdateState] = createSignal<'idle' | 'checking' | 'available' | 'downloading' | 'downloaded' | 'up-to-date' | 'error'>('idle')
  const [updateInfo, setUpdateInfo] = createSignal<{ current: string; latest: string; error: string | null } | null>(null)
  const [updateProgress, setUpdateProgress] = createSignal<{ downloaded: number; total: number } | null>(null)
  let unlistenUpdate: (() => void) | null = null
  // API 密钥管理（对标 Claude 设置）
  const [apiKey, setApiKey] = createSignal('')
  const [hasKey, setHasKey] = createSignal<boolean | null>(null)
  const [keyBusy, setKeyBusy] = createSignal(false)
  // 标签快速新建
  const [newTagInput, setNewTagInput] = createSignal('')
  // MCP 服务器管理（stdio 注册，会话内生效；对标 Claude Desktop MCP 配置）
  const [mcpServers, setMcpServers] = createSignal<McpServerInfo[]>([])
  const [mcpToolList, setMcpToolList] = createSignal<McpToolInfo[]>([])
  const [mcpLoading, setMcpLoading] = createSignal(false)
  const [mcpBusy, setMcpBusy] = createSignal(false)
  const [showMcpTools, setShowMcpTools] = createSignal(false)
  const [mcpName, setMcpName] = createSignal('')
  const [mcpCommand, setMcpCommand] = createSignal('')
  const [mcpArgs, setMcpArgs] = createSignal('')
  // 统一确认模态：破坏性操作（清空记忆 / 删除密钥 / 删除标签）
  const [modalReq, setModalReq] = createSignal<ModalReq | null>(null)
  const [pendingDeleteTag, setPendingDeleteTag] = createSignal<string | null>(null)
  const [pendingClearMem, setPendingClearMem] = createSignal(false)
  const [pendingDeleteKey, setPendingDeleteKey] = createSignal(false)
  const closeModal = () => {
    setModalReq(null)
    setPendingDeleteTag(null)
    setPendingClearMem(false)
    setPendingDeleteKey(false)
  }
  // 自动消失的通知：成功/错误提示 8s 后自动清除（对标 toast 规范）
  let noticeTimer: ReturnType<typeof setTimeout> | null = null
  const showNotice = (msg: string) => {
    setNotice(msg)
    if (noticeTimer) clearTimeout(noticeTimer)
    noticeTimer = setTimeout(() => setNotice(null), 8000)
  }
  onCleanup(() => {
    if (noticeTimer) clearTimeout(noticeTimer)
  })

  // 偏好持久化：localStorage + 根元素 data-* 属性（CSS 属性选择器响应）
  const applyPrefs = (density: 'comfortable' | 'compact', motion: 'full' | 'reduced', fontSize: 'sm' | 'md' | 'lg') => {
    const root = document.documentElement
    root.dataset.density = density
    root.dataset.motion = motion
    root.dataset.fontSize = fontSize
    root.dataset.theme = 'light'
    try {
      localStorage.setItem('neotrix:prefs', JSON.stringify({ density, motion, theme: 'light', fontSize }))
    } catch { /* 持久化失败静默 */ }
  }

  const setDensity = (d: 'comfortable' | 'compact') => {
    setDensityPref(d)
    applyPrefs(d, motionPref(), fontSizePref())
  }

  const setMotion = (m: 'full' | 'reduced') => {
    setMotionPref(m)
    applyPrefs(densityPref(), m, fontSizePref())
  }

  const setFontSize = (s: 'sm' | 'md' | 'lg') => {
    setFontSizePref(s)
    applyPrefs(densityPref(), motionPref(), s)
  }

  // 启动时恢复偏好
  createEffect(() => {
    if (props.open) {
      try {
        const raw = localStorage.getItem('neotrix:prefs')
        if (raw) {
          const p = JSON.parse(raw)
          if (p.density) setDensityPref(p.density)
          if (p.motion) setMotionPref(p.motion)
          if (p.fontSize) setFontSizePref(p.fontSize)
          applyPrefs(p.density ?? 'comfortable', p.motion ?? 'full', p.fontSize ?? 'md')
        } else {
          applyPrefs('comfortable', 'full', 'md')
        }
      } catch { /* 解析失败用默认 */ }
    }
  })

  const loadConfig = async () => {
    setLoading(true)
    try {
      setConfig(await neocodex.providerConfig())
    } catch (e) {
      showNotice(String(e))
    } finally {
      setLoading(false)
    }
  }

  const loadMemStats = async () => {
    try {
      setMemStats(await memory.memoryStats())
    } catch { /* 记忆统计非关键 */ }
  }

  const loadAppVersion = async () => {
    try {
      setAppVersion(await neocodex.appVersion())
    } catch { /* 版本非关键 */ }
  }

  /* ── 热更新：检查 / 下载 / 进度事件 ── */
  const checkForUpdate = async () => {
    setUpdateState('checking')
    setUpdateInfo(null)
    try {
      const result = await neocodex.checkUpdate()
      setUpdateInfo({ current: result.current, latest: result.latest, error: result.error })
      if (result.available) {
        setUpdateState('available')
      } else {
        setUpdateState(result.error ? 'error' : 'up-to-date')
      }
    } catch (e) {
      setUpdateInfo({ current: appVersion() ?? '', latest: '', error: String(e) })
      setUpdateState('error')
    }
  }

  const downloadUpdate = async () => {
    if (!unlistenUpdate) {
      try {
        unlistenUpdate = await listenUpdateEvents({
          onProgress: (p) => {
            setUpdateProgress(p)
            setUpdateState('downloading')
          },
          onDownloaded: () => {
            setUpdateState('downloaded')
            showNotice('新版本已下载，重启应用即可完成安装')
          },
        })
      } catch (e) {
        console.error('[Settings] Failed to subscribe update events:', e)
      }
    }
    setUpdateState('downloading')
    setUpdateProgress(null)
    setNotice(null)
    try {
      await neocodex.downloadUpdate()
      // 下载完成后由 onDownloaded 事件驱动状态；若事件未到达，轮询确认
    } catch (e) {
      showNotice(String(e))
      setUpdateState('error')
    }
  }

  const restartToInstall = async () => {
    try {
      await neocodex.restartApp()
    } catch (e) {
      showNotice(String(e))
    }
  }

  const exportMemory = async () => {
    setDataBusy(true)
    setNotice(null)
    try {
      const json = await memory.memoryExport('json')
      const path = await save({
        defaultPath: `neotrix-memory-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await writeTextFile(path, json)
        showNotice(`已导出记忆到 ${path}`)
      }
    } catch (e) {
      showNotice(String(e))
    } finally {
      setDataBusy(false)
    }
  }

  const clearMemory = async () => {
    // 破坏性操作确认（对标 Claude 数据管理）
    setPendingClearMem(true)
    setModalReq({
      title: '清空全部记忆',
      message: '确定清空全部记忆？此操作不可恢复，会删除所有经验与知识条目。',
      danger: true,
      confirmLabel: '清空',
    })
  }

  const doClearMemory = async () => {
    closeModal()
    setDataBusy(true)
    setNotice(null)
    try {
      const n = await memory.memoryClear(null)
      showNotice(`已清空 ${n} 条记忆`)
      await loadMemStats()
    } catch (e) {
      showNotice(String(e))
    } finally {
      setDataBusy(false)
    }
  }

  /* API 密钥：读状态 / 保存 / 删除（对标 Claude 设置中的 API 密钥管理） */
  const loadApiKeyStatus = async () => {
    try {
      setHasKey(await memory.hasApiKey())
    } catch { /* 非关键 */ }
  }

  const loadMcp = async () => {
    setMcpLoading(true)
    try {
      const [servers, tools] = await Promise.all([neocodex.mcpList(), neocodex.mcpTools()])
      setMcpServers(servers)
      setMcpToolList(tools)
    } catch (e) {
      showNotice(String(e))
    } finally {
      setMcpLoading(false)
    }
  }

  const registerMcp = async () => {
    const name = mcpName().trim()
    const command = mcpCommand().trim()
    if (!name || !command) {
      showNotice('服务器名称与启动命令不能为空')
      return
    }
    const args = mcpArgs().split(',').map(s => s.trim()).filter(Boolean)
    setMcpBusy(true)
    setNotice(null)
    try {
      const servers = await neocodex.mcpRegister(name, command, args)
      setMcpServers(servers)
      setMcpName('')
      setMcpCommand('')
      setMcpArgs('')
      showNotice(`已注册 MCP 服务器 ${name}`)
      void loadMcp()
    } catch (e) {
      showNotice(String(e))
    } finally {
      setMcpBusy(false)
    }
  }

  const saveApiKey = async () => {
    const key = apiKey().trim()
    if (!key) return
    setKeyBusy(true)
    setNotice(null)
    try {
      await memory.saveApiKey(key)
      setApiKey('')
      await loadApiKeyStatus()
      showNotice('API 密钥已保存')
    } catch (e) {
      showNotice(String(e))
    } finally {
      setKeyBusy(false)
    }
  }

  const deleteApiKey = async () => {
    // 破坏性操作确认
    setPendingDeleteKey(true)
    setModalReq({
      title: '删除 API 密钥',
      message: '确定删除本地保存的 ANTHROPIC_API_KEY？删除后需重新配置。',
      danger: true,
      confirmLabel: '删除',
    })
  }

  const doDeleteApiKey = async () => {
    closeModal()
    setKeyBusy(true)
    setNotice(null)
    try {
      await memory.deleteApiKey()
      await loadApiKeyStatus()
      showNotice('API 密钥已删除')
    } catch (e) {
      showNotice(String(e))
    } finally {
      setKeyBusy(false)
    }
  }

  createEffect(() => {
    if (props.open) {
      setSection('general')
      setNotice(null)
      loadConfig()
      loadMemStats()
      loadAppVersion()
      loadApiKeyStatus()
      loadMcp()
    } else {
      // 关闭时释放更新事件订阅（避免重复监听）
      if (unlistenUpdate) {
        unlistenUpdate()
        unlistenUpdate = null
      }
    }
  })

  const switchProvider = async (name: string) => {
    setSwitching(true)
    setNotice(null)
    try {
      await neocodex.setProvider(name)
      showNotice(`已切换到 ${name}`)
      await loadConfig()
      // 广播提供商变更，输入区 ProviderSelector 即时刷新
      window.dispatchEvent(new CustomEvent('neotrix:provider-changed', { detail: { name } }))
    } catch (e) {
      showNotice(String(e))
    } finally {
      setSwitching(false)
    }
  }

  const activeProvider = () => {
    const cfg = config()
    if (!cfg) return null
    return cfg.providers.find((p) => p.model === cfg.active_model) ?? cfg.providers[0] ?? null
  }

  /* ── 提供商按分类分组（对标 Claude Desktop 分类设置） ── */
  const providerGroups = () => {
    const cfg = config()
    if (!cfg) return []
    const groups: { category: string; title: string; providers: ProviderMeta[] }[] = []
    for (const cat of CATEGORY_ORDER) {
      const list = cfg.providers.filter((p) => (p.category ?? 'unknown') === cat)
      if (list.length > 0) {
        groups.push({ category: cat, title: CATEGORY_TITLE[cat] ?? cat, providers: list })
      }
    }
    // 未知分类兜底（目录外 provider 不丢）
    const rest = cfg.providers.filter((p) => !CATEGORY_ORDER.includes((p.category ?? 'unknown') as (typeof CATEGORY_ORDER)[number]))
    if (rest.length > 0) groups.push({ category: 'unknown', title: '其他', providers: rest })
    return groups
  }

  /* ── 标签：快速新建 / 推荐标签 ── */
  const addTag = () => {
    const raw = newTagInput().trim()
    if (!raw) return
    const name = tagsStore.registerTag(raw)
    if (name) {
      showNotice(`已新建标签 #${name}`)
      setNewTagInput('')
    } else {
      showNotice('标签名无效（需非空，层级用 / 分隔）')
    }
  }

  const missingRecommended = () => RECOMMENDED_TAGS.filter(({ name }) => !tagsStore.state.tags[name])

  const seedTags = () => {
    const n = tagsStore.seedRecommendedTags()
    showNotice(n > 0 ? `已添加 ${n} 个推荐标签` : '推荐标签已全部就绪')
  }

  const [navRef, setNavRef] = createSignal<HTMLElement | null>(null)
  const [dialogEl, setDialogEl] = createSignal<HTMLDivElement | null>(null)
  // 打开前触发元素：关闭后还原焦点（对标 Claude/Cursor 弹窗规范）
  let restoreFocusEl: HTMLElement | null = null

  // 弹窗键盘：Esc 关闭 + 打开聚焦 + Tab 焦点循环 + 关闭还原焦点
  createEffect(() => {
    if (!props.open) return
    restoreFocusEl = document.activeElement as HTMLElement | null
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        // 确认框优先：确认框打开时 Esc 仅关闭确认框，不关整个设置弹窗
        if (modalReq()) {
          closeModal()
          return
        }
        props.onClose()
        return
      }
      // 确认框打开时不劫持 Tab（焦点循环由 ConfirmModal 自管理，避免双陷阱冲突）
      if (e.key !== 'Tab' || modalReq()) return
      const root = dialogEl()
      if (!root) return
      const focusables = root.querySelectorAll<HTMLElement>('button, input, [href], [tabindex]:not([tabindex="-1"])')
      if (focusables.length === 0) return
      const first = focusables[0]
      const last = focusables[focusables.length - 1]
      const active = document.activeElement as HTMLElement | null
      // 焦点在弹窗外（背景/body）：Tab 拉回弹窗内
      if (!active || !root.contains(active)) {
        e.preventDefault()
        if (e.shiftKey) last.focus()
        else first.focus()
        return
      }
      if (e.shiftKey && active === first) {
        e.preventDefault()
        last.focus()
      } else if (!e.shiftKey && active === last) {
        e.preventDefault()
        first.focus()
      }
    }
    window.addEventListener('keydown', onKey)
    navRef()?.querySelector<HTMLButtonElement>('button')?.focus()
    return () => {
      window.removeEventListener('keydown', onKey)
      if (restoreFocusEl?.isConnected) restoreFocusEl.focus()
    }
  })

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/25 backdrop-blur-[2px] animate-fade-in"
        onClick={props.onClose}
      >
        <div
          ref={setDialogEl}
          class="w-[780px] max-w-[94vw] h-[620px] max-h-[88vh] rounded-2xl glass-modal border border-white/40 overflow-hidden flex animate-slide-in"
          onClick={(e) => e.stopPropagation()}
          role="dialog"
          aria-label="设置"
          aria-modal="true"
        >
          {/* ── 左侧分组导航（对标 osaurus ManagementView） ── */}
          <nav ref={setNavRef} class="w-[190px] flex-shrink-0 border-r border-white/30 bg-white/10 py-4 px-2 flex flex-col gap-1 overflow-y-auto" role="tablist" aria-label="设置分类">
            {/* tablist 仅允许 tab / presentation 子节点：非 tab 结构用 presentation 去掉 tab 语义 */}
            <div class="px-3 pb-3" role="presentation">
                <div class="flex items-center gap-2">
                  <TrafficLights />
                </div>
              </div>
            <For each={NAV_GROUPS}>
              {(group) => (
                <div class="mb-2" role="presentation">
                  <div class="px-3 pb-2 pt-2 text-[10px] uppercase tracking-[0.14em] text-text-muted/70 font-medium" role="presentation">
                    {group.title}
                  </div>
                  <For each={group.ids}>
                    {(id) => {
                      const s = sectionById(id)
                      const isActive = section() === id
                      return (
                        <button
                          class={clsx(
                            'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[12.5px] transition-colors',
                            isActive
                              ? 'bg-nt-io-500/12 text-nt-io-700 font-medium shadow-[inset_0_1px_0_rgba(255,255,255,0.6)]'
                              : 'text-text-secondary hover:text-text-primary hover:bg-white/40'
                          )}
                          role="tab"
                          id={`settings-tab-${id}`}
                          aria-selected={isActive}
                          aria-controls="settings-tabpanel"
                          tabIndex={isActive ? 0 : -1}
                          onClick={() => setSection(id)}
                          onKeyDown={(e) => {
                            if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                              e.preventDefault()
                              const flat = NAV_GROUPS.flatMap((g) => g.ids)
                              const idx = flat.indexOf(id)
                              const dir = e.key === 'ArrowDown' ? 1 : -1
                              setSection(flat[(idx + dir + flat.length) % flat.length])
                            } else if (e.key === 'Home') {
                              e.preventDefault(); setSection(NAV_GROUPS[0].ids[0])
                            } else if (e.key === 'End') {
                              e.preventDefault(); const flat = NAV_GROUPS.flatMap((g) => g.ids); setSection(flat[flat.length - 1])
                            }
                          }}
                        >
                          <span class={clsx('w-4 h-4 flex-shrink-0', isActive ? 'text-nt-io-600' : 'text-text-muted')}>
                            <s.icon />
                          </span>
                          <span class="flex-1 text-left truncate">{s.label}</span>
                          {isActive && <span class="w-2 h-2 rounded-full bg-nt-io-500 flex-shrink-0" />}
                        </button>
                      )
                    }}
                  </For>
                </div>
              )}
            </For>
          </nav>

          {/* ── 右侧内容 ── */}
          <div class="flex-1 flex flex-col min-w-0">
            <header class="flex items-center justify-between px-6 py-4 border-b border-border-primary/40 flex-shrink-0 bg-white/20">
              <div class="flex items-center gap-3">
                <span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center flex-shrink-0">
                  {sectionById(section()).icon()}
                </span>
                <div>
                  <div class="text-[15px] font-semibold text-text-primary">
                    {sectionById(section()).label}
                  </div>
                  <div class="text-[11px] text-text-muted">
                    {section() === 'general' && '模型提供商与运行参数'}
                    {section() === 'appearance' && '界面视觉与动效'}
                    {section() === 'plugins' && '技能插件与扩展'}
                    {section() === 'data' && '记忆与数据管理'}
                    {section() === 'tags' && '标签色板与层级管理'}
                    {section() === 'about' && '版本与诊断信息'}
                  </div>
                </div>
              </div>
              <button
                class="p-2 rounded-lg text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors focus-visible:ring-2 focus-visible:ring-nt-io-500 focus-visible:outline-none"
                onClick={props.onClose}
                aria-label="关闭设置"
                title="关闭设置"
              >
                <XIcon />
              </button>
            </header>

            <div class="flex-1 overflow-y-auto px-6 py-5" role="tabpanel" id="settings-tabpanel" aria-labelledby={`settings-tab-${section()}`}>
              {/* ── 通用：模型提供商（统一 v3，对标 Claude Desktop 分类） ── */}
              <Show when={section() === 'general'}>
                <Show when={loading() && !config()}>
                  <div class="text-xs text-text-muted py-6 text-center">加载配置…</div>
                </Show>
                <Show when={config()}>
                  {(cfg) => (
                    <div class="space-y-4">
                      {/* 当前激活提供商 */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <ExpandIcon />
                          当前提供商
                        </div>
                        <div class="ss-card-body">
                          <div class="flex items-center justify-between gap-3">
                            <div class="flex items-center gap-3 min-w-0">
                              <Show when={activeProvider()} fallback={<span class="w-8 h-8 rounded-lg bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center text-[14px] font-semibold flex-shrink-0">?</span>}>
                                {(ap) => (
                                  <>
                                    <ProviderIcon name={ap().name} />
                                    <div class="min-w-0">
                                      <div class="flex items-center gap-1.5">
                                        <span class="text-[13px] font-medium text-text-primary truncate">{ap().display_name}</span>
                                        <CategoryBadge category={ap().category} />
                                        <Show when={ap().is_free}><FreeBadge free /></Show>
                                      </div>
                                      <div class="text-[11px] text-text-muted font-mono truncate mt-0.5">{cfg().active_model}</div>
                                    </div>
                                  </>
                                )}
                              </Show>
                            </div>
                            <span class={clsx('text-[10px] px-2 py-1 rounded-full font-medium flex-shrink-0', cfg().resolvable ? 'bg-nt-core-500/10 text-nt-core-700' : 'bg-nt-shield-500/10 text-nt-shield-600')}>
                              {cfg().resolvable ? 'API 可达' : 'API 不可达'}
                            </span>
                          </div>
                        </div>
                      </div>

                      {/* 提供商列表 — 按分类分组 */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <DataIcon />
                          {cfg().provider_count} 个提供商
                        </div>
                        <div class="ss-card-body space-y-4">
                          <For each={providerGroups()}>
                            {(group) => (
                              <div>
                                <div class="flex items-center gap-2 mb-2">
                                  <span class="text-[10px] uppercase tracking-[0.1em] text-text-muted/80 font-medium">{group.title}</span>
                                  <span class="text-[9px] text-text-muted/60 font-mono">{group.providers.length}</span>
                                </div>
                                <div class="space-y-2">
                                  <For each={group.providers}>
                                    {(p) => {
                                      const isActive = p.model === cfg().active_model
                                      return (
                                        <button
                                          class={clsx(
                                            'w-full flex items-center justify-between gap-3 px-3 py-3 rounded-xl border transition-colors',
                                            isActive
                                              ? 'border-nt-io-500/40 bg-nt-io-500/6'
                                              : 'border-border-primary/50 bg-white/40 hover:bg-white/70'
                                          )}
                                          onClick={() => !isActive && switchProvider(p.name)}
                                          disabled={switching()}
                                          role="radio"
                                          aria-checked={isActive}
                                        >
                                          <div class="flex items-center gap-3 min-w-0">
                                            <ProviderIcon name={p.name} size="sm" />
                                            <div class="min-w-0">
                                              <div class="flex items-center gap-1.5">
                                                <span class="text-[12.5px] text-text-primary font-medium truncate">{p.display_name}</span>
                                                <Show when={p.is_free}><FreeBadge free /></Show>
                                              </div>
                                              <div class="text-[10.5px] text-text-muted font-mono truncate">{p.model}</div>
                                            </div>
                                          </div>
                                          <div class="flex items-center gap-2 flex-shrink-0">
                                            <CategoryBadge category={p.category} className="hidden sm:inline-flex" />
                                            <Show when={isActive}>
                                              <span class="text-[10px] text-nt-io-600 font-medium">✓ 当前</span>
                                            </Show>
                                          </div>
                                        </button>
                                      )
                                    }}
                                  </For>
                                </div>
                              </div>
                            )}
                          </For>
                        </div>
                      </div>

                      {/* API 密钥管理（明确作用域：ANTHROPIC_API_KEY） */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <InfoIcon />
                          API 密钥
                        </div>
                        <div class="ss-card-body space-y-3">
                          <p class="text-[11px] text-text-muted leading-relaxed -mt-1">
                            密钥保存在本地 <span class="font-mono text-text-secondary">ANTHROPIC_API_KEY</span>（Claude 网关）。
                            各云端提供商分别读取自己的环境变量（如 <span class="font-mono">OPENAI_API_KEY</span> / <span class="font-mono">GOOGLE_API_KEY</span>）。
                          </p>
                          <div class="flex items-center gap-2">
                            <input
                              type="password"
                              class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                              placeholder={hasKey() === false ? '输入 API 密钥…' : '输入新密钥替换…'}
                              value={apiKey()}
                              onInput={(e) => setApiKey(e.currentTarget.value)}
                              onKeyDown={(e) => { if (e.key === 'Enter') saveApiKey() }}
                              aria-label="API 密钥"
                            />
                            <button
                              class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
                              onClick={saveApiKey}
                              disabled={keyBusy() || !apiKey().trim()}
                            >
                              保存
                            </button>
                          </div>
                          <div class="flex items-center justify-between">
                            <span class={clsx('text-[11px]', hasKey() === true ? 'text-nt-core-700' : 'text-text-muted')}>
                              {hasKey() === true ? '✓ 已配置 API 密钥' : hasKey() === false ? '未配置 API 密钥' : '检测中…'}
                            </span>
                            <Show when={hasKey() === true}>
                              <button
                                class="px-3 py-1 rounded-lg border border-red-500/30 bg-red-500/5 text-[11px] text-red-500 hover:bg-red-500/10 disabled:opacity-50 transition-colors"
                                onClick={deleteApiKey}
                                disabled={keyBusy()}
                              >
                                删除
                              </button>
                            </Show>
                          </div>
                        </div>
                      </div>

                      {/* MCP 服务器（stdio 注册；对标 Claude Desktop MCP 配置） */}
                      <div class="ss-card">
                        <div class="ss-card-header">
                          <DataIcon />
                          MCP 服务器
                          <span class="ml-auto text-[10px] text-text-muted font-mono">{mcpServers().length} 个</span>
                        </div>
                        <div class="ss-card-body space-y-3">
                          <p class="text-[11px] text-text-muted leading-relaxed -mt-1">
                            注册本地 stdio MCP 服务器，为代理附加外部工具（如文件系统 / 数据库 / 浏览器）。
                            当前会话内生效，重启后重新注册。
                          </p>

                          {/* 服务器列表 */}
                          <Show when={mcpLoading() && mcpServers().length === 0}>
                            <div class="text-xs text-text-muted py-2 text-center">加载 MCP 服务器…</div>
                          </Show>
                          <Show when={!mcpLoading() && mcpServers().length === 0}>
                            <div class="text-[11px] text-text-muted py-3 text-center border border-dashed border-border-primary/60 rounded-lg">
                              暂无 MCP 服务器，填写下方表单注册
                            </div>
                          </Show>
                          <div class="space-y-1.5">
                            <For each={mcpServers()}>
                              {(srv) => (
                                <div class="flex items-center gap-2 px-3 py-2 rounded-lg border border-border-primary/40 bg-white/40">
                                  <span class={clsx('w-2 h-2 rounded-full flex-shrink-0', srv.healthy ? 'bg-emerald-500' : 'bg-red-500')} />
                                  <span class="text-[12px] text-text-primary font-medium truncate flex-1">{srv.name}</span>
                                  <span class="text-[10px] text-text-muted font-mono flex-shrink-0">{srv.transport}</span>
                                  <span class="text-[10px] text-text-muted font-mono flex-shrink-0">{srv.tool_count} 工具</span>
                                  <span class={clsx('text-[10px] px-1.5 py-0.5 rounded-full font-medium flex-shrink-0', srv.healthy ? 'bg-emerald-500/10 text-emerald-600' : 'bg-red-500/10 text-red-500')}>
                                    {srv.healthy ? '健康' : '异常'}
                                  </span>
                                </div>
                              )}
                            </For>
                          </div>

                          {/* 工具一览（可折叠） */}
                          <Show when={mcpToolList().length > 0}>
                            <button
                              class="flex items-center gap-1 text-[11px] text-nt-io-600 hover:text-nt-io-700"
                              onClick={() => setShowMcpTools(!showMcpTools())}
                              aria-expanded={showMcpTools()}
                            >
                              {showMcpTools() ? '▾' : '▸'} 查看工具（{mcpToolList().length}）
                            </button>
                            <Show when={showMcpTools()}>
                              <div class="space-y-1 max-h-40 overflow-y-auto">
                                <For each={mcpToolList()}>
                                  {(tool) => (
                                    <div class="px-2 py-1 rounded bg-bg-primary/40 text-[11px] font-mono break-all">
                                      <span class="text-nt-io-600">{tool.server}.</span>
                                      <span class="text-text-primary">{tool.name}</span>
                                      <span class="text-text-muted"> — {tool.description}</span>
                                    </div>
                                  )}
                                </For>
                              </div>
                            </Show>
                          </Show>

                          {/* 添加表单 */}
                          <div class="border-t border-border-primary/40 pt-3 space-y-2">
                            <div class="grid grid-cols-[1fr_1fr] gap-2">
                              <input
                                class="px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                                placeholder="服务器名称（如 filesystem）"
                                value={mcpName()}
                                onInput={(e) => setMcpName(e.currentTarget.value)}
                                aria-label="MCP 服务器名称"
                              />
                              <input
                                class="px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                                placeholder="启动命令（如 npx）"
                                value={mcpCommand()}
                                onInput={(e) => setMcpCommand(e.currentTarget.value)}
                                onKeyDown={(e) => { if (e.key === 'Enter') registerMcp() }}
                                aria-label="MCP 启动命令"
                              />
                            </div>
                            <div class="flex items-center gap-2">
                              <input
                                class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder:text-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500 font-mono"
                                placeholder="参数（逗号分隔，如 @modelcontextprotocol/server-filesystem, /tmp）"
                                value={mcpArgs()}
                                onInput={(e) => setMcpArgs(e.currentTarget.value)}
                                onKeyDown={(e) => { if (e.key === 'Enter') registerMcp() }}
                                aria-label="MCP 启动参数"
                              />
                              <button
                                class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
                                onClick={registerMcp}
                                disabled={mcpBusy()}
                                aria-label="注册 MCP 服务器"
                              >
                                {mcpBusy() ? '注册中…' : '注册'}
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  )}
                </Show>
              </Show>

              {/* ── 外观：主题 / 字号 / 动效 / 密度 ── */}
              <Show when={section() === 'appearance'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <PaletteIcon />
                      主题
                    </div>
                    <div class="ss-card-body">
                      <div class="ss-row">
                        <div>
                          <div class="ss-row-label">雪域白 · 浅橙</div>
                          <div class="ss-row-desc">唯一主题 · 极简 Mac 圆角</div>
                        </div>
                        <span class="text-[10px] text-nt-io-600">✓ 当前</span>
                      </div>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <ExpandIcon />
                      界面字号
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'sm' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('sm')}
                        role="radio"
                        aria-checked={fontSizePref() === 'sm'}
                      >
                        <div class="text-[12.5px] text-text-primary">小</div>
                        <Show when={fontSizePref() === 'sm'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'md' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('md')}
                        role="radio"
                        aria-checked={fontSizePref() === 'md'}
                      >
                        <div class="text-[12.5px] text-text-primary">中</div>
                        <Show when={fontSizePref() === 'md'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', fontSizePref() === 'lg' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setFontSize('lg')}
                        role="radio"
                        aria-checked={fontSizePref() === 'lg'}
                      >
                        <div class="text-[12.5px] text-text-primary">大</div>
                        <Show when={fontSizePref() === 'lg'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      动效强度
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', motionPref() === 'full' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotion('full')}
                        role="radio"
                        aria-checked={motionPref() === 'full'}
                      >
                        <div class="text-[12.5px] text-text-primary">完整动效</div>
                        <Show when={motionPref() === 'full'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', motionPref() === 'reduced' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setMotion('reduced')}
                        role="radio"
                        aria-checked={motionPref() === 'reduced'}
                      >
                        <div class="text-[12.5px] text-text-primary">减弱动效</div>
                        <Show when={motionPref() === 'reduced'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <div class="px-4 pb-3 -mt-1">
                      <p class="text-[10.5px] text-text-muted">减弱后移除无限循环动画，减少视觉干扰</p>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <InfoIcon />
                      界面密度
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', densityPref() === 'comfortable' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensity('comfortable')}
                        role="radio"
                        aria-checked={densityPref() === 'comfortable'}
                      >
                        <div class="text-[12.5px] text-text-primary">舒适</div>
                        <Show when={densityPref() === 'comfortable'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                      <button
                        class={clsx('w-full flex items-center justify-between px-3 py-3 rounded-xl border transition-colors', densityPref() === 'compact' ? 'border-nt-io-500/40 bg-nt-io-500/6' : 'border-border-primary/50 bg-white/40')}
                        onClick={() => setDensity('compact')}
                        role="radio"
                        aria-checked={densityPref() === 'compact'}
                      >
                        <div class="text-[12.5px] text-text-primary">紧凑</div>
                        <Show when={densityPref() === 'compact'}><span class="text-[10px] text-nt-io-600">✓ 当前</span></Show>
                      </button>
                    </div>
                    <div class="px-4 pb-3 -mt-1">
                      <p class="text-[10.5px] text-text-muted">紧凑模式缩小消息间距与面板内边距，单屏承载更多信息</p>
                    </div>
                  </div>
                </div>
              </Show>

              {/* ── 插件：技能插件市场（内嵌，试错/扩展入口） ── */}
              <Show when={section() === 'plugins'}>
                <PluginMarketplace embedded open onClose={() => {}} />
              </Show>

              {/* ── 数据：记忆统计 + 导出/清空 ── */}
              <Show when={section() === 'data'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      记忆统计
                    </div>
                    <div class="ss-card-body">
                      <Show when={memStats()} fallback={<div class="text-xs text-text-muted py-2">加载记忆统计…</div>}>
                        {(ms) => (
                          <div class="grid grid-cols-2 gap-2">
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">记忆条目</div>
                              <div class="text-[13px] text-text-primary font-medium">{ms().total_entries}</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">分类</div>
                              <div class="text-[13px] text-text-primary font-medium">{ms().total_categories}</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">平均置信度</div>
                              <div class="text-[13px] text-text-primary font-medium">{(ms().avg_confidence * 100).toFixed(0)}%</div>
                            </div>
                            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                              <div class="text-[10px] text-text-muted mb-1">占用空间</div>
                              <div class="text-[13px] text-text-primary font-medium">{(ms().memory_usage_bytes / 1024).toFixed(1)} KB</div>
                            </div>
                          </div>
                        )}
                      </Show>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <ExpandIcon />
                      数据操作
                    </div>
                    <div class="ss-card-body space-y-2">
                      <button
                        class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-border-primary/50 bg-white/40 hover:bg-white/70 transition-colors"
                        onClick={exportMemory}
                        disabled={dataBusy()}
                      >
                        <span class="text-[12.5px] text-text-primary">导出记忆（JSON）</span>
                        <span class="text-[10px] text-text-muted">→ 文件</span>
                      </button>
                      <button
                        class="w-full flex items-center justify-between px-3 py-3 rounded-xl border border-red-500/30 bg-red-500/5 hover:bg-red-500/10 transition-colors"
                        onClick={clearMemory}
                        disabled={dataBusy()}
                      >
                        <span class="text-[12.5px] text-red-500">清空全部记忆</span>
                        <span class="text-[10px] text-red-400">不可恢复</span>
                      </button>
                    </div>
                  </div>
                </div>
              </Show>

              {/* ── 标签：快速新建/层级/计数/推荐（对标 Linear+Obsidian） ── */}
              <Show when={section() === 'tags'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <TagIcon />
                      标签管理
                    </div>
                    <div class="ss-card-body">
                      <div class="text-[11px] text-text-muted leading-relaxed pb-3">
                        标签用于组织会话，支持层级嵌套（如 <span class="nt-tag-hint-inline">工作/功能</span>）。
                        点击色块可单独设色；重命名与删除全局生效；计数徽章显示使用该标签的会话数。
                      </div>

                      {/* 快速新建 */}
                      <div class="flex items-center gap-2 mb-3">
                        <input
                          class="flex-1 min-w-0 px-3 py-2 rounded-lg bg-white/70 border border-border-primary text-[12.5px] text-text-primary placeholder-text-muted/60 focus:outline-none focus:ring-1 focus:ring-nt-io-500"
                          placeholder="新建标签，如 工作/功能 或 领域/前端…"
                          value={newTagInput()}
                          onInput={(e) => setNewTagInput(e.currentTarget.value)}
                          onKeyDown={(e) => { if (e.key === 'Enter') addTag() }}
                          aria-label="新建标签"
                        />
                        <button
                          class="px-3 py-2 rounded-lg bg-nt-io-500 text-text-primary text-[12px] font-medium hover:bg-nt-io-600 disabled:opacity-50 transition-colors flex-shrink-0"
                          onClick={addTag}
                          disabled={!newTagInput().trim()}
                        >
                          添加
                        </button>
                      </div>

                      <Show
                        when={Object.keys(tagsStore.state.tags).length > 0}
                        fallback={
                          <div class="px-3 py-6 text-center text-[11px] text-text-muted border border-dashed border-border-primary/60 rounded-lg">
                            暂无标签。输入上方名称新建，或从下方<b>推荐标签</b>一键套用。
                          </div>
                        }
                      >
                        <ul class="divide-y divide-border-primary/30 rounded-lg border border-border-primary/40 overflow-hidden">
                          <For each={Object.entries(tagsStore.state.tags).sort((a, b) => a[0].localeCompare(b[0]))}>
                            {([name, color]) => (
                              <TagRow
                                name={name}
                                color={color}
                                count={tagsStore.tagCounts()[name] ?? 0}
                                onColorChange={(c) => tagsStore.setTagColor(name, c)}
                                onRename={(next) => {
                                  // renameTag 重名冲突返回错误串（不覆盖不合并），冲突时提示用户
                                  const err = tagsStore.renameTag(name, next)
                                  if (err) showNotice(err)
                                  else showNotice(`已重命名标签 #${next}`)
                                }}
                                onDelete={() => {
                                  // 删除标签全局生效，需确认（对标 Obsidian 标签管理）
                                  setPendingDeleteTag(name)
                                  setModalReq({
                                    title: `删除标签 #${name}`,
                                    message: '删除后该标签将从所有会话移除，此操作不可撤销。',
                                    danger: true,
                                    confirmLabel: '删除',
                                  })
                                }}
                              />
                            )}
                          </For>
                        </ul>
                      </Show>
                    </div>
                  </div>

                  {/* 推荐标签：预置工作流标签（对标 Linear/GitHub 默认 label） */}
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <ExpandIcon />
                      推荐标签
                      <Show when={missingRecommended().length === 0}>
                        <span class="ml-auto text-[10px] font-medium text-nt-core-700 bg-nt-core-500/10 px-2 py-0.5 rounded-full">已全部添加</span>
                      </Show>
                    </div>
                    <div class="ss-card-body">
                      <p class="text-[11px] text-text-muted leading-relaxed pb-3">
                        一套面向 AI 开发工作流的预置标签：<b>工作</b> 归类任务类型，<b>领域</b> 归类技术栈。
                        仅添加缺失项，不会覆盖你已有的标签。
                      </p>
                      <div class="flex items-center gap-2 flex-wrap">
                        <For each={RECOMMENDED_TAGS}>
                          {(r) => {
                            const exists = () => !!tagsStore.state.tags[r.name]
                            return (
                              <button
                                class={clsx(
                                  'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[11px] font-medium transition-all',
                                  exists()
                                    ? 'border-border-primary/40 bg-white/30 text-text-muted cursor-default'
                                    : 'border-white/70 bg-white/55 text-text-primary hover:bg-white/85 hover:shadow-sm cursor-pointer'
                                )}
                                style={exists() ? undefined : { 'border-color': r.color + '55' }}
                                onClick={() => { if (!exists()) { tagsStore.setTagColor(r.name, r.color); showNotice(`已添加推荐标签 #${r.name}`) } }}
                                disabled={exists()}
                                aria-label={exists() ? `${r.name} 已添加` : `添加推荐标签 ${r.name}`}
                              >
                                <span class="w-2 h-2 rounded-full flex-shrink-0" style={{ background: r.color }} />
                                <span class="font-mono">#{r.name}</span>
                                {exists() && <span class="text-[9px] text-nt-core-700">✓</span>}
                              </button>
                            )
                          }}
                        </For>
                      </div>
                      <Show when={missingRecommended().length > 0}>
                        <button
                          class="mt-3 px-3 py-1.5 rounded-lg bg-nt-io-500 text-white text-[11px] font-medium hover:bg-nt-io-600 transition-colors"
                          onClick={seedTags}
                        >
                          一键添加全部推荐（{missingRecommended().length}）
                        </button>
                      </Show>
                    </div>
                  </div>

                  <div class="ss-card">
                    <div class="ss-card-header">
                      <PaletteIcon />
                      标签色板
                    </div>
                    <div class="ss-card-body">
                      <div class="flex items-center gap-2 flex-wrap">
                        <For each={TAG_PALETTE}>
                          {(c) => (
                            <span
                              class="w-6 h-6 rounded-full border border-white/70 shadow-sm"
                              style={{ background: c }}
                              aria-label={`色板 ${c}`}
                            />
                          )}
                        </For>
                      </div>
                      <p class="text-[10.5px] text-text-muted mt-2">
                        新标签自动按名称分配色板颜色，可在上方标签列表手动覆盖。
                      </p>
                    </div>
                  </div>
                </div>
              </Show>

              {/* ── 关于 ── */}
              <Show when={section() === 'about'}>
                <div class="space-y-4">
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <InfoIcon />
                      NeoTrix Desktop
                    </div>
                    <div class="ss-card-body">
                      <div class="flex items-center gap-4">
                        <span class="w-12 h-12 rounded-xl bg-nt-io-500/12 text-nt-io-600 flex items-center justify-center flex-shrink-0">
                          <ExpandIcon />
                        </span>
                        <div>
                          <div class="text-[14px] font-semibold text-text-primary">NeoTrix Desktop</div>
                          <div class="text-[11px] text-text-muted font-mono">v{appVersion() ?? '0.18.0'} · ai.neotrix.desktop</div>
                        </div>
                      </div>
                    </div>
                  </div>
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <InfoIcon />
                      检查更新
                    </div>
                    <div class="ss-card-body space-y-3">
                      <div class="flex items-center gap-3">
                        <div class="text-[11px] text-text-muted flex-1">
                          {updateState() === 'available' && (
                            <span>发现新版本 <span class="font-mono text-nt-io-700">{updateInfo()?.latest}</span>（当前 v{updateInfo()?.current ?? appVersion() ?? '0.18.0'}）</span>
                          )}
                          {updateState() === 'up-to-date' && <span>当前已是最新版本 ✓</span>}
                          {updateState() === 'checking' && <span>正在检查更新…</span>}
                          {updateState() === 'downloading' && (
                            <span>正在下载更新… {updateProgress() ? `${Math.round((updateProgress()!.downloaded / Math.max(updateProgress()!.total, 1)) * 100)}%` : ''}</span>
                          )}
                          {updateState() === 'downloaded' && (
                            <span>新版本已下载，重启应用完成安装</span>
                          )}
                          {updateState() === 'error' && <span class="text-nt-shield-600">检查更新失败：{updateInfo()?.error ?? '未知错误'}</span>}
                          {updateState() === 'idle' && <span>检查是否有可用的新版本</span>}
                        </div>
                        <Show when={updateState() === 'idle' || updateState() === 'available' || updateState() === 'error' || updateState() === 'up-to-date'}>
                          <button
                            class="px-3 py-1.5 rounded-lg text-[12px] font-medium bg-nt-io-500/10 text-nt-io-600 hover:bg-nt-io-500/20 transition-colors disabled:opacity-50"
                            onClick={updateState() === 'available' ? downloadUpdate : checkForUpdate}
                            disabled={updateState() === 'checking' || updateState() === 'downloading'}
                          >
                            {updateState() === 'available' ? '下载并安装' : '检查更新'}
                          </button>
                        </Show>
                        <Show when={updateState() === 'downloaded'}>
                          <button
                            class="px-3 py-1.5 rounded-lg text-[12px] font-medium bg-nt-io-500 text-white hover:bg-nt-io-600 transition-colors"
                            onClick={restartToInstall}
                          >
                            立即重启
                          </button>
                          <button
                            class="px-3 py-1.5 rounded-lg text-[12px] font-medium bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                            onClick={() => setUpdateState('up-to-date')}
                          >
                            稍后
                          </button>
                        </Show>
                      </div>
                      <Show when={updateState() === 'downloading' && updateProgress() && updateProgress()!.total > 0}>
                        <div class="h-1.5 rounded-full bg-bg-tertiary overflow-hidden">
                          <div
                            class="h-full rounded-full bg-nt-io-500 transition-all duration-200"
                            style={{ width: `${Math.min((updateProgress()!.downloaded / updateProgress()!.total) * 100, 100)}%` }}
                          />
                        </div>
                      </Show>
                    </div>
                  </div>
                  <div class="ss-card">
                    <div class="ss-card-header">
                      <DataIcon />
                      诊断信息
                    </div>
                    <div class="ss-card-body">
                      <div class="grid grid-cols-2 gap-2">
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">提供商</div>
                          <div class="text-[12.5px] text-text-primary font-medium">{config()?.provider_count ?? '—'} 个</div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">API 状态</div>
                          <div class={clsx('text-[12.5px] font-medium', config()?.resolvable ? 'text-nt-core-700' : 'text-nt-shield-600')}>
                            {config()?.resolvable ? '可用' : '不可达'}
                          </div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">当前模型</div>
                          <div class="text-[12.5px] text-text-primary font-mono truncate">{config()?.active_model ?? '—'}</div>
                        </div>
                        <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
                          <div class="text-[10px] text-text-muted mb-1">平台</div>
                          <div class="text-[12.5px] text-text-primary">macOS</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </Show>
            </div>

            {/* 底部通知条（可手动关闭，8s 自动消失） */}
            <Show when={notice()}>
              <div class="flex items-center gap-2 px-5 py-2 border-t border-border-primary/40 text-[11px] text-text-secondary flex-shrink-0">
                <span class="flex-1 min-w-0 truncate">{notice()}</span>
                <button
                  class="p-0.5 rounded text-text-muted hover:text-text-primary hover:bg-bg-tertiary transition-colors flex-shrink-0"
                  onClick={() => setNotice(null)}
                  aria-label="关闭提示"
                  title="关闭提示"
                >
                  <XIcon />
                </button>
              </div>
            </Show>
          </div>
        </div>
      </div>

      {/* 破坏性操作确认模态 */}
      <ConfirmModal
        req={modalReq()}
        onConfirm={() => {
          if (pendingDeleteTag()) {
            tagsStore.deleteTag(pendingDeleteTag()!)
            closeModal()
          } else if (pendingClearMem()) {
            void doClearMemory()
          } else if (pendingDeleteKey()) {
            void doDeleteApiKey()
          } else {
            closeModal()
          }
        }}
        onClose={closeModal}
      />
    </Show>
  )
}

/* ════════════════════════════════════════════
   TagRow — 设置内标签行（对标 Obsidian 标签设置 + Linear label row）
   色点 + 名称(层级缩进) + 使用计数 + 色板快捷选色 + 自定义取色 + 重命名/删除
   ════════════════════════════════════════════ */
function TagRow(props: {
  name: string
  color: string
  count?: number
  onColorChange: (color: string) => void
  onRename: (next: string) => void
  onDelete: () => void
}) {
  const [editing, setEditing] = createSignal(false)
  const [editValue, setEditValue] = createSignal(props.name)
  const [pickerOpen, setPickerOpen] = createSignal(false)

  const confirmRename = () => {
    const next = normalizeTagName(editValue())
    if (next) props.onRename(next)
    else setEditValue(props.name)
    setEditing(false)
  }

  const indent = () => (tagDepth(props.name) - 1) * 16

  return (
    <li
      class="flex items-center gap-3 py-2.5 bg-white/30 hover:bg-white/55 transition-colors"
      style={{ 'padding-left': `${12 + indent()}px` }}
    >
      {/* 层级缩进导轨 */}
      <Show when={indent() > 0}>
        <span class="w-px h-6 bg-border-primary/60 flex-shrink-0 -ml-3" aria-hidden="true" />
      </Show>
      {/* 色点（点击展开快捷取色） */}
      <button
        class="w-5 h-5 rounded-full border border-white/80 shadow-sm flex-shrink-0 cursor-pointer transition-transform hover:scale-110"
        style={{ background: props.color }}
        onClick={() => setPickerOpen((v) => !v)}
        aria-label={`标签 ${props.name} 选色`}
        title="选择颜色"
      />
      <Show when={pickerOpen()}>
        <div class="flex items-center gap-1.5 px-2 py-1.5 rounded-lg bg-white/80 border border-border-primary shadow-sm">
          <For each={TAG_PALETTE}>
            {(c) => (
              <button
                class="w-4 h-4 rounded-full border border-white/70 transition-transform hover:scale-115"
                style={{ background: c, 'box-shadow': c === props.color ? '0 0 0 2px #fff, 0 0 0 4px rgba(240,145,58,0.6)' : undefined }}
                onClick={() => { props.onColorChange(c); setPickerOpen(false) }}
                aria-label={`设为 ${c}`}
              />
            )}
          </For>
          <label class="relative flex-shrink-0">
            <span class="w-4 h-4 rounded-full border border-dashed border-text-muted flex items-center justify-center text-[9px] text-text-muted cursor-pointer" title="自定义颜色">
              +
            </span>
            <input
              type="color"
              class="opacity-0 absolute inset-0 w-4 h-4 cursor-pointer"
              value={props.color}
              onInput={(e) => props.onColorChange(e.currentTarget.value)}
              aria-label="自定义颜色"
            />
          </label>
        </div>
      </Show>

      {/* 名称（编辑 / 展示） */}
      <Show
        when={editing()}
        fallback={
          <button
            class="flex-1 min-w-0 text-left group"
            onClick={() => setEditing(true)}
            title="重命名"
          >
            <span class="inline-flex items-center gap-1 text-[12px] text-text-primary truncate">
              <span class="font-mono text-text-muted">#</span>
              <span class="truncate">{props.name}</span>
            </span>
          </button>
        }
      >
        <span class="flex-1 min-w-0 flex items-center gap-1.5">
          <input
            class="flex-1 min-w-0 px-2 py-1 rounded-md bg-white/80 border border-nt-io-500/50 text-[12px] text-text-primary font-mono focus:outline-none focus:ring-1 focus:ring-nt-io-500"
            value={editValue()}
            onInput={(e) => setEditValue(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') confirmRename()
              else if (e.key === 'Escape') { setEditValue(props.name); setEditing(false) }
            }}
            autofocus
          />
          <button
            class="px-2 py-1 rounded-md bg-nt-io-500 text-white text-[10.5px] font-medium hover:bg-nt-io-600 transition-colors flex-shrink-0"
            onClick={confirmRename}
          >
            保存
          </button>
        </span>
      </Show>

      {/* 使用计数（Obsidian 风格徽章） */}
      <Show when={!editing() && (props.count ?? 0) > 0}>
        <span
          class="text-[9px] font-mono font-semibold px-1.5 py-0.5 rounded-full bg-black/4 text-text-muted tabular-nums flex-shrink-0"
          title={`${props.count} 个会话使用此标签`}
        >
          {props.count}
        </span>
      </Show>

      {/* 操作：重命名 / 删除 */}
      <div class="flex items-center gap-1 flex-shrink-0">
        <Show when={!editing()}>
          <button
            class="p-1.5 rounded-md text-text-muted hover:text-text-primary hover:bg-black/5 transition-colors"
            onClick={() => { setEditValue(props.name); setEditing(true) }}
            aria-label={`重命名标签 ${props.name}`}
            title="重命名"
          >
            <svg viewBox="0 0 14 14" fill="none" class="w-3.5 h-3.5">
              <path d="M3 11l.5-2.5L9 3 11 5l-5.5 5.5L3 11z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            </svg>
          </button>
          <button
            class="p-1.5 rounded-md text-text-muted hover:text-red-600 hover:bg-red-500/10 transition-colors"
            onClick={props.onDelete}
            aria-label={`删除标签 ${props.name}`}
            title="删除标签"
          >
            <svg viewBox="0 0 14 14" fill="none" class="w-3.5 h-3.5">
              <path d="M3.5 4.5h7v6a1 1 0 01-1 1h-5a1 1 0 01-1-1v-6z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
              <line x1="2.5" y1="4.5" x2="11.5" y2="4.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
            </svg>
          </button>
        </Show>
      </div>
    </li>
  )
}