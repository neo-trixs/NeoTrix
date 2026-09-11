/// 代理配置模块
/// 支持 SOCKS5/HTTP/HTTPS 代理，从环境变量自动检测

use reqwest::Proxy;

/// 代理配置
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 完整代理 URL (如 "http://127.0.0.1:7897", "socks5://127.0.0.1:7898")
    pub url: String,
}

impl ProxyConfig {
    /// 创建新的代理配置
    pub fn new(url: String) -> Self {
        ProxyConfig { url }
    }

    /// 将配置转换为 reqwest Proxy
    pub fn to_reqwest(&self) -> Result<Proxy, String> {
        Proxy::all(self.url.as_str()).map_err(|e| format!("Invalid proxy URL '{}': {}", self.url, e))
    }

    /// 检查代理是否有效
    pub fn is_valid(&self) -> bool {
        !self.url.is_empty()
    }
}

/// 从环境变量检测代理配置
/// 优先级: HTTPS_PROXY > https_proxy > HTTP_PROXY > http_proxy > ALL_PROXY > all_proxy
pub fn from_env() -> Option<ProxyConfig> {
    let vars = ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"];
    for var in &vars {
        if let Ok(url) = std::env::var(var) {
            let url = url.trim().to_string();
            if !url.is_empty() {
                // 规范化: 补全 scheme
                let url = if !url.contains("://") {
                    format!("http://{}", url)
                } else {
                    url
                };
                return Some(ProxyConfig::new(url));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_with_proxy() {
        std::env::set_var("https_proxy", "http://127.0.0.1:7897");
        let cfg = from_env().unwrap();
        assert_eq!(cfg.url, "http://127.0.0.1:7897");
        std::env::remove_var("https_proxy");
    }

    #[test]
    fn test_from_env_no_proxy() {
        std::env::remove_var("HTTPS_PROXY");
        std::env::remove_var("https_proxy");
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("http_proxy");
        std::env::remove_var("ALL_PROXY");
        std::env::remove_var("all_proxy");
        assert!(from_env().is_none());
    }

    #[test]
    fn test_to_reqwest() {
        let cfg = ProxyConfig::new("http://127.0.0.1:7897".to_string());
        assert!(cfg.to_reqwest().is_ok());
    }
}
