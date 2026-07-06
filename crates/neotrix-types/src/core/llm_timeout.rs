use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct LlmTimeoutConfig {
    pub wall: Option<Duration>,
    pub idle: Option<Duration>,
}

impl Default for LlmTimeoutConfig {
    fn default() -> Self {
        Self {
            wall: Some(Duration::from_secs(3600)),
            idle: Some(Duration::from_secs(120)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_timeout_config_default() {
        let cfg = LlmTimeoutConfig::default();
        assert_eq!(cfg.wall, Some(Duration::from_secs(3600)));
        assert_eq!(cfg.idle, Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_llm_timeout_config_custom() {
        let cfg = LlmTimeoutConfig {
            wall: Some(Duration::from_secs(7200)),
            idle: None,
        };
        assert_eq!(cfg.wall, Some(Duration::from_secs(7200)));
        assert_eq!(cfg.idle, None);
    }
}
