#![forbid(unsafe_code)]

//! 凭证哨兵 (Credential Sentinel)
//!
//! 将真实凭证替换为不可提取的哨兵令牌，出站请求使用哨兵，
//! 入站响应中将哨兵还原为真实凭证。支持多种凭证类型，
//! 哨兵本身无法逆向还原原始值。

use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use blake2::{Blake2b512, Digest};

// ============================================================================
// Public Types
// ============================================================================

/// 凭证类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CredentialType {
    /// API Key
    ApiKey,
    /// Bearer Token
    BearerToken,
    /// OAuth Token
    OAuthToken,
    /// SSH Private Key
    SshKey,
    /// 数据库密码
    DatabasePassword,
    /// 通用密钥
    GenericSecret,
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialType::ApiKey => write!(f, "ApiKey"),
            CredentialType::BearerToken => write!(f, "BearerToken"),
            CredentialType::OAuthToken => write!(f, "OAuthToken"),
            CredentialType::SshKey => write!(f, "SshKey"),
            CredentialType::DatabasePassword => write!(f, "DatabasePassword"),
            CredentialType::GenericSecret => write!(f, "GenericSecret"),
        }
    }
}

/// 注册的凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    /// 凭证唯一 ID
    pub id: Uuid,
    /// 凭证类型
    pub cred_type: CredentialType,
    /// 凭证名称（人类可读）
    pub name: String,
    /// 凭证所有者/来源
    pub owner: String,
    /// 注册时间
    pub registered_at: DateTime<Utc>,
    /// 最后使用时间
    pub last_used: Option<DateTime<Utc>>,
    /// 使用次数
    pub use_count: u64,
    /// 是否活跃
    pub active: bool,
}

/// 哨兵令牌 — 替代真实凭证的不可逆令牌
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SentinelToken {
    /// 哨兵 ID（与凭证 ID 对应）
    pub sentinel_id: Uuid,
    /// 哈希后的哨兵值（不可逆）
    pub hash: String,
    /// 凭证类型
    pub cred_type: CredentialType,
    /// 凭证名称
    pub name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 哨兵替换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentinelizedPayload {
    /// 替换后的内容
    pub content: String,
    /// 替换的凭证数量
    pub replacements: usize,
    /// 使用的哨兵令牌
    pub sentinels: Vec<SentinelToken>,
    /// 处理耗时（微秒）
    pub duration_us: u64,
}

/// 哨兵还原结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredPayload {
    /// 还原后的内容
    pub content: String,
    /// 还原的凭证数量
    pub restorations: usize,
    /// 处理耗时（微秒）
    pub duration_us: u64,
}

/// 凭证哨兵统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentinelStats {
    /// 注册的凭证数
    pub registered_credentials: u64,
    /// 活跃哨兵数
    pub active_sentinels: u64,
    /// 总替换次数
    pub total_replacements: u64,
    /// 总还原次数
    pub total_restorations: u64,
    /// 拦截的泄露尝试数
    pub intercepted_leaks: u64,
}

// ============================================================================
// CredentialSentinel
// ============================================================================

/// 凭证哨兵引擎
pub struct CredentialSentinel {
    /// 凭证注册表: ID → (凭证值, 元数据)
    credentials: HashMap<Uuid, (String, CredentialEntry)>,
    /// 哨兵令牌映射: 哨兵哈希 → 凭证 ID
    sentinel_map: HashMap<String, Uuid>,
    /// 哨兵前缀（用于识别哨兵令牌）
    sentinel_prefix: String,
    /// 统计
    pub stats: SentinelStats,
}

impl CredentialSentinel {
    /// 创建新的凭证哨兵
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
            sentinel_map: HashMap::new(),
            sentinel_prefix: "SENTINEL_".into(),
            stats: SentinelStats::default(),
        }
    }

    /// 使用自定义哨兵前缀
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            credentials: HashMap::new(),
            sentinel_map: HashMap::new(),
            sentinel_prefix: prefix.into(),
            stats: SentinelStats::default(),
        }
    }

    /// 注册凭证
    pub fn register(
        &mut self,
        credential_value: impl Into<String>,
        cred_type: CredentialType,
        name: impl Into<String>,
        owner: impl Into<String>,
    ) -> SentinelToken {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let cred_str = credential_value.into();

        let entry = CredentialEntry {
            id,
            cred_type: cred_type.clone(),
            name: name.into(),
            owner: owner.into(),
            registered_at: now,
            last_used: None,
            use_count: 0,
            active: true,
        };

        // 生成哨兵哈希
        let hash = self.generate_sentinel_hash(id, &cred_str);

        let token = SentinelToken {
            sentinel_id: id,
            hash: hash.clone(),
            cred_type,
            name: entry.name.clone(),
            created_at: now,
        };

        self.credentials.insert(id, (cred_str, entry));
        self.sentinel_map.insert(hash, id);
        self.stats.registered_credentials += 1;
        self.stats.active_sentinels += 1;

        token
    }

    /// 注销凭证
    pub fn deregister(&mut self, credential_id: Uuid) -> bool {
        if let Some((_, mut entry)) = self.credentials.remove(&credential_id) {
            entry.active = false;
            // 移除哨兵映射
            self.sentinel_map.retain(|_, &mut id| id != credential_id);
            self.stats.active_sentinels = self.stats.active_sentinels.saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// 将内容中的真实凭证替换为哨兵令牌
    pub fn sentinelize(&mut self, content: &str) -> SentinelizedPayload {
        let start = std::time::Instant::now();
        let mut result = content.to_string();
        let mut sentinels = Vec::new();
        let mut replacements = 0;
        let mut updated_ids = Vec::new();

        // 按凭证值长度降序排序，优先替换长凭证避免部分匹配
        let mut sorted_creds: Vec<_> = self.credentials.iter().collect();
        sorted_creds.sort_by(|a, b| b.1 .0.len().cmp(&a.1 .0.len()));

        for (&cred_id, (cred_value, entry)) in &sorted_creds {
            if !entry.active || cred_value.is_empty() {
                continue;
            }

            if result.contains(cred_value.as_str()) {
                let hash = self.generate_sentinel_hash(cred_id, cred_value);
                let sentinel_display = format!("{}{}", self.sentinel_prefix, &hash[..16]);

                result = result.replace(cred_value.as_str(), &sentinel_display);

                updated_ids.push(cred_id);

                sentinels.push(SentinelToken {
                    sentinel_id: cred_id,
                    hash: hash.clone(),
                    cred_type: entry.cred_type.clone(),
                    name: entry.name.clone(),
                    created_at: Utc::now(),
                });

                replacements += 1;
            }
        }

        // 更新使用统计（避免借用冲突）
        let now = Utc::now();
        for cred_id in &updated_ids {
            if let Some((_, ref mut e)) = self.credentials.get_mut(cred_id) {
                e.last_used = Some(now);
                e.use_count += 1;
            }
        }

        let duration = start.elapsed().as_micros() as u64;
        self.stats.total_replacements += replacements as u64;

        SentinelizedPayload {
            content: result,
            replacements,
            sentinels,
            duration_us: duration,
        }
    }

    /// 将哨兵令牌还原为真实凭证
    pub fn restore(&mut self, content: &str) -> RestoredPayload {
        let start = std::time::Instant::now();
        let mut result = content.to_string();
        let mut restorations = 0;

        // 查找所有哨兵令牌
        let sentinel_pattern = format!(r"{}[a-f0-9]{{16}}", regex::escape(&self.sentinel_prefix));
        if let Ok(re) = regex::Regex::new(&sentinel_pattern) {
            let matches: Vec<String> = re.find_iter(&result).map(|m| m.as_str().to_string()).collect();

            for sentinel_display in &matches {
                // 从哨兵显示值反查凭证 ID
                let hash_suffix = &sentinel_display[self.sentinel_prefix.len()..];

                // 遍历哨兵映射找到匹配项
                if let Some(&cred_id) = self.find_sentinel_by_hash_prefix(hash_suffix) {
                    if let Some((cred_value, _)) = self.credentials.get(&cred_id) {
                        result = result.replace(sentinel_display.as_str(), cred_value);
                        restorations += 1;
                    }
                }
            }
        }

        let duration = start.elapsed().as_micros() as u64;
        self.stats.total_restorations += restorations as u64;

        RestoredPayload {
            content: result,
            restorations,
            duration_us: duration,
        }
    }

    /// 检查内容是否包含凭证（不修改）
    pub fn detect(&self, content: &str) -> Vec<CredentialDetection> {
        let mut detections = Vec::new();

        for (&cred_id, (cred_value, entry)) in &self.credentials {
            if !entry.active || cred_value.is_empty() {
                continue;
            }

            let mut offset = 0;
            let content_bytes = content.as_bytes();
            let cred_bytes = cred_value.as_bytes();

            while offset <= content_bytes.len().saturating_sub(cred_bytes.len()) {
                if &content_bytes[offset..offset + cred_bytes.len()] == cred_bytes {
                    detections.push(CredentialDetection {
                        credential_id: cred_id,
                        credential_name: entry.name.clone(),
                        cred_type: entry.cred_type.clone(),
                        offset,
                        length: cred_value.len(),
                    });
                    offset += 1;
                } else {
                    offset += 1;
                }
            }
        }

        detections
    }

    /// 检查内容中是否包含哨兵令牌
    pub fn detect_sentinels(&self, content: &str) -> Vec<SentinelToken> {
        let sentinel_pattern = format!(r"{}[a-f0-9]{{16}}", regex::escape(&self.sentinel_prefix));
        let mut sentinels = Vec::new();

        if let Ok(re) = regex::Regex::new(&sentinel_pattern) {
            for mat in re.find_iter(content) {
                let hash_suffix = &mat.as_str()[self.sentinel_prefix.len()..];
                if let Some(&cred_id) = self.find_sentinel_by_hash_prefix(hash_suffix) {
                    if let Some((_, entry)) = self.credentials.get(&cred_id) {
                        sentinels.push(SentinelToken {
                            sentinel_id: cred_id,
                            hash: self.generate_sentinel_hash(cred_id, ""),
                            cred_type: entry.cred_type.clone(),
                            name: entry.name.clone(),
                            created_at: Utc::now(),
                        });
                    }
                }
            }
        }

        sentinels
    }

    /// 列出所有注册的凭证
    pub fn list_credentials(&self) -> Vec<&CredentialEntry> {
        self.credentials.values().map(|(_, entry)| entry).collect()
    }

    /// 生成哨兵哈希（Blake2b-512，截取前 32 字节）
    fn generate_sentinel_hash(&self, cred_id: Uuid, cred_value: &str) -> String {
        let mut hasher = Blake2b512::new();
        hasher.update(cred_id.as_bytes());
        hasher.update(cred_value.as_bytes());
        hasher.update(self.sentinel_prefix.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..32])
    }

    /// 通过哈希前缀查找哨兵
    fn find_sentinel_by_hash_prefix(&self, prefix: &str) -> Option<&Uuid> {
        self.sentinel_map
            .iter()
            .find(|(hash, _)| hash.starts_with(prefix))
            .map(|(_, id)| id)
    }
}

impl Default for CredentialSentinel {
    fn default() -> Self { Self::new() }
}

/// 凭证检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDetection {
    pub credential_id: Uuid,
    pub credential_name: String,
    pub cred_type: CredentialType,
    pub offset: usize,
    pub length: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_credential() {
        let mut sentinel = CredentialSentinel::new();
        let token = sentinel.register(
            "sk-1234567890abcdefghijklmnop",
            CredentialType::ApiKey,
            "OpenAI Key",
            "test_user",
        );
        assert_eq!(token.cred_type, CredentialType::ApiKey);
        assert_eq!(token.name, "OpenAI Key");
        assert_eq!(sentinel.stats.registered_credentials, 1);
    }

    #[test]
    fn test_sentinelize_replaces_credential() {
        let mut sentinel = CredentialSentinel::new();
        sentinel.register(
            "sk-1234567890abcdefghijklmnop",
            CredentialType::ApiKey,
            "OpenAI Key",
            "test_user",
        );

        let payload = sentinel.sentinelize("My API key is sk-1234567890abcdefghijklmnop, use it wisely");
        assert_eq!(payload.replacements, 1);
        assert!(!payload.content.contains("sk-1234567890abcdefghijklmnop"));
        assert!(payload.content.contains("SENTINEL_"));
    }

    #[test]
    fn test_restore_recovers_credential() {
        let mut sentinel = CredentialSentinel::new();
        let original = "My API key is sk-1234567890abcdefghijklmnop, use it wisely";
        sentinel.register(
            "sk-1234567890abcdefghijklmnop",
            CredentialType::ApiKey,
            "OpenAI Key",
            "test_user",
        );

        let payload = sentinel.sentinelize(original);
        let restored = sentinel.restore(&payload.content);
        assert_eq!(restored.restorations, 1);
        assert_eq!(restored.content, original);
    }

    #[test]
    fn test_detect_finds_credential() {
        let mut sentinel = CredentialSentinel::new();
        sentinel.register(
            "sk-secret123",
            CredentialType::ApiKey,
            "Key",
            "user",
        );

        let detections = sentinel.detect("key: sk-secret123 here");
        assert_eq!(detections.len(), 1);
        assert_eq!(detections[0].offset, 5);
    }

    #[test]
    fn test_deregister() {
        let mut sentinel = CredentialSentinel::new();
        let token = sentinel.register("secret", CredentialType::GenericSecret, "S", "U");
        assert!(sentinel.deregister(token.sentinel_id));
        assert_eq!(sentinel.stats.active_sentinels, 0);
    }

    #[test]
    fn test_multiple_credentials() {
        let mut sentinel = CredentialSentinel::new();
        sentinel.register("key-one", CredentialType::ApiKey, "Key1", "U");
        sentinel.register("key-two", CredentialType::BearerToken, "Token1", "U");

        let payload = sentinel.sentinelize("Use key-one and key-two together");
        assert_eq!(payload.replacements, 2);
    }

    #[test]
    fn test_sentinel_non_extractable() {
        let mut sentinel = CredentialSentinel::new();
        let token = sentinel.register(
            "super-secret-value",
            CredentialType::GenericSecret,
            "Secret",
            "user",
        );
        // 哈希不可逆：从哨兵哈希无法还原原始值
        assert!(!token.hash.contains("super-secret-value"));
        assert!(token.hash.len() > 32); // Blake2b-512 truncated = 64 hex chars
    }

    #[test]
    fn test_custom_prefix() {
        let mut sentinel = CredentialSentinel::with_prefix("REDACTED_");
        sentinel.register("secret-data", CredentialType::GenericSecret, "S", "U");
        let payload = sentinel.sentinelize("data: secret-data end");
        assert!(payload.content.contains("REDACTED_"));
    }
}
