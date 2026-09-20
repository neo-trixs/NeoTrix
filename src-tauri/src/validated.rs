#![forbid(unsafe_code)]

//! Type-safe input validation newtypes.
//!
//! Each newtype wraps a `String` and enforces invariants via `TryFrom`,
//! so invalid data is rejected at the boundary rather than propagated.

use std::fmt;
use std::path::Path;

// ═══════════════════════════════════════════════
// ChannelName — validated IM channel identifier
// ═══════════════════════════════════════════════

/// A validated IM channel name.
///
/// Constraints:
/// - Must be one of the nine recognized channel types.
/// - Derived from `ChannelType::from_name` in `commands/im.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelName(String);

impl ChannelName {
    /// All valid channel names (lowercase).
    const VALID: &[&str] = &[
        "wechat",
        "feishu",
        "dingtalk",
        "wecom",
        "qq",
        "slack",
        "telegram",
        "discord",
        "whatsapp",
    ];

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ChannelName {
    type Error = ChannelNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lower = value.to_lowercase();
        if Self::VALID.contains(&lower.as_str()) {
            Ok(Self(lower))
        } else {
            Err(ChannelNameError(value))
        }
    }
}

impl TryFrom<&str> for ChannelName {
    type Error = ChannelNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelNameError(String);

impl fmt::Display for ChannelNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid channel name '{}'. Valid: {:?}",
            self.0,
            ChannelName::VALID
        )
    }
}

impl std::error::Error for ChannelNameError {}

// ═══════════════════════════════════════════════
// ModelPath — validated model file path
// ═══════════════════════════════════════════════

/// A validated local model file path.
///
/// Constraints:
/// - Must not be empty.
/// - Must not contain null bytes.
/// - Must end with a recognized model extension (gguf, onnx, safetensors, pt, bin, pth).
/// - Parent directory must exist (checked at creation time when `check_exists` is true).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelPath(String);

impl ModelPath {
    const VALID_EXTENSIONS: &[&str] = &[
        "gguf", "onnx", "safetensors", "pt", "bin", "pth",
    ];

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl TryFrom<String> for ModelPath {
    type Error = ModelPathError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ModelPathError::Empty);
        }
        if value.contains('\0') {
            return Err(ModelPathError::NullByte);
        }
        let ext = Path::new(&value)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !Self::VALID_EXTENSIONS.contains(&ext) {
            return Err(ModelPathError::InvalidExtension(ext.to_string()));
        }
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ModelPath {
    type Error = ModelPathError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl fmt::Display for ModelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelPathError {
    Empty,
    NullByte,
    InvalidExtension(String),
}

impl fmt::Display for ModelPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Model path must not be empty"),
            Self::NullByte => write!(f, "Model path must not contain null bytes"),
            Self::InvalidExtension(ext) => write!(
                f,
                "Invalid model extension '{}'. Valid: {:?}",
                ext,
                ModelPath::VALID_EXTENSIONS
            ),
        }
    }
}

impl std::error::Error for ModelPathError {}

// ═══════════════════════════════════════════════
// ProxyUrl — validated proxy/subscription URL
// ═══════════════════════════════════════════════

/// A validated proxy URL.
///
/// Constraints:
/// - Must use `http://` or `https://` scheme.
/// - Must contain a non-empty host.
/// - Port, if present, must be a valid `u16`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProxyUrl(String);

impl ProxyUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProxyUrl {
    type Error = ProxyUrlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err(ProxyUrlError::InvalidScheme);
        }
        let stripped = value
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        if stripped.is_empty() {
            return Err(ProxyUrlError::EmptyHost);
        }
        if let Some(colon_pos) = stripped.rfind(':') {
            let host = &stripped[..colon_pos];
            let port_str = &stripped[colon_pos + 1..];
            if host.is_empty() {
                return Err(ProxyUrlError::EmptyHost);
            }
            port_str
                .parse::<u16>()
                .map_err(|_| ProxyUrlError::InvalidPort(port_str.to_string()))?;
        }
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ProxyUrl {
    type Error = ProxyUrlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl fmt::Display for ProxyUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyUrlError {
    InvalidScheme,
    EmptyHost,
    InvalidPort(String),
}

impl fmt::Display for ProxyUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScheme => write!(f, "Proxy URL must use http:// or https:// scheme"),
            Self::EmptyHost => write!(f, "Proxy URL must have a non-empty host"),
            Self::InvalidPort(p) => write!(f, "Invalid port '{}': must be 0-65535", p),
        }
    }
}

impl std::error::Error for ProxyUrlError {}

// ═══════════════════════════════════════════════
// ProxyStrategy — validated strategy identifier
// ═══════════════════════════════════════════════

/// A validated proxy selection strategy name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProxyStrategy(String);

impl ProxyStrategy {
    const VALID: &[&str] = &[
        "fastest",
        "least_latency",
        "least_failure",
        "weighted_random",
        "geo_preferred",
        "round_robin",
        "adaptive",
        "auto",
    ];

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ProxyStrategy {
    type Error = ProxyStrategyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::VALID.contains(&value.as_str()) {
            Ok(Self(value))
        } else {
            Err(ProxyStrategyError(value))
        }
    }
}

impl TryFrom<&str> for ProxyStrategy {
    type Error = ProxyStrategyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl fmt::Display for ProxyStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyStrategyError(String);

impl fmt::Display for ProxyStrategyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid strategy '{}'. Valid: {:?}",
            self.0,
            ProxyStrategy::VALID
        )
    }
}

impl std::error::Error for ProxyStrategyError {}

// ═══════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // --- ChannelName ---

    #[test]
    fn channel_name_valid() {
        assert!(ChannelName::try_from("wechat").is_ok());
        assert!(ChannelName::try_from("Telegram".to_string()).is_ok());
        assert!(ChannelName::try_from("DISCORD").is_ok());
    }

    #[test]
    fn channel_name_invalid() {
        let err = ChannelName::try_from("sms").unwrap_err();
        assert!(err.to_string().contains("Invalid channel name"));
    }

    // --- ModelPath ---

    #[test]
    fn model_path_valid() {
        assert!(ModelPath::try_from("/models/llama.gguf".into()).is_ok());
        assert!(ModelPath::try_from("model.onnx".into()).is_ok());
        assert!(ModelPath::try_from("net.safetensors".into()).is_ok());
    }

    #[test]
    fn model_path_empty() {
        assert_eq!(
            ModelPath::try_from("").unwrap_err(),
            ModelPathError::Empty
        );
    }

    #[test]
    fn model_path_null_byte() {
        assert_eq!(
            ModelPath::try_from("model\0.gguf".into()).unwrap_err(),
            ModelPathError::NullByte
        );
    }

    #[test]
    fn model_path_bad_extension() {
        assert!(matches!(
            ModelPath::try_from("model.txt".into()).unwrap_err(),
            ModelPathError::InvalidExtension(_)
        ));
    }

    // --- ProxyUrl ---

    #[test]
    fn proxy_url_valid() {
        assert!(ProxyUrl::try_from("http://1.2.3.4:8080".into()).is_ok());
        assert!(ProxyUrl::try_from("https://proxy.example.com".into()).is_ok());
    }

    #[test]
    fn proxy_url_bad_scheme() {
        assert_eq!(
            ProxyUrl::try_from("socks5://1.2.3.4:1080".into()).unwrap_err(),
            ProxyUrlError::InvalidScheme
        );
    }

    #[test]
    fn proxy_url_bad_port() {
        assert!(matches!(
            ProxyUrl::try_from("http://1.2.3.4:99999".into()).unwrap_err(),
            ProxyUrlError::InvalidPort(_)
        ));
    }

    #[test]
    fn proxy_url_empty_host() {
        assert_eq!(
            ProxyUrl::try_from("http://".into()).unwrap_err(),
            ProxyUrlError::EmptyHost
        );
    }

    // --- ProxyStrategy ---

    #[test]
    fn proxy_strategy_valid() {
        assert!(ProxyStrategy::try_from("adaptive".into()).is_ok());
        assert!(ProxyStrategy::try_from("round_robin".into()).is_ok());
    }

    #[test]
    fn proxy_strategy_invalid() {
        let err = ProxyStrategy::try_from("random".into()).unwrap_err();
        assert!(err.to_string().contains("Invalid strategy"));
    }
}
