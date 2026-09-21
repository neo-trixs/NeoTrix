import { call as domainCall } from './domain'
import { ApiError } from './client'
import type { BackgroundTask } from './types'

/* ════════════════════════════════════════════
   api/tasks.ts — 定时后台任务
   chat-first 迁移后经 ai_orchestration 域插件路由。
   后端动作：submit/get/list/cancel_background_task；
   pause / resume / run_now 后端不存在，明确抛出 NOT_IMPLEMENTED。
   ════════════════════════════════════════════ */

interface BackendTask {
  id: string
  description: string
  status: string
  created_at?: string
  started_at?: string | null
  completed_at?: string | null
  progress?: number
  result?: string | null
  error?: string | null
}

function adaptTask(t: BackendTask): BackgroundTask {
  return {
    id: t.id,
    name: t.description,
    prompt: t.description,
    schedule: '',
    last_run: null,
    next_run: null,
    status: t.status,
    runs: [],
  }
}

function notImplemented(op: string): ApiError {
  return new ApiError(`后台任务操作 ${op} 后端未实现`, 'NOT_IMPLEMENTED')
}

export async function listBackgroundTasks(): Promise<BackgroundTask[]> {
  const tasks = await domainCall<BackendTask[]>('ai_orchestration', 'list_background_tasks')
  return tasks.map(adaptTask)
}

export async function createBackgroundTask(name: string, prompt: string, schedule: string): Promise<BackgroundTask> {
  const description = schedule ? `${name}: ${prompt} @${schedule}` : `${name}: ${prompt}`
  const task = await domainCall<BackendTask>('ai_orchestration', 'submit_background_task', { description })
  return adaptTask(task)
}

export async function pauseBackgroundTask(_id: string): Promise<void> {
  throw notImplemented('pause')
}

export async function resumeBackgroundTask(_id: string): Promise<void> {
  throw notImplemented('resume')
}

export async function deleteBackgroundTask(id: string): Promise<void> {
  await domainCall('ai_orchestration', 'cancel_background_task', { task_id: id })
}

export async function runBackgroundTaskNow(_id: string): Promise<string> {
  throw notImplemented('run_now')
}
