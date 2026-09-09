/// API Key 安全存储 (从环境变量读取，不硬编码)
pub struct SecurityBridge;

impl SecurityBridge {
    /// 从环境变量获取 API Key
    pub fn get_api_key(source: &str) -> Option<String> {
        let env_key = format!(
            "NEOTRIX_MEDIA_{}_API_KEY",
            source.to_uppercase().replace('-', "_")
        );
        std::env::var(&env_key).ok().filter(|v| !v.is_empty())
    }

    /// 检查源是否有配置的 API Key
    pub fn has_api_key(source: &str) -> bool {
        Self::get_api_key(source).is_some()
    }

    /// 获取所有已配置的源
    pub fn configured_sources() -> Vec<String> {
        let sources = vec![
            "kugou", "netease", "migu", "qqmusic", "kuwo",
            "spotify", "deezer", "soundcloud", "pixabay", "pexels",
        ];
        sources.into_iter()
            .filter(|s| Self::has_api_key(s))
            .map(|s| s.to_string())
            .collect()
    }
}
