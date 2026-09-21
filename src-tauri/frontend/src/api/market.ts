// NeoTrix Frontend API — Market (市场发现引擎)
//
// 迁移到新架构：状态查询走 chatSend。

import { chatSend, extractResult } from './chat';

// ═══════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════

export interface MarketStatus {
  dsh_enabled: boolean;
  github_enabled: boolean;
  installed_count: number;
  cache_dir: string;
  plugin_dir: string;
}

export interface MarketEntry {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  category: string;
  tags: string[];
  downloads: number;
  rating: number;
  source_type: string;
  source_repo: string | null;
  icon: string | null;
  homepage: string | null;
  latest_version: string;
}

export interface MarketSearchResult {
  entries: MarketEntry[];
  total: number;
  source: string;
}

export interface PluginManifest {
  plugin: {
    id: string;
    name: string;
    version: string;
    description: string;
    author: string;
    license: string | null;
    category: string;
    tags: string[];
    min_runtime: string;
  };
  source: {
    type: string;
    repo: string | null;
    asset: string | null;
  };
  permissions: {
    network: string[];
    filesystem: string[];
    capabilities: string[];
  };
}

// ═══════════════════════════════════════════════
// API Functions
// ═══════════════════════════════════════════════

/** 获取市场状态 */
export async function marketStatus(): Promise<MarketStatus> {
  const response = await chatSend("查看市场状态")
  return extractResult<MarketStatus>(response, { dsh_enabled: false, github_enabled: false, installed_count: 0, cache_dir: '', plugin_dir: '' })
}

/** 搜索插件 */
export async function marketSearch(
  query: string,
  category?: string,
  page?: number,
  perPage?: number,
): Promise<MarketSearchResult[]> {
  const response = await chatSend(`搜索插件 ${query}`)
  const results = extractResult<MarketSearchResult[] | MarketSearchResult>(response, [])
  return Array.isArray(results) ? results : [results]
}

/** 获取插件详情 */
export async function marketGetDetail(
  pluginId: string,
  source: string,
): Promise<MarketEntry> {
  const response = await chatSend(`查看插件 ${pluginId} 详情`)
  return extractResult<MarketEntry>(response, { id: pluginId, name: pluginId, version: '', description: '', author: '', category: '', tags: [], downloads: 0, rating: 0, source_type: source, source_repo: null, icon: null, homepage: null, latest_version: '' })
}

/** 下载插件 */
export async function marketDownload(
  pluginId: string,
  source: string,
  version: string,
): Promise<string> {
  const response = await chatSend(`下载插件 ${pluginId} 版本 ${version}`)
  return response.message
}

/** 安装插件 */
export async function marketInstall(
  pluginId: string,
  source: string,
  version: string,
): Promise<PluginManifest> {
  const response = await chatSend(`安装插件 ${pluginId}`)
  return extractResult<PluginManifest>(response, {
    plugin: { id: pluginId, name: pluginId, version, description: '', author: '', license: null, category: '', tags: [], min_runtime: '' },
    source: { type: source, repo: null, asset: null },
    permissions: { network: [], filesystem: [], capabilities: [] },
  })
}

/** 卸载插件 */
export async function marketUninstall(pluginId: string): Promise<boolean> {
  const response = await chatSend(`卸载插件 ${pluginId}`)
  return response.actions.length > 0
}

/** 获取已安装插件列表 */
export async function marketListInstalled(): Promise<PluginManifest[]> {
  const response = await chatSend("查看已安装插件列表")
  return extractResult<PluginManifest[]>(response, [])
}

/** 检查更新 */
export async function marketCheckUpdates(): Promise<[string, string, string][]> {
  const response = await chatSend("检查插件更新")
  return extractResult<[string, string, string][]>(response, [])
}

/** 设置市场配置 */
export async function marketConfig(options: {
  dshEnabled?: boolean;
  dshApiEndpoint?: string;
  dshAuthToken?: string;
  githubEnabled?: boolean;
  githubToken?: string;
}): Promise<MarketStatus> {
  await chatSend("更新市场配置")
  return { dsh_enabled: options.dshEnabled ?? false, github_enabled: options.githubEnabled ?? false, installed_count: 0, cache_dir: '', plugin_dir: '' }
}
