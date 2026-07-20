use std::collections::HashMap;
use std::time::Instant;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
