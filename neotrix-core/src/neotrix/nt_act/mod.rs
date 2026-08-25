//! # NT-ACT — Autonomous Communication (行动执行者)
//!
//! 统一自主外部通信能力，提供：
//! - HTTP/WebSocket 客户端（自带重试、熔断、限流）
//! - MCP (Model Context Protocol) 客户端/服务端
//! - ACP (Agent Communication Protocol) 支持
//! - Web 搜索/爬取/浏览（集成 nt_world_browse）
//! - MCP 服务发现与工具调用
//! - 自主决策的工具选择与执行

pub mod client;
pub mod mcp;
pub mod acp;
pub mod search;
pub mod tools;
pub mod decision;
pub mod types;

pub use client::{HttpClient, HttpClientConfig, HttpClientBuilder};
pub use mcp::{McpClient, McpServer, McpConfig, McpTool, McpResource, McpPrompt};
pub use acp::{AcpClient, AcpServer, AcpConfig, AcpMessage, AcpSession};
pub use search::{WebSearch, SearchConfig, SearchResult, SearchEngine};
pub use tools::{Tool, ToolSpec, ToolRegistry, ToolExecutor, ToolResult};
pub use decision::{CommunicationDecision, DecisionContext, DecisionMaker};
pub use types::*;

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::nt_act::{CommunicationConfig, CommunicationIntent, CommunicationDecision, CommunicationMethod, CommunicationCapabilities};

use crate::nt_act::CommunicationConfig;

/// 统一通信入口
pub struct AutonomousCommunicator {
    config: CommunicationConfig,
    http_client: Arc<crate::nt_act::client::HttpClient>,
    mcp_client: Option<Arc<crate::nt_act::mcp::McpClient>>,
    acp_client: Option<Arc<crate::nt_act::acp::AcpClient>>,
    search_engine: Arc<crate::nt_act::search::WebSearch>,
    tool_registry: Arc<crate::nt_act::tools::ToolRegistry>,
    decision_maker: Arc<crate::nt_act::decision::DecisionMaker>,
    circuit_breaker: Arc<tokio::sync::Mutex<HashMap<String, u32>>>,
}

impl AutonomousCommunicator {
    pub fn new(config: CommunicationConfig) -> Self {
        let http_client = Arc::new(crate::nt_act::client::HttpClient::new(config.http.clone()));
        let search_engine = Arc::new(crate::nt_act::search::WebSearch::new(config.search.clone()));
        let tool_registry = Arc::new(crate::nt_act::tools::ToolRegistry::new(config.tools.clone()));
        let decision_maker = Arc::new(crate::nt_act::decision::DecisionMaker::new(config.decision.clone()));
        
        Self {
            config,
            http_client,
            mcp_client: None,
            acp_client: None,
            search_engine,
            tool_registry,
            decision_maker,
            circuit_breaker: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    /// 初始化 MCP 客户端
    pub async fn init_mcp(&mut self, config: crate::nt_act::McpConfig) -> Result<(), String> {
        let client = crate::nt_act::mcp::McpClient::new(config).await?;
        self.mcp_client = Some(Arc::new(client));
        Ok(())
    }

    /// 初始化 ACP 客户端
    pub async fn init_acp(&mut self, config: crate::nt_act::AcpConfig) -> Result<(), String> {
        let client = crate::nt_act::acp::AcpClient::new(config).await?;
        self.acp_client = Some(Arc::new(client));
        Ok(())
    }

    /// 注册工具
    pub fn register_tool(&self, tool: Box<dyn crate::nt_act::tools::Tool>) {
        self.tool_registry.register(tool);
    }

    /// 自主通信主入口：根据目标自主决定通信方式并执行
    pub async fn communicate(&self, intent: crate::nt_act::CommunicationIntent) -> Result<crate::nt_act::CommunicationResult, String> {
        // 熔断检查
        self.check_circuit_breaker(&intent.target).await?;

        // 自主决策：选择最佳通信方式
        let decision = self.decision_maker.decide(&intent, &self.get_capabilities()).await?;

        // 执行通信
        let result = match decision.method {
            crate::nt_act::CommunicationMethod::Http => self.execute_http(&intent, &decision).await,
            crate::nt_act::CommunicationMethod::Mcp => self.execute_mcp(&intent, &decision).await,
            crate::nt_act::CommunicationMethod::Acp => self.execute_acp(&intent, &decision).await,
            crate::nt_act::CommunicationMethod::Search => self.execute_search(&intent, &decision).await,
            crate::nt_act::CommunicationMethod::Tool => self.execute_tool(&intent, &decision).await,
        }.await;

        // 更新熔断器
        self.update_circuit_breaker(&intent.target, result.is_ok()).await;

        result
    }

    /// 批量自主通信
    pub async fn communicate_batch(&self, intents: Vec<crate::nt_act::CommunicationIntent>) -> Vec<Result<crate::nt_act::CommunicationResult, String>> {
        let futures = intents.into_iter().map(|intent| self.communicate(intent));
        futures::future::join_all(futures).await
    }

    async fn execute_http(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) -> Result<crate::nt_act::CommunicationResult, String> {
        let response = self.http_client.request(decision.http_request.clone().unwrap()).await?;
        Ok(crate::nt_act::CommunicationResult::Http(response))
    }

    async fn execute_mcp(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) -> Result<crate::nt_act::CommunicationResult, String> {
        let client = self.mcp_client.as_ref().ok_or("MCP client not initialized")?;
        let result = client.call_tool(decision.mcp_tool.as_ref().unwrap(), decision.mcp_args.clone()).await?;
        Ok(crate::nt_act::CommunicationResult::Mcp(result))
    }

    async fn execute_acp(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) -> Result<crate::nt_act::CommunicationResult, String> {
        let client = self.acp_client.as_ref().ok_or("ACP client not initialized")?;
        let result = client.send_message(decision.acp_message.clone().unwrap()).await?;
        Ok(crate::nt_act::CommunicationResult::Acp(result))
    }

    async fn execute_search(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) -> Result<crate::nt_act::CommunicationResult, String> {
        let results = self.search_engine.search(decision.search_query.as_ref().unwrap(), decision.search_options.clone()).await?;
        Ok(crate::nt_act::CommunicationResult::Search(results))
    }

    async fn execute_tool(&self, intent: &crate::nt_act::CommunicationIntent, decision: &crate::nt_act::CommunicationDecision) -> Result<crate::nt_act::CommunicationResult, String> {
        let result = self.tool_registry.execute(decision.tool_name.as_ref().unwrap(), decision.tool_args.clone().unwrap()).await?;
        Ok(crate::nt_act::CommunicationResult::Tool(result))
    }

    fn get_capabilities(&self) -> crate::nt_act::CommunicationCapabilities {
        crate::nt_act::CommunicationCapabilities {
            http: true,
            mcp: self.mcp_client.is_some(),
            acp: self.acp_client.is_some(),
            search: true,
            tools: self.tool_registry.list_tools(),
        }
    }

    async fn check_circuit_breaker(&self, target: &str) -> Result<(), String> {
        let cb = self.circuit_breaker.lock().await;
        let failures = cb.get(target).copied().unwrap_or(0);
        if failures >= self.config.circuit_breaker_threshold {
            return Err(format!("Circuit breaker open for target: {}", target));
        }
        Ok(())
    }

    async fn update_circuit_breaker(&self, target: &str, success: bool) {
        let mut cb = self.circuit_breaker.lock().await;
        let failures = cb.entry(target.to_string()).or_insert(0);
        if success {
            *failures = 0;
        } else {
            *failures += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_communicator_creation() {
        let config = CommunicationConfig::default();
        let communicator = AutonomousCommunicator::new(config);
        let caps = communicator.get_capabilities();
        assert!(caps.http);
        assert!(caps.search);
    }
}
