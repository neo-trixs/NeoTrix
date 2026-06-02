use super::{VoiceError, VoiceSample};

#[derive(Debug, Clone)]
pub enum TranscribeEngine {
    Mock,
    Whisper,
    ExternalAPI {
        endpoint: String,
        api_key: String,
    },
}

impl TranscribeEngine {
    pub fn name(&self) -> &str {
        match self {
            TranscribeEngine::Mock => "mock",
            TranscribeEngine::Whisper => "whisper",
            TranscribeEngine::ExternalAPI { .. } => "external_api",
        }
    }
}

pub struct MockTranscriber;

impl MockTranscriber {
    pub fn transcribe(_sample: &VoiceSample) -> Result<String, VoiceError> {
        let phrases = [
            "test nt_act_voice input",
            "hello neotrix",
            "open settings",
            "run command test",
            "switch session default",
            "show help",
        ];
        let idx = _sample.audio_data.len() % phrases.len();
        Ok(phrases[idx].to_string())
    }
}

pub struct WhisperTranscriber;

impl WhisperTranscriber {
    pub fn transcribe(_sample: &VoiceSample) -> Result<String, VoiceError> {
        Err(VoiceError::EngineNotAvailable)
    }
}

pub struct ExternalAPITranscriber;

impl ExternalAPITranscriber {
    pub fn transcribe(_sample: &VoiceSample) -> Result<String, VoiceError> {
        Err(VoiceError::EngineNotAvailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_mock_transcriber_returns_string() {
        let sample = VoiceSample::new(vec![0.0; 16000], Duration::from_secs(1), 16000);
        let result = MockTranscriber::transcribe(&sample);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_whisper_returns_engine_unavailable() {
        let sample = VoiceSample::new(vec![], Duration::from_secs(1), 16000);
        let result = WhisperTranscriber::transcribe(&sample);
        assert!(matches!(result, Err(VoiceError::EngineNotAvailable)));
    }

    #[test]
    fn test_external_api_returns_engine_unavailable() {
        let sample = VoiceSample::new(vec![], Duration::from_secs(1), 16000);
        let result = ExternalAPITranscriber::transcribe(&sample);
        assert!(matches!(result, Err(VoiceError::EngineNotAvailable)));
    }

    #[test]
    fn test_transcribe_engine_name() {
        assert_eq!(TranscribeEngine::Mock.name(), "mock");
        assert_eq!(TranscribeEngine::Whisper.name(), "whisper");
        assert_eq!(
            TranscribeEngine::ExternalAPI {
                endpoint: "https://example.com".into(),
                api_key: "key".into(),
            }.name(),
            "external_api"
        );
    }

    #[test]
    fn test_mock_deterministic_output() {
        let sample1 = VoiceSample::new(vec![0.0; 0], Duration::from_secs(1), 16000);
        let sample2 = VoiceSample::new(vec![0.0; 7], Duration::from_secs(1), 16000);
        let r1 = MockTranscriber::transcribe(&sample1).unwrap();
        let r2 = MockTranscriber::transcribe(&sample2).unwrap();
        assert_ne!(r1, r2);
    }
}
