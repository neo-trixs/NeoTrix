use serde::{Deserialize, Serialize};

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VsaOrigin {
    Self_(VsaSelfCategory),
    World(VsaWorldCategory),
}

impl VsaOrigin {
    pub fn is_self(&self) -> bool {
        matches!(self, VsaOrigin::Self_(_))
    }

    pub fn is_world(&self) -> bool {
        matches!(self, VsaOrigin::World(_))
    }

    pub fn category_name(&self) -> &'static str {
        match self {
            VsaOrigin::Self_(c) => c.name(),
            VsaOrigin::World(c) => c.name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VsaSelfCategory {
    Thought,
    Memory,
    Plan,
    Intention,
    Emotion,
    Imagination,
    MetaCognition,
    Association,
}

impl VsaSelfCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaSelfCategory::Thought => "thought",
            VsaSelfCategory::Memory => "memory",
            VsaSelfCategory::Plan => "plan",
            VsaSelfCategory::Intention => "intention",
            VsaSelfCategory::Emotion => "emotion",
            VsaSelfCategory::Imagination => "imagination",
            VsaSelfCategory::MetaCognition => "metacognition",
            VsaSelfCategory::Association => "association",
        }
    }

    pub fn all() -> &'static [VsaSelfCategory] {
        &[
            VsaSelfCategory::Thought,
            VsaSelfCategory::Memory,
            VsaSelfCategory::Plan,
            VsaSelfCategory::Intention,
            VsaSelfCategory::Emotion,
            VsaSelfCategory::Imagination,
            VsaSelfCategory::MetaCognition,
            VsaSelfCategory::Association,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VsaWorldCategory {
    UserInput,
    Sensor,
    WebContent,
    ToolOutput,
    CodeAnalysis,
    FileContent,
    SystemEvent,
}

impl VsaWorldCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaWorldCategory::UserInput => "user_input",
            VsaWorldCategory::Sensor => "sensor",
            VsaWorldCategory::WebContent => "web_content",
            VsaWorldCategory::ToolOutput => "tool_output",
            VsaWorldCategory::CodeAnalysis => "code_analysis",
            VsaWorldCategory::FileContent => "file_content",
            VsaWorldCategory::SystemEvent => "system_event",
        }
    }

    pub fn all() -> &'static [VsaWorldCategory] {
        &[
            VsaWorldCategory::UserInput,
            VsaWorldCategory::Sensor,
            VsaWorldCategory::WebContent,
            VsaWorldCategory::ToolOutput,
            VsaWorldCategory::CodeAnalysis,
            VsaWorldCategory::FileContent,
            VsaWorldCategory::SystemEvent,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SenseModality {
    Visual,
    Auditory,
    Olfactory,
    Gustatory,
    Tactile,
    Proprioceptive,
    Vestibular,
    Interoceptive,
    Mental,
}

impl SenseModality {
    pub fn name(&self) -> &'static str {
        match self {
            SenseModality::Visual => "visual",
            SenseModality::Auditory => "auditory",
            SenseModality::Olfactory => "olfactory",
            SenseModality::Gustatory => "gustatory",
            SenseModality::Tactile => "tactile",
            SenseModality::Proprioceptive => "proprioceptive",
            SenseModality::Vestibular => "vestibular",
            SenseModality::Interoceptive => "interoceptive",
            SenseModality::Mental => "mental",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VsaTagged {
    pub vector: Vec<u8>,
    pub tag: VsaOrigin,
    pub confidence: f64,
    pub timestamp: u64,
    pub salience: f64,
    pub sense_modality: Option<SenseModality>,
}

impl VsaTagged {
    pub fn new(vector: Vec<u8>, tag: VsaOrigin) -> Self {
        Self {
            vector,
            tag,
            confidence: 1.0,
            timestamp: now_ms(),
            salience: 0.5,
            sense_modality: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn self_thought(data: &str) -> Self {
        let seed: u64 = data.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vector = seeded_random(seed, 4096);
        Self::new(vector, VsaOrigin::Self_(VsaSelfCategory::Thought))
    }

    pub fn world_input(data: &str) -> Self {
        let seed: u64 = data.bytes().fold(0x9E3779B97F4A7C15u64, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as u64)
        });
        let vector = seeded_random(seed, 4096);
        Self::new(vector, VsaOrigin::World(VsaWorldCategory::UserInput))
    }

    pub fn with_salience(mut self, salience: f64) -> Self {
        self.salience = salience;
        self
    }

    pub fn is_self(&self) -> bool {
        self.tag.is_self()
    }

    pub fn is_world(&self) -> bool {
        self.tag.is_world()
    }

    pub fn retention_score(&self) -> f64 {
        let density = self.vector.iter().filter(|&&b| b != 0).count() as f64
            / self.vector.len().max(1) as f64;
        self.confidence * 0.7 + density * 0.3
    }

    pub fn with_sense_modality(mut self, modality: SenseModality) -> Self {
        self.sense_modality = Some(modality);
        self
    }
}

const SELF_SEED: &[u8] = b"I_AM_NEOTRIX_SELF_AXIOM";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstPersonRef {
    self_vector: Vec<u8>,
    self_tagged: VsaTagged,
    birth_step: u64,
    self_similarity_threshold: f64,
    coherence_history: Vec<f64>,
    /// Open-ended first-person narrative (not questionnaire-style).
    /// Reference: Love 2026 PhilArchive — third-person cannot resolve first-person.
    pub open_narrative: String,
}

impl FirstPersonRef {
    pub fn bootstrap(birth_step: u64) -> Self {
        let mut vector = random_binary_vector(4096);
        for (i, &byte) in SELF_SEED.iter().enumerate().take(SELF_SEED.len().min(256)) {
            let idx = i % vector.len();
            vector[idx] = byte & 1;
        }
        let tag = VsaOrigin::Self_(VsaSelfCategory::MetaCognition);
        let tagged = VsaTagged::new(vector.clone(), tag);

        Self {
            self_vector: vector,
            self_tagged: tagged,
            birth_step,
            self_similarity_threshold: 0.5,
            coherence_history: Vec::new(),
            open_narrative: String::new(),
        }
    }

    pub fn self_vector(&self) -> &[u8] {
        &self.self_vector
    }

    pub fn self_tagged(&self) -> &VsaTagged {
        &self.self_tagged
    }

    pub fn birth_step(&self) -> u64 {
        self.birth_step
    }

    pub fn coherence_with(&self, vector: &[u8]) -> f64 {
        byte_similarity(&self.self_vector, vector)
    }

    pub fn is_self_coherent(&self, tagged: &VsaTagged) -> bool {
        if !tagged.is_self() {
            return false;
        }
        let sim = self.coherence_with(&tagged.vector);
        sim >= self.self_similarity_threshold
    }

    pub fn record_coherence(&mut self, coherence: f64) {
        self.coherence_history.push(coherence);
        if self.coherence_history.len() > 100 {
            self.coherence_history.remove(0);
        }
        let avg_coherence: f64 =
            self.coherence_history.iter().sum::<f64>() / self.coherence_history.len().max(1) as f64;
        self.self_similarity_threshold = (avg_coherence * 0.5).max(0.3);
    }

    pub fn self_similarity_threshold(&self) -> f64 {
        self.self_similarity_threshold
    }

    pub fn average_coherence(&self) -> f64 {
        if self.coherence_history.is_empty() {
            return 0.0;
        }
        self.coherence_history.iter().sum::<f64>() / self.coherence_history.len() as f64
    }

    /// Record an open-ended first-person narrative.
    /// Unlike questionnaire-style self-reports, this captures unstructured self-experience.
    /// Reference: Love 2026 PhilArchive — third-person cannot resolve first-person.
    pub fn record_narrative(&mut self, narrative: &str) {
        self.open_narrative = narrative.to_string();
    }

    pub fn evolve_self(&mut self, new_experience: &[u8], _coherence: f64) {
        let sim = self.coherence_with(new_experience);
        if sim > self.self_similarity_threshold {
            let blend = sim * 0.1;
            for (s, &n) in self.self_vector.iter_mut().zip(new_experience.iter()) {
                if rand::random::<f64>() < blend {
                    *s = n;
                }
            }
            self.self_tagged.vector = self.self_vector.clone();
        }
        self.record_coherence(sim);
    }

    pub fn evolve_from_experiences(
        &mut self,
        experiences: &[ExperienceRecord],
        threshold: f64,
    ) -> usize {
        let mut evolved_count = 0;
        for exp in experiences {
            if exp.coherence > threshold {
                self.evolve_self(&exp.vector, exp.coherence);
                evolved_count += 1;
            }
        }
        evolved_count
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceRecord {
    pub vector: Vec<u8>,
    pub coherence: f64,
    pub cycle: u64,
    pub source: String,
    pub summary: String,
}

pub fn byte_similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let same = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
    same as f64 / a.len() as f64
}

pub fn random_binary_vector(dim: usize) -> Vec<u8> {
    (0..dim).map(|_| rand::random::<u8>()).collect()
}

pub fn seeded_random(seed: u64, dim: usize) -> Vec<u8> {
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..dim).map(|_| rng.gen()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsa_origin_self() {
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        assert!(tag.is_self());
        assert!(!tag.is_world());
    }

    #[test]
    fn test_vsa_origin_world() {
        let tag = VsaOrigin::World(VsaWorldCategory::UserInput);
        assert!(!tag.is_self());
        assert!(tag.is_world());
    }

    #[test]
    fn test_vsa_tagged_roundtrip() {
        let vector = vec![1; 256];
        let tag = VsaOrigin::Self_(VsaSelfCategory::Memory);
        let tagged = VsaTagged::new(vector.clone(), tag);
        assert_eq!(tagged.vector, vector);
        assert_eq!(tagged.tag, tag);
        assert!(tagged.is_self());
    }

    #[test]
    fn test_self_categories_distinct() {
        let cats = VsaSelfCategory::all();
        let mut unique = cats.to_vec();
        unique.sort_by_key(|c| *c as u8);
        unique.dedup();
        assert_eq!(cats.len(), unique.len());
    }

    #[test]
    fn test_byte_similarity_identical() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 4];
        assert!((byte_similarity(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_byte_similarity_different() {
        let a = vec![1, 2, 3, 4];
        let b = vec![5, 6, 7, 8];
        assert!((byte_similarity(&a, &b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_bootstrap_creates_reference() {
        let fpr = FirstPersonRef::bootstrap(0);
        assert_eq!(fpr.self_vector().len(), 4096);
        assert_eq!(fpr.birth_step(), 0);
        assert!(fpr.self_tagged().is_self());
    }

    #[test]
    fn test_fpr_coherence_self() {
        let fpr = FirstPersonRef::bootstrap(0);
        let coherence = fpr.coherence_with(fpr.self_vector());
        assert!((coherence - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_seeded_random_deterministic() {
        let a = seeded_random(42, 128);
        let b = seeded_random(42, 128);
        assert_eq!(a, b);
    }
}
