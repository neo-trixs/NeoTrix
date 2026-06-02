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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_axis_has_16_variants() {
        assert_eq!(DimensionAxis::all().len(), 16);
    }

    #[test]
    fn test_dimension_axis_id_unique() {
        let ids: Vec<usize> = DimensionAxis::all().iter().map(|a| a.id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len());
    }

    #[test]
    fn test_dimension_axis_id_ranges_0_to_15() {
        for a in DimensionAxis::all() {
            let id = a.id();
            assert!(id < 16, "id {} out of range", id);
        }
    }

    #[test]
    fn test_dimension_axis_count_matches_all_len() {
        assert_eq!(DimensionAxis::count(), DimensionAxis::all().len());
    }

    #[test]
    fn test_code_understanding_is_first() {
        assert_eq!(DimensionAxis::all()[0], DimensionAxis::CodeUnderstanding);
    }

    #[test]
    fn test_modality_is_last() {
        assert_eq!(DimensionAxis::all()[15], DimensionAxis::Modality);
    }
}
