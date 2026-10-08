//! Transcode segment: codec / stream / transcode config + transcoder.

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
    pub fn _ffmpeg_name(&self) -> &'static str {
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
pub enum _StreamProtocol {
    HLS,
    DASH,
    Progressive,
    RTMP,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct _StreamInfo {
    pub url: String,
    pub protocol: _StreamProtocol,
    pub codec: VideoCodec,
    pub width: u32,
    pub height: u32,
    pub bitrate_kbps: u32,
    pub fps: f64,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub has_audio: bool,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
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
pub struct _TranscodeResult {
    pub output_path: String,
    pub duration_ms: u64,
    pub output_size_bytes: u64,
    pub actual_codec: VideoCodec,
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub hardware_used: bool,
    pub compression_ratio: f64,
}

pub struct _Transcoder {
    config: TranscodeConfig,
}

impl _Transcoder {
    pub fn new(config: TranscodeConfig) -> Self {
        Self { config }
    }

    pub fn _transcode(&self, input: &_StreamInfo) -> _TranscodeResult {
        let compression = if input.width > 0 && self.config.target_width > 0 {
            (input.width as f64 / self.config.target_width as f64)
                .max(0.0)
                .min(1.0)
        } else {
            1.0
        };
        _TranscodeResult {
            output_path: format!("/tmp/transcode_{}.mp4", input.codec._ffmpeg_name()),
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

    pub fn _should_transcode(&self, stream: &_StreamInfo) -> bool {
        stream.codec != self.config.target_codec
            || stream.width > self.config.target_width
            || stream.height > self.config.target_height
            || self.config.subtitle_burn
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcode_decision() {
        let config = TranscodeConfig::default();
        let transcoder = _Transcoder::new(config);
        let stream = _StreamInfo {
            url: "test".into(), protocol: _StreamProtocol::HLS, codec: VideoCodec::VP9,
            width: 3840, height: 2160, bitrate_kbps: 20000, fps: 60.0,
            has_audio: true, has_subtitles: false,
        };
        assert!(transcoder._should_transcode(&stream));
        let h264_stream = _StreamInfo {
            url: "test".into(), protocol: _StreamProtocol::HLS, codec: VideoCodec::H264,
            width: 1920, height: 1080, bitrate_kbps: 8000, fps: 30.0,
            has_audio: true, has_subtitles: false,
        };
        assert!(!transcoder._should_transcode(&h264_stream));
    }

    #[test]
    fn test_codec_ffmpeg_names() {
        assert_eq!(VideoCodec::H264._ffmpeg_name(), "h264");
        assert_eq!(VideoCodec::AV1._ffmpeg_name(), "av1");
    }
}
