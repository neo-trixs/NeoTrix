use crate::core::nt_core_time::unix_now;

use super::content::NegentropyScore;
use super::pipeline::NegentropyPipeline;
use super::source_trait::ExplorationSource;
use super::sources::{
    papers_with_code::{PaperQueryMode, PaperSource},
    ApiSource, BrowserSource, FileSource, PdfSource, SearchSource,
};

/// 探索调度决策
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleDecision {
    ExploreBrowser,
    ExploreApi,
    ExploreSearch,
    ExploreFile,
    ExplorePdf,
    ExplorePaper,
    Skip, // 所有来源暂无待处理项
    Wait, // 来源忙碌/冷却中
}

/// 探索编排器 — 统一管理所有外部探索源
///
/// 核心职责:
/// 1. 按好奇心/知识缺口调度探索源
/// 2. 所有来源的输出经过统一负熵管线评分
/// 3. 高负熵内容注入意识核心
/// 4. 反馈闭环: 产生 seed queries 给待探索缺口
pub struct ExplorationOrchestrator {
    pub pipeline: NegentropyPipeline,
    pub browser: BrowserSource,
    pub api: ApiSource,
    pub search: SearchSource,
    pub file: FileSource,
    pub pdf: PdfSource,
    pub paper: PaperSource,
    pub known_concepts: Vec<String>,
    /// 探索冷却 (秒) — 每个来源探索后冷却
    cooldowns: [u64; 6], // [browser, api, search, file, pdf, paper]
    last_explore: [u64; 6],
}

impl ExplorationOrchestrator {
    pub fn new() -> Self {
        Self {
            pipeline: NegentropyPipeline::new(),
            browser: BrowserSource::new("https://x.com/home"),
            api: ApiSource::new(super::sources::api::ApiMode::GitHub),
            search: SearchSource::new(),
            file: FileSource::new(),
            pdf: PdfSource::new(),
            paper: PaperSource::new(PaperQueryMode::Trending),
            known_concepts: Vec::new(),
            cooldowns: [30, 60, 30, 120, 300, 120],
            last_explore: [0; 6],
        }
    }

    pub fn with_cooldowns(
        mut self,
        browser: u64,
        api: u64,
        search: u64,
        file: u64,
        pdf: u64,
        paper: u64,
    ) -> Self {
        self.cooldowns = [browser, api, search, file, pdf, paper];
        self
    }

    /// 根据好奇心水平决定探索什么
    pub fn schedule(&self, curiosity_bonus: f64) -> ScheduleDecision {
        if curiosity_bonus < 0.1 {
            return ScheduleDecision::Skip;
        }

        let now = unix_now() as u64;

        // 高好奇心 → 优先浏览器 (实时性最高)
        if curiosity_bonus > 0.6 && now - self.last_explore[0] > self.cooldowns[0] {
            if self.browser.is_ready() {
                return ScheduleDecision::ExploreBrowser;
            }
        }

        // API 查询 (知识注入)
        if now - self.last_explore[1] > self.cooldowns[1] && self.api.pending_count() > 0 {
            return ScheduleDecision::ExploreApi;
        }

        // 搜索引擎
        if curiosity_bonus > 0.3
            && now - self.last_explore[2] > self.cooldowns[2]
            && self.search.pending_count() > 0
        {
            return ScheduleDecision::ExploreSearch;
        }

        // 文件
        if curiosity_bonus > 0.5 && now - self.last_explore[3] > self.cooldowns[3] {
            return ScheduleDecision::ExploreFile;
        }

        // PDF文档库 (结构化知识注入, 低好奇心也调度)
        if now - self.last_explore[4] > self.cooldowns[4] && self.pdf.pending_count() > 0 {
            return ScheduleDecision::ExplorePdf;
        }

        // 论文数据库 (结构化知识注入)
        if now - self.last_explore[5] > self.cooldowns[5] && self.paper.pending_count() > 0 {
            return ScheduleDecision::ExplorePaper;
        }

        ScheduleDecision::Wait
    }

    /// 执行一次探索循环 — 由 SEAL 或好奇心驱动调用
    pub fn explore_cycle(&mut self, curiosity_bonus: f64) -> Result<Vec<NegentropyScore>, String> {
        match self.schedule(curiosity_bonus) {
            ScheduleDecision::Skip | ScheduleDecision::Wait => Ok(Vec::new()),
            ScheduleDecision::ExploreBrowser => {
                let raw = self.browser.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[0] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.3, 10);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
            ScheduleDecision::ExploreApi => {
                let raw = self.api.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[1] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.3, 10);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
            ScheduleDecision::ExploreSearch => {
                let raw = self.search.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[2] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.3, 10);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
            ScheduleDecision::ExploreFile => {
                let raw = self.file.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[3] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.3, 10);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
            ScheduleDecision::ExplorePdf => {
                let raw = self.pdf.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[4] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.5, 15);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
            ScheduleDecision::ExplorePaper => {
                let raw = self.paper.explore()?;
                if raw.is_empty() {
                    return Ok(Vec::new());
                }
                self.last_explore[5] = unix_now() as u64;
                let results = self
                    .pipeline
                    .process_batch(&raw, &self.known_concepts, 0.5, 15);
                for score in &results {
                    for k in NegentropyPipeline::extract_keywords(&score.content.text) {
                        if !self.known_concepts.contains(&k) {
                            self.known_concepts.push(k);
                        }
                    }
                }
                self.known_concepts.truncate(500);
                Ok(results)
            }
        }
    }

    /// 从知识缺口生成 API/搜索/论文查询
    pub fn seed_from_gaps(&mut self, gap_terms: &[String]) {
        for term in gap_terms {
            self.api.enqueue(term.clone());
            self.search.search(term.clone());
            self.paper.search(term.clone());
        }
        // PDF扫描由目录配置驱动, 不从gap自动补种
    }

    pub fn reset_cooldowns(&mut self) {
        self.last_explore = [0; 6];
    }

    pub fn stats(&self) -> OrchestratorStats {
        OrchestratorStats {
            total_seen: self.pipeline.total_seen,
            absorbed: self.pipeline.absorbed_count,
            dedup_size: self.pipeline.seen.len(),
            api_pending: self.api.pending_count(),
            search_pending: self.search.pending_count(),
            pdf_pending: self.pdf.pending_count(),
            paper_pending: self.paper.pending_count(),
            known_concepts: self.known_concepts.len(),
        }
    }

    pub fn pipeline_mut(&mut self) -> &mut NegentropyPipeline {
        &mut self.pipeline
    }
}

#[derive(Debug, Clone)]
pub struct OrchestratorStats {
    pub total_seen: usize,
    pub absorbed: usize,
    pub dedup_size: usize,
    pub api_pending: usize,
    pub search_pending: usize,
    pub pdf_pending: usize,
    pub paper_pending: usize,
    pub known_concepts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_world_exploration::content::SourceContent;

    fn mock_content(id: &str, text: &str) -> SourceContent {
        SourceContent::new(
            id,
            text,
            super::super::content::ExplorationSourceType::BrowserSocial,
        )
    }

    #[test]
    fn test_schedule_skip_low_curiosity() {
        let o = ExplorationOrchestrator::new();
        assert_eq!(o.schedule(0.05), ScheduleDecision::Skip);
    }

    #[test]
    fn test_schedule_wait_when_no_pending() {
        let o = ExplorationOrchestrator::new();
        let d = o.schedule(0.4);
        assert!(d == ScheduleDecision::Wait || d == ScheduleDecision::Skip);
    }

    #[test]
    fn test_schedule_api_when_pending() {
        let mut o = ExplorationOrchestrator::new();
        o.api.enqueue("machine learning");
        o.reset_cooldowns();
        let d = o.schedule(0.5);
        assert_eq!(d, ScheduleDecision::ExploreApi);
    }

    #[test]
    fn test_schedule_search_when_pending() {
        let mut o = ExplorationOrchestrator::new();
        o.search.search("quantum computing");
        o.reset_cooldowns();
        let d = o.schedule(0.4);
        assert_eq!(d, ScheduleDecision::ExploreSearch);
    }

    #[test]
    fn test_seed_from_gaps() {
        let mut o = ExplorationOrchestrator::new();
        assert_eq!(o.api.pending_count(), 0);
        assert_eq!(o.search.pending_count(), 0);
        assert_eq!(o.paper.pending_count(), 0);

        o.seed_from_gaps(&["neural networks".into(), "reinforcement learning".into()]);

        assert!(o.api.pending_count() >= 2);
        assert!(o.search.pending_count() >= 2);
        assert!(o.paper.pending_count() >= 2);
    }

    #[test]
    fn test_schedule_paper_when_pending() {
        let mut o = ExplorationOrchestrator::new();
        o.paper.search("transformer architecture");
        o.reset_cooldowns();
        let d = o.schedule(0.2);
        assert_eq!(d, ScheduleDecision::ExplorePaper);
    }

    #[test]
    fn test_stats_include_paper() {
        let o = ExplorationOrchestrator::new();
        let stats = o.stats();
        assert_eq!(stats.paper_pending, 0);
    }

    #[test]
    fn test_explore_cycle_with_results_via_direct_pipeline() {
        let mut p = NegentropyPipeline::new();
        let items = vec![
            mock_content("t1", "This is a substantive tweet about artificial intelligence and machine learning systems that contains enough text to score above the absorption threshold"),
            mock_content("t2", "short"),
        ];
        let results = p.process_batch(&items, &["ai".into()], 0.5, 10);
        assert!(
            results.len() >= 1,
            "at least the long tweet should be absorbed"
        );
        for r in &results {
            assert!(r.negentropy > 0.25);
        }
    }

    #[test]
    fn test_orchestrator_cycle_api() {
        let mut o = ExplorationOrchestrator::new();
        o.seed_from_gaps(&["quantum physics".into()]);
        o.reset_cooldowns();

        let d = o.schedule(0.6);
        assert_eq!(d, ScheduleDecision::ExploreApi);

        let results = o.explore_cycle(0.6).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_stats() {
        let o = ExplorationOrchestrator::new();
        let stats = o.stats();
        assert_eq!(stats.total_seen, 0);
        assert_eq!(stats.absorbed, 0);
        assert_eq!(stats.known_concepts, 0);
    }

    #[test]
    fn test_pdf_default_pending_zero() {
        let o = ExplorationOrchestrator::new();
        assert_eq!(o.pdf.pending_count(), 0);
    }
}
