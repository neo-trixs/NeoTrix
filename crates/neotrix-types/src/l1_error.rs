use std::fmt;
use std::path::PathBuf;

/// L1 层本地错误类型 — 跨层共享，避免 L3 直接依赖 L1 模块
#[derive(Debug, Clone)]
pub enum L1Error {
    Config(String),
    Io(String),
    Serde(String),
    Network(String),
    Command { cmd: String, exit_code: Option<i32>, stderr: String },
    Path { path: PathBuf, detail: String },
    Wasm(String),
    Crypto(String),
    Keyring(String),
    Brain(String),
}

impl fmt::Display for L1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            L1Error::Config(msg) => write!(f, "Config error: {}", msg),
            L1Error::Io(msg) => write!(f, "IO error: {}", msg),
            L1Error::Serde(msg) => write!(f, "Serde error: {}", msg),
            L1Error::Network(msg) => write!(f, "Network error: {}", msg),
            L1Error::Command { cmd, exit_code, stderr } => {
                write!(f, "Command '{}' failed (exit={:?}): {}", cmd, exit_code, stderr)
            }
            L1Error::Path { path, detail } => write!(f, "Path error at {}: {}", path.display(), detail),
            L1Error::Wasm(msg) => write!(f, "WASM error: {}", msg),
            L1Error::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            L1Error::Keyring(msg) => write!(f, "Keyring error: {}", msg),
            L1Error::Brain(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for L1Error {}

impl From<std::io::Error> for L1Error {
    fn from(err: std::io::Error) -> Self { L1Error::Io(err.to_string()) }
}

impl From<String> for L1Error {
    fn from(msg: String) -> Self { L1Error::Brain(msg) }
}

impl From<&str> for L1Error {
    fn from(msg: &str) -> Self { L1Error::Brain(msg.to_string()) }
}

pub type L1Result<T> = Result<T, L1Error>;
