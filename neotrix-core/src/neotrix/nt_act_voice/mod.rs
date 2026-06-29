use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod command;
pub mod transcribe;
pub mod trigger;

use super::nt_world_sense::real_sensors::mic::MicCapture;
use crate::core::nt_core_sense::{Sensor, SensoryEvent, SensoryEventKind, Transcription};
pub use command::VoiceCommand;
pub use transcribe::TranscribeEngine;
use transcribe::{ExternalAPITranscriber, MockTranscriber, WhisperTranscriber};
pub use trigger::VoiceTrigger;

#[derive(Debug, Clone)]
pub struct VoiceSample {
    pub audio_data: Vec<f32>,
    pub duration: Duration,
    pub sample_rate: u32,
    pub timestamp: SystemTime,
}

impl VoiceSample {
    pub fn new(audio_data: Vec<f32>, duration: Duration, sample_rate: u32) -> Self {
        Self {
            audio_data,
            duration,
            sample_rate,
            timestamp: SystemTime::now(),
        }
    }

    pub fn from_raw_wav(wav_bytes: &[u8], duration: Duration, sample_rate: u32) -> Self {
        let audio_data = decode_wav_to_f32(wav_bytes);
        Self {
            audio_data,
            duration,
            sample_rate,
            timestamp: SystemTime::now(),
        }
    }

    pub fn to_wav_bytes(&self) -> Vec<u8> {
        if self.audio_data.is_empty() {
            return vec![];
        }
        let sample_rate = self.sample_rate;
        let num_samples = self.audio_data.len();
        let data_size = num_samples * 2;
        let file_size = 36 + data_size;

        let mut bytes = Vec::with_capacity(44 + data_size);

        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(file_size as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");

        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());

        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data_size as u32).to_le_bytes());

        for &sample in &self.audio_data {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * 32767.0) as i16;
            bytes.extend_from_slice(&int_sample.to_le_bytes());
        }

        bytes
    }
}

fn decode_wav_to_f32(wav_bytes: &[u8]) -> Vec<f32> {
    if wav_bytes.is_empty() {
        return vec![];
    }
    let header_size = 44;
    if wav_bytes.len() <= header_size {
        return vec![];
    }
    let data = &wav_bytes[header_size..];
    let sample_count = data.len() / 2;
    let mut samples = Vec::with_capacity(sample_count);
    for chunk in data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample as f32 / 32768.0);
    }
    samples
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceError {
    NoAudio,
    TranscriptionFailed(String),
    EngineNotAvailable,
    AlreadyRecording,
    NotRecording,
    WakeWordTimeout,
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::NoAudio => write!(f, "no audio captured"),
            VoiceError::TranscriptionFailed(msg) => write!(f, "transcription failed: {}", msg),
            VoiceError::EngineNotAvailable => write!(f, "transcribe engine not available"),
            VoiceError::AlreadyRecording => write!(f, "already recording"),
            VoiceError::NotRecording => write!(f, "not recording"),
            VoiceError::WakeWordTimeout => write!(f, "wake word detection timed out"),
        }
    }
}

impl std::error::Error for VoiceError {}

pub struct VoiceInput {
    mic: MicCapture,
    engine: TranscribeEngine,
    whisper: WhisperTranscriber,
    active: bool,
    continuous: bool,
    last_transcription: Option<String>,
    last_sample: Option<VoiceSample>,
    trigger: VoiceTrigger,
}

impl VoiceInput {
    pub fn new() -> Self {
        Self {
            mic: MicCapture::new(),
            engine: TranscribeEngine::Mock,
            whisper: WhisperTranscriber::new(),
            active: false,
            continuous: false,
            last_transcription: None,
            last_sample: None,
            trigger: VoiceTrigger::new(),
        }
    }

    pub fn with_engine(mut self, engine: TranscribeEngine) -> Self {
        self.engine = engine;
        self
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.mic.activate();
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.mic.deactivate();
        self.continuous = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn is_continuous(&self) -> bool {
        self.continuous
    }

    pub fn record(&mut self, duration: Duration) -> Result<VoiceSample, VoiceError> {
        if !self.active {
            self.activate();
        }
        let dur = duration.as_secs_f64();
        let old = std::mem::replace(&mut self.mic, MicCapture::new());
        self.mic = old.with_duration(dur);
        let sample = self.mic.poll().ok_or(VoiceError::NoAudio)?;
        let dur = duration;
        let vs = VoiceSample::from_raw_wav(&sample.raw_bytes, dur, 16000);
        self.last_sample = Some(vs.clone());
        Ok(vs)
    }

    pub fn transcribe(&self, sample: &VoiceSample) -> Result<String, VoiceError> {
        match &self.engine {
            TranscribeEngine::Mock => MockTranscriber::transcribe(sample),
            TranscribeEngine::Whisper => self.whisper.transcribe(sample),
            TranscribeEngine::ExternalAPI { .. } => ExternalAPITranscriber::transcribe(sample),
        }
    }

    pub fn start_continuous(&mut self) {
        self.continuous = true;
        self.activate();
    }

    pub fn stop_continuous(&mut self) {
        self.continuous = false;
    }

    pub fn poll_transcription(&mut self) -> Option<String> {
        if !self.active || !self.continuous {
            return None;
        }
        let sample = self.mic.poll()?;
        let dur = Duration::from_secs_f64(
            sample
                .metadata
                .get("duration_secs")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(2.0),
        );
        let vs = VoiceSample::from_raw_wav(&sample.raw_bytes, dur, 16000);
        match self.transcribe(&vs) {
            Ok(text) => {
                self.last_transcription = Some(text.clone());
                self.last_sample = Some(vs);
                Some(text)
            }
            Err(_) => None,
        }
    }

    pub fn check_wake_word(&mut self) -> bool {
        if !self.active {
            return false;
        }
        let sample = match self.mic.poll() {
            Some(s) => s,
            None => return false,
        };
        let dur = Duration::from_secs_f64(
            sample
                .metadata
                .get("duration_secs")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(2.0),
        );
        let vs = VoiceSample::from_raw_wav(&sample.raw_bytes, dur, 16000);
        match self.transcribe(&vs) {
            Ok(text) => self.trigger.detect(&text),
            Err(_) => false,
        }
    }

    pub fn last_transcription(&self) -> Option<&str> {
        self.last_transcription.as_deref()
    }

    pub fn last_sample(&self) -> Option<&VoiceSample> {
        self.last_sample.as_ref()
    }

    pub fn set_trigger(&mut self, trigger: VoiceTrigger) {
        self.trigger = trigger;
    }

    pub fn trigger(&self) -> &VoiceTrigger {
        &self.trigger
    }

    pub fn engine(&self) -> &TranscribeEngine {
        &self.engine
    }

    pub fn to_event(&self, sample: &VoiceSample, text: &str) -> SensoryEvent {
        let dur = sample.duration.as_secs_f64();
        let ts = sample
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        SensoryEvent {
            id: rand::random::<u64>(),
            timestamp_ms: ts,
            kind: SensoryEventKind::Auditory(Transcription {
                text: text.to_string(),
                language: "en".into(),
                confidence: 0.85,
                duration_secs: dur,
            }),
            source: "nt_act_voice_input".into(),
            priority: 6,
            confidence: 0.85,
            description: format!("Voice: \"{}\" ({}s)", text, dur),
            raw_data_size: sample.audio_data.len(),
        }
    }

    pub fn record_and_transcribe(
        &mut self,
        duration: Duration,
    ) -> Result<(VoiceSample, String), VoiceError> {
        let sample = self.record(duration)?;
        let text = self.transcribe(&sample)?;
        Ok((sample, text))
    }
}

impl Default for VoiceInput {
    fn default() -> Self {
        Self::new()
    }
}
