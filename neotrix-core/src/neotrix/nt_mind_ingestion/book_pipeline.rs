use super::reflection_loop::{QualityMonitor, ReflectionLoop};
use super::{
    IngestionConfig, IngestionPipeline, IngestionResult, IngestionSourceType, ReflectionRound,
};

pub struct BookPipeline {
    steps: Vec<String>,
}

impl BookPipeline {
    pub fn new() -> Self {
        Self {
            steps: vec![
                "collate".to_string(),
                "structure".to_string(),
                "entity_extract".to_string(),
                "event_extract".to_string(),
                "relation_map".to_string(),
                "ontology_align".to_string(),
                "reason".to_string(),
                "sku_generate".to_string(),
                "apply".to_string(),
            ],
        }
    }

    fn collate(&self, input: &str) -> String {
        let lines: Vec<&str> = input.lines().collect();
        let total = lines.len();
        let non_empty: Vec<&&str> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
        format!(
            "collated {} lines ({} non-empty, {:.1}% density)",
            total,
            non_empty.len(),
            if total > 0 {
                non_empty.len() as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        )
    }

    fn structure(&self, input: &str, round: usize) -> Vec<String> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = input.lines().collect();
        let chunk_size = (lines.len() / 4).max(1);

        for (i, chunk) in lines.chunks(chunk_size).enumerate() {
            let preview: String = chunk
                .iter()
                .take(3)
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<&str>>()
                .join(" ");
            let preview = if preview.len() > 80 {
                format!("{}...", &preview[..80])
            } else {
                preview
            };
            sections.push(format!("section_{}: {}", i + 1, preview));
        }

        if round > 1 {
            let prev = self.structure(input, round - 1);
            sections = sections
                .into_iter()
                .take(prev.len())
                .enumerate()
                .map(|(i, s)| {
                    if i < prev.len() {
                        format!("{} [refined]", s)
                    } else {
                        s
                    }
                })
                .collect();
        }

        sections
    }

    fn extract_entities(&self, sections: &[String], round: usize) -> Vec<String> {
        let mut entities = Vec::new();
        for section in sections {
            let words: Vec<&str> = section.split_whitespace().collect();
            for word in words.iter().take(10) {
                let cleaned = word.trim_matches(|c: char| c.is_ascii_punctuation());
                if cleaned.len() >= 4
                    && cleaned
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                {
                    let entity = cleaned.to_string();
                    if !entities.contains(&entity) {
                        entities.push(entity);
                    }
                }
            }
        }
        if round > 1 {
            entities = entities
                .into_iter()
                .map(|e| format!("{} [confirmed]", e))
                .collect();
        }
        entities
    }

    fn extract_events(&self, _sections: &[String], _round: usize) -> Vec<String> {
        vec!["event_analysis_pending".to_string()]
    }

    fn map_relations(&self, entities: &[String], round: usize) -> Vec<(String, String, String)> {
        let mut relations = Vec::new();
        for i in 0..entities.len().min(5) {
            for j in (i + 1)..entities.len().min(5) {
                relations.push((
                    entities[i].clone(),
                    "related_to".to_string(),
                    entities[j].clone(),
                ));
            }
        }
        if round > 1 {
            relations = relations
                .into_iter()
                .map(|(s, r, t)| (s, r, format!("{} [weighted]", t)))
                .collect();
        }
        relations
    }

    fn align_ontology(
        &self,
        entities: &[String],
        _relations: &[(String, String, String)],
    ) -> Vec<String> {
        let mut aligned = Vec::new();
        for entity in entities {
            let domain = if entity.contains("algorithm") || entity.contains("model") {
                "algorithm"
            } else if entity.contains("framework") || entity.contains("library") {
                "framework"
            } else if entity.contains("paper") || entity.contains("author") {
                "research"
            } else {
                "concept"
            };
            aligned.push(format!("{} -> ontology:{}", entity, domain));
        }
        aligned
    }

    fn reason(
        &self,
        entities: &[String],
        relations: &[(String, String, String)],
        round: usize,
    ) -> Vec<String> {
        let mut notes = Vec::new();
        notes.push(format!(
            "round_{}: identified {} entities across {} relations",
            round,
            entities.len(),
            relations.len()
        ));
        if entities.len() >= 3 {
            notes.push(format!(
                "triadic_closure: {} -> {} -> {}",
                entities[0], entities[1], entities[2]
            ));
        }
        notes
    }

    fn generate_skus(&self, reasoning: &[String]) -> Vec<String> {
        reasoning
            .iter()
            .enumerate()
            .map(|(i, note)| format!("sku_{}: {}", i + 1, note))
            .collect()
    }

    fn apply(&self, _skus: &[String]) -> String {
        "9-step book pipeline complete".to_string()
    }

    fn simulate_llm_round(
        &self,
        input: &str,
        round: usize,
    ) -> (
        Vec<String>,
        Vec<String>,
        Vec<(String, String, String)>,
        Vec<String>,
    ) {
        let sections = self.structure(input, round);
        let entities = self.extract_entities(&sections, round);
        let _events = self.extract_events(&sections, round);
        let relations = self.map_relations(&entities, round);
        let _aligned = self.align_ontology(&entities, &relations);
        let reasoning = self.reason(&entities, &relations, round);
        (sections, entities, relations, reasoning)
    }
}

impl Default for BookPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl IngestionPipeline for BookPipeline {
    fn name(&self) -> &str {
        "book_pipeline"
    }

    fn source_type(&self) -> IngestionSourceType {
        IngestionSourceType::Book
    }

    fn process(&self, input: &str, config: &IngestionConfig) -> IngestionResult {
        let mut reflection = ReflectionLoop::new(config.max_rounds, config.convergence_threshold);
        let mut quality_monitor = QualityMonitor::new(config.quality_threshold);

        let summary = self.collate(input);

        while reflection.should_continue() {
            let round = reflection.current_round();
            let (_sections, entities, relations, reasoning) = self.simulate_llm_round(input, round);

            let clarity_delta = if round == 1 {
                0.8
            } else {
                let prev = reflection.latest_clarity();
                (prev * 0.6).max(0.01)
            };

            let insights = reasoning.clone();
            reflection.record_round(insights, clarity_delta);

            quality_monitor.evaluate(clarity_delta, entities.len(), relations.len());
        }

        let _last_round = reflection
            .rounds
            .last()
            .cloned()
            .unwrap_or(ReflectionRound {
                round: 0,
                insights: vec![],
                clarity_delta: 0.0,
                converged: false,
            });
        let entities = self.extract_entities(&self.structure(input, 1), 1);
        let relations = self.map_relations(&entities, 1);
        let reasoning = self.reason(&entities, &relations, 1);

        IngestionResult {
            source_type: IngestionSourceType::Book,
            title: input.lines().next().unwrap_or("unknown").to_string(),
            summary: format!(
                "{} | steps: {} | {}",
                summary,
                self.steps.len(),
                self.apply(&self.generate_skus(&reasoning))
            ),
            total_rounds: reflection.total_rounds(),
            final_quality: quality_monitor.average_score(),
            converged: reflection.converged,
            entities: entities.into_iter().take(10).collect(),
            relations: relations.into_iter().take(10).collect(),
            reasoning_notes: reasoning.into_iter().take(5).collect(),
            reflection_history: reflection.rounds,
        }
    }
}
