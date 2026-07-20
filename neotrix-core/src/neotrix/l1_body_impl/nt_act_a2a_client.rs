use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub skills: Vec<AgentSkill>,
    pub authentication: Option<AgentAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: Option<Value>,
    pub output_schema: Option<Value>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAuth {
    pub scheme: String,
    pub credentials: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ARequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<Value>,
    pub error: Option<A2AError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl A2ARequest {
    pub fn new(id: u64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }

    pub fn send_card_request(id: u64) -> Self {
        Self::new(id, "a2a.agentCard", serde_json::json!({}))
    }

    pub fn send_skill_request(id: u64, skill_id: &str, input: Value) -> Self {
        Self::new(id, "a2a.execute", serde_json::json!({
            "skill_id": skill_id,
            "input": input,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct A2AClientConfig {
    pub base_url: String,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub auth: Option<AgentAuth>,
}

impl Default for A2AClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout_secs: 30,
            retry_count: 3,
            auth: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct A2AClient {
    pub config: A2AClientConfig,
    pub card: Option<AgentCard>,
    next_id: u64,
    pub connected_at: u64,
}

impl A2AClient {
    pub fn new(config: A2AClientConfig) -> Self {
        Self {
            config,
            card: None,
            next_id: 1,
            connected_at: 0,
        }
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn discover_card(&mut self) -> Result<AgentCard, String> {
        let request = A2ARequest::send_card_request(self.next_id());
        let response = self.send_request(&request, &format!("{}/a2a", self.config.base_url))?;
        let result = response.result.ok_or("No result in response".to_string())?;
        let card: AgentCard = serde_json::from_value(result).map_err(|e| format!("Deserialize card: {}", e))?;
        self.card = Some(card.clone());
        self.connected_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Ok(card)
    }

    pub fn execute_skill(&mut self, skill_id: &str, input: Value) -> Result<Value, String> {
        if self.card.is_none() {
            self.discover_card()?;
        }
        let request = A2ARequest::send_skill_request(self.next_id(), skill_id, input);
        let endpoint = self.card.as_ref().map(|c| c.url.clone()).unwrap_or_else(|| self.config.base_url.clone());
        let response = self.send_request(&request, &format!("{}/a2a", endpoint))?;
        response.result.ok_or_else(|| {
            response.error
                .map(|e| format!("A2A error {}: {}", e.code, e.message))
                .unwrap_or_else(|| "No result in response".to_string())
        })
    }

    pub fn list_skills(&self) -> Vec<AgentSkill> {
        self.card.as_ref().map(|c| c.skills.clone()).unwrap_or_default()
    }

    pub fn find_skill(&self, skill_id: &str) -> Option<&AgentSkill> {
        self.card.as_ref().and_then(|c| c.skills.iter().find(|s| s.id == skill_id))
    }

    fn send_request(&self, request: &A2ARequest, url: &str) -> Result<A2AResponse, String> {
        let body = serde_json::to_string(request).map_err(|e| format!("Serialize: {}", e))?;
        let client = reqwest::blocking::Client::new();
        let mut req_builder = client.post(url).header("Content-Type", "application/json");
        if let Some(auth) = &self.config.auth {
            req_builder = req_builder.header("Authorization", &format!("{} {}", auth.scheme, auth.credentials));
        }
        match req_builder.body(body).send() {
            Ok(resp) => {
                let text = resp.text().map_err(|e| format!("Read response: {}", e))?;
                serde_json::from_str(&text).map_err(|e| format!("Deserialize: {}", e))
            }
            Err(e) => Err(format!("HTTP error: {}", e)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct A2ARegistry {
    clients: HashMap<String, A2AClient>,
}

impl A2ARegistry {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, config: A2AClientConfig) -> &mut A2AClient {
        self.clients.entry(name.to_string()).or_insert_with(|| A2AClient::new(config))
    }

    pub fn get(&self, name: &str) -> Option<&A2AClient> {
        self.clients.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut A2AClient> {
        self.clients.get_mut(name)
    }

    pub fn discover_all(&mut self) -> Vec<(String, Result<AgentCard, String>)> {
        let mut results = Vec::new();
        let names: Vec<String> = self.clients.keys().cloned().collect();
        for name in &names {
            if let Some(client) = self.clients.get_mut(name.as_str()) {
                let result = client.discover_card();
                results.push((name.clone(), result));
            }
        }
        results
    }

    pub fn broadcast(&mut self, skill_id: &str, input: Value) -> Vec<(String, Result<Value, String>)> {
        let mut results = Vec::new();
        let names: Vec<String> = self.clients.keys().cloned().collect();
        for name in &names {
            if let Some(client) = self.clients.get_mut(name.as_str()) {
                let result = client.execute_skill(skill_id, input.clone());
                results.push((name.clone(), result));
            }
        }
        results
    }
}

impl Default for A2ARegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_card_serde() {
        let card = AgentCard {
            name: "TestAgent".to_string(),
            description: "A test agent".to_string(),
            url: "http://localhost".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["chat".to_string(), "code".to_string()],
            skills: vec![AgentSkill {
                id: "skill-1".to_string(),
                name: "Greet".to_string(),
                description: "Greets the user".to_string(),
                input_schema: None,
                output_schema: None,
                tags: vec!["utility".to_string()],
            }],
            authentication: None,
        };
        let json = serde_json::to_string(&card).unwrap();
        let deserialized: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "TestAgent");
        assert_eq!(deserialized.skills.len(), 1);
    }

    #[test]
    fn test_a2a_request_card() {
        let req = A2ARequest::send_card_request(1);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "a2a.agentCard");
        assert_eq!(req.id, 1);
    }

    #[test]
    fn test_a2a_request_skill() {
        let req = A2ARequest::send_skill_request(2, "greet", serde_json::json!({"name": "World"}));
        assert_eq!(req.method, "a2a.execute");
        assert_eq!(req.params["skill_id"], "greet");
        assert_eq!(req.params["input"]["name"], "World");
    }

    #[test]
    fn test_client_next_id_increments() {
        let mut client = A2AClient::new(A2AClientConfig::default());
        assert_eq!(client.next_id(), 1);
        assert_eq!(client.next_id(), 2);
        assert_eq!(client.next_id(), 3);
    }

    #[test]
    fn test_client_discover_card_not_connected() {
        let mut client = A2AClient::new(A2AClientConfig::default());
        assert!(client.card.is_none());
        assert_eq!(client.connected_at, 0);
    }

    #[test]
    fn test_client_find_skill_before_discovery() {
        let mut client = A2AClient::new(A2AClientConfig::default());
        assert!(client.list_skills().is_empty());
        assert!(client.find_skill("nonexistent").is_none());
    }

    #[test]
    fn test_a2a_error_serde() {
        let error = A2AError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: A2AError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, -32601);
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = A2ARegistry::new();
        registry.register("agent-a", A2AClientConfig {
            base_url: "http://agent-a:8080".to_string(),
            ..Default::default()
        });
        registry.register("agent-b", A2AClientConfig {
            base_url: "http://agent-b:8080".to_string(),
            ..Default::default()
        });
        assert!(registry.get("agent-a").is_some());
        assert!(registry.get("agent-b").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_broadcast_no_clients() {
        let mut registry = A2ARegistry::new();
        let results = registry.broadcast("test-skill", serde_json::json!({}));
        assert!(results.is_empty());
    }

    #[test]
    fn test_skill_filtering() {
        let card = AgentCard {
            name: "Worker".to_string(),
            description: "Worker agent".to_string(),
            url: "http://worker".to_string(),
            version: "1.0".to_string(),
            capabilities: vec![],
            skills: vec![
                AgentSkill { id: "code".to_string(), name: "Code Gen".to_string(), description: "Generate code".to_string(), input_schema: None, output_schema: None, tags: vec!["code".to_string()] },
                AgentSkill { id: "search".to_string(), name: "Web Search".to_string(), description: "Search web".to_string(), input_schema: None, output_schema: None, tags: vec!["search".to_string()] },
            ],
            authentication: None,
        };
        let mut client = A2AClient::new(A2AClientConfig::default());
        client.card = Some(card);
        assert_eq!(client.list_skills().len(), 2);
        assert!(client.find_skill("code").is_some());
        assert!(client.find_skill("search").is_some());
        assert!(client.find_skill("nonexistent").is_none());
    }

    #[test]
    fn test_a2a_response_serde() {
        let resp = A2AResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: A2AResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, Some(1));
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_a2a_response_error() {
        let resp = A2AResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(1),
            result: None,
            error: Some(A2AError { code: -32000, message: "Server error".to_string(), data: None }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: A2AResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.result.is_none());
        assert!(deserialized.error.is_some());
        assert_eq!(deserialized.error.unwrap().code, -32000);
    }
}