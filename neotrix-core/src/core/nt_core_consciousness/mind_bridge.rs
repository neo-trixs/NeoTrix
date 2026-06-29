use neotrix_mind::distillation::{
    capture::{CaptureBuffer, CapturedInteraction},
    cross_model_distiller::{CrossModelDistiller, DistillationReport},
};
use neotrix_mind::evolution::self_harness::SelfHarnessLoop;
use neotrix_mind::metacognition::calibrator::ConfidenceCalibrator;
use neotrix_mind::metacognition::cognitive_load::CognitiveLoadMonitor;
use neotrix_mind::metacognition::curiosity::CuriosityEngine;
use neotrix_mind::perception::dci_retriever::{DciResult, DciRetriever};
use neotrix_mind::reasoning::misalignment_probe::{
    MisalignmentIndicator, MisalignmentProbe, ProbeObservation,
};
use neotrix_mind::scheduler::cycle_registry::{
    CycleNode, CycleRegistry, CycleStep as RegistryStep,
};
use neotrix_mind::traits::ToolExecutor;
pub use neotrix_mind::traits::ToolExecutor as MindToolExecutor;
use std::sync::{Arc, Mutex};

pub struct MindBridge {
    pub dci_retriever: Option<DciRetriever>,
    pub calibrator: Option<ConfidenceCalibrator>,
    pub curiosity: Option<CuriosityEngine>,
    pub cognitive_load: Option<CognitiveLoadMonitor>,
    pub misalignment_probe: Option<MisalignmentProbe>,
    pub self_harness: Option<SelfHarnessLoop>,
    pub cycle_registry: Option<CycleRegistry>,
    // Cross-model distillation system
    pub distillation_buffer: Option<Arc<Mutex<CaptureBuffer>>>,
    pub cross_model_distiller: Option<CrossModelDistiller>,
    pub last_distillation_report: Option<DistillationReport>,
}

impl Clone for MindBridge {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl MindBridge {
    pub fn new() -> Self {
        let buffer = Arc::new(Mutex::new(CaptureBuffer::new(1000)));
        let distiller = CrossModelDistiller::new(buffer.clone());
        Self {
            dci_retriever: Some(DciRetriever::new()),
            calibrator: Some(ConfidenceCalibrator::new()),
            curiosity: Some(CuriosityEngine::new()),
            cognitive_load: Some(CognitiveLoadMonitor::new()),
            misalignment_probe: Some(MisalignmentProbe::new()),
            self_harness: Some(SelfHarnessLoop::new()),
            cycle_registry: Some(CycleRegistry::new()),
            distillation_buffer: Some(buffer),
            cross_model_distiller: Some(distiller),
            last_distillation_report: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            dci_retriever: None,
            calibrator: None,
            curiosity: None,
            cognitive_load: None,
            misalignment_probe: None,
            self_harness: None,
            cycle_registry: None,
            distillation_buffer: None,
            cross_model_distiller: None,
            last_distillation_report: None,
        }
    }

    pub fn step_gather_dci(
        &mut self,
        concept: Option<&str>,
        executor: &dyn ToolExecutor,
    ) -> Vec<DciResult> {
        self.dci_retriever
            .as_mut()
            .map(|r| {
                if let Some(c) = concept {
                    r.retrieve_deep(c, executor)
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default()
    }

    pub fn step_gather_curiosity(&mut self, prediction_error: f64, novelty: f64, timestamp: u64) {
        if let Some(ref mut c) = self.curiosity {
            c.observe(prediction_error, novelty, timestamp);
        }
    }

    pub fn step_gather_load_begin(&mut self) {
        if let Some(ref mut cl) = self.cognitive_load {
            cl.begin_cycle();
        }
    }

    pub fn step_gather_load_end(&mut self, elapsed_ms: u64) {
        if let Some(ref mut cl) = self.cognitive_load {
            cl.end_cycle(elapsed_ms);
        }
    }

    pub fn step_judge_calibrate(
        &mut self,
        prediction: f64,
        actual: f64,
        confidence: f64,
        timestamp: u64,
    ) {
        if let Some(ref mut c) = self.calibrator {
            c.record(prediction, actual, confidence, timestamp);
        }
    }

    pub fn calibrated_confidence(&self, raw: f64) -> f64 {
        self.calibrator
            .as_ref()
            .map(|c| c.calibrate(raw))
            .unwrap_or(raw)
    }

    pub fn step_meta_probe(
        &mut self,
        activations: Vec<(MisalignmentIndicator, f64)>,
    ) -> Vec<ProbeObservation> {
        self.misalignment_probe
            .as_mut()
            .map(|p| p.observe(activations))
            .unwrap_or_default()
    }

    pub fn step_meta_evolve(&mut self, traces: Vec<String>) {
        if let Some(ref mut h) = self.self_harness {
            h.closed_loop(traces);
        }
    }

    pub fn register_cycle_node(&mut self, name: &'static str, step: RegistryStep, priority: usize) {
        if let Some(ref mut r) = self.cycle_registry {
            r.register(CycleNode {
                name,
                step,
                priority,
                enabled: true,
                call_count: 0,
            });
        }
    }

    pub fn curiosity_free_energy(&self) -> f64 {
        self.curiosity
            .as_ref()
            .map(|c| c.free_energy())
            .unwrap_or(0.0)
    }

    pub fn curiosity_urge(&self) -> f64 {
        self.curiosity
            .as_ref()
            .map(|c| c.exploration_urge())
            .unwrap_or(0.0)
    }

    pub fn cognitive_load(&self) -> f64 {
        self.cognitive_load
            .as_ref()
            .map(|c| c.load())
            .unwrap_or(0.0)
    }

    pub fn should_throttle(&self) -> bool {
        self.cognitive_load
            .as_ref()
            .map(|c| c.should_throttle())
            .unwrap_or(false)
    }

    pub fn misalignment_risk(&self) -> f64 {
        self.misalignment_probe
            .as_ref()
            .map(|p| p.risk_score())
            .unwrap_or(0.0)
    }

    pub fn self_harness_success_rate(&self) -> f64 {
        self.self_harness
            .as_ref()
            .map(|h| h.success_rate())
            .unwrap_or(0.0)
    }

    pub fn registered_nodes(&self) -> usize {
        self.cycle_registry
            .as_ref()
            .map(|r| r.registered_count())
            .unwrap_or(0)
    }

    /// Capture an LLM interaction for distillation
    pub fn capture_interaction(&mut self, interaction: CapturedInteraction) {
        if let Some(ref buffer) = self.distillation_buffer {
            if let Ok(mut buf) = buffer.lock() {
                buf.push(interaction);
            }
        }
    }

    /// Run cross-model distillation
    pub fn run_distillation(&mut self) -> Option<DistillationReport> {
        if let Some(ref mut distiller) = self.cross_model_distiller {
            let report = distiller.distill();
            self.last_distillation_report = Some(report.clone());
            Some(report)
        } else {
            None
        }
    }

    /// Get the last distillation report
    pub fn last_distillation_report(&self) -> Option<&DistillationReport> {
        self.last_distillation_report.as_ref()
    }

    /// Run distillation by model
    pub fn run_distillation_by_model(
        &mut self,
    ) -> Option<std::collections::HashMap<String, DistillationReport>> {
        if let Some(ref mut distiller) = self.cross_model_distiller {
            Some(distiller.distill_by_model())
        } else {
            None
        }
    }

    /// Check if distillation buffer has data
    pub fn has_distillation_data(&self) -> bool {
        self.distillation_buffer
            .as_ref()
            .and_then(|b| b.lock().ok())
            .map(|b| b.len() > 0)
            .unwrap_or(false)
    }

    /// Get distillation buffer length
    pub fn distillation_buffer_len(&self) -> usize {
        self.distillation_buffer
            .as_ref()
            .and_then(|b| b.lock().ok())
            .map(|b| b.len())
            .unwrap_or(0)
    }

    /// Get distillation count
    pub fn distillation_count(&self) -> u64 {
        self.cross_model_distiller
            .as_ref()
            .map(|d| d.distillation_count())
            .unwrap_or(0)
    }

    /// Run distillation if enough data accumulated (every N interactions)
    pub fn maybe_run_distillation(&mut self, threshold: usize) -> Option<DistillationReport> {
        if self.distillation_buffer_len() >= threshold {
            self.run_distillation()
        } else {
            None
        }
    }
}

impl Default for MindBridge {
    fn default() -> Self {
        Self::new()
    }
}
