use crate::core::nt_core_hcube::QuantizedVSA;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControlPhrase {
    Rephrase,
    Correct,
}

impl ControlPhrase {
    pub fn from_reward(reward: bool) -> Self {
        if reward {
            ControlPhrase::Rephrase
        } else {
            ControlPhrase::Correct
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ControlPhrase::Rephrase => "Let me rephrase",
            ControlPhrase::Correct => "This is wrong. Let me correct.",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RevisionTrace {
    pub initial: Vec<u8>,
    pub reward: bool,
    pub control_phrase: String,
    pub revised: Vec<u8>,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct SelfRevisionLoop {
    pub revision_traces: Vec<RevisionTrace>,
    pub teacher: Option<Box<SelfRevisionLoop>>,
    pub max_traces: usize,
    pub generation_count: usize,
    pub distillation_rate: f64,
}

impl SelfRevisionLoop {
    pub fn new() -> Self {
        SelfRevisionLoop {
            revision_traces: Vec::new(),
            teacher: None,
            max_traces: 6000,
            generation_count: 0,
            distillation_rate: 0.01,
        }
    }

    pub fn with_max_traces(max_traces: usize) -> Self {
        SelfRevisionLoop {
            revision_traces: Vec::new(),
            teacher: None,
            max_traces,
            generation_count: 0,
            distillation_rate: 0.01,
        }
    }

    pub fn collect_trace(&mut self, initial: Vec<u8>, reward: bool, revised: Vec<u8>) {
        let control = Self::control_phrase(reward);
        let trace = RevisionTrace {
            initial,
            reward,
            control_phrase: control,
            revised,
            verified: true,
        };
        if self.revision_traces.len() >= self.max_traces {
            self.revision_traces.remove(0);
        }
        self.revision_traces.push(trace);
    }

    pub fn collect_revision_traces(&mut self, traces: Vec<(Vec<u8>, bool, Vec<u8>)>) {
        for (initial, reward, revised) in traces {
            let sim = QuantizedVSA::similarity(&initial, &revised);
            if sim > 0.5 {
                self.collect_trace(initial, reward, revised);
            }
        }
    }

    pub fn sync_teacher(&mut self) {
        let frozen = SelfRevisionLoop {
            revision_traces: self.revision_traces.clone(),
            teacher: None,
            max_traces: self.max_traces,
            generation_count: self.generation_count,
            distillation_rate: self.distillation_rate,
        };
        self.teacher = Some(Box::new(frozen));
    }

    pub fn distill_step(&self) -> f64 {
        let teacher = match &self.teacher {
            Some(t) => t,
            None => return 0.0,
        };
        if self.revision_traces.is_empty() || teacher.revision_traces.is_empty() {
            return 0.0;
        }

        let n = self
            .revision_traces
            .len()
            .min(teacher.revision_traces.len());
        let mut total_kl = 0.0f64;

        for i in 0..n {
            let s_initial = &self.revision_traces[i].initial;
            let s_revised = &self.revision_traces[i].revised;
            let t_initial = &teacher.revision_traces[i].initial;
            let t_revised = &teacher.revision_traces[i].revised;

            let s_sim = QuantizedVSA::similarity(s_initial, s_revised);
            let t_sim = QuantizedVSA::similarity(t_initial, t_revised);

            let p = (s_sim + 1.0) / 2.0;
            let q = (t_sim + 1.0) / 2.0;
            let p = p.clamp(1e-12, 1.0 - 1e-12);
            let q = q.clamp(1e-12, 1.0 - 1e-12);

            total_kl += p * (p / q).ln() + (1.0 - p) * ((1.0 - p) / (1.0 - q)).ln();
        }

        total_kl / n as f64
    }

    pub fn distill_from_teacher(&mut self) -> f64 {
        if self.teacher.is_none() {
            return 0.0;
        }
        let loss = self.distill_step();
        for trace in &mut self.revision_traces {
            let mut perturbed = trace.revised.clone();
            for byte in &mut perturbed {
                if fastrand::f64() < self.distillation_rate {
                    *byte ^= 1;
                }
            }
            trace.revised = perturbed;
        }
        self.generation_count += 1;
        loss
    }

    pub fn apply_revision(&self, initial: &[u8]) -> Vec<u8> {
        if self.revision_traces.is_empty() {
            return initial.to_vec();
        }

        let mut scored: Vec<(f64, &RevisionTrace)> = self
            .revision_traces
            .iter()
            .map(|t| (QuantizedVSA::similarity(initial, &t.initial), t))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let top_k: Vec<&[u8]> = scored
            .iter()
            .take(3)
            .map(|(_, t)| t.revised.as_slice())
            .collect();

        if top_k.len() == 1 {
            return top_k[0].to_vec();
        }

        QuantizedVSA::bundle(&top_k)
    }

    pub fn control_phrase(reward: bool) -> String {
        ControlPhrase::from_reward(reward).as_str().to_string()
    }

    pub fn trace_count(&self) -> usize {
        self.revision_traces.len()
    }

    pub fn verified_count(&self) -> usize {
        self.revision_traces.iter().filter(|t| t.verified).count()
    }
}

impl Default for SelfRevisionLoop {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa_vec(value: u8) -> Vec<u8> {
        vec![value; crate::core::nt_core_hcube::VSA_DIM]
    }

    fn random_vsa() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_new_creates_empty_loop() {
        let srl = SelfRevisionLoop::new();
        assert_eq!(srl.trace_count(), 0);
        assert!(srl.teacher.is_none());
        assert_eq!(srl.max_traces, 6000);
        assert_eq!(srl.generation_count, 0);
    }

    #[test]
    fn test_with_max_traces() {
        let srl = SelfRevisionLoop::with_max_traces(100);
        assert_eq!(srl.max_traces, 100);
    }

    #[test]
    fn test_control_phrase_reward_true() {
        assert_eq!(SelfRevisionLoop::control_phrase(true), "Let me rephrase");
    }

    #[test]
    fn test_control_phrase_reward_false() {
        assert_eq!(
            SelfRevisionLoop::control_phrase(false),
            "This is wrong. Let me correct."
        );
    }

    #[test]
    fn test_control_phrase_enum() {
        assert_eq!(ControlPhrase::from_reward(true), ControlPhrase::Rephrase);
        assert_eq!(ControlPhrase::from_reward(false), ControlPhrase::Correct);
        assert_eq!(ControlPhrase::Rephrase.as_str(), "Let me rephrase");
        assert_eq!(
            ControlPhrase::Correct.as_str(),
            "This is wrong. Let me correct."
        );
    }

    #[test]
    fn test_collect_trace_stores_verified() {
        let mut srl = SelfRevisionLoop::new();
        let initial = make_vsa_vec(1);
        let revised = make_vsa_vec(0);

        srl.collect_trace(initial.clone(), true, revised.clone());

        assert_eq!(srl.trace_count(), 1);
        assert_eq!(srl.verified_count(), 1);
        assert_eq!(srl.revision_traces[0].initial, initial);
        assert_eq!(srl.revision_traces[0].reward, true);
        assert_eq!(srl.revision_traces[0].revised, revised);
        assert_eq!(srl.revision_traces[0].verified, true);
    }

    #[test]
    fn test_collect_trace_evicts_oldest_when_full() {
        let mut srl = SelfRevisionLoop::with_max_traces(2);
        let a = make_vsa_vec(1);
        let b = make_vsa_vec(2);
        let c = make_vsa_vec(3);

        srl.collect_trace(a.clone(), true, make_vsa_vec(0));
        srl.collect_trace(b.clone(), true, make_vsa_vec(0));
        assert_eq!(srl.trace_count(), 2);

        srl.collect_trace(c.clone(), false, make_vsa_vec(0));
        assert_eq!(srl.trace_count(), 2);
        assert_eq!(srl.revision_traces[0].initial, b);
        assert_eq!(srl.revision_traces[1].initial, c);
    }

    #[test]
    fn test_sync_teacher_creates_independent_copy() {
        let mut srl = SelfRevisionLoop::new();
        let initial = make_vsa_vec(5);
        srl.collect_trace(initial.clone(), true, make_vsa_vec(10));
        srl.sync_teacher();

        assert!(srl.teacher.is_some());
        assert_eq!(srl.teacher.as_ref().unwrap().trace_count(), 1);

        srl.collect_trace(make_vsa_vec(15), false, make_vsa_vec(20));
        assert_eq!(srl.trace_count(), 2);
        assert_eq!(srl.teacher.as_ref().unwrap().trace_count(), 1);
    }

    #[test]
    fn test_distill_step_without_teacher_returns_zero() {
        let srl = SelfRevisionLoop::new();
        assert_eq!(srl.distill_step(), 0.0);
    }

    #[test]
    fn test_distill_step_returns_non_zero_loss() {
        let mut srl = SelfRevisionLoop::new();
        srl.collect_trace(make_vsa_vec(0), true, make_vsa_vec(1));
        srl.collect_trace(make_vsa_vec(2), false, make_vsa_vec(3));
        srl.sync_teacher();

        let loss = srl.distill_step();
        assert!(
            loss > 0.0,
            "distillation loss should be non-zero, got {}",
            loss
        );
    }

    #[test]
    fn test_distill_step_converges_with_identical_student_and_teacher() {
        let mut srl = SelfRevisionLoop::new();
        srl.collect_trace(make_vsa_vec(0), true, make_vsa_vec(1));
        srl.sync_teacher();

        let loss = srl.distill_step();
        assert!(
            (loss - 0.0).abs() < 1e-6,
            "identical student and teacher should have near-zero loss, got {}",
            loss
        );
    }

    #[test]
    fn test_apply_revision_returns_initial_when_no_traces() {
        let srl = SelfRevisionLoop::new();
        let initial = make_vsa_vec(42);
        let result = srl.apply_revision(&initial);
        assert_eq!(result, initial);
    }

    #[test]
    fn test_apply_revision_with_single_trace() {
        let mut srl = SelfRevisionLoop::new();
        let initial = make_vsa_vec(100);
        let revised = make_vsa_vec(200);
        srl.collect_trace(initial.clone(), true, revised.clone());

        let result = srl.apply_revision(&initial);
        assert_eq!(result, revised);
    }

    #[test]
    fn test_apply_revision_bundles_top_k() {
        let mut srl = SelfRevisionLoop::new();
        let query = make_vsa_vec(128);
        srl.collect_trace(make_vsa_vec(127), true, make_vsa_vec(200));
        srl.collect_trace(make_vsa_vec(129), false, make_vsa_vec(201));
        srl.collect_trace(make_vsa_vec(0), true, make_vsa_vec(202));
        srl.collect_trace(make_vsa_vec(255), false, make_vsa_vec(203));

        let result = srl.apply_revision(&query);
        assert_eq!(result.len(), crate::core::nt_core_hcube::VSA_DIM);
    }

    #[test]
    fn test_apply_revision_with_random_vectors() {
        let mut srl = SelfRevisionLoop::new();
        for _ in 0..5 {
            let init = random_vsa();
            let rev = random_vsa();
            srl.collect_trace(init, true, rev);
        }

        let query = random_vsa();
        let result = srl.apply_revision(&query);
        assert_eq!(result.len(), crate::core::nt_core_hcube::VSA_DIM);
    }

    #[test]
    fn test_collect_revision_traces_filters_by_similarity() {
        let mut srl = SelfRevisionLoop::new();
        let similar_init = make_vsa_vec(100);
        let similar_rev = make_vsa_vec(100);
        let dissimilar_init = make_vsa_vec(0);
        let dissimilar_rev = make_vsa_vec(255);

        let traces = vec![
            (similar_init, true, similar_rev),
            (dissimilar_init, false, dissimilar_rev),
        ];

        srl.collect_revision_traces(traces);
        assert_eq!(srl.trace_count(), 1);
    }

    #[test]
    fn test_distill_from_teacher_increments_generation() {
        let mut srl = SelfRevisionLoop::new();
        srl.collect_trace(make_vsa_vec(1), true, make_vsa_vec(2));
        srl.sync_teacher();
        assert_eq!(srl.generation_count, 0);

        let loss = srl.distill_from_teacher();
        assert!(loss > 0.0);
        assert_eq!(srl.generation_count, 1);
    }

    #[test]
    fn test_distill_from_teacher_without_teacher_returns_zero() {
        let mut srl = SelfRevisionLoop::new();
        assert_eq!(srl.distill_from_teacher(), 0.0);
        assert_eq!(srl.generation_count, 0);
    }

    #[test]
    fn test_empty_traces_edge() {
        let srl = SelfRevisionLoop::new();
        assert_eq!(srl.verified_count(), 0);
        assert_eq!(srl.distill_step(), 0.0);

        let initial = make_vsa_vec(1);
        let result = srl.apply_revision(&initial);
        assert_eq!(result, initial);
    }

    #[test]
    fn test_single_trace_edge() {
        let mut srl = SelfRevisionLoop::new();
        srl.collect_trace(make_vsa_vec(3), false, make_vsa_vec(7));
        assert_eq!(srl.trace_count(), 1);
        assert_eq!(srl.verified_count(), 1);
        assert_eq!(srl.revision_traces[0].reward, false);
        assert_eq!(
            srl.revision_traces[0].control_phrase,
            "This is wrong. Let me correct."
        );
    }

    #[test]
    fn test_default_impl() {
        let srl: SelfRevisionLoop = Default::default();
        assert_eq!(srl.trace_count(), 0);
        assert_eq!(srl.max_traces, 6000);
    }
}
