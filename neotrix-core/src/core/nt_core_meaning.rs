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

/// 实体关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

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

    /// 实体关系抽取 (SVO 模式)
    pub fn detect_entity_relation(&self, text: &str) -> Option<EntityRelation> {
        for marker in ["保护", "尊重", "帮助", "攻击", "破坏", "使用"] {
            if let Some(idx) = text.find(marker) {
                let subject = text[..idx].trim().to_string();
                let obj_part = &text[idx + marker.len()..];
                if !subject.is_empty() && !obj_part.trim().is_empty() {
                    return Some(EntityRelation {
                        subject,
                        predicate: marker.to_string(),
                        object: obj_part.trim().to_string(),
                    });
                }
            }
        }
        None
    }

    /// 时序模式检测
    pub fn detect_temporal_sequence(&self, text: &str) -> Vec<String> {
        let markers = ["首先", "然后", "接着", "最后"];
        for m in markers {
            if text.contains(m) {
                let parts: Vec<String> = text.split(m)
                    .flat_map(|s| s.split("然后").map(|s| s.trim().to_string()).collect::<Vec<_>>())
                    .filter(|s| !s.is_empty())
                    .collect();
                if parts.len() >= 2 { return parts; }
            }
        }
        // 中文逗号切分作为回退
        text.split('，')
            .map(|s| s.trim().to_string())
            .filter(|s| s.chars().count() > 2)
            .collect()
    }

    /// 价值冲突检测
    pub fn detect_value_conflict(&self, text: &str) -> Option<(String, String)> {
        let pairs = [
            ("自由", "安全"), ("隐私", "便利"), ("公平", "效率"),
            ("自主", "保护"), ("真相", "和谐"),
        ];
        for (v1, v2) in pairs {
            if text.contains(v1) && text.contains(v2) {
                return Some((v1.into(), v2.into()));
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
        // 实体关系
        if let Some(er) = self.detect_entity_relation(obs) {
            u.push(MeaningUnit {
                id: format!("mu_{}_er", self.units.len()),
                pattern: MeaningPattern::ConceptCluster(vec![er.subject, er.predicate, er.object]),
                confidence: 0.7,
            });
        }
        // 价值冲突
        if let Some((v1, v2)) = self.detect_value_conflict(obs) {
            u.push(MeaningUnit {
                id: format!("mu_{}_vc", self.units.len()),
                pattern: MeaningPattern::ValueConflict(v1, v2),
                confidence: 0.85,
            });
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
