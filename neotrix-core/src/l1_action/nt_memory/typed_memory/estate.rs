#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Memory estates — typed semantic memory categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MemoryEstate {
    Procedural,
    Episodic,
    Semantic,
    Working,
    Reflexive,
}

impl MemoryEstate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Procedural => "procedural",
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Working => "working",
            Self::Reflexive => "reflexive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "procedural" => Self::Procedural,
            "episodic" => Self::Episodic,
            "semantic" => Self::Semantic,
            "working" => Self::Working,
            "reflexive" => Self::Reflexive,
            _ => Self::Semantic,
        }
    }

    pub fn all_variants() -> &'static [Self] {
        &[Self::Procedural, Self::Episodic, Self::Semantic, Self::Working, Self::Reflexive]
    }
}

impl Default for MemoryEstate {
    fn default() -> Self { Self::Semantic }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_estate_as_str() {
        assert_eq!(MemoryEstate::Procedural.as_str(), "procedural");
        assert_eq!(MemoryEstate::Semantic.as_str(), "semantic");
    }
    #[test] fn test_estate_from_str() {
        assert_eq!(MemoryEstate::from_str("procedural"), MemoryEstate::Procedural);
        assert_eq!(MemoryEstate::from_str("SEMANTIC"), MemoryEstate::Semantic);
    }
    #[test] fn test_all_variants() { assert_eq!(MemoryEstate::all_variants().len(), 5); }
}
