import type { UnlistenFn } from '@tauri-apps/api/event'
import type { IncompatibleVersion, PolicyBlockedVersion, PreinstallLogPayload, PreinstallPlugin, PreinstallSelection } from './types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import i18next from 'i18next'
import { defineStore } from 'valtio-define'
import { hooks } from '@/config/hooks'
import { harness } from '../harness'
import { harnessUpdater } from '../harness-updater'

/** 预装安装日志在界面上的保留行数 */
const LOG_LIMIT = 200

/**
 * 后端「拦截清单」的两套前缀：其后都是清单 JSON。
 *
 * dsh 的版本兼容性与 pnpm 的发布时长策略（`minimumReleaseAge`）是两套互不相干的拦截，
 * 但形状一致：后端把清单挂在错误串上（Tauri 命令的错误通道只有字符串），前端解出来
 * 交给用户逐项授权后重跑。
 */
const INCOMPATIBLE_PREFIX = 'PLUGIN_VERSION_INCOMPATIBLE:'
const POLICY_BLOCKED_PREFIX = 'PLUGIN_POLICY_BLOCKED:'
const NO_CHANGE_PREFIX = 'PLUGIN_UPDATE_NO_CHANGE:'

/** 被拦下的清单：核心版本兼容性拒绝、pnpm 发布时长策略拒绝，或「升级没落地」。 */
export type BlockedRefusal
  = | { kind: 'incompatible', versions: IncompatibleVersion[] }
    | { kind: 'policy', versions: PolicyBlockedVersion[] }
    | { kind: 'update-hold', versions: PolicyBlockedVersion[], retryable: boolean }

/**
 * 解析后端拦截清单的错误载荷；识别不出来返回 null。
 *
 * 引导页与插件面板共用这一处解析：两边的错误通道都一样，规则必须一致。解析失败一律
 * 退回普通失败展示——绝不把读不出来的 JSON 当成可授权项（那会变成一次来路不明的授权）。
 */
export function parseBlockedRefusal(error: string): BlockedRefusal | null {
  const incompatible = parseVersions<IncompatibleVersion>(error, INCOMPATIBLE_PREFIX)
  if (incompatible)
    return { kind: 'incompatible', versions: incompatible }
  const policy = parseVersions<PolicyBlockedVersion>(error, POLICY_BLOCKED_PREFIX)
  if (policy)
    return { kind: 'policy', versions: policy }
  return parseUpdateHold(error)
}

/**
 * 「升级以 0 退出但版本没动」：只有「新版本还在发布保护期内」这一种成因有出路，后端因此
 * 连同目标版本（更新探测缓存里的 registry latest）与 `retryable` 一起给出。已经授权过还
 * 不动时 `retryable` 为 false——那是档案把来源钉死，界面就不要再给按钮。
 */
function parseUpdateHold(error: string): BlockedRefusal | null {
  if (!error.startsWith(NO_CHANGE_PREFIX))
    return null
  try {
    const payload = JSON.parse(error.slice(NO_CHANGE_PREFIX.length)) as {
      name?: unknown
      latest?: unknown
      retryable?: unknown
    }
    if (typeof payload.name !== 'string')
      return null
    const latest = typeof payload.latest === 'string' && payload.latest !== '' ? payload.latest : null
    return {
      kind: 'update-hold',
      versions: latest ? [{ name: payload.name, version: latest }] : [],
      retryable: payload.retryable === true && latest !== null,
    }
  }
  catch (err) {
    console.error(`[Harness] failed to parse ${NO_CHANGE_PREFIX} payload:`, err)
    return null
  }
}

function parseVersions<T>(error: string, prefix: string): T[] | null {
  if (!error.startsWith(prefix))
    return null
  try {
    const parsed = JSON.parse(error.slice(prefix.length)) as T[]
    return parsed.length > 0 ? parsed : null
  }
  catch (err) {
    console.error(`[Harness] failed to parse ${prefix} payload:`, err)
    return null
  }
}

/**
 * 预装插件引导模块：首次安装、老版本升级或资源清单 plugins 节内容变更后，
 * 展示推荐插件列表（可增选/减选），确认或跳过后继续启动服务。
 *
 * 与 harness 模块的分工：本模块只负责「引导页的选择与安装」，
 * 收尾（拉起服务、检查更新）通过 harness / harnessUpdater 完成。
 */
export const preinstall = defineStore({
  state: () => ({
    /** 推荐插件列表（含已安装检测） */
    plugins: [] as PreinstallPlugin[],
    loading: false,
    installing: false,
    /** 用户触发了“取消”但仍需等后端结束进程树 */
    cancelling: false,
    logs: [] as string[],
    /** 安装失败错误（区别于下面列表加载失败） */
    error: '',
    /**
     * 被核心版本兼容性拦截、等待用户逐项授权的插件版本。
     * 非空时界面展示风险提示与勾选清单（而非插件列表或错误态）。
     */
    incompatible: [] as IncompatibleVersion[],
    /**
     * 被 pnpm 发布时长策略拦下的精确版本（`minimumReleaseAge`）。
     * 非空时同样展示风险提示与勾选清单——档案已声明这些太新的版本，不授权则**每次**
     * 插件操作都会失败，用户在这里勾选授权后重跑安装即可解开。
     */
    policyBlocked: [] as PolicyBlockedVersion[],
    /** 拉取预装插件列表失败（区别于空列表；UI 据此展示错误态 + 重试） */
    loadError: '',
    /** 是否为首次安装引导（决定默认勾选策略）：boot 流程进入时为 true，侧边栏手动打开为 false */
    isFirstTime: true,
  }),
  actions: {
    /** 拉取预装插件列表（含已安装检测），供首次启动引导界面渲染 */
    async load(): Promise<PreinstallPlugin[]> {
      if (this.loading)
        return this.plugins
      this.loading = true
      try {
        this.plugins = await invoke<PreinstallPlugin[]>('get_preinstall_plugins')
        // 成功加载后清除历史加载错误，避免残留错误态遮蔽新列表
        this.loadError = ''
      }
      catch (err) {
        console.error('[Harness] failed to load preinstall plugins:', err)
        // 记录错误而非伪装空列表：UI 据此展示错误态与重试按钮
        this.loadError = String(err)
      }
      finally {
        this.loading = false
      }
      return this.plugins
    },

    /** 预装安装日志流：dsh plugin 进程输出逐行追加 */
    async listenLog(): Promise<UnlistenFn> {
      return listen<PreinstallLogPayload>('preinstall-log', (e) => {
        this.logs = [...this.logs, e.payload.line].slice(-LOG_LIMIT)
      })
    },

    /**
     * 确认安装/卸载预装插件：流式日志，完成后继续启动服务。
     *
     * 前端传入 diff 结果（installIds = 新增勾选需安装；uninstallIds = 取消勾选需卸载），
     * 无变化时两者均为空，后端直接标记完成。
     */
    async confirm(input: PreinstallSelection | string[]) {
      // 兼容旧调用方（直接传数组）与新调用方（传 {installIds, uninstallIds}）
      const installIds = Array.isArray(input) ? input : (input.installIds ?? [])
      const uninstallIds = Array.isArray(input) ? [] : (input.uninstallIds ?? [])
      if (this.installing || (installIds.length === 0 && uninstallIds.length === 0))
        return
      this.installing = true
      this.error = ''
      this.incompatible = []
      this.logs = []
      let unlisten: UnlistenFn | null = null
      try {
        unlisten = await this.listenLog()
        await invoke('install_preinstall_plugins', { installIds, uninstallIds })
        // 后端装完已把服务停掉，这里在日志面板讲清接下来的重启（issue #48），
        // 避免用户把"插件安装后的自动重启"误认为崩溃/故障。
        this.logs = [...this.logs, i18next.t('preinstall.restarting_hint')].slice(-LOG_LIMIT)
        await this.continueStartup()
      }
      catch (err) {
        console.error('[Harness] preinstall failed:', err)
        const error = String(err)
        const refusal = parseBlockedRefusal(error)
        if (refusal?.kind === 'incompatible') {
          // 不是故障而是待授权清单：交给界面展示风险与勾选，用户授权后重跑安装
          this.incompatible = refusal.versions
        }
        else if (refusal?.kind === 'policy') {
          // 同上，但拦下它的是 pnpm 的发布时长策略：清单同样交给界面勾选授权
          this.policyBlocked = refusal.versions
        }
        else {
          this.error = error.startsWith('NETWORK_ERROR:')
            ? i18next.t('preinstall.network_error')
            : error
        }
      }
      finally {
        unlisten?.()
        this.installing = false
        this.cancelling = false
      }
    },

    /**
     * 授予被核心拒绝的插件精确版本豁免，成功后由界面重跑安装。
     *
     * 只提交用户在风险提示中勾选的条目；授权失败按普通安装失败展示（错误态 + 重试），
     * 届时重试会再次触发拒绝、重新给出勾选清单。
     */
    async allowIncompatible(versions: IncompatibleVersion[]): Promise<boolean> {
      if (this.installing || versions.length === 0)
        return false
      try {
        await invoke('allow_plugin_versions', { versions })
        this.incompatible = []
        return true
      }
      catch (err) {
        console.error('[Harness] granting plugin version exemption failed:', err)
        this.incompatible = []
        this.error = String(err)
        return false
      }
    },

    /**
     * 授予被 pnpm 发布时长策略拦下的精确版本豁免，成功后由界面重跑安装。
     *
     * 与 [`allowIncompatible`] 是两套互不相干的授权（一个写档案的 `compatibility.json`、
     * 一个写 `pnpm-workspace.yaml` 的 `minimumReleaseAgeExclude`），但形状一致：只提交
     * 用户勾选的精确 `包名@版本`，失败按普通安装失败展示并回到可重试态。
     */
    async allowPolicyVersions(versions: PolicyBlockedVersion[]): Promise<boolean> {
      if (this.installing || versions.length === 0)
        return false
      try {
        await invoke('allow_plugin_policy_versions', { versions })
        this.policyBlocked = []
        return true
      }
      catch (err) {
        console.error('[Harness] granting release-age exemption failed:', err)
        this.policyBlocked = []
        this.error = String(err)
        return false
      }
    },

    /**
     * 取消正在进行的预装插件安装：网络抖动/拉包限流（429）时可能长时间卡在
     * pnpm 重试；调用后端强杀插件安装进程树，回到可重试的选择态。
     */
    async cancel() {
      if (!this.installing || this.cancelling)
        return
      // 后端结束进程树导致 `install_preinstall_plugins` 提前返回并进入 catch，
      // 通过 installing=false 让其回到列表态而不是报错态。
      this.cancelling = true
      // 一次性监听：先挂事件（拿到注销函数再 invoke），finally 里注销，
      // 避免每次取消都永久注册一个 `preinstall-cancelled` 监听（泄漏）。
      let unlisten: (() => void) | undefined
      try {
        unlisten = await listen<unknown>('preinstall-cancelled', () => {
          this.installing = false
          this.cancelling = false
        })
        await invoke('cancel_preinstall_plugins')
      }
      catch (err) {
        console.error('[Harness] cancel preinstall failed:', err)
        this.cancelling = false
      }
      finally {
        unlisten?.()
      }
    },

    /** 跳过预装插件引导：记录状态后继续启动服务 */
    async skip() {
      if (this.installing)
        return
      try {
        await invoke('skip_preinstall_plugins')
        await this.continueStartup()
      }
      catch (err) {
        console.error('[Harness] skip preinstall failed:', err)
        this.error = String(err)
      }
    },

    /** 预装引导结束后的收尾：拉起服务等待就绪，并静默检查更新 */
    async continueStartup() {
      await harness.launchAndWait()
      void harnessUpdater.checkForUpdate()
    },

    /**
     * 从侧边栏重新打开预装插件引导：可重新选择/安装推荐插件。
     * 关闭引导（确定/跳过）后回到正常启动流程，服务若在运行则保持原状态。
     */
    async open() {
      if (this.installing)
        return
      void hooks['config.dialog.hidden'].trigger()
      this.error = ''
      // 重新打开引导时清掉上次留下的拦截清单，否则会直接落在授权对话框上
      this.incompatible = []
      this.policyBlocked = []
      this.logs = []
      // 侧边栏手动打开：非首次安装，默认勾选策略为「仅已安装」
      this.isFirstTime = false
      harness.status = 'preinstall'
      await this.load()
    },
  },
})
