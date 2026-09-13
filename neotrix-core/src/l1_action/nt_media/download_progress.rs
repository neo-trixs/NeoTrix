//! Real-time download progress display with visual progress bar.
//!
//! Provides terminal-friendly progress output with:
//! - Visual progress bar (█░ symbols)
//! - Speed display (B/s, KB/s, MB/s)
//! - ETA calculation
//! - File size tracking
//!
//! Accepts `PipelineStatus` directly for unified progress display.

use super::streaming::PipelineStatus;
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Progress bar configuration
pub struct ProgressConfig {
    pub bar_width: usize,
    pub update_interval: Duration,
    pub show_speed: bool,
    pub show_eta: bool,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            bar_width: 40,
            update_interval: Duration::from_millis(500),
            show_speed: true,
            show_eta: true,
        }
    }
}

/// Tracks and displays download progress
pub struct DownloadProgressBar {
    total_bytes: u64,
    downloaded_bytes: u64,
    start_time: Instant,
    last_update: Instant,
    last_bytes: u64,
    speed_bytes_per_sec: f64,
    config: ProgressConfig,
}

impl DownloadProgressBar {
    pub fn new(total_bytes: u64, config: ProgressConfig) -> Self {
        let now = Instant::now();
        Self {
            total_bytes,
            downloaded_bytes: 0,
            start_time: now,
            last_update: now,
            last_bytes: 0,
            speed_bytes_per_sec: 0.0,
            config,
        }
    }

    /// Create a DownloadProgressBar directly from a PipelineStatus variant.
    ///
    /// Extracts `downloaded`, `total`, and `speed_bps` from the status.
    /// Returns None for non-downloading statuses (Resolving, Complete, Failed, Cancelled).
    pub fn from_pipeline_status(status: &PipelineStatus, config: ProgressConfig) -> Option<Self> {
        match status {
            PipelineStatus::Downloading {
                downloaded,
                total,
                speed_bps,
            }
            | PipelineStatus::Playing {
                downloaded,
                total,
                speed_bps,
            } => {
                let total_bytes = total.unwrap_or(0);
                let mut progress = Self::new(total_bytes, config);
                progress.downloaded_bytes = *downloaded;
                progress.speed_bytes_per_sec = *speed_bps;
                Some(progress)
            }
            _ => None,
        }
    }

    /// Update progress from a PipelineStatus, returning whether display should refresh.
    pub fn update_from_pipeline_status(&mut self, status: &PipelineStatus) -> bool {
        match status {
            PipelineStatus::Downloading {
                downloaded,
                total,
                speed_bps,
            }
            | PipelineStatus::Playing {
                downloaded,
                total,
                speed_bps,
            } => {
                self.downloaded_bytes = *downloaded;
                if let Some(t) = total {
                    self.total_bytes = *t;
                }
                self.speed_bytes_per_sec = *speed_bps;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self, bytes: u64) {
        self.downloaded_bytes = bytes;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        if elapsed >= self.config.update_interval {
            let delta_bytes = bytes.saturating_sub(self.last_bytes);
            self.speed_bytes_per_sec = delta_bytes as f64 / elapsed.as_secs_f64();
            self.last_bytes = bytes;
            self.last_update = now;
        }
    }

    pub fn display(&self) {
        let pct = if self.total_bytes > 0 {
            (self.downloaded_bytes as f64 / self.total_bytes as f64 * 100.0) as u32
        } else {
            0
        };

        let filled = (pct as usize * self.config.bar_width) / 100;
        let empty = self.config.bar_width.saturating_sub(filled);

        let bar: String = "█".repeat(filled) + &"░".repeat(empty);

        let down_mb = self.downloaded_bytes as f64 / 1_048_576.0;
        let total_mb = self.total_bytes as f64 / 1_048_576.0;

        let speed_str = if self.config.show_speed {
            let speed = self.speed_bytes_per_sec;
            if speed > 1_048_576.0 {
                format!(" | {:.1} MB/s", speed / 1_048_576.0)
            } else if speed > 1024.0 {
                format!(" | {:.1} KB/s", speed / 1024.0)
            } else {
                format!(" | {:.0} B/s", speed)
            }
        } else {
            String::new()
        };

        let eta_str = if self.config.show_eta && self.speed_bytes_per_sec > 0.0 {
            let remaining = (self.total_bytes - self.downloaded_bytes) as f64
                / self.speed_bytes_per_sec;
            let mins = remaining as u64 / 60;
            let secs = remaining as u64 % 60;
            format!(" | 剩余 {}m{:02}s", mins, secs)
        } else {
            String::new()
        };

        let elapsed = self.start_time.elapsed();
        let elapsed_mins = elapsed.as_secs() / 60;
        let elapsed_secs = elapsed.as_secs() % 60;

        print!(
            "\r\033[K  {:3}% {} {:.1}/{:.1} MB{}{} | 已用 {}m{:02}s",
            pct, bar, down_mb, total_mb, speed_str, eta_str, elapsed_mins, elapsed_secs
        );
        let _ = io::stdout().flush();
    }

    pub fn finish(&self) {
        let elapsed = self.start_time.elapsed();
        let total_mb = self.total_bytes as f64 / 1_048_576.0;
        let avg_speed = if elapsed.as_secs() > 0 {
            self.total_bytes as f64 / elapsed.as_secs_f64() / 1_048_576.0
        } else {
            0.0
        };
        println!(
            "\n✅ 完成: {:.1} MB | 平均 {:.1} MB/s | 用时 {}m{:02}s",
            total_mb,
            avg_speed,
            elapsed.as_secs() / 60,
            elapsed.as_secs() % 60
        );
    }
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    if bytes > 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes > 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes > 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_progress_display() {
        let mut progress = DownloadProgressBar::new(1_000_000_000, ProgressConfig::default());
        progress.update(500_000_000);
        // Just verify it doesn't panic
        progress.display();
    }

    #[test]
    fn test_from_pipeline_status_downloading() {
        let status = PipelineStatus::Downloading {
            downloaded: 512,
            total: Some(1024),
            speed_bps: 256.0,
        };
        let progress = DownloadProgressBar::from_pipeline_status(&status, ProgressConfig::default());
        assert!(progress.is_some());
        let p = progress.unwrap();
        assert_eq!(p.downloaded_bytes, 512);
        assert_eq!(p.total_bytes, 1024);
        assert_eq!(p.speed_bytes_per_sec, 256.0);
    }

    #[test]
    fn test_from_pipeline_status_non_downloading() {
        let status = PipelineStatus::Resolving;
        let progress = DownloadProgressBar::from_pipeline_status(&status, ProgressConfig::default());
        assert!(progress.is_none());
    }

    #[test]
    fn test_update_from_pipeline_status() {
        let status = PipelineStatus::Downloading {
            downloaded: 256,
            total: Some(512),
            speed_bps: 128.0,
        };
        let mut progress = DownloadProgressBar::new(0, ProgressConfig::default());
        let updated = progress.update_from_pipeline_status(&status);
        assert!(updated);
        assert_eq!(progress.downloaded_bytes, 256);
        assert_eq!(progress.total_bytes, 512);
        assert_eq!(progress.speed_bytes_per_sec, 128.0);
    }
}
