use std::time::Duration;

use crate::neotrix::nt_agent_protocol::a2a_negotiation::{
    CapabilityVector, NegotiationOffer, NegotiationResponse, ProtocolBinding, ProtocolVersion,
};

use super::types::{
    A2ATask, AgentCard, CancelTaskResponse, GetTaskResponse, SendTaskRequest, SendTaskResponse,
    TaskEvent,
};

// ── A2A Client ─────────────────────────────────────────────────────────────

pub struct A2AClient {
    base_url: String,
    client: reqwest::Client,
    pub negotiated_version: Option<ProtocolVersion>,
    pub negotiated_binding: Option<ProtocolBinding>,
    pub negotiated_capabilities: Option<CapabilityVector>,
}

impl A2AClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("A2A client: reqwest TLS/networking backend failed to initialize"),
            negotiated_version: None,
            negotiated_binding: None,
            negotiated_capabilities: None,
        }
    }

    pub async fn connect(base_url: &str, agent_name: &str) -> Result<Self, String> {
        let mut client = Self::new(base_url);
        let card = client.fetch_agent_card().await?;
        let negotiation_endpoint = card
            .negotiation_endpoint
            .unwrap_or_else(|| "/.well-known/negotiate".into());
        if let Some(version) = card.version.parse::<ProtocolVersion>().ok() {
            let offer = NegotiationOffer::new(
                vec![version],
                vec![ProtocolBinding::HttpJsonRest],
                CapabilityVector::all(),
                agent_name,
            );
            let resp = client
                .client
                .post(&format!("{}{}", client.base_url, negotiation_endpoint))
                .json(&offer)
                .send()
                .await
                .map_err(|e| format!("negotiate: {}", e))?;
            let negotiation: NegotiationResponse = resp
                .json()
                .await
                .map_err(|e| format!("parse negotiation: {}", e))?;
            if !negotiation.accepted {
                return Err(format!(
                    "negotiation rejected: no common version/binding with {}",
                    card.name
                ));
            }
            client.negotiated_version = Some(negotiation.selected_version);
            client.negotiated_binding = Some(negotiation.selected_binding);
            client.negotiated_capabilities = Some(negotiation.common_capabilities);
        }
        Ok(client)
    }

    pub async fn fetch_agent_card(&self) -> Result<AgentCard, String> {
        let url = format!("{}/.well-known/agent-card", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("fetch card: {}", e))?;
        resp.json().await.map_err(|e| format!("parse card: {}", e))
    }

    pub async fn negotiate(
        &mut self,
    ) -> Result<(ProtocolVersion, ProtocolBinding, CapabilityVector), String> {
        let card = self.fetch_agent_card().await?;
        let negotiation_endpoint = card
            .negotiation_endpoint
            .unwrap_or_else(|| "/.well-known/negotiate".into());
        let version = card
            .version
            .parse::<ProtocolVersion>()
            .map_err(|e| format!("invalid agent version: {}", e))?;
        let offer = NegotiationOffer::new(
            vec![version],
            vec![ProtocolBinding::HttpJsonRest],
            CapabilityVector::all(),
            "neotrix",
        );
        let resp = self
            .client
            .post(&format!("{}{}", self.base_url, negotiation_endpoint))
            .json(&offer)
            .send()
            .await
            .map_err(|e| format!("negotiate: {}", e))?;
        let negotiation: NegotiationResponse = resp
            .json()
            .await
            .map_err(|e| format!("parse negotiation: {}", e))?;
        if !negotiation.accepted {
            return Err(format!("negotiation rejected by {}", card.name));
        }
        self.negotiated_version = Some(negotiation.selected_version);
        self.negotiated_binding = Some(negotiation.selected_binding);
        self.negotiated_capabilities = Some(negotiation.common_capabilities);
        Ok((
            negotiation.selected_version,
            negotiation.selected_binding,
            negotiation.common_capabilities,
        ))
    }

    pub async fn send_task(&self, request: SendTaskRequest) -> Result<A2ATask, String> {
        if self.negotiated_version.is_none() {
            return Err("A2AClient: send_task requires prior negotiate() call".into());
        }
        let url = format!("{}/tasks/send", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("send task: {}", e))?;
        let status = resp.status();
        let body: SendTaskResponse = resp
            .json()
            .await
            .map_err(|e| format!("parse response: {}", e))?;
        if !status.is_success() {
            return Err(format!("A2A server returned {}", status));
        }
        Ok(body.task)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<A2ATask, String> {
        let url = format!("{}/tasks/{}", self.base_url, task_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("get task: {}", e))?;
        let body: GetTaskResponse = resp
            .json()
            .await
            .map_err(|e| format!("parse task: {}", e))?;
        Ok(body.task)
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<A2ATask, String> {
        let url = format!("{}/tasks/{}/cancel", self.base_url, task_id);
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| format!("cancel task: {}", e))?;
        let body: CancelTaskResponse = resp
            .json()
            .await
            .map_err(|e| format!("parse cancel: {}", e))?;
        Ok(body.task)
    }

    pub async fn stream_tasks(
        &self,
        task_id: &str,
    ) -> Result<impl futures::Stream<Item = Result<TaskEvent, String>>, String> {
        let url = format!("{}/tasks/{}/stream", self.base_url, task_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("stream: {}", e))?;
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("read body: {}", e))?;
        let text = String::from_utf8_lossy(&body);
        let mut events = Vec::new();
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(event) = serde_json::from_str::<TaskEvent>(data) {
                    events.push(Ok(event));
                }
            }
        }
        Ok(futures::stream::iter(events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2a_client_creation() {
        let client = A2AClient::new("http://localhost:42069");
        // No network call — just verify construction
        assert!(client.base_url.contains("localhost"));
    }
}
