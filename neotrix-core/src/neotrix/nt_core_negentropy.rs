use crate::core::nt_core_negentropy::{
    NegentropyComponents, NegentropyFlux, NegentropyMetric, NegentropyReport,
};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use crate::core::nt_core_gwt::resonance::{compute_semantic_entropy, MODULE_COUNT};
use crate::core::nt_core_consciousness::ConsciousnessStream;
use crate::core::ReasoningHexagram;
use crate::neotrix::nt_core_iit_phi::IITPhiCalculator;
use crate::neotrix::nt_world_jepa::JepaWorldModel;
use crate::neotrix::nt_memory_kb::KnowledgeBase;

pub struct NegentropyCalculator {
    pub metric: NegentropyMetric,
    pub phi_calc: IITPhiCalculator,
    pub prev_total: f64,
    pub prev_import_ts: f64,
    pub prev_export_count: usize,
    pub operational_cost_ma: f64,
}

impl Default for NegentropyCalculator {
    fn default() -> Self {
        Self {
            metric: NegentropyMetric::default(),
            phi_calc: IITPhiCalculator::new(),
            prev_total: 0.0,
            prev_import_ts: 0.0,
            prev_export_count: 0,
            operational_cost_ma: 1.0,
        }
    }
}

impl NegentropyCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 传感器 #1: N_Φ = IIT integrated information
    pub fn sensor_phi(state: &[f64], calc: &mut IITPhiCalculator) -> f64 {
        let report = calc.compute_phi(state);
        calc.record(report.phi);
        report.phi
    }

    /// 传感器 #2: N_VSA = VSA space ordering
    /// Measures how structured the VSA space is vs random baseline.
    /// Structured space: similar concepts cluster → lower pairwise distances among neighbors.
    pub fn sensor_vsa_coherence(vsa_pool: &[Vec<u8>]) -> f64 {
        if vsa_pool.len() < 5 {
            return 0.5;
        }
        let sample_size = vsa_pool.len().min(50);
        let mut total_dist: f64 = 0.0;
        let mut count: usize = 0;
        for i in 0..sample_size {
            for j in (i + 1)..sample_size {
                let d = QuantizedVSA::hamming_distance(&vsa_pool[i], &vsa_pool[j]);
                total_dist += d as f64;
                count += 1;
            }
        }
        if count == 0 {
            return 0.5;
        }
        let mean_dist = total_dist / count as f64;
        let expected_random: f64 = 0.5;
        1.0 - (mean_dist / expected_random).clamp(0.0, 1.0)
    }

    /// 传感器 #3: N_KB = knowledge graph structural order
    pub fn sensor_kb_order(kb: Option<&KnowledgeBase>) -> f64 {
        let Some(kb) = kb else { return 0.0 };
        let stats = kb.stats().unwrap_or_default();
        let nodes = stats.total_nodes as f64;
        let edges = stats.total_edges as f64;
        if nodes < 2.0 {
            return 0.0;
        }
        let ratio = edges / nodes;
        let optimal: f64 = 4.0;
        let ratio_score = 1.0 - (ratio - optimal).abs() / optimal.max(ratio);
        let ratio_score: f64 = ratio_score.clamp(0.0, 1.0);

        let deg_entropy: f64 = 1.0;
        let entropy_score = 1.0 - deg_entropy.clamp(0.0, 1.0);

        0.6 * ratio_score + 0.4 * entropy_score
    }

    /// 传感器 #4: N_JEPA = prediction accuracy with closed-loop error feedback
    pub fn sensor_prediction_acc(jepa: Option<&JepaWorldModel>, jepa_error: Option<f64>) -> f64 {
        let Some(jepa) = jepa else { return 0.5 };
        let stability = if jepa.check_rollout_stability() { 0.8 } else { 0.3 };
        match jepa_error {
            Some(err) => {
                let error_penalty = (err * 2.0).clamp(0.0, 1.0);
                stability * (1.0 - error_penalty * 0.5)
            }
            None => stability,
        }
    }

    /// 传感器 #5: N_Attn = GWT attention focus
    /// Focused attention = low entropy of specialist distribution
    pub fn sensor_attention_focus() -> f64 {
        let entropy = compute_semantic_entropy(&[0.0_f64; MODULE_COUNT]);
        let max_entropy: f64 = (64.0_f64).ln();
        1.0 - (entropy / max_entropy).clamp(0.0, 1.0)
    }

    /// 传感器 #6: N_E8 = strategy differentiation
    /// Diverse strategy matrix = low pairwise similarity
    pub fn sensor_strategy_diff(matrix: &[[ReasoningHexagram; 8]; 8]) -> f64 {
        let mut total_sim: f64 = 0.0;
        let mut count: usize = 0;
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    for l in 0..8 {
                        if (i, j) >= (k, l) {
                            continue;
                        }
                        let a = &matrix[i][j];
                        let b = &matrix[k][l];
                        let sim = a.resonance_strength(b) as f64 / 6.0;
                        total_sim += sim;
                        count += 1;
                    }
                }
            }
        }
        if count == 0 {
            return 0.5;
        }
        let mean_sim = total_sim / count as f64;
        1.0 - mean_sim.clamp(0.0, 1.0)
    }

    /// 传感器 #7: N_Stream = temporal coherence of consciousness stream
    pub fn sensor_temporal_coherence(stream: &ConsciousnessStream) -> f64 {
        let n = stream.len();
        if n == 0 {
            return 0.5;
        }
        let recency: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();
        let mean = recency.iter().copied().sum::<f64>() / recency.len() as f64;
        let variance = recency.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recency.len() as f64;
        let cv: f64 = variance.sqrt() / mean.max(0.01);
        (1.0 - (cv / 2.0).clamp(0.0, 1.0)).max(0.0)
    }

    pub fn compute_flux(
        &mut self,
        import_rate: f64,
        export_count: usize,
        operational_cost: f64,
    ) -> NegentropyFlux {
        let export = export_count.saturating_sub(self.prev_export_count) as f64;
        self.prev_export_count = export_count;

        self.operational_cost_ma = self.operational_cost_ma * 0.9 + operational_cost * 0.1;

        let efficiency = if self.operational_cost_ma > 0.01 {
            import_rate / self.operational_cost_ma
        } else {
            0.0
        };

        NegentropyFlux {
            import_rate,
            export_rate: export.max(0.0),
            net_flux: import_rate - export.max(0.0),
            efficiency,
            operational_cost: self.operational_cost_ma,
        }
    }

    pub fn compute_full(
        &mut self,
        state: &[f64],
        vsa_pool: &[Vec<u8>],
        kb: Option<&KnowledgeBase>,
        jepa: Option<&JepaWorldModel>,
        matrix: &[[ReasoningHexagram; 8]; 8],
        stream: &ConsciousnessStream,
        import_rate: f64,
        export_count: usize,
        operational_cost: f64,
    ) -> NegentropyReport {
        self.compute_full_with_jepa_error(state, vsa_pool, kb, jepa, None, matrix, stream, import_rate, export_count, operational_cost)
    }

    pub fn compute_full_with_jepa_error(
        &mut self,
        state: &[f64],
        vsa_pool: &[Vec<u8>],
        kb: Option<&KnowledgeBase>,
        jepa: Option<&JepaWorldModel>,
        jepa_error: Option<f64>,
        matrix: &[[ReasoningHexagram; 8]; 8],
        stream: &ConsciousnessStream,
        import_rate: f64,
        export_count: usize,
        operational_cost: f64,
    ) -> NegentropyReport {
        let components = NegentropyComponents {
            phi: Self::sensor_phi(state, &mut self.phi_calc),
            vsa_coherence: Self::sensor_vsa_coherence(vsa_pool),
            kb_order: Self::sensor_kb_order(kb),
            prediction_acc: Self::sensor_prediction_acc(jepa, jepa_error),
            attention_focus: Self::sensor_attention_focus(),
            strategy_diff: Self::sensor_strategy_diff(matrix),
            temporal_coherence: Self::sensor_temporal_coherence(stream),
        };

        let flux = self.compute_flux(import_rate, export_count, operational_cost);
        self.metric.record(components, flux);

        let report = NegentropyReport::new(
            self.metric.history.len() as u64,
            &self.metric,
            self.prev_total,
        );
        self.prev_total = self.metric.total;
        report
    }

    pub fn compute_kb_only(&mut self, nodes: f64, edges: f64, round_recs: usize, round_time: f64) -> NegentropyReport {
        let ratio = if nodes > 1.0 { edges / nodes } else { 0.0 };
        let optimal = 4.0;
        let ratio_score = 1.0 - (ratio - optimal).abs() / optimal.max(ratio);
        let ratio_score = ratio_score.clamp(0.0, 1.0);

        let import_rate = if round_time > 0.0 { round_recs as f64 / round_time } else { 0.0 };

        let components = NegentropyComponents {
            phi: 0.0,
            vsa_coherence: 0.0,
            kb_order: ratio_score,
            prediction_acc: 0.0,
            attention_focus: 0.0,
            strategy_diff: 0.0,
            temporal_coherence: 0.0,
        };

        let flux = self.compute_flux(import_rate, 0, round_time.max(0.1));
        self.metric.record(components, flux);

        let report = NegentropyReport::new(
            self.metric.history.len() as u64,
            &self.metric,
            self.prev_total,
        );
        self.prev_total = self.metric.total;
        report
    }
}
