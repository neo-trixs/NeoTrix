pub struct PluginSandbox {
    allowed_hosts: Vec<String>,
    max_requests_per_minute: u32,
    // (timeout_ms 写-only 已删除; new(size) 式容量语义不变, 本 struct new() 无参)
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginSandbox {
    pub fn new() -> Self {
        Self {
            allowed_hosts: vec![
                "*.kugou.com".into(),
                "*.163.com".into(),
                "*.migu.cn".into(),
                "*.qq.com".into(),
                "*.kuwo.cn".into(),
                "*.spotify.com".into(),
            ],
            max_requests_per_minute: 60,
        }
    }
    pub fn _is_host_allowed(&self, url: &str) -> bool {
        self.allowed_hosts
            .iter()
            .any(|h| url.contains(&h.replace("*.", "")))
    }
    pub fn check_rate_limit(&self, requests_this_minute: u32) -> bool {
        requests_this_minute < self.max_requests_per_minute
    }
}
