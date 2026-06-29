use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::card::AgentCard;

/// Registration request sent by agents to the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub card: AgentCard,
    pub transport: TransportInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportInfo {
    pub host: String,
    pub port: u16,
    pub protocol: String,
}

/// Search query to find agents by skill/capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSearch {
    pub skill: Option<String>,
    pub tag: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSearchResult {
    pub agents: Vec<AgentCard>,
}

/// Registry client — an agent uses this to register with a central registry.
pub struct RegistryClient {
    registry_url: String,
    client: reqwest::Client,
}

impl RegistryClient {
    pub fn new(registry_url: &str) -> Self {
        Self {
            registry_url: registry_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest::Client::builder() should not fail with default settings"),
        }
    }

    pub async fn heartbeat(&self, agent_name: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(format!("{}/agents/{agent_name}/heartbeat", self.registry_url))
            .send()
            .await
            .map_err(|e| format!("heartbeat request failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("heartbeat rejected (status {})", resp.status()))
        }
    }

    pub async fn register(
        &self,
        card: &AgentCard,
        transport: &TransportInfo,
    ) -> Result<(), String> {
        let registration = AgentRegistration {
            card: card.clone(),
            transport: transport.clone(),
        };
        let resp = self
            .client
            .post(format!("{}/agents/register", self.registry_url))
            .json(&registration)
            .send()
            .await
            .map_err(|e| format!("registration request failed: {e}"))?;
        if resp.status().is_success() {
            tracing::info!("registered agent '{}' with registry", card.name);
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(format!("registry rejected registration: {body}"))
        }
    }

    pub async fn unregister(&self, agent_name: &str) -> Result<(), String> {
        let resp = self
            .client
            .post(format!("{}/agents/unregister/{agent_name}", self.registry_url))
            .send()
            .await
            .map_err(|e| format!("unregister request failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("unregister failed: {}", resp.text().await.unwrap_or_default()))
        }
    }

    pub async fn search_by_skill(&self, skill_id: &str) -> Result<Vec<AgentCard>, String> {
        let resp = self
            .client
            .get(format!(
                "{}/agents/search?skill={skill_id}",
                self.registry_url
            ))
            .send()
            .await
            .map_err(|e| format!("search request failed: {e}"))?;
        let result: AgentSearchResult = resp
            .json()
            .await
            .map_err(|e| format!("search response parse failed: {e}"))?;
        Ok(result.agents)
    }

    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<AgentCard>, String> {
        let resp = self
            .client
            .get(format!("{}/agents/search?tag={tag}", self.registry_url))
            .send()
            .await
            .map_err(|e| format!("search request failed: {e}"))?;
        let result: AgentSearchResult = resp
            .json()
            .await
            .map_err(|e| format!("search response parse failed: {e}"))?;
        Ok(result.agents)
    }

    pub async fn list_all(&self) -> Result<Vec<AgentCard>, String> {
        let resp = self
            .client
            .get(format!("{}/agents", self.registry_url))
            .send()
            .await
            .map_err(|e| format!("list request failed: {e}"))?;
        let result: AgentSearchResult = resp
            .json()
            .await
            .map_err(|e| format!("list response parse failed: {e}"))?;
        Ok(result.agents)
    }
}
