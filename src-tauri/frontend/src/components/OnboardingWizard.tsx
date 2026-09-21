/**
 * OnboardingWizard — 首次运行依赖检查 + 引导
 *
 * Flow: check prerequisites → show status → tips → complete
 */
import { createSignal, onMount, Show } from 'solid-js'
import { call as domainCall } from '../api/domain'
import { chatHelp } from '../api/chat'

const ONBOARDED_KEY = 'neotrix.onboarded'

interface Prerequisite {
  name: string
  installed: boolean
  version?: string
  install_hint?: string
  required: boolean
}

interface SystemInfo {
  os: string
  arch: string
  hostname?: string
}

interface OnboardingStatus {
  prerequisites: Prerequisite[]
  all_required_met: boolean
  first_run: boolean
  system: SystemInfo
}

interface Props {
  onComplete: () => void
}

export function OnboardingWizard(props: Props) {
  const [step, setStep] = createSignal<'loading' | 'check' | 'tips' | 'done'>('loading')
  const [status, setStatus] = createSignal<OnboardingStatus | null>(null)
  const [tips, setTips] = createSignal<string[]>([])

  onMount(async () => {
    try {
      const firstRun = (() => {
        try {
          return localStorage.getItem(ONBOARDED_KEY) !== '1'
        } catch {
          return true
        }
      })()
      const [sys, help] = await Promise.all([
        domainCall<{ platform: string; arch: string; hostname?: string; neotrix_version?: string }>('system', 'system_info').catch(() => null),
        chatHelp().catch(() => ''),
      ])
      const version = sys?.neotrix_version
      const s: OnboardingStatus = {
        prerequisites: [
          {
            name: 'Tauri WebView Runtime',
            installed: true,
            version,
            required: true,
          },
        ],
        all_required_met: true,
        first_run: firstRun,
        system: {
          os: sys?.platform ?? 'unknown',
          arch: sys?.arch ?? 'unknown',
          hostname: sys?.hostname,
        },
      }
      const t = help
        ? help.split('\n').map((line) => line.trim()).filter((line) => line.length > 0)
        : ['在输入框中用自然语言描述你的需求', '输入“帮助”查看支持的操作', '所有操作均经由本地领域插件执行']
      setStatus(s)
      setTips(t)
      setStep(s.first_run ? 'check' : 'done')
      if (!s.first_run) {
        props.onComplete()
      }
    } catch (e) {
      console.error('Onboarding check failed:', e)
      setStep('done')
      props.onComplete()
    }
  })

  const handleComplete = async () => {
    try {
      localStorage.setItem(ONBOARDED_KEY, '1')
    } catch (e) {
      console.error('Failed to mark onboarding complete:', e)
    }
    props.onComplete()
  }

  return (
    <Show when={step() !== 'done'}>
      <div class="onboarding-overlay">
        <div class="onboarding-wizard">
          <Show when={step() === 'loading'}>
            <div class="onboarding-loading">
              <div class="onboarding-spinner" />
              <p>正在检查系统环境...</p>
            </div>
          </Show>

          <Show when={step() === 'check' && status()}>
            <div class="onboarding-step">
              <h2>Welcome to NeoTrix</h2>
              <p class="onboarding-subtitle">AI-Native Developer Toolkit</p>

              <div class="onboarding-system">
                <span class="onboarding-badge">
                  {status()!.system.os} / {status()!.system.arch}
                </span>
                <Show when={status()!.system.hostname}>
                  <span class="onboarding-badge">{status()!.system.hostname}</span>
                </Show>
              </div>

              <div class="prereq-list">
                {status()!.prerequisites.map((p) => (
                  <div class={`prereq-item ${p.installed ? 'met' : 'unmet'}`}>
                    <span class="prereq-icon">{p.installed ? '✅' : p.required ? '❌' : '⚠️'}</span>
                    <span class="prereq-name">{p.name}</span>
                    <Show when={p.version}>
                      <span class="prereq-version">{p.version}</span>
                    </Show>
                    <Show when={!p.installed && p.install_hint}>
                      <span class="prereq-hint">{p.install_hint}</span>
                    </Show>
                    <Show when={!p.required}>
                      <span class="prereq-optional">可选</span>
                    </Show>
                  </div>
                ))}
              </div>

              <div class="onboarding-actions">
                <button
                  class="btn-primary"
                  disabled={!status()!.all_required_met}
                  onClick={() => setStep('tips')}
                >
                  {status()!.all_required_met ? '继续' : '请先安装缺失依赖'}
                </button>
              </div>
            </div>
          </Show>

          <Show when={step() === 'tips'}>
            <div class="onboarding-step">
              <h2>快速入门</h2>
              <div class="tips-list">
                {tips().map((tip) => (
                  <div class="tip-item">{tip}</div>
                ))}
              </div>
              <div class="onboarding-actions">
                <button class="btn-primary" onClick={handleComplete}>
                  开始使用
                </button>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  )
}
