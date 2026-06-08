use super::{IngestionPipeline, IngestionConfig, IngestionResult, SourceType};
use super::reflection_loop::ReflectionLoop;

pub struct PaperPipeline {
    steps: Vec<String>,
}

impl PaperPipeline {
    pub fn new() -> Self {
        Self {
            steps: vec![
                "extract_abstract".to_string(),
                "identify_contributions".to_string(),
                "extract_methods".to_string(),
                "capture_results".to_string(),
                "map_related_work".to_string(),
                "gather_citations".to_string(),
                "assess_reproducibility".to_string(),
                "generate_summary".to_string(),
            ],
        }
    }

    fn extract_title(&self, input: &str) -> String {
        input.lines()
            .next()
            .map(|l| {
                let t = l.trim();
                if t.len() > 100 { format!("{}...", &t[..100]) } else { t.to_string() }
            })
            .unwrap_or_else(|| "unknown_paper".to_string())
    }

    fn simulate_round_analysis(&self, input: &str, round: usize) -> (Vec<String>, f64) {
        let mut insights = Vec::new();
        let title = self.extract_title(input);

        if round == 1 {
            insights.push(format!("analyzed: {}", title));
            insights.push("abstract_extracted".to_string());
            insights.push("contributions_identified".to_string());
        } else if round == 2 {
            insights.push("methods_classified".to_string());
            insights.push("results_captured".to_string());
        } else {
            insights.push(format!("deep_analysis_round_{}", round));
            insights.push("related_work_mapped".to_string());
            insights.push("citations_gathered".to_string());
        }

        let clarity_delta = (0.8_f64).powi(round as i32);
        (insights, clarity_delta)
    }
}

impl Default for PaperPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl IngestionPipeline for PaperPipeline {
    fn name(&self) -> &str {
        "paper_pipeline"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Paper
    }

    fn process(&self, input: &str, config: &IngestionConfig) -> IngestionResult {
        let mut reflection = ReflectionLoop::new(config.max_rounds, config.convergence_threshold);
        let title = self.extract_title(input);

        while reflection.should_continue() {
            let round = reflection.current_round();
            let (insights, clarity_delta) = self.simulate_round_analysis(input, round);
            reflection.record_round(insights, clarity_delta);
        }

        let _step_count = self.steps.len();
        IngestionResult {
            source_type: SourceType::Paper,
            title: title.clone(),
            summary: format!("processed via {} steps in {} rounds | title: {}",
                self.steps.len(), reflection.total_rounds(), title),
            total_rounds: reflection.total_rounds(),
            final_quality: reflection.best_quality(),
            converged: reflection.converged,
            entities: vec!["paper".to_string(), "method".to_string(), "result".to_string()],
            relations: vec![
                ("paper".to_string(), "proposes".to_string(), "method".to_string()),
                ("method".to_string(), "produces".to_string(), "result".to_string()),
            ],
            reasoning_notes: reflection.all_insights().into_iter().take(5).collect(),
            reflection_history: reflection.rounds,
        }
    }
}
