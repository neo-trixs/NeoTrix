//! NeoTrix 统一错误类型
//!
//! 替代散布各处的 `Result<T, String>`，提供结构化错误信息

use std::fmt;
use std::path::PathBuf;

/// NeoTrix 统一错误枚举
#[non_exhaustive]
#[derive(Debug)]
pub enum NeoTrixError {
    /// 配置错误
    Config(String),
    /// IO 错误
    Io(std::io::Error),
    /// 序列化/反序列化错误
    Serde(String),
    /// 网络请求错误
    Network(String),
    /// MCP 工具错误
    Mcp(String),
    /// ReasoningBrain 错误
    Brain(String),
    /// 记忆/知识库错误
    Memory(String),
    /// 命令执行错误
    Command { cmd: String, exit_code: Option<i32>, stderr: String },
    /// 路径错误
    Path { path: PathBuf, detail: String },
    /// 未实现
    Unimplemented(String),
    /// 一般错误
    General { msg: String, backtrace: Option<String> },
}

impl fmt::Display for NeoTrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoTrixError::Config(msg) => write!(f, "配置错误: {}", msg),
            NeoTrixError::Io(err) => write!(f, "IO 错误: {}", err),
            NeoTrixError::Serde(msg) => write!(f, "序列化错误: {}", msg),
            NeoTrixError::Network(msg) => write!(f, "网络错误: {}", msg),
            NeoTrixError::Mcp(msg) => write!(f, "MCP 错误: {}", msg),
            NeoTrixError::Brain(msg) => write!(f, "Brain 错误: {}", msg),
            NeoTrixError::Memory(msg) => write!(f, "记忆错误: {}", msg),
            NeoTrixError::Command { cmd, exit_code, stderr } => {
                write!(f, "命令执行失败 [{}] exit={:?}: {}", cmd, exit_code, stderr)
            }
            NeoTrixError::Path { path, detail } => {
                write!(f, "路径错误 {:?}: {}", path, detail)
            }
            NeoTrixError::Unimplemented(msg) => write!(f, "未实现: {}", msg),
            NeoTrixError::General { msg, backtrace: None } => write!(f, "{}", msg),
            NeoTrixError::General { msg, backtrace: Some(bt) } => write!(f, "{} (backtrace: {})", msg, bt),
        }
    }
}

impl std::error::Error for NeoTrixError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NeoTrixError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NeoTrixError {
    fn from(err: std::io::Error) -> Self {
        NeoTrixError::Io(err)
    }
}

impl From<String> for NeoTrixError {
    fn from(msg: String) -> Self {
        NeoTrixError::General { msg, backtrace: None }
    }
}

impl From<&str> for NeoTrixError {
    fn from(msg: &str) -> Self {
        NeoTrixError::General { msg: msg.to_string(), backtrace: None }
    }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for NeoTrixError {
    fn from(err: anyhow::Error) -> Self {
        NeoTrixError::General { msg: err.to_string(), backtrace: None }
    }
}

impl NeoTrixError {
    /// Attach a captured backtrace to the error
    pub fn with_backtrace(self) -> Self {
        let bt = std::backtrace::Backtrace::capture();
        let bt_str = format!("{}", bt);
        match self {
            NeoTrixError::General { msg, .. } => {
                NeoTrixError::General { msg, backtrace: Some(bt_str) }
            }
            other => other,
        }
    }
}

pub type NeoTrixResult<T> = Result<T, NeoTrixError>;

/// 从 `Result<(), String>` 转换为 `NeoTrixResult<()>`
pub fn from_string_result<T>(r: Result<T, String>) -> NeoTrixResult<T> {
    r.map_err(|e| NeoTrixError::General { msg: e, backtrace: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_error_config_display() {
        let e = NeoTrixError::Config("missing key".into());
        assert_eq!(format!("{}", e), "配置错误: missing key");
    }

    #[test]
    fn test_error_io_display() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = NeoTrixError::Io(io);
        assert!(format!("{}", e).contains("file missing"));
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
    fn test_error_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "io error");
        let e: NeoTrixError = io.into();
        match e {
            NeoTrixError::Io(_) => {},
            _ => panic!("expected Io variant"),
        }
    }

    #[test]
    fn test_error_source_io() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "src");
        let e = NeoTrixError::Io(io);
        assert!(e.source().is_some());
    }

    #[test]
    fn test_error_source_non_io() {
        let e = NeoTrixError::Config("test".into());
        assert!(e.source().is_none());
    }

    #[test]
    fn test_error_with_backtrace_on_general() {
        let e = NeoTrixError::General { msg: "test".into(), backtrace: None };
        let e2 = e.with_backtrace();
        match e2 {
            NeoTrixError::General { backtrace, .. } => assert!(backtrace.is_some()),
            _ => panic!("expected General"),
        }
    }

    #[test]
    fn test_error_with_backtrace_other_variant() {
        let e = NeoTrixError::Config("test".into());
        let e2 = e.with_backtrace();
        match e2 {
            NeoTrixError::Config(_) => {},
            _ => panic!("expected Config"),
        }
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
}
