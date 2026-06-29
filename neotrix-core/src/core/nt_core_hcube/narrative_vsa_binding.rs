use crate::core::nt_core_hcube::vsa_vector::{MapVsaBackend, VsaBackend, VsaVector};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const VSA_NARRATIVE_DIM: usize = 4096;
type NarrativeVsa = VsaVector<VSA_NARRATIVE_DIM>;

#[derive(Debug, Clone)]
pub struct RoleFillerBinding {
    pub role_subject: NarrativeVsa,
    pub role_predicate: NarrativeVsa,
    pub role_object: NarrativeVsa,
    pub role_tense: NarrativeVsa,
    pub role_certainty: NarrativeVsa,
    pub role_emotion: NarrativeVsa,
    pub role_time: NarrativeVsa,
    pub role_place: NarrativeVsa,
    pub role_cause: NarrativeVsa,
    pub role_effect: NarrativeVsa,
    pub filler_cache: HashMap<String, NarrativeVsa>,
}

#[derive(Debug, Clone)]
pub struct NarrativeFrame {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub tense: NarrativeTense,
    pub certainty: f64,
    pub emotional_weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NarrativeTense {
    Past,
    Present,
    Future,
    Generic,
}

#[derive(Debug, Clone)]
pub struct BoundNarrative {
    pub frame: NarrativeFrame,
    pub vsa_vector: NarrativeVsa,
    pub binding_strength: f64,
}

pub fn hash_to_seed(label: &str) -> u64 {
    let mut h = DefaultHasher::new();
    label.hash(&mut h);
    h.finish()
}

pub fn tense_to_role_modulator(tense: &NarrativeTense) -> NarrativeVsa {
    let seed = match tense {
        NarrativeTense::Past => 0x1111_1111_1111_1111u64,
        NarrativeTense::Present => 0x2222_2222_2222_2222u64,
        NarrativeTense::Future => 0x3333_3333_3333_3333u64,
        NarrativeTense::Generic => 0x4444_4444_4444_4444u64,
    };
    VsaVector::random(seed)
}

impl RoleFillerBinding {
    pub fn new() -> Self {
        Self {
            role_subject: VsaVector::random(0xABCD_0001),
            role_predicate: VsaVector::random(0xABCD_0002),
            role_object: VsaVector::random(0xABCD_0003),
            role_tense: VsaVector::random(0xABCD_0004),
            role_certainty: VsaVector::random(0xABCD_0005),
            role_emotion: VsaVector::random(0xABCD_0006),
            role_time: VsaVector::random(0xABCD_0007),
            role_place: VsaVector::random(0xABCD_0008),
            role_cause: VsaVector::random(0xABCD_0009),
            role_effect: VsaVector::random(0xABCD_000A),
            filler_cache: HashMap::new(),
        }
    }

    pub fn get_or_create_filler(&mut self, label: &str) -> NarrativeVsa {
        if let Some(cached) = self.filler_cache.get(label) {
            return cached.clone();
        }
        let seed = hash_to_seed(label);
        let vec = VsaVector::random(seed);
        self.filler_cache.insert(label.to_string(), vec.clone());
        vec
    }

    pub fn bind(&mut self, frame: &NarrativeFrame) -> BoundNarrative {
        let backend = MapVsaBackend;

        let subject_vec = self.get_or_create_filler(&frame.subject);
        let predicate_vec = self.get_or_create_filler(&frame.predicate);
        let object_vec = self.get_or_create_filler(&frame.object);

        let bound_s = backend.bind(&self.role_subject, &subject_vec);
        let bound_p = backend.bind(&self.role_predicate, &predicate_vec);
        let bound_o = backend.bind(&self.role_object, &object_vec);

        let combined = backend.bundle(&[&bound_s, &bound_p, &bound_o]);

        let sim_s = backend.similarity(&combined, &bound_s);
        let binding_strength = (sim_s
            + backend.similarity(&combined, &bound_p)
            + backend.similarity(&combined, &bound_o))
            / 3.0;

        BoundNarrative {
            frame: frame.clone(),
            vsa_vector: combined,
            binding_strength,
        }
    }

    pub fn unbind(bound: &NarrativeVsa, role: &NarrativeVsa) -> NarrativeVsa {
        MapVsaBackend.bind(bound, role)
    }

    pub fn query_subject(&self, bound: &NarrativeVsa) -> String {
        self.query_role(bound, &self.role_subject)
    }

    pub fn query_predicate(&self, bound: &NarrativeVsa) -> String {
        self.query_role(bound, &self.role_predicate)
    }

    pub fn query_object(&self, bound: &NarrativeVsa) -> String {
        self.query_role(bound, &self.role_object)
    }

    fn query_role(&self, bound: &NarrativeVsa, role: &NarrativeVsa) -> String {
        let recovered = Self::unbind(bound, role);
        let filler_vecs: Vec<NarrativeVsa> = self.filler_cache.values().cloned().collect();
        if filler_vecs.is_empty() {
            return String::new();
        }
        let backend = MapVsaBackend;
        let idx = backend.cleanup(&recovered, &filler_vecs);
        match idx {
            Some(i) => self.filler_cache.keys().nth(i).cloned().unwrap_or_default(),
            None => String::new(),
        }
    }

    pub fn similarity(a: &BoundNarrative, b: &BoundNarrative) -> f64 {
        MapVsaBackend.similarity(&a.vsa_vector, &b.vsa_vector)
    }

    pub fn compose(&mut self, frames: &[NarrativeFrame]) -> NarrativeVsa {
        let backend = MapVsaBackend;
        let bound_vecs: Vec<NarrativeVsa> =
            frames.iter().map(|f| self.bind(f).vsa_vector).collect();
        let refs: Vec<&NarrativeVsa> = bound_vecs.iter().collect();
        backend.bundle(&refs)
    }

    pub fn cache_size(&self) -> usize {
        self.filler_cache.len()
    }
}

impl Default for RoleFillerBinding {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> MapVsaBackend {
        MapVsaBackend
    }

    fn sample_frame() -> NarrativeFrame {
        NarrativeFrame {
            subject: "Alice".to_string(),
            predicate: "trusts".to_string(),
            object: "Bob".to_string(),
            tense: NarrativeTense::Present,
            certainty: 0.9,
            emotional_weight: 0.7,
        }
    }

    fn different_frame() -> NarrativeFrame {
        NarrativeFrame {
            subject: "Charlie".to_string(),
            predicate: "distrusts".to_string(),
            object: "Dave".to_string(),
            tense: NarrativeTense::Past,
            certainty: 0.6,
            emotional_weight: 0.4,
        }
    }

    #[test]
    fn test_new_initializes_role_vectors() {
        let rfb = RoleFillerBinding::new();
        assert_eq!(rfb.role_subject.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_predicate.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_object.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_tense.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_certainty.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_emotion.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_time.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_place.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_cause.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(rfb.role_effect.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_ne!(rfb.role_subject, rfb.role_predicate);
        assert_eq!(rfb.cache_size(), 0);
    }

    #[test]
    fn test_get_or_create_filler_caches() {
        let mut rfb = RoleFillerBinding::new();
        let v1 = rfb.get_or_create_filler("Alice");
        let v2 = rfb.get_or_create_filler("Alice");
        assert_eq!(v1, v2);
        assert_eq!(rfb.cache_size(), 1);

        let v3 = rfb.get_or_create_filler("Bob");
        assert_ne!(v1, v3);
        assert_eq!(rfb.cache_size(), 2);
    }

    #[test]
    fn test_bind_returns_vector() {
        let mut rfb = RoleFillerBinding::new();
        let frame = sample_frame();
        let bound = rfb.bind(&frame);
        assert_eq!(bound.vsa_vector.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert_eq!(bound.frame.subject, "Alice");
        assert!(bound.binding_strength >= 0.0);
    }

    #[test]
    fn test_unbind_roundtrip_recovers_filler() {
        let mut rfb = RoleFillerBinding::new();
        let frame = sample_frame();
        let bound = rfb.bind(&frame);

        let filler_alice = rfb.get_or_create_filler("Alice");
        let recovered = RoleFillerBinding::unbind(&bound.vsa_vector, &rfb.role_subject);
        let sim = backend().similarity(&recovered, &filler_alice);
        assert!(sim > 0.4, "unbind roundtrip similarity = {}", sim);
    }

    #[test]
    fn test_query_subject_after_bind() {
        let mut rfb = RoleFillerBinding::new();
        let frame = sample_frame();
        let bound = rfb.bind(&frame);
        let subject = rfb.query_subject(&bound.vsa_vector);
        assert_eq!(subject, "Alice");
    }

    #[test]
    fn test_similarity_same_frame_high() {
        let mut rfb = RoleFillerBinding::new();
        let frame = sample_frame();
        let a = rfb.bind(&frame);
        let b = rfb.bind(&frame);
        let sim = RoleFillerBinding::similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "same frame similarity = {}", sim);
    }

    #[test]
    fn test_similarity_different_frame_lower() {
        let mut rfb = RoleFillerBinding::new();
        let a = rfb.bind(&sample_frame());
        let b = rfb.bind(&different_frame());
        let sim_same = RoleFillerBinding::similarity(&a, &a);
        let sim_diff = RoleFillerBinding::similarity(&a, &b);
        assert!(sim_same > 0.99);
        assert!(
            sim_diff < sim_same,
            "different frames should be less similar"
        );
    }

    #[test]
    fn test_cache_size_increases() {
        let mut rfb = RoleFillerBinding::new();
        assert_eq!(rfb.cache_size(), 0);
        rfb.get_or_create_filler("X");
        assert_eq!(rfb.cache_size(), 1);
        rfb.get_or_create_filler("Y");
        assert_eq!(rfb.cache_size(), 2);
        rfb.get_or_create_filler("X");
        assert_eq!(rfb.cache_size(), 2);
    }

    #[test]
    fn test_compose_multiple_frames() {
        let mut rfb = RoleFillerBinding::new();
        let frames = vec![sample_frame(), different_frame()];
        let story = rfb.compose(&frames);
        assert_eq!(story.as_bytes().len(), VSA_NARRATIVE_DIM);
        assert!(rfb.cache_size() >= 4);
    }

    #[test]
    fn test_deterministic_seeds() {
        let a = hash_to_seed("Alice");
        let b = hash_to_seed("Alice");
        assert_eq!(a, b);
        let c = hash_to_seed("Bob");
        assert_ne!(a, c);
    }

    #[test]
    fn test_tense_modulator_distinct() {
        let past = tense_to_role_modulator(&NarrativeTense::Past);
        let present = tense_to_role_modulator(&NarrativeTense::Present);
        let future = tense_to_role_modulator(&NarrativeTense::Future);
        let generic = tense_to_role_modulator(&NarrativeTense::Generic);
        assert_ne!(past, present);
        assert_ne!(present, future);
        assert_ne!(future, generic);
        assert_eq!(tense_to_role_modulator(&NarrativeTense::Past), past);
    }

    #[test]
    fn test_query_predicate_and_object() {
        let mut rfb = RoleFillerBinding::new();
        let frame = sample_frame();
        let bound = rfb.bind(&frame);
        assert_eq!(rfb.query_predicate(&bound.vsa_vector), "trusts");
        assert_eq!(rfb.query_object(&bound.vsa_vector), "Bob");
    }
}
