use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::core::nt_core_sense::{Sensor, SensorSample, SensoryEvent, SensoryEventKind, Transcription};

/// macOS microphone capture using `rec` (SoX) or `ffmpeg`.
/// Falls back to placeholder on non-macOS.
#[derive(Clone)]
pub struct MicCapture {
    active: bool,
    tmp_dir: PathBuf,
    last_sample: Option<SensorSample>,
    duration_secs: f64,
}

impl MicCapture {
    pub fn new() -> Self {
        Self {
            active: false,
            tmp_dir: std::env::temp_dir().join("neotrix_mic"),
            last_sample: None,
            duration_secs: 2.0,
        }
    }

    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    fn capture_mic_macos(&self) -> Option<SensorSample> {
        std::fs::create_dir_all(&self.tmp_dir).ok()?;
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64;
        let out_path = self.tmp_dir.join(format!("mic_{}.wav", ts));

        let dur_ms = (self.duration_secs * 1000.0) as u32;

        // Try rec (SoX) first, fall back to ffmpeg
        let status = Command::new("rec")
            .arg("-q")
            .arg(&out_path)
            .arg("trim")
            .arg("0")
            .arg(&dur_ms.to_string())
            .status()
            .ok()
            .or_else(|| {
                Command::new("ffmpeg")
                    .arg("-f")
                    .arg("avfoundation")
                    .arg("-i")
                    .arg(":0")
                    .arg("-t")
                    .arg(&self.duration_secs.to_string())
                    .arg("-y")
                    .arg(&out_path)
                    .status()
                    .ok()
            });

        match status {
            Some(s) if s.success() => {}
            _ => return None,
        }

        let raw = std::fs::read(&out_path).ok()?;
        let meta = std::fs::metadata(&out_path).ok()?;

        let mut sample = SensorSample::new(raw);
        sample.metadata.insert("method".into(), if cfg!(target_os = "macos") { "avfoundation" } else { "unknown" }.into());
        sample.metadata.insert("format".into(), "wav".into());
        sample.metadata.insert("duration_secs".into(), self.duration_secs.to_string());
        sample.metadata.insert("file_size_bytes".into(), meta.len().to_string());
        sample.confidence = 0.75;

        let _ = std::fs::remove_file(&out_path);
        Some(sample)
    }

    fn capture_placeholder(&self) -> Option<SensorSample> {
        Some(SensorSample::new(vec![]))
    }
}

impl Sensor for MicCapture {
    fn name(&self) -> &str { "mic_capture" }

    fn poll(&mut self) -> Option<SensorSample> {
        if !self.active { return None; }
        let sample = if cfg!(target_os = "macos") {
            self.capture_mic_macos()
        } else {
            self.capture_placeholder()
        };
        if let Some(ref s) = sample {
            self.last_sample = Some(s.clone());
        }
        sample
    }

    fn activate(&mut self) { self.active = true; }
    fn deactivate(&mut self) { self.active = false; }
    fn is_active(&self) -> bool { self.active }

    fn to_event(&self, sample: SensorSample) -> SensoryEvent {
        let dur = sample.metadata.get("duration_secs").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
        let desc = format!("Mic capture: {}s, {} bytes", dur, sample.size());
        SensoryEvent {
            id: rand::random::<u64>(),
            timestamp_ms: sample.timestamp_ms,
            kind: SensoryEventKind::Auditory(Transcription {
                text: desc.clone(),
                language: "unknown".into(),
                confidence: sample.confidence,
                duration_secs: dur,
            }),
            source: "mic_capture".into(),
            priority: 5,
            confidence: sample.confidence,
            description: desc,
            raw_data_size: sample.size(),
        }
    }
}

impl Default for MicCapture {
    fn default() -> Self { Self::new() }
}
