use crate::core::nt_core_sense::*;
use crate::neotrix::nt_world_sense::visual_cortex::VisualCortex;
use crate::neotrix::nt_world_sense::auditory_cortex::AuditoryCortex;
use crate::neotrix::nt_world_sense::real_sensors::screen::ScreenCapture;
use crate::neotrix::nt_world_sense::real_sensors::mic::MicCapture;
use std::time::Duration;
use crate::neotrix::nt_act_voice::{VoiceInput, VoiceSample};

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
                    self.memory.push(event.clone());
                    events.push(event);
                }
            }
            if self.mic_cap.is_active() {
                if let Some(sample) = self.mic_cap.poll() {
                    let event = self.mic_cap.to_event(sample.clone());
                    self.memory.push(event.clone());
                    events.push(event);
                    if let Some(ref mut vi) = self.nt_act_voice_input {
                        let dur = Duration::from_secs_f64(
                            sample.metadata.get("duration_secs")
                                .and_then(|v| v.parse::<f64>().ok())
                                .unwrap_or(2.0)
                        );
                        let vs = VoiceSample::from_raw_wav(&sample.raw_bytes, dur, 16000);
                        if let Ok(text) = vi.transcribe(&vs) {
                            let nt_act_voice_event = vi.to_event(&vs, &text);
                            self.memory.push(nt_act_voice_event.clone());
                            events.push(nt_act_voice_event);
                        }
                    }
                }
            }
        } else {
            // File-simulated path (existing behavior)
            if self.visual.is_active() {
                if let Some(ref path) = self.sight_path {
                    if let Some(event) = self.visual.scan_from_file(path) {
                        self.memory.push(event.clone());
                        events.push(event);
                    }
                }
            }
            if self.auditory.is_active() {
                if let Some(ref path) = self.hearing_path {
                    if let Some(event) = self.auditory.listen_from_file(path) {
                        self.memory.push(event.clone());
                        events.push(event);
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
        let lv = match &self.visual.last_scan {
            Some(e) => format!("last: {}", e.description),
            None => "no data".to_string(),
        };
        let la = match &self.auditory.last_hearing {
            Some(e) => format!("last: {}", e.description),
            None => "no data".to_string(),
        };
        format!(
            "Perception [mode:{} | screen:{} | mic:{} | file-vis:{} | file-aud:{} | memory:{} events]\n  Visual: {}\n  Auditory: {}",
            real, sc, mc, vs, aus, mem_count, lv, la,
        )
    }
}

impl Default for SensoryIntegrationHub {
    fn default() -> Self { Self::new() }
}
