import { call } from './client'
import type { BackgroundTask } from './types'

/* ════════════════════════════════════════════
   api/tasks.ts — 定时后台任务
   对应 background_cmds.rs
   ════════════════════════════════════════════ */

export function listBackgroundTasks(): Promise<BackgroundTask[]> {
  return call('list_background_tasks', {})
}

export function createBackgroundTask(name: string, prompt: string, schedule: string): Promise<BackgroundTask> {
  return call('create_background_task', { name, prompt, schedule })
}

export function pauseBackgroundTask(id: string): Promise<void> {
  return call('pause_background_task', { id })
}

export function resumeBackgroundTask(id: string): Promise<void> {
  return call('resume_background_task', { id })
}

export function deleteBackgroundTask(id: string): Promise<void> {
  return call('delete_background_task', { id })
}

export function runBackgroundTaskNow(id: string): Promise<string> {
  return call('run_background_task_now', { id })
}
