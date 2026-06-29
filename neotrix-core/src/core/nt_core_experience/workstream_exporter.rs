use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkstreamReport {
    pub cycle: u64,
    pub timestamp: u64,
    pub active_goals: Vec<String>,
    pub failure_clusters: usize,
    pub skills_mastered: usize,
    pub skills_total: usize,
    pub calibration_error: f64,
    pub ece: f64,
    pub meta_accuracy: f64,
    pub negentropy: f64,
    pub c_score: f64,
    pub insights: Vec<String>,
    pub recent_blocks: Vec<String>,
}

impl WorkstreamReport {
    pub fn to_markdown(&self) -> String {
        let goals = self.active_goals.join(", ");
        let blocks = self.recent_blocks.join(", ");
        format!(
            "\
# NeoTrix Workstream Report
- Cycle: {}
- Timestamp: {}
- Active Goals: {}
- Failure Clusters: {}
- Skills: {}/{}
- Calibration Error: {:.4}
- Meta Accuracy: {:.4}
- C-Score: {:.4}
- Negentropy: {:.4}
- Blocks: {}
",
            self.cycle,
            self.timestamp,
            goals,
            self.failure_clusters,
            self.skills_mastered,
            self.skills_total,
            self.ece,
            self.meta_accuracy,
            self.c_score,
            self.negentropy,
            blocks,
        )
    }
}

#[derive(Debug, Clone)]
pub struct WorkstreamExporter {
    pub output_dir: PathBuf,
    pub last_report: Option<WorkstreamReport>,
    pub export_count: u64,
    pub auto_export: bool,
    pub export_interval_cycles: u64,
    pub last_export_cycle: u64,
}

impl WorkstreamExporter {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            last_report: None,
            export_count: 0,
            auto_export: true,
            export_interval_cycles: 100,
            last_export_cycle: 0,
        }
    }

    pub fn export(&mut self, report: &WorkstreamReport) -> io::Result<PathBuf> {
        if let Err(e) = fs::create_dir_all(&self.output_dir) {
            log::warn!("failed to create output dir {:?}: {}", self.output_dir, e);
        }
        let tmp_path = self
            .output_dir
            .join(format!("workstream_{}.tmp", report.cycle));
        let final_path = self
            .output_dir
            .join(format!("workstream_{}.md", report.cycle));
        let markdown = report.to_markdown();
        {
            let mut f = fs::File::create(&tmp_path)?;
            f.write_all(markdown.as_bytes())?;
            f.flush()?;
        }
        fs::rename(&tmp_path, &final_path)?;
        self.export_count += 1;
        self.last_export_cycle = report.cycle;
        Ok(final_path)
    }

    pub fn should_export(&self, current_cycle: u64) -> bool {
        self.auto_export && current_cycle >= self.last_export_cycle + self.export_interval_cycles
    }

    pub fn stats(&self) -> WorkstreamExporterStats {
        WorkstreamExporterStats {
            total_exports: self.export_count,
            auto_export: self.auto_export,
            interval_cycles: self.export_interval_cycles,
        }
    }
}

fn default_workstream_dir() -> String {
    crate::core::nt_core_util::home_dir()
        .join(".neotrix")
        .join("workstream")
        .to_string_lossy()
        .to_string()
}

impl Default for WorkstreamExporter {
    fn default() -> Self {
        Self::new(PathBuf::from(default_workstream_dir()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorkstreamExporterStats {
    pub total_exports: u64,
    pub auto_export: bool,
    pub interval_cycles: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_to_markdown() {
        let report = WorkstreamReport {
            cycle: 42,
            timestamp: 1_700_000_000,
            active_goals: vec!["explore".into(), "learn".into()],
            failure_clusters: 3,
            skills_mastered: 5,
            skills_total: 20,
            calibration_error: 0.12,
            ece: 0.08,
            meta_accuracy: 0.85,
            negentropy: 0.75,
            c_score: 0.62,
            insights: vec![],
            recent_blocks: vec!["timeout".into()],
        };
        let md = report.to_markdown();
        assert!(!md.is_empty());
        assert!(md.contains("Cycle: 42"));
        assert!(md.contains("Skills: 5/20"));
        assert!(md.contains("timeout"));
    }

    #[test]
    fn test_should_export() {
        let mut exporter = WorkstreamExporter::new(PathBuf::from("/tmp/neotrix_workstream_test"));
        exporter.auto_export = true;
        exporter.export_interval_cycles = 50;
        exporter.last_export_cycle = 0;

        assert!(exporter.should_export(50));
        assert!(exporter.should_export(100));
        assert!(!exporter.should_export(49));
        assert!(!exporter.should_export(25));

        exporter.last_export_cycle = 100;
        assert!(exporter.should_export(150));
        assert!(!exporter.should_export(149));

        exporter.auto_export = false;
        assert!(!exporter.should_export(200));
    }

    #[test]
    fn test_default() {
        let exporter = WorkstreamExporter::default();
        assert_eq!(exporter.output_dir, PathBuf::from(default_workstream_dir()));
        assert!(exporter.auto_export);
        assert_eq!(exporter.export_interval_cycles, 100);
        assert_eq!(exporter.export_count, 0);
        assert!(exporter.last_report.is_none());
    }

    #[test]
    fn test_handle_workstream_export_format() {
        let dir = PathBuf::from("/tmp/neotrix_workstream_test_export_fmt");
        let _ = fs::remove_dir_all(&dir);
        let mut exporter = WorkstreamExporter::new(dir.clone());
        exporter.export_interval_cycles = 1;
        exporter.last_export_cycle = 0;
        let report = WorkstreamReport {
            cycle: 50,
            timestamp: 1_700_000_000,
            active_goals: vec![],
            failure_clusters: 0,
            skills_mastered: 0,
            skills_total: 0,
            calibration_error: 0.0,
            ece: 0.0,
            meta_accuracy: 0.0,
            negentropy: 0.0,
            c_score: 0.0,
            insights: vec![],
            recent_blocks: vec![],
        };
        let result = exporter.export(&report);
        assert!(result.is_ok());
        let path_str = format!("workstream_export:{}", result.unwrap().display());
        assert!(path_str.starts_with("workstream_export:"));
        assert!(path_str.contains("50"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_export_creates_file() {
        let dir = PathBuf::from("/tmp/neotrix_workstream_test_export");
        let _ = fs::remove_dir_all(&dir);
        let mut exporter = WorkstreamExporter::new(dir.clone());
        let report = WorkstreamReport {
            cycle: 1,
            timestamp: 1_700_000_000,
            active_goals: vec![],
            failure_clusters: 0,
            skills_mastered: 0,
            skills_total: 0,
            calibration_error: 0.0,
            ece: 0.0,
            meta_accuracy: 0.0,
            negentropy: 0.0,
            c_score: 0.0,
            insights: vec![],
            recent_blocks: vec![],
        };
        let result = exporter.export(&report);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        assert_eq!(exporter.export_count, 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
