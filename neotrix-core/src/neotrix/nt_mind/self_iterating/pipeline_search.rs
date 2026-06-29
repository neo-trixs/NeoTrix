use super::super::self_edit::MicroEdit;
use super::pipeline_core::{BrainStage, StageDecision};
use super::SelfIteratingBrain;
use crate::make_stage;
pub(crate) use crate::neotrix::nt_core_error::NeoTrixError;

make_stage!(MemoryRetrievalStage);
impl BrainStage for MemoryRetrievalStage {
    fn name(&self) -> &str {
        "memory_retrieval"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain.task_scratch.current_task_type;
        let embedding = brain._task_embedding();
        if let Some(ref emb) = embedding {
            brain
                .reasoning_bank
                .retrieve_relevant_by_embedding(emb, Some(task_type), 5);
        } else {
            brain
                .reasoning_bank
                .retrieve_relevant(&task, Some(task_type), 5);
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(GapAnalysisStage);
impl BrainStage for GapAnalysisStage {
    fn name(&self) -> &str {
        "gap_analysis"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref router) = brain.attention_router {
            let gap_reports = router.bridge.analyze_gaps();
            let domains = router.bridge.sparse_domains(&gap_reports);

            let mut gap_lines: Vec<String> = Vec::new();
            for report in &gap_reports {
                if report.gap > 0.0 {
                    gap_lines.push(format!(
                        "[dim {}] current={:.3}, target={:.3}, gap={:.3}",
                        report.dim_index, report.current_value, report.target_value, report.gap
                    ));
                    log::info!(
                        "[gap-analysis] dim {}: current={:.3}, target={:.3}, gap={:.3}",
                        report.dim_index,
                        report.current_value,
                        report.target_value,
                        report.gap
                    );
                }
            }

            if !domains.is_empty() {
                let domain_str: Vec<String> = domains.iter().map(|d| format!("{:?}", d)).collect();
                log::info!(
                    "[gap-analysis] exploration domains suggested: {:?}",
                    domain_str
                );
                gap_lines.push(format!("Suggested exploration: {}", domain_str.join(", ")));
            }

            if !gap_lines.is_empty() {
                let gap_summary = gap_lines.join(" | ");
                let existing = brain._open_source_insights.clone().unwrap_or_default();
                let combined = if existing.is_empty() {
                    format!("Knowledge gaps: {}", gap_summary)
                } else {
                    format!("{} | Knowledge gaps: {}", existing, gap_summary)
                };
                brain._open_source_insights = Some(combined);
            }
        } else {
            log::trace!("[gap-analysis] no attention_router, skipping");
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(OpenSourceCompareStage);
impl BrainStage for OpenSourceCompareStage {
    fn name(&self) -> &str {
        "open_source_compare"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task = brain._current_task();
        let task_type = brain.task_scratch.current_task_type;
        let capability = brain.brain.capability.clone();
        let benchmarker =
            crate::neotrix::nt_mind::open_source_benchmark::OpenSourceBenchmarker::new();

        let reports = benchmarker.benchmark_top3(&task, task_type, &capability);

        let dyn_edits: Vec<MicroEdit> = Vec::new();

        if reports.is_empty() || (reports[0].relevance_score < 0.3 && dyn_edits.is_empty()) {
            brain._open_source_insights = None;
            brain._set_open_source_edits(Vec::new());
            return Ok(StageDecision::Skip(
                "No relevant open-source projects found".into(),
            ));
        }

        let mut summary_lines = Vec::new();
        let mut all_edits = Vec::new();

        for report in &reports {
            summary_lines.push(report.summary.clone());
            for (idx, delta, detail) in &report.gap_areas {
                let edit = MicroEdit::AdjustDimension(idx.to_string(), *delta);
                if !all_edits.iter().any(|e: &MicroEdit| matches!(e, MicroEdit::AdjustDimension(i, _) if *i == idx.to_string())) {
                    all_edits.push(edit);
                }
                log::info!("[open-source] gap: {}", detail);
            }
        }

        for edit in dyn_edits {
            if !all_edits.iter().any(|e: &MicroEdit| {
                matches!((e, &edit), (MicroEdit::AdjustDimension(i1, _), MicroEdit::AdjustDimension(i2, _)) if *i1 == *i2)
            }) {
                all_edits.push(edit);
            }
        }

        if !all_edits.is_empty() {
            all_edits.push(MicroEdit::NormalizeVector);
        }

        let insights = summary_lines.join(" | ");
        brain._open_source_insights = Some(insights);
        brain._set_open_source_edits(all_edits);
        Ok(StageDecision::Continue)
    }
}

make_stage!(CodeSearchStage);
impl BrainStage for CodeSearchStage {
    fn name(&self) -> &str {
        "code_search"
    }
    fn frequency(&self) -> usize {
        3
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        use crate::neotrix::nt_world_code_search::CodeSearchEngine;
        let workspace = std::env::var("NEOTRIX_WORKSPACE").unwrap_or_else(|_| ".".to_string());
        let query = std::env::var("NEOTRIX_CODE_QUERY").ok();
        if let Some(q) = query {
            if !q.trim().is_empty() {
                let mut engine = CodeSearchEngine::with_root(std::path::Path::new(&workspace));
                match engine.format_results(&q, 5) {
                    Ok(output) => {
                        log::info!("[code_search] query='{}' → {} chars", q, output.len());
                        brain.code_search_cache = Some(output);
                    }
                    Err(e) => {
                        log::warn!("[code_search] failed: {}", e);
                        brain.code_search_cache = Some(format!("Code search error: {}", e));
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(EmbeddingRefreshStage);
impl BrainStage for EmbeddingRefreshStage {
    fn name(&self) -> &str {
        "embedding_refresh"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let kb = match brain.reasoning_engine.as_mut().and_then(|e| e.kb.as_mut()) {
            Some(k) => k,
            None => return Ok(StageDecision::Skip("no KB attached".to_string())),
        };
        let has_config = kb
            .embedding_config
            .read()
            .map(|r| r.is_some())
            .unwrap_or(false);
        if !has_config {
            return Ok(StageDecision::Skip("no embedding config".to_string()));
        }
        match kb.ensure_embeddings() {
            Ok(count) => {
                if count > 0 {
                    log::info!("[embedding-refresh] generated {} missing embeddings", count);
                }
            }
            Err(e) => {
                log::warn!("[embedding-refresh] {}", e);
            }
        }
        Ok(StageDecision::Continue)
    }
}

make_stage!(KnowledgeAbsorbStage);
impl BrainStage for KnowledgeAbsorbStage {
    fn name(&self) -> &str {
        "knowledge_absorb"
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let task_type = brain.task_scratch.current_task_type;
        let score_before = brain._snapshot_score();
        if score_before < brain.quality_threshold && brain.auto_absorb {
            let sources = brain.select_relevant_sources(task_type);
            brain.brain.absorb_batch(&sources);
        }
        Ok(StageDecision::Continue)
    }
}
