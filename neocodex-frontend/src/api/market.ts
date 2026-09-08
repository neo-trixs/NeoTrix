// NeoTrix Frontend API — Market (市场发现引擎)
//
// 统一市场搜索、安装、卸载功能。

import { invoke } from '@tauri-apps/api/core';

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
  return invoke('market_status');
}

/** 搜索插件 */
export async function marketSearch(
  query: string,
  category?: string,
  page?: number,
  perPage?: number,
): Promise<MarketSearchResult[]> {
  return invoke('market_search', {
    query,
    category: category ?? null,
    page: page ?? 1,
    per_page: perPage ?? 20,
  });
}

/** 获取插件详情 */
export async function marketGetDetail(
  pluginId: string,
  source: string,
): Promise<MarketEntry> {
  return invoke('market_get_detail', {
    pluginId,
    source,
  });
}

/** 下载插件 */
export async function marketDownload(
  pluginId: string,
  source: string,
  version: string,
): Promise<string> {
  return invoke('market_download', {
    pluginId,
    source,
    version,
  });
}

/** 安装插件 */
export async function marketInstall(
  pluginId: string,
  source: string,
  version: string,
): Promise<PluginManifest> {
  return invoke('market_install', {
    pluginId,
    source,
    version,
  });
}

/** 卸载插件 */
export async function marketUninstall(pluginId: string): Promise<boolean> {
  return invoke('market_uninstall', {
    pluginId,
  });
}

/** 获取已安装插件列表 */
export async function marketListInstalled(): Promise<PluginManifest[]> {
  return invoke('market_list_installed');
}

/** 检查更新 */
export async function marketCheckUpdates(): Promise<[string, string, string][]> {
  return invoke('market_check_updates');
}

/** 设置市场配置 */
export async function marketConfig(options: {
  dshEnabled?: boolean;
  dshApiEndpoint?: string;
  dshAuthToken?: string;
  githubEnabled?: boolean;
  githubToken?: string;
}): Promise<MarketStatus> {
  return invoke('market_config', {
    dsh_enabled: options.dshEnabled ?? null,
    dsh_api_endpoint: options.dshApiEndpoint ?? null,
    dsh_auth_token: options.dshAuthToken ?? null,
    github_enabled: options.githubEnabled ?? null,
    github_token: options.githubToken ?? null,
  });
}
