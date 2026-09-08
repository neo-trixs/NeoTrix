//! Ordered Backend Router — browser-act/skills + R-P82 吸收
//! 
//! 有序后端路由: 首选 + 备选列表，真实探测可用性，首个完整可用者当选
//! 支持 browser-act, camoufox, playwright 等浏览器后端

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendType {
    BrowserAct,
    Camoufox,
    Playwright,
    Puppeteer,
    Selenium,
    HttpOnly, // 纯 HTTP fallback
}

/// 后端健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendHealth {
    pub backend: BackendType,
    pub available: bool,
    pub latency_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
    pub capabilities: BackendCapabilities,
}

/// 后端能力
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub javascript: bool,
    pub screenshots: bool,
    pub network_capture: bool,
    pub proxy_support: bool,
    pub multi_session: bool,
    pub stealth_mode: bool,
    pub cdpsupport: bool,
}

/// 后端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub backend: BackendType,
    pub priority: u8, // 越小越优先
    pub enabled: bool,
    pub config: HashMap<String, String>,
    pub health_check_cmd: Option<String>,
    pub timeout_ms: u64,
}

/// 路由决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub selected: BackendType,
    pub fallback_chain: Vec<BackendType>,
    pub reason: String,
    pub health_snapshot: Vec<BackendHealth>,
}

/// 有序后端路由器
#[derive(Debug)]
pub struct OrderedBackendRouter {
    backends: Vec<BackendConfig>,
    health_cache: Arc<RwLock<HashMap<BackendType, BackendHealth>>>,
    check_interval: Duration,
    last_full_check: Arc<RwLock<Option<Instant>>>,
}

impl OrderedBackendRouter {
    pub fn new() -> Self {
        // 默认后端优先级 (按 R-P82 探测顺序)
        let backends = vec![
            BackendConfig {
                backend: BackendType::BrowserAct,
                priority: 1,
                enabled: true,
                config: HashMap::new(),
                health_check_cmd: Some("browser-act --version".to_string()),
                timeout_ms: 5000,
            },
            BackendConfig {
                backend: BackendType::Camoufox,
                priority: 2,
                enabled: true,
                config: HashMap::new(),
                health_check_cmd: Some("camoufox --version".to_string()),
                timeout_ms: 5000,
            },
            BackendConfig {
                backend: BackendType::Playwright,
                priority: 3,
                enabled: true,
                config: HashMap::new(),
                health_check_cmd: Some("playwright --version".to_string()),
                timeout_ms: 5000,
            },
            BackendConfig {
                backend: BackendType::HttpOnly,
                priority: 10,
                enabled: true,
                config: HashMap::new(),
                health_check_cmd: None,
                timeout_ms: 1000,
            },
        ];

        Self {
            backends,
            health_cache: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(300), // 5分钟
            last_full_check: Arc::new(RwLock::new(None)),
        }
    }

    /// 添加/更新后端配置
    pub fn upsert_backend(&mut self, config: BackendConfig) {
        if let Some(existing) = self.backends.iter_mut().find(|b| b.backend == config.backend) {
            *existing = config;
        } else {
            self.backends.push(config);
        }
        // 重新排序
        self.backends.sort_by_key(|b| b.priority);
    }

    /// 禁用后端
    pub fn disable_backend(&mut self, backend: BackendType) {
        if let Some(b) = self.backends.iter_mut().find(|b| b.backend == backend) {
            b.enabled = false;
        }
    }

    /// 启用后端
    pub fn enable_backend(&mut self, backend: BackendType) {
        if let Some(b) = self.backends.iter_mut().find(|b| b.backend == backend) {
            b.enabled = true;
        }
    }

    /// 选择最优后端 (核心路由逻辑)
    pub async fn select_backend(&self) -> RoutingDecision {
        let mut health_snapshot = Vec::new();
        
        // 检查是否需要全量健康检查
        let needs_check = {
            let last = self.last_full_check.read().await;
            last.map_or(true, |t| t.elapsed() > self.check_interval)
        };

        if needs_check {
            self.perform_health_checks().await;
            *self.last_full_check.write().await = Some(Instant::now());
        }

        // 从缓存读取健康状态
        let cache = self.health_cache.read().await;
        for backend_config in &self.backends {
            if !backend_config.enabled {
                continue;
            }
            let health = cache.get(&backend_config.backend).cloned().unwrap_or_else(|| BackendHealth {
                backend: backend_config.backend,
                available: false,
                latency_ms: 0,
                last_check: chrono::Utc::now(),
                error: Some("Not checked".to_string()),
                capabilities: BackendCapabilities::default(),
            });
            health_snapshot.push(health);
        }

        // 选择第一个可用的
        for health in &health_snapshot {
            if health.available {
                let fallback_chain: Vec<BackendType> = health_snapshot
                    .iter()
                    .filter(|h| h.backend != health.backend && h.available)
                    .map(|h| h.backend)
                    .collect();

                return RoutingDecision {
                    selected: health.backend,
                    fallback_chain,
                    reason: format!("{} available (latency: {}ms)", format!("{:?}", health.backend), health.latency_ms),
                    health_snapshot,
                };
            }
        }

        // 无可用后端，返回 HTTP fallback
        RoutingDecision {
            selected: BackendType::HttpOnly,
            fallback_chain: vec![],
            reason: "No browser backend available, falling back to HTTP-only".to_string(),
            health_snapshot,
        }
    }

    /// 执行健康检查
    async fn perform_health_checks(&self) {
        let mut cache = self.health_cache.write().await;
        
        for config in &self.backends {
            if !config.enabled {
                continue;
            }

            let start = Instant::now();
            let (available, capabilities, error) = if let Some(cmd) = &config.health_check_cmd {
                Self::check_backend(cmd, config.timeout_ms).await
            } else {
                // HTTP-only 总是可用
                (true, BackendCapabilities::default(), None)
            };
            let latency = start.elapsed().as_millis() as u64;

            cache.insert(config.backend, BackendHealth {
                backend: config.backend,
                available,
                latency_ms: latency,
                last_check: chrono::Utc::now(),
                error,
                capabilities,
            });
        }
    }

    /// 单个后端健康检查
    async fn check_backend(cmd: &str, timeout_ms: u64) -> (bool, BackendCapabilities, Option<String>) {
        let capabilities = BackendCapabilities::default();
        
        let output = tokio::time::timeout(
            Duration::from_millis(timeout_ms),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
        ).await;

        match output {
            Ok(Ok(result)) => {
                if result.status.success() {
                    // 根据后端类型设置能力
                    (true, capabilities, None)
                } else {
                    (false, capabilities, Some(String::from_utf8_lossy(&result.stderr).to_string()))
                }
            }
            Ok(Err(e)) => (false, capabilities, Some(e.to_string())),
            Err(_) => (false, capabilities, Some("Timeout".to_string())),
        }
    }

    /// 获取所有后端健康状态 (用于诊断)
    pub async fn get_health_report(&self) -> Vec<BackendHealth> {
        let cache = self.health_cache.read().await;
        self.backends.iter()
            .filter(|b| b.enabled)
            .map(|b| cache.get(&b.backend).cloned().unwrap_or_else(|| BackendHealth {
                backend: b.backend,
                available: false,
                latency_ms: 0,
                last_check: chrono::Utc::now(),
                error: Some("Not checked".to_string()),
                capabilities: BackendCapabilities::default(),
            }))
            .collect()
    }

    /// Doctor 命令：显示当前路由路径
    pub async fn doctor(&self) -> String {
        let decision = self.select_backend().await;
        let mut report = String::new();
        report.push_str(&format!("Selected Backend: {:?}\n", decision.selected));
        report.push_str(&format!("Reason: {}\n\n", decision.reason));
        report.push_str("Fallback Chain:\n");
        for (i, b) in decision.fallback_chain.iter().enumerate() {
            report.push_str(&format!("  {}. {:?}\n", i + 1, b));
        }
        report.push_str("\nHealth Snapshot:\n");
        for h in &decision.health_snapshot {
            report.push_str(&format!("  {:?}: {} (latency: {}ms) {}\n", 
                h.backend, 
                if h.available { "✓" } else { "✗" },
                h.latency_ms,
                h.error.as_deref().unwrap_or("")
            ));
        }
        report
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let router = OrderedBackendRouter::new();
        
        // Test 1: Default backends configured
        assert_eq!(router.backends.len(), 4);
        assert!(router.backends.iter().any(|b| b.backend == BackendType::BrowserAct));
        assert!(router.backends.iter().any(|b| b.backend == BackendType::HttpOnly));
        
        // Test 2: Priority ordering
        let priorities: Vec<u8> = router.backends.iter().map(|b| b.priority).collect();
        let mut sorted = priorities.clone();
        sorted.sort();
        assert_eq!(priorities, sorted);
        
        // Test 3: Upsert backend
        let mut router = OrderedBackendRouter::new();
        router.upsert_backend(BackendConfig {
            backend: BackendType::Playwright,
            priority: 0, // Highest
            enabled: true,
            config: HashMap::new(),
            health_check_cmd: None,
            timeout_ms: 1000,
        });
        assert_eq!(router.backends[0].backend, BackendType::Playwright);
        
        // Test 4: Disable/enable
        router.disable_backend(BackendType::BrowserAct);
        assert!(!router.backends.iter().find(|b| b.backend == BackendType::BrowserAct).unwrap().enabled);
        router.enable_backend(BackendType::BrowserAct);
        assert!(router.backends.iter().find(|b| b.backend == BackendType::BrowserAct).unwrap().enabled);

        Ok(())
    }
}

impl Default for OrderedBackendRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = OrderedBackendRouter::new();
        assert_eq!(router.backends.len(), 4);
    }

    #[test]
    fn test_backend_priority_ordering() {
        let router = OrderedBackendRouter::new();
        assert_eq!(router.backends[0].backend, BackendType::BrowserAct);
        assert_eq!(router.backends[1].backend, BackendType::Camoufox);
    }

    #[test]
    fn test_upsert_backend() {
        let mut router = OrderedBackendRouter::new();
        router.upsert_backend(BackendConfig {
            backend: BackendType::Playwright,
            priority: 0,
            enabled: true,
            config: HashMap::new(),
            health_check_cmd: None,
            timeout_ms: 1000,
        });
        assert_eq!(router.backends[0].backend, BackendType::Playwright);
    }

    #[test]
    fn test_disable_enable_backend() {
        let mut router = OrderedBackendRouter::new();
        router.disable_backend(BackendType::BrowserAct);
        assert!(!router.backends.iter().find(|b| b.backend == BackendType::BrowserAct).unwrap().enabled);
        router.enable_backend(BackendType::BrowserAct);
        assert!(router.backends.iter().find(|b| b.backend == BackendType::BrowserAct).unwrap().enabled);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(OrderedBackendRouter::self_test().is_ok());
    }
}