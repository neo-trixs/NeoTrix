//! Attack Class Registry — cloudflare/security-audit-skill ATTACK-CLASSES.md 吸收
//! 
//! 8 攻击类别 + 4 域伴侣路由 + Wildcard + ObviousThings

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 攻击类别定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackClassDef {
    pub class: AttackClass,
    pub description: String,
    pub dangerous_sinks: Vec<String>,
    pub sub_categories: Vec<String>,
    pub domain_companion: Option<DomainCompanion>,
    pub hunting_angles: Vec<HuntingAngle>,
}

/// 域伴侣
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DomainCompanion {
    MemorySafetyBinary,
    AiAndLlm,
    WebProtocolAuth,
    ClientSide,
}

/// 攻击类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttackClass {
    Injection,
    AccessControl,
    ResourceFileHandling,
    CryptographySecrets,
    BusinessLogic,
    FeatureAbuseDataLeakage,
    ChainedAttacksTrustBoundaries,
    Wildcard,
    ObviousThings,
}

/// 狩猎角度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HuntingAngle {
    SadPaths,
    Boundaries,
    ComponentAssumptions,
    WrongOrder,
    Concurrency,
    ParserDisagreement,
    RoundTrips,
    ConfigControl,
    FollowMoneyPrivilege,
    LeakedContext,
    ParameterOverrides,
    UnverifiedClaims,
}

/// 攻击类别注册表
#[derive(Debug)]
pub struct AttackClassRegistry {
    classes: HashMap<AttackClass, AttackClassDef>,
    companion_routes: HashMap<DomainCompanion, Vec<AttackClass>>,
}

impl AttackClassRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            classes: HashMap::new(),
            companion_routes: HashMap::new(),
        };
        
        registry.register_defaults();
        registry
    }

    fn register_defaults(&mut self) {
        // Injection
        self.register(AttackClassDef {
            class: AttackClass::Injection,
            description: "Trace untrusted input from entry point to dangerous sink".to_string(),
            dangerous_sinks: vec![
                "SQL queries".to_string(),
                "HTML output".to_string(),
                "Shell commands".to_string(),
                "Template engines".to_string(),
                "File paths".to_string(),
                "HTTP redirects".to_string(),
                "Deserialization".to_string(),
            ],
            sub_categories: vec![
                "Direct injection".to_string(),
                "Indirect injection (stored)".to_string(),
                "Field/key/header injection".to_string(),
                "Secondary system injection (logs, cache, search)".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::SadPaths,
                HuntingAngle::Boundaries,
                HuntingAngle::ParserDisagreement,
                HuntingAngle::RoundTrips,
            ],
        });

        // Access Control
        self.register(AttackClassDef {
            class: AttackClass::AccessControl,
            description: "Can a caller do something they shouldn't?".to_string(),
            dangerous_sinks: vec![
                "State-changing operations".to_string(),
                "Privileged endpoints".to_string(),
                "Bulk/batch operations".to_string(),
            ],
            sub_categories: vec![
                "Auth bypass".to_string(),
                "Authorization logic".to_string(),
                "Field override".to_string(),
                "Inconsistent checks".to_string(),
                "Per-item permissions".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::ComponentAssumptions,
                HuntingAngle::FollowMoneyPrivilege,
                HuntingAngle::ParameterOverrides,
            ],
        });

        // Resource/File Handling
        self.register(AttackClassDef {
            class: AttackClass::ResourceFileHandling,
            description: "Path traversal, SSRF, unsafe deserialization, memory safety".to_string(),
            dangerous_sinks: vec![
                "File system".to_string(),
                "Network requests".to_string(),
                "Deserializers".to_string(),
                "Archive extractors".to_string(),
            ],
            sub_categories: vec![
                "Path traversal".to_string(),
                "SSRF".to_string(),
                "Unsafe deserialization".to_string(),
                "Archive extraction (zip slip)".to_string(),
                "Memory safety (if applicable)".to_string(),
                "TOCTOU".to_string(),
            ],
            domain_companion: Some(DomainCompanion::MemorySafetyBinary),
            hunting_angles: vec![
                HuntingAngle::Boundaries,
                HuntingAngle::ParserDisagreement,
                HuntingAngle::WrongOrder,
            ],
        });

        // Cryptography and Secrets
        self.register(AttackClassDef {
            class: AttackClass::CryptographySecrets,
            description: "Weak randomness, hardcoded secrets, broken crypto, timing side-channels".to_string(),
            dangerous_sinks: vec![
                "Random number generators".to_string(),
                "Key derivation".to_string(),
                "Encryption/decryption".to_string(),
                "Comparison operations".to_string(),
            ],
            sub_categories: vec![
                "Weak randomness".to_string(),
                "Hardcoded secrets".to_string(),
                "Broken key derivation".to_string(),
                "Timing side-channels".to_string(),
                "Crypto misuse".to_string(),
                "Crypto failure fallbacks".to_string(),
            ],
            domain_companion: Some(DomainCompanion::MemorySafetyBinary),
            hunting_angles: vec![
                HuntingAngle::Boundaries,
                HuntingAngle::ComponentAssumptions,
            ],
        });

        // Business Logic
        self.register(AttackClassDef {
            class: AttackClass::BusinessLogic,
            description: "State machine violations, race conditions, numeric manipulation, implicit trust".to_string(),
            dangerous_sinks: vec![
                "State transitions".to_string(),
                "Concurrent operations".to_string(),
                "Numeric operations".to_string(),
                "Cross-operation boundaries".to_string(),
            ],
            sub_categories: vec![
                "State machine violations".to_string(),
                "Race conditions with business impact".to_string(),
                "Numeric/quantity manipulation".to_string(),
                "Access boundary violations".to_string(),
                "Implicit trust assumptions".to_string(),
                "Time-based logic".to_string(),
                "Default/fallback behavior".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::WrongOrder,
                HuntingAngle::Concurrency,
                HuntingAngle::FollowMoneyPrivilege,
                HuntingAngle::UnverifiedClaims,
                HuntingAngle::ConfigControl,
            ],
        });

        // Feature Abuse and Data Leakage
        self.register(AttackClassDef {
            class: AttackClass::FeatureAbuseDataLeakage,
            description: "Legitimate features used for unintended purposes".to_string(),
            dangerous_sinks: vec![
                "Export/backup".to_string(),
                "Import/restore".to_string(),
                "Search/filter/sort".to_string(),
                "Enumeration side effects".to_string(),
                "Preview/draft/staging".to_string(),
                "Notification/webhook".to_string(),
            ],
            sub_categories: vec![
                "Export as exfiltration".to_string(),
                "Import as injection".to_string(),
                "Search as oracle".to_string(),
                "Enumeration through side effects".to_string(),
                "Preview leakage".to_string(),
                "Notification as SSRF".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::LeakedContext,
                HuntingAngle::ComponentAssumptions,
            ],
        });

        // Chained Attacks and Trust Boundaries
        self.register(AttackClassDef {
            class: AttackClass::ChainedAttacksTrustBoundaries,
            description: "Individual safe behaviors that become dangerous in combination".to_string(),
            dangerous_sinks: vec![
                "Multi-step chains".to_string(),
                "Cross-component trust".to_string(),
                "Second-order attacks".to_string(),
                "Scope escalation".to_string(),
                "Timing/ordering gaps".to_string(),
                "Rollback abuse".to_string(),
            ],
            sub_categories: vec![
                "Multi-step chains".to_string(),
                "Cross-component trust gaps".to_string(),
                "Second-order attacks".to_string(),
                "Scope/capability escalation".to_string(),
                "Timing and ordering".to_string(),
                "Rollback/recovery abuse".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::ComponentAssumptions,
                HuntingAngle::WrongOrder,
                HuntingAngle::Concurrency,
                HuntingAngle::FollowMoneyPrivilege,
            ],
        });

        // Wildcard
        self.register(AttackClassDef {
            class: AttackClass::Wildcard,
            description: "Creative attacks - the thing nobody thought to look for".to_string(),
            dangerous_sinks: vec!["Anything unusual".to_string()],
            sub_categories: vec![
                "Strangest code".to_string(),
                "Half-finished features".to_string(),
                "API vs UI mismatch".to_string(),
                "Hidden endpoints".to_string(),
                "Feature mixing".to_string(),
                "Git history secrets".to_string(),
                "Sabotage (not escalation)".to_string(),
                "Irreversible operations".to_string(),
                "Environment assumptions".to_string(),
                "Test gaps".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::SadPaths,
                HuntingAngle::Boundaries,
                HuntingAngle::ComponentAssumptions,
                HuntingAngle::LeakedContext,
                HuntingAngle::UnverifiedClaims,
            ],
        });

        // Obvious Things
        self.register(AttackClassDef {
            class: AttackClass::ObviousThings,
            description: "Dumb stuff everyone assumes someone else checked".to_string(),
            dangerous_sinks: vec!["Anywhere basics fail".to_string()],
            sub_categories: vec![
                "Hardcoded secrets".to_string(),
                "TODO/FIXME security comments".to_string(),
                "Debug mode in prod".to_string(),
                "Test credentials in prod".to_string(),
                "Unprotected debug endpoints".to_string(),
                "Committed secret files".to_string(),
                "Gitignore gaps".to_string(),
                "Dependency CVEs".to_string(),
                "Dynamic code execution".to_string(),
                "CORS misconfiguration".to_string(),
                "Cookie security flags".to_string(),
                "Open redirects".to_string(),
                "TLS enforcement".to_string(),
                "Error response leakage".to_string(),
            ],
            domain_companion: None,
            hunting_angles: vec![
                HuntingAngle::SadPaths,
                HuntingAngle::Boundaries,
                HuntingAngle::ConfigControl,
                HuntingAngle::LeakedContext,
            ],
        });

        // Build companion routes
        self.companion_routes.insert(DomainCompanion::MemorySafetyBinary, vec![
            AttackClass::ResourceFileHandling,
            AttackClass::CryptographySecrets,
        ]);
        self.companion_routes.insert(DomainCompanion::AiAndLlm, vec![
            // AI/LLM 相关类别会动态添加
        ]);
        self.companion_routes.insert(DomainCompanion::WebProtocolAuth, vec![
            // HTTP 协议/认证相关
        ]);
        self.companion_routes.insert(DomainCompanion::ClientSide, vec![
            // 客户端相关
        ]);
    }

    fn register(&mut self, def: AttackClassDef) {
        let class = def.class;
        // 路由到域伴侣
        if let Some(companion) = def.domain_companion {
            self.companion_routes.entry(companion).or_default().push(class);
        }
        self.classes.insert(class, def);
    }

    /// 获取攻击类别定义
    pub fn get(&self, class: AttackClass) -> Option<&AttackClassDef> {
        self.classes.get(&class)
    }

    /// 根据应用类型选择相关类别
    pub fn select_for_application(&self, app_type: &str) -> Vec<AttackClass> {
        let mut selected = vec![
            AttackClass::Injection,
            AttackClass::AccessControl,
            AttackClass::BusinessLogic,
            AttackClass::FeatureAbuseDataLeakage,
            AttackClass::ChainedAttacksTrustBoundaries,
            AttackClass::Wildcard,
            AttackClass::ObviousThings,
        ];

        match app_type {
            "native" | "binary" | "kernel" | "parser" | "runtime" | "firmware" => {
                selected.push(AttackClass::ResourceFileHandling);
                selected.push(AttackClass::CryptographySecrets);
            }
            "ai" | "llm" | "agent" | "rag" | "mcp" => {
                // AI/LLM 类别动态添加
            }
            "http" | "proxy" | "gateway" | "auth" => {
                // Web 协议/认证类别
            }
            "spa" | "extension" | "webview" => {
                // 客户端类别
            }
            _ => {}
        }

        selected
    }

    /// 获取域伴侣路由的类别
    pub fn get_companion_classes(&self, companion: DomainCompanion) -> Vec<AttackClass> {
        self.companion_routes.get(&companion).cloned().unwrap_or_default()
    }

    /// 获取所有类别
    pub fn all_classes(&self) -> Vec<AttackClass> {
        self.classes.keys().cloned().collect()
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let registry = AttackClassRegistry::new();
        
        // Test 1: All 9 classes registered
        assert_eq!(registry.classes.len(), 9);
        
        // Test 2: Key classes present
        assert!(registry.classes.contains_key(&AttackClass::Injection));
        assert!(registry.classes.contains_key(&AttackClass::AccessControl));
        assert!(registry.classes.contains_key(&AttackClass::BusinessLogic));
        assert!(registry.classes.contains_key(&AttackClass::Wildcard));
        assert!(registry.classes.contains_key(&AttackClass::ObviousThings));
        
        // Test 3: Companion routes
        let memory_classes = registry.get_companion_classes(DomainCompanion::MemorySafetyBinary);
        assert!(memory_classes.contains(&AttackClass::ResourceFileHandling));
        assert!(memory_classes.contains(&AttackClass::CryptographySecrets));
        
        // Test 4: Application type selection
        let native_classes = registry.select_for_application("native");
        assert!(native_classes.contains(&AttackClass::ResourceFileHandling));
        assert!(native_classes.contains(&AttackClass::CryptographySecrets));
        
        Ok(())
    }
}

impl Default for AttackClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AttackClassRegistry::new();
        assert_eq!(registry.classes.len(), 9);
    }

    #[test]
    fn test_companion_routes() {
        let registry = AttackClassRegistry::new();
        let memory = registry.get_companion_classes(DomainCompanion::MemorySafetyBinary);
        assert!(memory.contains(&AttackClass::ResourceFileHandling));
    }

    #[test]
    fn test_application_selection() {
        let registry = AttackClassRegistry::new();
        let web = registry.select_for_application("api");
        assert!(web.contains(&AttackClass::Injection));
        assert!(web.contains(&AttackClass::AccessControl));
        
        let native = registry.select_for_application("native");
        assert!(native.contains(&AttackClass::ResourceFileHandling));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(AttackClassRegistry::self_test().is_ok());
    }
}