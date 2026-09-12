import { createSignal, onMount, For, Show } from 'solid-js'
import { workflow } from '../api/domain'

interface WorkflowRun {
  id: string
  workflow_id: string
  status: string
  current_step: number
  progress_pct: number
  started_at: number
  results: Record<string, unknown>
}

export function WorkflowExecutionMonitor() {
  const [runs, setRuns] = createSignal<WorkflowRun[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [selectedRun, setSelectedRun] = createSignal<WorkflowRun | null>(null)

  const fetchRuns = async () => {
    setLoading(true)
    setError(null)
    try {
      // 假设有 workflow.runs() API，或者从 list 获取
      const result = await workflow.list()
      // 这里需要根据实际 API 调整
      setRuns([])
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  onMount(fetchRuns)

  const handleRefresh = () => {
    fetchRuns()
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'blue'
      case 'completed': return 'green'
      case 'failed': return 'red'
      case 'cancelled': return 'gray'
      default: return 'gray'
    }
  }

  return (
    <div class="workflow-execution-monitor">
      <div class="header">
        <h3>工作流执行监控</h3>
        <button onClick={handleRefresh} disabled={loading()}>
          {loading() ? '刷新中...' : '刷新'}
        </button>
      </div>
      
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
      
      <For each={runs()}>
        {(run) => (
          <div 
            class={`run-item ${selectedRun()?.id === run.id ? 'selected' : ''}`}
            onClick={() => setSelectedRun(run)}
          >
            <div class="run-header">
              <span class="run-id">{run.id.substring(0, 8)}</span>
              <span class={`status-badge ${getStatusColor(run.status)}`}>
                {run.status}
              </span>
            </div>
            <div class="progress-bar">
              <div 
                class="progress-fill" 
                style={{ width: `${run.progress_pct}%` }}
              />
            </div>
            <div class="run-info">
              <span>步骤: {run.current_step}</span>
              <span>进度: {run.progress_pct.toFixed(1)}%</span>
            </div>
          </div>
        )}
      </For>
      
      <Show when={runs().length === 0 && !loading()}>
        <div class="empty">暂无执行记录</div>
      </Show>
      
      <Show when={selectedRun()}>
        <div class="run-detail">
          <h4>执行详情</h4>
          <p>ID: {selectedRun()!.id}</p>
          <p>工作流: {selectedRun()!.workflow_id}</p>
          <p>状态: {selectedRun()!.status}</p>
          <p>进度: {selectedRun()!.progress_pct.toFixed(1)}%</p>
          <p>开始时间: {new Date(selectedRun()!.started_at * 1000).toLocaleString()}</p>
        </div>
      </Show>
    </div>
  )
}
