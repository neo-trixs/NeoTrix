//! Market Module — 市场发现引擎
//!
//! 统一多源插件发现、下载和安装。

pub mod schema;
pub mod dsh_discoverer;
pub mod github_discoverer;
pub mod commands;

use std::collections::HashMap;
use std::path::PathBuf;

use schema::{MarketEntry, MarketSearchResult, PluginManifest};

/// 市场配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketConfig {
    /// DSH 市场启用
    pub dsh_enabled: bool,
    /// DSH API 端点
    pub dsh_api_endpoint: String,
    /// DSH 认证令牌
    pub dsh_auth_token: Option<String>,
    /// GitHub 启用
    pub github_enabled: bool,
    /// GitHub Token
    pub github_token: Option<String>,
    /// 缓存目录
    pub cache_dir: PathBuf,
    /// 插件安装目录
    pub plugin_dir: PathBuf,
}

impl Default for MarketConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            dsh_enabled: true,
            dsh_api_endpoint: "https://dshfind.com/api".into(),
            dsh_auth_token: None,
            github_enabled: true,
            github_token: None,
            cache_dir: home.join(".neotrix").join("market").join("cache"),
            plugin_dir: home.join(".neotrix").join("plugins"),
        }
    }
}

/// 市场发现引擎
pub struct MarketEngine {
    config: MarketConfig,
    dsh: dsh_discoverer::DshMarketDiscoverer,
    github: github_discoverer::GitHubDiscoverer,
    installed: HashMap<String, PluginManifest>,
}

impl MarketEngine {
    /// 创建市场引擎
    pub fn new(config: MarketConfig) -> Self {
        let dsh = dsh_discoverer::DshMarketDiscoverer::new(
            dsh_discoverer::DshMarketConfig {
                api_endpoint: config.dsh_api_endpoint.clone(),
                auth_token: config.dsh_auth_token.clone(),
                cache_ttl_secs: 3600,
            },
        );

        let github = github_discoverer::GitHubDiscoverer::new(
            github_discoverer::GitHubConfig {
                token: config.github_token.clone(),
                api_base: "https://api.github.com".into(),
            },
        );

        Self {
            config,
            dsh,
            github,
            installed: HashMap::new(),
        }
    }

    /// 加载已安装插件
    pub fn load_installed(&mut self) -> Result<(), String> {
        self.installed.clear();

        if !self.config.plugin_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.config.plugin_dir)
            .map_err(|e| format!("Read plugin dir: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 尝试加载 plugin.toml
                let toml_path = path.join("plugin.toml");
                if toml_path.exists() {
                    if let Ok(manifest) = PluginManifest::from_file(&toml_path) {
                        self.installed.insert(manifest.plugin.id.clone(), manifest);
                    }
                }
                // 兼容旧格式 manifest.json
                let json_path = path.join("manifest.json");
                if json_path.exists() {
                    if let Ok(manifest) = PluginManifest::from_json_file(&json_path) {
                        self.installed.insert(manifest.plugin.id.clone(), manifest);
                    }
                }
            }
        }

        Ok(())
    }

    /// 统一搜索 (多源)
    pub async fn search(
        &self,
        query: &str,
        category: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<Vec<MarketSearchResult>, String> {
        let mut results = vec![];

        // DSH 市场搜索
        if self.config.dsh_enabled {
            match self.dsh.search(query, category, page, per_page).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("[market] DSH search error: {e}"),
            }
        }

        // GitHub 搜索
        if self.config.github_enabled {
            match self.github.search(query, page, per_page).await {
                Ok(result) => results.push(result),
                Err(e) => eprintln!("[market] GitHub search error: {e}"),
            }
        }

        Ok(results)
    }

    /// 合并搜索结果 (去重+排序)
    pub fn merge_results(&self, results: Vec<MarketSearchResult>) -> Vec<MarketEntry> {
        let mut merged: HashMap<String, MarketEntry> = HashMap::new();

        for result in results {
            for entry in result.entries {
                // 保留评分最高的版本
                match merged.get(&entry.id) {
                    Some(existing) if existing.rating >= entry.rating => {}
                    _ => {
                        merged.insert(entry.id.clone(), entry);
                    }
                }
            }
        }

        let mut entries: Vec<MarketEntry> = merged.into_values().collect();
        entries.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        entries
    }

    /// 获取插件详情
    pub async fn get_detail(&self, plugin_id: &str, source: &str) -> Result<MarketEntry, String> {
        match source {
            "dsh-market" => self.dsh.get_detail(plugin_id).await,
            "github" => self.github.get_detail(plugin_id).await,
            _ => Err(format!("Unknown source: {}", source)),
        }
    }

    /// 下载插件
    pub async fn download(
        &self,
        plugin_id: &str,
        source: &str,
        version: &str,
    ) -> Result<PathBuf, String> {
        let dest_dir = self.config.cache_dir.join(plugin_id);
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| format!("Create cache dir: {e}"))?;

        match source {
            "dsh-market" => {
                // 获取资产列表
                let assets = self.dsh.get_assets(plugin_id, version).await?;
                if let Some(asset) = assets.first() {
                    self.dsh
                        .download(plugin_id, version, &asset.name, &dest_dir)
                        .await
                } else {
                    Err("No assets found".into())
                }
            }
            "github" => {
                // 获取 release 资产
                let assets = self.github.get_releases(plugin_id).await?;
                if let Some(asset) = assets.first() {
                    self.github
                        .download_asset(&asset.url, &dest_dir, &asset.name)
                        .await
                } else {
                    Err("No release assets found".into())
                }
            }
            _ => Err(format!("Unknown source: {}", source)),
        }
    }

    /// 检查更新
    pub async fn check_updates(&self) -> Result<Vec<(String, String, String)>, String> {
        let mut updates = vec![];

        for (plugin_id, manifest) in &self.installed {
            let source = &manifest.source;
            match source.source_type {
                schema::PluginSourceType::DshMarket => {
                    if let Some(repo) = &source.repo {
                        match self.dsh.check_updates(&[(repo.clone(), manifest.plugin.version.clone())]).await {
                            Ok(u) => updates.extend(u),
                            Err(e) => eprintln!("[market] Check updates for {}: {e}", plugin_id),
                        }
                    }
                }
                schema::PluginSourceType::GitHub => {
                    if let Some(repo) = &source.repo {
                        match self.github.check_updates(&[(repo.clone(), manifest.plugin.version.clone())]).await {
                            Ok(u) => updates.extend(u),
                            Err(e) => eprintln!("[market] Check updates for {}: {e}", plugin_id),
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(updates)
    }

    /// 安装插件
    pub async fn install(
        &mut self,
        manifest: PluginManifest,
        source_path: &PathBuf,
    ) -> Result<(), String> {
        let plugin_id = &manifest.plugin.id;
        let dest_dir = self.config.plugin_dir.join(plugin_id);

        // 创建目标目录
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| format!("Create plugin dir: {e}"))?;

        // 复制文件
        if source_path.is_dir() {
            copy_dir_recursive(source_path, &dest_dir)
                .map_err(|e| format!("Copy plugin files: {e}"))?;
        } else {
            let dest_file = dest_dir.join(
                source_path
                    .file_name()
                    .unwrap_or_default(),
            );
            std::fs::copy(source_path, &dest_file)
                .map_err(|e| format!("Copy plugin file: {e}"))?;
        }

        // 写入 plugin.toml
        let toml_content = manifest.to_toml()
            .map_err(|e| format!("Serialize manifest: {e}"))?;
        std::fs::write(dest_dir.join("plugin.toml"), toml_content)
            .map_err(|e| format!("Write plugin.toml: {e}"))?;

        // 更新已安装列表
        self.installed.insert(plugin_id.clone(), manifest);

        Ok(())
    }

    /// 卸载插件
    pub fn uninstall(&mut self, plugin_id: &str) -> Result<(), String> {
        let dest_dir = self.config.plugin_dir.join(plugin_id);

        if dest_dir.exists() {
            std::fs::remove_dir_all(&dest_dir)
                .map_err(|e| format!("Remove plugin dir: {e}"))?;
        }

        self.installed.remove(plugin_id);
        Ok(())
    }

    /// 获取已安装插件列表
    pub fn list_installed(&self) -> Vec<&PluginManifest> {
        self.installed.values().collect()
    }
}

/// 递归复制目录
fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dest)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}
