pub mod capture;
pub mod vad;

pub use capture::AudioCapture;
pub use vad::{VadEngine, VadState, VoiceSegment};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_capture_builder_chain() {
        let cap = AudioCapture::new()
            .with_duration(5.0)
            .with_sample_rate(44100);
        assert!(!cap.active);
        assert!((cap.duration_secs - 5.0).abs() < 1e-6);
        assert_eq!(cap.sample_rate, 44100);
    }

    #[test]
    fn test_vad_engine_creation() {
        let vad = VadEngine::new(16000);
        assert_eq!(vad.state, VadState::Idle);
        assert!((vad.threshold - 0.02).abs() < 1e-6);
    }

    #[test]
    fn test_audio_capture_activate_toggle() {
        let mut cap = AudioCapture::new();
        assert!(!cap.is_active());
        cap.activate();
        assert!(cap.is_active());
        cap.deactivate();
        assert!(!cap.is_active());
    }

    #[test]
    fn test_vad_rms_silence_via_pub_api() {
        let rms = VadEngine::rms(&[0.0_f32; 800]);
        assert!(rms < 0.001);
    }

    #[test]
    fn test_vad_rms_speech_via_pub_api() {
        let samples: Vec<f32> = (0..800).map(|i| (i as f32 / 800.0) * 0.5).collect();
        let rms = VadEngine::rms(&samples);
        assert!(rms > 0.02);
    }
}
