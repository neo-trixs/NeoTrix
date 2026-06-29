#![allow(unused_imports)]
#![allow(dead_code)]
use super::types::*;
use super::ConsciousnessIntegration;
use crate::core::nt_core_audio::vad::{VadEngine, VadState, VoiceSegment};
use crate::core::nt_core_audio::AudioCapture;

// AUDIO handlers
// 1 handler: audio_capture_tick

const AUDIO_CAPTURE_DURATION: f64 = 3.0;
const AUDIO_SAMPLE_RATE: u32 = 16000;

impl ConsciousnessIntegration {
    pub fn handle_audio_capture_tick(&mut self) -> String {
        if self.cycle % 5 != 0 {
            return "audio_capture:skip".to_string();
        }

        let capture = match self.audio_capture.as_mut() {
            Some(c) => c,
            None => return "audio_capture:unwired".to_string(),
        };

        if !capture.is_active() {
            return "audio_capture:inactive".to_string();
        }

        let captured = match capture.capture() {
            Some(a) => a,
            None => return "audio_capture:no_input".to_string(),
        };

        if captured.samples.is_empty() || captured.samples.len() < 1600 {
            return "audio_capture:too_short".to_string();
        }

        // VAD
        let mut vad = VadEngine::new(AUDIO_SAMPLE_RATE);
        let frame_len = 1600; // 100ms at 16kHz
        let mut segments = Vec::new();

        for (i, chunk) in captured.samples.chunks(frame_len).enumerate() {
            let frame_time_ms = (i as u64 * 100) as u64;
            if let Some(seg) = vad.process_frame(chunk, frame_time_ms) {
                segments.push(seg);
            }
        }
        if let Some(seg) = vad.flush() {
            segments.push(seg);
        }

        if segments.is_empty() {
            return "audio_capture:no_speech".to_string();
        }

        // Transcribe the first/loudest speech segment
        let best = segments
            .into_iter()
            .max_by_key(|s| (s.peak_amplitude * 1000.0) as u64)
            .unwrap();
        let sample = crate::neotrix::nt_act_voice::VoiceSample::new(
            best.samples,
            std::time::Duration::from_secs_f64(best.duration_ms as f64 / 1000.0),
            AUDIO_SAMPLE_RATE,
        );

        let transcribed =
            match crate::neotrix::nt_act_voice::transcribe::MockTranscriber::transcribe(&sample) {
                Ok(text) => text,
                Err(_) => {
                    log::warn!("audio_capture: mock transcribe failed");
                    return "audio_capture:transcribe_failed".to_string();
                }
            };

        if !transcribed.is_empty() {
            log::info!(
                "audio_capture: transcribe ok ({:.1}s)",
                best.duration_ms as f64 / 1000.0
            );
            self.push_text_buffer(format!("[voice: {}]", transcribed));
            "audio_capture:ok".to_string()
        } else {
            "audio_capture:empty".to_string()
        }
    }
}
