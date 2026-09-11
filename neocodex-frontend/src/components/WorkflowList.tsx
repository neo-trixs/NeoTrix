import { onMount, For, Show } from 'solid-js'
import { createWorkflowStore } from '../stores/workflow'

export function WorkflowList() {
  const store = createWorkflowStore()

  onMount(() => {
    store.load()
  })

  const handleCreate = async () => {
    const name = prompt('工作流名称')
    if (name) {
      await store.create({ name, description: '', steps: [], tags: [] })
      await store.load()
    }
  }

  const handleRun = async (id: string) => {
    await store.run(id)
  }

  const handleDelete = async (id: string) => {
    if (confirm('确定删除？')) {
      await store.remove(id)
      await store.load()
    }
  }

  return (
    <div class="workflow-list">
      <div class="header">
        <h2>工作流</h2>
        <button onClick={handleCreate}>新建</button>
      </div>

      <For each={store.workflows()}>
        {(workflow) => (
          <div class="workflow-item">
            <div class="info">
              <h3>{workflow.name}</h3>
              <p>{workflow.description || '无描述'}</p>
              <span>{workflow.steps?.length || 0} 步骤</span>
            </div>
            <div class="actions">
              <button onClick={() => handleRun(workflow.id)}>运行</button>
              <button onClick={() => handleDelete(workflow.id)}>删除</button>
            </div>
          </div>
        )}
      </For>

      <Show when={store.loading()}>
        <div class="loading">加载中...</div>
      </Show>
    </div>
  )
}
