//! cookies — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。

use std::collections::HashMap;

use super::error::BrowserError;

/// 自研 CookieJar：host → cookies（支持 Domain 后缀匹配 + Secure 语义简化版）
//⛔ 不`derive(Debug)`：它会递归打印 `entries` 里的每个 `CookieEntry`。
//   改用手工实现（见文件末尾），保证 `{:?}` 只出现 host 与条目**数**。
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CookieJar {    pub(crate) entries: HashMap<String, Vec<CookieEntry>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CookieEntry {
    pub(crate) name: String,
    pub(crate) value: String,
    secure_only: bool,
}

/// ⭐⭐ 手工实现 `Debug`：**`value` 永不打印**。
///
/// 【为什么不能用 `derive(Debug)`】
/// `value` 就是 cookie 的**凭据本身**（等价于 `Set-Cookie` 里的值）。
/// `derive(Debug)` 会让它出现在：任何 `{:?}`、`unwrap()`/`expect()` 的 panic 消息、
/// 结构体被塞进错误类型后的 `Display`/`Debug` 链。
///
/// 【实测现状（2026-10-03）】
/// 全仓当前**没有**任何 `{:?}` 打印 `CookieJar`/`CookieEntry`
/// ⇒泄漏是**潜在的**，不是活的。
/// ⛔ 但「当前没人打印」**不是**护栏：将来任何人给错误类型加一个
/// `CookieJar` 字段、或在日志里打一个 `{:?}`，就会**静默**开始泄露，
/// 而 `cookiejar`/`CookieJar` 这种类型名**不会提醒他**。
///
/// 【吸收来源】`markfulton/agent-cookie-sync`（MIT）的核心主张：
/// 注入脚本 **never prints cookie values**。
/// ⇒ 与本仓既有纪律同向：`Provider::key_env` 只存环境变量**名**，
///   `Provider::looks_like_secret` 对明文形态告警。
///
/// 【代价】调试时看不到值 —— 这是**刻意的**。
/// 需要核对值时请在**受控环境**里直接看字段，不要把它加回`Debug`。
impl std::fmt::Debug for CookieEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CookieEntry")
            .field("name", &self.name)
            .field("value", &"<redacted>")
            .field("secure_only", &self.secure_only)
            .finish()
    }
}

impl CookieJar {
    pub fn new() -> Self {
        Self::default()
    }

    /// 从响应 `Set-Cookie` 头存入（`response_host` 为实际响应主机）
    pub fn store_from_headers<'a>(
        &mut self,
        response_host: &str,
        set_cookies: impl Iterator<Item = &'a str>,
    ) {
        for raw in set_cookies {
            let mut parts = raw.split(';');
            let first = parts.next().unwrap_or("").trim();
            let (name, value) = match first.split_once('=') {
                Some((n, v)) => (n.trim(), v.trim()),
                None => continue,
            };
            if name.is_empty() {
                continue;
            }
            let mut domain = response_host.to_lowercase();
            let mut secure_only = false;
            for attr in parts {
                let attr = attr.trim();
                if let Some((k, v)) = attr.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("domain") {
                        domain = v.trim().trim_start_matches('.').to_lowercase();
                    }
                } else if attr.eq_ignore_ascii_case("secure") {
                    secure_only = true;
                }
            }
            if domain.is_empty() {
                continue;
            }
            let bucket = self.entries.entry(domain).or_default();
            if let Some(slot) = bucket.iter_mut().find(|c| c.name == name) {
                slot.value = value.to_string();
                slot.secure_only = secure_only;
            } else {
                bucket.push(CookieEntry {
                    name: name.to_string(),
                    value: value.to_string(),
                    secure_only,
                });
            }
        }
    }

    /// 为 `(scheme, host)` 组装 `Cookie` 头（无匹配返回 None）
    pub fn header_for(&self, scheme: &str, host: &str) -> Option<String> {
        let host = host.to_lowercase();
        let is_https = scheme.eq_ignore_ascii_case("https");
        let mut pairs = Vec::new();
        for (domain, cookies) in &self.entries {
            if host == *domain || host.ends_with(&format!(".{domain}")) {
                for c in cookies {
                    if c.secure_only && !is_https {
                        continue;
                    }
                    pairs.push(format!("{}={}", c.name, c.value));
                }
            }
        }
        if pairs.is_empty() {
            None
        } else {
            Some(pairs.join("; "))
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn cookie_count(&self) -> usize {
        self.entries.values().map(Vec::len).sum()
    }
}

// ============================================================================
// 认证管理（P1）：Token 文件热加载 + 401 重试 + 过期精确指引
// ============================================================================
//
// 设计目标：7 天过期类 token（如 Lingee `openwork.server.token`）不断连：
// - Token 放文件（默认 `~/.config/neotrix/lingee.token`），轮换 = 改文件，零重启；
// - 每次请求前按内容比对热加载（小文件读开销可忽略，比 mtime 可靠）；
// - 已知过期点则提前拒收（省一次无效请求）；401/403 先重载重试一次，
//   仍失败则返回携带精确重取指引的 `AuthExpired`（操作者/上游 Agent 一步恢复）；
// - 真·无人值守续期需要账号密码 + 短信/RSA 登录流（见模块文档 P2），
//   本层不存密码、不碰钥匙串。

/// Token 来源
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthSource {
    /// 内存直给（测试/短命场景）
    Literal(String),
    /// 文件热加载（生产推荐：轮换零重启）
    File(std::path::PathBuf),
}

/// 会话认证配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthConfig {
    pub source: AuthSource,
    /// 请求头名（默认 Authorization）
    pub header_name: String,
    /// 方案前缀（默认 Bearer；空字符串 = 直接放 token）
    pub scheme: String,
    /// 已知过期点（毫秒时间戳；None = 纯被动，靠 401 触发）
    pub expires_at_ms: Option<u64>,
    /// 过期后给操作者的精确指引（空则按 source 自动生成）
    pub reseed_hint: String,
}

impl AuthConfig {
    /// 文件 token（生产推荐）
    pub fn file(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            source: AuthSource::File(path.into()),
            header_name: "Authorization".to_string(),
            scheme: "Bearer".to_string(),
            expires_at_ms: None,
            reseed_hint: String::new(),
        }
    }

    /// 内存 token（测试/短命场景）
    pub fn literal(token: impl Into<String>) -> Self {
        Self {
            source: AuthSource::Literal(token.into()),
            header_name: "Authorization".to_string(),
            scheme: "Bearer".to_string(),
            expires_at_ms: None,
            reseed_hint: String::new(),
        }
    }

    pub(crate) fn default_hint(&self) -> String {
        match &self.source {
            AuthSource::File(p) => format!(
                "token 已过期或被拒：请更新文件 {} 后重试（轮换零重启，无需重启引擎）",
                p.display()
            ),
            AuthSource::Literal(_) => {
                "token 已过期或被拒：请用 set_session_auth 重新设置".to_string()
            }
        }
    }

    pub(crate) fn hint(&self) -> String {
        if self.reseed_hint.is_empty() {
            self.default_hint()
        } else {
            self.reseed_hint.clone()
        }
    }
}

/// 会话认证态（缓存 + 热加载）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthState {
    pub config: AuthConfig,
    pub(crate) cached_token: String,
}

impl AuthState {
    pub(crate) fn load(config: &AuthConfig) -> Result<Self, BrowserError> {
        let token = read_auth_token(config)?;
        Ok(Self {
            config: config.clone(),
            cached_token: token,
        })
    }

    /// 文件源按内容比对热加载；Literal 直接返回缓存
    pub(crate) fn refresh(&mut self) -> Result<bool, BrowserError> {
        match &self.config.source {
            AuthSource::Literal(_) => Ok(false),
            AuthSource::File(_) => {
                let fresh = read_auth_token(&self.config)?;
                if fresh != self.cached_token {
                    self.cached_token = fresh;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub(crate) fn header_value(&self) -> String {
        if self.config.scheme.is_empty() {
            self.cached_token.clone()
        } else {
            format!("{} {}", self.config.scheme, self.cached_token)
        }
    }

    pub(crate) fn expired(&self) -> bool {
        match self.config.expires_at_ms {
            Some(ts) => now_ms() >= ts,
            None => false,
        }
    }
}

pub(crate) fn read_auth_token(config: &AuthConfig) -> Result<String, BrowserError> {
    match &config.source {
        AuthSource::Literal(t) => {
            let t = t.trim().to_string();
            if t.is_empty() {
                return Err(BrowserError::ActionFailed(
                    "auth token 为空".to_string(),
                ));
            }
            Ok(t)
        }
        AuthSource::File(p) => {
            let raw = std::fs::read_to_string(p).map_err(|e| {
                BrowserError::ActionFailed(format!(
                    "读 token 文件 {}: {e}",
                    p.display()
                ))
            })?;
            let t = raw.trim().to_string();
            if t.is_empty() {
                return Err(BrowserError::ActionFailed(format!(
                    "token 文件 {} 为空",
                    p.display()
                )));
            }
            Ok(t)
        }
    }
}

pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// ⭐ `CookieJar` 的 `Debug`：**只暴露 host 与条目数**，不含任何值。
impl std::fmt::Debug for CookieJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hosts: Vec<&String> = self.entries.keys().collect();
        hosts.sort();
        let mut m = f.debug_map();
        for h in hosts {
            let n = self.entries.get(h).map(|v| v.len()).unwrap_or(0);
            m.entry(h, &n);
        }
        m.finish()
    }
}

#[cfg(test)]
mod redacted_debug_tests {
    use super::*;

    /// ⭐⭐ 反向锁：`{:?}` **绝不**包含 cookie 值。
    /// 吸收 `agent-cookie-sync`（MIT）的「注入脚本从不打印凭据值」。
    #[test]
    fn debug_never_prints_cookie_value() {
        let e = CookieEntry { name: "session".into(), value: "SUPER_SECRET_VALUE".into(), secure_only: true };
        let shown = format!("{e:?}");
        assert!(!shown.contains("SUPER_SECRET_VALUE"), "⛔ Debug 泄露了 cookie 值：{shown}");
        assert!(shown.contains("redacted"), "应显示占位符：{shown}");
        // name 不是秘密，应当可见（否则调试价值为零）
        assert!(shown.contains("session"));
    }

    /// `CookieJar` 的 `{:?}` 同样不得含值，且应给出 host 与条目数。
    #[test]
    fn jar_debug_never_prints_cookie_value() {
        let mut jar = CookieJar::new();
        jar.entries.insert(
            "example.com".to_string(),
            vec![CookieEntry { name: "sid".into(), value: "JAR_SECRET".into(), secure_only: false }],
        );
        let shown = format!("{jar:?}");
        assert!(!shown.contains("JAR_SECRET"), "⛔ CookieJar Debug 泄露了值：{shown}");
        assert!(shown.contains("example.com"), "host 应可见：{shown}");
        assert!(shown.contains('1'), "条目数应可见：{shown}");
    }
}

