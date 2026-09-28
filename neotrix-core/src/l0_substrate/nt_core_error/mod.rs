//! NeoTrix 统一错误类型 (L0)
//!
//! 定义在 core 层以防 L7/L5/L4 模块因依赖此类型而反向引用 neotrix 层。
//! 子模块: parse (编译器错误解析), recovery (错误恢复策略)。

pub mod parse;
pub mod recovery;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// NeoTrix 统一错误枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NeoTrixError {
    Config(String),
    Io(String),
    Serde(String),
    Network(String),
    Mcp(String),
    Brain(String),
    Memory(String),
    Command {
        cmd: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    Path {
        path: PathBuf,
        detail: String,
    },
    Unimplemented(String),
    Wasm(String),
    Crypto(String),
    Keyring(String),
    Shield(String),
    /// pi-agent steer (缺陷②): 慢进程重规划建议 — 保留 brain 进度, 上层可重定向任务。
    /// 与 Brain(abort) 的区别: Steer 不丢弃已产出成果, 仅请求换路线。
    Steer(String),
    /// 资源/实体未找到
    NotFound(String),
    /// 输入参数非法
    InvalidInput(String),
    /// 系统状态非法 (前置条件未满足)
    InvalidState(String),
    /// 操作尚未实现
    NotImplemented(String),
    /// 通用操作失败
    OperationFailed(String),
    /// 安全策略违规 (NT-SHIELD)
    SafetyViolation(String),
}

impl fmt::Display for NeoTrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoTrixError::Config(msg) => write!(f, "配置错误: {}", msg),
            NeoTrixError::Io(msg) => write!(f, "IO 错误: {}", msg),
            NeoTrixError::Serde(msg) => write!(f, "序列化错误: {}", msg),
            NeoTrixError::Network(msg) => write!(f, "网络错误: {}", msg),
            NeoTrixError::Mcp(msg) => write!(f, "MCP 错误: {}", msg),
            NeoTrixError::Brain(msg) => write!(f, "Brain 错误: {}", msg),
            NeoTrixError::Memory(msg) => write!(f, "记忆错误: {}", msg),
            NeoTrixError::Command {
                cmd,
                exit_code,
                stderr,
            } => {
                write!(f, "命令执行失败 [{}] exit={:?}: {}", cmd, exit_code, stderr)
            }
            NeoTrixError::Path { path, detail } => {
                write!(f, "路径错误 {:?}: {}", path, detail)
            }
            NeoTrixError::Unimplemented(msg) => write!(f, "未实现: {}", msg),
            NeoTrixError::Wasm(msg) => write!(f, "WASM 错误: {}", msg),
            NeoTrixError::Crypto(msg) => write!(f, "加密错误: {}", msg),
            NeoTrixError::Keyring(msg) => write!(f, "密钥环错误: {}", msg),
            NeoTrixError::Shield(msg) => write!(f, "护盾拦截: {}", msg),
            NeoTrixError::Steer(msg) => write!(f, "steer 重定向建议: {}", msg),
            NeoTrixError::NotFound(msg) => write!(f, "未找到: {}", msg),
            NeoTrixError::InvalidInput(msg) => write!(f, "非法输入: {}", msg),
            NeoTrixError::InvalidState(msg) => write!(f, "非法状态: {}", msg),
            NeoTrixError::NotImplemented(msg) => write!(f, "未实现: {}", msg),
            NeoTrixError::OperationFailed(msg) => write!(f, "操作失败: {}", msg),
            NeoTrixError::SafetyViolation(msg) => write!(f, "安全违规: {}", msg),
        }
    }
}

impl std::error::Error for NeoTrixError {}

impl From<std::io::Error> for NeoTrixError {
    fn from(err: std::io::Error) -> Self {
        NeoTrixError::Io(err.to_string())
    }
}

impl From<String> for NeoTrixError {
    fn from(msg: String) -> Self {
        NeoTrixError::Brain(msg)
    }
}

impl From<&str> for NeoTrixError {
    fn from(msg: &str) -> Self {
        NeoTrixError::Brain(msg.to_string())
    }
}

// ════════════════════════════════════════════════════════════════
// From 实现 — 统一错误层级
// ════════════════════════════════════════════════════════════════

// L1 Action 层错误 — moved to crate::l1_action::error_conversions

// L2 Perception 层错误 — moved to l2_perception::error_conversions

// L3 Embodiment 层错误 — moved to l3_embodiment::error_conversions

// L5 Cognition 层错误 — moved to l5_cognition::error_conversions
// (注: nt_core_skill_registry / nt_core_multi_agent 已迁至独立 crates)

// L6 Meta 层错误 — moved to l6_meta::error_conversions

// Core 层错误
impl From<crate::l0_substrate::nt_core_platform::PlatformError> for NeoTrixError {
    fn from(e: crate::l0_substrate::nt_core_platform::PlatformError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

impl From<crate::l0_substrate::nt_core_platform::AgentError> for NeoTrixError {
    fn from(e: crate::l0_substrate::nt_core_platform::AgentError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

// L5/L6/neotrix 层其余错误 — moved to 各层 error_conversions
// (BoundaryError → l5_cognition; ErrorRecoveryError/CapabilityError → l6_meta;
//  CapabilityError/FileAbilityError/SuperResolutionError/RegistryError → neotrix)

// Core 层补充错误 (L0 内部, 保留)

impl From<crate::l0_substrate::nt_core_hot_data::HotDataError> for NeoTrixError {
    fn from(e: crate::l0_substrate::nt_core_hot_data::HotDataError) -> Self {
        NeoTrixError::OperationFailed(e.to_string())
    }
}

// ════════════════════════════════════════════════════════════════
// From<NeoTrixError> for NtError — bridge to unified error type
// ════════════════════════════════════════════════════════════════

impl From<NeoTrixError> for neotrix_types::NtError {
    fn from(err: NeoTrixError) -> Self {
        match err {
            NeoTrixError::Config(msg) => neotrix_types::NtError::Config(msg),
            NeoTrixError::Io(msg) => neotrix_types::NtError::Io(msg),
            NeoTrixError::Serde(msg) => neotrix_types::NtError::Serde(msg),
            NeoTrixError::Network(msg) => neotrix_types::NtError::Network(msg),
            NeoTrixError::Mcp(msg) => neotrix_types::NtError::Mcp(msg),
            NeoTrixError::Brain(msg) => neotrix_types::NtError::Brain(msg),
            NeoTrixError::Memory(msg) => neotrix_types::NtError::Memory(msg),
            NeoTrixError::Command { cmd, exit_code, stderr } => neotrix_types::NtError::Command { cmd, exit_code, stderr },
            NeoTrixError::Path { path, detail } => neotrix_types::NtError::Path { path, detail },
            NeoTrixError::Unimplemented(msg) => neotrix_types::NtError::Unimplemented(msg),
            NeoTrixError::Wasm(msg) => neotrix_types::NtError::Wasm(msg),
            NeoTrixError::Crypto(msg) => neotrix_types::NtError::Crypto(msg),
            NeoTrixError::Keyring(msg) => neotrix_types::NtError::Keyring(msg),
            NeoTrixError::Shield(msg) => neotrix_types::NtError::Shield(msg),
            NeoTrixError::Steer(msg) => neotrix_types::NtError::Steer(msg),
            NeoTrixError::NotFound(msg) => neotrix_types::NtError::NotFound(msg),
            NeoTrixError::InvalidInput(msg) => neotrix_types::NtError::InvalidInput(msg),
            NeoTrixError::InvalidState(msg) => neotrix_types::NtError::InvalidState(msg),
            NeoTrixError::NotImplemented(msg) => neotrix_types::NtError::NotImplemented(msg),
            NeoTrixError::OperationFailed(msg) => neotrix_types::NtError::OperationFailed(msg),
            NeoTrixError::SafetyViolation(msg) => neotrix_types::NtError::SafetyViolation(msg),
        }
    }
}

pub type NeoTrixResult<T> = Result<T, NeoTrixError>;

pub fn from_string_result<T>(r: Result<T, String>) -> NeoTrixResult<T> {
    r.map_err(NeoTrixError::Brain)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_config_display() {
        let e = NeoTrixError::Config("missing key".into());
        assert_eq!(format!("{}", e), "配置错误: missing key");
    }

    #[test]
    fn test_error_io_display() {
        let e = NeoTrixError::Io("file not found".into());
        assert!(format!("{}", e).contains("file not found"));
    }

    #[test]
    fn test_error_network_display() {
        let e = NeoTrixError::Network("connection refused".into());
        assert_eq!(format!("{}", e), "网络错误: connection refused");
    }

    #[test]
    fn test_error_command_display() {
        let e = NeoTrixError::Command {
            cmd: "cargo build".into(),
            exit_code: Some(1),
            stderr: "error[E0308]".into(),
        };
        let msg = format!("{}", e);
        assert!(msg.contains("cargo build"));
        assert!(msg.contains("error[E0308]"));
    }

    #[test]
    fn test_error_unimplemented_display() {
        let e = NeoTrixError::Unimplemented("feature x".into());
        assert_eq!(format!("{}", e), "未实现: feature x");
    }

    #[test]
    fn test_error_from_string() {
        let e: NeoTrixError = "something went wrong".into();
        assert!(format!("{}", e).contains("something went wrong"));
    }

    #[test]
    fn test_error_from_io() -> Result<(), String> {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let e: NeoTrixError = io.into();
        match e {
            NeoTrixError::Io(_) => {}
            _ => return Err("expected Io variant".to_string()),
        }
        Ok(())
    }

    #[test]
    fn test_from_string_result_ok() {
        let r: Result<i32, String> = Ok(42);
        let result = from_string_result(r);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_from_string_result_err() {
        let r: Result<i32, String> = Err("failed".into());
        let result = from_string_result(r);
        assert!(result.is_err());
    }

    #[test]
    fn test_result_type_alias() {
        let r: NeoTrixResult<i32> = Ok(1);
        assert!(r.is_ok());
    }

    #[test]
    fn test_steer_variant_display() {
        // pi-agent steer (缺陷②): Steer 变体是可显示的错误, 携带重定向建议。
        let e = NeoTrixError::Steer("保留进度, 请重定向任务目标".into());
        let s = e.to_string();
        assert!(
            s.contains("steer 重定向"),
            "display 应含 steer 标记, got: {}",
            s
        );
        assert!(
            s.contains("重定向任务目标"),
            "display 应含建议内容, got: {}",
            s
        );
    }
}
