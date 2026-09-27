#![forbid(unsafe_code)]
//! 统一认证纳管（nt_auth_store）
//!
//! 浏览器需要的所有账号/密码/token 配置收敛到一处：
//!
//! ```toml
//! # ~/.config/neotrix/auth.toml（600 权限）
//! [sites.lingee]
//! token_file = "~/.config/neotrix/lingee.token"  # 热加载，轮换零重启
//! expires_at_ms = 1790661672135                  # 已知过期点（可选）
//! reseed_hint = "..."                            # 过期指引（可选，默认自动生成）
//!
//! [sites.example]
//! username = "bot@example.com"
//! password_keyring = "neotrix/example-password"  # "service/account"，真密码只活在系统钥匙串
//! ```
//!
//! 纪律：
//! - token 明文只允许出现在 `token_file` 指向的文件（600）或钥匙串；
//! - `token_literal` 保留给测试，生产配置里写了会被 `validate()` 警告；
//! - 密码**永不**进 toml，只存钥匙串（`keyring` 特性，默认开启），读不到就报缺失不猜。

use std::collections::HashMap;
use std::path::PathBuf;

/// 默认配置文件位置
pub fn default_config_path() -> PathBuf {
    let base = std::env::var("NEOTRIX_AUTH_CONFIG")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            dirs::home_dir().map(|h| h.join(".config").join("neotrix").join("auth.toml"))
        });
    base.unwrap_or_else(|| PathBuf::from("auth.toml"))
}

/// 单站点认证条目
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SiteAuth {
    /// token 文件（~ 自动展开），热加载
    pub token_file: Option<String>,
    /// 内存 token（仅测试；生产用会被 validate 警告）
    pub token_literal: Option<String>,
    /// 请求头名（默认 Authorization）
    pub header_name: Option<String>,
    /// 方案前缀（默认 Bearer；空字符串 = 直接放 token）
    pub scheme: Option<String>,
    /// 已知过期点（毫秒时间戳）
    pub expires_at_ms: Option<u64>,
    /// 过期指引（默认按来源自动生成）
    pub reseed_hint: Option<String>,
    /// 登录用户名（P2 全自动续期用，密码走钥匙串）
    pub username: Option<String>,
    /// 钥匙串定位 "service/account"
    pub password_keyring: Option<String>,
}

/// 认证仓库文件结构
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AuthStoreFile {
    #[serde(default)]
    pub sites: HashMap<String, SiteAuth>,
}

/// 统一认证仓库
#[derive(Debug, Clone, Default)]
pub struct AuthStore {
    sites: HashMap<String, SiteAuth>,
    path: PathBuf,
}

impl AuthStore {
    /// 从默认位置加载（文件不存在 = 空仓库，不报错，保证旧流程可用）
    pub fn load() -> Self {
        Self::load_from(&default_config_path())
    }

    /// 从指定路径加载
    pub fn load_from(path: &std::path::Path) -> Self {
        let sites = std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| toml::from_str::<AuthStoreFile>(&raw).ok())
            .map(|f| f.sites)
            .unwrap_or_default();
        Self {
            sites,
            path: path.to_path_buf(),
        }
    }

    /// 配置文件路径（诊断用）
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// 站点列表
    pub fn site_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sites.keys().cloned().collect();
        names.sort();
        names
    }

    /// 取站点条目（不存在返回 None，不报错）
    pub fn get(&self, site: &str) -> Option<&SiteAuth> {
        self.sites.get(site)
    }

    /// 校验：返回所有问题（生产建议逐条清零）
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        for (name, site) in &self.sites {
            if site.token_literal.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false) {
                issues.push(format!(
                    "[{name}] token_literal 含明文 token：请迁到 token_file 或钥匙串"
                ));
            }
            if site.token_file.is_none()
                && site
                    .token_literal
                    .as_ref()
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true)
                && site.username.is_none()
            {
                issues.push(format!("[{name}] 无任何凭证来源（token_file/username 全空）"));
            }
            if let Some(p) = site.password_keyring.as_ref() {
                if !p.contains('/') {
                    issues.push(format!(
                        "[{name}] password_keyring 格式应为 service/account，当前：{p}"
                    ));
                }
            }
        }
        issues
    }

    /// 展开 `~` 前缀
    pub fn expand_tilde(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest);
            }
        }
        PathBuf::from(path)
    }
}

/// 钥匙串读写（密码唯一合法去处；无特性时诚实报错）
pub mod keyring_vault {
    /// 存密码到系统钥匙串（service/account 定位）
    #[cfg(feature = "keyring")]
    pub fn store_password(
        service: &str,
        account: &str,
        password: &str,
    ) -> Result<(), String> {
        let entry = keyring::Entry::new(service, account)
            .map_err(|e| format!("keyring open: {e}"))?;
        entry
            .set_password(password)
            .map_err(|e| format!("keyring store: {e}"))
    }

    /// 从系统钥匙串读密码
    #[cfg(feature = "keyring")]
    pub fn read_password(service: &str, account: &str) -> Result<String, String> {
        let entry = keyring::Entry::new(service, account)
            .map_err(|e| format!("keyring open: {e}"))?;
        entry
            .get_password()
            .map_err(|e| format!("keyring read: {e}"))
    }

    #[cfg(not(feature = "keyring"))]
    pub fn store_password(_service: &str, _account: &str, _password: &str) -> Result<(), String> {
        Err("keyring 特性未启用，密码无处可存".to_string())
    }

    #[cfg(not(feature = "keyring"))]
    pub fn read_password(_service: &str, _account: &str) -> Result<String, String> {
        Err("keyring 特性未启用".to_string())
    }

    /// 解析 "service/account"
    pub fn split_keyring_ref(keyref: &str) -> Option<(String, String)> {
        let (service, account) = keyref.split_once('/')?;
        if service.trim().is_empty() || account.trim().is_empty() {
            return None;
        }
        Some((service.trim().to_string(), account.trim().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[sites.lingee]
token_file = "~/.config/neotrix/lingee.token"
expires_at_ms = 1790661672135

[sites.demo]
token_literal = "abc"
username = "bot@example.com"
password_keyring = "badformat"
"#;

    fn sample_store() -> AuthStore {
        let f: AuthStoreFile = toml::from_str(SAMPLE).unwrap();
        AuthStore {
            sites: f.sites,
            path: PathBuf::from("test"),
        }
    }

    #[test]
    fn test_parse_and_lookup() {
        let store = sample_store();
        assert_eq!(store.site_names(), vec!["demo".to_string(), "lingee".to_string()]);
        let lingee = store.get("lingee").unwrap();
        assert_eq!(
            lingee.token_file.as_deref(),
            Some("~/.config/neotrix/lingee.token")
        );
        assert_eq!(lingee.expires_at_ms, Some(1790661672135));
        assert!(store.get("missing").is_none());
    }

    #[test]
    fn test_validate_flags_plaintext_and_format() {
        let issues = sample_store().validate();
        assert!(
            issues.iter().any(|s| s.contains("token_literal")),
            "应警告明文 token: {issues:?}"
        );
        assert!(
            issues.iter().any(|s| s.contains("service/account")),
            "应警告 keyring 格式: {issues:?}"
        );
        assert!(!issues.iter().any(|s| s.contains("[lingee]")));
    }

    #[test]
    fn test_expand_tilde() {
        let p = AuthStore::expand_tilde("~/.config/neotrix/lingee.token");
        assert!(p.to_string_lossy().ends_with(".config/neotrix/lingee.token"));
        assert_eq!(
            AuthStore::expand_tilde("/abs/x"),
            PathBuf::from("/abs/x")
        );
    }

    #[test]
    fn test_missing_file_is_empty_store() {
        let store = AuthStore::load_from(std::path::Path::new("/definitely/not/here.toml"));
        assert!(store.site_names().is_empty());
        assert!(store.validate().is_empty());
    }

    #[test]
    fn test_split_keyring_ref() {
        assert_eq!(
            keyring_vault::split_keyring_ref("neotrix/lingee-password"),
            Some(("neotrix".to_string(), "lingee-password".to_string()))
        );
        assert!(keyring_vault::split_keyring_ref("badformat").is_none());
        assert!(keyring_vault::split_keyring_ref("a/").is_none());
    }

    #[test]
    fn test_keyring_missing_entry_errors() {
        // 不存在的条目必须报错（不 panic、不猜）
        assert!(keyring_vault::read_password(
            "neotrix-test-nonexistent",
            "nope"
        )
        .is_err());
    }
}
