use crate::core::nt_core_sense::{
    AnalysisReport, Sensor, SensorSample, SensoryEvent, SensoryEventKind,
};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// macOS screen capture using `screencapture` CLI.
/// Falls back to file metadata analysis on non-macOS.
pub struct ScreenCapture {
    active: bool,
    tmp_dir: PathBuf,
    last_sample: Option<SensorSample>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            active: false,
            tmp_dir: std::env::temp_dir().join("neotrix_screencap"),
            last_sample: None,
        }
    }

    fn capture_screen_macos(&self) -> Option<SensorSample> {
        std::fs::create_dir_all(&self.tmp_dir).ok()?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let out_path = self.tmp_dir.join(format!("screen_{}.png", ts));

        let status = Command::new("screencapture")
            .arg("-x") // no sound
            .arg("-C") // capture cursor
            .arg(&out_path)
            .status()
            .ok()?;

        if !status.success() {
            return None;
        }

        let raw = std::fs::read(&out_path).ok()?;
        let meta = std::fs::metadata(&out_path).ok()?;
        let file_size = meta.len();

        let mut sample = SensorSample::new(raw);
        sample
            .metadata
            .insert("method".into(), "screencapture".into());
        sample.metadata.insert("format".into(), "png".into());
        sample
            .metadata
            .insert("file_size_bytes".into(), file_size.to_string());
        sample.confidence = 0.85;

        let _ = std::fs::remove_file(&out_path);
        Some(sample)
    }

    fn capture_placeholder(&self) -> Option<SensorSample> {
        let sample = SensorSample::new(vec![]);
        Some(sample)
    }
}

impl Sensor for ScreenCapture {
    fn name(&self) -> &str {
        "screen_capture"
    }

    fn poll(&mut self) -> Option<SensorSample> {
        if !self.active {
            return None;
        }
        let sample = if cfg!(target_os = "macos") {
            self.capture_screen_macos()
        } else {
            self.capture_placeholder()
        };
        if let Some(ref s) = sample {
            self.last_sample = Some(s.clone());
        }
        sample
    }

    fn activate(&mut self) {
        self.active = true;
    }
    fn deactivate(&mut self) {
        self.active = false;
    }
    fn is_active(&self) -> bool {
        self.active
    }

    fn to_event(&self, sample: SensorSample) -> SensoryEvent {
        let desc = format!(
            "Screen capture: {}px, {} bytes",
            sample
                .metadata
                .get("file_size_bytes")
                .unwrap_or(&"?".into()),
            sample.size()
        );
        SensoryEvent {
            id: rand::random::<u64>(),
            timestamp_ms: sample.timestamp_ms,
            kind: SensoryEventKind::Visual(AnalysisReport {
                description: desc.clone(),
                detected_elements: vec![],
                dominant_colors: vec![],
                layout_summary: String::new(),
            }),
            source: "screen_capture".into(),
            priority: 5,
            confidence: sample.confidence,
            description: desc,
            raw_data_size: sample.size(),
        }
    }
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}
