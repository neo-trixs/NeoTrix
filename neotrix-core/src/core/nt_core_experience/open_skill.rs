use crate::core::nt_core_hcube::vsa_quantized::{QuantizedVSA, VSA_DIM};
use crate::core::nt_core_self::attention_head::AttentionDomain;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct KnowledgeAnchor {
    pub id: u64,
    pub topic: String,
    pub encoded: Vec<u8>,
    pub source: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct SkillBlueprint {
    pub id: u64,
    pub name: String,
    pub domain: AttentionDomain,
    pub trigger: Vec<u8>,
    pub action: Vec<u8>,
    pub outcome: Vec<u8>,
    pub anchor_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct VirtualTask {
    pub id: u64,
    pub description: String,
    pub success_criteria: Vec<u8>,
    pub difficulty: f64,
    pub verifier_id: u64,
}

pub struct SelfBuiltVerifier {
    pub id: u64,
    pub anchor_vectors: Vec<Vec<u8>>,
    pub consolidated: Vec<u8>,
    pub confidence: f64,
    pub pass_count: u64,
    pub fail_count: u64,
}

impl SelfBuiltVerifier {
    pub fn new(id: u64, anchors: &[KnowledgeAnchor]) -> Self {
        let anchor_vectors: Vec<Vec<u8>> = anchors.iter().map(|a| a.encoded.clone()).collect();
        let slices: Vec<&[u8]> = anchor_vectors.iter().map(|v| v.as_slice()).collect();
        let consolidated = if slices.is_empty() {
            QuantizedVSA::seeded_random(id, VSA_DIM)
        } else {
            QuantizedVSA::bundle(&slices)
        };
        Self {
            id,
            anchor_vectors,
            consolidated,
            confidence: anchors.iter().map(|a| a.confidence).sum::<f64>()
                / anchors.len().max(1) as f64,
            pass_count: 0,
            fail_count: 0,
        }
    }

    pub fn verify(&mut self, result: &[u8], threshold: f64) -> bool {
        let sim = QuantizedVSA::similarity(result, &self.consolidated);
        if sim >= threshold {
            self.pass_count += 1;
            true
        } else {
            self.fail_count += 1;
            false
        }
    }

    pub fn stats(&self) -> VerifierStats {
        let total = self.pass_count + self.fail_count;
        VerifierStats {
            id: self.id,
            anchor_count: self.anchor_vectors.len(),
            confidence: self.confidence,
            pass_rate: if total > 0 {
                self.pass_count as f64 / total as f64
            } else {
                0.0
            },
            total_tests: total,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct VerifierStats {
    pub id: u64,
    pub anchor_count: usize,
    pub confidence: f64,
    pub pass_rate: f64,
    pub total_tests: u64,
}

pub struct OpenSkillEngine {
    pub anchors: Vec<KnowledgeAnchor>,
    pub blueprints: Vec<SkillBlueprint>,
    pub verifiers: Vec<SelfBuiltVerifier>,
    pub virtual_tasks: Vec<VirtualTask>,
    next_id: u64,
}

impl OpenSkillEngine {
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
            blueprints: Vec::new(),
            verifiers: Vec::new(),
            virtual_tasks: Vec::new(),
            next_id: 0,
        }
    }

    pub fn acquire(&mut self, topic: &str, source: &str, texts: &[&str]) -> Vec<u64> {
        let mut ids = Vec::new();
        for _text in texts {
            let encoded = QuantizedVSA::seeded_random(
                self.next_id.wrapping_mul(17).wrapping_add(31),
                VSA_DIM,
            );
            let id = self.next_id;
            self.next_id += 1;
            self.anchors.push(KnowledgeAnchor {
                id,
                topic: topic.to_string(),
                encoded,
                source: source.to_string(),
                confidence: 0.7,
            });
            ids.push(id);
        }
        ids
    }

    pub fn bootstrap(
        &mut self,
        name: &str,
        domain: AttentionDomain,
        anchor_ids: &[u64],
    ) -> Option<SkillBlueprint> {
        let related: Vec<&KnowledgeAnchor> = self
            .anchors
            .iter()
            .filter(|a| anchor_ids.contains(&a.id))
            .collect();
        if related.is_empty() {
            return None;
        }
        let slices: Vec<&[u8]> = related.iter().map(|a| a.encoded.as_slice()).collect();
        let trigger = QuantizedVSA::bundle(&slices);
        let action = QuantizedVSA::seeded_random(self.next_id, VSA_DIM);
        let outcome = QuantizedVSA::xor_bind(&trigger, &action);
        self.next_id += 1;

        let id = self.next_id;
        self.next_id += 1;
        let bp = SkillBlueprint {
            id,
            name: name.to_string(),
            domain,
            trigger,
            action,
            outcome,
            anchor_ids: anchor_ids.to_vec(),
        };
        self.blueprints.push(bp.clone());
        Some(bp)
    }

    pub fn build_verifier(&mut self, anchor_ids: &[u64]) -> Option<u64> {
        let selected: Vec<KnowledgeAnchor> = self
            .anchors
            .iter()
            .filter(|a| anchor_ids.contains(&a.id))
            .cloned()
            .collect();
        if selected.is_empty() {
            return None;
        }
        let id = self.next_id;
        self.next_id += 1;
        let verifier = SelfBuiltVerifier::new(id, &selected);
        self.verifiers.push(verifier);
        Some(id)
    }

    pub fn virtual_task(&mut self, verifier_id: u64, difficulty: f64) -> Option<VirtualTask> {
        let verifier = self.verifiers.iter().find(|v| v.id == verifier_id)?;
        let id = self.next_id;
        self.next_id += 1;
        let task = VirtualTask {
            id,
            description: format!("virtual_task_v{}_d{:.1}", id, difficulty),
            success_criteria: verifier.consolidated.clone(),
            difficulty,
            verifier_id,
        };
        self.virtual_tasks.push(task.clone());
        Some(task)
    }

    pub fn verify_skill(
        &mut self,
        _blueprint_id: u64,
        verifier_id: u64,
        result: &[u8],
    ) -> Option<bool> {
        let verifier = self.verifiers.iter_mut().find(|v| v.id == verifier_id)?;
        Some(verifier.verify(result, 0.65))
    }

    pub fn leakage_free(&self, anchor_ids: &[u64], target_task: &str) -> bool {
        let anchor_set: HashSet<u64> = anchor_ids.iter().copied().collect();
        !self
            .anchors
            .iter()
            .any(|a| anchor_set.contains(&a.id) && a.topic.contains(target_task))
    }

    pub fn stats(&self) -> OpenSkillStats {
        OpenSkillStats {
            anchors: self.anchors.len(),
            blueprints: self.blueprints.len(),
            verifiers: self.verifiers.len(),
            virtual_tasks: self.virtual_tasks.len(),
            avg_verifier_confidence: if self.verifiers.is_empty() {
                0.0
            } else {
                self.verifiers.iter().map(|v| v.confidence).sum::<f64>()
                    / self.verifiers.len() as f64
            },
        }
    }
}

impl Default for OpenSkillEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenSkillStats {
    pub anchors: usize,
    pub blueprints: usize,
    pub verifiers: usize,
    pub virtual_tasks: usize,
    pub avg_verifier_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_skill_new() {
        let e = OpenSkillEngine::new();
        assert_eq!(e.anchors.len(), 0);
        assert_eq!(e.next_id, 0);
    }

    #[test]
    fn test_acquire_knowledge() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("rust", "docs", &["pattern matching", "ownership", "traits"]);
        assert_eq!(ids.len(), 3);
        assert_eq!(e.anchors.len(), 3);
    }

    #[test]
    fn test_bootstrap_skill() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("rust", "docs", &["pattern matching"]);
        let bp = e.bootstrap("match_expr", AttentionDomain::Code, &ids);
        assert!(bp.is_some());
        assert_eq!(e.blueprints.len(), 1);
        assert_eq!(e.blueprints[0].name, "match_expr");
    }

    #[test]
    fn test_bootstrap_empty_anchors() {
        let mut e = OpenSkillEngine::new();
        let bp = e.bootstrap("test", AttentionDomain::Code, &[999]);
        assert!(bp.is_none());
    }

    #[test]
    fn test_build_verifier() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("rust", "docs", &["ownership", "borrowing"]);
        let vid = e.build_verifier(&ids);
        assert!(vid.is_some());
        assert_eq!(e.verifiers.len(), 1);
    }

    #[test]
    fn test_verifier_verify_pass() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("rust", "docs", &["ownership"]);
        let _vid = e.build_verifier(&ids).unwrap();
        let verifier = e.verifiers.get_mut(0).unwrap();
        let consolidated = verifier.consolidated.clone();
        assert!(verifier.verify(&consolidated, 0.65));
        assert_eq!(verifier.pass_count, 1);
    }

    #[test]
    fn test_virtual_task_generation() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("math", "docs", &["algebra", "geometry"]);
        let vid = e.build_verifier(&ids).unwrap();
        let task = e.virtual_task(vid, 0.5);
        assert!(task.is_some());
        assert!((task.unwrap().difficulty - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_leakage_barrier() {
        let mut e = OpenSkillEngine::new();
        let ids = e.acquire("rust", "docs", &["ownership"]);
        assert!(e.leakage_free(&ids, "python"));
        assert!(!e.leakage_free(&ids, "rust"));
    }

    #[test]
    fn test_stats() {
        let e = OpenSkillEngine::new();
        let s = e.stats();
        assert_eq!(s.anchors, 0);
        assert_eq!(s.blueprints, 0);
    }

    #[test]
    fn test_full_pipeline() {
        let mut e = OpenSkillEngine::new();
        let docs = &["error handling", "result type", "option type"];
        let anchors = e.acquire("rust_error", "docs", docs);
        let bp = e.bootstrap("rust_errors", AttentionDomain::Code, &anchors);
        assert!(bp.is_some());
        let vid = e.build_verifier(&anchors).unwrap();
        let task = e.virtual_task(vid, 0.3);
        assert!(task.is_some());
        let verifier = e.verifiers.get_mut(0).unwrap();
        let result = QuantizedVSA::seeded_random(42, VSA_DIM);
        verifier.verify(&result, 0.65);
        assert!(verifier.pass_count + verifier.fail_count >= 1);
    }
}
