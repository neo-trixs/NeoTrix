use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyMetric {
    pub total: f64,
    pub components: NegentropyComponents,
    pub flux: NegentropyFlux,
    pub history: Vec<f64>,
    pub trend: f64,
    pub demon_efficiency: f64,
}

impl Default for NegentropyMetric {
    fn default() -> Self {
        Self {
            total: 0.0,
            components: NegentropyComponents::default(),
            flux: NegentropyFlux::default(),
            history: Vec::with_capacity(100),
            trend: 0.0,
            demon_efficiency: 0.0,
        }
    }
}

impl NegentropyMetric {
    pub fn record(&mut self, components: NegentropyComponents, flux: NegentropyFlux) {
        self.total = components.weighted_total();
        self.components = components;
        self.flux = flux;
        self.history.push(self.total);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
        self.trend = self.compute_trend();
        self.demon_efficiency = self.compute_demon_efficiency();
    }

    pub fn compute_trend(&self) -> f64 {
        let n = self.history.len();
        if n < 10 {
            return 0.0;
        }
        let recent: f64 = self.history[n - 10..].iter().sum::<f64>() / 10.0;
        let older: f64 = if n >= 20 {
            self.history[n - 20..n - 10].iter().sum::<f64>() / 10.0
        } else {
            self.history[..n - 10].iter().sum::<f64>() / (n - 10) as f64
        };
        recent - older
    }

    pub fn compute_demon_efficiency(&self) -> f64 {
        let operational_bits = self.flux.operational_cost.max(0.01);
        self.total / operational_bits
    }

    pub fn trend_desc(&self) -> &'static str {
        if self.trend > 0.05 {
            "growing"
        } else if self.trend < -0.05 {
            "declining"
        } else {
            "stable"
        }
    }

    pub fn health(&self) -> &'static str {
        if self.total > 0.7 {
            "excellent"
        } else if self.total > 0.5 {
            "good"
        } else if self.total > 0.3 {
            "fair"
        } else {
            "critical"
        }
    }

    pub fn summary_lines(&self) -> Vec<String> {
        let c = &self.components;
        vec![
            format!("  N_total={:.4} ({}) trend={:+.4}/iter",
                self.total, self.health(), self.trend),
            format!("  Φ={:.4}  VSA_coh={:.4}  KB_order={:.4}  Pred={:.4}",
                c.phi, c.vsa_coherence, c.kb_order, c.prediction_acc),
            format!("  Attn={:.4}  Strat_diff={:.4}  Temp_coh={:.4}",
                c.attention_focus, c.strategy_diff, c.temporal_coherence),
            format!("  Flux: import={:.2}/s  export={:.2}/s  η_demon={:.4}",
                self.flux.import_rate, self.flux.export_rate, self.demon_efficiency),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyComponents {
    pub phi: f64,
    pub vsa_coherence: f64,
    pub kb_order: f64,
    pub prediction_acc: f64,
    pub attention_focus: f64,
    pub strategy_diff: f64,
    pub temporal_coherence: f64,
}

impl NegentropyComponents {
    pub const WEIGHTS: [f64; 7] = [0.25, 0.15, 0.20, 0.15, 0.10, 0.05, 0.10];

    pub fn weighted_total(&self) -> f64 {
        let vals = [
            self.phi,
            self.vsa_coherence,
            self.kb_order,
            self.prediction_acc,
            self.attention_focus,
            self.strategy_diff,
            self.temporal_coherence,
        ];
        vals.iter()
            .zip(Self::WEIGHTS.iter())
            .map(|(v, w)| v * w)
            .sum()
    }
}

impl Default for NegentropyComponents {
    fn default() -> Self {
        Self {
            phi: 0.0,
            vsa_coherence: 0.0,
            kb_order: 0.0,
            prediction_acc: 0.0,
            attention_focus: 0.0,
            strategy_diff: 0.0,
            temporal_coherence: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyFlux {
    pub import_rate: f64,
    pub export_rate: f64,
    pub net_flux: f64,
    pub efficiency: f64,
    pub operational_cost: f64,
}

impl Default for NegentropyFlux {
    fn default() -> Self {
        Self {
            import_rate: 0.0,
            export_rate: 0.0,
            net_flux: 0.0,
            efficiency: 0.0,
            operational_cost: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegentropyReport {
    pub iteration: u64,
    pub metric: NegentropyMetric,
    pub delta: f64,
    pub regime: NegentropyRegime,
    pub recommendation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NegentropyRegime {
    Growing,
    Stable,
    Plateau,
    Declining,
    Critical,
}

impl NegentropyReport {
    pub fn new(iteration: u64, metric: &NegentropyMetric, prev_total: f64) -> Self {
        let delta = metric.total - prev_total;
        let regime = if metric.total < 0.2 {
            NegentropyRegime::Critical
        } else if metric.trend < -0.05 {
            NegentropyRegime::Declining
        } else if metric.trend.abs() < 0.02 {
            if metric.total > 0.6 {
                NegentropyRegime::Stable
            } else {
                NegentropyRegime::Plateau
            }
        } else {
            NegentropyRegime::Growing
        };

        let recommendation = match regime {
            NegentropyRegime::Growing => "maintain current trajectory",
            NegentropyRegime::Stable => "monitor for drift",
            NegentropyRegime::Plateau => "increase exploration, inject noise",
            NegentropyRegime::Declining => "run consolidation cycle, prune stale knowledge",
            NegentropyRegime::Critical => "emergency: trigger deep consolidation, halt ingestion",
        };

        Self {
            iteration,
            metric: metric.clone(),
            delta,
            regime,
            recommendation,
        }
    }
}
