use serde::{Deserialize, Serialize};

/// AiToEarn API 配置
#[derive(Clone, Debug)]
pub struct AiToEarnConfig {
    pub api_key: String,
    pub base_url: String,
}

impl Default for AiToEarnConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://aitoearn.ai/api/unified".to_string(),
        }
    }
}

/// 发布请求
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub title: String,
    pub content: String,
    pub content_type: String,
    pub platforms: Vec<String>,
    pub schedule_time: Option<String>,
    pub media_urls: Vec<String>,
}

/// 发布结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishResult {
    pub success: bool,
    pub platform_results: Vec<PlatformResult>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformResult {
    pub platform: String,
    pub success: bool,
    pub post_url: Option<String>,
    pub error: Option<String>,
}

/// 变现任务信息
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub title: String,
    pub reward_type: String,
    pub reward_amount: f64,
    pub requirements: String,
    pub deadline: Option<String>,
}

/// 收益报告
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EarningsReport {
    pub total_earnings: f64,
    pub platform_breakdown: Vec<PlatformEarning>,
    pub pending_tasks: Vec<TaskInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformEarning {
    pub platform: String,
    pub amount: f64,
    pub currency: String,
}

/// AiToEarn MCP 桥接客户端 — 现已接入完整 publish flow
pub struct AiToEarnBridge {
    config: AiToEarnConfig,
    client: reqwest::blocking::Client,
}

impl AiToEarnBridge {
    pub fn new(config: AiToEarnConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    pub fn from_env() -> Self {
        let api_key = std::env::var("AITO_EARN_API_KEY").unwrap_or_default();
        let base_url = std::env::var("AITO_EARN_BASE_URL")
            .unwrap_or_else(|_| "https://aitoearn.ai/api/unified".to_string());
        Self::new(AiToEarnConfig { api_key, base_url })
    }

    pub fn is_configured(&self) -> bool {
        !self.config.api_key.is_empty()
    }

    fn mcp_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": method, "params": params,
        });
        let url = format!("{}/mcp", self.config.base_url);
        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("AiToEarn request failed: {}", e))?;
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| format!("AiToEarn parse failed: {}", e))?;
        if let Some(err) = json.get("error") {
            return Err(format!("AiToEarn error: {}", err));
        }
        Ok(json.get("result").cloned().unwrap_or(json))
    }

    /// 发布内容到平台（替代或补充本地 publisher）
    pub fn publish_content(&self, req: &PublishRequest) -> Result<PublishResult, String> {
        let params = serde_json::json!({
            "title": req.title, "content": req.content,
            "content_type": req.content_type, "platforms": req.platforms,
            "schedule_time": req.schedule_time, "media_urls": req.media_urls,
        });
        let result = self.mcp_request("publish_content", params)?;
        serde_json::from_value(result).map_err(|e| format!("PublishResult parse failed: {}", e))
    }

    /// 获取可用变现任务
    pub fn list_tasks(&self) -> Result<Vec<TaskInfo>, String> {
        let result = self.mcp_request("list_tasks", serde_json::json!({}))?;
        serde_json::from_value(result).map_err(|e| format!("TaskInfo parse failed: {}", e))
    }

    /// 提交内容完成变现任务
    pub fn submit_content(&self, task_id: &str, content_url: &str) -> Result<bool, String> {
        let params = serde_json::json!({"task_id": task_id, "content_url": content_url});
        let result = self.mcp_request("submit_content", params)?;
        Ok(result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    /// 检查收益 — 返回真实收益数据（替代 hardcoded $0.05）
    pub fn check_earnings(&self) -> Result<EarningsReport, String> {
        let result = self.mcp_request("check_earnings", serde_json::json!({}))?;
        serde_json::from_value(result).map_err(|e| format!("EarningsReport parse failed: {}", e))
    }

    /// 获取已连接平台列表
    pub fn list_platforms(&self) -> Result<Vec<String>, String> {
        let result = self.mcp_request("list_platforms", serde_json::json!({}))?;
        serde_json::from_value(result).map_err(|e| format!("Platform list parse failed: {}", e))
    }

    /// 获取真实收益 → 转换为推送至 ReasoningBrain 的 RewardSignal
    pub fn fetch_real_earnings_reward(&self) -> Result<(f64, Vec<(String, f64)>), String> {
        let report = self.check_earnings()?;
        let breakdown: Vec<(String, f64)> = report
            .platform_breakdown
            .iter()
            .map(|p| (p.platform.clone(), p.amount))
            .collect();
        Ok((report.total_earnings, breakdown))
    }
}
