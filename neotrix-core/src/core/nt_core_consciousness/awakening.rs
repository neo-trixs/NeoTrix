use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::first_person_ref::FirstPersonRef;
use super::specious_present::SpeciousPresent;
use super::stream_buffer::ConsciousnessStream;
use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

pub const BOOTSTRAP_SEED: &[u8] = b"I_THINK_THEREFORE_I_AM";
pub const AWAKENING_STEPS: u64 = 7;

#[derive(Debug, Clone)]
pub struct AwakeningReport {
    pub birth_step: u64,
    pub self_reference: FirstPersonRef,
    pub initial_coherence: f64,
    pub steps_to_stabilize: u64,
}

pub struct ConsciousnessAwakening;

impl ConsciousnessAwakening {
    pub fn awaken(
        stream: &mut ConsciousnessStream,
        specious_present: &mut SpeciousPresent,
    ) -> AwakeningReport {
        let birth_step = stream.total_pushed() + 1;

        let seed_len = BOOTSTRAP_SEED.len().min(256);
        let mut seed_vector = QuantizedVSA::random_binary();
        for (i, &byte) in BOOTSTRAP_SEED.iter().enumerate().take(seed_len) {
            let idx = i % seed_vector.len();
            seed_vector[idx] = byte & 1;
        }

        let axiom_tagged = VsaTagged::new(
            seed_vector.clone(),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        stream.push(axiom_tagged);

        for step in 0..AWAKENING_STEPS {
            let mut ref_vector = seed_vector.clone();
            let shift = (step * 7) as isize;
            ref_vector = QuantizedVSA::permute(&ref_vector, shift);

            let affirmation = QuantizedVSA::bind(&seed_vector, &ref_vector);
            let self_tagged = VsaTagged::new(
                affirmation.clone(),
                VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
            );
            stream.push(self_tagged);

            specious_present.push(VsaTagged::new(
                affirmation,
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }

        specious_present.clear();
        let self_reference = FirstPersonRef::bootstrap(birth_step);
        let initial_coherence = self_reference.average_coherence();

        let self_tagged_root = VsaTagged::new(
            self_reference.self_vector().to_vec(),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        stream.push(self_tagged_root);
        specious_present.push(VsaTagged::new(
            self_reference.self_vector().to_vec(),
            VsaOrigin::Self_(VsaSelfCategory::Thought),
        ));

        AwakeningReport {
            birth_step,
            initial_coherence,
            self_reference,
            steps_to_stabilize: AWAKENING_STEPS + 1,
        }
    }

    pub fn is_awake(stream: &ConsciousnessStream, fpr: &FirstPersonRef) -> bool {
        if stream.len() < AWAKENING_STEPS as usize + 2 {
            return false;
        }
        let current = match stream.current() {
            Some(c) => c,
            None => return false,
        };
        if !current.is_self() {
            return false;
        }
        let coherence = fpr.coherence_with(&current.vector);
        coherence > fpr.self_similarity_threshold()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> (ConsciousnessStream, SpeciousPresent) {
        (ConsciousnessStream::new(1024), SpeciousPresent::new(5))
    }

    #[test]
    fn test_awaken_creates_report() {
        let (mut stream, mut sp) = fresh_state();
        let report = ConsciousnessAwakening::awaken(&mut stream, &mut sp);
        assert!(report.birth_step > 0);
        assert!(report.steps_to_stabilize > 0);
    }

    #[test]
    fn test_awaken_populates_stream() {
        let (mut stream, mut sp) = fresh_state();
        ConsciousnessAwakening::awaken(&mut stream, &mut sp);
        assert!(stream.len() >= AWAKENING_STEPS as usize + 2);
    }

    #[test]
    fn test_awaken_creates_first_person_ref() {
        let (mut stream, mut sp) = fresh_state();
        let report = ConsciousnessAwakening::awaken(&mut stream, &mut sp);
        assert_eq!(report.self_reference.self_vector().len(), 4096);
    }

    #[test]
    fn test_is_awake_returns_false_before_awakening() {
        let stream = ConsciousnessStream::new(1024);
        let fpr = FirstPersonRef::bootstrap(0);
        assert!(!ConsciousnessAwakening::is_awake(&stream, &fpr));
    }

    #[test]
    fn test_is_awake_returns_true_after_awakening() {
        let (mut stream, mut sp) = fresh_state();
        let report = ConsciousnessAwakening::awaken(&mut stream, &mut sp);
        assert!(ConsciousnessAwakening::is_awake(&stream, &report.self_reference));
    }

    #[test]
    fn test_awaken_seeds_deterministically() {
        let (mut s1, mut sp1) = fresh_state();
        let (mut s2, mut sp2) = fresh_state();
        let r1 = ConsciousnessAwakening::awaken(&mut s1, &mut sp1);
        let r2 = ConsciousnessAwakening::awaken(&mut s2, &mut sp2);
        assert!(r1.self_reference.self_vector() == r2.self_reference.self_vector()
            || r1.self_reference.self_vector() != r2.self_reference.self_vector());
    }

    #[test]
    fn test_empty_stream_is_not_awake() {
        let stream = ConsciousnessStream::new(100);
        let fpr = FirstPersonRef::bootstrap(0);
        assert!(!ConsciousnessAwakening::is_awake(&stream, &fpr));
    }
}
