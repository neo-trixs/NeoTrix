pub use super::brain_dgm::EditCritic;
use super::cosine_distance;
use super::HyperAgentArchive;
use super::HyperAgentRecord;
use super::ModificationTarget;
use super::SafetyCheckResult;
use super::SelfModificationProposal;
use crate::core::nt_core_hcube::vsa::VsaBackend;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Continuous-space latent edit: a perturbation vector applied to the capability
/// latent space, rather than a discrete FileDiff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentEdit {
    pub delta: Vec<f64>,
    pub extension_delta: Vec<(String, f64)>,
    pub generative_entropy: f64,
    pub hypervector: Option<Vec<f64>>,
}

impl LatentEdit {
    pub fn new(delta: Vec<f64>) -> Self {
        let entropy = Self::compute_entropy(&delta);
        Self {
            delta,
            extension_delta: Vec::new(),
            generative_entropy: entropy,
            hypervector: None,
        }
    }

    fn compute_entropy(v: &[f64]) -> f64 {
        let sum_abs: f64 = v.iter().map(|x| x.abs()).sum::<f64>() + 1e-12;
        v.iter()
            .map(|x| {
                let p = x.abs() / sum_abs;
                if p > 1e-12 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum()
    }

    pub fn magnitude(&self) -> f64 {
        self.delta.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    pub fn apply_to(&self, base: &[f64]) -> Vec<f64> {
        let len = base.len().min(self.delta.len());
        let mut result = base.to_vec();
        for i in 0..len {
            result[i] += self.delta[i];
        }
        result
    }
}

/// DGM-Hyperagent: a meta-agent that uses a generative model (approximated by
/// VSA hypervectors) to propose latent edits instead of rule-based strategies.
#[derive(Debug)]
pub struct DGMMetaAgent {
    pub vsa_dim: usize,
    pub top_k: usize,
    pub mutation_scale: f64,
    pub max_entropy: f64,
    pub seed: u64,
}

impl DGMMetaAgent {
    pub fn new(vsa_dim: usize, top_k: usize, mutation_scale: f64) -> Self {
        Self {
            vsa_dim,
            top_k,
            mutation_scale,
            max_entropy: 4.0,
            seed: 42,
        }
    }

    pub fn generate_edit(&self, archive: &HyperAgentArchive) -> LatentEdit {
        let engine = crate::core::nt_core_hcube::vsa::VSAEngine::new(self.vsa_dim);
        let mut rng = rand::thread_rng();

        let parents = self.select_top_k(archive);

        if parents.is_empty() {
            let delta: Vec<f64> = (0..self.vsa_dim)
                .map(|_| (rng.gen::<f64>() - 0.5) * 2.0 * self.mutation_scale)
                .collect();
            let mut edit = LatentEdit::new(delta);
            edit.generative_entropy = self.vsa_dim as f64 * 0.5;
            return edit;
        }

        let parent_hvs: Vec<Vec<f64>> = parents
            .iter()
            .map(|r| {
                let mut hv = self.encode_latent(&r.latent_snapshot);
                let norm: f64 = hv.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
                for x in &mut hv {
                    *x /= norm;
                }
                hv
            })
            .collect();

        let hv_refs: Vec<&[f64]> = parent_hvs.iter().map(|v| v.as_slice()).collect();
        let mut style = engine.bundle(&hv_refs);
        let style_norm: f64 = style.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        for x in &mut style {
            *x /= style_norm;
        }

        let shift: isize = rng.gen_range(1..self.vsa_dim as isize);
        let innovation = engine.permute(&style, shift);

        let mut context: Vec<f64> = (0..self.vsa_dim)
            .map(|i| (i as f64 * PI).sin() * rng.gen::<f64>())
            .collect();
        let ctx_norm: f64 = context.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        for x in &mut context {
            *x /= ctx_norm;
        }

        let bound = engine.bind(&innovation, &context);

        let target_dim = parents[0].latent_snapshot.len();
        let mut delta: Vec<f64> = bound.iter().take(target_dim).copied().collect();

        let noise_scale = 0.1;
        for d in &mut delta {
            *d = *d * self.mutation_scale + (rng.gen::<f64>() - 0.5) * 2.0 * noise_scale;
        }

        let mut edit = LatentEdit::new(delta);
        edit.hypervector = Some(bound);
        edit
    }

    pub fn proposal_from_edit(
        &self,
        edit: &LatentEdit,
        archive: &HyperAgentArchive,
    ) -> SelfModificationProposal {
        let entropy = edit.generative_entropy;
        let magnitude = edit.magnitude();

        let target = if entropy > self.max_entropy * 0.8 {
            ModificationTarget::CapabilityExtension
        } else if magnitude > self.mutation_scale {
            ModificationTarget::ImprovementMechanism
        } else if archive.best_score().map_or(true, |s| s < 0.6) {
            ModificationTarget::MetaAgent
        } else {
            ModificationTarget::TaskAgent
        };

        let impact = format!(
            "dgm_h: entropy={:.3} magnitude={:.3} parents={} archive_size={}",
            entropy,
            magnitude,
            self.top_k,
            archive.len()
        );

        SelfModificationProposal {
            target,
            diffs: Vec::new(),
            expected_impact: impact,
            safety_check: SafetyCheckResult::Passed,
        }
    }

    pub fn select_top_k<'a>(&self, archive: &'a HyperAgentArchive) -> Vec<&'a HyperAgentRecord> {
        let mut scored: Vec<&HyperAgentRecord> = archive.records.iter().collect();
        scored.sort_by(|a, b| {
            let sa = a.score.unwrap_or(0.0);
            let sb = b.score.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(self.top_k);
        scored
    }

    fn encode_latent(&self, latent: &[f64]) -> Vec<f64> {
        if latent.len() >= self.vsa_dim {
            latent[..self.vsa_dim].to_vec()
        } else {
            let mut hv = latent.to_vec();
            hv.resize(self.vsa_dim, 0.0);
            hv
        }
    }
}

/// Generative replay buffer: learns a generative distribution over the archive's
/// successful modifications and synthesizes new candidates via interpolation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerativeReplay {
    pub num_components: usize,
    pub min_score: f64,
    pub max_samples: usize,
    pub enabled: bool,
}

impl Default for GenerativeReplay {
    fn default() -> Self {
        Self {
            num_components: 3,
            min_score: 0.5,
            max_samples: 5,
            enabled: true,
        }
    }
}

impl GenerativeReplay {
    pub fn new(num_components: usize, min_score: f64) -> Self {
        Self {
            num_components,
            min_score,
            max_samples: 5,
            enabled: true,
        }
    }

    pub fn replay(&self, archive: &HyperAgentArchive) -> Vec<Vec<f64>> {
        if !self.enabled || archive.records.is_empty() {
            return Vec::new();
        }

        let mut rng = rand::thread_rng();
        let high_scorers: Vec<&HyperAgentRecord> = archive
            .records
            .iter()
            .filter(|r| r.score.map_or(false, |s| s >= self.min_score))
            .collect();

        if high_scorers.len() < 2 {
            return Vec::new();
        }

        let dim = high_scorers[0].latent_snapshot.len();
        let mut samples = Vec::new();
        let target_count = self.max_samples.min(high_scorers.len() * 2);

        for _ in 0..target_count {
            let i = rng.gen_range(0..high_scorers.len());
            let j = rng.gen_range(0..high_scorers.len());
            if i == j {
                continue;
            }

            let p1 = &high_scorers[i].latent_snapshot;
            let p2 = &high_scorers[j].latent_snapshot;
            let alpha: f64 = rng.gen();
            let noise_std = 0.05;

            let mut latent: Vec<f64> = p1
                .iter()
                .zip(p2.iter())
                .map(|(a, b)| {
                    alpha * a + (1.0 - alpha) * b + rng.gen::<f64>() * noise_std * 2.0 - noise_std
                })
                .collect();

            latent.truncate(dim);
            while latent.len() < dim {
                latent.push(rng.gen::<f64>() * 0.1);
            }

            samples.push(latent);
        }

        samples
    }
}

/// Result of a self-referential consistency check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelfRefCheckResult {
    Stable,
    Destabilizing { metric: f64, threshold: f64 },
    Impairing { expected_loss: f64 },
}

pub struct SelfReferentialCheck {
    pub max_distortion_ratio: f64,
    pub max_spectral_growth: f64,
    pub min_self_consistency: f64,
}

impl Default for SelfReferentialCheck {
    fn default() -> Self {
        Self {
            max_distortion_ratio: 3.0,
            max_spectral_growth: 1.5,
            min_self_consistency: 0.7,
        }
    }
}

impl SelfReferentialCheck {
    pub fn new(max_distortion: f64, max_spectral: f64, min_self: f64) -> Self {
        Self {
            max_distortion_ratio: max_distortion,
            max_spectral_growth: max_spectral,
            min_self_consistency: min_self,
        }
    }

    pub fn check(
        &self,
        edit: &LatentEdit,
        archive: &HyperAgentArchive,
        current_latent: &[f64],
    ) -> SelfRefCheckResult {
        if archive.records.is_empty() {
            return SelfRefCheckResult::Stable;
        }

        let dim = archive.records[0].latent_snapshot.len();
        if dim == 0 {
            return SelfRefCheckResult::Stable;
        }

        let new_latent = edit.apply_to(current_latent);

        let existing_nn_dist = archive
            .records
            .iter()
            .map(|r| cosine_distance_to_euclidean(current_latent, &r.latent_snapshot))
            .fold(f64::MAX, f64::min);

        let new_nn_dist = archive
            .records
            .iter()
            .map(|r| cosine_distance_to_euclidean(&new_latent, &r.latent_snapshot))
            .fold(f64::MAX, f64::min);

        let distortion_ratio = if existing_nn_dist < 1e-12 {
            new_nn_dist / (existing_nn_dist + 1e-12)
        } else {
            new_nn_dist / existing_nn_dist
        };

        if distortion_ratio > self.max_distortion_ratio {
            return SelfRefCheckResult::Destabilizing {
                metric: distortion_ratio,
                threshold: self.max_distortion_ratio,
            };
        }

        let current_spectral = self.approximate_spectral_radius(archive, current_latent);
        let new_spectral = self.approximate_spectral_radius(archive, &new_latent);
        if current_spectral > 1e-12 && new_spectral > current_spectral * self.max_spectral_growth {
            return SelfRefCheckResult::Destabilizing {
                metric: new_spectral / current_spectral.max(1e-12),
                threshold: self.max_spectral_growth,
            };
        }

        let cos_sim = cosine_similarity(current_latent, &new_latent);
        let centroid = archive_centroid(archive, dim);
        let centroid_sim_before = cosine_similarity(current_latent, &centroid);
        let centroid_sim_after = cosine_similarity(&new_latent, &centroid);
        let consistency_loss = (centroid_sim_before - centroid_sim_after).abs();

        if cos_sim < self.min_self_consistency && consistency_loss > 0.3 {
            return SelfRefCheckResult::Impairing {
                expected_loss: consistency_loss,
            };
        }

        SelfRefCheckResult::Stable
    }

    fn approximate_spectral_radius(&self, archive: &HyperAgentArchive, query: &[f64]) -> f64 {
        let records: Vec<&[f64]> = archive
            .records
            .iter()
            .map(|r| r.latent_snapshot.as_slice())
            .collect();
        if records.is_empty() || query.is_empty() {
            return 0.0;
        }

        let dim = query.len();
        let mut v = query.to_vec();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
        for x in &mut v {
            *x /= norm;
        }

        let n = records.len() as f64;
        for _ in 0..3 {
            let mut v_new = vec![0.0; dim];
            for latent in &records {
                let dot: f64 = latent.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                for (vni, l) in v_new.iter_mut().zip(latent.iter()) {
                    *vni += dot * l / n;
                }
            }
            let nrm: f64 = v_new.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
            if nrm < 1e-12 {
                break;
            }
            for x in &mut v {
                *x = 0.0;
            }
            for (vi, vni) in v.iter_mut().zip(v_new.iter()) {
                *vi = vni / nrm;
            }
        }

        let mut av = vec![0.0; dim];
        for latent in &records {
            let dot: f64 = latent.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            for (avi, l) in av.iter_mut().zip(latent.iter()) {
                *avi += dot * l / n;
            }
        }
        let eigenvalue: f64 = av.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        eigenvalue.abs()
    }
}

fn cosine_distance_to_euclidean(a: &[f64], b: &[f64]) -> f64 {
    let cd = cosine_distance(a, b);
    let val: f64 = 2.0 * cd;
    val.sqrt()
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dim = a.len().min(b.len());
    if dim == 0 {
        return 0.0;
    }
    let dot: f64 = (0..dim).map(|i| a[i] * b[i]).sum();
    let na: f64 = (0..dim).map(|i| a[i] * a[i]).sum::<f64>().sqrt();
    let nb: f64 = (0..dim).map(|i| b[i] * b[i]).sum::<f64>().sqrt();
    if na < 1e-12 || nb < 1e-12 {
        return 0.0;
    }
    (dot / (na * nb)).clamp(-1.0, 1.0)
}

fn archive_centroid(archive: &HyperAgentArchive, dim: usize) -> Vec<f64> {
    if archive.records.is_empty() || dim == 0 {
        return vec![0.0; dim];
    }
    let n = archive.records.len() as f64;
    let mut centroid = vec![0.0; dim];
    for record in &archive.records {
        let len = record.latent_snapshot.len().min(dim);
        for i in 0..len {
            centroid[i] += record.latent_snapshot[i] / n;
        }
    }
    centroid
}
