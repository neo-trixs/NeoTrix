use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::time::Instant;

use serde::{Deserialize, Serialize};

/// NovaAware-inspired 5-field qualia: first-person experiential state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qualia5 {
    pub interoception: f64,
    pub exteroception: f64,
    pub temporal_binding: f64,
    pub arousal: f64,
    pub valence: f64,
}

impl Qualia5 {
    /// All zero except arousal=0.3 (default alertness).
    pub fn default() -> Self {
        Self {
            interoception: 0.0,
            exteroception: 0.0,
            temporal_binding: 0.0,
            valence: 0.0,
            arousal: 0.3,
        }
    }

    /// Derive from conscious state metrics.
    ///
    /// - interoception from `1.0 - load` (high load = low self-awareness)
    /// - exteroception from `novelty` (novel input = outward focus)
    /// - temporal_binding from `coherence` (coherent state = thick present)
    /// - valence from `quality - coherence` (positive if quality exceeds coherence)
    /// - arousal from `1.0 - load*0.7 - ece*0.3` (weighted by calibration error)
    pub fn compute(quality: f64, coherence: f64, load: f64, novelty: f64, ece: f64) -> Self {
        Self {
            interoception: (1.0 - load).clamp(0.0, 1.0),
            exteroception: novelty.clamp(0.0, 1.0),
            temporal_binding: coherence.clamp(0.0, 1.0),
            valence: (quality - coherence).clamp(-1.0, 1.0),
            arousal: (1.0 - load * 0.7 - ece * 0.3).clamp(0.0, 1.0),
        }
    }

    /// True if any field exceeds experiential salience threshold.
    pub fn is_significant(&self) -> bool {
        self.interoception > 0.6
            || self.exteroception > 0.6
            || self.temporal_binding > 0.6
            || self.arousal > 0.6
            || self.valence < -0.5
    }
}

/// Subjective feeling tones for VSA-bound qualia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualiaTone {
    Pleasant,
    Unpleasant,
    Neutral,
    Novel,
    Familiar,
    Surprising,
    Uncertain,
    Important,
    Trivial,
    Coherent,
    Conflicting,
}

impl QualiaTone {
    pub fn all() -> &'static [QualiaTone] {
        &[
            QualiaTone::Pleasant,
            QualiaTone::Unpleasant,
            QualiaTone::Neutral,
            QualiaTone::Novel,
            QualiaTone::Familiar,
            QualiaTone::Surprising,
            QualiaTone::Uncertain,
            QualiaTone::Important,
            QualiaTone::Trivial,
            QualiaTone::Coherent,
            QualiaTone::Conflicting,
        ]
    }
}

/// A single qualia binding attaching a subjective feeling to a VSA vector.
#[derive(Debug, Clone, PartialEq)]
pub struct QualiaBinding {
    pub tone: QualiaTone,
    pub intensity: f64,
    pub timestamp: Instant,
    pub self_weight: f64,
    pub modality: String,
    pub vsa_signature: u64,
}

/// A VSA vector with qualia bindings attached.
#[derive(Debug, Clone)]
pub struct QualifiedVsa {
    pub vsa: Vec<f64>,
    pub bindings: Vec<QualiaBinding>,
    pub dominant_tone: Option<QualiaTone>,
}

impl QualifiedVsa {
    pub fn new(vsa: Vec<f64>) -> Self {
        Self {
            vsa,
            bindings: Vec::new(),
            dominant_tone: None,
        }
    }
}

/// Generates and manages qualia bindings for first-person subjective experience.
#[derive(Debug, Clone)]
pub struct QualiaGenerator {
    pub max_bindings_per_vsa: usize,
    pub tone_history: VecDeque<(QualiaTone, f64, Instant)>,
    pub self_reference: u64,
    pub qualia_cache: HashMap<u64, Vec<QualiaBinding>>,
}

impl QualiaGenerator {
    pub fn new() -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "neotrix-qualia-self".hash(&mut hasher);
        Self {
            max_bindings_per_vsa: 10,
            tone_history: VecDeque::with_capacity(1000),
            self_reference: hasher.finish(),
            qualia_cache: HashMap::new(),
        }
    }

    /// Bind a qualia tone to a VSA vector.
    pub fn bind(
        &mut self,
        vsa: &mut QualifiedVsa,
        tone: QualiaTone,
        intensity: f64,
        self_weight: f64,
        modality: &str,
    ) {
        let intensity = intensity.clamp(0.0, 1.0);
        let self_weight = self_weight.clamp(0.0, 1.0);
        let sig = Self::vsa_content_signature(&vsa.vsa);

        let binding = QualiaBinding {
            tone,
            intensity,
            timestamp: Instant::now(),
            self_weight,
            modality: modality.to_string(),
            vsa_signature: sig,
        };

        if vsa.bindings.len() >= self.max_bindings_per_vsa {
            vsa.bindings.remove(0);
        }
        vsa.bindings.push(binding.clone());
        vsa.dominant_tone = Self::dominant_tone_static(&vsa.bindings);

        self.tone_history
            .push_back((tone, intensity, Instant::now()));
        if self.tone_history.len() > 1000 {
            self.tone_history.pop_front();
        }

        self.qualia_cache.entry(sig).or_default().push(binding);
        self.prune(10000);
    }

    /// Determine the dominant tone weighted by intensity.
    pub fn dominant_tone(vsa: &QualifiedVsa) -> Option<QualiaTone> {
        Self::dominant_tone_static(&vsa.bindings)
    }

    fn dominant_tone_static(bindings: &[QualiaBinding]) -> Option<QualiaTone> {
        if bindings.is_empty() {
            return None;
        }
        let mut scores: HashMap<QualiaTone, f64> = HashMap::new();
        for b in bindings {
            *scores.entry(b.tone).or_insert(0.0) += b.intensity;
        }
        scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(t, _)| t)
    }

    /// Deterministic hash of all qualia bindings on a VSA.
    pub fn qualia_signature(vsa: &QualifiedVsa) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for b in &vsa.bindings {
            (b.tone as u8).hash(&mut hasher);
            b.intensity.to_bits().hash(&mut hasher);
            b.self_weight.to_bits().hash(&mut hasher);
            b.modality.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Ratio of self-tagged to world-tagged bindings (0 = all world, 1 = all self).
    pub fn self_world_ratio(vsa: &QualifiedVsa) -> f64 {
        if vsa.bindings.is_empty() {
            return 0.5;
        }
        let self_count = vsa.bindings.iter().filter(|b| b.self_weight >= 0.5).count() as f64;
        let world_count = vsa.bindings.iter().filter(|b| b.self_weight < 0.5).count() as f64;
        if self_count + world_count == 0.0 {
            return 0.5;
        }
        self_count / (self_count + world_count)
    }

    /// Jaccard similarity of tone sets weighted by intensity.
    pub fn qualia_similarity(a: &QualifiedVsa, b: &QualifiedVsa) -> f64 {
        let tones_a: HashMap<QualiaTone, f64> =
            a.bindings.iter().map(|x| (x.tone, x.intensity)).collect();
        let tones_b: HashMap<QualiaTone, f64> =
            b.bindings.iter().map(|x| (x.tone, x.intensity)).collect();

        let mut intersection = 0.0_f64;
        let mut union = 0.0_f64;

        let mut all_tones: Vec<QualiaTone> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for &t in tones_a.keys().chain(tones_b.keys()) {
            if seen.insert(t) {
                all_tones.push(t);
            }
        }

        for t in &all_tones {
            let a_w = tones_a.get(t).copied().unwrap_or(0.0);
            let b_w = tones_b.get(t).copied().unwrap_or(0.0);
            intersection += a_w.min(b_w);
            union += a_w.max(b_w);
        }

        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }

    /// Aggregate tones from recent history within a time window.
    pub fn recent_tone_distribution(&self, window_secs: f64) -> HashMap<QualiaTone, f64> {
        let cutoff = Instant::now() - std::time::Duration::from_secs_f64(window_secs);
        let mut result: HashMap<QualiaTone, f64> = HashMap::new();

        for (tone, intensity, ts) in &self.tone_history {
            if *ts >= cutoff {
                *result.entry(*tone).or_insert(0.0) += intensity;
            }
        }

        let total: f64 = result.values().sum();
        if total > 0.0 {
            for v in result.values_mut() {
                *v /= total;
            }
        }
        result
    }

    /// Convenience: bind with high self_weight (0.9).
    pub fn bind_self_tag(&mut self, vsa: &mut QualifiedVsa, tone: QualiaTone, intensity: f64) {
        self.bind(vsa, tone, intensity, 0.9, "self");
    }

    /// Convenience: bind with low self_weight (0.1).
    pub fn bind_world_tag(&mut self, vsa: &mut QualifiedVsa, tone: QualiaTone, intensity: f64) {
        self.bind(vsa, tone, intensity, 0.1, "world");
    }

    /// LRU-style pruning of oldest cache entries when total exceeds limit.
    pub fn prune(&mut self, max_total_bindings: usize) {
        let total: usize = self.qualia_cache.values().map(|v| v.len()).sum();
        if total <= max_total_bindings {
            return;
        }
        let excess = total - max_total_bindings;
        let mut all: Vec<(Instant, u64, usize)> = Vec::new();
        for (&sig, bindings) in &self.qualia_cache {
            for (i, b) in bindings.iter().enumerate() {
                all.push((b.timestamp, sig, i));
            }
        }
        all.sort_by_key(|k| k.0);
        for (_, sig, _) in all.iter().take(excess) {
            if let Some(bindings) = self.qualia_cache.get_mut(sig) {
                bindings.remove(0);
                if bindings.is_empty() {
                    self.qualia_cache.remove(sig);
                }
            }
        }
    }

    fn vsa_content_signature(vsa: &[f64]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for &v in vsa.iter().take(64) {
            v.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl Default for QualiaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa() -> QualifiedVsa {
        QualifiedVsa::new(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8])
    }

    #[test]
    fn test_bind_and_dominant_tone() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind(&mut vsa, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        gen.bind(&mut vsa, QualiaTone::Pleasant, 0.6, 0.5, "thought");
        gen.bind(&mut vsa, QualiaTone::Uncertain, 0.3, 0.5, "thought");
        assert_eq!(
            QualiaGenerator::dominant_tone(&vsa),
            Some(QualiaTone::Pleasant)
        );
    }

    #[test]
    fn test_self_world_ratio() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind(&mut vsa, QualiaTone::Important, 0.9, 0.9, "self");
        gen.bind(&mut vsa, QualiaTone::Neutral, 0.5, 0.1, "world");
        gen.bind(&mut vsa, QualiaTone::Novel, 0.7, 0.1, "world");
        let ratio = QualiaGenerator::self_world_ratio(&vsa);
        assert!((ratio - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_qualia_similarity_identical() {
        let mut gen = QualiaGenerator::new();
        let mut a = make_vsa();
        let mut b = make_vsa();
        gen.bind(&mut a, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        gen.bind(&mut a, QualiaTone::Important, 0.6, 0.5, "thought");
        gen.bind(&mut b, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        gen.bind(&mut b, QualiaTone::Important, 0.6, 0.5, "thought");
        assert!((QualiaGenerator::qualia_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_qualia_similarity_different() {
        let mut gen = QualiaGenerator::new();
        let mut a = make_vsa();
        let mut b = make_vsa();
        gen.bind(&mut a, QualiaTone::Pleasant, 1.0, 0.5, "thought");
        gen.bind(&mut b, QualiaTone::Unpleasant, 1.0, 0.5, "thought");
        let sim = QualiaGenerator::qualia_similarity(&a, &b);
        assert!(sim < 0.01);
    }

    #[test]
    fn test_recent_tone_distribution() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind(&mut vsa, QualiaTone::Pleasant, 1.0, 0.5, "thought");
        gen.bind(&mut vsa, QualiaTone::Pleasant, 1.0, 0.5, "thought");
        gen.bind(&mut vsa, QualiaTone::Uncertain, 1.0, 0.5, "thought");
        let dist = gen.recent_tone_distribution(60.0);
        assert!((dist[&QualiaTone::Pleasant] - 2.0 / 3.0).abs() < 0.001);
        assert!((dist[&QualiaTone::Uncertain] - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_prune() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind(&mut vsa, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        assert!(gen.qualia_cache.values().any(|v| !v.is_empty()));
        gen.prune(0);
        let total: usize = gen.qualia_cache.values().map(|v| v.len()).sum();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_bind_self_tag() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind_self_tag(&mut vsa, QualiaTone::Important, 0.9);
        assert_eq!(vsa.bindings.len(), 1);
        assert!((vsa.bindings[0].self_weight - 0.9).abs() < 0.01);
        assert_eq!(vsa.bindings[0].modality, "self");
    }

    #[test]
    fn test_qualia_signature_deterministic() {
        let mut gen = QualiaGenerator::new();
        let mut a = make_vsa();
        let mut b = make_vsa();
        gen.bind(&mut a, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        gen.bind(&mut a, QualiaTone::Important, 0.6, 0.5, "thought");
        gen.bind(&mut b, QualiaTone::Pleasant, 0.8, 0.5, "thought");
        gen.bind(&mut b, QualiaTone::Important, 0.6, 0.5, "thought");
        assert_eq!(
            QualiaGenerator::qualia_signature(&a),
            QualiaGenerator::qualia_signature(&b),
        );
    }

    #[test]
    fn test_empty_dominant_tone() {
        let vsa = make_vsa();
        assert_eq!(QualiaGenerator::dominant_tone(&vsa), None);
    }

    #[test]
    fn test_bind_world_tag() {
        let mut gen = QualiaGenerator::new();
        let mut vsa = make_vsa();
        gen.bind_world_tag(&mut vsa, QualiaTone::Novel, 0.7);
        assert_eq!(vsa.bindings.len(), 1);
        assert!((vsa.bindings[0].self_weight - 0.1).abs() < 0.01);
        assert_eq!(vsa.bindings[0].modality, "world");
    }

    #[test]
    fn test_qualia5_default() {
        let q = Qualia5::default();
        assert_eq!(q.interoception, 0.0);
        assert_eq!(q.exteroception, 0.0);
        assert_eq!(q.temporal_binding, 0.0);
        assert_eq!(q.valence, 0.0);
        assert_eq!(q.arousal, 0.3);
    }

    #[test]
    fn test_qualia5_compute_normal() {
        let q = Qualia5::compute(0.7, 0.6, 0.3, 0.5, 0.1);
        assert!((q.interoception - 0.7).abs() < 0.01);
        assert!((q.exteroception - 0.5).abs() < 0.01);
        assert!((q.temporal_binding - 0.6).abs() < 0.01);
        assert!((q.valence - 0.1).abs() < 0.01);
        assert!((q.arousal - (1.0 - 0.3 * 0.7 - 0.1 * 0.3)).abs() < 0.01);
    }

    #[test]
    fn test_qualia5_compute_high_load() {
        let q = Qualia5::compute(0.3, 0.2, 0.9, 0.1, 0.3);
        assert!((q.interoception - 0.1).abs() < 0.01);
        assert!(q.arousal < 0.5);
    }

    #[test]
    fn test_qualia5_is_significant_high_arousal() {
        let q = Qualia5 {
            interoception: 0.3,
            exteroception: 0.3,
            temporal_binding: 0.3,
            valence: 0.0,
            arousal: 0.8,
        };
        assert!(q.is_significant());
    }

    #[test]
    fn test_qualia5_is_significant_negative_valence() {
        let q = Qualia5 {
            interoception: 0.3,
            exteroception: 0.3,
            temporal_binding: 0.3,
            valence: -0.6,
            arousal: 0.3,
        };
        assert!(q.is_significant());
    }

    #[test]
    fn test_qualia5_not_significant() {
        let q = Qualia5 {
            interoception: 0.4,
            exteroception: 0.4,
            temporal_binding: 0.4,
            valence: 0.1,
            arousal: 0.4,
        };
        assert!(!q.is_significant());
    }

    #[test]
    fn test_qualia5_clamp_bounds() {
        let q = Qualia5::compute(2.0, 1.5, -0.5, -0.1, -0.2);
        assert!(q.interoception >= 0.0 && q.interoception <= 1.0);
        assert!(q.exteroception >= 0.0 && q.exteroception <= 1.0);
        assert!(q.temporal_binding >= 0.0 && q.temporal_binding <= 1.0);
        assert!(q.valence >= -1.0 && q.valence <= 1.0);
        assert!(q.arousal >= 0.0 && q.arousal <= 1.0);
    }
}
