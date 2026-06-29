use crate::core::nt_core_knowledge::evidence_inspector::{
    Claim, EvidenceVerificationResult, VerificationStatus,
};
use std::collections::HashMap;

/// Audience type for story angle selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    General,
    Technical,
    Executive,
    Research,
    Beginner,
    Expert,
}

/// Story angle / perspective
#[derive(Debug, Clone)]
pub struct Angle {
    pub name: String,
    pub description: String,
    pub audience: Audience,
    pub focus_points: Vec<String>,
}

/// Data set for storytelling
#[derive(Debug, Clone)]
pub struct DataSet {
    pub claims: Vec<Claim>,
    pub verifications: Vec<EvidenceVerificationResult>,
    pub metadata: HashMap<String, String>,
}

/// Modality for story output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    Text,
    InteractiveChart,
    Map,
    Audio,
    Video,
    Timeline,
    NetworkGraph,
}

/// Story plan: angle + modalities
#[derive(Debug, Clone)]
pub struct StoryPlan {
    pub angle: Angle,
    pub modalities: Vec<Modality>,
    pub sections: Vec<StorySection>,
}

#[derive(Debug, Clone)]
pub struct StorySection {
    pub title: String,
    pub modality: Modality,
    pub content: String,
    pub evidence_refs: Vec<u64>,
}

/// Story rendering options
#[derive(Debug, Clone)]
pub struct Story {
    pub title: String,
    pub plan: StoryPlan,
    pub sections: Vec<StorySection>,
    pub summary: String,
    pub quality_score: f64,
}

/// Angle selector: chooses story angle based on audience and data
#[derive(Debug, Clone)]
pub struct AngleSelector;

impl AngleSelector {
    pub fn new() -> Self {
        Self
    }

    /// Select angle based on audience and data
    pub fn select_angle(&self, data: &DataSet, audience: Audience) -> Angle {
        let verified_count = data
            .verifications
            .iter()
            .filter(|v| v.status == VerificationStatus::Verified)
            .count();
        let total = data.claims.len().max(1);
        let _confidence_ratio = verified_count as f64 / total as f64;

        let (name, description, focus) = match audience {
            Audience::General => (
                "Main Narrative".into(),
                "Accessible overview of key findings with visual highlights".into(),
                vec![
                    "key insights".into(),
                    "real-world impact".into(),
                    "visual summary".into(),
                ],
            ),
            Audience::Technical => (
                "Technical Deep-Dive".into(),
                "Detailed analysis with methodology and evidence chains".into(),
                vec![
                    "methodology".into(),
                    "data provenance".into(),
                    "reproducibility".into(),
                ],
            ),
            Audience::Executive => (
                "Executive Brief".into(),
                "Strategic implications and actionable recommendations".into(),
                vec![
                    "executive summary".into(),
                    "risk assessment".into(),
                    "recommendations".into(),
                ],
            ),
            Audience::Research => (
                "Research Report".into(),
                "Comprehensive findings with full evidence traceability".into(),
                vec![
                    "literature context".into(),
                    "experimental results".into(),
                    "open questions".into(),
                ],
            ),
            Audience::Beginner => (
                "Getting Started Guide".into(),
                "Gentle introduction with step-by-step walkthrough".into(),
                vec![
                    "basic concepts".into(),
                    "tutorial".into(),
                    "glossary".into(),
                ],
            ),
            Audience::Expert => (
                "Advanced Analysis".into(),
                "Specialized insights with quantitative benchmarks".into(),
                vec![
                    "quantitative results".into(),
                    "comparative analysis".into(),
                    "limitations".into(),
                ],
            ),
        };
        Angle {
            name,
            description,
            audience,
            focus_points: focus,
        }
    }
}

/// Modality planner: selects output modalities based on data and angle
#[derive(Debug, Clone)]
pub struct ModalityPlanner;

impl ModalityPlanner {
    pub fn new() -> Self {
        Self
    }

    /// Plan which modalities to use based on angle and data characteristics
    pub fn plan_modalities(&self, angle: &Angle, data: &DataSet) -> Vec<Modality> {
        let mut modalities = vec![Modality::Text];
        let has_numerical = data
            .claims
            .iter()
            .any(|c| c.text.chars().any(|ch| ch.is_ascii_digit()));
        let has_geospatial =
            data.metadata.contains_key("location") || data.metadata.contains_key("coordinates");
        let has_temporal =
            data.metadata.contains_key("timestamp") || data.metadata.contains_key("date_range");
        let verification_rate = if data.claims.is_empty() {
            0.0
        } else {
            data.verifications
                .iter()
                .filter(|v| v.status == VerificationStatus::Verified)
                .count() as f64
                / data.claims.len() as f64
        };

        if has_numerical || matches!(angle.audience, Audience::Executive | Audience::Technical) {
            modalities.push(Modality::InteractiveChart);
        }
        if has_geospatial || angle.focus_points.iter().any(|f| f.contains("map")) {
            modalities.push(Modality::Map);
        }
        if has_temporal {
            modalities.push(Modality::Timeline);
        }
        if verification_rate > 0.5 {
            modalities.push(Modality::Audio);
        }
        if angle
            .focus_points
            .iter()
            .any(|f| f.contains("network") || f.contains("relation"))
        {
            modalities.push(Modality::NetworkGraph);
        }
        modalities
    }
}

/// Story renderer: produces final story output
#[derive(Debug, Clone)]
pub struct StoryRenderer;

impl StoryRenderer {
    pub fn new() -> Self {
        Self
    }

    /// Render a story from plan, data, and modalities
    pub fn render(&self, data: &DataSet, plan: &StoryPlan) -> Story {
        let mut sections = Vec::new();
        let mut all_evidence: Vec<u64> = Vec::new();
        let title = format!("{} — {}", plan.angle.name, plan.angle.description);

        for (i, &modality) in plan.modalities.iter().enumerate() {
            let (section_title, content) = match modality {
                Modality::Text => {
                    let verified = data
                        .verifications
                        .iter()
                        .filter(|v| v.status == VerificationStatus::Verified)
                        .count();
                    let total = data.claims.len();
                    (format!("Text Summary"), format!("Analysis of {} claims ({} verified). Key findings from {} evidence items.", total, verified, data.verifications.len()))
                }
                Modality::InteractiveChart => {
                    let values: Vec<String> =
                        data.claims.iter().take(5).map(|c| c.text.clone()).collect();
                    (
                        format!("Interactive Chart"),
                        format!("Chart showing: {}", values.join(" | ")),
                    )
                }
                Modality::Map => {
                    let loc = data.metadata.get("location").cloned().unwrap_or_default();
                    (format!("Geospatial Map"), format!("Location: {}", loc))
                }
                Modality::Audio => (
                    format!("Audio Narrative"),
                    format!("Audio summary of {}", plan.angle.name),
                ),
                Modality::Timeline => {
                    let dates = data.metadata.get("date_range").cloned().unwrap_or_default();
                    (format!("Timeline"), format!("Time period: {}", dates))
                }
                Modality::NetworkGraph => (
                    format!("Network Graph"),
                    format!(
                        "Relationships between {} evidence nodes",
                        data.verifications.len()
                    ),
                ),
                Modality::Video => (
                    format!("Video"),
                    format!("Animated visualization of {}", plan.angle.name),
                ),
            };
            let refs: Vec<u64> = data
                .verifications
                .iter()
                .skip(i * 3)
                .take(3)
                .map(|v| v.claim_id)
                .collect();
            all_evidence.extend(&refs);
            sections.push(StorySection {
                title: section_title,
                modality,
                content,
                evidence_refs: refs,
            });
        }

        let verified_count = data
            .verifications
            .iter()
            .filter(|v| v.status == VerificationStatus::Verified)
            .count();
        let summary = format!(
            "Story with {} sections, {} claims ({} verified), {} modalities",
            sections.len(),
            data.claims.len(),
            verified_count,
            plan.modalities.len()
        );
        let quality = self.evaluate_quality(&sections);

        Story {
            title,
            plan: plan.clone(),
            sections,
            summary,
            quality_score: quality,
        }
    }

    fn evaluate_quality(&self, sections: &[StorySection]) -> f64 {
        if sections.is_empty() {
            return 0.0;
        }
        let modality_diversity = sections
            .iter()
            .map(|s| s.modality as u8)
            .collect::<std::collections::HashSet<_>>()
            .len() as f64
            / sections.len() as f64;
        let has_content = sections.iter().filter(|s| !s.content.is_empty()).count() as f64
            / sections.len() as f64;
        let has_evidence = sections
            .iter()
            .filter(|s| !s.evidence_refs.is_empty())
            .count() as f64
            / sections.len() as f64;
        (modality_diversity * 0.3 + has_content * 0.4 + has_evidence * 0.3).clamp(0.0, 1.0)
    }
}

/// Full multimodal storyteller pipeline
#[derive(Debug, Clone)]
pub struct MultimodalStoryteller {
    pub angle_selector: AngleSelector,
    pub modality_planner: ModalityPlanner,
    pub renderer: StoryRenderer,
}

impl MultimodalStoryteller {
    pub fn new() -> Self {
        Self {
            angle_selector: AngleSelector::new(),
            modality_planner: ModalityPlanner::new(),
            renderer: StoryRenderer::new(),
        }
    }

    /// Full pipeline: data + audience → story
    pub fn tell_story(&self, data: &DataSet, audience: Audience) -> Story {
        let angle = self.angle_selector.select_angle(data, audience);
        let modalities = self.modality_planner.plan_modalities(&angle, data);
        let plan = StoryPlan {
            angle,
            modalities,
            sections: Vec::new(),
        };
        self.renderer.render(data, &plan)
    }

    /// Generate a story from verified evidence only
    pub fn tell_verified_story(&self, data: &DataSet, audience: Audience) -> Story {
        let verified_claims: Vec<Claim> = data
            .claims
            .iter()
            .filter(|c| {
                data.verifications
                    .iter()
                    .any(|v| v.claim_id == c.id && v.status == VerificationStatus::Verified)
            })
            .cloned()
            .collect();
        let filtered_data = DataSet {
            claims: verified_claims,
            verifications: data.verifications.clone(),
            metadata: data.metadata.clone(),
        };
        self.tell_story(&filtered_data, audience)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_dataset() -> DataSet {
        let claims = vec![
            Claim {
                id: 1,
                text: "NeoTrix achieves 98% accuracy on benchmark".into(),
                evidence_refs: vec![101],
                verification_status: VerificationStatus::Verified,
                confidence: 0.95,
            },
            Claim {
                id: 2,
                text: "System processes 1000 queries/second".into(),
                evidence_refs: vec![102],
                verification_status: VerificationStatus::Verified,
                confidence: 0.88,
            },
            Claim {
                id: 3,
                text: "Memory footprint is 2.3 GB under load".into(),
                evidence_refs: vec![103],
                verification_status: VerificationStatus::Failed,
                confidence: 0.45,
            },
        ];
        let verifications = vec![
            EvidenceVerificationResult {
                claim_id: 1,
                status: VerificationStatus::Verified,
                matched_evidence: vec![101],
                confidence: 0.95,
                details: "passed".into(),
            },
            EvidenceVerificationResult {
                claim_id: 2,
                status: VerificationStatus::Verified,
                matched_evidence: vec![102],
                confidence: 0.88,
                details: "passed".into(),
            },
            EvidenceVerificationResult {
                claim_id: 3,
                status: VerificationStatus::Failed,
                matched_evidence: vec![],
                confidence: 0.0,
                details: "failed".into(),
            },
        ];
        let mut metadata = HashMap::new();
        metadata.insert("location".into(), "San Francisco".into());
        metadata.insert("date_range".into(), "2026-01 to 2026-06".into());
        DataSet {
            claims,
            verifications,
            metadata,
        }
    }

    #[test]
    fn test_angle_selector_general() {
        let selector = AngleSelector::new();
        let data = make_test_dataset();
        let angle = selector.select_angle(&data, Audience::General);
        assert_eq!(angle.name, "Main Narrative");
    }

    #[test]
    fn test_angle_selector_technical() {
        let selector = AngleSelector::new();
        let data = make_test_dataset();
        let angle = selector.select_angle(&data, Audience::Technical);
        assert_eq!(angle.name, "Technical Deep-Dive");
    }

    #[test]
    fn test_modality_planner_has_numerical() {
        let planner = ModalityPlanner::new();
        let data = make_test_dataset();
        let angle = Angle {
            name: "test".into(),
            description: "desc".into(),
            audience: Audience::General,
            focus_points: vec![],
        };
        let modalities = planner.plan_modalities(&angle, &data);
        assert!(modalities.contains(&Modality::InteractiveChart));
    }

    #[test]
    fn test_modality_planner_has_map() {
        let planner = ModalityPlanner::new();
        let data = make_test_dataset();
        let angle = Angle {
            name: "test".into(),
            description: "desc".into(),
            audience: Audience::General,
            focus_points: vec!["map".into()],
        };
        let modalities = planner.plan_modalities(&angle, &data);
        assert!(modalities.contains(&Modality::Map));
    }

    #[test]
    fn test_story_renderer_sections() {
        let renderer = StoryRenderer::new();
        let data = make_test_dataset();
        let angle = Angle {
            name: "Test".into(),
            description: "Description".into(),
            audience: Audience::General,
            focus_points: vec![],
        };
        let modalities = vec![Modality::Text, Modality::InteractiveChart];
        let plan = StoryPlan {
            angle,
            modalities,
            sections: Vec::new(),
        };
        let story = renderer.render(&data, &plan);
        assert_eq!(story.sections.len(), 2);
        assert!(story.quality_score > 0.0);
    }

    #[test]
    fn test_multimodal_storyteller_pipeline() {
        let storyteller = MultimodalStoryteller::new();
        let data = make_test_dataset();
        let story = storyteller.tell_story(&data, Audience::Executive);
        assert!(story.title.contains("Executive Brief"));
        assert!(!story.sections.is_empty());
        assert!(!story.summary.is_empty());
    }

    #[test]
    fn test_tell_verified_story() {
        let storyteller = MultimodalStoryteller::new();
        let data = make_test_dataset();
        let story = storyteller.tell_verified_story(&data, Audience::Research);
        assert!(!story.sections.is_empty());
    }

    #[test]
    fn test_story_quality_score_range() {
        let renderer = StoryRenderer::new();
        let data = make_test_dataset();
        let angle = Angle {
            name: "".into(),
            description: "".into(),
            audience: Audience::General,
            focus_points: vec![],
        };
        let plan = StoryPlan {
            angle,
            modalities: vec![Modality::Text],
            sections: Vec::new(),
        };
        let story = renderer.render(&data, &plan);
        assert!(story.quality_score >= 0.0 && story.quality_score <= 1.0);
    }

    #[test]
    fn test_angle_focus_points_by_audience() {
        let selector = AngleSelector::new();
        let data = make_test_dataset();
        let exec = selector.select_angle(&data, Audience::Executive);
        assert!(exec
            .focus_points
            .iter()
            .any(|f| f.contains("recommendation")));
    }
}
