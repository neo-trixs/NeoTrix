use crate::core::nt_core_sense::*;
use std::path::PathBuf;

pub struct VisualCortex {
    pub active: bool,
    pub last_scan: Option<SensoryEvent>,
}

impl VisualCortex {
    pub fn new() -> Self {
        Self {
            active: false,
            last_scan: None,
        }
    }

    pub fn scan_from_file(&mut self, path: &PathBuf) -> Option<SensoryEvent> {
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
            kind: SensoryEventKind::Visual(AnalysisReport {
                description: format!("Visual scan of \"{}\" ({} bytes)", stem, file_len),
                detected_elements: vec![],
                dominant_colors: vec![],
                layout_summary: String::new(),
            }),
            source,
            priority,
            confidence: 0.8,
            description: format!("Visual scan of \"{}\" ({} bytes)", stem, file_len),
            raw_data_size: file_len as usize,
        };
        self.last_scan = Some(event.clone());
        Some(event)
    }

    pub fn activate(&mut self) {
        self.active = true;
    }
    pub fn deactivate(&mut self) {
        self.active = false;
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Default for VisualCortex {
    fn default() -> Self {
        Self::new()
    }
}
