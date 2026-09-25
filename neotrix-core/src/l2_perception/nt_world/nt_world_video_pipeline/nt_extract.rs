//! Extract segment: frame pipeline + web extractor + extraction pipeline.

use std::collections::HashMap;
use std::time::Instant;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use super::nt_transcode::{
    TranscodeConfig, VideoCodec, _StreamInfo, _StreamProtocol, _TranscodeResult, _Transcoder,
};

// ──────────────────────────────────────────────
// File 1: _VideoFrame / _VideoPipeline / process
// ──────────────────────────────────────────────

/// A single decoded video frame with a 16×16 grayscale descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _VideoFrame {
    /// Timestamp in seconds from the start of the video.
    pub timestamp: f64,
    /// 64-bit hash of the raw frame data (e.g. dHash, xxhash).
    pub data_hash: u64,
    /// 16×16 downsampled grayscale descriptor (pixel values 0–255).
    pub grayscale_16x16: [[u8; 16]; 16],
}

impl _VideoFrame {
    /// Compute mean absolute pixel difference against another frame.
    pub fn _mean_diff(&self, other: &_VideoFrame) -> f64 {
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
pub struct _VideoSummary {
    /// Total number of frames in the video.
    pub frame_count: u64,
    /// Number of unique / key frames after dedup.
    pub key_frame_count: u64,
    /// Estimated duration in seconds.
    pub duration_secs: f64,
}

/// Video frame processing pipeline.
///
/// Ingests a sequence of [`_VideoFrame`]s, deduplicates via grayscale
/// comparison, and produces a [`_VideoSummary`].
pub struct _VideoPipeline {
    /// All ingested frames (in temporal order).
    pub frames: Vec<_VideoFrame>,
    /// Indices into `frames` that were kept as key frames.
    pub key_frames: Vec<usize>,
}

impl Default for _VideoPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl _VideoPipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            key_frames: Vec::new(),
        }
    }

    /// Push a new frame into the pipeline.
    pub fn _push_frame(&mut self, frame: _VideoFrame) {
        self.frames.push(frame);
    }

    /// Deduplicate frames by comparing each frame's 16×16 grayscale
    /// against the **last kept** (key) frame. A frame is kept when the
    /// mean absolute pixel difference exceeds `threshold` (default 5.0).
    ///
    /// The very first frame is always kept.
    pub fn _dedup_frames(&mut self) {
        if self.frames.is_empty() {
            return;
        }

        self.key_frames.clear();
        // First frame is always a key frame.
        self.key_frames.push(0);

        for i in 1..self.frames.len() {
            let last_kept = &self.frames[*self.key_frames.last().unwrap_or(&0)];
            let diff = self.frames[i]._mean_diff(last_kept);
            if diff > 5.0 {
                self.key_frames.push(i);
            }
        }
    }

    /// Run the full pipeline: dedup then produce a summary.
    pub fn process(&mut self) -> _VideoSummary {
        self._dedup_frames();
        let duration = self.frames.last().map(|f| f.timestamp - self.frames[0].timestamp).unwrap_or(0.0);
        _VideoSummary {
            frame_count: self.frames.len() as u64,
            key_frame_count: self.key_frames.len() as u64,
            duration_secs: duration,
        }
    }

    /// Produce a summary without deduplicating (uses all frames as key).
    pub fn _summary_raw(&self) -> _VideoSummary {
        let duration = self.frames.last().map(|f| f.timestamp - self.frames[0].timestamp).unwrap_or(0.0);
        _VideoSummary {
            frame_count: self.frames.len() as u64,
            key_frame_count: self.frames.len() as u64,
            duration_secs: duration,
        }
    }
}

/// Convenience: open a video file and run the pipeline.
///
/// Real decoder path: shells out to `ffmpeg` for scene-aware frame extraction
/// (research absorption_video.md — first keyframe is often a black/logo intro,
/// so `select='gt(scene,0.3)'` picks one frame per visual scene, falling back
/// to uniform 1 FPS for static videos). Each extracted frame is decoded via
/// the `image` crate into the 16×16 grayscale descriptor the dedup comparator
/// expects. When ffmpeg is unavailable or extraction fails, falls back to the
/// previous file-size heuristic so the summary never hard-errors.
pub fn _process_video(path: &str) -> Result<_VideoSummary, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("video file not found: {}", path));
    }
    let meta = fs::metadata(p).map_err(|e| format!("cannot read metadata: {}", e))?;

    // Probe duration via ffprobe (for the uniform-fallback FPS).
    let duration: Option<f64> = std::process::Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path,
        ])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok());

    // Scene-aware extraction: one frame per visual scene (threshold 0.3),
    // bounded to a sane budget. Fallback to uniform 1 FPS when static.
    let tmp = std::env::temp_dir().join(format!(
        "nt_vp_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let pattern = tmp.join("f_%03d.png");
    let vpath = Path::new(path);
    let mut frames = extract_scene_frames(vpath, &pattern, 50);
    if frames.len() < 2 {
        if let Some(dur) = duration {
            if dur > 0.5 {
                let fps = (50.0 / dur).clamp(0.1, 30.0);
                frames = extract_uniform_frames(vpath, &pattern, &format!("{:.3}", fps), 50);
            }
        }
    }

    // Build VideoFrames from decoded pixels: 16×16 grayscale descriptor + phash.
    let mut pipeline = _VideoPipeline::new();
    for (i, bytes) in frames.iter().enumerate() {
        let mut frame = _VideoFrame {
            timestamp: i as f64,
            data_hash: 0,
            grayscale_16x16: [[0u8; 16]; 16],
        };
        if let Ok(img) = image::load_from_memory(bytes) {
            let rgb = img.to_rgb8();
            let (w, h) = (rgb.width().max(1), rgb.height().max(1));
            let mut hash: u64 = 0;
            let mut bit = 0u64;
            for gy in 0..16usize {
                for gx in 0..15usize {
                    let x0 = (gx as u32 * w) / 16;
                    let x1 = (((gx + 1) as u32 * w) / 16).min(w);
                    let y0 = (gy as u32 * h) / 16;
                    let y1 = (((gy + 1) as u32 * h) / 16).min(h);
                    let a = cell_lum(&rgb, x0, x1, y0, y1);
                    let x1b = (((gx + 2) as u32 * w) / 16).min(w);
                    let b = cell_lum(&rgb, x1, x1b.max(x1 + 1), y0, y1);
                    frame.grayscale_16x16[gy][gx] = (a * 255.0) as u8;
                    if a >= b {
                        hash |= 1u64 << bit;
                    }
                    bit += 1;
                }
            }
            frame.data_hash = hash;
        }
        pipeline._push_frame(frame);
    }
    let _ = std::fs::remove_dir_all(&tmp);

    if pipeline.frames.is_empty() {
        // Fallback: heuristic summary (decoder unavailable).
        let file_size = meta.len();
        let estimated_frames = (file_size / 50_000).max(1);
        let estimated_duration = estimated_frames as f64 / 30.0;
        let frame = _VideoFrame {
            timestamp: 0.0,
            data_hash: file_size,
            grayscale_16x16: [[0u8; 16]; 16],
        };
        let mut p2 = _VideoPipeline::new();
        p2._push_frame(frame);
        p2._dedup_frames();
        return Ok(_VideoSummary {
            frame_count: estimated_frames,
            key_frame_count: p2.key_frames.len() as u64,
            duration_secs: duration.unwrap_or(estimated_duration),
        });
    }

    Ok(pipeline.process())
}

/// Extract up to `max` scene-detected frames via ffmpeg, returning PNG bytes.
fn extract_scene_frames(path: &Path, pattern: &Path, max: usize) -> Vec<Vec<u8>> {
    let _ = std::process::Command::new("ffmpeg")
        .args([
            "-y", "-v", "error",
            "-i", path.to_str().unwrap_or(""),
            "-vf", "select='gt(scene,0.3)',scale=768:-2",
            "-frames:v", &max.to_string(),
            pattern.to_str().unwrap_or(""),
        ])
        .output();
    collect_pattern(pattern)
}

/// Extract up to `max` uniform frames at the given FPS via ffmpeg.
fn extract_uniform_frames(path: &Path, pattern: &Path, fps: &str, max: usize) -> Vec<Vec<u8>> {
    let _ = std::process::Command::new("ffmpeg")
        .args([
            "-y", "-v", "error",
            "-i", path.to_str().unwrap_or(""),
            "-vf", &format!("fps={},scale=768:-2", fps),
            "-frames:v", &max.to_string(),
            pattern.to_str().unwrap_or(""),
        ])
        .output();
    collect_pattern(pattern)
}

/// Collect and sort PNG frame bytes matching the ffmpeg output pattern.
fn collect_pattern(pattern: &Path) -> Vec<Vec<u8>> {
    let parent = match pattern.parent() {
        Some(p) => p.to_path_buf(),
        None => return Vec::new(),
    };
    let stem = pattern.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&parent) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with(&stem) && name.ends_with(".png") {
                files.push(e.path());
            }
        }
    }
    files.sort();
    files.into_iter().filter_map(|f| fs::read(&f).ok()).collect()
}

/// Mean luminance of a pixel block in 0..1.
fn cell_lum(rgb: &image::RgbImage, x0: u32, x1: u32, y0: u32, y1: u32) -> f64 {
    let (mut sum, mut n) = (0.0f64, 0.0f64);
    let mut py = y0;
    while py < y1 {
        let mut px = x0;
        while px < x1 {
            let p = rgb.get_pixel(px.min(rgb.width() - 1), py.min(rgb.height() - 1));
            sum += 0.2126 * p[0] as f64 / 255.0 + 0.7152 * p[1] as f64 / 255.0 + 0.0722 * p[2] as f64 / 255.0;
            n += 1.0;
            px += 1;
        }
        py += 1;
    }
    if n > 0.0 { sum / n } else { 0.0 }
}

/// Build a pipeline from a pre-collected vector of frames,
/// run dedup, and return a summary.
pub fn _process_frames(frames: Vec<_VideoFrame>) -> _VideoSummary {
    let mut pipeline = _VideoPipeline {
        frames,
        key_frames: Vec::new(),
    };
    pipeline.process()
}

pub struct _VideoExtractor {
    stream_cache: HashMap<String, _StreamInfo>,
    _extraction_count: u64,
    last_extraction: Option<Instant>,
}

impl Default for _VideoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl _VideoExtractor {
    pub fn new() -> Self {
        Self {
            stream_cache: HashMap::new(),
            _extraction_count: 0,
            last_extraction: None,
        }
    }

    pub fn _extract_from_page(&mut self, url: &str) -> Vec<_StreamInfo> {
        self._extraction_count += 1;
        self.last_extraction = Some(Instant::now());
        let stream = _StreamInfo {
            url: url.to_string(),
            protocol: _StreamProtocol::HLS,
            codec: VideoCodec::H264,
            width: 1920,
            height: 1080,
            bitrate_kbps: 6000,
            fps: 30.0,
            has_audio: true,
            has_subtitles: true,
        };
        self.stream_cache.insert(url.to_string(), stream.clone());
        vec![stream]
    }

    pub fn _extract_from_page_with_page_param(&mut self, url: &str, html: &str) -> Vec<_StreamInfo> {
        self._extraction_count += 1;
        self.last_extraction = Some(Instant::now());
        let mut streams = Vec::new();
        for line in html.lines() {
            if line.contains(".m3u8") {
                let stream_url = Self::extract_url(line);
                streams.push(_StreamInfo {
                    url: stream_url,
                    protocol: _StreamProtocol::HLS,
                    codec: VideoCodec::H264,
                    width: 1920,
                    height: 1080,
                    bitrate_kbps: 6000,
                    fps: 30.0,
                    has_audio: true,
                    has_subtitles: false,
                });
            }
            if line.contains(".mpd") {
                let stream_url = Self::extract_url(line);
                streams.push(_StreamInfo {
                    url: stream_url,
                    protocol: _StreamProtocol::DASH,
                    codec: VideoCodec::H264,
                    width: 1280,
                    height: 720,
                    bitrate_kbps: 4000,
                    fps: 30.0,
                    has_audio: true,
                    has_subtitles: false,
                });
            }
        }
        if streams.is_empty() {
            streams.push(_StreamInfo {
                url: url.to_string(),
                protocol: _StreamProtocol::Progressive,
                codec: VideoCodec::Unknown,
                width: 0,
                height: 0,
                bitrate_kbps: 0,
                fps: 0.0,
                has_audio: false,
                has_subtitles: false,
            });
        }
        for s in &streams {
            self.stream_cache.insert(s.url.clone(), s.clone());
        }
        streams
    }

    fn extract_url(line: &str) -> String {
        if let Some(start) = line.find("https://").or_else(|| line.find("http://")) {
            let rest = &line[start..];
            let end = rest.find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '>')
                .unwrap_or(rest.len());
            rest[..end].to_string()
        } else {
            line.trim().to_string()
        }
    }

    pub fn _get_cached(&self, url: &str) -> Option<&_StreamInfo> {
        self.stream_cache.get(url)
    }

    pub fn _extraction_count(&self) -> u64 {
        self._extraction_count
    }

    pub fn _best_stream<'a>(&self, streams: &'a [_StreamInfo]) -> Option<&'a _StreamInfo> {
        streams.iter().max_by(|a, b| {
            (a.width * a.height).cmp(&(b.width * b.height))
                .then_with(|| a.bitrate_kbps.cmp(&b.bitrate_kbps))
        })
    }
}

pub struct _ExtractionPipeline {
    extractor: _VideoExtractor,
    transcoder: _Transcoder,
    pipeline_active: bool,
    total_processed: u64,
}

impl _ExtractionPipeline {
    pub fn new(transcode_config: TranscodeConfig) -> Self {
        Self {
            extractor: _VideoExtractor::new(),
            transcoder: _Transcoder::new(transcode_config),
            pipeline_active: false,
            total_processed: 0,
        }
    }

    pub fn run(&mut self, page_url: &str) -> Result<_PipelineOutput, String> {
        self.pipeline_active = true;
        let streams = self.extractor._extract_from_page(page_url);
        if streams.is_empty() {
            return Err("No streams found".into());
        }
        let mut outputs = Vec::new();
        for stream in &streams {
            let needs_transcode = self.transcoder._should_transcode(stream);
            let result = if needs_transcode {
                self.transcoder._transcode(stream)
            } else {
                _TranscodeResult {
                    output_path: stream.url.clone(),
                    duration_ms: 0,
                    output_size_bytes: 0,
                    actual_codec: stream.codec,
                    hardware_used: false,
                    compression_ratio: 1.0,
                }
            };
            outputs.push((stream.clone(), result));
        }
        self.total_processed += outputs.len() as u64;
        self.pipeline_active = false;
        Ok(_PipelineOutput {
            streams: outputs,
            _extraction_count: self.extractor._extraction_count(),
        })
    }

    pub fn is_active(&self) -> bool {
        self.pipeline_active
    }

    pub fn total_processed(&self) -> u64 {
        self.total_processed
    }
}

#[derive(Debug, Clone)]
pub struct _PipelineOutput {
    pub streams: Vec<(_StreamInfo, _TranscodeResult)>,
    pub _extraction_count: u64,
}
#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(ts: f64, pattern: u8) -> _VideoFrame {
        _VideoFrame {
            timestamp: ts,
            data_hash: pattern as u64,
            grayscale_16x16: [[pattern; 16]; 16],
        }
    }

    #[test]
    fn test_empty_pipeline() {
        let mut p = _VideoPipeline::new();
        let s = p.process();
        assert_eq!(s.frame_count, 0);
        assert_eq!(s.key_frame_count, 0);
    }

    #[test]
    fn test_single_frame_always_kept() {
        let mut p = _VideoPipeline::new();
        p._push_frame(make_frame(0.0, 128));
        let s = p.process();
        assert_eq!(s.frame_count, 1);
        assert_eq!(s.key_frame_count, 1);
    }

    #[test]
    fn test_identical_frames_deduped() {
        let mut p = _VideoPipeline::new();
        p._push_frame(make_frame(0.0, 128));
        p._push_frame(make_frame(1.0, 128));
        p._push_frame(make_frame(2.0, 128));
        p._dedup_frames();
        assert_eq!(p.key_frames, vec![0]);
    }

    #[test]
    fn test_high_diff_frames_kept() {
        let mut p = _VideoPipeline::new();
        p._push_frame(make_frame(0.0, 0));
        p._push_frame(make_frame(1.0, 200));
        p._push_frame(make_frame(2.0, 100));
        p._dedup_frames();
        assert_eq!(p.key_frames, vec![0, 1, 2]);
    }

    #[test]
    fn test_kept_vs_last_kept_not_previous() {
        let mut p = _VideoPipeline::new();
        p._push_frame(make_frame(0.0, 0));
        p._push_frame(make_frame(1.0, 2));
        p._push_frame(make_frame(2.0, 200));
        p._dedup_frames();
        assert_eq!(p.key_frames, vec![0, 2]);
    }

    #[test]
    fn test_mean_diff_identical() {
        let a = make_frame(0.0, 100);
        let b = make_frame(1.0, 100);
        assert!((a._mean_diff(&b) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_mean_diff_maximum() {
        let a = make_frame(0.0, 0);
        let b = make_frame(1.0, 255);
        assert!((a._mean_diff(&b) - 255.0).abs() < 1e-9);
    }

    #[test]
    fn test_mean_diff_half_plane() {
        let mut a = make_frame(0.0, 0);
        for y in 0..8 {
            for x in 0..16 {
                a.grayscale_16x16[y][x] = 255;
            }
        }
        let b = make_frame(1.0, 0);
        let diff = a._mean_diff(&b);
        assert!((diff - 127.5).abs() < 1.0, "expected ~127.5, got {}", diff);
    }

    #[test]
    fn test_process_frames_function() {
        let frames = vec![make_frame(0.0, 0), make_frame(1.0, 0), make_frame(2.0, 100)];
        let summary = _process_frames(frames);
        assert_eq!(summary.frame_count, 3);
        assert_eq!(summary.key_frame_count, 2);
        assert!((summary.duration_secs - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_process_video_missing_file() {
        let result = _process_video("/nonexistent/video.mp4");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_video_frame_serde() {
        let frame = make_frame(1.5, 42);
        let json = serde_json::to_string(&frame).unwrap();
        let back: _VideoFrame = serde_json::from_str(&json).unwrap();
        assert!((back.timestamp - 1.5).abs() < 1e-9);
        assert_eq!(back.data_hash, 42);
        assert_eq!(back.grayscale_16x16[0][0], 42);
    }

    #[test]
    fn test_video_summary_serde() {
        let s = _VideoSummary {
            frame_count: 100,
            key_frame_count: 12,
            duration_secs: 30.0,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: _VideoSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.frame_count, 100);
        assert_eq!(back.key_frame_count, 12);
    }

    #[test]
    fn test_video_extraction_from_html() {
        let mut ext = _VideoExtractor::new();
        let html = r#"<video src="https://example.com/stream.m3u8">"#;
        let streams = ext._extract_from_page_with_page_param("https://example.com", html);
        assert!(!streams.is_empty());
        assert_eq!(streams[0].protocol, _StreamProtocol::HLS);
    }

    #[test]
    fn test_dash_detection() {
        let mut ext = _VideoExtractor::new();
        let html = r#"<source src="https://example.com/video.mpd" type="application/dash+xml">"#;
        let streams = ext._extract_from_page_with_page_param("https://example.com", html);
        assert!(streams.iter().any(|s| s.protocol == _StreamProtocol::DASH));
    }

    #[test]
    fn test_best_stream_selection() {
        let ext = _VideoExtractor::new();
        let streams = vec![
            _StreamInfo {
                url: "low".into(), protocol: _StreamProtocol::HLS, codec: VideoCodec::H264,
                width: 640, height: 360, bitrate_kbps: 1000, fps: 30.0,
                has_audio: true, has_subtitles: false,
            },
            _StreamInfo {
                url: "high".into(), protocol: _StreamProtocol::HLS, codec: VideoCodec::H264,
                width: 1920, height: 1080, bitrate_kbps: 8000, fps: 60.0,
                has_audio: true, has_subtitles: true,
            },
        ];
        let best = ext._best_stream(&streams).unwrap();
        assert_eq!(best.url, "high");
    }

    #[test]
    fn test_extraction_pipeline() {
        let config = TranscodeConfig {
            target_codec: VideoCodec::H264,
            target_width: 1920,
            target_height: 1080,
            target_bitrate_kbps: 8000,
            hardware_accel: false,
            subtitle_burn: false,
            subtitle_language: "eng".into(),
        };
        let mut pipeline = _ExtractionPipeline::new(config);
        let result = pipeline.run("https://example.com/video");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output._extraction_count > 0);
    }

    #[test]
    fn test_url_extraction() {
        let url = _VideoExtractor::extract_url(r#"src="https://example.com/stream.m3u8"#);
        assert_eq!(url, "https://example.com/stream.m3u8");
    }
}
