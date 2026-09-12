//! Real-time download progress display with visual progress bar.
//!
//! Provides terminal-friendly progress output with:
//! - Visual progress bar (█░ symbols)
//! - Speed display (B/s, KB/s, MB/s)
//! - ETA calculation
//! - File size tracking

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
pub struct DownloadProgress {
    total_bytes: u64,
    downloaded_bytes: u64,
    start_time: Instant,
    last_update: Instant,
    last_bytes: u64,
    speed_bytes_per_sec: f64,
    config: ProgressConfig,
}

impl DownloadProgress {
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
        io::stdout().flush().unwrap();
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
        let mut progress = DownloadProgress::new(1_000_000_000, ProgressConfig::default());
        progress.update(500_000_000);
        // Just verify it doesn't panic
        progress.display();
    }
}
