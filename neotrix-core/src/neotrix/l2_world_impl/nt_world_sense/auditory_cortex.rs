use std::path::PathBuf;
use crate::core::nt_core_sense::*;

pub struct AuditoryCortex {
    pub active: bool,
    pub last_hearing: Option<SensoryEvent>,
}

impl AuditoryCortex {
    pub fn new() -> Self {
        Self { active: false, last_hearing: None }
    }

    pub fn is_active(&self) -> bool { self.active }

    pub fn listen_from_file(&mut self, path: &PathBuf) -> Option<SensoryEvent> {
        if !path.exists() || !path.is_file() {
            return None;
        }
        let metadata = std::fs::metadata(path).ok()?;
        let file_len = metadata.len();
        let source = path.file_name()?.to_string_lossy().to_string();
        let stem = path.file_stem()?.to_string_lossy().to_string();
        let priority = (file_len as f64 / 1024.0_f64).min(1.0) as u8;

        let event = SensoryEvent {
            id: rand::random::<u64>(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            kind: SensoryEventKind::Auditory(Transcription {
                text: format!("Audio analysis of \"{}\" ({} bytes)", stem, file_len),
                language: "unknown".into(),
                confidence: 0.7,
                duration_secs: file_len as f64 / 16000.0,
            }),
            source,
            priority,
            confidence: 0.7,
            description: format!("Audio analysis of \"{}\" ({} bytes)", stem, file_len),
            raw_data_size: file_len as usize,
        };
        self.last_hearing = Some(event.clone());
        Some(event)
    }

    pub fn activate(&mut self) { self.active = true; }
    pub fn deactivate(&mut self) { self.active = false; }
}

impl Default for AuditoryCortex {
    fn default() -> Self { Self::new() }
}
