use std::collections::HashMap;

use crate::core::CapabilityVector;
use crate::core::edit::MicroEdit;
use crate::core::nt_core_hcube::axis::DimensionAxis;
use crate::core::nt_core_hcube::coord::HyperCoord;
use crate::core::knowledge::{TaskType, RewardSource};
use crate::core::nt_core_bank::{ReasoningMemory, T3Views, MemoryTier, MemoryLifecycle};
use crate::ReasoningBank;

use super::classifier::ClassifiedContent;
use super::config::{CrawlFormat, CrawlTopic};

pub struct MappedKnowledge {
    pub url: String,
    pub title: String,
    pub topic: CrawlTopic,
    pub format: CrawlFormat,
    pub confidence: f64,
    pub edits: Vec<(String, f64)>,
    pub insights: Vec<String>,
}

pub struct KnowledgeMapper {
    mapped_count: u64,
    total_edits: u64,
    topic_edit_map: HashMap<CrawlTopic, Vec<(String, f64)>>,
}

impl Default for KnowledgeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeMapper {
    pub fn new() -> Self {
        let mut topic_edit_map: HashMap<CrawlTopic, Vec<(String, f64)>> = HashMap::new();

        topic_edit_map.insert(CrawlTopic::LawAndGovernance, vec![
            ("domain_specificity".into(), 0.15),
            ("inference_depth".into(), 0.12),
            ("analysis".into(), 0.10),
            ("quality_gates".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::PolicyAndRegulation, vec![
            ("domain_specificity".into(), 0.12),
            ("analysis".into(), 0.10),
            ("synthesis".into(), 0.08),
            ("inference_depth".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::ScienceAndTechnology, vec![
            ("inference_depth".into(), 0.15),
            ("analysis".into(), 0.12),
            ("domain_specificity".into(), 0.10),
            ("experimental".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::HumanitiesAndCulture, vec![
            ("creativity".into(), 0.12),
            ("synthesis".into(), 0.10),
            ("domain_specificity".into(), 0.08),
            ("compound_composition".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::SocietyAndEconomics, vec![
            ("analysis".into(), 0.15),
            ("inference_depth".into(), 0.10),
            ("synthesis".into(), 0.10),
            ("domain_specificity".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::HealthAndMedicine, vec![
            ("domain_specificity".into(), 0.15),
            ("analysis".into(), 0.12),
            ("quality_gates".into(), 0.10),
            ("inference_depth".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::EducationAndAcademia, vec![
            ("synthesis".into(), 0.12),
            ("analysis".into(), 0.10),
            ("inference_depth".into(), 0.08),
            ("domain_specificity".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::NewsAndMedia, vec![
            ("analysis".into(), 0.08),
            ("synthesis".into(), 0.06),
            ("creativity".into(), 0.04),
        ]);

        topic_edit_map.insert(CrawlTopic::PhilosophyAndEthics, vec![
            ("inference_depth".into(), 0.18),
            ("analysis".into(), 0.12),
            ("synthesis".into(), 0.10),
        ]);

        topic_edit_map.insert(CrawlTopic::HistoryAndArcheology, vec![
            ("domain_specificity".into(), 0.12),
            ("synthesis".into(), 0.10),
            ("inference_depth".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::ArtsAndLiterature, vec![
            ("creativity".into(), 0.15),
            ("compound_composition".into(), 0.10),
            ("synthesis".into(), 0.08),
        ]);

        topic_edit_map.insert(CrawlTopic::General, vec![
            ("synthesis".into(), 0.04),
            ("analysis".into(), 0.04),
        ]);

        KnowledgeMapper {
            mapped_count: 0,
            total_edits: 0,
            topic_edit_map,
        }
    }

    pub fn map(&mut self, classified: &ClassifiedContent) -> MappedKnowledge {
        self.mapped_count += 1;

        let edits = self.topic_edit_map.get(&classified.topic)
            .cloned()
            .unwrap_or_default();

        let num_edits = edits.len();
        self.total_edits += num_edits as u64;

        let insights = vec![
            format!("{} content from {}", classified.topic.name(), classified.url),
            format!("format: {}, confidence: {:.2}", classified.format.name(), classified.confidence),
        ];

        MappedKnowledge {
            url: classified.url.clone(),
            title: classified.title.clone(),
            topic: classified.topic,
            format: classified.format,
            confidence: classified.confidence,
            edits,
            insights,
        }
    }

    pub fn apply_to_brain(
        &mut self,
        mapped: &MappedKnowledge,
        brain: &mut crate::neotrix::nt_mind::ReasoningBrain,
        bank: &mut ReasoningBank,
    ) {
        if mapped.confidence < 0.3 {
            return;
        }

        let source_name = format!("crawl:{}", mapped.topic.name());
        let edits = &mapped.edits;

        let mut vector = CapabilityVector::default();
        for (dim_name, delta) in edits {
            if let Some(idx) = Self::dim_name_to_index(dim_name) {
                vector.arr_mut()[idx] = (*delta).min(1.0);
            }
        }

        brain.register_knowledge_source(&source_name, vector);

        for (dim_name, delta) in edits {
            if let Some(idx) = Self::dim_name_to_index(dim_name) {
                let current = brain.capability.arr_mut()[idx];
                brain.capability.arr_mut()[idx] = (current + delta).clamp(0.0, 1.0);
            }
        }

        brain.capability.normalize();

        let micro_edits: Vec<MicroEdit> = edits.iter()
            .map(|(dim, delta)| MicroEdit::AdjustDimension(dim.clone(), *delta))
            .collect();

        let mem = ReasoningMemory {
            id: format!("crawl-{}-{}", self.mapped_count, mapped.url.len()),
            task_description: format!("Crawl {} - {}", mapped.topic.name(), mapped.title),
            task_type: TaskType::Research,
            micro_edits,
            reward: mapped.confidence,
            reward_source: RewardSource::Internal,
            success: mapped.confidence > 0.5,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("result")
                .as_secs() as i64,
            embedding: None,
            tier: MemoryTier::Working,
            lifecycle: MemoryLifecycle::new(mapped.confidence),
            t3_views: T3Views::new(),
        };

        bank.store(mem);
    }

    pub fn absorb_to_hypercube(
        _mapped: &MappedKnowledge,
        topic: CrawlTopic,
        confidence: f64,
    ) -> HyperCoord {
        let (domain, abstraction) = match topic {
            CrawlTopic::LawAndGovernance => (0.9, 0.7),
            CrawlTopic::PolicyAndRegulation => (0.8, 0.6),
            CrawlTopic::PhilosophyAndEthics => (0.3, 0.95),
            CrawlTopic::ScienceAndTechnology => (0.7, 0.8),
            CrawlTopic::HealthAndMedicine => (0.6, 0.5),
            CrawlTopic::EducationAndAcademia => (0.5, 0.75),
            CrawlTopic::SocietyAndEconomics => (0.4, 0.65),
            CrawlTopic::NewsAndMedia => (0.2, 0.3),
            CrawlTopic::HistoryAndArcheology => (0.35, 0.7),
            CrawlTopic::HumanitiesAndCulture => (0.25, 0.6),
            CrawlTopic::ArtsAndLiterature => (0.15, 0.55),
            CrawlTopic::General => (0.1, 0.5),
        };
        let mut coord = HyperCoord::new();
        coord.set(DimensionAxis::Domain, domain);
        coord.set(DimensionAxis::Abstraction, abstraction);
        coord.set(DimensionAxis::Certainty, confidence);
        coord
    }

    fn dim_name_to_index(name: &str) -> Option<usize> {
        let dims = [
            "typography", "grid", "color", "whitespace",
            "data_viz", "emotion", "minimalism", "experimental",
            "inference_depth", "creativity", "analysis", "synthesis",
            "domain_specificity", "accessibility", "compound_composition",
            "tailwind_proficiency", "react_aria_usage", "bem_naming",
            "figma_integration", "ai_native_states", "semantic_layer",
            "quality_gates", "verification",
        ];
        dims.iter().position(|d| *d == name)
    }

    pub fn summary(&self) -> MapperSummary {
        MapperSummary {
            total_mapped: self.mapped_count,
            total_edits: self.total_edits,
        }
    }
}

pub struct MapperSummary {
    pub total_mapped: u64,
    pub total_edits: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_world_crawl::classifier::ContentClassifier;

    #[test]
    fn test_map_legal_content() {
        let mut classifier = ContentClassifier::new();
        let mut mapper = KnowledgeMapper::new();

        let classified = classifier.classify("https://example.com/law", "statute law constitution amendment rights act");
        let mapped = mapper.map(&classified);

        assert_eq!(mapped.topic, CrawlTopic::LawAndGovernance);
        assert!(!mapped.edits.is_empty());
    }

    #[test]
    fn test_map_philosophy_content() {
        let mut classifier = ContentClassifier::new();
        let mut mapper = KnowledgeMapper::new();

        let classified = classifier.classify("https://plato.stanford.edu/ethics", "philosophy ethics moral reasoning logic virtue");
        let mapped = mapper.map(&classified);

        assert_eq!(mapped.topic, CrawlTopic::PhilosophyAndEthics);
    }

    #[test]
    fn test_low_confidence_skipped() {
        let mut mapper = KnowledgeMapper::new();
        let classified = ClassifiedContent {
            url: "https://example.com".into(),
            title: "test".into(),
            topic: CrawlTopic::General,
            format: CrawlFormat::Other,
            confidence: 0.2,
            summary: "text".into(),
            keywords: vec![],
            content_length: 4,
        };
        let mapped = mapper.map(&classified);
        assert_eq!(mapped.edits.len(), 2);
        assert_eq!(mapped.confidence, 0.2);
    }

    #[test]
    fn test_dim_name_to_index() {
        assert!(KnowledgeMapper::dim_name_to_index("inference_depth").is_some());
        assert!(KnowledgeMapper::dim_name_to_index("analysis").is_some());
        assert!(KnowledgeMapper::dim_name_to_index("nonexistent").is_none());
    }

    #[test]
    fn test_summary() {
        let mut mapper = KnowledgeMapper::new();
        let mut classifier = ContentClassifier::new();

        let classified = classifier.classify("https://example.com", "law constitution");
        let _mapped = mapper.map(&classified);
        let summary = mapper.summary();
        assert_eq!(summary.total_mapped, 1);
        assert!(summary.total_edits > 0);
    }
}
