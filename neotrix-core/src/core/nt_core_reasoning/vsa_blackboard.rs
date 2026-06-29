use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::unix_now_ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpertType {
    Analogical,
    Causal,
    MultiHop,
    Contradiction,
    Synthesis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: u64,
    pub content: Vec<u8>,
    pub confidence: f64,
    pub expert: ExpertType,
    pub supporting_evidence: Vec<u64>,
    pub created_at: u64,
    pub is_contradicted: bool,
}

#[derive(Debug, Clone)]
pub struct VsaBlackboard {
    pub hypotheses: Vec<Hypothesis>,
    next_id: u64,
    max_hypotheses: usize,
}

impl VsaBlackboard {
    pub fn new(max: usize) -> Self {
        Self {
            hypotheses: Vec::with_capacity(max),
            next_id: 1,
            max_hypotheses: max,
        }
    }

    pub fn post_hypothesis(
        &mut self,
        content: Vec<u8>,
        confidence: f64,
        expert: ExpertType,
        evidence: Vec<u64>,
    ) -> u64 {
        if self.hypotheses.len() >= self.max_hypotheses {
            return 0;
        }
        let id = self.next_id;
        self.next_id += 1;
        let hypothesis = Hypothesis {
            id,
            content,
            confidence: confidence.clamp(0.0, 1.0),
            expert,
            supporting_evidence: evidence,
            created_at: unix_now_ms(),
            is_contradicted: false,
        };
        self.hypotheses.push(hypothesis);
        id
    }

    pub fn get_hypothesis(&self, id: u64) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    pub fn get_by_expert(&self, expert: ExpertType) -> Vec<&Hypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| h.expert == expert)
            .collect()
    }

    pub fn get_contradicted(&self) -> Vec<&Hypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| h.is_contradicted)
            .collect()
    }

    pub fn resolve_conflicts(&mut self) {
        let ids: Vec<u64> = self.hypotheses.iter().map(|h| h.id).collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let sim = QuantizedVSA::similarity(
                    &self.hypotheses[i].content,
                    &self.hypotheses[j].content,
                );
                let conf_diff =
                    (self.hypotheses[i].confidence - self.hypotheses[j].confidence).abs();
                if sim > 0.9 && conf_diff > 0.4 {
                    if self.hypotheses[i].confidence < self.hypotheses[j].confidence {
                        self.hypotheses[i].is_contradicted = true;
                    } else {
                        self.hypotheses[j].is_contradicted = true;
                    }
                }
            }
        }
    }

    pub fn best_hypothesis(&self) -> Option<&Hypothesis> {
        self.hypotheses
            .iter()
            .filter(|h| !h.is_contradicted)
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn top_hypotheses(&self, n: usize) -> Vec<&Hypothesis> {
        let mut sorted: Vec<&Hypothesis> = self
            .hypotheses
            .iter()
            .filter(|h| !h.is_contradicted)
            .collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(n);
        sorted
    }

    pub fn clear(&mut self) {
        self.hypotheses.clear();
    }

    pub fn evidence_chain(&self, id: u64) -> Vec<u64> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            chain.push(current);
            if let Some(h) = self.get_hypothesis(current) {
                for &eid in &h.supporting_evidence {
                    if !visited.contains(&eid) {
                        stack.push(eid);
                    }
                }
            }
        }
        chain
    }
}
