//! MCP Bridge — Model Context Protocol 桥接
//!
//! 暴露 NeoTrix 工具给外部 MCP 客户端
//! 调用外部 MCP 服务器的工具

use serde::{Deserialize, Serialize};
// ⭐ 2026-10-03：接入 waterfall 扩展点（兑现该文件自称的 R-P79，见 tool_hooks 字段注释）。
use crate::l5_cognition::nt_core_dispatch::Dispatcher;
use std::collections::HashMap;

/// MCP 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// MCP 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// MCP 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// MCP Bridge — 只做 IO 桥接，不做能力注册。
///
/// 在本地工具与外部 MCP 服务器之间转发调用；能力注册由传输层注册表负责，
/// 本桥不做能力注册。
pub struct McpBridge {
    /// 已注册的本地工具 (供外部调用)
    local_tools: HashMap<String, McpTool>,
    /// 外部 MCP 服务器
    servers: HashMap<String, McpServer>,
    /// ⭐⭐ 工具调用 waterfall 钩子链（2026-10-03 接线）
    ///
    /// ⭐ **兑现 `nt_core_dispatch.rs` 文件头自称的 R-P79**：
    /// 该文件（**354 行**，吸收自 deepseek-harness `vendor/cordis/src/events.ts`）
    /// 写着「NeoTrix 消费方 (R-P79): McpServer 工具调用 pre/post 钩子
    /// (Waterfall 中间件链)」。
    /// ⛔ **但实测那 174 处 R-P79 注释里，声明与实现脱节的不止一处**：
    ///   全仓对 `nt_core_dispatch` 的引用只有 `mod.rs`（声明）与
    ///   `l0_substrate/nt_core_event_bus.rs`（字段类型），**本文件零引用**
    ///   ⇒ 那句 R-P79 是**未兑现的声明**（`AGENTS.md` §4.4「导出 ≠ 接入」）。
    /// ⇒ 本 commit 让它**第一次**为真。
    ///
    /// ⭐ 语义（取自 `Dispatcher::dispatch_waterfall` 的文档，非猜）：
    /// 任一 handler 返回 `true` 即**短路**，剩余链不运行
    /// ⇒ **短路 = 拦截**（阻止工具真正执行），与该文档「用于守卫链」的表述一致。
    tool_hooks: Dispatcher<McpToolCall>,
}

/// 外部 MCP 服务器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub transport: McpTransport,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
    },
    Http {
        url: String,
    },
    Sse {
        url: String,
    },
}

/// ⭐⭐ **默认参数体积上限（64 KiB）**
/// `McpBridge` 是**外部 MCP 客户端可达**的 IPC 面，参数无界即无界分配。
pub const DEFAULT_TOOL_GUARD_MAX_ARGS_BYTES: usize = 64 * 1024;

/// ⭐⭐ **生产工具守卫**（2026-10-03）—— 让 `on_tool_call` 达 **L3（有生产注册）**。
///
/// 做两件与 schema 无关的**真实**策略：参数体积上限、空工具名（皆 fail-closed）。
///
/// ⭐ 为什么 `required` 校验**不在这里**：钩子签名 `Fn(&McpToolCall, &dyn Fn())`
/// 是 `'static` 的，**拿不到 `self.local_tools`** ⇒ 需要 schema 的校验只能 inline
/// 在 `call_local_tool`。⭐ **职责按可得性划分**，不为了「都在钩子里」而引入
/// `Arc<RwLock<..>>` 共享可变状态。
/// ⛔ 刻意**不**引 JSON Schema 库、**不**支持仓里不存在的 `oneOf`/`anyOf`
/// —— 凭空实现即是 AGENTS.md 说的「代码里的谎言」。
fn make_default_tool_guard(
    max_args_bytes: usize,
) -> impl Fn(&McpToolCall, &dyn Fn()) -> bool + Send + Sync + 'static {
    move |call: &McpToolCall, _next: &dyn Fn()| {
        if call.name.trim().is_empty() {
            log::warn!("[mcp] 拦截：空工具名");
            return true;
        }
        let len = serde_json::to_string(&call.arguments).map(|v| v.len()).unwrap_or(0);
        if len > max_args_bytes {
            log::warn!(
                "[mcp] 拦截：工具 '{}' 参数 {} 字节超过 {} 上限",
                call.name, len, max_args_bytes
            );
            return true;
        }
        false
    }
}

/// ⭐⭐ 返回 `args` 相对 `schema` 缺失的 `required` 字段（空 = 全部齐备）。
///
/// ⛔ **刻意只支持仓里真出现的最简形式** `{"required": ["a", "b"]}`
///（见本文件既有测试 `:188` 的 `kb_search`）。
/// ⛔ **不**支持 `oneOf`/`anyOf`/嵌套 schema —— 那些**仓里不存在**，
/// 凭空实现即是 AGENTS.md 说的「**代码里的谎言**」。
/// ⭐ 返回空 `Vec` 而非 `Result`：**无法解析的 schema 视为不拦截**
/// （fail-open）—— 因为「声明写错」不该让所有工具调用失败。
///    ⭐ 但空名/体积这类**与 schema 无关**的守卫仍是 fail-closed（见 make_default_tool_guard）。
fn missing_required(schema: &serde_json::Value, args: &serde_json::Value) -> Option<Vec<String>> {
    let required = schema.get("required")?.as_array()?;
    let obj = args.as_object()?;
    let missing: Vec<String> = required
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|k| !obj.contains_key(*k))
        .map(|k| k.to_owned())
        .collect();
    if missing.is_empty() { None } else { Some(missing) }
}

impl McpBridge {
    pub fn new() -> Self {
        let mut bridge = Self {
            local_tools: HashMap::new(),
            servers: HashMap::new(),
            tool_hooks: Dispatcher::new(),
        };
        // ⭐⭐⭐ 2026-10-03：**生产注册**默认工具守卫 ⇒ `on_tool_call` 达
        // **L3（有生产注册）**，不再是 L2「只有分发没有使用」。
        //
        // ⭐ 这是本日沉淀的三级判据（声明 / 分发 / 使用）的直接应用：
        // `ee2cc276`…`54a2fa64` 一路都在 L1/L2 级就宣称「已接线」，
        // ⛔ 而 L3 的判据是「**有生产代码真的注册它吗**」。
        //
        // ⭐ 守卫职责（与 schema 无关，故可作 `'static` 钩子）：
        // · 参数序列化体积上限（外部 MCP 客户端可达的 IPC 面 ⇒ 无界即无界分配）
        // · 空工具名拒绝
        bridge
            .tool_hooks
            .register(make_default_tool_guard(DEFAULT_TOOL_GUARD_MAX_ARGS_BYTES));
        bridge
    }

    /// ⭐⭐ 注册工具调用 waterfall 钩子（2026-10-03 接线，见 `tool_hooks` 字段注释）
    ///
    /// - 返回 `true` ⇒ **拦截**：本次调用不会进入真实工具执行。
    /// - 调用 `next()` ⇒ 委托给链上剩余钩子（around 中间件语义）。
    ///
    /// ⛔ 这不是「把已有能力换个地方放」——接线前 `nt_core_dispatch` 的
    /// 354 行 waterfall 机制**真实消费者为零**，本方法是它**第一个**生产入口。
    pub fn on_tool_call<F>(&mut self, handler: F) -> usize
    where
        F: Fn(&McpToolCall, &dyn Fn()) -> bool + Send + Sync + 'static,
    {
        self.tool_hooks.register(handler)
    }

    /// 当前已注册的工具钩子数（可观测 ⇒ 接线可证伪）
    pub fn tool_hook_count(&self) -> usize {
        self.tool_hooks.len()
    }

    /// 注册本地工具
    pub fn register_local_tool(&mut self, tool: McpTool) {
        self.local_tools.insert(tool.name.clone(), tool);
    }

    /// 列出本地工具
    pub fn list_local_tools(&self) -> Vec<&McpTool> {
        self.local_tools.values().collect()
    }

    /// 添加外部 MCP 服务器
    pub fn add_server(&mut self, server: McpServer) {
        self.servers.insert(server.id.clone(), server);
    }

    /// 列出外部服务器
    pub fn list_servers(&self) -> Vec<&McpServer> {
        self.servers.values().collect()
    }

    /// 调用本地工具 (供外部 MCP 客户端)
    ///
    /// ⭐ 2026-10-03：先过 `tool_hooks` waterfall 链（守卫/中间件），
    /// 任一钩子返回 `true` 即短路，**不进入真实工具执行**。
    pub fn call_local_tool(&self, call: &McpToolCall) -> McpToolResult {
        if self.tool_hooks.dispatch_waterfall(call) {
            return McpToolResult {
                call_id: call.id.clone(),
                content: format!("Tool '{}' blocked by hook", call.name),
                is_error: true,
            };
        }
        match self.local_tools.get(&call.name) {
            Some(tool) => {
                // ⭐⭐ 2026-10-03：**校验工具自己声明的 `required`（此前从不做）**
                //
                // ⛔ 改前此处绑定的是 **`_tool`（下划线 = 从未使用）**
                // ⇒ 工具声明 `"required": ["keyword"]` 却从不被校验
                // ⇒ 外部 MCP 客户端缺参数也能拿到「executed successfully」。
                // 缺陷证据：本文件测试 `现状_缺required参数仍报成功`。
                //
                // ⭐ **为什么放在这里而不是钩子**：`required` 校验需要读**该工具的
                // schema**，而钩子签名是 `Fn(&McpToolCall, &dyn Fn())`（`'static`，
                // 拿不到 `self.local_tools`）⇒ ⭐ **职责按可得性划分**：
                // · 钩子（无 self）= 与具体工具无关的策略：参数体积上限、空名
                // · inline（有 self）= 需要 schema 的校验：`required`
                // ⛔ 强行把 schema 塞进钩子只能靠 `Arc<RwLock<..>>` 克隆映射，
                //    那是为一次字段读取引入共享可变状态 —— 不划算。
                if let Some(missing) = missing_required(&tool.parameters, &call.arguments) {
                    log::warn!(
                        "[mcp] 拦截：工具 '{}' 缺必需参数 {:?}",
                        call.name, missing
                    );
                    return McpToolResult {
                        call_id: call.id.clone(),
                        content: format!(
                            "Tool '{}' missing required argument(s): {}",
                            call.name,
                            missing.join(", ")
                        ),
                        is_error: true,
                    };
                }
                // 实际调用逻辑需要根据工具类型实现
                // 这里返回一个示例结果
                McpToolResult {
                    call_id: call.id.clone(),
                    content: format!("Tool '{}' executed successfully", call.name),
                    is_error: false,
                }
            }
            None => McpToolResult {
                call_id: call.id.clone(),
                content: format!("Tool '{}' not found", call.name),
                is_error: true,
            },
        }
    }
}

impl Default for McpBridge {
    fn default() -> Self {
        let mut bridge = Self::new();

        // 预注册 NeoTrix 核心工具
        bridge.register_local_tool(McpTool {
            name: "kb_query".to_string(),
            description: "知识库查询 — 从知识库中检索信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "查询内容"
                    }
                },
                "required": ["query"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "kb_search".to_string(),
            description: "知识库搜索 — 在知识库中搜索相关信息".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "keyword": {
                        "type": "string",
                        "description": "搜索关键词"
                    }
                },
                "required": ["keyword"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "experience_query".to_string(),
            description: "经验查询 — 查询历史经验记录".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "查询条件"
                    }
                },
                "required": ["query"]
            }),
        });

        bridge.register_local_tool(McpTool {
            name: "agent_list".to_string(),
            description: "列出 agents — 显示所有可用的 agents".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        bridge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_local_tool() {
        let mut bridge = McpBridge::new();
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool);
        assert_eq!(bridge.local_tools.len(), 1);
        assert!(bridge.local_tools.contains_key("test_tool"));
    }

    #[test]
    fn test_list_tools() {
        let mut bridge = McpBridge::new();
        let tool1 = McpTool {
            name: "tool1".to_string(),
            description: "Tool 1".to_string(),
            parameters: serde_json::json!({}),
        };
        let tool2 = McpTool {
            name: "tool2".to_string(),
            description: "Tool 2".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool1);
        bridge.register_local_tool(tool2);

        let tools = bridge.list_local_tools();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_add_server() {
        let mut bridge = McpBridge::new();
        let server = McpServer {
            id: "server1".to_string(),
            name: "Test Server".to_string(),
            url: "http://localhost:8080".to_string(),
            transport: McpTransport::Http {
                url: "http://localhost:8080".to_string(),
            },
            available: true,
        };

        bridge.add_server(server);
        assert_eq!(bridge.servers.len(), 1);
        assert!(bridge.servers.contains_key("server1"));
    }

    #[test]
    fn test_call_local_tool() {
        let mut bridge = McpBridge::new();
        let tool = McpTool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: serde_json::json!({}),
        };

        bridge.register_local_tool(tool);

        let call = McpToolCall {
            id: "call1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = bridge.call_local_tool(&call);
        assert_eq!(result.call_id, "call1");
        assert!(!result.is_error);
        assert!(result.content.contains("test_tool"));
    }

    // ════════════════════════════════════════════════════════════════
    // ⭐⭐ waterfall 接线回归（2026-10-03）
    // ⭐ 这三条测试的价值不在「钩子能跑」，而在**证伪那句未兑现的 R-P79 声明**：
    //   接线前 `nt_core_dispatch` 的 354 行 waterfall 机制真实消费者为零。
    //   ⇒ 若将来有人把 `tool_hooks` 摘掉，这三条会立刻红。
    // ════════════════════════════════════════════════════════════════

    /// ⭐ 建一个**已注册**工具的 bridge —— 否则 `call_local_tool` 会走
    /// 「工具未找到」分支返回 `is_error=true`，我的「无钩子不改变行为」
    /// 就会因为**前提错误**而失败（⭐ 第一版就这样翻车了）。
    fn bridge_with_tool() -> McpBridge {
        let mut b = McpBridge::new();
        b.register_local_tool(McpTool {
            name: "test_tool".to_owned(),
            description: "A test tool".to_owned(),
            parameters: serde_json::json!({}),
        });
        b
    }

    fn sample_call() -> McpToolCall {
        McpToolCall {
            id: "c1".to_owned(),
            name: "test_tool".to_owned(),
            arguments: serde_json::json!({}),
        }
    }

    /// ⭐ 无钩子时行为**不变**（接线不能改变既有语义）
    /// ⭐ 生产守卫已在 `new()` 注册 ⇒ 默认就是 1 条（本测试改名为「默认守卫」）
    #[test]
    fn 默认已注册一条生产守卫() {
        let b = bridge_with_tool();
        assert_eq!(b.tool_hook_count(), 1, "默认守卫必须在 new() 里注册");
        let r = b.call_local_tool(&sample_call());
        assert!(!r.is_error, "默认守卫不应拦截正常调用");
    }

    /// ⭐ 返回 `true` ⇒ **拦截**，工具不执行（守卫语义）
    #[test]
    fn 钩子返回true则短路拦截() {
        let mut b = bridge_with_tool();
        assert_eq!(b.tool_hook_count(), 1, "只有默认守卫");
        b.on_tool_call(|_c, _next| true);
        assert_eq!(b.tool_hook_count(), 2);
        let r = b.call_local_tool(&sample_call());
        assert!(r.is_error, "被拦截的结果必须是错误");
        assert!(r.content.contains("blocked by hook"), "实际: {}", r.content);
    }

    /// ⭐ 守卫链：前面的放行、后面的拦截 ⇒ **任一**返回 true 即拦截
    /// （语义取自 `Dispatcher::dispatch_waterfall` 文档，非猜）
    #[test]
    fn 守卫链任一返回true即拦截() {
        let mut b = bridge_with_tool();
        b.on_tool_call(|_c, _next| false); // 放行
        b.on_tool_call(|_c, _next| true); // 拦截
        assert_eq!(b.tool_hook_count(), 3);
        let r = b.call_local_tool(&sample_call());
        assert!(r.is_error, "链上后段拦截应生效");
    }

    /// ⭐⭐ **缺陷回归**：工具声明了 `"required": ["keyword"]`（见本文件既有测试
    /// `:188` 的 `kb_search`），但 `call_local_tool` 曾把工具绑定为 `_tool`
    /// （下划线 = **从未使用**）⇒ **缺参数也返回「executed successfully」**。
    /// ⭐ 本测试**先以「仍报成功」钉住缺陷**、修好后**再翻转为断言被拦截**
    /// —— ⭐ 这就是「缺陷证据 → 修复判据」的完整闭环。
    #[test]
    fn 缺required参数必被拦截() {
        let mut b = McpBridge::new();
        b.register_local_tool(McpTool {
            name: "kb_search".to_owned(),
            description: "知识库搜索".to_owned(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "keyword": { "type": "string" } },
                "required": ["keyword"]
            }),
        });
        let call = McpToolCall {
            id: "c1".to_owned(),
            name: "kb_search".to_owned(),
            arguments: serde_json::json!({}), // ⛔ 缺 required 的 keyword
        };
        let r = b.call_local_tool(&call);
        // ⭐ 修复前此处断言 `!r.is_error`（缺陷真实存在）；修复后必须被拦截。
        assert!(r.is_error, "缺 required 参数必须被拦截");
        assert!(
            r.content.contains("missing required argument"),
            "应指出缺失字段，实际: {}",
            r.content
        );
    }

    /// ⭐ 齐备参数必须正常通过（**防过度拦截**：守卫不能把合法调用也挡掉）
    #[test]
    fn 齐备required参数正常通过() {
        let mut b = McpBridge::new();
        b.register_local_tool(McpTool {
            name: "kb_search".to_owned(),
            description: "知识库搜索".to_owned(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "keyword": { "type": "string" } },
                "required": ["keyword"]
            }),
        });
        let call = McpToolCall {
            id: "c2".to_owned(),
            name: "kb_search".to_owned(),
            arguments: serde_json::json!({ "keyword": "涌现" }),
        };
        let r = b.call_local_tool(&call);
        assert!(!r.is_error, "参数齐备不应被拦截，实际: {}", r.content);
    }
}
