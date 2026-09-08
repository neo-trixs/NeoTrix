//! DSH Market Discoverer — dshfind.com 市场发现器
//!
//! 从 DSH 市场搜索和下载插件。

use serde::{Deserialize, Serialize};

use super::schema::{MarketEntry, MarketSearchResult, PluginAsset, PluginManifest};

/// DSH 市场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DshMarketConfig {
    pub api_endpoint: String,
    pub auth_token: Option<String>,
    pub cache_ttl_secs: u64,
}

impl Default for DshMarketConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://dshfind.com/api".into(),
            auth_token: None,
            cache_ttl_secs: 3600,
        }
    }
}

/// DSH 市场发现器
pub struct DshMarketDiscoverer {
    config: DshMarketConfig,
    client: reqwest::Client,
}

impl DshMarketDiscoverer {
    pub fn new(config: DshMarketConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// 搜索插件
    pub async fn search(
        &self,
        query: &str,
        category: Option<&str>,
        page: usize,
        per_page: usize,
    ) -> Result<MarketSearchResult, String> {
        let mut url = format!(
            "{}/plugins/search?q={}&page={}&per_page={}",
            self.config.api_endpoint, query, page, per_page
        );

        if let Some(cat) = category {
            url.push_str(&format!("&category={}", cat));
        }

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("DSH market request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("DSH market error: {}", response.status()));
        }

        let data: DshSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse DSH response: {e}"))?;

        Ok(MarketSearchResult {
            entries: data
                .plugins
                .into_iter()
                .map(|p| {
                    let version = p.version.clone();
                    MarketEntry {
                    id: p.id,
                    name: p.name,
                    version: version.clone(),
                    description: p.description,
                    author: p.author,
                    category: p.category,
                    tags: p.tags,
                    downloads: p.downloads,
                    rating: p.rating,
                    source_type: "dsh-market".into(),
                    source_repo: p.repo,
                    icon: p.icon,
                    homepage: p.homepage,
                    latest_version: version,
                }})
                .collect(),
            total: data.total,
            page,
            per_page,
            source: "dsh-market".into(),
        })
    }

    /// 获取插件详情
    pub async fn get_detail(&self, plugin_id: &str) -> Result<MarketEntry, String> {
        let url = format!(
            "{}/plugins/{}",
            self.config.api_endpoint, plugin_id
        );

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("DSH market request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("DSH market error: {}", response.status()));
        }

        let data: DshPluginDetail = response
            .json()
            .await
            .map_err(|e| format!("Parse DSH response: {e}"))?;

        let version = data.version.clone();

        Ok(MarketEntry {
            id: data.id,
            name: data.name,
            version: version.clone(),
            description: data.description,
            author: data.author,
            category: data.category,
            tags: data.tags,
            downloads: data.downloads,
            rating: data.rating,
            source_type: "dsh-market".into(),
            source_repo: data.repo,
            icon: data.icon,
            homepage: data.homepage,
            latest_version: version,
        })
    }

    /// 获取插件资产列表
    pub async fn get_assets(&self, plugin_id: &str, version: &str) -> Result<Vec<PluginAsset>, String> {
        let url = format!(
            "{}/plugins/{}/versions/{}/assets",
            self.config.api_endpoint, plugin_id, version
        );

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("DSH market request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("DSH market error: {}", response.status()));
        }

        let data: Vec<DshAsset> = response
            .json()
            .await
            .map_err(|e| format!("Parse DSH response: {e}"))?;

        Ok(data
            .into_iter()
            .map(|a| PluginAsset {
                name: a.name,
                url: a.url,
                size: a.size,
                sha256: a.sha256,
                platform: a.platform,
            })
            .collect())
    }

    /// 下载插件
    pub async fn download(
        &self,
        plugin_id: &str,
        version: &str,
        asset_name: &str,
        dest_dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, String> {
        let url = format!(
            "{}/plugins/{}/versions/{}/assets/{}/download",
            self.config.api_endpoint, plugin_id, version, asset_name
        );

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("DSH download failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("DSH download error: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Read download: {e}"))?;

        // 确保目标目录存在
        std::fs::create_dir_all(dest_dir)
            .map_err(|e| format!("Create dir: {e}"))?;

        let dest_path = dest_dir.join(asset_name);
        std::fs::write(&dest_path, &bytes)
            .map_err(|e| format!("Write file: {e}"))?;

        Ok(dest_path)
    }

    /// 检查更新
    pub async fn check_updates(
        &self,
        installed: &[(String, String)], // (plugin_id, current_version)
    ) -> Result<Vec<(String, String, String)>, String> {
        // TODO: 实现批量更新检查
        // 目前返回空列表
        Ok(vec![])
    }
}

// ═══════════════════════════════════════════════
// DSH API 响应类型
// ═══════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct DshSearchResponse {
    plugins: Vec<DshPluginSummary>,
    total: usize,
    page: usize,
    per_page: usize,
}

#[derive(Debug, Deserialize)]
struct DshPluginSummary {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    category: String,
    tags: Vec<String>,
    downloads: u64,
    rating: f32,
    repo: Option<String>,
    icon: Option<String>,
    homepage: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DshPluginDetail {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    category: String,
    tags: Vec<String>,
    downloads: u64,
    rating: f32,
    repo: Option<String>,
    icon: Option<String>,
    homepage: Option<String>,
    assets: Vec<DshAsset>,
}

#[derive(Debug, Deserialize)]
struct DshAsset {
    name: String,
    url: String,
    size: u64,
    sha256: Option<String>,
    platform: Option<String>,
}
