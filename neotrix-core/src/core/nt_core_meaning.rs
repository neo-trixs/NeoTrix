//! 意义建构器 — 最小可用版本。
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MeaningPattern {
    ConceptCluster(Vec<String>),
    CausalChain(String, String),
    ValueConflict(String, String),
}

impl std::fmt::Display for MeaningPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MeaningPattern::ConceptCluster(kw) => write!(f, "[{}]", kw.join(",")),
            MeaningPattern::CausalChain(c, e) => write!(f, "{} -> {}", c, e),
            MeaningPattern::ValueConflict(a, b) => write!(f, "{} vs {}", a, b),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeaningUnit {
    pub id: String,
    pub pattern: MeaningPattern,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MeaningContext {
    pub tags: Vec<String>,
    pub domain: Option<String>,
}

pub struct MeaningConstructor { units: Vec<MeaningUnit> }

impl Default for MeaningConstructor { fn default() -> Self { Self::new() } }

impl MeaningConstructor {
    pub fn new() -> Self { Self { units: Vec::new() } }

    pub fn detect_causal_chain(&self, text: &str) -> Option<(String, String)> {
        for m in ["因为", "导致", "所以"] {
            if let Some(i) = text.find(m) {
                let c = text[..i].trim().to_string();
                let e = text[i + m.len()..].trim().to_string();
                if !c.is_empty() && !e.is_empty() { return Some((c, e)); }
            }
        }
        None
    }

    pub fn extract_meaning(&mut self, obs: &str, _ctx: &MeaningContext) -> Result<Vec<MeaningUnit>, String> {
        let mut u = Vec::new();
        if let Some((c, e)) = self.detect_causal_chain(obs) {
            u.push(MeaningUnit { id: format!("mu_{}", self.units.len()), pattern: MeaningPattern::CausalChain(c, e), confidence: 0.8 });
        }
        let kw: Vec<String> = obs.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.chars().count() > 1)
            .map(|s| s.to_lowercase())
            .collect::<HashSet<_>>().into_iter().collect();
        if kw.len() >= 2 {
            u.push(MeaningUnit { id: format!("mu_{}_kw", self.units.len()), pattern: MeaningPattern::ConceptCluster(kw), confidence: 0.6 });
        }
        self.units.extend(u.clone());
        Ok(u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_causal() {
        let mc = MeaningConstructor::new();
        assert!(mc.detect_causal_chain("因为下雨所以地湿").is_some());
    }
    #[test]
    fn test_extract() {
        let mut mc = MeaningConstructor::new();
        assert!(!mc.extract_meaning("因为下雨所以地湿", &MeaningContext::default()).unwrap().is_empty());
    }
}
