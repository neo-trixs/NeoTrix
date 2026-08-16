#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use std::time::Instant;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
// File 1: VideoFrame / VideoPipeline / process
// ──────────────────────────────────────────────

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
/// Real decoder path: shells out to `ffmpeg` for scene-aware frame extraction
/// (research absorption_video.md — first keyframe is often a black/logo intro,
/// so `select='gt(scene,0.3)'` picks one frame per visual scene, falling back
/// to uniform 1 FPS for static videos). Each extracted frame is decoded via
/// the `image` crate into the 16×16 grayscale descriptor the dedup comparator
/// expects. When ffmpeg is unavailable or extraction fails, falls back to the
/// previous file-size heuristic so the summary never hard-errors.
pub fn process_video(path: &str) -> Result<VideoSummary, String> {
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
    let mut pipeline = VideoPipeline::new();
    for (i, bytes) in frames.iter().enumerate() {
        let mut frame = VideoFrame {
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
        pipeline.push_frame(frame);
    }
    let _ = std::fs::remove_dir_all(&tmp);

    if pipeline.frames.is_empty() {
        // Fallback: heuristic summary (decoder unavailable).
        let file_size = meta.len();
        let estimated_frames = (file_size / 50_000).max(1);
        let estimated_duration = estimated_frames as f64 / 30.0;
        let frame = VideoFrame {
            timestamp: 0.0,
            data_hash: file_size,
            grayscale_16x16: [[0u8; 16]; 16],
        };
        let mut p2 = VideoPipeline::new();
        p2.push_frame(frame);
        p2.dedup_frames();
        return Ok(VideoSummary {
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
pub fn process_frames(frames: Vec<VideoFrame>) -> VideoSummary {
    let mut pipeline = VideoPipeline {
        frames,
        key_frames: Vec::new(),
    };
    pipeline.process()
}

// ──────────────────────────────────────────────
// File 2: VideoExtractor / Transcoder / Subtitle
//         DeviceDiscovery / ExtractionPipeline
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    AV1,
    MPEG4,
    Unknown,
}

impl VideoCodec {
    pub fn ffmpeg_name(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "h264",
            VideoCodec::H265 => "hevc",
            VideoCodec::VP9 => "vp9",
            VideoCodec::AV1 => "av1",
            VideoCodec::MPEG4 => "mpeg4",
            VideoCodec::Unknown => "copy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamProtocol {
    HLS,
    DASH,
    Progressive,
    RTMP,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub url: String,
    pub protocol: StreamProtocol,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub bitrate_kbps: u32,
    pub fps: f64,
    pub has_audio: bool,
    pub has_subtitles: bool,
}

#[derive(Debug, Clone)]
pub struct TranscodeConfig {
    pub target_codec: VideoCodec,
    pub target_width: u32,
    pub target_height: u32,
    pub target_bitrate_kbps: u32,
    pub hardware_accel: bool,
    pub subtitle_burn: bool,
    pub subtitle_language: String,
}

impl Default for TranscodeConfig {
    fn default() -> Self {
        Self {
            target_codec: VideoCodec::H264,
            target_width: 1920,
            target_height: 1080,
            target_bitrate_kbps: 8000,
            hardware_accel: false,
            subtitle_burn: false,
            subtitle_language: "eng".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranscodeResult {
    pub output_path: String,
    pub duration_ms: u64,
    pub output_size_bytes: u64,
    pub actual_codec: VideoCodec,
    pub hardware_used: bool,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CastProtocol {
    DLNA,
    Chromecast,
    AirPlay,
}

#[derive(Debug, Clone)]
pub struct CastTarget {
    pub name: String,
    pub protocol: CastProtocol,
    pub address: String,
    pub port: u16,
    pub supports_transcoding: bool,
}

pub struct VideoExtractor {
    stream_cache: HashMap<String, StreamInfo>,
    extraction_count: u64,
    last_extraction: Option<Instant>,
}

impl Default for VideoExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoExtractor {
    pub fn new() -> Self {
        Self {
            stream_cache: HashMap::new(),
            extraction_count: 0,
            last_extraction: None,
        }
    }

    pub fn extract_from_page(&mut self, url: &str) -> Vec<StreamInfo> {
        self.extraction_count += 1;
        self.last_extraction = Some(Instant::now());
        let stream = StreamInfo {
            url: url.to_string(),
            protocol: StreamProtocol::HLS,
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

    pub fn extract_from_page_with_page_param(&mut self, url: &str, html: &str) -> Vec<StreamInfo> {
        self.extraction_count += 1;
        self.last_extraction = Some(Instant::now());
        let mut streams = Vec::new();
        for line in html.lines() {
            if line.contains(".m3u8") {
                let stream_url = Self::extract_url(line);
                streams.push(StreamInfo {
                    url: stream_url,
                    protocol: StreamProtocol::HLS,
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
                streams.push(StreamInfo {
                    url: stream_url,
                    protocol: StreamProtocol::DASH,
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
            streams.push(StreamInfo {
                url: url.to_string(),
                protocol: StreamProtocol::Progressive,
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

    pub fn get_cached(&self, url: &str) -> Option<&StreamInfo> {
        self.stream_cache.get(url)
    }

    pub fn extraction_count(&self) -> u64 {
        self.extraction_count
    }

    pub fn best_stream<'a>(&self, streams: &'a [StreamInfo]) -> Option<&'a StreamInfo> {
        streams.iter().max_by(|a, b| {
            (a.width * a.height).cmp(&(b.width * b.height))
                .then_with(|| a.bitrate_kbps.cmp(&b.bitrate_kbps))
        })
    }
}

pub struct Transcoder {
    config: TranscodeConfig,
}

impl Transcoder {
    pub fn new(config: TranscodeConfig) -> Self {
        Self { config }
    }

    pub fn transcode(&self, input: &StreamInfo) -> TranscodeResult {
        let compression = if input.width > 0 && self.config.target_width > 0 {
            (input.width as f64 / self.config.target_width as f64)
                .max(0.0)
                .min(1.0)
        } else {
            1.0
        };
        TranscodeResult {
            output_path: format!("/tmp/transcode_{}.mp4", input.codec.ffmpeg_name()),
            duration_ms: 30000,
            output_size_bytes: (self.config.target_bitrate_kbps as u64 * 30000 / 8 / 1000),
            actual_codec: self.config.target_codec,
            hardware_used: self.config.hardware_accel,
            compression_ratio: compression,
        }
    }

    pub fn config(&self) -> &TranscodeConfig {
        &self.config
    }

    pub fn should_transcode(&self, stream: &StreamInfo) -> bool {
        stream.codec != self.config.target_codec
            || stream.width > self.config.target_width
            || stream.height > self.config.target_height
            || self.config.subtitle_burn
    }
}

pub struct SubtitleEngine;

impl SubtitleEngine {
    pub fn generate(text: &str, _language: &str) -> Vec<SubtitleEntry> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = 10.max(words.len() / 5);
        let mut entries = Vec::new();
        let mut start_ms = 0u64;
        for chunk in words.chunks(chunk_size) {
            let duration_ms = (chunk.len() as u64) * 200;
            entries.push(SubtitleEntry {
                index: entries.len() + 1,
                start_ms,
                end_ms: start_ms + duration_ms,
                text: chunk.join(" "),
            });
            start_ms += duration_ms;
        }
        entries
    }
}

#[derive(Debug, Clone)]
pub struct SubtitleEntry {
    pub index: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct DeviceDiscovery;

impl DeviceDiscovery {
    pub fn scan(protocol: CastProtocol) -> Vec<CastTarget> {
        match protocol {
            CastProtocol::DLNA => vec![CastTarget {
                name: "Living Room TV (DLNA)".into(),
                protocol: CastProtocol::DLNA,
                address: "192.168.1.100".into(),
                port: 8200,
                supports_transcoding: false,
            }],
            CastProtocol::Chromecast => vec![CastTarget {
                name: "Living Room TV (Chromecast)".into(),
                protocol: CastProtocol::Chromecast,
                address: "192.168.1.101".into(),
                port: 8009,
                supports_transcoding: true,
            }],
            CastProtocol::AirPlay => vec![],
        }
    }
}

pub struct ExtractionPipeline {
    extractor: VideoExtractor,
    transcoder: Transcoder,
    pipeline_active: bool,
    total_processed: u64,
}

impl ExtractionPipeline {
    pub fn new(transcode_config: TranscodeConfig) -> Self {
        Self {
            extractor: VideoExtractor::new(),
            transcoder: Transcoder::new(transcode_config),
            pipeline_active: false,
            total_processed: 0,
        }
    }

    pub fn run(&mut self, page_url: &str) -> Result<PipelineOutput, String> {
        self.pipeline_active = true;
        let streams = self.extractor.extract_from_page(page_url);
        if streams.is_empty() {
            return Err("No streams found".into());
        }
        let mut outputs = Vec::new();
        for stream in &streams {
            let needs_transcode = self.transcoder.should_transcode(stream);
            let result = if needs_transcode {
                self.transcoder.transcode(stream)
            } else {
                TranscodeResult {
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
        Ok(PipelineOutput {
            streams: outputs,
            extraction_count: self.extractor.extraction_count(),
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
pub struct PipelineOutput {
    pub streams: Vec<(StreamInfo, TranscodeResult)>,
    pub extraction_count: u64,
}

// ──────────────────────────────────────────────
// NEW: VideoOrchestrator — unifies both pipelines
// ──────────────────────────────────────────────

// ── P2-Checkpoint: 视频链 checkpoint / resume (Claude Workflows 吸收) ──
// 阶段化视频生产链可中断/续跑: 每完成一个阶段落 checkpoint, 重跑从最近
// checkpoint 恢复, 避免 ffmpeg/转码/烧字幕这类重活重复执行。checkpoint
// 由 JSON 序列化 (纯 Rust, 零外部依赖)。

/// 视频生产链阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoStage {
    Extract,
    Transcode,
    Dedup,
    Subtitle,
    Publish,
}

impl VideoStage {
    pub fn label(self) -> &'static str {
        match self {
            VideoStage::Extract => "extract",
            VideoStage::Transcode => "transcode",
            VideoStage::Dedup => "dedup",
            VideoStage::Subtitle => "subtitle",
            VideoStage::Publish => "publish",
        }
    }

    /// 阶段执行顺序索引 (供推进/恢复)。
    fn order(self) -> u8 {
        match self {
            VideoStage::Extract => 0,
            VideoStage::Transcode => 1,
            VideoStage::Dedup => 2,
            VideoStage::Subtitle => 3,
            VideoStage::Publish => 4,
        }
    }

    /// 顺序中的下一个阶段。
    pub fn next(self) -> Option<VideoStage> {
        match self {
            VideoStage::Extract => Some(VideoStage::Transcode),
            VideoStage::Transcode => Some(VideoStage::Dedup),
            VideoStage::Dedup => Some(VideoStage::Subtitle),
            VideoStage::Subtitle => Some(VideoStage::Publish),
            VideoStage::Publish => None,
        }
    }
}

/// 单个阶段的 checkpoint 状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCheckpoint {
    pub stage: VideoStage,
    /// 该阶段是否已完成。
    pub done: bool,
    /// 完成时的时间戳 (unix 秒, 0 = 未完成)。
    pub finished_at: i64,
    /// 阶段产物摘要 (如输出路径/帧数)。
    pub artifact: String,
}

/// 视频链 checkpoint 集合 — 序列化为 JSON 支持持久化/恢复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoChainCheckpoint {
    /// 关联视频源。
    pub source: String,
    /// 各阶段状态 (按顺序)。
    pub stages: Vec<StageCheckpoint>,
    /// 已消耗的 token 预算 (供成本追踪)。
    pub budget_used: u64,
}

impl VideoChainCheckpoint {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            stages: vec![
                StageCheckpoint { stage: VideoStage::Extract, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: VideoStage::Transcode, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: VideoStage::Dedup, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: VideoStage::Subtitle, done: false, finished_at: 0, artifact: String::new() },
                StageCheckpoint { stage: VideoStage::Publish, done: false, finished_at: 0, artifact: String::new() },
            ],
            budget_used: 0,
        }
    }

    /// 标记某阶段完成并记录产物。
    pub fn complete(&mut self, stage: VideoStage, artifact: impl Into<String>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        if let Some(s) = self.stages.iter_mut().find(|s| s.stage == stage) {
            s.done = true;
            s.finished_at = now;
            s.artifact = artifact.into();
        }
    }

    /// 查询阶段是否已完成。
    pub fn is_done(&self, stage: VideoStage) -> bool {
        self.stages.iter().find(|s| s.stage == stage).map(|s| s.done).unwrap_or(false)
    }

    /// 下一个未完成阶段 (从 Extract 顺序推进) — resume 起点。
    pub fn next_pending(&self) -> Option<VideoStage> {
        self.stages.iter().find(|s| !s.done).map(|s| s.stage)
    }

    /// 是否全部完成。
    pub fn all_done(&self) -> bool {
        self.stages.iter().all(|s| s.done)
    }

    /// 序列化为 JSON (持久化)。
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("serialize checkpoint: {}", e))
    }

    /// 从 JSON 恢复。
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("deserialize checkpoint: {}", e))
    }

    /// 记录预算消耗。
    pub fn spend_budget(&mut self, tokens: u64) {
        self.budget_used = self.budget_used.saturating_add(tokens);
    }
}

/// 视频链执行器 — 按阶段推进, 支持中断后从 checkpoint 恢复。
#[derive(Debug, Clone)]
pub struct VideoChainRunner {
    pub checkpoint: VideoChainCheckpoint,
    /// 每阶段最大 token 预算 (0 = 无限)。
    pub stage_budget: u64,
    /// 已执行阶段计数。
    pub executed: u64,
}

impl VideoChainRunner {
    pub fn new(source: &str) -> Self {
        Self {
            checkpoint: VideoChainCheckpoint::new(source),
            stage_budget: 0,
            executed: 0,
        }
    }

    /// 从最近 checkpoint 恢复: 返回下一个待执行阶段 (None = 已全部完成)。
    pub fn resume(&self) -> Option<VideoStage> {
        self.checkpoint.next_pending()
    }

    /// 执行单个阶段 (玩具: 以 stage_budget 为 token 消耗模拟重活)。
    /// 若阶段已 done → 跳过 (resume 语义)。
    pub fn run_stage(&mut self, stage: VideoStage) -> Result<StageCheckpoint, String> {
        if self.checkpoint.is_done(stage) {
            // 已完成的阶段跳过 — 不重复执行
            return self
                .checkpoint
                .stages
                .iter()
                .find(|s| s.stage == stage)
                .cloned()
                .ok_or_else(|| "stage not found".into());
        }
        let cost = 1 + stage.order() as u64;
        if self.stage_budget > 0 && cost > self.stage_budget {
            return Err(format!("stage {} exceeds per-stage budget {}", stage.label(), self.stage_budget));
        }
        let artifact = format!("{}-output", stage.label());
        self.checkpoint.complete(stage, &artifact);
        self.checkpoint.spend_budget(cost);
        self.executed += 1;
        self.checkpoint
            .stages
            .iter()
            .find(|s| s.stage == stage)
            .cloned()
            .ok_or_else(|| "stage not found".into())
    }

    /// 全链执行: 从 resume 起点顺序推进到 Publish (或首个预算失败处)。
    /// 返回已完成阶段列表。
    pub fn run_all(&mut self) -> Result<Vec<VideoStage>, String> {
        let mut done = Vec::new();
        let mut current = self.resume();
        while let Some(stage) = current {
            match self.run_stage(stage) {
                Ok(_) => {
                    done.push(stage);
                    current = stage.next();
                }
                Err(e) => return Err(e),
            }
        }
        Ok(done)
    }
}

// ──────────────────────────────────────────────
// G19 视频生产全链 (MoneyPrinterTurbo 吸收) —
// 脚本 → 素材检索 → TTS 字幕 → 合成 → 发布
// ──────────────────────────────────────────────

/// MoneyPrinterTurbo 生产链阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionStage {
    Script,
    Material,
    Tts,
    Compose,
    Publish,
}

impl ProductionStage {
    pub fn label(self) -> &'static str {
        match self {
            ProductionStage::Script => "script",
            ProductionStage::Material => "material",
            ProductionStage::Tts => "tts",
            ProductionStage::Compose => "compose",
            ProductionStage::Publish => "publish",
        }
    }

    fn order(self) -> u8 {
        match self {
            ProductionStage::Script => 0,
            ProductionStage::Material => 1,
            ProductionStage::Tts => 2,
            ProductionStage::Compose => 3,
            ProductionStage::Publish => 4,
        }
    }

    pub fn next(self) -> Option<ProductionStage> {
        match self {
            ProductionStage::Script => Some(ProductionStage::Material),
            ProductionStage::Material => Some(ProductionStage::Tts),
            ProductionStage::Tts => Some(ProductionStage::Compose),
            ProductionStage::Compose => Some(ProductionStage::Publish),
            ProductionStage::Publish => None,
        }
    }
}

/// 生产链阶段产物。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionArtifact {
    pub stage: ProductionStage,
    pub done: bool,
    pub artifact: String,
}

/// 视频生产链 — MoneyPrinterTurbo 全链, 支持 checkpoint 恢复。
#[derive(Debug, Clone)]
pub struct VideoProductionChain {
    pub topic: String,
    pub stages: Vec<ProductionArtifact>,
    /// 已消耗 token 预算。
    pub budget_used: u64,
    /// 已执行阶段计数。
    pub executed: u64,
}

impl VideoProductionChain {
    pub fn new(topic: &str) -> Self {
        Self {
            topic: topic.to_string(),
            stages: vec![
                ProductionArtifact { stage: ProductionStage::Script, done: false, artifact: String::new() },
                ProductionArtifact { stage: ProductionStage::Material, done: false, artifact: String::new() },
                ProductionArtifact { stage: ProductionStage::Tts, done: false, artifact: String::new() },
                ProductionArtifact { stage: ProductionStage::Compose, done: false, artifact: String::new() },
                ProductionArtifact { stage: ProductionStage::Publish, done: false, artifact: String::new() },
            ],
            budget_used: 0,
            executed: 0,
        }
    }

    /// 下一个未完成阶段 (resume 起点)。
    pub fn next_pending(&self) -> Option<ProductionStage> {
        self.stages.iter().find(|s| !s.done).map(|s| s.stage)
    }

    pub fn all_done(&self) -> bool {
        self.stages.iter().all(|s| s.done)
    }

    /// 执行单个阶段 (脚本为模板生成, 其余为确定性模拟重活)。
    pub fn run_stage(&mut self, stage: ProductionStage) -> Result<ProductionArtifact, String> {
        if self.stages.iter().any(|s| s.stage == stage && s.done) {
            return self
                .stages
                .iter()
                .find(|s| s.stage == stage)
                .cloned()
                .ok_or_else(|| "stage not found".into());
        }
        let artifact = match stage {
            ProductionStage::Script => format!("script-{}", slugify(&self.topic)),
            ProductionStage::Material => format!("materials/{}-assets", slugify(&self.topic)),
            ProductionStage::Tts => format!("tts/{}-subtitles.srt", slugify(&self.topic)),
            ProductionStage::Compose => format!("out/{}-final.mp4", slugify(&self.topic)),
            ProductionStage::Publish => format!("published/{}-final.mp4", slugify(&self.topic)),
        };
        if let Some(s) = self.stages.iter_mut().find(|s| s.stage == stage) {
            s.done = true;
            s.artifact = artifact.clone();
        }
        self.budget_used = self.budget_used.saturating_add(1 + stage.order() as u64);
        self.executed += 1;
        Ok(ProductionArtifact { stage, done: true, artifact })
    }

    /// 全链推进到 Publish。
    pub fn run_all(&mut self) -> Result<Vec<ProductionStage>, String> {
        let mut done = Vec::new();
        let mut current = self.next_pending();
        while let Some(stage) = current {
            self.run_stage(stage)?;
            done.push(stage);
            current = stage.next();
        }
        Ok(done)
    }

    /// 产物摘要 (供入库描述)。
    pub fn output_manifest(&self) -> Vec<(ProductionStage, String)> {
        self.stages.iter().map(|s| (s.stage, s.artifact.clone())).collect()
    }
}

fn slugify(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect::<String>()
}

// ──────────────────────────────────────────────
// G20 资产 ML 富化 (immich 吸收) —
// 资产去重 + 语义标签 (CLIP 式轻量指纹)
// ──────────────────────────────────────────────

/// 单个媒体资产。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: String,
    pub path: String,
    /// 内容指纹 (如感知哈希 / 帧差分签名)。
    pub fingerprint: String,
    /// 已有标签。
    pub tags: Vec<String>,
}

/// immich 风格资产富化结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEnrichment {
    pub asset_id: String,
    /// 判定为重复的资产 id (去重)。
    pub dedup_of: Option<String>,
    /// 语义标签 (CLIP 式: 从路径/指纹推导的主题词)。
    pub semantic_tags: Vec<String>,
    /// 是否需纳入产出清单。
    pub keep: bool,
}

/// 资产富化器 — immich 吸收: 内容去重 + 语义标签。
#[derive(Debug, Clone, Default)]
pub struct AssetEnricher {
    pub seen: HashMap<String, String>,
    pub enriched: Vec<AssetEnrichment>,
}

impl AssetEnricher {
    pub fn new() -> Self {
        Self::default()
    }

    /// 富化单个资产: 指纹重复 → 标记去重; 否则提取语义标签。
    pub fn enrich(&mut self, asset: &MediaAsset) -> AssetEnrichment {
        let mut sem = asset.tags.clone();
        // 从路径/文件名提取主题词 (轻量语义标签)
        for token in asset.path.split(['/', '\\', '.', '_', '-']) {
            let t = token.trim();
            if t.len() >= 4 && t.chars().all(|c| c.is_alphanumeric() || c == '_') {
                let t = t.to_lowercase();
                if !sem.contains(&t) {
                    sem.push(t);
                }
            }
        }
        let result = if let Some(orig) = self.seen.get(&asset.fingerprint) {
            AssetEnrichment {
                asset_id: asset.id.clone(),
                dedup_of: Some(orig.clone()),
                semantic_tags: sem,
                keep: false,
            }
        } else {
            self.seen.insert(asset.fingerprint.clone(), asset.id.clone());
            AssetEnrichment {
                asset_id: asset.id.clone(),
                dedup_of: None,
                semantic_tags: sem,
                keep: true,
            }
        };
        self.enriched.push(result.clone());
        result
    }

    /// 去重统计: (总资产, 判定重复数, 保留数)。
    pub fn stats(&self) -> (usize, usize, usize) {
        let total = self.enriched.len();
        let dup = self.enriched.iter().filter(|e| e.dedup_of.is_some()).count();
        (total, dup, total - dup)
    }
}

// ──────────────────────────────────────────────

/// Unified orchestrator wrapping both frame-level [`VideoPipeline`]
/// and web-based [`ExtractionPipeline`].
pub struct VideoOrchestrator {
    pub frame_pipeline: VideoPipeline,
    pub extraction_pipeline: ExtractionPipeline,
    total_videos_processed: u64,
    transcode_config: TranscodeConfig,
    /// G19 视频生产全链 (MoneyPrinterTurbo 吸收)。
    pub production_chain: Option<VideoProductionChain>,
    /// G20 资产 ML 富化 (immich 吸收)。
    pub asset_enricher: AssetEnricher,
}

impl VideoOrchestrator {
    pub fn new(transcode_config: TranscodeConfig) -> Self {
        Self {
            frame_pipeline: VideoPipeline::new(),
            extraction_pipeline: ExtractionPipeline::new(transcode_config.clone()),
            total_videos_processed: 0,
            transcode_config,
            production_chain: None,
            asset_enricher: AssetEnricher::new(),
        }
    }

    /// G19 运行视频生产全链 (脚本→素材→TTS→合成→发布), 产物入库存档。
    pub fn produce_video(&mut self, topic: &str) -> Result<Vec<(ProductionStage, String)>, String> {
        let mut chain = VideoProductionChain::new(topic);
        chain.run_all()?;
        let manifest = chain.output_manifest();
        self.total_videos_processed += 1;
        self.production_chain = Some(chain);
        Ok(manifest)
    }

    /// G20 富化一批素材 (去重 + 语义标签)。
    pub fn enrich_assets(&mut self, assets: &[MediaAsset]) -> Vec<AssetEnrichment> {
        assets.iter().map(|a| self.asset_enricher.enrich(a)).collect()
    }

    pub fn asset_enrichment_stats(&self) -> (usize, usize, usize) {
        self.asset_enricher.stats()
    }

    pub fn process_video_file(&mut self, path: &str) -> Result<VideoSummary, String> {
        let result = process_video(path)?;
        self.total_videos_processed += 1;
        Ok(result)
    }

    pub fn extract_from_web(&mut self, url: &str) -> Result<PipelineOutput, String> {
        let result = self.extraction_pipeline.run(url);
        if result.is_ok() {
            self.total_videos_processed += 1;
        }
        result
    }

    pub fn total_processed(&self) -> u64 {
        self.total_videos_processed
    }

    pub fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        if self.transcode_config.target_bitrate_kbps == 0 {
            failures.push("transcode config has zero bitrate".into());
        }
        if self.transcode_config.target_width == 0 || self.transcode_config.target_height == 0 {
            failures.push("transcode config has zero dimensions".into());
        }
        if self.extraction_pipeline.is_active() {
            failures.push("extraction pipeline stuck in active state".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── File 1 tests (12) ─────────────────────

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
        p.push_frame(make_frame(1.0, 128));
        p.push_frame(make_frame(2.0, 128));
        p.dedup_frames();
        assert_eq!(p.key_frames, vec![0]);
    }

    #[test]
    fn test_high_diff_frames_kept() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 0));
        p.push_frame(make_frame(1.0, 200));
        p.push_frame(make_frame(2.0, 100));
        p.dedup_frames();
        assert_eq!(p.key_frames, vec![0, 1, 2]);
    }

    #[test]
    fn test_kept_vs_last_kept_not_previous() {
        let mut p = VideoPipeline::new();
        p.push_frame(make_frame(0.0, 0));
        p.push_frame(make_frame(1.0, 2));
        p.push_frame(make_frame(2.0, 200));
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

    // ── File 2 tests (10) ─────────────────────

    #[test]
    fn test_video_extraction_from_html() {
        let mut ext = VideoExtractor::new();
        let html = r#"<video src="https://example.com/stream.m3u8">"#;
        let streams = ext.extract_from_page_with_page_param("https://example.com", html);
        assert!(!streams.is_empty());
        assert_eq!(streams[0].protocol, StreamProtocol::HLS);
    }

    #[test]
    fn test_dash_detection() {
        let mut ext = VideoExtractor::new();
        let html = r#"<source src="https://example.com/video.mpd" type="application/dash+xml">"#;
        let streams = ext.extract_from_page_with_page_param("https://example.com", html);
        assert!(streams.iter().any(|s| s.protocol == StreamProtocol::DASH));
    }

    #[test]
    fn test_best_stream_selection() {
        let ext = VideoExtractor::new();
        let streams = vec![
            StreamInfo {
                url: "low".into(), protocol: StreamProtocol::HLS, codec: VideoCodec::H264,
                width: 640, height: 360, bitrate_kbps: 1000, fps: 30.0,
                has_audio: true, has_subtitles: false,
            },
            StreamInfo {
                url: "high".into(), protocol: StreamProtocol::HLS, codec: VideoCodec::H264,
                width: 1920, height: 1080, bitrate_kbps: 8000, fps: 60.0,
                has_audio: true, has_subtitles: true,
            },
        ];
        let best = ext.best_stream(&streams).unwrap();
        assert_eq!(best.url, "high");
    }

    #[test]
    fn test_transcode_decision() {
        let config = TranscodeConfig::default();
        let transcoder = Transcoder::new(config);
        let stream = StreamInfo {
            url: "test".into(), protocol: StreamProtocol::HLS, codec: VideoCodec::VP9,
            width: 3840, height: 2160, bitrate_kbps: 20000, fps: 60.0,
            has_audio: true, has_subtitles: false,
        };
        assert!(transcoder.should_transcode(&stream));
        let h264_stream = StreamInfo {
            url: "test".into(), protocol: StreamProtocol::HLS, codec: VideoCodec::H264,
            width: 1920, height: 1080, bitrate_kbps: 8000, fps: 30.0,
            has_audio: true, has_subtitles: false,
        };
        assert!(!transcoder.should_transcode(&h264_stream));
    }

    #[test]
    fn test_subtitle_generation() {
        let entries = SubtitleEngine::generate("Hello world this is a test of subtitle generation from whisper", "eng");
        assert!(!entries.is_empty());
        assert_eq!(entries[0].index, 1);
        assert!(entries[0].end_ms > entries[0].start_ms);
    }

    #[test]
    fn test_device_discovery() {
        let dlna = DeviceDiscovery::scan(CastProtocol::DLNA);
        assert!(!dlna.is_empty());
        assert_eq!(dlna[0].protocol, CastProtocol::DLNA);
        let airplay = DeviceDiscovery::scan(CastProtocol::AirPlay);
        assert!(airplay.is_empty());
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
        let mut pipeline = ExtractionPipeline::new(config);
        let result = pipeline.run("https://example.com/video");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.extraction_count > 0);
    }

    #[test]
    fn test_url_extraction() {
        let url = VideoExtractor::extract_url(r#"src="https://example.com/stream.m3u8"#);
        assert_eq!(url, "https://example.com/stream.m3u8");
    }

    #[test]
    fn test_codec_ffmpeg_names() {
        assert_eq!(VideoCodec::H264.ffmpeg_name(), "h264");
        assert_eq!(VideoCodec::AV1.ffmpeg_name(), "av1");
    }

    // ── NEW test (1) ──────────────────────────

    #[test]
    fn test_orchestrator_unified() {
        let config = TranscodeConfig::default();
        let mut orchestrator = VideoOrchestrator::new(config);

        assert!(orchestrator.self_test().is_ok());
        assert_eq!(orchestrator.total_processed(), 0);

        let extract_result = orchestrator.extract_from_web("https://example.com/video");
        assert!(extract_result.is_ok());
        assert_eq!(orchestrator.total_processed(), 1);

        let file_result = orchestrator.process_video_file("/nonexistent/video.mp4");
        assert!(file_result.is_err());
        assert_eq!(orchestrator.total_processed(), 1);

        let output = extract_result.unwrap();
        assert!(!output.streams.is_empty());
        assert_eq!(output.streams[0].0.protocol, StreamProtocol::HLS);
        assert_eq!(output.streams[0].1.actual_codec, VideoCodec::H264);
    }

    // ── P2-Checkpoint tests ────────────────────────────────────────────

    #[test]
    fn checkpoint_fresh_starts_at_extract() {
        let cp = VideoChainCheckpoint::new("video.mp4");
        assert_eq!(cp.next_pending(), Some(VideoStage::Extract));
        assert!(!cp.all_done());
    }

    #[test]
    fn checkpoint_complete_advances_pending() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        cp.complete(VideoStage::Extract, "frames/");
        assert!(cp.is_done(VideoStage::Extract));
        assert_eq!(cp.next_pending(), Some(VideoStage::Transcode));
    }

    #[test]
    fn checkpoint_all_done_returns_none_pending() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        for stage in [VideoStage::Extract, VideoStage::Transcode, VideoStage::Dedup, VideoStage::Subtitle, VideoStage::Publish] {
            cp.complete(stage, "ok");
        }
        assert!(cp.all_done());
        assert_eq!(cp.next_pending(), None);
    }

    #[test]
    fn checkpoint_json_roundtrip() {
        let mut cp = VideoChainCheckpoint::new("v.mp4");
        cp.complete(VideoStage::Extract, "frames/");
        cp.spend_budget(42);
        let json = cp.to_json().unwrap();
        let back = VideoChainCheckpoint::from_json(&json).unwrap();
        assert_eq!(back.source, "v.mp4");
        assert!(back.is_done(VideoStage::Extract));
        assert!(!back.is_done(VideoStage::Transcode));
        assert_eq!(back.budget_used, 42);
    }

    #[test]
    fn runner_skips_completed_stages_on_resume() {
        // 模拟中断: Extract+Transcode 已完成 → resume 从 Dedup 开始。
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.checkpoint.complete(VideoStage::Extract, "f/");
        runner.checkpoint.complete(VideoStage::Transcode, "t/");
        assert_eq!(runner.resume(), Some(VideoStage::Dedup));
        runner.run_stage(VideoStage::Extract).unwrap(); // 已完成 → 跳过, 不重执行
        assert_eq!(runner.executed, 0, "completed stage must not re-execute");
        runner.run_stage(VideoStage::Dedup).unwrap();
        assert_eq!(runner.executed, 1);
    }

    #[test]
    fn runner_run_all_completes_chain() {
        let mut runner = VideoChainRunner::new("v.mp4");
        let done = runner.run_all().unwrap();
        assert_eq!(done.len(), 5);
        assert!(runner.checkpoint.all_done());
        assert_eq!(runner.checkpoint.budget_used, 1 + 2 + 3 + 4 + 5);
    }

    #[test]
    fn runner_budget_blocks_expensive_stage() {
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.stage_budget = 2; // Dedup(order2) 需 cost 3 → 超限
        let err = runner.run_all().unwrap_err();
        assert!(err.contains("dedup"), "err: {err}");
        // Extract+Transcode 已落 checkpoint, 后续未执行
        assert!(runner.checkpoint.is_done(VideoStage::Extract));
        assert!(runner.checkpoint.is_done(VideoStage::Transcode));
        assert!(!runner.checkpoint.is_done(VideoStage::Dedup));
    }

    #[test]
    fn runner_persist_and_recover() {
        let mut runner = VideoChainRunner::new("v.mp4");
        runner.run_stage(VideoStage::Extract).unwrap();
        runner.run_stage(VideoStage::Transcode).unwrap();
        let json = runner.checkpoint.to_json().unwrap();
        drop(runner);

        let mut recovered = VideoChainRunner::new("v.mp4");
        recovered.checkpoint = VideoChainCheckpoint::from_json(&json).unwrap();
        assert_eq!(recovered.resume(), Some(VideoStage::Dedup));
        let done = recovered.run_all().unwrap();
        assert_eq!(done, vec![VideoStage::Dedup, VideoStage::Subtitle, VideoStage::Publish]);
        assert!(recovered.checkpoint.all_done());
    }

    #[test]
    fn stage_order_and_next() {
        assert_eq!(VideoStage::Extract.next(), Some(VideoStage::Transcode));
        assert_eq!(VideoStage::Subtitle.next(), Some(VideoStage::Publish));
        assert_eq!(VideoStage::Publish.next(), None);
        assert_eq!(VideoStage::Extract.order(), 0);
        assert_eq!(VideoStage::Publish.order(), 4);
    }

    // ── G19 VideoProductionChain tests ─────────────────────────────────

    #[test]
    fn production_chain_full_run_manifest() {
        let mut chain = VideoProductionChain::new("Rust Async Deep Dive");
        assert_eq!(chain.next_pending(), Some(ProductionStage::Script));
        let done = chain.run_all().unwrap();
        assert_eq!(done.len(), 5, "全链 5 阶段");
        assert!(chain.all_done());
        let manifest = chain.output_manifest();
        assert!(manifest.iter().any(|(s, a)| *s == ProductionStage::Compose && a.contains("final.mp4")));
        assert!(chain.executed >= 5);
    }

    #[test]
    fn production_chain_resume_from_partial() {
        let mut chain = VideoProductionChain::new("T");
        chain.run_stage(ProductionStage::Script).unwrap();
        chain.run_stage(ProductionStage::Material).unwrap();
        assert_eq!(chain.next_pending(), Some(ProductionStage::Tts), "从 TTS 恢复");
        let remaining = chain.run_all().unwrap();
        assert_eq!(remaining, vec![ProductionStage::Tts, ProductionStage::Compose, ProductionStage::Publish]);
    }

    #[test]
    fn production_chain_skips_completed_stage() {
        let mut chain = VideoProductionChain::new("T");
        chain.run_stage(ProductionStage::Script).unwrap();
        let before = chain.executed;
        chain.run_stage(ProductionStage::Script).unwrap(); // 已完成 → 跳过
        assert_eq!(chain.executed, before, "不重复执行");
    }

    // ── G20 AssetEnricher tests ────────────────────────────────────────

    #[test]
    fn asset_enricher_dedups_by_fingerprint() {
        let mut enr = AssetEnricher::new();
        let a = MediaAsset { id: "a1".into(), path: "cats/happy_cat.png".into(), fingerprint: "fp1".into(), tags: vec!["cat".into()] };
        let b = MediaAsset { id: "a2".into(), path: "cats/happy_cat_copy.png".into(), fingerprint: "fp1".into(), tags: vec![] };
        let ea = enr.enrich(&a);
        assert_eq!(ea.keep, true);
        let eb = enr.enrich(&b);
        assert_eq!(eb.dedup_of.as_deref(), Some("a1"), "同指纹判定重复");
        assert_eq!(eb.keep, false);
        let (total, dup, kept) = enr.stats();
        assert_eq!((total, dup, kept), (2, 1, 1));
    }

    #[test]
    fn asset_enricher_semantic_tags_from_path() {
        let mut enr = AssetEnricher::new();
        let asset = MediaAsset {
            id: "a1".into(),
            path: "materials/nature_sunset_beach.png".into(),
            fingerprint: "f".into(),
            tags: vec!["photo".into()],
        };
        let e = enr.enrich(&asset);
        assert!(e.semantic_tags.contains(&"nature".to_string()), "路径 token 提取语义标签");
        assert!(e.semantic_tags.contains(&"photo".to_string()));
    }

    #[test]
    fn orchestrator_produce_and_enrich_wired() {
        let cfg = TranscodeConfig {
            target_bitrate_kbps: 4000,
            target_width: 1920,
            target_height: 1080,
            ..TranscodeConfig::default()
        };
        let mut orch = VideoOrchestrator::new(cfg);
        let manifest = orch.produce_video("Consciousness Engineering").unwrap();
        assert_eq!(manifest.len(), 5);
        let enriched = orch.enrich_assets(&[
            MediaAsset { id: "x".into(), path: "a/b_shot.png".into(), fingerprint: "F".into(), tags: vec![] },
            MediaAsset { id: "y".into(), path: "a/b_shot.png".into(), fingerprint: "F".into(), tags: vec![] },
        ]);
        assert_eq!(enriched[1].dedup_of.as_deref(), Some("x"));
        assert_eq!(orch.total_processed(), 1);
        let (_, dup, _) = orch.asset_enrichment_stats();
        assert_eq!(dup, 1);
    }
}

// ──────────────────────────────────────────────────────────────
// P22 media_sniff — 媒体流嗅探 + m3u8 管线 (res-downloader 吸收)
// MITM 嗅探分类 + 分片清单构建 + 顺序播放游标; 纯确定性, 无真实网络。
// ──────────────────────────────────────────────────────────────

/// 媒体类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    /// HLS (m3u8)。
    Hls,
    /// DASH (mpd)。
    Dash,
    /// 渐进式下载 (mp4/webm)。
    Progressive,
    /// 直播流。
    Live,
}

impl MediaKind {
    pub fn label(self) -> &'static str {
        match self {
            MediaKind::Hls => "hls",
            MediaKind::Dash => "dash",
            MediaKind::Progressive => "progressive",
            MediaKind::Live => "live",
        }
    }
}

/// MITM 嗅探到的媒体资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffedMedia {
    pub url: String,
    pub kind: MediaKind,
    pub headers: Vec<(String, String)>,
}

/// HTTP 响应快照 (嗅探输入, 本文件定义)。
#[derive(Debug, Clone)]
pub struct HttpSniff {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpSniff {
    /// 小写化 Content-Type 头值 (无则空串)。
    pub fn content_type(&self) -> String {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.to_ascii_lowercase())
            .unwrap_or_default()
    }
}

/// 确定性媒体类型检测: m3u8 → Hls; mpd → Dash; mp4/webm → Progressive;
/// "live" token → Live; 其余 None。
fn detect_kind(sniff: &HttpSniff) -> Option<MediaKind> {
    let url = sniff.url.to_ascii_lowercase();
    let ct = sniff.content_type();
    let body = sniff.body.to_ascii_lowercase();
    if url.contains("m3u8") || ct.contains("application/vnd.apple.mpegurl") {
        return Some(MediaKind::Hls);
    }
    if url.contains("mpd") || ct.contains("application/dash+xml") {
        return Some(MediaKind::Dash);
    }
    if ct.contains("video/mp4") || ct.contains("video/webm") {
        return Some(MediaKind::Progressive);
    }
    if url.contains("live") || body.contains("live") {
        return Some(MediaKind::Live);
    }
    None
}

/// 单个分片 (含 f64 时长, 不实现 Eq)。
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub uri: String,
    pub duration_s: f64,
    pub index: u32,
}

/// 播放清单。
#[derive(Debug, Clone)]
pub struct MediaPlaylist {
    pub segments: Vec<Segment>,
    pub duration_total_s: f64,
}

/// 顺序播放游标 (循环)。
pub struct PipelineStatus {
    cursor: usize,
    cycle_len: usize,
    steps_in_cycle: usize,
    cycles_completed: usize,
}

impl Default for PipelineStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineStatus {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            cycle_len: 0,
            steps_in_cycle: 0,
            cycles_completed: 0,
        }
    }

    /// 取下一分片 (循环播放); 空清单返回 None。
    pub fn next_segment(&mut self, playlist: &MediaPlaylist) -> Option<Segment> {
        if playlist.segments.is_empty() {
            return None;
        }
        self.cycle_len = playlist.segments.len();
        let seg = playlist.segments[self.cursor].clone();
        self.cursor = (self.cursor + 1) % playlist.segments.len();
        self.steps_in_cycle = self.cursor;
        if self.cursor == 0 {
            self.cycles_completed += 1;
        }
        Some(seg)
    }

    /// 当前循环内进度 0..1 (空清单为 0)。
    pub fn progress(&self) -> f64 {
        if self.cycle_len == 0 {
            0.0
        } else {
            self.steps_in_cycle as f64 / self.cycle_len as f64
        }
    }

    pub fn cycles_completed(&self) -> usize {
        self.cycles_completed
    }
}

/// 媒体嗅探器 — 记录嗅探历史 + 播放状态。
pub struct MediaSniffer {
    pub sniffed: Vec<SniffedMedia>,
    pub status: PipelineStatus,
}

impl Default for MediaSniffer {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaSniffer {
    pub fn new() -> Self {
        Self {
            sniffed: Vec::new(),
            status: PipelineStatus::new(),
        }
    }

    /// 嗅探单条 HTTP 响应 → 命中则记录并返回; 未知返回 None。
    pub fn sniff(&mut self, http_response: &HttpSniff) -> Option<SniffedMedia> {
        let kind = detect_kind(http_response)?;
        let media = SniffedMedia {
            url: http_response.url.clone(),
            kind,
            headers: http_response.headers.clone(),
        };
        self.sniffed.push(media.clone());
        Some(media)
    }

    /// 生成 variant URL + N 个顺序分片 (index 0..segments)。
    pub fn build_playlist(
        &mut self,
        master_uri: &str,
        variant: &str,
        segments: u32,
        seg_dur: f64,
    ) -> MediaPlaylist {
        let base = format!("{}/{}", master_uri.trim_end_matches('/'), variant);
        let segs = (0..segments)
            .map(|i| Segment {
                uri: format!("{base}/seg_{i:04}.ts"),
                duration_s: seg_dur,
                index: i,
            })
            .collect();
        MediaPlaylist {
            segments: segs,
            duration_total_s: segments as f64 * seg_dur,
        }
    }

    pub fn sniffed_count(&self) -> usize {
        self.sniffed.len()
    }
}

/// SelfTest (T1): "nt_world_video_pipeline_media_sniff" — 嗅探/清单/游标自检。
impl crate::core::nt_core_self_test::SelfTest for MediaSniffer {
    fn name(&self) -> &str {
        "nt_world_video_pipeline_media_sniff"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut s = MediaSniffer::new();
        let hls = HttpSniff {
            url: "https://cdn.example.com/playlist.m3u8".into(),
            headers: vec![("content-type".into(), "application/vnd.apple.mpegurl".into())],
            body: String::new(),
        };
        match s.sniff(&hls) {
            Some(m) => {
                if m.kind != MediaKind::Hls {
                    failures.push("m3u8 sniff must classify as Hls".into());
                }
            }
            None => failures.push("m3u8 sniff returned None".into()),
        }
        let pl = s.build_playlist("https://cdn.example.com/master.m3u8", "720p", 3, 4.0);
        if pl.segments.len() != 3 {
            failures.push("playlist should have 3 segments".into());
        }
        match s.status.next_segment(&pl) {
            Some(first) => {
                if first.index != 0 {
                    failures.push("first segment index should be 0".into());
                }
            }
            None => failures.push("next_segment returned None for non-empty playlist".into()),
        }
        if s.status.progress() <= 0.0 {
            failures.push("progress should advance after next_segment".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod media_sniff_tests {
    use super::*;

    fn sniff(url: &str, ct: &str, body: &str) -> HttpSniff {
        HttpSniff {
            url: url.to_string(),
            headers: vec![("content-type".into(), ct.into())],
            body: body.to_string(),
        }
    }

    #[test]
    fn media_sniff_hls_from_m3u8_url() {
        let mut s = MediaSniffer::new();
        let m = s.sniff(&sniff("https://cdn.example.com/master.m3u8", "text/plain", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Hls);
        assert_eq!(s.sniffed_count(), 1);
    }

    #[test]
    fn media_sniff_hls_from_content_type() {
        let mut s = MediaSniffer::new();
        let m = s.sniff(&sniff("https://cdn.example.com/stream", "application/vnd.apple.mpegurl", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Hls);
    }

    #[test]
    fn media_sniff_dash_from_mpd_url() {
        let mut s = MediaSniffer::new();
        let m = s.sniff(&sniff("https://cdn.example.com/video.mpd", "application/dash+xml", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Dash);
    }

    #[test]
    fn media_sniff_progressive_from_video_content_type() {
        let mut s = MediaSniffer::new();
        let mp4 = s.sniff(&sniff("https://cdn.example.com/movie.mp4", "video/mp4", "")).expect("sniffed");
        assert_eq!(mp4.kind, MediaKind::Progressive);
        let webm = s.sniff(&sniff("https://cdn.example.com/clip", "video/webm", "")).expect("sniffed");
        assert_eq!(webm.kind, MediaKind::Progressive);
    }

    #[test]
    fn media_sniff_live_from_token() {
        let mut s = MediaSniffer::new();
        let url = s.sniff(&sniff("https://cdn.example.com/live/room1", "text/html", "")).expect("sniffed");
        assert_eq!(url.kind, MediaKind::Live);
        let body = s.sniff(&sniff("https://cdn.example.com/room", "text/html", "this is a live stream")).expect("sniffed");
        assert_eq!(body.kind, MediaKind::Live);
    }

    #[test]
    fn media_sniff_unknown_returns_none() {
        let mut s = MediaSniffer::new();
        assert!(s.sniff(&sniff("https://cdn.example.com/other", "text/html", "static page")).is_none());
        assert_eq!(s.sniffed_count(), 0);
    }

    #[test]
    fn media_sniff_playlist_segments_count_and_indices() {
        let mut s = MediaSniffer::new();
        let pl = s.build_playlist("https://cdn.example.com/master.m3u8", "1080p", 5, 6.0);
        assert_eq!(pl.segments.len(), 5);
        assert!((pl.duration_total_s - 30.0).abs() < 1e-9);
        for (i, seg) in pl.segments.iter().enumerate() {
            assert_eq!(seg.index, i as u32);
            assert_eq!(seg.duration_s, 6.0);
            assert!(seg.uri.contains("1080p"));
            assert!(seg.uri.ends_with(&format!("seg_{i:04}.ts")));
        }
    }

    #[test]
    fn media_sniff_next_segment_cycles() {
        let mut s = MediaSniffer::new();
        let pl = s.build_playlist("https://cdn.example.com/master.m3u8", "720p", 3, 4.0);
        let got: Vec<u32> = (0..5)
            .filter_map(|_| s.status.next_segment(&pl).map(|seg| seg.index))
            .collect();
        assert_eq!(got, vec![0, 1, 2, 0, 1], "sequential cursor must cycle");
        assert_eq!(s.status.cycles_completed(), 1);
    }

    #[test]
    fn media_sniff_next_segment_empty_playlist_none() {
        let mut s = MediaSniffer::new();
        let pl = MediaPlaylist {
            segments: vec![],
            duration_total_s: 0.0,
        };
        assert!(s.status.next_segment(&pl).is_none());
        assert_eq!(s.status.progress(), 0.0);
        assert_eq!(s.status.cycles_completed(), 0);
    }

    #[test]
    fn media_sniff_progress_tracks_cycle() {
        let mut s = MediaSniffer::new();
        let pl = s.build_playlist("https://cdn.example.com/master.m3u8", "720p", 4, 4.0);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.25).abs() < 1e-9);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.5).abs() < 1e-9);
        s.status.next_segment(&pl);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.0).abs() < 1e-9, "wrap must reset progress");
    }

    #[test]
    fn media_sniff_selftest_name_matches() {
        use crate::core::nt_core_self_test::SelfTest;
        let s = MediaSniffer::new();
        assert_eq!(s.name(), "nt_world_video_pipeline_media_sniff");
        assert!(s.self_test().is_ok());
    }
}

// ──────────────────────────────────────────────────────────────
// P8 translate_dub — 端到端视频翻译+配音链 (KrillinAI 吸收)
// 语音转写→翻译→对齐→TTS 配音→合并音轨; 模块化链式处理, 多语种。
// ──────────────────────────────────────────────────────────────

/// 配音链单个阶段 (order 决定执行顺序)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DubStage {
    pub order: u8,
    pub name: String,
    pub language: String,
}

/// 待翻译的时间段 — 语音转写/翻译/配音对齐的最小单元。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranslationSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub source: String,
    pub translated: String,
}

/// 配音作业 — 源/目标语言 + 阶段链 + 待处理时间段。
#[derive(Debug, Clone)]
pub struct DubJob {
    pub source_lang: String,
    pub target_lang: String,
    pub stages: Vec<DubStage>,
    pub segments: Vec<TranslationSegment>,
}

impl Default for DubJob {
    fn default() -> Self {
        Self {
            source_lang: "zh".into(),
            target_lang: "en".into(),
            stages: vec![
                DubStage { order: 0, name: "transcribe".into(), language: "zh".into() },
                DubStage { order: 1, name: "translate".into(), language: "en".into() },
                DubStage { order: 2, name: "align".into(), language: "en".into() },
                DubStage { order: 3, name: "tts".into(), language: "en".into() },
                DubStage { order: 4, name: "merge".into(), language: "en".into() },
            ],
            segments: Vec::new(),
        }
    }
}

/// 配音流水线 — 按 `enabled_stages` 逐段驱动整个翻译+配音链。
#[derive(Debug, Clone)]
pub struct DubPipeline {
    pub enabled_stages: Vec<String>,
    pub max_segment_ms: u64,
    pub overlap_ms: u64,
}

impl Default for DubPipeline {
    fn default() -> Self {
        Self {
            enabled_stages: vec![
                "transcribe".into(),
                "translate".into(),
                "align".into(),
                "tts".into(),
                "merge".into(),
            ],
            max_segment_ms: 10000,
            overlap_ms: 250,
        }
    }
}

impl DubPipeline {
    /// 内置 5 阶段 (顺序即执行顺序)。
    const BUILTIN_STAGES: [&'static str; 5] =
        ["transcribe", "translate", "align", "tts", "merge"];

    /// 创建流水线 (仅启用指定阶段)。
    pub fn new(enabled_stages: Vec<String>) -> Self {
        Self {
            enabled_stages,
            max_segment_ms: 10000,
            overlap_ms: 250,
        }
    }

    /// 逐阶段逐段生成动作描述; 遇到非内置阶段即中止报错。
    pub fn run(&self, job: &mut DubJob) -> Result<Vec<String>, String> {
        let mut actions = Vec::new();
        for stage in &self.enabled_stages {
            if !Self::BUILTIN_STAGES.contains(&stage.as_str()) {
                return Err(format!("unknown stage: {}", stage));
            }
            for (i, seg) in job.segments.iter().enumerate() {
                actions.push(format!(
                    "{} seg {} ({}-{}ms)",
                    stage, i, seg.start_ms, seg.end_ms
                ));
            }
        }
        Ok(actions)
    }

    /// 翻译单段: 在译文前加目标语言标记 (确定性玩具实现)。
    pub fn translate_segment(&self, seg: &TranslationSegment, target: &str) -> TranslationSegment {
        TranslationSegment {
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            source: seg.source.clone(),
            translated: format!("[{}] {}", target, seg.source),
        }
    }

    /// 估算配音总时长 = 各段 (end_ms - start_ms) 之和。
    pub fn estimate_dub_duration(&self, job: &DubJob) -> u64 {
        job.segments
            .iter()
            .map(|s| s.end_ms.saturating_sub(s.start_ms))
            .sum()
    }
}

/// SelfTest (T1): "nt_world_video_pipeline_dub" — 配音链自检。
impl crate::core::nt_core_self_test::SelfTest for DubPipeline {
    fn name(&self) -> &str {
        "nt_world_video_pipeline_dub"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut job = DubJob::default();
        job.segments.push(TranslationSegment {
            start_ms: 0,
            end_ms: 10000,
            source: "hello".into(),
            translated: String::new(),
        });
        match self.run(&mut job) {
            Ok(actions) => {
                if actions.len() != 5 {
                    failures.push("default run should emit 5 actions".into());
                }
            }
            Err(e) => failures.push(format!("default run failed: {}", e)),
        }
        let bad = DubPipeline::new(vec!["bogus".into()]);
        if bad.run(&mut job).is_ok() {
            failures.push("unknown stage must error".into());
        }
        let t = self.translate_segment(&job.segments[0], "en");
        if t.translated != "[en] hello" {
            failures.push("translate_segment must tag target language".into());
        }
        if self.estimate_dub_duration(&job) != 10000 {
            failures.push("duration estimate mismatch".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod dub_pipeline_tests {
    use super::*;

    fn sample_job() -> DubJob {
        let mut job = DubJob::default();
        job.segments.push(TranslationSegment {
            start_ms: 0,
            end_ms: 10000,
            source: "hello world".into(),
            translated: String::new(),
        });
        job.segments.push(TranslationSegment {
            start_ms: 10000,
            end_ms: 15000,
            source: "second line".into(),
            translated: String::new(),
        });
        job
    }

    #[test]
    fn dub_pipeline_default_has_five_stages() {
        let p = DubPipeline::default();
        assert_eq!(
            p.enabled_stages,
            vec!["transcribe", "translate", "align", "tts", "merge"]
        );
        assert_eq!(p.max_segment_ms, 10000);
        assert_eq!(p.overlap_ms, 250);
        let job = DubJob::default();
        assert_eq!(job.stages.len(), 5);
        assert_eq!(job.stages[0].name, "transcribe");
        assert_eq!(job.stages[4].name, "merge");
        assert_eq!(job.stages[0].order, 0);
        assert_eq!(job.stages[4].order, 4);
    }

    #[test]
    fn dub_pipeline_unknown_stage_errors() {
        let p = DubPipeline::new(vec!["transcribe".into(), "mix".into()]);
        let mut job = sample_job();
        let err = p.run(&mut job).unwrap_err();
        assert!(err.contains("unknown stage: mix"), "err: {}", err);
    }

    #[test]
    fn dub_pipeline_translate_segment_marks_target_language() {
        let p = DubPipeline::default();
        let seg = TranslationSegment {
            start_ms: 0,
            end_ms: 5000,
            source: "你好".into(),
            translated: String::new(),
        };
        let out = p.translate_segment(&seg, "en");
        assert_eq!(out.translated, "[en] 你好");
        assert_eq!(out.start_ms, seg.start_ms);
        assert_eq!(out.end_ms, seg.end_ms);
        assert_eq!(out.source, seg.source);
    }

    #[test]
    fn dub_pipeline_duration_estimate_sums_segments() {
        let p = DubPipeline::default();
        let mut job = sample_job();
        assert_eq!(p.estimate_dub_duration(&job), 15000);
        job.segments.clear();
        assert_eq!(p.estimate_dub_duration(&job), 0);
    }

    #[test]
    fn dub_pipeline_run_covers_all_enabled_stages() {
        let p = DubPipeline::new(vec!["transcribe".into(), "tts".into()]);
        let mut job = sample_job();
        let actions = p.run(&mut job).unwrap();
        assert_eq!(actions.len(), 4, "2 stages x 2 segments");
        assert_eq!(actions[0], "transcribe seg 0 (0-10000ms)");
        assert_eq!(actions[1], "transcribe seg 1 (10000-15000ms)");
        assert_eq!(actions[2], "tts seg 0 (0-10000ms)");
        assert_eq!(actions[3], "tts seg 1 (10000-15000ms)");
        for stage in ["transcribe", "tts"] {
            assert!(actions.iter().any(|a| a.starts_with(stage)));
        }
    }

    #[test]
    fn dub_pipeline_run_empty_segments_yields_no_actions() {
        let p = DubPipeline::default();
        let mut job = DubJob::default();
        let actions = p.run(&mut job).unwrap();
        assert!(actions.is_empty());
    }

    #[test]
    fn dub_pipeline_selftest_matches() {
        use crate::core::nt_core_self_test::SelfTest;
        let p = DubPipeline::default();
        assert_eq!(p.name(), "nt_world_video_pipeline_dub");
        assert!(p.self_test().is_ok());
    }
}
