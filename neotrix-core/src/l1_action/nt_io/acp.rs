//! ACP (Agent Client Protocol) — IDE 集成协议
//!
//! JSON-RPC over stdin/stdout，用于 Zed/Neovim/Emacs 集成。

use std::collections::HashMap;

#[allow(dead_code)]

/// JSON-RPC 请求
pub struct AcpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// JSON-RPC 响应
pub struct AcpResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<AcpError>,
}

/// JSON-RPC 错误
#[allow(dead_code)]
pub struct AcpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// ACP 服务器
#[allow(dead_code)]
pub struct AcpServer {
    handlers: HashMap<String, Box<dyn Fn(&serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>>,
}

#[allow(dead_code)]
impl AcpServer {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    /// 注册方法处理器
    pub fn register_method(&mut self, method: &str, handler: impl Fn(&serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync + 'static) {
        self.handlers.insert(method.to_string(), Box::new(handler));
    }

    /// 处理请求
    pub fn handle_request(&self, method: &str, params: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.handlers.get(method)
            .ok_or_else(|| format!("Method not found: {}", method))
            .and_then(|h| h(params))
    }

    /// 获取支持的方法列表
    pub fn methods(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
}

/// 创建默认 ACP 服务器
#[allow(dead_code)]
pub fn create_default_acp_server() -> AcpServer {
    let mut server = AcpServer::new();

    server.register_method("ping", |_params| {
        Ok(serde_json::json!({"pong": true}))
    });

    server.register_method("capabilities", |_params| {
        Ok(serde_json::json!({
            "tools": true,
            "context": true,
            "memory": true
        }))
    });

    server
}
