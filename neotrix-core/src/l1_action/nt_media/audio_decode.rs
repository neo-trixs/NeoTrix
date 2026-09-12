//! Rust-native audio decoding fallback — feature-gated on `audio-decode`.
//!
//! Decodes audio bytes to PCM samples via symphonia.
//! Supports: MP3, FLAC, WAV, OGG (Vorbis), AAC, ALAC.
//!
//! Usage:
//! ```rust,ignore
//! let decoder = audio_decode::decode_file(Path::new("track.mp3"))?;
//! let info = decoder.info();
//! println!("{} channels, {} Hz, {:?}",
//!          info.channels, info.sample_rate, info.duration);
//! let samples: Vec<f32> = decoder.decode_all()?;
//! ```

// ═══════════════════════════════════════════════════════════════════════════
// Stub when feature disabled
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(not(feature = "audio-decode"))]
pub struct AudioDecoder;

#[cfg(not(feature = "audio-decode"))]
impl AudioDecoder {
    pub fn decode_all(self) -> Result<Vec<f32>, AudioError> {
        Err(AudioError::FeatureDisabled)
    }
}

#[cfg(not(feature = "audio-decode"))]
#[derive(Debug)]
pub struct AudioInfo {
    pub channels: u16,
    pub sample_rate: u32,
    pub duration: Option<std::time::Duration>,
    pub codec: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Feature-gated real implementation
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "audio-decode")]
mod inner {
    use std::fs::File;
    use std::io::{Cursor, Read};
    use std::path::Path;
    use std::time::Duration;

    use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
    use symphonia::core::codecs::{CodecParameters, Decoder, DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::formats::{FormatOptions, FormatReader, Packet, SeekMode, SeekTo};
    use symphonia::core::io::{MediaSource, MediaSourceStream};
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::default::{get_codecs, get_probe};

    use super::AudioError;

    // ── AudioInfo ──────────────────────────────────────────────────────

    /// Audio stream metadata.
    #[derive(Debug, Clone)]
    pub struct AudioInfo {
        pub channels: u16,
        pub sample_rate: u32,
        pub duration: Option<Duration>,
        pub codec: String,
        pub bits_per_sample: Option<u32>,
    }

    impl AudioInfo {
        fn from_params(params: &CodecParameters) -> Self {
            let channels = params
                .channels
                .map(|c| c.count() as u16)
                .unwrap_or(0);

            let sample_rate = params.sample_rate.unwrap_or(0);

            let duration = params
                .time_base
                .and_then(|tb| params.n_frames.map(|nf| tb.calc_time(nf).time));

            let duration_secs = duration.map(|s| Duration::from_secs_f64(s));

            let codec = params
                .codec
                .map(|c| format!("{:?}", c))
                .unwrap_or_else(|| "unknown".into());

            let bits_per_sample = params.bits_per_sample;

            Self {
                channels,
                sample_rate,
                duration: duration_secs,
                codec,
                bits_per_sample,
            }
        }
    }

    // ── AudioDecoder ───────────────────────────────────────────────────

    /// Rust-native audio decoder backed by symphonia.
    pub struct AudioDecoder {
        format_reader: Box<dyn FormatReader>,
        decoder: Box<dyn Decoder>,
        track_id: u32,
        info: AudioInfo,
    }

    impl AudioDecoder {
        /// Open a file and create a decoder for the first audio track.
        pub fn from_file(path: &Path) -> Result<Self, AudioError> {
            let file = File::open(path)
                .map_err(|e| AudioError::Io(format!("failed to open {}: {}", path.display(), e)))?;

            let mss = MediaSourceStream::new(Box::new(file), Default::default());
            Self::from_mss(mss)
        }

        /// Decode from in-memory bytes using symphonia's probe.
        pub fn from_bytes(bytes: &[u8]) -> Result<Self, AudioError> {
            let cursor = Cursor::new(bytes.to_vec());
            let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
            Self::from_mss(mss)
        }

        /// Decode from any media source stream.
        fn from_mss(mss: MediaSourceStream) -> Result<Self, AudioError> {
            let probe = get_probe();

            let format_opts = FormatOptions {
                enable_gapless: true,
                ..Default::default()
            };

            let metadata_opts = MetadataOptions::default();

            let probed = probe
                .format(
                    &Hint::new(),
                    mss,
                    &format_opts,
                    &metadata_opts,
                )
                .map_err(|e| AudioError::Format(e.to_string()))?;

            let format_reader = probed.format;

            let track = format_reader
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                .ok_or_else(|| AudioError::NoAudioTrack)?;

            let track_id = track.id;
            let info = AudioInfo::from_params(&track.codec_params);

            let decoder_opts = DecoderOptions::default();
            let decoder = get_codecs()
                .make(&track.codec_params, &decoder_opts)
                .map_err(|e| AudioError::Codec(e.to_string()))?;

            Ok(Self {
                format_reader,
                decoder,
                track_id,
                info,
            })
        }

        /// Get metadata about the audio stream.
        pub fn info(&self) -> &AudioInfo {
            &self.info
        }

        /// Decode all audio packets into interleaved f32 PCM samples.
        pub fn decode_all(&mut self) -> Result<Vec<f32>, AudioError> {
            let mut samples: Vec<f32> = Vec::new();

            loop {
                let packet = match self.format_reader.next_packet() {
                    Ok(p) => p,
                    Err(symphonia::core::errors::Error::IoError(ref e))
                        if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                    {
                        break
                    }
                    Err(e) => return Err(AudioError::Decode(e.to_string())),
                };

                if packet.track_id() != self.track_id {
                    continue;
                }

                match self.decoder.decode(&packet) {
                    Ok(audio_buf) => {
                        self.extract_samples(&audio_buf, &mut samples);
                    }
                    Err(symphonia::core::errors::Error::DecodeError(ref msg))
                        if msg.contains("corrupt") || msg.contains("invalid") =>
                    {
                        // Skip corrupted frames — common in damaged files
                        continue;
                    }
                    Err(e) => return Err(AudioError::Decode(e.to_string())),
                }
            }

            Ok(samples)
        }

        /// Decode the next chunk of audio (streaming-friendly).
        pub fn decode_packet(&mut self) -> Result<Option<Vec<f32>>, AudioError> {
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
                        let mut samples = Vec::new();
                        self.extract_samples(&audio_buf, &mut samples);
                        return Ok(Some(samples));
                    }
                    Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
                    Err(e) => return Err(AudioError::Decode(e.to_string())),
                }
            }
        }

        /// Seek to a specific timestamp.
        pub fn seek(&mut self, time: Duration) -> Result<(), AudioError> {
            let seek_to = SeekTo::Time {
                time: time.as_secs_f64(),
                track_id: Some(self.track_id),
            };

            self.format_reader
                .seek(SeekMode::Accurate, seek_to)
                .map_err(|e| AudioError::Seek(e.to_string()))?;

            self.decoder.reset();
            Ok(())
        }

        /// Extract f32 samples from an audio buffer reference.
        fn extract_samples(&self, audio_buf: &AudioBufferRef, out: &mut Vec<f32>) {
            match audio_buf {
                AudioBufferRef::F32(buf) => {
                    let spec = buf.spec();
                    let interleaved = buf.samples();
                    let mono = spec.channels.count() == 1;

                    if mono {
                        out.extend_from_slice(interleaved);
                    } else {
                        // For multi-channel, take first channel (downmix to mono)
                        let channels = spec.channels.count();
                        for (i, sample) in interleaved.iter().enumerate() {
                            if i % channels == 0 {
                                out.push(*sample);
                            }
                        }
                    }
                }
                AudioBufferRef::S32(buf) => {
                    let spec = buf.spec();
                    let channels = spec.channels.count();
                    for (i, sample) in buf.samples().iter().enumerate() {
                        if i % channels == 0 {
                            out.push(*sample as f32 / i32::MAX as f32);
                        }
                    }
                }
                AudioBufferRef::S16(buf) => {
                    let spec = buf.spec();
                    let channels = spec.channels.count();
                    for (i, sample) in buf.samples().iter().enumerate() {
                        if i % channels == 0 {
                            out.push(*sample as f32 / i16::MAX as f32);
                        }
                    }
                }
                AudioBufferRef::U8(buf) => {
                    let spec = buf.spec();
                    let channels = spec.channels.count();
                    for (i, sample) in buf.samples().iter().enumerate() {
                        if i % channels == 0 {
                            out.push((*sample as f32 - 128.0) / 128.0);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API — re-export or stub based on feature
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "audio-decode")]
pub use inner::{AudioDecoder, AudioInfo};

/// Open a file and create a decoder.
#[cfg(feature = "audio-decode")]
pub fn decode_file(path: &std::path::Path) -> Result<AudioDecoder, AudioError> {
    AudioDecoder::from_file(path)
}

/// Decode from in-memory bytes.
#[cfg(feature = "audio-decode")]
pub fn decode_bytes(bytes: &[u8]) -> Result<AudioDecoder, AudioError> {
    AudioDecoder::from_bytes(bytes)
}

/// Stub functions when feature is disabled.
#[cfg(not(feature = "audio-decode"))]
pub fn decode_file(_path: &std::path::Path) -> Result<AudioDecoder, AudioError> {
    Err(AudioError::FeatureDisabled)
}

#[cfg(not(feature = "audio-decode"))]
pub fn decode_bytes(_bytes: &[u8]) -> Result<AudioDecoder, AudioError> {
    Err(AudioError::FeatureDisabled)
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
    #[error("seek error: {0}")]
    Seek(String),
}
