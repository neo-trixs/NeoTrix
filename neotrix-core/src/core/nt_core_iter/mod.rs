pub mod self_ref_code;

pub use self_ref_code::{
    CodeMutation, MutationRequest, MutationResult, MutationRisk, MutationType,
    RollbackPlan, SelfCodeMonitor, SelfRefStats,
};

/// 自迭代公共 trait
pub trait SelfIteration {
    type IterationResult;
    type Evaluation;

    fn iterate(&mut self) -> Self::IterationResult;
    fn evaluate(&self) -> Self::Evaluation;
    fn absorb_feedback(&mut self, feedback: f64);
    fn should_continue(&self, threshold: f64) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CounterIter {
        count: u32,
        feedback: f64,
    }

    impl SelfIteration for CounterIter {
        type IterationResult = u32;
        type Evaluation = f64;

        fn iterate(&mut self) -> u32 { self.count += 1; self.count }
        fn evaluate(&self) -> f64 { self.count as f64 * 0.5 }
        fn absorb_feedback(&mut self, fb: f64) { self.feedback = fb; }
        fn should_continue(&self, t: f64) -> bool { self.evaluate() < t }
    }

    #[test]
    fn test_iterate_increments() {
        let mut c = CounterIter { count: 0, feedback: 0.0 };
        assert_eq!(c.iterate(), 1);
        assert_eq!(c.iterate(), 2);
    }

    #[test]
    fn test_evaluate_scales() {
        let mut c = CounterIter { count: 0, feedback: 0.0 };
        c.iterate();
        assert!((c.evaluate() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_absorb_feedback() {
        let mut c = CounterIter { count: 0, feedback: 0.0 };
        c.absorb_feedback(0.8);
        assert!((c.feedback - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_should_continue() {
        let mut c = CounterIter { count: 0, feedback: 0.0 };
        assert!(c.should_continue(10.0));
        c.iterate();
        c.iterate();
        c.iterate();
        assert!(!c.should_continue(1.0));
    }
}
