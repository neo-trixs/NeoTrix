//! LLM 代理池 — 统一管理第三方 API key 及其模型
//!
//! 持久化位置: `~/.config/neotrix/provider_pool.toml`
//!
//! 设计 (LiteLLM / One-API 式统一池):
//! - 每个池条目 = provider 类型 + 标签 + api_key + 模型 + 自定义标签集
//! - `add` 注册第三方 key (如 `provider pool add openai --key sk-... --model gpt-4o --tag code`)
//! - 启动时 `register_into_gateway` 将全部条目注册进 GatewayV2 (统一路由/健康/配额)
//!   并同步注册进 AccountPool (健康感知租约, 并发/隔离/自动恢复)
//! - key 明文存本机配置文件 (0600), 不落 git; 敏感环境可改用 env 引用 (见 [`PoolEntry`])
//!
//! R-P42: 强化现有节点 (GatewayV2/AccountPool/factory), 不做平行适配器模块。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::l1_action::nt_io::nt_io_provider::common::factory::{self, LlmProviderType, ProviderConfig};
use crate::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
use super::account_pool::AccountPool;

/// 池条目 — 一个第三方 API key + 模型绑定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolEntry {
    /// 唯一标签 (用户自定义, 如 "my-openai" / "dashscope-qwen")
    pub label: String,
    /// provider 类型名 (同 factory::LlmProviderType::from_name, 如 "openai"/"opencode-zen")
    pub provider: String,
    /// API key。可为明文, 或 `env:OPENAI_API_KEY` 形式引用环境变量 (推荐)。
    pub api_key: String,
    /// 默认模型 ID (如 "gpt-4o-mini" / "deepseek-v4-flash-free")
    pub model: String,
    /// 自定义标签集 (按用途/团队/项目分组)
    #[serde(default)]
    pub tags: Vec<String>,
    /// 可选自定义 base_url (覆盖 provider 默认)
    #[serde(default)]
    pub base_url: Option<String>,
    /// 创建时间 (unix epoch secs)
    pub created_ts: u64,
}

impl PoolEntry {
    /// 解析 api_key 字段: `env:NAME` → 读环境变量; 否则返回明文。
    pub fn resolve_key(&self) -> Option<String> {
        if let Some(name) = self.api_key.strip_prefix("env:") {
            std::env::var(name).ok().filter(|k| !k.is_empty())
        } else if self.api_key.is_empty() {
            None
        } else {
            Some(self.api_key.clone())
        }
    }

    /// 构造 ProviderConfig (供 factory::create_provider 使用)。
    pub fn to_provider_config(&self) -> Option<ProviderConfig> {
        let provider_type = LlmProviderType::from_name(&self.provider)?;
        let api_key = self.resolve_key()?;
        Some(ProviderConfig {
            provider_type,
            api_key: Some(api_key),
            base_url: self.base_url.clone(),
            model: Some(self.model.clone()),
            timeout_secs: 120,
            proxy: crate::l1_action::nt_io::nt_io_http_factory::proxy_from_env(),
        })
    }
}

/// 持久化池容器。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProviderPool {
    pub entries: Vec<PoolEntry>,
}

impl ProviderPool {
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("neotrix")
            .join("provider_pool.toml")
    }

    pub fn load() -> Self {
        let p = Self::path();
        if p.exists() {
            match std::fs::read_to_string(&p) {
                Ok(content) => match toml::from_str::<Self>(&content) {
                    Ok(pool) => pool,
                    Err(e) => {
                        eprintln!("[provider-pool] parse error in {}: {}", p.display(), e);
                        Self::default()
                    }
                },
                Err(e) => {
                    eprintln!("[provider-pool] read error: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// 保存到磁盘 (创建目录, 0600 权限)。
    pub fn save(&self) -> Result<(), String> {
        let p = Self::path();
        if let Some(dir) = p.parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;
            }
        }
        let content = toml::to_string(self).map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(&p, content).map_err(|e| format!("write {}: {}", p.display(), e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    /// 新增/更新条目 (按 label 幂等覆盖)。
    pub fn upsert(&mut self, entry: PoolEntry) -> Result<(), String> {
        if entry.label.trim().is_empty() {
            return Err("label 不能为空".into());
        }
        self.entries.retain(|e| e.label != entry.label);
        self.entries.push(entry);
        self.save()
    }

    /// 移除条目。
    pub fn remove(&mut self, label: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.label != label);
        let removed = self.entries.len() != before;
        if removed {
            let _ = self.save();
        }
        removed
    }

    pub fn get(&self, label: &str) -> Option<&PoolEntry> {
        self.entries.iter().find(|e| e.label == label)
    }

    /// 按 provider 或 tag 过滤。
    pub fn filter(&self, provider: Option<&str>, tag: Option<&str>) -> Vec<&PoolEntry> {
        self.entries
            .iter()
            .filter(|e| provider.is_none_or(|p| e.provider == p))
            .filter(|e| tag.is_none_or(|t| e.tags.iter().any(|x| x == t)))
            .collect()
    }

    /// 将全部条目注册进 GatewayV2 + AccountPool。
    ///
    /// - GatewayV2: 以 `{label}` 为 provider 名注册 (可路由/可 health), 每条目独立健康状态。
    /// - AccountPool: 以 `{provider}/{label}` 注册账户租约, 启用并发控制/检疫/自动恢复。
    pub fn register_into_gateway(&self, gateway: &mut GatewayV2) -> usize {
        let mut registered = 0;
        for entry in &self.entries {
            let Some(config) = entry.to_provider_config() else {
                log::warn!("[provider-pool] skip '{}': key/provider 不可解析", entry.label);
                continue;
            };
            let provider = factory::create_provider(config);
            let is_free = LlmProviderType::from_name(&entry.provider)
                .map(|t| t.is_free())
                .unwrap_or(false);
            // 契约 (provider_pool.rs + factory::tests::test_pool_entry_registers_into_gateway):
            // gateway 注册名必须为 `{provider}/{model}`, 使 provider_model() 的 '/' 拆分
            // 与候选链前缀路由直接可用; label 仅作为 AccountPool 账户键 (多账户租约/检疫),
            // 不再兼任 gateway provider 名 (否则 t-pool-gw 无法被模型前缀路由命中)。
            let gateway_name = format!("{}/{}", entry.provider, entry.model);
            gateway.register_provider_with_category(
                &gateway_name,
                provider.into(),
                is_free,
                crate::l1_action::nt_io::nt_io_provider::provider_catalog::ProviderCategory::Cloud,
            );
            if let Ok(pool) = gateway.account_pool.lock() {
                pool.register_default(&entry.provider, &entry.label);
            }
            registered += 1;
            log::info!(
                "[provider-pool] registered '{}' ({}/{})",
                entry.label, entry.provider, entry.model
            );
        }
        registered
    }

    /// 只将条目注册进 AccountPool (不注册 gateway provider)。
    pub(crate) fn _register_into_account_pool(&self, pool: &AccountPool) -> usize {
        for entry in &self.entries {
            pool.register_default(&entry.provider, &entry.label);
        }
        self.entries.len()
    }

    /// 汇总显示 (列表视图)。
    pub fn describe(&self) -> String {
        if self.entries.is_empty() {
            return "LLM 代理池为空。用 `neotrix provider pool add` 注册第三方 API key.".into();
        }
        let mut out = String::from("LLM 代理池:\n");
        for e in &self.entries {
            let key_masked = if e.resolve_key().is_some() {
                "✓".to_string()
            } else {
                "✗ (key 缺失)".to_string()
            };
            let tags = if e.tags.is_empty() {
                "-".to_string()
            } else {
                e.tags.join(",")
            };
            out.push_str(&format!(
                "  [{label}] {provider}/{model}  key:{key_masked}  tags:({tags})\n",
                label = e.label,
                provider = e.provider,
                model = e.model,
                key_masked = key_masked,
                tags = tags,
            ));
        }
        out
    }

    /// 汇总映射: label → (provider, model), 供快速路由查询。
    pub fn label_index(&self) -> HashMap<String, (String, String)> {
        self.entries
            .iter()
            .map(|e| (e.label.clone(), (e.provider.clone(), e.model.clone())))
            .collect()
    }
}

/// 全局池句柄 (惰性加载, 进程级共享)。
pub fn global_provider_pool() -> &'static std::sync::Mutex<ProviderPool> {
    use std::sync::OnceLock;
    static POOL: OnceLock<std::sync::Mutex<ProviderPool>> = OnceLock::new();
    POOL.get_or_init(|| std::sync::Mutex::new(ProviderPool::load()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_roundtrip() {
        let mut pool = ProviderPool::default();
        pool.entries.push(PoolEntry {
            label: "t-openai".into(),
            provider: "openai".into(),
            api_key: "sk-test123".into(),
            model: "gpt-4o-mini".into(),
            tags: vec!["code".into()],
            base_url: None,
            created_ts: 1,
        });
        // 用临时路径验证序列化
        let content = toml::to_string(&pool).expect("serialize");
        let parsed: ProviderPool = toml::from_str(&content).expect("parse");
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].label, "t-openai");
        assert_eq!(parsed.entries[0].tags, vec!["code"]);
    }

    #[test]
    fn test_resolve_key_env_and_plain() {
        let plain = PoolEntry {
            label: "a".into(),
            provider: "openai".into(),
            api_key: "sk-plain".into(),
            model: "gpt-4o-mini".into(),
            tags: vec![],
            base_url: None,
            created_ts: 1,
        };
        assert_eq!(plain.resolve_key().as_deref(), Some("sk-plain"));

        std::env::set_var("NEOTRIX_TEST_POOL_KEY", "sk-from-env");
        let env_ref = PoolEntry {
            label: "b".into(),
            provider: "openai".into(),
            api_key: "env:NEOTRIX_TEST_POOL_KEY".into(),
            model: "gpt-4o-mini".into(),
            tags: vec![],
            base_url: None,
            created_ts: 1,
        };
        assert_eq!(env_ref.resolve_key().as_deref(), Some("sk-from-env"));
    }

    #[test]
    fn test_upsert_and_remove_idempotent() {
        let mut pool = ProviderPool::default();
        let e = |label: &str| PoolEntry {
            label: label.into(),
            provider: "openai".into(),
            api_key: "sk".into(),
            model: "m".into(),
            tags: vec![],
            base_url: None,
            created_ts: 2,
        };
        pool.upsert(e("x")).expect("upsert");
        pool.upsert(e("x")).expect("upsert again");
        assert_eq!(pool.entries.len(), 1, "label 幂等覆盖");
        assert!(pool.remove("x"));
        assert!(!pool.remove("x"));
        assert!(pool.entries.is_empty());
    }

    #[test]
    fn test_filter_by_provider_and_tag() {
        let pool = ProviderPool {
            entries: vec![
                PoolEntry {
                    label: "a".into(),
                    provider: "openai".into(),
                    api_key: "k1".into(),
                    model: "gpt-4o-mini".into(),
                    tags: vec!["code".into(), "fast".into()],
                    base_url: None,
                    created_ts: 1,
                },
                PoolEntry {
                    label: "b".into(),
                    provider: "gemini".into(),
                    api_key: "k2".into(),
                    model: "gemini-2.0-flash".into(),
                    tags: vec!["vision".into()],
                    base_url: None,
                    created_ts: 1,
                },
            ],
        };
        assert_eq!(pool.filter(Some("openai"), None).len(), 1);
        assert_eq!(pool.filter(None, Some("vision")).len(), 1);
        assert_eq!(pool.filter(Some("openai"), Some("code")).len(), 1);
        assert_eq!(pool.filter(Some("openai"), Some("vision")).len(), 0);
        assert_eq!(pool.label_index().len(), 2);
    }
}

// ════════════════════════════════════════════════════════════════
// Unified Architecture: L1Capability + LlmRouter trait
// ════════════════════════════════════════════════════════════════

impl crate::l1_action::traits::L1Capability for ProviderPool {
    fn capability_id(&self) -> &str { "io.llm_router" }
    fn category(&self) -> crate::l1_action::traits::CapabilityCategory {
        crate::l1_action::traits::CapabilityCategory::Cognition
    }
    fn constellation(&self) -> crate::l1_action::traits::ConstellationLevel {
        crate::l1_action::traits::ConstellationLevel::C2Integration
    }
    fn health_check(&self) -> crate::l1_action::traits::CapabilityHealth {

        crate::l1_action::traits::CapabilityHealth {
            healthy: !self.entries.is_empty(),
            latency_ms: None,
            error_rate: 0.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: Some(format!("{} providers registered", self.entries.len())),
        }
    }
    fn description(&self) -> &str { "LLM provider pool — unified router across OpenAI/Anthropic/Gemini/Ollama" }
    fn stats(&self) -> crate::l1_action::traits::CapabilityStats { crate::l1_action::traits::CapabilityStats::default() }
}

impl crate::l1_action::traits::LlmRouter for ProviderPool {
    fn route(&self, _request: &crate::l1_action::traits::LlmRequest) -> Result<crate::l1_action::traits::LlmRoute, crate::l1_action::traits::CapabilityError> {
        let entry = self.entries.first()
            .ok_or_else(|| crate::l1_action::traits::CapabilityError::NotAvailable("No providers in pool".into()))?;
        Ok(crate::l1_action::traits::LlmRoute {
            provider: entry.provider.clone(),
            model: entry.model.clone(),
            estimated_cost: 0.0,
        })
    }

    fn providers(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.label.clone()).collect()
    }
}

// ════════════════════════════════════════════════════════════════
// Registry + Router + Bridge
// ════════════════════════════════════════════════════════════════

use crate::l1_action::traits::{
    LlmRouter as LlmRouterTrait,
    CapabilityHealth as CH,
    CapabilityError as CE,
    LlmRequest, LlmRoute,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// LLM 能力注册中心
pub(crate) struct LlmRegistry {
    routers: Vec<Box<dyn LlmRouterTrait>>,
}

impl Default for LlmRegistry {
    fn default() -> Self { Self::new() }
}

impl LlmRegistry {
    pub fn new() -> Self { Self { routers: Vec::new() } }
    pub fn register(&mut self, router: Box<dyn LlmRouterTrait>) { self.routers.push(router); }
    pub fn get(&self, id: &str) -> Option<&dyn LlmRouterTrait> {
        self.routers.iter().find(|r| r.capability_id() == id).map(|r| r.as_ref())
    }
    pub fn health_check_all(&self) -> Vec<(String, CH)> {
        self.routers.iter().map(|r| (r.capability_id().to_string(), r.health_check())).collect()
    }
    pub fn optimal(&self) -> Option<&dyn LlmRouterTrait> {
        self.routers.iter()
            .filter(|r| r.health_check().healthy)
            .max_by(|a, b| {
                let a_s = 1.0 - a.health_check().error_rate;
                let b_s = 1.0 - b.health_check().error_rate;
                a_s.partial_cmp(&b_s).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.as_ref())
    }
}

/// LLM 智能路由器 — 按请求上下文选择最佳 Provider
pub(crate) struct LlmSmartRouter {
    registry: LlmRegistry,
}

impl LlmSmartRouter {
    pub fn new(registry: LlmRegistry) -> Self { Self { registry } }
    pub fn route(&self, request: &LlmRequest) -> Result<LlmRoute, CE> {
        self.registry.optimal()
            .ok_or_else(|| CE::NotAvailable("No LLM router".into()))?
            .route(request)
    }
    pub fn providers(&self) -> Vec<String> {
        self.registry.optimal().map_or(vec![], |r| r.providers())
    }
}

/// LLM 桥接 — L5 领域技能 → LlmSmartRouter
pub(crate) struct LlmBridge {
    router: LlmSmartRouter,
}

impl LlmBridge {
    pub fn new(router: LlmSmartRouter) -> Self { Self { router } }
    pub fn route(&self, request: &LlmRequest) -> Result<LlmRoute, CE> { self.router.route(request) }
    pub fn providers(&self) -> Vec<String> { self.router.providers() }
}