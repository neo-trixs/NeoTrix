use crate::core::nt_core_sense::*;
use crate::neotrix::nt_world_sense::visual_cortex::VisualCortex;
use crate::neotrix::nt_world_sense::auditory_cortex::AuditoryCortex;
use crate::neotrix::nt_world_sense::real_sensors::screen::ScreenCapture;
use crate::neotrix::nt_world_sense::real_sensors::mic::MicCapture;
use std::time::Duration;
use crate::neotrix::nt_act_voice::{VoiceInput, VoiceSample};

/// Goal context for CraniMEM gating — matches nt_memory_search GoalContext.
#[derive(Debug, Clone)]
pub struct GoalContext {
    pub goal_id: String,
    pub keywords: Vec<String>,
    pub weight: f64,
}

/// CraniMEM goal-conditioned gating mechanism.
/// Filters sensory events before GWT salience computation,
/// preventing irrelevant stimuli from consuming attention bandwidth.
#[derive(Debug, Clone)]
pub struct CraniMEMGate {
    pub active_goals: Vec<GoalContext>,
    pub threshold: f64,
    pub emergency_override: bool,
}

impl Default for CraniMEMGate {
    fn default() -> Self {
        Self {
            active_goals: Vec::new(),
            threshold: 0.1,
            emergency_override: false,
        }
    }
}

impl CraniMEMGate {
    pub fn new(threshold: f64) -> Self {
        Self {
            active_goals: Vec::new(),
            threshold,
            emergency_override: false,
        }
    }

    /// Gate a sensory event — returns (passed, relevance_score).
    pub fn gate(&self, description: &str) -> (bool, f64) {
        if self.emergency_override || self.active_goals.is_empty() {
            return (true, 1.0);
        }
        let desc_lower = description.to_lowercase();
        let max_relevance: f64 = self.active_goals.iter().map(|g| {
            let hits = g.keywords.iter()
                .filter(|kw| desc_lower.contains(&kw.to_lowercase()))
                .count();
            if hits == 0 { 0.0 }
            else { g.weight * (hits as f64 / g.keywords.len() as f64).min(1.0) }
        }).fold(0.0, f64::max);
        (max_relevance >= self.threshold, max_relevance)
    }

    pub fn set_goals(&mut self, goals: Vec<GoalContext>) {
        self.active_goals = goals;
    }

    pub fn set_emergency(&mut self, active: bool) {
        self.emergency_override = active;
    }
}

pub struct SensoryIntegrationHub {
    pub visual: VisualCortex,
    pub auditory: AuditoryCortex,
    pub memory: SensoryMemory,
    pub active: bool,
    pub screen_cap: ScreenCapture,
    pub mic_cap: MicCapture,
    pub nt_act_voice_input: Option<VoiceInput>,
    pub use_real_sensors: bool,
    pub sight_path: Option<std::path::PathBuf>,
    pub hearing_path: Option<std::path::PathBuf>,
    /// CraniMEM goal-conditioned gating — filters sensory events before GWT salience.
    pub crani_gate: CraniMEMGate,
}

impl SensoryIntegrationHub {
    pub fn new() -> Self {
        Self {
            visual: VisualCortex::new(),
            auditory: AuditoryCortex::new(),
            memory: SensoryMemory::with_capacity(100),
            active: false,
            screen_cap: ScreenCapture::new(),
            mic_cap: MicCapture::new(),
            nt_act_voice_input: Some(VoiceInput::new()),
            use_real_sensors: false,
            sight_path: None,
            hearing_path: None,
            crani_gate: CraniMEMGate::default(),
        }
    }

    pub fn enable_real_sensors(&mut self) {
        self.use_real_sensors = true;
    }

    pub fn disable_real_sensors(&mut self) {
        self.use_real_sensors = false;
    }

    pub fn set_sight_path(&mut self, path: std::path::PathBuf) {
        self.sight_path = Some(path);
    }
    pub fn set_hearing_path(&mut self, path: std::path::PathBuf) {
        self.hearing_path = Some(path);
    }

    /// Set CraniMEM active goals for event gating.
    pub fn set_crani_goals(&mut self, goals: Vec<GoalContext>) {
        self.crani_gate.set_goals(goals);
    }

    /// Enable/disable CraniMEM emergency override (system alerts bypass gating).
    pub fn set_crani_emergency(&mut self, active: bool) {
        self.crani_gate.set_emergency(active);
    }

    pub fn poll_all(&mut self) -> Vec<SensoryEvent> {
        if !self.active {
            return vec![];
        }
        let mut events: Vec<SensoryEvent> = vec![];

        if self.use_real_sensors {
            // Real sensor path
            if self.screen_cap.is_active() {
                if let Some(sample) = self.screen_cap.poll() {
                    let event = self.screen_cap.to_event(sample);
                    // CraniMEM gating: filter before memory push
                    let (passed, _score) = self.crani_gate.gate(&event.description);
                    if passed {
                        self.memory.push(event.clone());
                        events.push(event);
                    }
                }
            }
            if self.mic_cap.is_active() {
                if let Some(sample) = self.mic_cap.poll() {
                    let event = self.mic_cap.to_event(sample.clone());
                    let (passed, _score) = self.crani_gate.gate(&event.description);
                    if passed {
                        self.memory.push(event.clone());
                        events.push(event);
                    }
                    if let Some(ref mut vi) = self.nt_act_voice_input {
                        let dur = Duration::from_secs_f64(
                            sample.metadata.get("duration_secs")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(2.0)
                        );
                        let vs = VoiceSample::from_raw_wav(&sample.raw_bytes, dur, 16000);
                        if let Ok(text) = vi.transcribe(&vs) {
                            let nt_act_voice_event = vi.to_event(&vs, &text);
                            let (passed, _score) = self.crani_gate.gate(&nt_act_voice_event.description);
                            if passed {
                                self.memory.push(nt_act_voice_event.clone());
                                events.push(nt_act_voice_event);
                            }
                        }
                    }
                }
            }
        } else {
            // File-simulated path (existing behavior)
            if self.visual.is_active() {
                if let Some(ref path) = self.sight_path {
                    if let Some(event) = self.visual.scan_from_file(path) {
                        let (passed, _score) = self.crani_gate.gate(&event.description);
                        if passed {
                            self.memory.push(event.clone());
                            events.push(event);
                        }
                    }
                }
            }
            if self.auditory.is_active() {
                if let Some(ref path) = self.hearing_path {
                    if let Some(event) = self.auditory.listen_from_file(path) {
                        let (passed, _score) = self.crani_gate.gate(&event.description);
                        if passed {
                            self.memory.push(event.clone());
                            events.push(event);
                        }
                    }
                }
            }
        }
        events
    }

    pub fn current_perception_narrative(&self) -> String {
        let real = if self.use_real_sensors { "real" } else { "file" };
        let sc = if self.screen_cap.is_active() { "active" } else { "inactive" };
        let mc = if self.mic_cap.is_active() { "active" } else { "inactive" };
        let vs = if self.visual.is_active() { "active" } else { "inactive" };
        let aus = if self.auditory.is_active() { "active" } else { "inactive" };
        let mem_count = self.memory.len();
        let gate_status = if self.crani_gate.active_goals.is_empty() {
            "open".to_string()
        } else {
            format!("{} goals, threshold={}", self.crani_gate.active_goals.len(), self.crani_gate.threshold)
        };
        let lv = match &self.visual.last_scan {
            Some(e) => format!("last: {}", e.description),
            None => "no data".to_string(),
        };
        let la = match &self.auditory.last_hearing {
            Some(e) => format!("last: {}", e.description),
            None => "no data".to_string(),
        };
        format!(
            "Perception [mode:{} | screen:{} | mic:{} | file-vis:{} | file-aud:{} | memory:{} events | crani_gate:{}]\n  Visual: {}\n  Auditory: {}",
            real, sc, mc, vs, aus, mem_count, gate_status, lv, la,
        )
    }
}

impl Default for SensoryIntegrationHub {
    fn default() -> Self { Self::new() }
}
