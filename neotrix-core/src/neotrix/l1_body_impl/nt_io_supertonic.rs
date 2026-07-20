use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TtsLanguage {
    EnUs,
    EnGb,
    ZhCn,
    JaJp,
    KoKr,
    FrFr,
    DeDe,
    EsEs,
    PtBr,
    RuRu,
    ItIt,
}

impl TtsLanguage {
    pub fn code(&self) -> &'static str {
        match self {
            TtsLanguage::EnUs => "en-US",
            TtsLanguage::EnGb => "en-GB",
            TtsLanguage::ZhCn => "zh-CN",
            TtsLanguage::JaJp => "ja-JP",
            TtsLanguage::KoKr => "ko-KR",
            TtsLanguage::FrFr => "fr-FR",
            TtsLanguage::DeDe => "de-DE",
            TtsLanguage::EsEs => "es-ES",
            TtsLanguage::PtBr => "pt-BR",
            TtsLanguage::RuRu => "ru-RU",
            TtsLanguage::ItIt => "it-IT",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            TtsLanguage::EnUs => "US English",
            TtsLanguage::EnGb => "British English",
            TtsLanguage::ZhCn => "Mandarin Chinese",
            TtsLanguage::JaJp => "Japanese",
            TtsLanguage::KoKr => "Korean",
            TtsLanguage::FrFr => "French",
            TtsLanguage::DeDe => "German",
            TtsLanguage::EsEs => "Spanish",
            TtsLanguage::PtBr => "Brazilian Portuguese",
            TtsLanguage::RuRu => "Russian",
            TtsLanguage::ItIt => "Italian",
        }
    }

    pub fn all() -> &'static [TtsLanguage] {
        &[
            TtsLanguage::EnUs, TtsLanguage::EnGb, TtsLanguage::ZhCn,
            TtsLanguage::JaJp, TtsLanguage::KoKr, TtsLanguage::FrFr,
            TtsLanguage::DeDe, TtsLanguage::EsEs, TtsLanguage::PtBr,
            TtsLanguage::RuRu, TtsLanguage::ItIt,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct TtsVoice {
    pub id: String,
    pub name: String,
    pub language: TtsLanguage,
    pub gender: String,
    pub sample_rate: u32,
}

impl TtsVoice {
    pub fn new(id: String, name: String, language: TtsLanguage, gender: String) -> Self {
        Self {
            id,
            name,
            language,
            gender,
            sample_rate: 44100,
        }
    }
}

pub static BUILTIN_VOICES: LazyLock<Vec<TtsVoice>> = LazyLock::new(|| {
    vec![
        TtsVoice::new("en-us-female".into(), "Emma".into(), TtsLanguage::EnUs, "female".into()),
        TtsVoice::new("en-us-male".into(), "James".into(), TtsLanguage::EnUs, "male".into()),
        TtsVoice::new("en-gb-female".into(), "Alice".into(), TtsLanguage::EnGb, "female".into()),
        TtsVoice::new("zh-cn-female".into(), "Xiaomei".into(), TtsLanguage::ZhCn, "female".into()),
        TtsVoice::new("zh-cn-male".into(), "Wei".into(), TtsLanguage::ZhCn, "male".into()),
        TtsVoice::new("ja-jp-female".into(), "Sakura".into(), TtsLanguage::JaJp, "female".into()),
        TtsVoice::new("ko-kr-female".into(), "Soo-jin".into(), TtsLanguage::KoKr, "female".into()),
        TtsVoice::new("fr-fr-female".into(), "Camille".into(), TtsLanguage::FrFr, "female".into()),
        TtsVoice::new("de-de-male".into(), "Finn".into(), TtsLanguage::DeDe, "male".into()),
        TtsVoice::new("es-es-female".into(), "Lucia".into(), TtsLanguage::EsEs, "female".into()),
        TtsVoice::new("pt-br-female".into(), "Isabela".into(), TtsLanguage::PtBr, "female".into()),
        TtsVoice::new("ru-ru-female".into(), "Anya".into(), TtsLanguage::RuRu, "female".into()),
        TtsVoice::new("it-it-male".into(), "Marco".into(), TtsLanguage::ItIt, "male".into()),
    ]
});

pub fn list_languages() -> Vec<(TtsLanguage, &'static str)> {
    TtsLanguage::all().iter().map(|l| (*l, l.name())).collect()
}

pub fn list_voices() -> &'static Vec<TtsVoice> {
    &BUILTIN_VOICES
}

pub fn find_voice(id: &str) -> Option<&'static TtsVoice> {
    BUILTIN_VOICES.iter().find(|v| v.id == id)
}

pub fn find_voices_by_language(lang: TtsLanguage) -> Vec<&'static TtsVoice> {
    BUILTIN_VOICES.iter().filter(|v| v.language == lang).collect()
}

#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub text: String,
    pub voice_id: String,
    pub speed: f64,
    pub pitch: f64,
    pub volume: f64,
}

impl TtsRequest {
    pub fn new(text: String, voice_id: String) -> Self {
        Self {
            text,
            voice_id,
            speed: 1.0,
            pitch: 1.0,
            volume: 1.0,
        }
    }

    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed.max(0.5).min(2.0);
        self
    }

    pub fn with_pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch.max(0.5).min(2.0);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.text.is_empty() {
            return Err("text cannot be empty".to_string());
        }
        if !(0.5..=2.0).contains(&self.speed) {
            return Err("speed must be 0.5-2.0".to_string());
        }
        if !(0.5..=2.0).contains(&self.pitch) {
            return Err("pitch must be 0.5-2.0".to_string());
        }
        if BUILTIN_VOICES.iter().all(|v| v.id != self.voice_id) {
            return Err(format!("voice '{}' not found", self.voice_id));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TtsEngine {
    pub sample_rate: u32,
    pub cache_enabled: bool,
    cache: HashMap<String, Vec<u8>>,
}

impl TtsEngine {
    pub fn new() -> Self {
        Self {
            sample_rate: 44100,
            cache_enabled: true,
            cache: HashMap::new(),
        }
    }

    pub fn synthesize(&mut self, request: &TtsRequest) -> Result<Vec<u8>, String> {
        request.validate()?;

        let cache_key = format!("{}-{}-{}", request.voice_id, request.text, request.speed as u32);
        if self.cache_enabled {
            if let Some(cached) = self.cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let audio = self.render_pcm(request)?;

        if self.cache_enabled {
            self.cache.insert(cache_key, audio.clone());
        }
        Ok(audio)
    }

    fn render_pcm(&self, request: &TtsRequest) -> Result<Vec<u8>, String> {
        let sample_count = ((self.sample_rate as f64 * request.text.len() as f64 * 0.08) / request.speed) as usize;
        let byte_count = sample_count * 2;
        let mut pcm = Vec::with_capacity(byte_count);

        let base_freq = 220.0;
        let freq_shift = (request.pitch - 1.0) * 100.0;
        let freq = base_freq + freq_shift;

        for i in 0..sample_count {
            let t = i as f64 / self.sample_rate as f64;
            let sample = (2.0 * std::f64::consts::PI * freq * t).sin();
            let amplitude = (request.volume * 0.3 * (1.0 - (i as f64 / sample_count as f64))).max(0.01);
            let value = (sample * amplitude * i16::MAX as f64) as i16;
            pcm.extend_from_slice(&value.to_le_bytes());
        }
        Ok(pcm)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for TtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(TtsLanguage::EnUs.code(), "en-US");
        assert_eq!(TtsLanguage::ZhCn.code(), "zh-CN");
        assert_eq!(TtsLanguage::JaJp.code(), "ja-JP");
    }

    #[test]
    fn test_all_languages() {
        let langs = TtsLanguage::all();
        assert_eq!(langs.len(), 11);
    }

    #[test]
    fn test_builtin_voices_count() {
        let voices = list_voices();
        assert_eq!(voices.len(), 13);
    }

    #[test]
    fn test_find_voice() {
        assert!(find_voice("en-us-female").is_some());
        assert!(find_voice("nonexistent").is_none());
    }

    #[test]
    fn test_find_voices_by_language() {
        let en_voices = find_voices_by_language(TtsLanguage::EnUs);
        assert_eq!(en_voices.len(), 2);
        let jp_voices = find_voices_by_language(TtsLanguage::JaJp);
        assert_eq!(jp_voices.len(), 1);
    }

    #[test]
    fn test_tts_request_validation_valid() {
        let req = TtsRequest::new("Hello world".to_string(), "en-us-female".to_string());
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_tts_request_validation_empty_text() {
        let req = TtsRequest::new("".to_string(), "en-us-female".to_string());
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_tts_request_validation_invalid_voice() {
        let req = TtsRequest::new("Hello".to_string(), "nonexistent".to_string());
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_tts_request_speed_clamping() {
        let req = TtsRequest::new("Hi".to_string(), "en-us-female".to_string())
            .with_speed(3.0);
        assert!((req.speed - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_synthesize_returns_pcm() {
        let mut engine = TtsEngine::new();
        let req = TtsRequest::new("Hi".to_string(), "en-us-female".to_string());
        let result = engine.synthesize(&req);
        assert!(result.is_ok());
        let audio = result.unwrap();
        assert!(!audio.is_empty());
        assert_eq!(audio.len() % 2, 0);
    }

    #[test]
    fn test_synthesize_invalid_voice() {
        let mut engine = TtsEngine::new();
        let req = TtsRequest::new("Hi".to_string(), "bad".to_string());
        assert!(engine.synthesize(&req).is_err());
    }

    #[test]
    fn test_cache_hit() {
        let mut engine = TtsEngine::new();
        let req = TtsRequest::new("Cache test".to_string(), "en-us-female".to_string());
        let first = engine.synthesize(&req).unwrap();
        let second = engine.synthesize(&req).unwrap();
        assert_eq!(first.len(), second.len());
    }

    #[test]
    fn test_clear_cache() {
        let mut engine = TtsEngine::new();
        let req = TtsRequest::new("Test".to_string(), "en-us-female".to_string());
        let _ = engine.synthesize(&req);
        assert_eq!(engine.cache_size(), 1);
        engine.clear_cache();
        assert_eq!(engine.cache_size(), 0);
    }

    #[test]
    fn test_list_languages() {
        let langs = list_languages();
        assert_eq!(langs.len(), 11);
        assert!(langs.iter().any(|(l, _)| *l == TtsLanguage::EnUs));
    }
}