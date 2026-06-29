use log;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use super::rules::{OutboundAction, OutboundRule, RuleCondition, RuleEngine, RuleOrigin};

const RULES_API_PORT_FILE: &str = "rules_api.port";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleRequest {
    pub label: String,
    pub condition_type: String,
    pub condition_value: String,
    pub action: String,
    pub action_param: Option<String>,
    pub priority: Option<u32>,
    pub source: Option<String>,
    pub ttl_secs: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RulesApiStatus {
    pub running: bool,
    pub port: u16,
    pub external_rules_count: usize,
    pub active_rules_count: usize,
}

pub struct RulesApiServer {
    ready: AtomicBool,
    running: Arc<AtomicBool>,
    rule_engine: Arc<RwLock<RuleEngine>>,
}

impl RulesApiServer {
    pub fn new(rule_engine: Arc<RwLock<RuleEngine>>) -> Self {
        Self {
            ready: AtomicBool::new(false),
            running: Arc::new(AtomicBool::new(true)),
            rule_engine,
        }
    }

    fn neotrix_dir() -> std::path::PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        home.join(".neotrix")
    }

    async fn write_port(port: u16) {
        let dir = Self::neotrix_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(RULES_API_PORT_FILE);
        let tmp = path.with_extension("tmp");
        let _ = std::fs::write(&tmp, port.to_string());
        let _ = std::fs::rename(&tmp, &path);
    }

    async fn read_port() -> Option<u16> {
        let content = match std::fs::read_to_string(Self::neotrix_dir().join(RULES_API_PORT_FILE)) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("[rule-api] read port: {}", e);
                return None;
            }
        };
        match content.trim().parse() {
            Ok(port) => Some(port),
            Err(e) => {
                log::warn!("[rule-api] parse port: {}", e);
                None
            }
        }
    }

    fn parse_rule(req: RuleRequest) -> Result<OutboundRule, String> {
        let condition = match req.condition_type.to_lowercase().as_str() {
            "domain_suffix" | "domainsuffix" => RuleCondition::DomainSuffix(req.condition_value),
            "domain_exact" | "domainexact" => RuleCondition::DomainExact(req.condition_value),
            "url_path_prefix" | "urlpathprefix" => {
                RuleCondition::UrlPathPrefix(req.condition_value)
            }
            "cidr" => RuleCondition::from_cidr(&req.condition_value)
                .ok_or_else(|| format!("Invalid CIDR: {}", req.condition_value))?,
            "scheme" => RuleCondition::SchemeMatch(req.condition_value),
            "always" => RuleCondition::Always,
            _ => return Err(format!("Unknown condition type: {}", req.condition_type)),
        };
        let action = match req.action.to_lowercase().as_str() {
            "direct" => OutboundAction::Direct,
            "block" => OutboundAction::Block,
            "tor" => OutboundAction::Tor,
            "proxy" => OutboundAction::Proxy(req.action_param.unwrap_or_default()),
            _ => return Err(format!("Unknown action: {}", req.action)),
        };
        let mut rule = OutboundRule::external(
            &req.label,
            condition,
            action,
            &req.source.unwrap_or_else(|| "unknown".into()),
        );
        if let Some(p) = req.priority {
            rule.priority = p;
        }
        rule.ttl_secs = req.ttl_secs;
        Ok(rule)
    }

    fn json_resp(success: bool, message: String, data: Option<serde_json::Value>) -> String {
        serde_json::to_string(&ApiResponse {
            success,
            message,
            data,
        })
        .unwrap_or_else(|e| {
            format!(
                "{{\"success\":false,\"message\":\"serialization error: {}\",\"data\":null}}",
                e
            )
        })
    }

    async fn handle_request(body: &str, engine: &RwLock<RuleEngine>) -> String {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(body);
        let req_obj = match parsed {
            Ok(v) => v,
            Err(e) => return Self::json_resp(false, format!("JSON error: {}", e), None),
        };
        let method = req_obj.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let payload = req_obj.get("payload");

        match method {
            "register_rule" | "POST" => {
                let rule_req: RuleRequest = match serde_json::from_value(
                    payload.cloned().unwrap_or(serde_json::Value::Null),
                ) {
                    Ok(r) => r,
                    Err(e) => return Self::json_resp(false, format!("Invalid rule: {}", e), None),
                };
                match Self::parse_rule(rule_req) {
                    Ok(rule) => {
                        engine.write().await.add_rule(rule);
                        Self::json_resp(true, "Rule registered".into(), None)
                    }
                    Err(e) => Self::json_resp(false, e, None),
                }
            }
            "remove_rule" | "DELETE" => {
                let label = payload
                    .and_then(|p| p.get("label"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("");
                if label.is_empty() {
                    return Self::json_resp(false, "Missing label".into(), None);
                }
                engine.write().await.remove_rule(label);
                Self::json_resp(true, format!("Rule '{}' removed", label), None)
            }
            "list_rules" | "GET" => {
                let rules = engine.read().await.export();
                let external: Vec<&OutboundRule> = rules
                    .iter()
                    .filter(|r| matches!(r.origin, RuleOrigin::External(_)))
                    .collect();
                let all: Vec<serde_json::Value> = external.iter().map(|r| {
                    serde_json::json!({"label": r.label, "action": format!("{:?}", r.action), "condition": format!("{:?}", r.condition), "priority": r.priority, "enabled": r.enabled})
                }).collect();
                Self::json_resp(
                    true,
                    "ok".into(),
                    Some(serde_json::json!({"rules": all, "total": rules.len()})),
                )
            }
            "clear_all" => {
                engine.write().await.clear_external();
                Self::json_resp(true, "Cleared external rules".into(), None)
            }
            "status" => {
                let rules = engine.read().await.export();
                let external = rules
                    .iter()
                    .filter(|r| matches!(r.origin, RuleOrigin::External(_)))
                    .count();
                Self::json_resp(
                    true,
                    "ok".into(),
                    Some(
                        serde_json::json!({"active_rules": rules.len(), "external_rules": external}),
                    ),
                )
            }
            _ => Self::json_resp(false, format!("Unknown method: {}", method), None),
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("[rules-api] bind: {}", e))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("get port: {}", e))?
            .port();
        Self::write_port(port).await;
        self.ready.store(true, Ordering::Release);
        log::info!("[rules-api] listening on 127.0.0.1:{}", port);
        while self.running.load(Ordering::Relaxed) {
            let (mut stream, _) = match listener.accept().await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("[rules-api] accept: {}", e);
                    continue;
                }
            };
            let engine = self.rule_engine.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = vec![0u8; 65536];
                let n = match stream.read(&mut buf).await {
                    Ok(0) | Err(_) => return,
                    Ok(n) => n,
                };
                let body = String::from_utf8_lossy(&buf[..n.min(buf.len())]).to_string();
                let response = Self::handle_request(&body, &engine).await;
                let http_resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.len(), response
                );
                let _ = stream.write_all(http_resp.as_bytes()).await;
            });
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub fn discover_port() -> Option<u16> {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                log::warn!("[rule-api] create runtime: {}", e);
                return None;
            }
        };
        rt.block_on(Self::read_port())
    }

    pub async fn stats(&self) -> RulesApiStatus {
        let rules = self.rule_engine.read().await.export();
        let external_count = rules
            .iter()
            .filter(|r| matches!(r.origin, RuleOrigin::External(_)))
            .count();
        RulesApiStatus {
            running: self.ready.load(Ordering::Acquire),
            port: 0,
            external_rules_count: external_count,
            active_rules_count: rules.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule_domain_suffix() {
        let req = RuleRequest {
            label: "test".into(),
            condition_type: "domain_suffix".into(),
            condition_value: ".google.com".into(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert_eq!(rule.label, "test");
        assert!(matches!(rule.condition, RuleCondition::DomainSuffix(_)));
        assert!(matches!(rule.action, OutboundAction::Block));
    }

    #[test]
    fn test_parse_rule_domain_exact() {
        let req = RuleRequest {
            label: "exact".into(),
            condition_type: "domainexact".into(),
            condition_value: "example.com".into(),
            action: "direct".into(),
            action_param: None,
            priority: Some(50),
            source: Some("test-app".into()),
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.origin, RuleOrigin::External(_)));
        assert_eq!(rule.priority, 50);
    }

    #[test]
    fn test_parse_rule_proxy_action() {
        let req = RuleRequest {
            label: "proxy-test".into(),
            condition_type: "always".into(),
            condition_value: String::new(),
            action: "proxy".into(),
            action_param: Some("http://127.0.0.1:8080".into()),
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.action, OutboundAction::Proxy(_)));
    }

    #[test]
    fn test_parse_rule_cidr() {
        let req = RuleRequest {
            label: "cidr-test".into(),
            condition_type: "cidr".into(),
            condition_value: "10.0.0.0/8".into(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.condition, RuleCondition::Cidr(_, _)));
    }

    #[test]
    fn test_parse_rule_invalid_cidr() {
        let req = RuleRequest {
            label: "bad".into(),
            condition_type: "cidr".into(),
            condition_value: "not-a-cidr".into(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        assert!(RulesApiServer::parse_rule(req).is_err());
    }

    #[test]
    fn test_parse_rule_unknown_condition() {
        let req = RuleRequest {
            label: "bad".into(),
            condition_type: "nonexistent".into(),
            condition_value: String::new(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        assert!(RulesApiServer::parse_rule(req).is_err());
    }

    #[test]
    fn test_parse_rule_unknown_action() {
        let req = RuleRequest {
            label: "bad".into(),
            condition_type: "always".into(),
            condition_value: String::new(),
            action: "nonexistent".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        assert!(RulesApiServer::parse_rule(req).is_err());
    }

    #[test]
    fn test_rule_request_serde_roundtrip() {
        let req = RuleRequest {
            label: "test".into(),
            condition_type: "domain_suffix".into(),
            condition_value: ".com".into(),
            action: "block".into(),
            action_param: None,
            priority: Some(100),
            source: Some("app".into()),
            ttl_secs: Some(300),
        };
        let json = serde_json::to_string(&req)
            .expect("serde_json to_string on RuleRequest should succeed");
        let deserialized: RuleRequest = serde_json::from_str(&json)
            .expect("serde_json from_str of roundtripped JSON should succeed");
        assert_eq!(deserialized.label, "test");
        assert_eq!(deserialized.priority, Some(100));
        assert_eq!(deserialized.ttl_secs, Some(300));
    }

    #[test]
    fn test_api_response_serde() {
        let resp = ApiResponse {
            success: true,
            message: "ok".into(),
            data: Some(serde_json::json!({"count": 42})),
        };
        let json = serde_json::to_string(&resp)
            .expect("serde_json to_string on ApiResponse should succeed");
        let deserialized: ApiResponse = serde_json::from_str(&json)
            .expect("serde_json from_str of roundtripped JSON should succeed");
        assert!(deserialized.success);
        assert_eq!(deserialized.message, "ok");
    }

    #[test]
    fn test_rules_api_status() {
        let status = RulesApiStatus {
            running: true,
            port: 12345,
            external_rules_count: 3,
            active_rules_count: 10,
        };
        assert!(status.running);
        assert_eq!(status.port, 12345);
        assert_eq!(status.active_rules_count, 10);
    }

    #[test]
    fn test_parse_rule_url_path_prefix() {
        let req = RuleRequest {
            label: "api".into(),
            condition_type: "url_path_prefix".into(),
            condition_value: "/api/".into(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.condition, RuleCondition::UrlPathPrefix(_)));
    }

    #[test]
    fn test_parse_rule_scheme() {
        let req = RuleRequest {
            label: "http-only".into(),
            condition_type: "scheme".into(),
            condition_value: "http".into(),
            action: "direct".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.condition, RuleCondition::SchemeMatch(_)));
    }

    #[test]
    fn test_parse_rule_tor_action() {
        let req = RuleRequest {
            label: "tor-traffic".into(),
            condition_type: "always".into(),
            condition_value: String::new(),
            action: "tor".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert!(matches!(rule.action, OutboundAction::Tor));
    }

    #[test]
    fn test_parse_rule_with_ttl() {
        let req = RuleRequest {
            label: "temp".into(),
            condition_type: "always".into(),
            condition_value: String::new(),
            action: "block".into(),
            action_param: None,
            priority: None,
            source: None,
            ttl_secs: Some(60),
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert_eq!(rule.ttl_secs, Some(60));
    }

    #[test]
    fn test_parse_rule_hostname_in_label() {
        let req = RuleRequest {
            label: "block-example-com".into(),
            condition_type: "domain_exact".into(),
            condition_value: "example.com".into(),
            action: "block".into(),
            action_param: None,
            priority: Some(1),
            source: Some("test".into()),
            ttl_secs: None,
        };
        let rule = RulesApiServer::parse_rule(req)
            .expect("parse_rule should succeed for hardcoded valid RuleRequest");
        assert_eq!(rule.label, "block-example-com");
        assert_eq!(rule.priority, 1);
    }

    #[test]
    fn test_handle_request_register_rule() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = r#"{"method":"register_rule","payload":{"label":"test","condition_type":"always","condition_value":"","action":"block"}}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
        assert!(parsed.message.contains("Rule registered"));
    }

    #[test]
    fn test_handle_request_remove_rule_empty_label() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = r#"{"method":"remove_rule","payload":{"label":""}}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(!parsed.success);
        assert!(parsed.message.contains("Missing label"));
    }

    #[test]
    fn test_handle_request_remove_rule_with_label() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        {
            let mut eng = engine.blocking_write();
            eng.add_rule(OutboundRule::new(
                "to-remove",
                RuleCondition::Always,
                OutboundAction::Block,
            ));
        }
        let body = r#"{"method":"remove_rule","payload":{"label":"to-remove"}}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
    }

    #[test]
    fn test_handle_request_list_rules() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        {
            let mut eng = engine.blocking_write();
            eng.add_rule(OutboundRule::external(
                "ext1",
                RuleCondition::Always,
                OutboundAction::Block,
                "app",
            ));
        }
        let body = r#"{"method":"list_rules"}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
        let data = parsed
            .data
            .expect("parsed.data should be Some in test response");
        assert_eq!(data["total"], 1);
    }

    #[test]
    fn test_handle_request_clear_all() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        {
            let mut eng = engine.blocking_write();
            eng.add_rule(OutboundRule::external(
                "ext1",
                RuleCondition::Always,
                OutboundAction::Block,
                "app",
            ));
        }
        let body = r#"{"method":"clear_all"}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
        assert!(parsed.message.contains("Cleared"));
        assert_eq!(engine.blocking_read().rule_count(), 0);
    }

    #[test]
    fn test_handle_request_status() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        {
            let mut eng = engine.blocking_write();
            eng.add_rule(OutboundRule::new(
                "local",
                RuleCondition::Always,
                OutboundAction::Direct,
            ));
            eng.add_rule(OutboundRule::external(
                "ext",
                RuleCondition::Always,
                OutboundAction::Block,
                "test",
            ));
        }
        let body = r#"{"method":"status"}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
        let data = parsed
            .data
            .expect("parsed.data should be Some in test response");
        assert_eq!(data["active_rules"], 2);
        assert_eq!(data["external_rules"], 1);
    }

    #[test]
    fn test_handle_request_invalid_json() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = "not json at all";
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(!parsed.success);
        assert!(parsed.message.contains("JSON error"));
    }

    #[test]
    fn test_handle_request_unknown_method() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = r#"{"method":"nonexistent"}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(!parsed.success);
        assert!(parsed.message.contains("Unknown method"));
    }

    #[test]
    fn test_handle_request_post_shorthand() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = r#"{"method":"POST","payload":{"label":"p","condition_type":"always","condition_value":"","action":"tor"}}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
        assert_eq!(engine.blocking_read().rule_count(), 1);
    }

    #[test]
    fn test_handle_request_get_shorthand() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        let body = r#"{"method":"GET"}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
    }

    #[test]
    fn test_handle_request_delete_shorthand() {
        let engine = tokio::sync::RwLock::new(RuleEngine::new_empty());
        {
            let mut eng = engine.blocking_write();
            eng.add_rule(OutboundRule::new(
                "del-me",
                RuleCondition::Always,
                OutboundAction::Block,
            ));
        }
        let body = r#"{"method":"DELETE","payload":{"label":"del-me"}}"#;
        let response = tokio::runtime::Runtime::new()
            .expect("Runtime::new should succeed in test environment")
            .block_on(RulesApiServer::handle_request(body, &engine));
        let parsed: ApiResponse = serde_json::from_str(&response)
            .expect("serde_json from_str of API response should succeed");
        assert!(parsed.success);
    }
}
