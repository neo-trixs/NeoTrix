//! 类型化 I/O 契约 — 基于 NVIDIA NOOA 模式
//!
//! 工具调用使用类型化契约，而非原始字符串。

#![allow(dead_code)]

use std::collections::HashMap;

/// 参数类型
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array { item_type: Box<ParamType> },
    Object { fields: HashMap<String, ParamType> },
}

/// 参数定义
pub struct ParamDef {
    pub name: String,
    pub param_type: ParamType,
    pub required: bool,
    pub description: String,
    pub default: Option<String>,
}

/// 工具 I/O 契约
pub struct ToolIoContract {
    pub tool_name: String,
    pub description: String,
    pub input_params: Vec<ParamDef>,
    pub output_fields: Vec<ParamDef>,
    pub estimated_tokens: u32,
    pub estimated_latency_ms: u32,
}

/// 契约注册表
pub struct ContractRegistry {
    contracts: HashMap<String, ToolIoContract>,
}

impl ContractRegistry {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
        }
    }

    /// 注册工具契约
    pub fn register(&mut self, contract: ToolIoContract) {
        self.contracts.insert(contract.tool_name.clone(), contract);
    }

    /// 查找工具契约
    pub fn find(&self, tool_name: &str) -> Option<&ToolIoContract> {
        self.contracts.get(tool_name)
    }

    /// 验证输入是否符合契约
    pub fn validate_input(
        &self,
        tool_name: &str,
        input: &HashMap<String, String>,
    ) -> Result<(), String> {
        let contract = self
            .contracts
            .get(tool_name)
            .ok_or_else(|| format!("Tool not found: {}", tool_name))?;

        for param in &contract.input_params {
            if param.required && !input.contains_key(&param.name) {
                return Err(format!("Missing required param: {}", param.name));
            }
        }
        Ok(())
    }

    /// 工具数量
    pub fn tool_count(&self) -> usize {
        self.contracts.len()
    }

    /// 列出所有工具
    pub fn list_tools(&self) -> Vec<&str> {
        self.contracts.keys().map(|s| s.as_str()).collect()
    }
}
