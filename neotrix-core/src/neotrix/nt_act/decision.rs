//! Decision Making for Autonomous Communication

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::{DecisionConfig, CommunicationIntent, CommunicationDecision, CommunicationMethod, CommunicationCapabilities};

/// Decision Maker for Autonomous Communication
pub struct DecisionMaker {
    config: crate::nt_act::DecisionConfig,
    history: Arc<Mutex<Vec<DecisionRecord>>>,
    learning_model: Arc<Mutex<Option<DecisionModel>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecisionRecord {
    intent: crate::nt_act::CommunicationIntent,
    decision: CommunicationDecision,
    outcome: DecisionOutcome,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum DecisionOutcome {
    Success,
    Failure,
    Partial,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DecisionModel {
    weights: HashMap<String, f64>,
    performance: HashMap<String, f64>,
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl DecisionMaker {
    pub fn new(config: crate::nt_act::DecisionConfig) -> Self {
        Self {
            config,
            history: Arc::new(Mutex::new(Vec::new())),
            learning_model: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn decide(&self, intent: &crate::nt_act::CommunicationIntent, capabilities: &CommunicationCapabilities) -> Result<crate::nt_act::CommunicationDecision, String> {
        // Score each available method
        let mut scores = HashMap::new();
        
        // HTTP scoring
        if capabilities.http {
            scores.insert(CommunicationMethod::Http, self.score_http(&intent).await);
        }
        
        // MCP scoring
        if capabilities.mcp {
            scores.insert(CommunicationMethod::Mcp, self.score_mcp(&intent).await);
        }
        
        // ACP scoring
        if capabilities.acp {
            scores.insert(CommunicationMethod::Acp, self.score_acp(&intent).await);
        }
        
        // Search scoring
        if capabilities.search {
            scores.insert(CommunicationMethod::Search, self.score_search(&intent).await);
        }
        
        // Tool scoring
        if !capabilities.tools.is_empty() {
            scores.insert(CommunicationMethod::Tool, self.score_tool(&intent).await);
        }
        
        // Select best method
        let best_method = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(m, _)| *m)
            .unwrap_or(CommunicationMethod::Http);
        
        let confidence = scores.get(&best_method).copied().unwrap_or(0.5);
        
        if confidence < self.config.min_confidence {
            // Fallback to most reliable method
            return Ok(self.fallback_decision(&intent, capabilities));
        }
        
        // Build decision
        let decision = self.build_decision(&best_method, &intent).await?;
        
        // Record decision for learning
        self.record_decision(&intent, &decision).await;
        
        Ok(decision)
    }

    async fn score_http(&self, intent: &crate::nt_act::CommunicationIntent) -> f64 {
        let mut score = 0.5;
        
        // HTTP is universal
        score += 0.2;
        
        // Prefer for REST APIs
        if intent.action.contains("api") || intent.action.contains("http") || intent.action.contains("rest") {
            score += 0.3;
        }
        
        // Prefer for known HTTP endpoints
        if intent.target.starts_with("http") || intent.target.contains("://") {
            score += 0.2;
        }
        
        score.min(1.0)
    }

    async fn score_mcp(&self, intent: &crate::nt_act::CommunicationIntent) -> f64 {
        let mut score = 0.0;
        
        if !self.config.prefer_mcp {
            return 0.0;
        }
        
        // Prefer for tool-like operations
        if intent.action.contains("tool") || intent.action.contains("call") || intent.action.contains("execute") {
            score += 0.4;
        }
        
        // Prefer for structured operations
        if intent.action.contains("list") || intent.action.contains("get") || intent.action.contains("create") {
            score += 0.2;
        }
        
        // Check if target looks like an MCP server
        if intent.target.contains("mcp") || intent.target.contains("tool") {
            score += 0.3;
        }
        
        score.min(1.0)
    }

    async fn score_acp(&self, intent: &crate::nt_act::CommunicationIntent) -> f64 {
        let mut score = 0.0;
        
        // ACP for agent-to-agent communication
        if intent.action.contains("agent") || intent.action.contains("message") || intent.action.contains("communicate") {
            score += 0.4;
        }
        
        // For multi-agent scenarios
        if intent.metadata.get("multi_agent").map(|v| v == "true").unwrap_or(false) {
            score += 0.3;
        }
        
        score.min(1.0)
    }

    async fn score_search(&self, intent: &crate::nt_act::CommunicationIntent) -> f64 {
        let mut score = 0.0;
        
        if !self.config.prefer_search {
            return 0.0;
        }
        
        // Search for information retrieval
        if intent.action.contains("search") || intent.action.contains("find") || intent.action.contains("lookup") || intent.action.contains("query") {
            score += 0.5;
        }
        
        // For information gathering
        if intent.action.contains("research") || intent.action.contains("investigate") || intent.action.contains("gather") {
            score += 0.3;
        }
        
        // For unknown targets - search as fallback
        if intent.target.len() < 50 && !intent.target.starts_with("http") {
            score += 0.2;
        }
        
        score.min(1.0)
    }

    async fn score_tool(&self, intent: &crate::nt_act::CommunicationIntent) -> f64 {
        let mut score = 0.0;
        
        // Tool execution for specific operations
        let tool_keywords = ["file", "shell", "code", "json", "http", "search", "calculate", "process", "transform", "execute", "run"];
        for keyword in &tool_keywords {
            if intent.action.contains(keyword) || intent.payload.to_string().contains(keyword) {
                score += 0.15;
                break;
            }
        }
        
        // For local operations
        if intent.target == "local" || intent.target == "localhost" || intent.target.starts_with("file://") {
            score += 0.4;
        }
        
        score.min(1.0)
    }

    fn fallback_decision(&self, intent: &crate::nt_act::CommunicationIntent, capabilities: &CommunicationCapabilities) -> crate::nt_act::CommunicationDecision {
        // Priority: MCP > HTTP > Search > Tool > ACP
        if capabilities.mcp {
            self.build_decision_sync(CommunicationMethod::Mcp, intent)
        } else if capabilities.http {
            self.build_decision_sync(CommunicationMethod::Http, intent)
        } else if capabilities.search {
            self.build_decision_sync(CommunicationMethod::Search, intent)
        } else if !capabilities.tools.is_empty() {
            self.build_decision_sync(CommunicationMethod::Tool, intent)
        } else if capabilities.acp {
            self.build_decision_sync(CommunicationMethod::Acp, intent)
        } else {
            crate::nt_act::CommunicationDecision {
                method: CommunicationMethod::Http,
                confidence: 0.1,
                reasoning: "No suitable method found, defaulting to HTTP".to_string(),
                http_request: None,
                mcp_tool: None,
                mcp_args: None,
                acp_message: None,
                search_query: None,
                search_options: None,
                tool_name: None,
                tool_args: None,
            }
        }
    }

    fn build_decision_sync(&self, method: CommunicationMethod, intent: &crate::nt_act::CommunicationIntent) -> crate::nt_act::CommunicationDecision {
        let mut decision = crate::nt_act::CommunicationDecision {
            method,
            confidence: 0.5,
            reasoning: format!("Selected {} as fallback", method as u8),
            http_request: None,
            mcp_tool: None,
            mcp_args: None,
            acp_message: None,
            search_query: None,
            search_options: None,
            tool_name: None,
            tool_args: None,
        };

        match method {
            crate::nt_act::CommunicationMethod::Http => {
                decision.http_request = Some(crate::nt_act::client::HttpRequest {
                    method: "POST".to_string(),
                    url: intent.target.clone(),
                    headers: HashMap::new(),
                    body: Some(serde_json::to_string(&intent.payload).unwrap_or_default()),
                    query: None,
                });
                decision.reasoning = "Using HTTP for general communication".to_string();
            }
            crate::nt_act::CommunicationMethod::Mcp => {
                decision.mcp_tool = Some(intent.action.clone());
                decision.mcp_args = Some(intent.payload.clone());
                decision.reasoning = "Using MCP for structured tool calls".to_string();
            }
            crate::nt_act::CommunicationMethod::Search => {
                decision.search_query = Some(format!("{} {}", intent.action, intent.target));
                decision.search_options = Some(crate::nt_act::search::SearchOptions::default());
                decision.reasoning = "Using search for information retrieval".to_string();
            }
            crate::nt_act::CommunicationMethod::Tool => {
                decision.tool_name = Some("shell_command".to_string());
                decision.tool_args = Some(serde_json::json!({
                    "command": "echo",
                    "args": [format!("{}: {}", intent.action, intent.target)]
                }));
                decision.reasoning = "Using local tool execution".to_string();
            }
            crate::nt_act::CommunicationMethod::Acp => {
                decision.acp_message = Some(crate::nt_act::acp::AcpMessage {
                    from: "neotrix".to_string(),
                    to: intent.target.clone(),
                    content: intent.payload.to_string(),
                    metadata: intent.metadata.clone(),
                });
                decision.reasoning = "Using ACP for agent communication".to_string();
            }
        }
        
        decision
    }

    async fn build_decision(&self, method: &CommunicationMethod, intent: &crate::nt_act::CommunicationIntent) -> Result<crate::nt_act::CommunicationDecision, String> {
        let mut decision = crate::nt_act::CommunicationDecision {
            method: *method,
            confidence: 0.7,
            reasoning: String::new(),
            http_request: None,
            mcp_tool: None,
            mcp_args: None,
            acp_message: None,
            search_query: None,
            search_options: None,
            tool_name: None,
            tool_args: None,
        };

        match method {
            CommunicationMethod::Http => {
                decision.http_request = Some(self.build_http_request(intent)?);
                decision.reasoning = format!("Using HTTP to communicate with {}", intent.target);
            }
            CommunicationMethod::Mcp => {
                decision.mcp_tool = Some(intent.action.clone());
                decision.mcp_args = Some(intent.payload.clone());
                decision.reasoning = format!("Using MCP to call tool {}", intent.action);
            }
            CommunicationMethod::Acp => {
                decision.acp_message = Some(crate::nt_act::acp::AcpMessage {
                    from: "neotrix".to_string(),
                    to: intent.target.clone(),
                    content: serde_json::to_string(&intent.payload).unwrap_or_default(),
                    metadata: intent.metadata.clone(),
                });
                decision.reasoning = format!("Using ACP to communicate with agent {}", intent.target);
            }
            CommunicationMethod::Search => {
                decision.search_query = Some(format!("{} {}", intent.action, intent.target));
                decision.search_options = Some(crate::nt_act::search::SearchOptions::default());
                decision.reasoning = format!("Searching for information about {}", intent.target);
            }
            CommunicationMethod::Tool => {
                decision.tool_name = Some(self.select_tool(intent)?);
                decision.tool_args = Some(intent.payload.clone());
                decision.reasoning = format!("Using local tool for {}", intent.action);
            }
        }

        Ok(decision)
    }

    fn build_http_request(&self, intent: &crate::nt_act::CommunicationIntent) -> Result<crate::nt_act::client::HttpRequest, String> {
        Ok(crate::nt_act::client::HttpRequest {
            method: "POST".to_string(),
            url: intent.target.clone(),
            headers: HashMap::new(),
            body: Some(serde_json::to_string(&intent.payload).unwrap_or_default()),
            query: None,
        })
    }

    fn select_tool(&self, intent: &crate::nt_act::CommunicationIntent) -> Result<String, String> {
        // Select best tool based on intent
        if intent.action.contains("file") || intent.payload.get("path").is_some() {
            Ok("file_operation".to_string())
        } else if intent.action.contains("shell") || intent.action.contains("command") || intent.action.contains("exec") {
            Ok("shell_command".to_string())
        } else if intent.action.contains("json") || intent.action.contains("parse") {
            Ok("json_processing".to_string())
        } else if intent.action.contains("code") || intent.action.contains("run") || intent.action.contains("execute") {
            Ok("code_execution".to_string())
        } else if intent.action.contains("search") || intent.action.contains("find") || intent.action.contains("query") {
            Ok("http_request".to_string())
        } else if intent.action.contains("knowledge") || intent.action.contains("query") || intent.action.contains("kb") {
            Ok("knowledge_query".to_string())
        } else {
            Ok("shell_command".to_string())
        }
    }

    async fn record_decision(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) {
        let mut history = self.history.lock().await;
        history.push(DecisionRecord {
            intent: intent.clone(),
            decision: decision.clone(),
            outcome: DecisionOutcome::Success, // Will be updated after execution
            timestamp: chrono::Utc::now(),
        });
        
        // Keep last 1000 decisions
        if history.len() > 1000 {
            history.drain(0..history.len() - 1000);
        }
    }

    pub async fn record_outcome(&self, intent: &crate::nt_act::CommunicationIntent, outcome: DecisionOutcome) {
        let mut history = self.history.lock().await;
        if let Some(record) = history.iter_mut().rev().find(|r| r.intent.target == intent.target && r.intent.action == intent.action) {
            record.outcome = outcome;
        }
        
        // Update learning model if enabled
        if self.config.enable_learning {
            self.update_learning_model().await;
        }
    }

    async fn update_learning_model(&self) {
        let history = self.history.lock().await;
        let mut model = self.learning_model.lock().await;
        
        let mut model = model.get_or_insert_with(|| DecisionModel {
            weights: HashMap::new(),
            performance: HashMap::new(),
            last_updated: chrono::Utc::now(),
        });
        
        // Simple weight update based on outcomes
        for record in history.iter() {
            let method_key = format!("{:?}", record.decision.method);
            let perf = model.performance.entry(method_key.clone()).or_insert(0.0);
            
            match record.outcome {
                DecisionOutcome::Success => *perf = (*perf * 0.9 + 0.1).min(1.0),
                DecisionOutcome::Failure => *perf = (*perf * 0.9).max(0.0),
                DecisionOutcome::Partial => *perf = (*perf * 0.9 + 0.05).min(1.0),
                DecisionOutcome::Timeout => *perf = (*perf * 0.95).max(0.0),
            }
            
            // Update weights based on performance
            model.weights.insert(method_key, *perf);
        }
        
        model.last_updated = chrono::Utc::now();
    }

    pub async fn get_decision_stats(&self) -> HashMap<String, f64> {
        let history = self.history.lock().await;
        let mut stats = HashMap::new();
        
        for record in history.iter() {
            let method = format!("{:?}", record.decision.method);
            let entry = stats.entry(method).or_insert(0.0);
            match record.outcome {
                DecisionOutcome::Success => *entry += 1.0,
                DecisionOutcome::Failure => *entry -= 1.0,
                _ => *entry += 0.5,
            }
        }
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_decision_maker() {
        let config = crate::nt_act::DecisionConfig::default();
        let maker = DecisionMaker::new(config);
        
        let intent = crate::nt_act::CommunicationIntent {
            target: "https://api.example.com".to_string(),
            action: "get_data".to_string(),
            payload: serde_json::json!({}),
            metadata: HashMap::new(),
            priority: crate::nt_act::Priority::Normal,
            timeout_secs: None,
        };
        
        let capabilities = crate::nt_act::CommunicationCapabilities {
            http: true,
            mcp: false,
            acp: false,
            search: true,
            tools: vec![],
        };
        
        let decision = maker.decide(&intent, &capabilities).await.unwrap();
        assert_eq!(decision.method, crate::nt_act::CommunicationMethod::Http);
    }
}


#[derive(Debug, Clone)]
pub struct DecisionContext { pub target: String }

#[derive(Debug, Clone)]
pub struct CommunicationDecision {
    pub method: crate::neotrix::nt_act::types::CommunicationMethod,
    pub confidence: f32,
}
