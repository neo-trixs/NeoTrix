use std::collections::HashSet;
use std::time::Duration;
use reqwest::Client;
use serde_json::Value;

/// 代理订阅源自动发现器
/// 
/// 来源:
/// - GitHub: 搜索包含 proxy list / subscription 的仓库
/// - 知名免费代理站点
/// - Telegram/频道聚合 (通过公开索引)
/// - 社区维护列表 (freefq, mahdibland, ssrsub 等)

pub struct ProxySubscriptionDiscoverer {
    client: Client,
    discovered: HashSet<String>,
    _max_concurrent: usize,
    timeout: Duration,
}

impl ProxySubscriptionDiscoverer {
    pub fn new(_max_concurrent: usize, timeout_secs: u64) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .user_agent("NeoTrix/0.19 ProxyDiscoverer")
                .build()
                .unwrap_or_else(|_| Client::new()),
            discovered: HashSet::new(),
            _max_concurrent,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// 从 GitHub 搜索代理订阅仓库
    pub async fn discover_from_github(&mut self) -> Vec<String> {
        let mut urls = Vec::new();
        
        // 预置的高质量订阅源仓库
        let known_repos = [
            ("freefq", "free", "v2"),
            ("mahdibland", "ShadowsocksAggregator", "master/Eternity.txt"),
            ("ssrsub", "ssr", "master/ss-sub"),
            ("proxy-list-org", "proxy-list", "main/proxy.txt"),
            ("clash-config", "clash", "config.yaml"),
        ];

        for (owner, repo, path) in known_repos {
            let url = format!("https://raw.githubusercontent.com/{}/{}/{}", owner, repo, path);
            if self.discovered.insert(url.clone()) {
                urls.push(url);
            }
        }

        // GitHub API 搜索 (需要 token 避免限流)
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if let Ok(api_urls) = self.search_github_repos(&token).await {
                for u in api_urls {
                    if self.discovered.insert(u.clone()) {
                        urls.push(u);
                    }
                }
            }
        }

        urls
    }

    /// 通过 GitHub Search API 发现代理相关仓库
    async fn search_github_repos(&self, token: &str) -> Result<Vec<String>, String> {
        let queries = [
            "proxy subscription in:readme stars:>100",
            "clash config proxy list",
            "v2ray subscription free",
            "shadowsocks free proxy list",
        ];

        let mut urls = Vec::new();
        
        for query in queries {
            let url = format!(
                "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=10",
                urlencoding::encode(query)
            );
            
            let resp = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await
                .map_err(|e| format!("GitHub search failed: {}", e))?;
            
            if !resp.status().is_success() {
                continue;
            }
            
            let json: Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(items) = json["items"].as_array() {
                for item in items {
                    if let (Some(owner), Some(repo), Some(default_branch)) = (
                        item["owner"]["login"].as_str(),
                        item["name"].as_str(),
                        item["default_branch"].as_str(),
                    ) {
                        // 常见订阅文件路径
                        for path in &["sub.txt", "proxy.txt", "config.yaml", "subs.txt", "list.txt"] {
                            urls.push(format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, default_branch, path));
                        }
                    }
                }
            }
        }
        
        Ok(urls)
    }

    /// 从知名免费代理聚合站点发现订阅
    pub async fn discover_from_web(&mut self) -> Vec<String> {
        let mut urls = Vec::new();
        
        // 知名聚合站点 (base64 编码的订阅链接)
        let sources = [
            ("geonode", "https://proxylist.geonode.com/api/proxy-list?limit=100&page=1&sort_by=lastChecked&sort_type=desc"),
            ("proxy-list", "https://www.proxy-list.download/api/v1/get?type=http"),
            ("proxyscrape", "https://api.proxyscrape.com/v2/?request=displayproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all"),
            ("free-proxy-list", "https://free-proxy-list.net/"),
            ("spys-one", "http://spys.one/en/"),
        ];

        for (name, url) in sources {
            if let Ok(subs) = self.fetch_and_parse_source(name, url).await {
                for s in subs {
                    if self.discovered.insert(s.clone()) {
                        urls.push(s);
                    }
                }
            }
        }
        
        urls
    }

    /// 解析单个来源
    async fn fetch_and_parse_source(&self, name: &str, url: &str) -> Result<Vec<String>, String> {
        let resp = self.client.get(url).send().await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("{} status {}", name, resp.status()));
        }
        let text = resp.text().await.map_err(|e| e.to_string())?;
        
        let mut subs = Vec::new();
        
        // 尝试作为 JSON 解析 (geonode 等)
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            if let Some(data) = json["data"].as_array() {
                for item in data {
                    if let (Some(ip), Some(port)) = (item["ip"].as_str(), item["port"].as_u64()) {
                        subs.push(format!("http://{}:{}", ip, port));
                    }
                }
            }
        }
        
        // 尝试作为纯文本代理列表解析 (proxyscrape 等)
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // 格式: ip:port 或 protocol://ip:port
            if line.contains(':') && (line.starts_with("http") || line.starts_with("socks") || line.chars().next().map_or(false, |c| c.is_ascii_digit())) {
                let normalized = if line.starts_with("http") || line.starts_with("socks") {
                    line.to_string()
                } else {
                    format!("http://{}", line)
                };
                subs.push(normalized);
            }
            // Base64 编码的订阅链接
            else if line.len() > 50 && self.is_base64(line) {
                if let Ok(decoded) = crate::neotrix::nt_shield_stealth_net::pool_health::base64_decode(line) {
                    let decoded_str: String = decoded;
                    for sub_line in decoded_str.lines() {
                        let sub_line: &str = sub_line.trim();
                        if !sub_line.is_empty() && (sub_line.starts_with("http") || sub_line.starts_with("socks")) {
                            subs.push(sub_line.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(subs)
    }

    /// 简单 base64 检测
    fn is_base64(&self, s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') 
            && s.len() % 4 == 0
    }

    /// 完整发现流程
    pub async fn discover_all(&mut self) -> Vec<String> {
        let mut all = Vec::new();
        
        // 1. GitHub 知名仓库
        let github = self.discover_from_github().await;
        println!("[discovery] GitHub 发现 {} 个订阅源", github.len());
        all.extend(github);
        
        // 2. Web 聚合站点
        let web = self.discover_from_web().await;
        println!("[discovery] Web 发现 {} 个代理节点", web.len());
        all.extend(web);
        
        println!("[discovery] 总计发现 {} 个唯一订阅/节点", all.len());
        all
    }

    /// 验证订阅源可用性
    pub async fn validate_subscriptions(&self, urls: &[String]) -> Vec<String> {
        let mut valid = Vec::new();
        
        for url in urls {
            if self.check_subscription(url).await {
                valid.push(url.clone());
            }
        }
        
        println!("[discovery] 有效订阅源: {}/{}", valid.len(), urls.len());
        valid
    }

    /// 检查单个订阅源
    async fn check_subscription(&self, url: &str) -> bool {
        // 快速 HEAD 请求
        match self.client.head(url).timeout(self.timeout).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

/// 便捷函数：发现并返回有效订阅
pub async fn discover_proxy_subscriptions() -> Vec<String> {
    let mut discoverer = ProxySubscriptionDiscoverer::new(10, 10);
    let all = discoverer.discover_all().await;
    discoverer.validate_subscriptions(&all).await
}

/// 后台任务：定期刷新订阅源
pub async fn start_discovery_loop(interval_hours: u64, pool: &crate::neotrix::nt_shield_stealth_net::proxy_pool::ProxyPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_hours * 3600));
    
    loop {
        interval.tick().await;
        
        let mut discoverer = ProxySubscriptionDiscoverer::new(10, 10);
        let valid = discoverer.discover_all().await;
        
        for url in valid {
            pool.add_subscription(&url).await;
        }
        
        println!("[proxy-discovery] 订阅刷新完成，当前池大小: {}", pool.total_count().await);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_github_known() {
        let mut d = ProxySubscriptionDiscoverer::new(5, 5);
        let urls = d.discover_from_github().await;
        assert!(!urls.is_empty());
        assert!(urls.iter().any(|u| u.contains("freefq")));
    }

    #[tokio::test]
    #[ignore = "需要网络"]
    async fn test_discover_web() {
        let mut d = ProxySubscriptionDiscoverer::new(5, 10);
        let urls = d.discover_from_web().await;
        assert!(!urls.is_empty());
    }
}
