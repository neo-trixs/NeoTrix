//! UnifiedEngine — 统一查询入口
//!
//! 整合所有数据源（媒体 29 + 情报 3+ + OSINT 20+ + LLM 30+ + 爬虫 + 本地），
//! 提供统一查询接口。
//!
//! 调用方无需知道具体数据源在哪里，只需声明需要什么域的数据。

use super::media_bridge;
use super::intel_bridge;
use super::osint_bridge;
use super::llm_bridge;
use super::unified::*;
use std::sync::Arc;

/// 统一数据源引擎
pub struct UnifiedEngine {
    registry: DataSourceRegistry,
}

impl UnifiedEngine {
    /// 创建空引擎
    pub fn new() -> Self {
        Self {
            registry: DataSourceRegistry::new(),
        }
    }

    /// 注册所有媒体源 (29 个)
    pub fn with_media_sources(mut self) -> Self {
        for source in media_bridge::bridge_all_media_sources() {
            self.registry.register_media(source);
        }
        self
    }

    /// 注册所有情报源 (3+ 个)
    pub fn with_intel_sources(mut self) -> Self {
        for source in intel_bridge::create_intel_bridges() {
            self.registry.register_intel(source);
        }
        self
    }

    /// 注册所有 OSINT 源 (20+ 个)
    pub fn with_osint_sources(mut self) -> Self {
        for source in osint_bridge::create_osint_bridges() {
            self.registry.register_osint(source);
        }
        self
    }

    /// 注册所有 LLM 提供者 (30+ 个)
    pub fn with_llm_providers(mut self) -> Self {
        for source in llm_bridge::create_llm_bridges() {
            self.registry.register_llm(source);
        }
        self
    }

    /// 注册单个媒体源
    pub fn register_media(&mut self, source: Arc<dyn MediaSource>) {
        self.registry.register_media(source);
    }

    /// 注册单个情报源
    pub fn register_intel(&mut self, source: Arc<dyn IntelSource>) {
        self.registry.register_intel(source);
    }

    /// 注册单个 OSINT 源
    pub fn register_osint(&mut self, source: Arc<dyn OsintSource>) {
        self.registry.register_osint(source);
    }

    /// 注册单个 LLM 提供者
    pub fn register_llm(&mut self, source: Arc<dyn LlmProvider>) {
        self.registry.register_llm(source);
    }

    /// 注册单个爬虫源
    pub fn register_crawl(&mut self, source: Arc<dyn CrawlSource>) {
        self.registry.register_crawl(source);
    }

    /// 注册单个本地源
    pub fn register_local(&mut self, source: Arc<dyn LocalSource>) {
        self.registry.register_local(source);
    }

    // ── 媒体查询 ─────────────────────────────────────────────

    /// 搜索媒体源
    pub async fn search_media(
        &self,
        query: &str,
        page: u32,
    ) -> Result<Vec<MediaSearchResult>, String> {
        let sources = self.registry.media_sources(&SourceDomain::Media);
        let mut results = Vec::new();

        for source in sources {
            match source.search(query, page).await {
                Ok(result) => {
                    if result.total > 0 {
                        results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("Media source {} failed: {}", source.id(), e);
                }
            }
        }

        Ok(results)
    }

    /// 搜索学术源
    pub async fn search_academic(
        &self,
        query: &str,
        page: u32,
    ) -> Result<Vec<MediaSearchResult>, String> {
        let sources = self.registry.media_sources(&SourceDomain::Academic);
        let mut results = Vec::new();

        for source in sources {
            match source.search(query, page).await {
                Ok(result) => {
                    if result.total > 0 {
                        results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("Academic source {} failed: {}", source.id(), e);
                }
            }
        }

        Ok(results)
    }

    // ── 情报查询 ─────────────────────────────────────────────

    /// 搜索情报源
    pub async fn search_intel(&self, query: &str) -> Result<Vec<IntelResult>, String> {
        let sources = self.registry.intel_sources(&SourceDomain::Intel);
        let mut results = Vec::new();

        for source in sources {
            match source.fetch(query).await {
                Ok(result) => {
                    if result.total > 0 {
                        results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("Intel source {} failed: {}", source.id(), e);
                }
            }
        }

        Ok(results)
    }

    // ── OSINT 查询 ───────────────────────────────────────────

    /// 调查 OSINT 目标
    pub async fn investigate_osint(
        &self,
        target: &str,
    ) -> Result<Vec<OsintResult>, String> {
        let sources = self.registry.osint_sources(&SourceDomain::Osint);
        let mut results = Vec::new();

        for source in sources {
            match source.investigate(target).await {
                Ok(result) => {
                    if !result.findings.is_empty() {
                        results.push(result);
                    }
                }
                Err(e) => {
                    eprintln!("OSINT source {} failed: {}", source.id(), e);
                }
            }
        }

        Ok(results)
    }

    // ── LLM 查询 ────────────────────────────────────────────

    /// 调用 LLM 完成推理
    pub async fn llm_complete(
        &self,
        provider_id: &str,
        request: &LlmRequest,
    ) -> Result<LlmResponse, String> {
        let sources = self.registry.llm_providers(&SourceDomain::Llm);
        let source = sources
            .iter()
            .find(|s| s.id() == provider_id)
            .ok_or_else(|| format!("LLM provider not found: {}", provider_id))?;

        source.complete(request).await
    }

    /// 获取所有免费 LLM 提供者
    pub fn free_llm_providers(&self) -> Vec<&Arc<dyn LlmProvider>> {
        self.registry
            .llm_providers(&SourceDomain::Llm)
            .into_iter()
            .filter(|p| p.is_free())
            .collect()
    }

    // ── 跨域查询 ─────────────────────────────────────────────

    /// 按域列表查询所有源
    pub async fn search_by_domains(
        &self,
        query: &str,
        domains: &[SourceDomain],
    ) -> UnifiedSearchResult {
        let mut result = UnifiedSearchResult {
            media: Vec::new(),
            academic: Vec::new(),
            intel: Vec::new(),
            osint: Vec::new(),
            errors: Vec::new(),
        };

        for domain in domains {
            match domain {
                SourceDomain::Media => {
                    match self.search_media(query, 1).await {
                        Ok(r) => result.media = r,
                        Err(e) => result.errors.push(("media".into(), e)),
                    }
                }
                SourceDomain::Academic => {
                    match self.search_academic(query, 1).await {
                        Ok(r) => result.academic = r,
                        Err(e) => result.errors.push(("academic".into(), e)),
                    }
                }
                SourceDomain::Intel => {
                    match self.search_intel(query).await {
                        Ok(r) => result.intel = r,
                        Err(e) => result.errors.push(("intel".into(), e)),
                    }
                }
                SourceDomain::Osint => {
                    match self.investigate_osint(query).await {
                        Ok(r) => result.osint = r,
                        Err(e) => result.errors.push(("osint".into(), e)),
                    }
                }
                _ => {}
            }
        }

        result
    }

    /// 搜索所有域
    pub async fn search_all(&self, query: &str) -> UnifiedSearchResult {
        self.search_by_domains(query, SourceDomain::all()).await
    }

    // ── 管理 ─────────────────────────────────────────────────

    /// 获取注册表引用
    pub fn registry(&self) -> &DataSourceRegistry {
        &self.registry
    }

    /// 获取可变注册表引用
    pub fn registry_mut(&mut self) -> &mut DataSourceRegistry {
        &mut self.registry
    }

    /// 健康检查所有源
    pub async fn health_check(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        for source in self.registry.all() {
            let healthy = match source {
                AnyDataSource::Media(s) => s.health_check().await,
                AnyDataSource::Intel(s) => s.health_check().await,
                AnyDataSource::Osint(s) => s.health_check().await,
                AnyDataSource::Llm(s) => s.health_check().await,
                AnyDataSource::Crawl(s) => s.health_check().await,
                AnyDataSource::Local(s) => s.health_check().await,
            };
            results.push((source.id().to_string(), healthy));
        }
        results
    }

    /// 获取源统计
    pub fn stats(&self) -> EngineStats {
        let all = self.registry.all();
        let mut stats = EngineStats::default();

        for source in all {
            stats.total += 1;
            for domain in source.domains() {
                match domain {
                    SourceDomain::Media => stats.media += 1,
                    SourceDomain::Intel => stats.intel += 1,
                    SourceDomain::Academic => stats.academic += 1,
                    SourceDomain::Tech => stats.tech += 1,
                    SourceDomain::Local => stats.local += 1,
                    SourceDomain::Osint => stats.osint += 1,
                    SourceDomain::Llm => stats.llm += 1,
                    SourceDomain::Crawl => stats.crawl += 1,
                }
            }
        }

        stats
    }
}

impl Default for UnifiedEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一查询结果
#[derive(Debug, Clone)]
pub struct UnifiedSearchResult {
    pub media: Vec<MediaSearchResult>,
    pub academic: Vec<MediaSearchResult>,
    pub intel: Vec<IntelResult>,
    pub osint: Vec<OsintResult>,
    pub errors: Vec<(String, String)>,
}

impl UnifiedSearchResult {
    /// 总结果数
    pub fn total(&self) -> usize {
        self.media.iter().map(|r| r.total).sum::<usize>()
            + self.academic.iter().map(|r| r.total).sum::<usize>()
            + self.intel.iter().map(|r| r.total).sum::<usize>()
            + self.osint.iter().map(|r| r.findings.len()).sum::<usize>()
    }

    /// 是否有结果
    pub fn has_results(&self) -> bool {
        self.total() > 0
    }
}

/// 引擎统计
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub total: usize,
    pub media: usize,
    pub intel: usize,
    pub academic: usize,
    pub tech: usize,
    pub local: usize,
    pub osint: usize,
    pub llm: usize,
    pub crawl: usize,
}

impl std::fmt::Display for EngineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnifiedEngine: {} sources (media={}, intel={}, academic={}, tech={}, local={}, osint={}, llm={}, crawl={})",
            self.total, self.media, self.intel, self.academic, self.tech, self.local, self.osint, self.llm, self.crawl
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_engine_creation() {
        let engine = UnifiedEngine::new()
            .with_media_sources()
            .with_intel_sources()
            .with_osint_sources()
            .with_llm_providers();

        let stats = engine.stats();
        println!("{}", stats);

        assert!(stats.media > 0);
        assert!(stats.intel > 0);
        assert!(stats.osint > 0);
        assert!(stats.llm > 0);
    }

    #[tokio::test]
    async fn test_search_media() {
        let engine = UnifiedEngine::new().with_media_sources();
        let results = engine.search_media("test", 1).await.unwrap();
        // 至少有一些源返回结果
        assert!(results.len() <= 29); // 最多 29 个媒体源
    }

    #[tokio::test]
    async fn test_free_llm_providers() {
        let engine = UnifiedEngine::new().with_llm_providers();
        let free = engine.free_llm_providers();
        assert!(!free.is_empty());
    }
}
