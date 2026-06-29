use super::{VoiceError, VoiceSample};

#[derive(Debug, Clone)]
pub enum TranscribeEngine {
    Mock,
    Whisper,
    ExternalAPI { endpoint: String, api_key: String },
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

pub struct OpenAiWhisperClient {
    api_key: String,
    model: String,
    endpoint: String,
    language: Option<String>,
    timeout_secs: u64,
}

impl OpenAiWhisperClient {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        Some(Self {
            api_key,
            model: "whisper-1".to_string(),
            endpoint: "https://api.openai.com/v1/audio/transcriptions".to_string(),
            language: None,
            timeout_secs: 30,
        })
    }

    pub fn transcribe(&self, audio_data: &[u8], format: &str) -> Result<String, VoiceError> {
        use reqwest::blocking::multipart::{Form, Part};
        use reqwest::blocking::Client as BlockingClient;

        let client = BlockingClient::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .build()
            .map_err(|e| VoiceError::TranscriptionFailed(format!("http client: {}", e)))?;

        let file_part = Part::bytes(audio_data.to_vec())
            .file_name(format!("audio.{}", format))
            .mime_str("audio/wav")
            .map_err(|e| VoiceError::TranscriptionFailed(format!("mime: {}", e)))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("model", self.model.clone())
            .text("response_format", "text".to_string());

        if let Some(ref lang) = self.language {
            form = form.text("language", lang.clone());
        }

        let resp = client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .map_err(|e| VoiceError::TranscriptionFailed(format!("request: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(VoiceError::TranscriptionFailed(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        resp.text()
            .map_err(|e| VoiceError::TranscriptionFailed(format!("read response: {}", e)))
    }
}

pub struct WhisperTranscriber {
    client: Option<OpenAiWhisperClient>,
}

impl WhisperTranscriber {
    pub fn new() -> Self {
        Self {
            client: OpenAiWhisperClient::from_env(),
        }
    }

    pub fn transcribe(&self, sample: &VoiceSample) -> Result<String, VoiceError> {
        match &self.client {
            Some(client) => {
                let wav_bytes = sample.to_wav_bytes();
                client.transcribe(&wav_bytes, "wav")
            }
            None => Err(VoiceError::EngineNotAvailable),
        }
    }
}

impl Default for WhisperTranscriber {
    fn default() -> Self {
        Self::new()
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
            }
            .name(),
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

    #[test]
    fn test_openai_client_from_env() {
        std::env::remove_var("OPENAI_API_KEY");
        assert!(OpenAiWhisperClient::from_env().is_none());
    }

    #[test]
    fn test_openai_client_constructor() {
        std::env::set_var("OPENAI_API_KEY", "sk-test123");
        let client = OpenAiWhisperClient::from_env().unwrap();
        assert_eq!(client.model, "whisper-1");
        assert_eq!(
            client.endpoint,
            "https://api.openai.com/v1/audio/transcriptions"
        );
        assert_eq!(client.timeout_secs, 30);
        assert!(client.language.is_none());
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_whisper_transcriber_no_key_fallback() {
        std::env::remove_var("OPENAI_API_KEY");
        let t = WhisperTranscriber::new();
        let sample = VoiceSample::new(vec![0.0; 100], Duration::from_secs(1), 16000);
        let result = t.transcribe(&sample);
        assert!(matches!(result, Err(VoiceError::EngineNotAvailable)));
    }

    #[test]
    fn test_to_wav_bytes_roundtrip() {
        let sample = VoiceSample::new(vec![0.5, -0.5, 0.0], Duration::from_secs(1), 16000);
        let wav = sample.to_wav_bytes();
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        let sample_rate = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(sample_rate, 16000);
        assert_eq!(&wav[36..40], b"data");
        assert!(wav.len() > 44, "must have PCM data beyond header");
    }
}
