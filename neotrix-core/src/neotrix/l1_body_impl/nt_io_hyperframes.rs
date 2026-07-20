use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoFormat {
    Mp4,
    WebM,
    Gif,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::WebM => "webm",
            VideoFormat::Gif => "gif",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "video/mp4",
            VideoFormat::WebM => "video/webm",
            VideoFormat::Gif => "image/gif",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VideoResolution {
    pub width: u32,
    pub height: u32,
}

impl VideoResolution {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub static PRESET_RESOLUTIONS: LazyLock<HashMap<&'static str, VideoResolution>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("720p", VideoResolution::new(1280, 720));
    m.insert("1080p", VideoResolution::new(1920, 1080));
    m.insert("vertical", VideoResolution::new(1080, 1920));
    m.insert("square", VideoResolution::new(1080, 1080));
    m
});

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub format: VideoFormat,
    pub resolution: VideoResolution,
    pub fps: u32,
    pub bitrate_kbps: u32,
    pub background_color: String,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            format: VideoFormat::Mp4,
            resolution: VideoResolution::new(1920, 1080),
            fps: 30,
            bitrate_kbps: 5000,
            background_color: "#000000".to_string(),
        }
    }
}

impl RenderConfig {
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = VideoResolution::new(width, height);
        self
    }

    pub fn with_format(mut self, format: VideoFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }
}

#[derive(Debug, Clone)]
pub struct VideoClip {
    pub html: String,
    pub duration_secs: f64,
    pub transition_in: Option<String>,
    pub transition_out: Option<String>,
}

impl VideoClip {
    pub fn new(html: impl Into<String>, duration_secs: f64) -> Self {
        Self {
            html: html.into(),
            duration_secs,
            transition_in: None,
            transition_out: None,
        }
    }

    pub fn with_transition_in(mut self, name: impl Into<String>) -> Self {
        self.transition_in = Some(name.into());
        self
    }

    pub fn with_transition_out(mut self, name: impl Into<String>) -> Self {
        self.transition_out = Some(name.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct RenderJob {
    pub clips: Vec<VideoClip>,
    pub config: RenderConfig,
    pub output_path: String,
}

impl RenderJob {
    pub fn new(output_path: impl Into<String>) -> Self {
        Self {
            clips: Vec::new(),
            config: RenderConfig::default(),
            output_path: output_path.into(),
        }
    }

    pub fn add_clip(&mut self, clip: VideoClip) {
        self.clips.push(clip);
    }

    pub fn total_duration(&self) -> f64 {
        self.clips.iter().map(|c| c.duration_secs).sum()
    }

    pub fn estimated_size_mb(&self) -> f64 {
        let bitrate_bps = self.config.bitrate_kbps as f64 * 1000.0;
        let total_seconds = self.total_duration();
        (bitrate_bps * total_seconds) / 8.0 / 1024.0 / 1024.0
    }
}

pub struct HyperframesEngine;

impl HyperframesEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn prepare_job(output_path: impl Into<String>) -> RenderJob {
        RenderJob::new(output_path)
    }

    pub fn validate_config(config: &RenderConfig) -> Result<(), String> {
        if config.resolution.width == 0 || config.resolution.height == 0 {
            return Err("resolution must be non-zero".to_string());
        }
        if config.fps == 0 || config.fps > 120 {
            return Err("fps must be between 1 and 120".to_string());
        }
        if config.bitrate_kbps < 100 {
            return Err("bitrate must be at least 100 kbps".to_string());
        }
        Ok(())
    }

    pub fn estimate_render_time(clips: &[VideoClip]) -> f64 {
        let total_duration: f64 = clips.iter().map(|c| c.duration_secs).sum();
        total_duration * 2.5
    }
}

impl Default for HyperframesEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn list_preset_resolutions() -> Vec<(&'static str, VideoResolution)> {
    let mut result: Vec<_> = PRESET_RESOLUTIONS
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    result.sort_by(|a, b| a.0.cmp(b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_format_extension() {
        assert_eq!(VideoFormat::Mp4.extension(), "mp4");
        assert_eq!(VideoFormat::WebM.extension(), "webm");
        assert_eq!(VideoFormat::Gif.extension(), "gif");
    }

    #[test]
    fn test_video_format_mime_type() {
        assert_eq!(VideoFormat::Mp4.mime_type(), "video/mp4");
        assert_eq!(VideoFormat::Gif.mime_type(), "image/gif");
    }

    #[test]
    fn test_render_config_default() {
        let cfg = RenderConfig::default();
        assert_eq!(cfg.fps, 30);
        assert_eq!(cfg.resolution.width, 1920);
        assert_eq!(cfg.resolution.height, 1080);
    }

    #[test]
    fn test_render_config_builder() {
        let cfg = RenderConfig::default()
            .with_resolution(1080, 1920)
            .with_format(VideoFormat::WebM)
            .with_fps(60);
        assert_eq!(cfg.resolution.width, 1080);
        assert_eq!(cfg.resolution.height, 1920);
        assert_eq!(cfg.format, VideoFormat::WebM);
        assert_eq!(cfg.fps, 60);
    }

    #[test]
    fn test_video_clip_duration() {
        let clip = VideoClip::new("<h1>Hello</h1>", 3.5);
        assert_eq!(clip.duration_secs, 3.5);
        assert_eq!(clip.html, "<h1>Hello</h1>");
    }

    #[test]
    fn test_video_clip_transitions() {
        let clip = VideoClip::new("<h1>Hello</h1>", 2.0)
            .with_transition_in("fade")
            .with_transition_out("slide");
        assert_eq!(clip.transition_in, Some("fade".to_string()));
        assert_eq!(clip.transition_out, Some("slide".to_string()));
    }

    #[test]
    fn test_render_job_basic() {
        let mut job = RenderJob::new("/tmp/output.mp4");
        job.add_clip(VideoClip::new("<h1>A</h1>", 5.0));
        job.add_clip(VideoClip::new("<h1>B</h1>", 3.0));
        assert_eq!(job.total_duration(), 8.0);
        assert_eq!(job.clips.len(), 2);
    }

    #[test]
    fn test_render_job_output_path() {
        let job = RenderJob::new("/tmp/test.mp4");
        assert_eq!(job.output_path, "/tmp/test.mp4");
    }

    #[test]
    fn test_estimated_size() {
        let mut job = RenderJob::new("/tmp/test.mp4");
        job.add_clip(VideoClip::new("<h1>A</h1>", 10.0));
        let size = job.estimated_size_mb();
        assert!(size > 0.0);
    }

    #[test]
    fn test_validate_config_ok() {
        let cfg = RenderConfig::default();
        assert!(HyperframesEngine::validate_config(&cfg).is_ok());
    }

    #[test]
    fn test_validate_config_zero_resolution() {
        let cfg = RenderConfig::default().with_resolution(0, 0);
        assert!(HyperframesEngine::validate_config(&cfg).is_err());
    }

    #[test]
    fn test_validate_config_bad_fps() {
        let cfg = RenderConfig::default().with_fps(200);
        assert!(HyperframesEngine::validate_config(&cfg).is_err());
    }

    #[test]
    fn test_preset_resolutions() {
        let presets = list_preset_resolutions();
        assert!(presets.iter().any(|(n, _)| *n == "1080p"));
        assert!(presets.iter().any(|(n, _)| *n == "720p"));
        assert!(presets.iter().any(|(n, _)| *n == "vertical"));
        assert!(presets.iter().any(|(n, _)| *n == "square"));
    }

    #[test]
    fn test_estimate_render_time() {
        let clips = vec![
            VideoClip::new("<h1>A</h1>", 10.0),
            VideoClip::new("<h1>B</h1>", 5.0),
        ];
        let estimated = HyperframesEngine::estimate_render_time(&clips);
        assert_eq!(estimated, 37.5);
    }
}
