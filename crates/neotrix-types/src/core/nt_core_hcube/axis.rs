use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionAxis {
    CodeUnderstanding,
    SystemDesign,
    Debugging,
    KnowledgeRetrieval,
    Creativity,
    Safety,
    Performance,
    Communication,
    Time,
    Domain,
    Abstraction,
    Culture,
    Scale,
    Certainty,
    Agency,
    Modality,
}

impl DimensionAxis {
    pub fn all() -> &'static [DimensionAxis] {
        &[
            DimensionAxis::CodeUnderstanding,
            DimensionAxis::SystemDesign,
            DimensionAxis::Debugging,
            DimensionAxis::KnowledgeRetrieval,
            DimensionAxis::Creativity,
            DimensionAxis::Safety,
            DimensionAxis::Performance,
            DimensionAxis::Communication,
            DimensionAxis::Time,
            DimensionAxis::Domain,
            DimensionAxis::Abstraction,
            DimensionAxis::Culture,
            DimensionAxis::Scale,
            DimensionAxis::Certainty,
            DimensionAxis::Agency,
            DimensionAxis::Modality,
        ]
    }

    pub fn id(&self) -> usize {
        *self as usize
    }

    pub fn count() -> usize {
        16
    }
}
