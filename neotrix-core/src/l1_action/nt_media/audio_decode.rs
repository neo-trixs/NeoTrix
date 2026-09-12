//! Rust-native audio decode — feature-gated fallback for ffplay/mpv.
//! Uses symphonia for format-agnostic decoding.

// ═══════════════════════════════════════════════════════════════════════════
// Feature-gated real implementation
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "audio-decode")]
mod inner {
    use std::path::Path;

    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::{FormatOptions, FormatReader};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::{get_codecs, get_probe};

    use super::AudioError;

    pub struct AudioInfo {
        pub channels: u16,
        pub sample_rate: u32,
        pub duration_secs: Option<f64>,
        pub codec: String,
    }

    pub struct AudioDecoder {
        format_reader: Box<dyn FormatReader>,
        decoder: Box<dyn Decoder>,
        track_id: u32,
    }

    impl AudioDecoder {
        pub fn from_file(path: &Path) -> Result<Self, AudioError> {
            let mut hint = Hint::new();
            if let Some(ext) = path.extension() {
                hint.with_extension(&ext.to_string_lossy());
            }
            let file = std::fs::File::open(path)
                .map_err(|e| AudioError::Io(format!("failed to open {}: {}", path.display(), e)))?;
            let source = std::io::BufReader::new(file);
            let mss = MediaSourceStream::new(Box::new(source), Default::default());
            let probed = get_probe()
                .format(
                    &hint,
                    mss,
                    &FormatOptions::default(),
                    &MetadataOptions::default(),
                )
                .map_err(|e| AudioError::Format(e.to_string()))?;
            let format_reader = probed.format;
            let track = format_reader
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                .ok_or_else(|| AudioError::NoAudioTrack)?;
            let track_id = track.id;
            let decoder = get_codecs()
                .make(&track.codec_params, &DecoderOptions::default())
                .map_err(|e| AudioError::Codec(e.to_string()))?;
            Ok(Self {
                format_reader,
                decoder,
                track_id,
            })
        }

        pub fn info(&self) -> AudioInfo {
            let track = self
                .format_reader
                .tracks()
                .iter()
                .find(|t| t.id == self.track_id)
                .expect("track must exist");

            let params = &track.codec_params;
            let channels = params.channels.map(|c| c.count() as u16).unwrap_or(0);
            let sample_rate = params.sample_rate.unwrap_or(0);
            let duration_secs = params
                .time_base
                .and_then(|tb| params.n_frames.map(|nf| tb.calc_time(nf).time));
            let codec = params
                .codec
                .map(|c| format!("{:?}", c))
                .unwrap_or_else(|| "unknown".into());

            AudioInfo {
                channels,
                sample_rate,
                duration_secs,
                codec,
            }
        }

        pub fn decode_next(&mut self) -> Result<Option<Vec<f32>>, AudioError> {
            loop {
                let packet = match self.format_reader.next_packet() {
                    Ok(p) => p,
                    Err(symphonia::core::errors::Error::IoError(ref e))
                        if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                    {
                        return Ok(None);
                    }
                    Err(e) => return Err(AudioError::Decode(e.to_string())),
                };

                if packet.track_id() != self.track_id {
                    continue;
                }

                match self.decoder.decode(&packet) {
                    Ok(audio_buf) => {
                        let spec = *audio_buf.spec();
                        let frames = audio_buf.frames();
                        let channels = spec.channels.count();
                        let mut buf =
                            SampleBuffer::<f32>::new(frames as u64, spec);
                        buf.copy_interleaved_ref(audio_buf);

                        let samples: Vec<f32> = if channels == 1 {
                            buf.samples().to_vec()
                        } else {
                            buf.samples()
                                .iter()
                                .step_by(channels)
                                .copied()
                                .collect()
                        };
                        return Ok(Some(samples));
                    }
                    Err(e) => return Err(AudioError::Decode(e.to_string())),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stub when feature disabled
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(not(feature = "audio-decode"))]
pub struct AudioDecoder;

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "audio-decode")]
pub use inner::{AudioDecoder, AudioInfo};

#[cfg(not(feature = "audio-decode"))]
pub struct AudioInfo {
    pub channels: u16,
    pub sample_rate: u32,
    pub duration_secs: Option<f64>,
    pub codec: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Error type
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio-decode feature is disabled — enable it in Cargo.toml")]
    FeatureDisabled,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("format error: {0}")]
    Format(String),
    #[error("codec error: {0}")]
    Codec(String),
    #[error("no audio track found")]
    NoAudioTrack,
    #[error("decode error: {0}")]
    Decode(String),
}
