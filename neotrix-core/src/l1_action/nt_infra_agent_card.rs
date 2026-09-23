//! L1 基础设施 — Agent Card (能力自描述)
//!
//! 每个 Provider 发布 Agent Card:
//! - id, name, description, capabilities, tags, endpoint
//! - 支持 A2A 协议格式
//! - Registry 自动发现 + 心跳

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Agent Card — 能力自描述清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub schema_version: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<AgentCapability>,
    pub tags: Vec<String>,
    pub endpoint: Option<String>,
    pub auth_type: Option<String>,
    pub max_concurrent: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
    // ── E1.2 新增（T12＋T13，全部 #[serde(default)]，旧快照兼容）──
    #[serde(default)]
    pub role: AgentRole,
    #[serde(default)]
    pub knowledge_domains: Vec<String>,
    #[serde(default)]
    pub task_templates: Vec<String>,
    #[serde(default)]
    pub deliverable_types: Vec<String>,
    #[serde(default)]
    pub installed_skills: Vec<String>,
    #[serde(default)]
    pub execution: ExecutionPolicy,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub mcp_permissions: Vec<McpPermission>,
    #[serde(default)]
    pub model_preference: ModelPreference,
    #[serde(default)]
    pub status: AgentStatus,
    #[serde(default)]
    pub visibility: String,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub template_version: Option<String>,
    #[serde(default)]
    pub package_codes: Vec<String>,
    #[serde(default)]
    pub trial_enabled: bool,
    #[serde(default)]
    pub connector: Option<String>,
    #[serde(default)]
    pub icon_class: Option<String>,
    #[serde(default)]
    pub admins: Vec<String>,
    #[serde(default)]
    pub usage_proof: UsageProof,
    // ── T35 E轨（蓝图桌面清单 P2-8＋R-P100；全部 serde-default，旧快照兼容）──
    /// 取值约定 trial/professional/gift/ultra（默认 trial）
    #[serde(default = "default_entitlement")]
    pub entitlement: String,
    /// 约定 zip/skmd 二选一（默认 zip）
    #[serde(default = "default_upload_format")]
    pub upload_format: String,
}

/// Agent 能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub tags: Vec<String>,
}

// ─── E1.2 新增类型（T12＋T13，同文件正典；禁止从 l5_cognition 引用）───

/// 权限等级（最小特权：默认 Read）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PermissionLevel {
    #[default]
    Read,
    Write,
    Admin,
    Sovereign,
}

/// 岗位定义：level＋组织归属＋汇报线＋角色链（`--` 分隔解析）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentRole {
    #[serde(default)]
    pub level: PermissionLevel,
    #[serde(default)]
    pub org_unit: Option<String>,
    #[serde(default)]
    pub reports_to: Option<String>,
    #[serde(default)]
    pub role_chain: Vec<String>,
}

/// 执行策略（AgentCard 扁平字段用；与 nt_act::ExecutionPolicy 同名但不同模块）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionPolicy {
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
}

fn default_max_tokens() -> u32 {
    4000
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout_secs() -> u64 {
    300
}

fn default_retry_count() -> u32 {
    1
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            timeout_secs: default_timeout_secs(),
            retry_count: default_retry_count(),
        }
    }
}

/// MCP 权限：server_name＋tools（空＝全部）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct McpPermission {
    #[serde(default)]
    pub server_name: String,
    #[serde(default)]
    pub tools: Vec<String>,
}

/// Agent 状态（默认 Offline：fail-closed，旧快照不误标 Available）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AgentStatus {
    Available,
    Thinking,
    Busy,
    Resting,
    #[default]
    Offline,
}

/// 模型偏好（本地新建单数版；与 L5 `ModelPreferences` 复数版区分，禁止互引）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPreference {
    #[serde(default = "default_auto")]
    pub tier: String,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub consumption_coefficient: f64,
    #[serde(default)]
    pub order_number: i32,
    #[serde(default)]
    pub auto_routing: bool,
    #[serde(default = "default_auto")]
    pub work_tier: String,
    #[serde(default = "default_auto")]
    pub scheduled_tier: String,
}

fn default_auto() -> String {
    "auto".to_string()
}

/// T35 E轨默认：trial（取值约定 trial/professional/gift/ultra）
fn default_entitlement() -> String {
    "trial".to_string()
}

/// T35 E轨默认：zip（约定 zip/skmd 二选一）
fn default_upload_format() -> String {
    "zip".to_string()
}

impl Default for ModelPreference {
    fn default() -> Self {
        Self {
            tier: default_auto(),
            providers: Vec::new(),
            consumption_coefficient: 0.0,
            order_number: 0,
            auto_routing: false,
            work_tier: default_auto(),
            scheduled_tier: default_auto(),
        }
    }
}

/// 用量实证：累计 token/credits/任务/产物/成功率
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UsageProof {
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub total_credits: f64,
    #[serde(default)]
    pub task_count: u64,
    #[serde(default)]
    pub artifact_count: u64,
    #[serde(default)]
    pub success_rate: f64,
}

impl AgentCard {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Self {
            schema_version: "1.0".into(),
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            version: "0.1.0".into(),
            capabilities: Vec::new(),
            tags: Vec::new(),
            endpoint: None,
            auth_type: None,
            max_concurrent: 10,
            created_at: now,
            last_heartbeat: now,
            role: AgentRole::default(),
            knowledge_domains: Vec::new(),
            task_templates: Vec::new(),
            deliverable_types: Vec::new(),
            installed_skills: Vec::new(),
            execution: ExecutionPolicy::default(),
            workspace_id: None,
            mcp_permissions: Vec::new(),
            model_preference: ModelPreference::default(),
            status: AgentStatus::default(),
            visibility: String::new(),
            template_id: None,
            template_version: None,
            package_codes: Vec::new(),
            trial_enabled: false,
            connector: None,
            icon_class: None,
            admins: Vec::new(),
            usage_proof: UsageProof::default(),
            entitlement: default_entitlement(),
            upload_format: default_upload_format(),
        }
    }

    pub fn with_capability(mut self, cap: AgentCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    /// 心跳更新
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    }

    /// 是否存活 (最近 60s 有心跳)
    pub fn is_alive(&self) -> bool {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        now - self.last_heartbeat < 60
    }
}

/// Agent Card 注册表
pub struct AgentCardRegistry {
    cards: HashMap<String, AgentCard>,
}

impl Default for AgentCardRegistry {
    fn default() -> Self { Self::new() }
}

impl AgentCardRegistry {
    pub fn new() -> Self { Self { cards: HashMap::new() } }

    pub fn register(&mut self, card: AgentCard) {
        self.cards.insert(card.id.clone(), card);
    }

    pub fn unregister(&mut self, id: &str) {
        self.cards.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&AgentCard> {
        self.cards.get(id)
    }

    pub fn heartbeat(&mut self, id: &str) {
        if let Some(card) = self.cards.get_mut(id) {
            card.heartbeat();
        }
    }

    /// 按能力查找
    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentCard> {
        self.cards.values()
            .filter(|c| c.capabilities.iter().any(|cap| cap.name == capability))
            .collect()
    }

    /// 按标签查找
    pub fn find_by_tag(&self, tag: &str) -> Vec<&AgentCard> {
        self.cards.values()
            .filter(|c| c.tags.contains(&tag.to_string()))
            .collect()
    }

    /// 按 workspace 查找（E1.2 预留：AgentCard.workspace_id 可查）
    pub fn find_by_workspace(&self, workspace_id: &str) -> Vec<&AgentCard> {
        self.cards
            .values()
            .filter(|c| c.workspace_id.as_deref() == Some(workspace_id))
            .collect()
    }

    /// 能力交集匹配（E3 Layer-2；重叠分降序，高者在前）
    pub fn match_capabilities(&self, wanted: &[String]) -> Vec<&AgentCard> {
        let mut scored: Vec<(&AgentCard, usize)> = self
            .cards
            .values()
            .map(|c| {
                let overlap = c
                    .capabilities
                    .iter()
                    .filter(|cap| wanted.iter().any(|w| w == &cap.name))
                    .count();
                (c, overlap)
            })
            .filter(|(_, overlap)| *overlap > 0)
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(c, _)| c).collect()
    }

    /// 最佳 Agent（交集最高者；E3 Layer-2 裁决）
    pub fn find_best_agent_for(&self, wanted: &[String]) -> Option<&AgentCard> {
        self.match_capabilities(wanted).into_iter().next()
    }

    /// 心跳存在性检查（T26 桥接：存在则刷新心跳返 true）
    pub fn upsert_heartbeat(&mut self, id: &str) -> bool {
        if self.cards.contains_key(id) {
            self.heartbeat(id);
            true
        } else {
            false
        }
    }

    /// 发布外部实例卡（T26 桥接：存在则更新能力/标签并心跳，否则注册）
    pub fn publish_external_card(&mut self, card: AgentCard) {
        match self.cards.get_mut(&card.id) {
            Some(existing) => {
                existing.capabilities = card.capabilities;
                existing.tags = card.tags;
                existing.heartbeat();
            }
            None => self.register(card),
        }
    }

    /// 查找所有存活的
    pub fn alive_agents(&self) -> Vec<&AgentCard> {
        self.cards.values().filter(|c| c.is_alive()).collect()
    }

    /// 清理过期
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        self.cards.retain(|_, c| now - c.last_heartbeat < 300);
    }

    pub(crate) fn _all_cards(&self) -> Vec<&AgentCard> {
        self.cards.values().collect()
    }
}

// 全局 Agent Card 注册表
lazy_static::lazy_static! {
    static ref GLOBAL_CARDS: Mutex<AgentCardRegistry> = Mutex::new(AgentCardRegistry::new());
}

pub fn agent_card_register(card: AgentCard) {
    GLOBAL_CARDS.lock().unwrap_or_else(|e| e.into_inner()).register(card);
}

pub fn agent_card_get(id: &str) -> Option<AgentCard> {
    GLOBAL_CARDS.lock().unwrap_or_else(|e| e.into_inner()).get(id).cloned()
}

pub fn agent_card_find_by_capability(cap: &str) -> Vec<AgentCard> {
    GLOBAL_CARDS.lock().unwrap_or_else(|e| e.into_inner()).find_by_capability(cap).into_iter().cloned().collect()
}

pub fn agent_card_alive() -> Vec<AgentCard> {
    GLOBAL_CARDS.lock().unwrap_or_else(|e| e.into_inner()).alive_agents().into_iter().cloned().collect()
}

/// 版本升级检查（T26）：installed 与 latest 不等即有升级（供 UpgradeAvailable 事件）。
pub fn check_upgrade(installed: &str, latest: &str) -> Option<(String, String)> {
    if installed != latest {
        Some((installed.to_string(), latest.to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_lifecycle() {
        let mut card = AgentCard::new("test", "Test Agent", "A test agent");
        assert!(card.is_alive());

        card.heartbeat();
        assert!(card.is_alive());
    }

    #[test]
    fn test_registry_find() {
        let mut reg = AgentCardRegistry::new();
        let card = AgentCard::new("a1", "Agent 1", "Search agent")
            .with_capability(AgentCapability {
                name: "search".into(),
                description: "Search".into(),
                input_schema: None,
                output_schema: None,
                tags: vec![],
            })
            .with_tag("search");
        reg.register(card);
        assert_eq!(reg.find_by_capability("search").len(), 1);
        assert_eq!(reg.find_by_tag("search").len(), 1);
    }

    #[test]
    fn test_agent_card_e12_serde_roundtrip() {
        let mut card = AgentCard::new("e12", "E12 Agent", "E1.2 full fields");
        card.role = AgentRole {
            level: PermissionLevel::Write,
            org_unit: Some("ops".to_string()),
            reports_to: Some("lead".to_string()),
            role_chain: vec!["member--role_dev".to_string()],
        };
        card.knowledge_domains = vec!["ops".to_string()];
        card.task_templates = vec!["t1".to_string()];
        card.deliverable_types = vec!["report".to_string()];
        card.installed_skills = vec!["s1".to_string()];
        card.execution = ExecutionPolicy {
            max_tokens: 8000,
            temperature: 0.5,
            timeout_secs: 600,
            retry_count: 2,
        };
        card.workspace_id = Some("ws1".to_string());
        card.mcp_permissions = vec![McpPermission {
            server_name: "srv".to_string(),
            tools: Vec::new(),
        }];
        card.model_preference = ModelPreference {
            tier: "fast".to_string(),
            providers: vec!["p1".to_string()],
            consumption_coefficient: 0.3,
            order_number: 1,
            auto_routing: true,
            work_tier: "fast".to_string(),
            scheduled_tier: "auto".to_string(),
        };
        card.status = AgentStatus::Available;
        card.visibility = "public".to_string();
        card.template_id = Some("tpl".to_string());
        card.template_version = Some("1.0".to_string());
        card.package_codes = vec!["pro".to_string()];
        card.trial_enabled = true;
        card.connector = Some("mcp://srv".to_string());
        card.icon_class = Some("icon".to_string());
        card.admins = vec!["admin".to_string()];
        card.usage_proof = UsageProof {
            total_tokens: 100,
            total_credits: 1.5,
            task_count: 2,
            artifact_count: 1,
            success_rate: 0.5,
        };
        let json = serde_json::to_string(&card);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: Result<AgentCard, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
            if let Ok(v) = back {
                assert_eq!(v.id, "e12");
                assert_eq!(v.workspace_id, Some("ws1".to_string()));
                assert_eq!(v.status, AgentStatus::Available);
                assert_eq!(v.model_preference.tier, "fast");
                // 旧快照兼容：缺新字段可过
                let legacy = serde_json::json!({
                    "schema_version": "1.0",
                    "id": "old",
                    "name": "Old",
                    "description": "legacy",
                    "version": "0.1.0",
                    "capabilities": [],
                    "tags": [],
                    "endpoint": null,
                    "auth_type": null,
                    "max_concurrent": 10,
                    "created_at": 0,
                    "last_heartbeat": 0
                });
                let parsed: Result<AgentCard, _> = serde_json::from_value(legacy);
                assert!(parsed.is_ok());
                if let Ok(old) = parsed {
                    assert_eq!(old.model_preference.tier, "auto");
                    assert_eq!(old.status, AgentStatus::Offline);
                }
            }
        }
    }

    #[test]
    fn test_match_capabilities_ordering() {
        fn cap(name: &str) -> AgentCapability {
            AgentCapability {
                name: name.into(),
                description: "".into(),
                input_schema: None,
                output_schema: None,
                tags: vec![],
            }
        }
        let mut reg = AgentCardRegistry::new();
        reg.register(
            AgentCard::new("a1", "A1", "")
                .with_capability(cap("search"))
                .with_capability(cap("code")),
        );
        reg.register(AgentCard::new("a2", "A2", "").with_capability(cap("search")));
        let hit = reg.match_capabilities(&["search".to_string(), "code".to_string()]);
        assert_eq!(hit.len(), 2);
        assert_eq!(hit[0].id, "a1");
        assert_eq!(
            reg.find_best_agent_for(&["code".to_string()])
                .map(|c| c.id.as_str()),
            Some("a1")
        );
        assert!(reg.find_best_agent_for(&["nope".to_string()]).is_none());
    }

    #[test]
    fn test_upsert_heartbeat_and_upgrade() {
        let mut reg = AgentCardRegistry::new();
        assert!(!reg.upsert_heartbeat("ghost"));
        reg.register(AgentCard::new("a1", "A1", ""));
        assert!(reg.upsert_heartbeat("a1"));
        reg.publish_external_card(AgentCard::new("a1", "A1", "").with_tag("t"));
        assert_eq!(reg.get("a1").map(|c| c.tags.len()), Some(1));
        assert_eq!(
            check_upgrade("1.0.5", "1.0.6"),
            Some(("1.0.5".to_string(), "1.0.6".to_string()))
        );
        assert_eq!(check_upgrade("1.0.6", "1.0.6"), None);
    }

    #[test]
    fn test_agent_card_t35_entitlement_upload_format() {
        // 默认值
        let card = AgentCard::new("t35", "T35 Agent", "T35 defaults");
        assert_eq!(card.entitlement, "trial");
        assert_eq!(card.upload_format, "zip");
        // 非默认往返
        let mut full = AgentCard::new("t35f", "T35 Full", "T35 full");
        full.entitlement = "professional".to_string();
        full.upload_format = "skmd".to_string();
        let json = serde_json::to_string(&full);
        assert!(json.is_ok());
        if let Ok(s) = json {
            let back: Result<AgentCard, _> = serde_json::from_str(&s);
            assert!(back.is_ok());
            if let Ok(v) = back {
                assert_eq!(v.entitlement, "professional");
                assert_eq!(v.upload_format, "skmd");
            }
        }
        // 旧快照兼容：缺两字段可过且落默认
        let legacy = serde_json::json!({
            "schema_version": "1.0",
            "id": "old35",
            "name": "Old35",
            "description": "legacy",
            "version": "0.1.0",
            "capabilities": [],
            "tags": [],
            "endpoint": null,
            "auth_type": null,
            "max_concurrent": 10,
            "created_at": 0,
            "last_heartbeat": 0
        });
        let parsed: Result<AgentCard, _> = serde_json::from_value(legacy);
        assert!(parsed.is_ok());
        if let Ok(old) = parsed {
            assert_eq!(old.entitlement, "trial");
            assert_eq!(old.upload_format, "zip");
        }
    }
}
