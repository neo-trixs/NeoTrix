/* ════════════════════════════════════════════
   api/index.ts — 统一 IPC 层 barrel
   组件/路由统一从 './api' 导入，禁止直接 import '@tauri-apps/api/core'
   ════════════════════════════════════════════ */

export * as computer from './computer'
export * as cowork from './cowork'
export * as geo from './geo'
export * as memory from './memory'
export * as neocodex from './neocodex'
export * as plugins from './plugins'
export * as system from './system'
export * as tasks from './tasks'
export * as unified from './unified'
export * from './client'
export * from './types'
