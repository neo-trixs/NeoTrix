/// P1.5 Meta-improvement Loop
/// Diagnoses pipeline health every 3 runs and triggers self-modification.
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    pub throughput: f64,     // stages/second
    pub duplicate_rate: f64, // 0.0-1.0
    pub keep_rate: f64,      // 0.0-1.0 (what fraction of outputs are kept)
    pub cycle: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImprovementPattern {
    HighDuplicates, // duplicate_rate > 0.3
    LowActivation,  // throughput < 0.5 * baseline
    LowKeepRate,    // keep_rate < 0.2
    Normal,
}

#[derive(Debug)]
pub struct MetaImprovementLoop {
    pub metrics_history: VecDeque<PipelineMetrics>,
    pub improvements: Vec<ImprovementAction>,
    pub cycle: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ImprovementAction {
    pub pattern: ImprovementPattern,
    pub applied_at: u64,
    pub description: String,
    pub revertible: bool,
}

impl MetaImprovementLoop {
    pub fn new() -> Self {
        MetaImprovementLoop {
            metrics_history: VecDeque::with_capacity(100),
            improvements: Vec::new(),
            cycle: 0,
            enabled: true,
        }
    }

    /// Called every pipeline run
    pub fn record_metrics(&mut self, metrics: PipelineMetrics) -> Option<ImprovementAction> {
        self.metrics_history.push_back(metrics);
        if self.metrics_history.len() > 100 {
            self.metrics_history.pop_front();
        }
        self.cycle += 1;

        // Diagnose every 3 runs
        if self.cycle % 3 != 0 || !self.enabled {
            return None;
        }

        self.diagnose_and_improve()
    }

    fn diagnose_and_improve(&mut self) -> Option<ImprovementAction> {
        if self.metrics_history.len() < 3 {
            return None;
        }

        let recent: Vec<_> = self.metrics_history.iter().rev().take(3).collect();
        let avg_dup: f64 = recent.iter().map(|m| m.duplicate_rate).sum::<f64>() / 3.0;
        let avg_keep: f64 = recent.iter().map(|m| m.keep_rate).sum::<f64>() / 3.0;
        let avg_throughput: f64 = recent.iter().map(|m| m.throughput).sum::<f64>() / 3.0;

        // Pattern matching
        let pattern = if avg_dup > 0.3 {
            ImprovementPattern::HighDuplicates
        } else if avg_throughput < 0.5 {
            // baseline assumed normalized
            ImprovementPattern::LowActivation
        } else if avg_keep < 0.2 {
            ImprovementPattern::LowKeepRate
        } else {
            ImprovementPattern::Normal
        };

        if pattern == ImprovementPattern::Normal {
            return None;
        }

        let action = ImprovementAction {
            pattern: pattern.clone(),
            applied_at: self.cycle,
            description: match &pattern {
                ImprovementPattern::HighDuplicates => {
                    "Reducing bundle frequency, enabling dedup filter".into()
                }
                ImprovementPattern::LowActivation => {
                    "Increasing sampling rate, lowering similarity threshold".into()
                }
                ImprovementPattern::LowKeepRate => {
                    "Adjusting output quality gate, raising keep threshold".into()
                }
                ImprovementPattern::Normal => "No improvement needed".into(),
            },
            revertible: true,
        };

        self.improvements.push(action.clone());
        Some(action)
    }

    pub fn kpi_summary(&self) -> String {
        if self.metrics_history.is_empty() {
            return "No metrics recorded yet".into();
        }
        let last = self.metrics_history.back().unwrap();
        format!(
            "MetaImprovement | cycle={} throughput={:.2} dup={:.2} keep={:.2} improvements={}",
            self.cycle,
            last.throughput,
            last.duplicate_rate,
            last.keep_rate,
            self.improvements.len()
        )
    }

    pub fn kpi_ring_buffer(&self) -> Vec<PipelineMetrics> {
        self.metrics_history.iter().cloned().collect()
    }
}
