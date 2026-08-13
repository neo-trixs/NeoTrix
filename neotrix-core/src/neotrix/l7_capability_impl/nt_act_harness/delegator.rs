//! nt_act::harness::delegator — 工具委派 + 多 agent 协调
//!
//! 节点: nt_act::harness::delegator (L1)
//! Provides: tool_delegation, capability_routing
//!
//! 委派器按 capability 标签路由到已注册工具/子 agent, 防止指令错误路由
//! (对齐 opencode MCP 工具注册与 valuecell 多 agent 分工)。

#![forbid(unsafe_code)]

use crate::core::nt_core_error::NeoTrixError;
use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub capability: String,
    pub invocations: u64,
}

/// 工具委派器
#[derive(Debug, Clone, Default)]
pub struct ToolDelegator {
    // Vec 保持注册顺序 (确定性路由, HashMap 迭代无序)
    tools: Vec<ToolSpec>,
}

impl ToolDelegator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_tool(&mut self, name: &str, capability: &str) -> Result<(), NeoTrixError> {
        if name.trim().is_empty() {
            return Err(NeoTrixError::InvalidInput("工具名不能为空".into()));
        }
        self.tools.push(ToolSpec {
            name: name.into(),
            capability: capability.into(),
            invocations: 0,
        });
        Ok(())
    }

    /// 委派: 按 capability 精确路由 (先注册者优先), 找不到对应工具报错
    pub fn delegate(&mut self, capability: &str) -> Result<&str, NeoTrixError> {
        let tool = self
            .tools
            .iter_mut()
            .find(|t| t.capability == capability)
            .ok_or_else(|| {
                NeoTrixError::NotFound(format!("capability {} 无对应工具", capability))
            })?;
        tool.invocations += 1;
        Ok(&tool.name)
    }

    pub fn invocations(&self, name: &str) -> u64 {
        self.tools
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.invocations)
            .unwrap_or(0)
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }
}

impl CapabilityNode for ToolDelegator {
    fn node_id(&self) -> &str {
        "nt_act::harness::delegator"
    }
    fn provides(&self) -> Vec<String> {
        vec!["tool_delegation".into(), "capability_routing".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["agent_harness".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Indigo]
    }
    fn constellation_level(&self) -> u8 {
        0
    }
    fn promote_constellation(&mut self) -> bool {
        false
    }
}

impl SelfTest for ToolDelegator {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut d = ToolDelegator::new();
        d.register_tool("kb_write", "knowledge_editing")
            .map_err(|e| vec![e.to_string()])?;
        let routed = d
            .delegate("knowledge_editing")
            .map_err(|e| vec![e.to_string()])?;
        assert_eq!(routed, "kb_write");
        assert_eq!(d.invocations("kb_write"), 1);
        assert!(d.delegate("missing_cap").is_err());
        Ok(())
    }

    fn name(&self) -> &str {
        "nt_act_harness_delegator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_by_capability() {
        let mut d = ToolDelegator::new();
        d.register_tool("fetch", "web_data_acquisition").unwrap();
        d.register_tool("edit", "code_editing").unwrap();
        assert_eq!(d.delegate("web_data_acquisition").unwrap(), "fetch");
        assert_eq!(d.delegate("code_editing").unwrap(), "edit");
        assert_eq!(d.invocations("fetch"), 1);
        assert_eq!(d.tool_count(), 2);
    }

    #[test]
    fn test_unknown_capability_rejected() {
        let mut d = ToolDelegator::new();
        d.register_tool("fetch", "web_data_acquisition").unwrap();
        assert!(d.delegate("nope").is_err());
    }

    #[test]
    fn test_empty_name_rejected() {
        let mut d = ToolDelegator::new();
        assert!(d.register_tool("", "x").is_err());
    }

    #[test]
    fn test_reroute_after_register() {
        let mut d = ToolDelegator::new();
        d.register_tool("a", "cap").unwrap();
        d.register_tool("b", "cap").unwrap();
        // 两个工具同 capability, 委派选先注册的
        assert_eq!(d.delegate("cap").unwrap(), "a");
    }
}
