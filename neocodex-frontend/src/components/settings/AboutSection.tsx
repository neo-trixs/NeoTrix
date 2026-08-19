/* ════════════════════════════════════════════
   components/settings/AboutSection.tsx — 关于：版本/更新/诊断
   update 状态机自管理（检查/下载/进度/重启 + unlisten 生命周期 onCleanup 释放）。
   仅依赖：appVersion 访问器 + config 访问器（诊断只读）。
   ════════════════════════════════════════════ */
import { createSignal, onCleanup, Show } from 'solid-js'
import { neocodex, errText } from '../../api'
import { listenUpdateEvents } from '../../api/system'
import type { ProviderConfig } from '../../api/types'
import { clsx } from 'clsx'
import { InfoIcon, ExpandIcon, DataIcon } from './settingsIcons'

interface Props {
  /** 应用版本访问器（父组件加载，只读展示） */
  appVersion: () => string | null
  /** 提供商配置访问器（诊断只读） */
  config: () => ProviderConfig | null
  /** 通知回调（下载完成/失败等反馈，由父组件 toast 展示） */
  showNotice?: (msg: string) => void
}

type UpdateState = 'idle' | 'checking' | 'available' | 'downloading' | 'downloaded' | 'up-to-date' | 'error'

export function AboutSection(props: Props) {
  const [updateState, setUpdateState] = createSignal<UpdateState>('idle')
  const [updateInfo, setUpdateInfo] = createSignal<{ current: string; latest: string; error: string | null } | null>(null)
  const [updateProgress, setUpdateProgress] = createSignal<{ downloaded: number; total: number } | null>(null)
  let unlistenUpdate: (() => void) | null = null

  onCleanup(() => {
    if (unlistenUpdate) {
      unlistenUpdate()
      unlistenUpdate = null
    }
  })

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
      setUpdateInfo({ current: props.appVersion() ?? '', latest: '', error: errText(e) })
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
            props.showNotice?.('新版本已下载，重启应用即可完成安装')
          },
        })
      } catch (e) {
        console.error('[AboutSection] Failed to subscribe update events:', e)
      }
    }
    setUpdateState('downloading')
    setUpdateProgress(null)
    try {
      await neocodex.downloadUpdate()
    } catch (e) {
      props.showNotice?.(errText(e))
      setUpdateState('error')
    }
  }

  const restartToInstall = async () => {
    try {
      await neocodex.restartApp()
    } catch (e) {
      props.showNotice?.(errText(e))
      setUpdateState('error')
    }
  }

  return (
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
              <div class="text-[11px] text-text-muted font-mono">v{props.appVersion() ?? '0.18.0'} · ai.neotrix.desktop</div>
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
                <span>发现新版本 <span class="font-mono text-nt-io-700">{updateInfo()?.latest}</span>（当前 v{updateInfo()?.current ?? props.appVersion() ?? '0.18.0'}）</span>
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
              <div class="text-[12.5px] text-text-primary font-medium">{props.config()?.provider_count ?? '—'} 个</div>
            </div>
            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
              <div class="text-[10px] text-text-muted mb-1">API 状态</div>
              <div class={clsx('text-[12.5px] font-medium', props.config()?.resolvable ? 'text-nt-core-700' : 'text-nt-shield-600')}>
                {props.config()?.resolvable ? '可用' : '不可达'}
              </div>
            </div>
            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
              <div class="text-[10px] text-text-muted mb-1">当前模型</div>
              <div class="text-[12.5px] text-text-primary font-mono truncate">{props.config()?.active_model ?? '—'}</div>
            </div>
            <div class="p-3 rounded-xl bg-white/40 border border-border-primary/40">
              <div class="text-[10px] text-text-muted mb-1">平台</div>
              <div class="text-[12.5px] text-text-primary">macOS</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}