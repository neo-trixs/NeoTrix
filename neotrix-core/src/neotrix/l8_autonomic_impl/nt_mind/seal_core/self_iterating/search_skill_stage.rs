//! SearchSkillStage — 联网搜索技能内化阶段
//!
//! smol-course 吸收：Unit 1 tool calling + Unit 3 VLM grounding + Unit 6 synthetic data
//! 将 nt_world_crawl (UnifiedCrawler/FetcherPool/ContentClassifier) 包装为可学习技能，
//! 接入 data_synthesis PSV 管线生成搜索训练数据，内化为 CapabilityVector 扩展维度。
//!
//! 核心能力维度：
//! - query_generation: 任务 → 搜索查询的质量
//! - result_filtering: 原始结果 → 相关证据的精准度
//! - evidence_synthesis: 多源证据 → 结构化知识的融合度
//! - grounding_quality: 回答对证据的忠实度 (grounding)

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use super::pipeline::StageResult;

/// 搜索技能缓冲大小
pub const SEARCH_SKILL_BUFFER_SIZE: usize = 200;

/// 搜索技能学习率
pub const SEARCH_SKILL_LEARNING_RATE: f64 = 1e-4;

/// 搜索任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchTaskType {
    FactLookup,       // 事实查找
    TechnicalQuery,   // 技术问题
    CodeExample,      // 代码示例搜索
    ResearchSurvey,   // 文献/调研
    Verification,     // 事实核查
    TrendAnalysis,    // 趋势分析
}

impl SearchTaskType {
    pub fn all() -> [SearchTaskType; 6] {
        [SearchTaskType::FactLookup, SearchTaskType::TechnicalQuery, SearchTaskType::CodeExample,
         SearchTaskType::ResearchSurvey, SearchTaskType::Verification, SearchTaskType::TrendAnalysis]
    }
    pub fn label(&self) -> &'static str {
        match self {
            SearchTaskType::FactLookup => "fact_lookup",
            SearchTaskType::TechnicalQuery => "technical_query",
            SearchTaskType::CodeExample => "code_example",
            SearchTaskType::ResearchSurvey => "research_survey",
            SearchTaskType::Verification => "verification",
            SearchTaskType::TrendAnalysis => "trend_analysis",
        }
    }
}

/// 单次搜索技能演练记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchExercise {
    pub exercise_id: String,
    pub task_type: SearchTaskType,
    pub query: String,                    // 生成的搜索查询
    pub raw_results: Vec<SearchResult>,   // 原始搜索结果
    pub filtered_evidence: Vec<Evidence>, // 筛选后的证据
    pub synthesized_answer: String,       // 综合后的回答
    pub grounding_score: f64,             // Grounding 质量 (0-1)
    pub relevance_score: f64,             // 证据相关性 (0-1)
    pub synthesis_quality: f64,           // 综合质量 (0-1)
    pub latency_ms: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub source_type: String,              // "web" | "academic" | "code" | "doc"
    pub credibility: f64,                 // 来源可信度 (0-1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub source_url: String,
    pub claim: String,
    pub confidence: f64,
    pub supports_answer: bool,
}

/// 搜索技能缓冲
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSkillBuffer {
    pub exercises: VecDeque<SearchExercise>,
    pub max_size: usize,
}

impl Default for SearchSkillBuffer {
    fn default() -> Self { Self::new() }
}

impl SearchSkillBuffer {
    pub fn new() -> Self {
        Self { exercises: VecDeque::with_capacity(SEARCH_SKILL_BUFFER_SIZE), max_size: SEARCH_SKILL_BUFFER_SIZE }
    }
    pub fn push(&mut self, ex: SearchExercise) {
        if self.exercises.len() >= self.max_size { self.exercises.pop_front(); }
        self.exercises.push_back(ex);
    }
    pub fn len(&self) -> usize { self.exercises.len() }
    pub fn is_empty(&self) -> bool { self.exercises.is_empty() }
    pub fn clear(&mut self) { self.exercises.clear(); }
    pub fn by_type(&self, t: SearchTaskType) -> Vec<&SearchExercise> {
        self.exercises.iter().filter(|e| e.task_type == t).collect()
    }
}

/// 搜索技能阶段报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSkillReport {
    pub total_updates: u64,
    pub buffer_size: usize,
    pub avg_grounding: f64,
    pub avg_relevance: f64,
    pub avg_synthesis: f64,
    pub type_distribution: std::collections::HashMap<String, usize>,
    pub extension_dims_updated: Vec<String>,
}

/// Search Skill Stage for SEAL pipeline.
///
/// 搜索技能内化：通过 PSV 管线生成搜索演练数据，学习 query/evidence/synthesis 三子技能，
/// 累积到 CapabilityVector.extension (nt_cap:search_*)。
#[derive(Debug, Clone)]
pub struct SearchSkillStage {
    pub buffer: SearchSkillBuffer,
    pub learning_rate: f64,
    pub total_updates: u64,
    /// 搜索子技能维度 (EMA 累积)
    pub query_generation: f64,
    pub result_filtering: f64,
    pub evidence_synthesis: f64,
    pub grounding_quality: f64,
    /// PSV 管线引用 (用于生成训练数据)
    pub synthesis_pipeline: Option<super::data_synthesis::AsymmetricSynthesisPipeline>,
}

impl Default for SearchSkillStage {
    fn default() -> Self { Self::new() }
}

impl SearchSkillStage {
    pub fn new() -> Self {
        Self {
            buffer: SearchSkillBuffer::new(),
            learning_rate: SEARCH_SKILL_LEARNING_RATE,
            total_updates: 0,
            query_generation: 0.0,
            result_filtering: 0.0,
            evidence_synthesis: 0.0,
            grounding_quality: 0.0,
            synthesis_pipeline: None,
        }
    }

    /// 绑定 PSV 合成管线 (用于生成搜索训练数据)
    pub fn with_synthesis_pipeline(mut self, pipeline: super::data_synthesis::AsymmetricSynthesisPipeline) -> Self {
        self.synthesis_pipeline = Some(pipeline);
        self
    }

    /// 从 nt_world_crawl 执行一次搜索演练 (需外部提供 crawler 实例)
    /// 返回 SearchExercise，由外部调用者负责执行实际搜索
    pub fn create_exercise_from_crawl(
        &self,
        task_type: SearchTaskType,
        query: String,
        crawl_results: Vec<crate::neotrix::nt_world_crawl::fetcher::FetchResult>,
        synthesized_answer: String,
        grounding_score: f64,
    ) -> SearchExercise {
        // 简化：将 FetchResult 转为 SearchResult
        let raw_results: Vec<SearchResult> = crawl_results.into_iter().map(|r| SearchResult {
            url: r.url.clone(),
            title: r.url.split('/').last().unwrap_or("result").to_string(),
            snippet: r.text.as_deref().unwrap_or("").chars().take(300).collect(),
            source_type: "web".into(),
            credibility: if r.is_success() { 0.8 } else { 0.1 },
        }).collect();

        // 简化证据筛选：取前 3 个高可信度结果
        let mut filtered: Vec<Evidence> = raw_results.iter()
            .filter(|r| r.credibility > 0.6)
            .take(3)
            .map(|r| Evidence {
                source_url: r.url.clone(),
                claim: r.snippet.clone(),
                confidence: r.credibility,
                supports_answer: true, // 简化假设
            }).collect();

        let relevance = if filtered.is_empty() { 0.0 } else {
            filtered.iter().map(|e| e.confidence).sum::<f64>() / filtered.len() as f64
        };
        let synthesis = grounding_score * relevance; // 简化综合分

        SearchExercise {
            exercise_id: format!("search-{}-{}", task_type.label(), current_timestamp()),
            task_type,
            query,
            raw_results,
            filtered_evidence: filtered,
            synthesized_answer,
            grounding_score,
            relevance_score: relevance,
            synthesis_quality: synthesis,
            latency_ms: 0,
            timestamp: current_timestamp(),
        }
    }

    /// 从 PSV 管线生成搜索训练数据 (Proposer→Solver→Verifier)
    /// Solver 闭包中调用实际搜索工具
    pub fn generate_training_data<F>(
        &mut self,
        gaps: Vec<super::data_synthesis::KnowledgeGap>,
        solver: F,
    ) -> Vec<SearchExercise>
    where
        F: Fn(&super::data_synthesis::DataProposal) -> super::data_synthesis::DataSolution,
    {
        let mut exercises = Vec::new();
        if let Some(ref mut pipeline) = self.synthesis_pipeline {
            // Proposer: 识别搜索类知识缺口
            pipeline.propose(gaps, |gap| {
                let task_type = match gap.suggested_task_type {
                    super::data_synthesis::DataTaskType::KnowledgeQA => SearchTaskType::FactLookup,
                    super::data_synthesis::DataTaskType::CodeGeneration => SearchTaskType::CodeExample,
                    super::data_synthesis::DataTaskType::Reasoning => SearchTaskType::TechnicalQuery,
                    super::data_synthesis::DataTaskType::ToolUse => SearchTaskType::Verification,
                    _ => SearchTaskType::ResearchSurvey,
                };
                super::data_synthesis::DataProposal {
                    id: format!("search-{}", gap.domain),
                    gap: gap.clone(),
                    prompt_template: format!("Search for: {}", gap.description),
                    expected_difficulty: gap.priority,
                    diversity_hash: current_timestamp(),
                }
            });

            // Solver: 执行搜索 (外部传入 solver 闭包)
            pipeline.solve(solver);

            // Verifier: 评估搜索质量
            pipeline.verify(|proposal, solution| {
                // 简化验证：confidence 作为 grounding_score
                let grounding = solution.confidence.clamp(0.0, 1.0);
                let relevance = (solution.confidence * 0.8).clamp(0.0, 1.0);
                super::data_synthesis::VerificationResult {
                    proposal_id: proposal.id.clone(),
                    passed: grounding > 0.6,
                    quality_score: (grounding + relevance) / 2.0,
                    issues: if grounding < 0.6 { vec!["low grounding".into()] } else { vec![] },
                    fix_suggestion: if grounding < 0.6 { Some("improve query specificity".into()) } else { None },
                    retry_count: 0,
                }
            });

            // Finalize: 转为 SearchExercise
            let stats = pipeline.finalize();
            for (proposal, (solution, verification)) in pipeline.proposals.iter()
                .zip(pipeline.solutions.iter().zip(pipeline.verifications.iter()))
            {
                if verification.passed {
                    let task_type = match proposal.gap.suggested_task_type {
                        super::data_synthesis::DataTaskType::KnowledgeQA => SearchTaskType::FactLookup,
                        super::data_synthesis::DataTaskType::CodeGeneration => SearchTaskType::CodeExample,
                        super::data_synthesis::DataTaskType::Reasoning => SearchTaskType::TechnicalQuery,
                        super::data_synthesis::DataTaskType::ToolUse => SearchTaskType::Verification,
                        _ => SearchTaskType::ResearchSurvey,
                    };
                    exercises.push(SearchExercise {
                        exercise_id: format!("psv-{}-{}", task_type.label(), current_timestamp()),
                        task_type,
                        query: proposal.prompt_template.clone(),
                        raw_results: vec![],
                        filtered_evidence: vec![],
                        synthesized_answer: solution.response.clone(),
                        grounding_score: verification.quality_score,
                        relevance_score: verification.quality_score * 0.9,
                        synthesis_quality: verification.quality_score,
                        latency_ms: solution.token_count,
                        timestamp: current_timestamp(),
                    });
                }
            }
        }
        exercises
    }

    /// 计算搜索技能 loss
    fn compute_search_loss(&self) -> f64 {
        if self.buffer.is_empty() { return 0.0; }
        let n = self.buffer.len() as f64;
        let total: f64 = self.buffer.exercises.iter().map(|ex| {
            let margin_q = self.query_generation - ex.grounding_score;
            let margin_f = self.result_filtering - ex.relevance_score;
            let margin_s = self.evidence_synthesis - ex.synthesis_quality;
            let margin_g = self.grounding_quality - ex.grounding_score;
            let loss = |m: f64| if m > 0.0 { (-m).exp().ln_1p() } else { m.exp().ln_1p() - m };
            (loss(margin_q) + loss(margin_f) + loss(margin_s) + loss(margin_g)) / 4.0
        }).sum();
        total / n
    }

    /// 处理搜索演练批次：更新子技能 EMA
    pub fn process(&mut self, exercises: Vec<SearchExercise>) -> (StageResult, f64) {
        let result = StageResult::new("search_skill_stage");
        if exercises.is_empty() { return (result, 0.0); }

        for ex in &exercises {
            self.buffer.push(ex.clone());
            let w = self.learning_rate;
            self.query_generation = self.query_generation * (1.0 - w) + ex.grounding_score * w;
            self.result_filtering = self.result_filtering * (1.0 - w) + ex.relevance_score * w;
            self.evidence_synthesis = self.evidence_synthesis * (1.0 - w) + ex.synthesis_quality * w;
            self.grounding_quality = self.grounding_quality * (1.0 - w) + ex.grounding_score * w;
        }
        self.total_updates += 1;

        let loss = self.compute_search_loss();
        log::trace!(
            "[search_skill_stage] loss={:.4} query={:.3} filter={:.3} synth={:.3} ground={:.3} exercises={}",
            loss, self.query_generation, self.result_filtering, self.evidence_synthesis,
            self.grounding_quality, self.buffer.len()
        );
        (result, loss)
    }

    /// 将搜索技能写入 CapabilityVector.extension
    pub fn sync_to_capability_vector(&self, cv: &mut crate::core::CapabilityVector) {
        cv.add_extension_dim("nt_cap:search_query_generation", self.query_generation);
        cv.add_extension_dim("nt_cap:search_result_filtering", self.result_filtering);
        cv.add_extension_dim("nt_cap:search_evidence_synthesis", self.evidence_synthesis);
        cv.add_extension_dim("nt_cap:search_grounding_quality", self.grounding_quality);
        cv.set_provenance("search_skill_stage".to_string());
    }

    /// 从 CapabilityVector 读取搜索技能
    pub fn load_from_capability_vector(cv: &crate::core::CapabilityVector) -> Self {
        let mut stage = Self::new();
        for (name, val) in cv.extension() {
            match name.as_str() {
                "nt_cap:search_query_generation" => stage.query_generation = *val,
                "nt_cap:search_result_filtering" => stage.result_filtering = *val,
                "nt_cap:search_evidence_synthesis" => stage.evidence_synthesis = *val,
                "nt_cap:search_grounding_quality" => stage.grounding_quality = *val,
                _ => {}
            }
        }
        stage
    }

    pub fn report(&self) -> SearchSkillReport {
        use std::collections::HashMap;
        let mut type_dist = HashMap::new();
        let mut sum_g = 0.0; let mut sum_r = 0.0; let mut sum_s = 0.0;
        for ex in &self.buffer.exercises {
            *type_dist.entry(ex.task_type.label().into()).or_insert(0) += 1;
            sum_g += ex.grounding_score;
            sum_r += ex.relevance_score;
            sum_s += ex.synthesis_quality;
        }
        let n = self.buffer.len().max(1) as f64;
        SearchSkillReport {
            total_updates: self.total_updates,
            buffer_size: self.buffer.len(),
            avg_grounding: sum_g / n,
            avg_relevance: sum_r / n,
            avg_synthesis: sum_s / n,
            type_distribution: type_dist,
            extension_dims_updated: vec![
                "nt_cap:search_query_generation".into(),
                "nt_cap:search_result_filtering".into(),
                "nt_cap:search_evidence_synthesis".into(),
                "nt_cap:search_grounding_quality".into(),
            ],
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_exercise(task_type: SearchTaskType, grounding: f64, relevance: f64, synthesis: f64) -> SearchExercise {
        SearchExercise {
            exercise_id: "test".into(), task_type, query: "test query".into(),
            raw_results: vec![], filtered_evidence: vec![], synthesized_answer: "ans".into(),
            grounding_score: grounding, relevance_score: relevance, synthesis_quality: synthesis,
            latency_ms: 100, timestamp: current_timestamp(),
        }
    }

    #[test]
    fn test_buffer_push_and_by_type() {
        let mut buf = SearchSkillBuffer::new();
        buf.push(make_exercise(SearchTaskType::FactLookup, 0.9, 0.8, 0.85));
        buf.push(make_exercise(SearchTaskType::CodeExample, 0.7, 0.6, 0.65));
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.by_type(SearchTaskType::FactLookup).len(), 1);
    }

    #[test]
    fn test_process_updates_ema() {
        let mut stage = SearchSkillStage::new();
        stage.learning_rate = 0.5;
        let ex = make_exercise(SearchTaskType::FactLookup, 1.0, 1.0, 1.0);
        stage.process(vec![ex]);
        assert!((stage.query_generation - 0.5).abs() < 1e-6);
        assert!((stage.grounding_quality - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sync_to_capability_vector() {
        let mut stage = SearchSkillStage::new();
        stage.query_generation = 0.8; stage.grounding_quality = 0.9;
        let mut cv = crate::core::CapabilityVector::default();
        stage.sync_to_capability_vector(&mut cv);
        assert!((cv.extension().iter().find(|(n,_)| n=="nt_cap:search_query_generation").unwrap().1 - 0.8).abs() < 1e-9);
    }

    #[test]
    fn test_load_from_capability_vector() {
        let mut cv = crate::core::CapabilityVector::default();
        cv.add_extension_dim("nt_cap:search_result_filtering", 0.7);
        cv.add_extension_dim("nt_cap:search_evidence_synthesis", 0.6);
        let stage = SearchSkillStage::load_from_capability_vector(&cv);
        assert!((stage.result_filtering - 0.7).abs() < 1e-9);
        assert!((stage.evidence_synthesis - 0.6).abs() < 1e-9);
    }

    #[test]
    fn test_search_task_type_labels() {
        assert_eq!(SearchTaskType::FactLookup.label(), "fact_lookup");
        assert_eq!(SearchTaskType::CodeExample.label(), "code_example");
        assert_eq!(SearchTaskType::all().len(), 6);
    }

    #[test]
    fn test_report_tracks_distribution() {
        let mut stage = SearchSkillStage::new();
        stage.process(vec![make_exercise(SearchTaskType::FactLookup, 0.9, 0.8, 0.85)]);
        let r = stage.report();
        assert_eq!(r.buffer_size, 1);
        assert!(r.type_distribution.contains_key("fact_lookup"));
    }
}