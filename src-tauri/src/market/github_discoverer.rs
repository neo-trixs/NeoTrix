//! GitHub Discoverer — GitHub 仓库搜索和下载
//!
//! 从 GitHub 搜索 NeoTrix 插件仓库并下载。

use serde::{Deserialize, Serialize};

use super::schema::{MarketEntry, MarketSearchResult, PluginAsset};

/// GitHub 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub api_base: String,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            api_base: "https://api.github.com".into(),
        }
    }
}

/// GitHub 发现器
pub struct GitHubDiscoverer {
    config: GitHubConfig,
    client: reqwest::Client,
}

impl GitHubDiscoverer {
    pub fn new(config: GitHubConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("NeoTrix-Market/0.21.0")
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// 搜索插件仓库
    pub async fn search(
        &self,
        query: &str,
        page: usize,
        per_page: usize,
    ) -> Result<MarketSearchResult, String> {
        let search_query = format!("{} neotrix-plugin in:readme", query);
        let url = format!(
            "{}/search/repositories?q={}&page={}&per_page={}&sort=stars&order=desc",
            self.config.api_base,
            urlencoding::encode(&search_query),
            page,
            per_page
        );

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        let data: GitHubSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse GitHub response: {e}"))?;

        let entries: Vec<MarketEntry> = data
            .items
            .into_iter()
            .filter_map(|repo| {
                // 尝试从仓库描述和主题推断插件信息
                let tags = repo.topics.clone();
                let category = if tags.contains(&"im-channel".to_string()) {
                    "im-channel".into()
                } else if tags.contains(&"tool".to_string()) {
                    "tool".into()
                } else if tags.contains(&"memory".to_string()) {
                    "memory".into()
                } else {
                    "other".into()
                };

                Some(MarketEntry {
                    id: repo.full_name.clone(),
                    name: repo.name.clone(),
                    version: repo.default_branch.clone().unwrap_or_else(|| "main".into()),
                    description: repo.description.unwrap_or_default(),
                    author: repo.owner.login.clone(),
                    category,
                    tags,
                    downloads: repo.forks_count as u64,
                    rating: repo.stargazers_count as f32 / 100.0, // 简化评分
                    source_type: "github".into(),
                    source_repo: Some(repo.full_name),
                    icon: None,
                    homepage: repo.homepage,
                    latest_version: "latest".into(),
                })
            })
            .collect();

        Ok(MarketSearchResult {
            total: data.total_count,
            entries,
            page,
            per_page,
            source: "github".into(),
        })
    }

    /// 获取仓库详情
    pub async fn get_detail(&self, repo_name: &str) -> Result<MarketEntry, String> {
        let url = format!("{}/repos/{}", self.config.api_base, repo_name);

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        let repo: GitHubRepo = response
            .json()
            .await
            .map_err(|e| format!("Parse GitHub response: {e}"))?;

        Ok(MarketEntry {
            id: repo.full_name.clone(),
            name: repo.name,
            version: repo.default_branch.unwrap_or_else(|| "main".into()),
            description: repo.description.unwrap_or_default(),
            author: repo.owner.login,
            category: "other".into(),
            tags: repo.topics,
            downloads: repo.forks_count as u64,
            rating: repo.stargazers_count as f32 / 100.0,
            source_type: "github".into(),
            source_repo: Some(repo.full_name),
            icon: None,
            homepage: repo.homepage,
            latest_version: "latest".into(),
        })
    }

    /// 获取仓库 release 资产
    pub async fn get_releases(&self, repo_name: &str) -> Result<Vec<PluginAsset>, String> {
        let url = format!(
            "{}/repos/{}/releases/latest",
            self.config.api_base, repo_name
        );

        let mut request = self.client.get(&url);

        if let Some(ref token) = self.config.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub API request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        let release: GitHubRelease = response
            .json()
            .await
            .map_err(|e| format!("Parse GitHub response: {e}"))?;

        Ok(release
            .assets
            .into_iter()
            .map(|a| PluginAsset {
                name: a.name,
                url: a.browser_download_url,
                size: a.size,
                sha256: None,
                platform: None,
            })
            .collect())
    }

    /// 下载 release 资产
    pub async fn download_asset(
        &self,
        download_url: &str,
        dest_dir: &std::path::Path,
        filename: &str,
    ) -> Result<std::path::PathBuf, String> {
        let mut request = self.client.get(download_url);

        if let Some(ref token) = self.config.token {
            request = request.header("Authorization", format!("token {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("GitHub download failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("GitHub download error: {}", response.status()));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Read download: {e}"))?;

        std::fs::create_dir_all(dest_dir).map_err(|e| format!("Create dir: {e}"))?;

        let dest_path = dest_dir.join(filename);
        std::fs::write(&dest_path, &bytes).map_err(|e| format!("Write file: {e}"))?;

        Ok(dest_path)
    }

    /// 检查更新
    pub async fn check_updates(
        &self,
        installed: &[(String, String)], // (repo_name, current_version)
    ) -> Result<Vec<(String, String, String)>, String> {
        let mut updates = vec![];

        for (repo_name, _current_version) in installed {
            match self.get_releases(repo_name).await {
                Ok(_assets) => {
                    // TODO: 比较版本号
                    // 目前简单检查是否有新 release
                }
                Err(_) => continue,
            }
        }

        Ok(updates)
    }
}

// ═══════════════════════════════════════════════
// GitHub API 响应类型
// ═══════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    total_count: usize,
    incomplete_results: bool,
    items: Vec<GitHubRepo>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepo {
    id: u64,
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    homepage: Option<String>,
    stargazers_count: u64,
    forks_count: u64,
    default_branch: Option<String>,
    topics: Vec<String>,
    owner: GitHubOwner,
}

#[derive(Debug, Deserialize)]
struct GitHubOwner {
    login: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    content_type: Option<String>,
}
