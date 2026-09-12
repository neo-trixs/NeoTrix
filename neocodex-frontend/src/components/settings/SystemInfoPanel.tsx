import { createSignal, onMount, Show } from 'solid-js'
import { system } from '../../api/domain'

interface SystemInfo {
  platform: string
  arch: string
  hostname: string
  cpu_count: number
  memory_total: number
}

export function SystemInfoPanel() {
  const [info, setInfo] = createSignal<SystemInfo | null>(null)
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)

  const fetchInfo = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await system.systemInfo()
      setInfo(result)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchInfo)

  const formatMemory = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024)
    return `${gb.toFixed(2)} GB`
  }

  return (
    <div class="system-info-panel">
      <div class="header">
        <h3>系统信息</h3>
        <button onClick={fetchInfo} disabled={loading()}>
          {loading() ? '刷新中...' : '刷新'}
        </button>
      </div>

      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>

      <Show when={info()}>
        <div class="info-grid">
          <div class="info-item">
            <span class="label">平台</span>
            <span class="value">{info()!.platform}</span>
          </div>
          <div class="info-item">
            <span class="label">架构</span>
            <span class="value">{info()!.arch}</span>
          </div>
          <div class="info-item">
            <span class="label">主机名</span>
            <span class="value">{info()!.hostname}</span>
          </div>
          <div class="info-item">
            <span class="label">CPU 核心</span>
            <span class="value">{info()!.cpu_count}</span>
          </div>
          <div class="info-item">
            <span class="label">内存总量</span>
            <span class="value">{formatMemory(info()!.memory_total)}</span>
          </div>
        </div>
      </Show>
    </div>
  )
}
