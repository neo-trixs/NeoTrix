// NeoTrix Frontend API - Unified Entry Point
//
// 统一导出所有前端需要的 API

// ===== 新架构：Domain Plugin System =====
export * as domain from './domain';
export { call as domainCall, list as domainList, has as domainHas, actionCount as domainActionCount } from './domain';

// ===== 新 API 模块（domain plugin 封装） =====
export * as sessionApi from './session';
export * as chatApi from './chat';
export * as agentApi from './agent';

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

// 导出 model-pool 模块
export {
  getModelPoolStatus,
  addModelProvider,
  removeModelProvider,
  updateModelProviderKey,
  checkModelProvider,
} from './model-pool';
export type { ModelPoolEntry, ModelPoolStatus } from './model-pool';

// 导出 proxy-pool 模块
export {
  getProxyPoolStatus,
  getProxyPoolSnapshot,
  addProxyNode,
  removeProxyNode,
  addSubscription,
  removeSubscription,
  setProxyStrategy,
  listProxyStrategies,
} from './proxy-pool';
export type { ProxyPoolEntry, ProxyPoolStatus, ProxyPoolSnapshot } from './proxy-pool';

// 导出 IM 模块
export {
  getImStatus,
  listChannels,
  getChannel,
  toggleChannel,
  addBot,
  removeBot,
  updateBot,
  setContextEnhancement,
  setProactiveDelivery,
  getDshMarketStatus,
  toggleDshMarket,
  updateDshMarketConfig,
  syncDshMarket,
} from './im';
export type { ChannelType, ChannelConfig, BotConfig, ImStatus, DshMarketConfig } from './im';

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

// 创建 im 命名空间对象
import * as imModule from './im';
export const im = imModule;

// 创建 model-pool 命名空间对象
import * as modelPoolModule from './model-pool';
export const modelPool = modelPoolModule;

// 创建 proxy-pool 命名空间对象
import * as proxyPoolModule from './proxy-pool';
export const proxyPool = proxyPoolModule;

// 导出 market 模块
export {
  marketStatus,
  marketSearch,
  marketGetDetail,
  marketDownload,
  marketInstall,
  marketUninstall,
  marketListInstalled,
  marketCheckUpdates,
  marketConfig,
} from './market';
export type { MarketStatus, MarketEntry, MarketSearchResult, PluginManifest } from './market';

// 创建 market 命名空间对象
import * as marketModule from './market';
export const market = marketModule;
