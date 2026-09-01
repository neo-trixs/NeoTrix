// NeoTrix Frontend API - Unified Entry Point
//
// 统一导出所有前端需要的 API

// ===== 新架构：Domain Plugin System =====
export * as domain from './domain';
export { call as domainCall, list as domainList, has as domainHas } from './domain';

// ===== 旧架构导出（保留向后兼容） =====
export * from './unified';

// 导出客户端工具函数
export { errText } from './client';

// 导出 harness 相关
export {
  harnessExecute,
  harnessRun,
  harnessApiMap,
  harnessToolCatalog,
  harnessResolve,
  harnessRouterStatus,
  harnessRouterSetProvider,
  harnessSandboxStatus,
  harnessThreadCreate,
} from './harness';

// 导出 neocodex 流式函数
export {
  listSessions,
  createSession,
  deleteSession,
  sendMessageStream,
  stopStream,
  subscribeStream,
} from './neocodex';

// 导出 system 模块
export {
  windowMinimize,
  windowMaximize,
  windowClose,
  projectTree,
  readFile,
  writeFile,
  listenUpdateEvents,
} from './system';

// 导出 kb 模块
export {
  kbDocIngest,
  kbDocList,
  kbDocDelete,
  kbDocReindex,
} from './kb';

// 创建 neocodex 命名空间对象（向后兼容）
import * as neocodexModule from './neocodex';
export const neocodex = neocodexModule;

// 创建 system 命名空间对象（向后兼容）
import * as systemModule from './system';
export const system = systemModule;

// 创建 unified 命名空间对象（向后兼容）
import { unifiedApi } from './unified';
export const unified = unifiedApi;

// 创建 harness 命名空间对象（向后兼容）
import * as harnessModule from './harness';
export const harness = harnessModule;

// 创建 plugins 命名空间对象（向后兼容）
import * as pluginsModule from './plugins';
export const plugins = pluginsModule;

// 创建 tasks 命名空间对象（向后兼容）
import * as tasksModule from './tasks';
export const tasks = tasksModule;

// 创建 memory 命名空间对象（向后兼容）
import * as memoryModule from './memory';
export const memory = memoryModule;

// 创建 fs 命名空间对象（向后兼容）
import * as fsModule from './fs';
export const fs = fsModule;
