import { createSignal, createEffect, onCleanup, For, Show } from 'solid-js'
import { clsx } from 'clsx'
import { PluginMarketplace } from './PluginMarketplace'
import { TrafficLights } from './TrafficLights'
import { ConfirmModal, type ModalReq } from './ConfirmModal'
import { tagsStore, RECOMMENDED_TAGS } from '../stores/tags'
import { memory, neocodex, errText, fs as fsApi } from '../api'
import { storageGet, storageSet } from '../lib/env'
import type { MemoryStats, ProviderConfig } from '../api/types'
import { GeneralSection } from './settings/GeneralSection'
import { AppearanceSection } from './settings/AppearanceSection'
import { DataSection } from './settings/DataSection'
import { TagsSection } from './settings/TagsSection'
import { AboutSection } from './settings/AboutSection'
import { XIcon, ExpandIcon, PaletteIcon, PluginsIcon, DataIcon, TagIcon, InfoIcon } from './settings/settingsIcons'

/* ════════════════════════════════════════════
   SettingsModal — 统一设置面板（设计 v3）
   对标主流产品（Claude/Cursor）设置结构：
   左侧分类导航（外扩线条图标） + 右侧内容分区
   图标语言：极简线条 · 外扩（open）而非内敛 —— 开阔心态
   - 通用：提供商统一视觉（ProviderIcon + 分类徽章 + 免费徽章）
   - 标签：推荐标签一键套用（对标 Linear/GitHub 默认 label）+ 计数 + 层级
   ════════════════════════════════════════════ */

/** 提供商分类分组（对标 Claude Desktop 分类设置） */
type SectionId = 'general' | 'appearance' | 'plugins' | 'data' | 'tags' | 'about'

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
  const [fontSizePref, setFontSizePref] = createSignal<'sm' | 'md' | 'lg'>('md')
  const [memStats, setMemStats] = createSignal<MemoryStats | null>(null)
  const [dataBusy, setDataBusy] = createSignal(false)
  const [appVersion, setAppVersion] = createSignal<string | null>(null)
  // 热更新状态（对标 Cursor/Claude 更新流：检查 → 下载进度 → 重启安装）
  // API 密钥管理（对标 Claude 设置）
  const [apiKey, setApiKey] = createSignal('')
  const [hasKey, setHasKey] = createSignal<boolean | null>(null)
  const [keyBusy, setKeyBusy] = createSignal(false)
  // 标签快速新建
  const [newTagInput, setNewTagInput] = createSignal('')
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
      storageSet('neotrix:prefs', JSON.stringify({ density, motion, theme: 'light', fontSize }))
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
        const raw = storageGet('neotrix:prefs')
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

  // 🟡 修复：loadConfig 请求序号守卫——switchProvider 内 await loadConfig() 与
  // 打开弹窗时 createEffect 触发的 loadConfig() 可并发，先发（切换前）的迟到响应
  // 会覆盖新配置，状态栏短暂显示旧模型。序号丢弃过期响应。
  let cfgReqSeq = 0
  const loadConfig = async () => {
    const seq = ++cfgReqSeq
    setLoading(true)
    try {
      const cfg = await neocodex.providerConfig()
      if (seq !== cfgReqSeq) return
      setConfig(cfg)
    } catch (e) {
      if (seq !== cfgReqSeq) return
      showNotice(errText(e))
    } finally {
      if (seq === cfgReqSeq) setLoading(false)
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

  const exportMemory = async () => {
    setDataBusy(true)
    setNotice(null)
    try {
      const json = await memory.memoryExport('json')
      const path = await fsApi.saveFileDialog({
        defaultPath: `neotrix-memory-${new Date().toISOString().slice(0, 10)}.json`,
        filters: [{ name: 'JSON', extensions: ['json'] }],
      })
      if (path) {
        await fsApi.writeTextFileAt(path, json)
        showNotice(`已导出记忆到 ${path}`)
      }
    } catch (e) {
      showNotice(errText(e))
    } finally {
      setDataBusy(false)
    }
  }

  const requestClearMemory = async () => {
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
      showNotice(errText(e))
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
      showNotice(errText(e))
    } finally {
      setKeyBusy(false)
    }
  }

  const requestDeleteKey = async () => {
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
      showNotice(errText(e))
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
      showNotice(errText(e))
    } finally {
      setSwitching(false)
    }
  }

  const activeProvider = () => {
    const cfg = config()
    if (!cfg) return null
    return cfg.providers.find((p) => p.model === cfg.active_model) ?? cfg.providers[0] ?? null
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

  // 删除标签全局生效，需确认（对标 Obsidian 标签管理）
  const requestDeleteTag = (name: string) => {
    setPendingDeleteTag(name)
    setModalReq({
      title: `删除标签 #${name}`,
      message: '删除后该标签将从所有会话移除，此操作不可撤销。',
      danger: true,
      confirmLabel: '删除',
    })
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
                              const next = flat[(idx + dir + flat.length) % flat.length]
                              setSection(next)
                              // 🟡 修复：切换 section 后同步移动焦点到目标 tab 按钮，
                              // 否则方向键只改状态、焦点仍留在原按钮（连续按键失能）。
                              document.getElementById(`settings-tab-${next}`)?.focus()
                            } else if (e.key === 'Home') {
                              e.preventDefault(); setSection(NAV_GROUPS[0].ids[0])
                              document.getElementById(`settings-tab-${NAV_GROUPS[0].ids[0]}`)?.focus()
                            } else if (e.key === 'End') {
                              e.preventDefault(); const flat = NAV_GROUPS.flatMap((g) => g.ids); const last = flat[flat.length - 1]; setSection(last)
                              document.getElementById(`settings-tab-${last}`)?.focus()
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
              <Show when={section() === 'general'}>
                <Show when={loading() && !config()}>
                  <div class="text-xs text-text-muted py-6 text-center">加载配置…</div>
                </Show>
                <Show when={config()}>
                  <GeneralSection
                    config={config}
                    loading={loading}
                    activeProvider={activeProvider}
                    switching={switching}
                    apiKey={apiKey}
                    setApiKey={setApiKey}
                    hasKey={hasKey}
                    keyBusy={keyBusy}
                    onSwitchProvider={switchProvider}
                    onSaveApiKey={saveApiKey}
                    onRequestDeleteKey={requestDeleteKey}
                    showNotice={showNotice}
                  />
                </Show>
              </Show>
              <Show when={section() === 'appearance'}>
                <AppearanceSection
                  fontSizePref={fontSizePref}
                  motionPref={motionPref}
                  densityPref={densityPref}
                  setFontSize={setFontSize}
                  setMotion={setMotion}
                  setDensity={setDensity}
                />
              </Show>
              <Show when={section() === 'plugins'}>
                <PluginMarketplace embedded open onClose={() => {}} />
              </Show>
              <Show when={section() === 'data'}>
                <DataSection
                  memStats={memStats}
                  dataBusy={dataBusy}
                  onExport={exportMemory}
                  onRequestClear={requestClearMemory}
                />
              </Show>
              <Show when={section() === 'tags'}>
                <TagsSection
                  newTagInput={newTagInput}
                  setNewTagInput={setNewTagInput}
                  onAdd={addTag}
                  onSeed={seedTags}
                  missingRecommended={missingRecommended}
                  onRequestDelete={requestDeleteTag}
                  showNotice={showNotice}
                />
              </Show>
              <Show when={section() === 'about'}>
                <AboutSection
                  appVersion={appVersion}
                  config={config}
                  showNotice={showNotice}
                />
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

