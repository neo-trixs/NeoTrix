import { createSignal } from 'solid-js'
import * as domain from '../api/domain'

export type { Workflow, WorkflowStep, WorkflowRun } from '../api/domain'

export function createWorkflowStore() {
  const [workflows, setWorkflows] = createSignal<domain.Workflow[]>([])
  const [loading, setLoading] = createSignal(false)
  const [error, setError] = createSignal<string | null>(null)
  const [activeRun, setActiveRun] = createSignal<domain.WorkflowRun | null>(null)

  async function load() {
    setLoading(true)
    setError(null)
    try {
      setWorkflows(await domain.workflow.list())
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  async function create(params: { name: string; description?: string; steps?: domain.WorkflowStep[]; tags?: string[] }) {
    const wf = await domain.workflow.create(params)
    setWorkflows((prev) => [wf, ...prev])
    return wf
  }

  async function run(id: string) {
    const run = await domain.workflow.run(id)
    setActiveRun(run)
    return run
  }

  async function remove(id: string) {
    await domain.workflow.delete(id)
    setWorkflows((prev) => prev.filter((w) => w.id !== id))
  }

  return {
    workflows,
    loading,
    error,
    activeRun,
    load,
    create,
    run,
    remove,
  }
}

export type WorkflowStore = ReturnType<typeof createWorkflowStore>

export const workflowStore = createWorkflowStore()
