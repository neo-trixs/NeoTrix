#[derive(Debug, Clone, PartialEq)]
pub enum Verdict { Allow, Deny, Flag }

pub struct RetrievalGate { pub threshold: f64 }

impl RetrievalGate {
    pub fn new(threshold: f64) -> Self { RetrievalGate { threshold } }
    pub fn evaluate(&self, similarity: f64, is_sensitive: bool) -> Verdict {
        if similarity >= self.threshold && is_sensitive { Verdict::Deny }
        else if similarity >= self.threshold { Verdict::Flag }
        else { Verdict::Allow }
    }
    pub fn threshold(&self) -> f64 { self.threshold }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_retrieval_gate_allow() { let g = RetrievalGate::new(0.8); assert_eq!(g.evaluate(0.5, false), Verdict::Allow); }
    #[test] fn test_retrieval_gate_flag() { let g = RetrievalGate::new(0.8); assert_eq!(g.evaluate(0.9, false), Verdict::Flag); }
    #[test] fn test_retrieval_gate_deny() { let g = RetrievalGate::new(0.8); assert_eq!(g.evaluate(0.9, true), Verdict::Deny); }
    #[test] fn test_retrieval_gate_threshold() { let g = RetrievalGate::new(0.8); assert!((g.threshold() - 0.8).abs() < 1e-6); }
}
