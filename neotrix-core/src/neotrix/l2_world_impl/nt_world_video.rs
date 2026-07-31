#![deny(clippy::unwrap_used)]

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A single decoded video frame with a 16×16 grayscale descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFrame {
    /// Timestamp in seconds from the start of the video.
    pub timestamp: f64,
    /// 64-bit hash of the raw frame data (e.g. dHash, xxhash).
    pub data_hash: u64,
    /// 16×16 downsampled grayscale descriptor (pixel values 0–255).
    pub grayscale_16x16: [[u8; 16]; 16],
}

impl VideoFrame {
    /// Compute mean absolute pixel difference against another frame.
    pub fn mean_diff(&self, other: &VideoFrame) -> f64 {
        let mut total = 0u64;
        for y in 0..16 {
            for x in 0..16 {
                let d = (self.grayscale_16x16[y][x] as i16 - other.grayscale_16x16[y][x] as i16)
                    .unsigned_abs();
                total += d as u64;
            }
        }
        total as f64 / 256.0
    }
}

/// Summary of a processed video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSummary {
    /// Total number of frames in the video.
    pub frame_count: u64,
    /// Number of unique / key frames after dedup.
    pub key_frame_count: u64,
    /// Estimated duration in seconds.
    pub duration_secs: f64,
}

/// Video frame processing pipeline.
///
/// Ingests a sequence of [`VideoFrame`]s, deduplicates via grayscale
/// comparison, and produces a [`VideoSummary`].
pub struct VideoPipeline {
    /// All ingested frames (in temporal order).
    pub frames: Vec<VideoFrame>,
    /// Indices into `frames` that were kept as key frames.
    pub key_frames: Vec<usize>,
}

impl Default for VideoPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoPipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            key_frames: Vec::new(),
        }
    }

    /// Push a new frame into the pipeline.
    pub fn push_frame(&mut self, frame: VideoFrame) {
        self.frames.push(frame);
    }

    /// Deduplicate frames by comparing each frame's 16×16 grayscale
    /// against the **last kept** (key) frame. A frame is kept when the
    /// mean absolute pixel difference exceeds `threshold` (default 5.0).
    ///
    /// The very first frame is always kept.
    pub fn dedup_frames(&mut self) {
        if self.frames.is_empty() {
            return;
        }

        self.key_frames.clear();
        // First frame is always a key frame.
        self.key_frames.push(0);

        for i in 1..self.frames.len() {
            let last_kept = &self.frames[*self.key_frames.last().expect("first frame is always a key frame")];
            let diff = self.frames[i].mean_diff(last_kept);
            if diff > 5.0 {
                self.key_frames.push(i);
            }
        }
    }

    /// Run the full pipeline: dedup then produce a summary.
    pub fn process(&mut self) -> VideoSummary {
        self.dedup_frames();
        let duration = if self.frames.is_empty() {
            0.0
        } else {
            self.frames.last().expect("non-empty frames").timestamp - self.frames[0].timestamp
        };
        VideoSummary {
            frame_count: self.frames.len() as u64,
            key_frame_count: self.key_frames.len() as u64,
            duration_secs: duration,
        }
    }

    /// Produce a summary without deduplicating (uses all frames as key).
    pub fn summary_raw(&self) -> VideoSummary {
        let duration = if self.frames.is_empty() {
            0.0
        } else {
            self.frames.last().expect("non-empty frames").timestamp - self.frames[0].timestamp
        };
        VideoSummary {
            frame_count: self.frames.len() as u64,
            key_frame_count: self.frames.len() as u64,
            duration_secs: duration,
        }
    }
}

/// Convenience: open a video file and run the pipeline.
///
/// # Notes
/// - Actual pixel decoding requires `ffmpeg` or a Rust video crate.
/// - This stub attempts to read file metadata and returns a simulated
///   summary when frame-level decoding is unavailable.
/// - The returned summary should not be used for analytical purposes;
///   integrate with a real decoder for production use.
pub fn process_video(path: &str) -> Result<VideoSummary, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("video file not found: {}", path));
    }
    let meta = fs::metadata(p).map_err(|e| format!("cannot read metadata: {}", e))?;

    // Heuristic: treat a non-empty file as having at least one frame.
    let file_size = meta.len();
    let estimated_frames = (file_size / 50_000).max(1);
    let estimated_duration = estimated_frames as f64 / 30.0; // assume 30 fps

    // Build a one-frame pipeline for illustration.
    let frame = VideoFrame {
        timestamp: 0.0,
        data_hash: file_size,
        grayscale_16x16: [[0u8; 16]; 16],
    };
    let mut pipeline = VideoPipeline::new();
    pipeline.push_frame(frame);
    pipeline.dedup_frames();

    Ok(VideoSummary {
        frame_count: estimated_frames,
        key_frame_count: pipeline.key_frames.len() as u64,
        duration_secs: estimated_duration,
    })
}

/// Build a pipeline from a pre-collected vector of frames,
/// run dedup, and return a summary.
pub fn process_frames(frames: Vec<VideoFrame>) -> VideoSummary {
    let mut pipeline = VideoPipeline {
        frames,
        key_frames: Vec::new(),
    };
    pipeline.process()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(ts: f64, pattern: u8) -> VideoFrame {
        VideoFrame {
            timestamp: ts,
            data_hash: pattern as u64,
            grayscale_16x16: [[pattern; 16]; 16],
        }
    }

    #[test]
    fn test_empty_pipeline() {
        let mut p = VideoPipeline::new();
        let s = p.process();
        assert_eq!(s.frame_count, 0);
        assert_eq!(s.key_frame_count, 0);
    }

    #[test]
    fn test_single_frame_always_kept() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 128));
        let s = p.process();
        assert_eq!(s.frame_count, 1);
        assert_eq!(s.key_frame_count, 1);
    }

    #[test]
    fn test_identical_frames_deduped() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 128));
        p.push_frame(make_frame(1.0, 128)); // diff = 0, NOT kept
        p.push_frame(make_frame(2.0, 128)); // diff = 0 against last kept (frame 0), NOT kept
        p.dedup_frames();
        assert_eq!(p.key_frames, vec![0]);
    }

    #[test]
    fn test_high_diff_frames_kept() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 0));
        p.push_frame(make_frame(1.0, 200)); // diff=200 vs frame 0, >5 → kept
        p.push_frame(make_frame(2.0, 100)); // diff=100 vs frame 1, >5 → kept
        p.dedup_frames();
        assert_eq!(p.key_frames, vec![0, 1, 2]);
    }

    #[test]
    fn test_kept_vs_last_kept_not_previous() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 0)); // key
        p.push_frame(make_frame(1.0, 2)); // diff=2 vs frame 0, <5 → skip
        p.push_frame(make_frame(2.0, 200)); // diff=200 vs frame 0 (last kept), >5 → key
        p.dedup_frames();
        assert_eq!(p.key_frames, vec![0, 2]);
    }

    #[test]
    fn test_mean_diff_identical() {
        let a = make_frame(0.0, 100);
        let b = make_frame(1.0, 100);
        assert!((a.mean_diff(&b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_mean_diff_maximum() {
        let a = make_frame(0.0, 0);
        let b = make_frame(1.0, 255);
        assert!((a.mean_diff(&b) - 255.0).abs() < 1e-9);
    }

    #[test]
    fn test_mean_diff_half_plane() {
        let mut a = make_frame(0.0, 0);
        // Upper half white, lower half black
        for y in 0..8 {
            for x in 0..16 {
                a.grayscale_16x16[y][x] = 255;
            }
        }
        let b = make_frame(1.0, 0);
        let diff = a.mean_diff(&b);
        assert!((diff - 127.5).abs() < 1.0, "expected ~127.5, got {}", diff);
    }

    #[test]
    fn test_process_frames_function() {
        let frames = vec![make_frame(0.0, 0), make_frame(1.0, 0), make_frame(2.0, 100)];
        let summary = process_frames(frames);
        assert_eq!(summary.frame_count, 3);
        assert_eq!(summary.key_frame_count, 2);
        assert!((summary.duration_secs - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_process_video_missing_file() {
        let result = process_video("/nonexistent/video.mp4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_video_frame_serde() {
        let frame = make_frame(1.5, 42);
        let json = serde_json::to_string(&frame).unwrap();
        let back: VideoFrame = serde_json::from_str(&json).unwrap();
        assert!((back.timestamp - 1.5).abs() < 1e-9);
        assert_eq!(back.data_hash, 42);
        assert_eq!(back.grayscale_16x16[0][0], 42);
    }

    #[test]
    fn test_video_summary_serde() {
        let s = VideoSummary {
            frame_count: 100,
            key_frame_count: 12,
            duration_secs: 30.0,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: VideoSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.frame_count, 100);
        assert_eq!(back.key_frame_count, 12);
    }
}
