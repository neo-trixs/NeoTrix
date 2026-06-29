#![forbid(unsafe_code)]

use std::collections::HashMap;

// ── Voice Profile ──

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

impl Gender {
    pub fn as_str(&self) -> &'static str {
        match self {
            Gender::Male => "male",
            Gender::Female => "female",
            Gender::Neutral => "neutral",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceProfile {
    pub name: String,
    pub gender: Gender,
    pub style_tags: Vec<String>,
    pub language: String,
}

impl VoiceProfile {
    pub fn new(name: &str, gender: Gender, style_tags: Vec<String>, language: &str) -> Self {
        Self {
            name: name.to_string(),
            gender,
            style_tags,
            language: language.to_string(),
        }
    }

    pub fn compatible_with(&self, language: &str) -> bool {
        self.language == language || self.language == "multilingual"
    }

    pub fn has_style(&self, style: &str) -> bool {
        self.style_tags.iter().any(|t| t == style)
    }
}

// ── Synthesis Request ──

#[derive(Debug, Clone)]
pub struct SynthesisRequest {
    pub text: String,
    pub voice_profile: VoiceProfile,
    pub api_endpoint: String,
    pub api_key_placeholder: String,
    pub parameters: HashMap<String, String>,
    pub estimated_duration_secs: f64,
}

// ── Voice Synth Error ──

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum VoiceSynthError {
    NoApiKey,
    EndpointUnreachable,
    UnsupportedVoice,
    SynthesisFailed(String),
}

impl std::fmt::Display for VoiceSynthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceSynthError::NoApiKey => write!(f, "no API key configured for TTS endpoint"),
            VoiceSynthError::EndpointUnreachable => write!(f, "TTS endpoint unreachable"),
            VoiceSynthError::UnsupportedVoice => {
                write!(f, "voice profile not supported by endpoint")
            }
            VoiceSynthError::SynthesisFailed(msg) => write!(f, "synthesis failed: {}", msg),
        }
    }
}

impl std::error::Error for VoiceSynthError {}

// ── Voice Designer (VoxCPM-inspired) ──

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceTraits {
    pub age: String,
    pub gender: Gender,
    pub tone: String,
    pub personality: Vec<String>,
    pub tempo: String,
}

impl VoiceTraits {
    pub fn new(
        age: &str,
        gender: Gender,
        tone: &str,
        personality: Vec<String>,
        tempo: &str,
    ) -> Self {
        Self {
            age: age.to_string(),
            gender,
            tone: tone.to_string(),
            personality,
            tempo: tempo.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoiceDesigner {
    pub descriptions: HashMap<String, String>,
}

impl VoiceDesigner {
    pub fn new() -> Self {
        Self {
            descriptions: HashMap::new(),
        }
    }

    pub fn generate_description(&self, traits: &VoiceTraits) -> String {
        let gender_str = traits.gender.as_str();
        let personality_str = traits.personality.join(", ");
        format!(
            "{} {}, voice with {} tone, {} personality, speaking at {} pace",
            traits.age, gender_str, traits.tone, personality_str, traits.tempo
        )
    }

    pub fn save_design(&mut self, name: &str, description: &str) {
        self.descriptions
            .insert(name.to_string(), description.to_string());
    }

    pub fn get_design(&self, name: &str) -> Option<&str> {
        self.descriptions.get(name).map(|s| s.as_str())
    }
}

impl Default for VoiceDesigner {
    fn default() -> Self {
        Self::new()
    }
}

// ── Synthesis Cache (LRU, max 50 entries) ──

const CACHE_MAX_ENTRIES: usize = 50;

#[derive(Debug, Clone)]
struct CacheEntry {
    request: SynthesisRequest,
    accessed: u64,
}

#[derive(Debug, Clone)]
pub struct SynthesisCache {
    entries: HashMap<(u64, String), CacheEntry>,
    access_order: Vec<(u64, String)>,
    next_access: u64,
    hits: u64,
    misses: u64,
}

impl SynthesisCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::with_capacity(CACHE_MAX_ENTRIES),
            next_access: 0,
            hits: 0,
            misses: 0,
        }
    }

    fn text_hash(text: &str) -> u64 {
        let mut h: u64 = 5381;
        for b in text.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        h
    }

    pub fn key(text: &str, voice_name: &str) -> (u64, String) {
        (Self::text_hash(text), voice_name.to_string())
    }

    pub fn get(&mut self, text: &str, voice_name: &str) -> Option<&SynthesisRequest> {
        let k = Self::key(text, voice_name);
        if let Some(entry) = self.entries.get_mut(&k) {
            entry.accessed = self.next_access;
            self.next_access += 1;
            self.hits += 1;
            if let Some(pos) = self.access_order.iter().position(|ak| ak == &k) {
                self.access_order.remove(pos);
                self.access_order.push(k);
            }
            Some(&entry.request)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn put(&mut self, request: SynthesisRequest) -> bool {
        let k = Self::key(&request.text, &request.voice_profile.name);
        let is_new = !self.entries.contains_key(&k);

        if is_new && self.entries.len() >= CACHE_MAX_ENTRIES {
            self.evict_one();
        }

        let entry = CacheEntry {
            request,
            accessed: self.next_access,
        };
        self.next_access += 1;

        if is_new {
            self.access_order.push(k.clone());
        }
        self.entries.insert(k, entry);
        is_new
    }

    fn evict_one(&mut self) {
        while let Some(front) = self.access_order.first() {
            let k = front.clone();
            self.access_order.remove(0);
            if self.entries.remove(&k).is_some() {
                break;
            }
        }
    }

    pub fn contains(&self, text: &str, voice_name: &str) -> bool {
        let k = Self::key(text, voice_name);
        self.entries.contains_key(&k)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.hits, self.misses)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.next_access = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

impl Default for SynthesisCache {
    fn default() -> Self {
        Self::new()
    }
}

// ── Predefined Voices ──

pub fn default_voices() -> Vec<VoiceProfile> {
    vec![
        VoiceProfile::new(
            "neutral_en_female",
            Gender::Female,
            vec!["neutral".to_string(), "clear".to_string()],
            "en",
        ),
        VoiceProfile::new(
            "neutral_en_male",
            Gender::Male,
            vec!["neutral".to_string(), "steady".to_string()],
            "en",
        ),
        VoiceProfile::new(
            "cheerful_en_female",
            Gender::Female,
            vec![
                "cheerful".to_string(),
                "bright".to_string(),
                "upbeat".to_string(),
            ],
            "en",
        ),
        VoiceProfile::new(
            "professional_en_male",
            Gender::Male,
            vec![
                "professional".to_string(),
                "authoritative".to_string(),
                "polished".to_string(),
            ],
            "en",
        ),
        VoiceProfile::new(
            "warm_en_female",
            Gender::Female,
            vec![
                "warm".to_string(),
                "gentle".to_string(),
                "soothing".to_string(),
            ],
            "en",
        ),
    ]
}

// ── Voice Synthesis Engine (Bridge / Planning Layer) ──

#[derive(Debug, Clone)]
pub struct VoiceSynthesisEngine {
    pub voices: Vec<VoiceProfile>,
    pub default_endpoint: String,
    pub api_key_var: String,
    pub cache: SynthesisCache,
}

impl VoiceSynthesisEngine {
    pub fn new(endpoint: &str, api_key_var: &str) -> Self {
        Self {
            voices: default_voices(),
            default_endpoint: endpoint.to_string(),
            api_key_var: api_key_var.to_string(),
            cache: SynthesisCache::new(),
        }
    }

    pub fn synthesize(
        &mut self,
        text: &str,
        voice: &VoiceProfile,
    ) -> Result<SynthesisRequest, VoiceSynthError> {
        if text.trim().is_empty() {
            return Err(VoiceSynthError::SynthesisFailed(
                "empty text provided for synthesis".to_string(),
            ));
        }

        if !self.supports_voice(voice) {
            return Err(VoiceSynthError::UnsupportedVoice);
        }

        let estimated = self.estimate_duration(text, voice);

        let mut parameters = HashMap::new();
        parameters.insert("voice".to_string(), voice.name.clone());
        parameters.insert("language".to_string(), voice.language.clone());
        parameters.insert("gender".to_string(), voice.gender.as_str().to_string());
        for (i, tag) in voice.style_tags.iter().enumerate() {
            parameters.insert(format!("style_{}", i), tag.clone());
        }

        let request = SynthesisRequest {
            text: text.to_string(),
            voice_profile: voice.clone(),
            api_endpoint: self.default_endpoint.clone(),
            api_key_placeholder: format!("${{{}}}", self.api_key_var),
            parameters,
            estimated_duration_secs: estimated,
        };

        self.cache.put(request.clone());
        Ok(request)
    }

    pub fn supported_voices(&self) -> Vec<VoiceProfile> {
        self.voices.clone()
    }

    pub fn supports_voice(&self, voice: &VoiceProfile) -> bool {
        self.voices.iter().any(|v| v.name == voice.name)
    }

    pub fn estimate_duration(&self, text: &str, voice: &VoiceProfile) -> f64 {
        let char_count = text.chars().count().max(1) as f64;

        let pace = if voice.has_style("cheerful") {
            12.0
        } else if voice.has_style("professional") {
            10.0
        } else if voice.has_style("warm")
            || voice.has_style("gentle")
            || voice.has_style("soothing")
        {
            8.0
        } else {
            10.0
        };

        let silence_breaks = text
            .chars()
            .filter(|&c| c == ',' || c == '.' || c == '!' || c == '?' || c == ';' || c == ':')
            .count() as f64;
        let pause_time = silence_breaks * 0.15;

        char_count / pace + pause_time
    }

    pub fn add_voice(&mut self, voice: VoiceProfile) {
        if !self.supports_voice(&voice) {
            self.voices.push(voice);
        }
    }

    pub fn with_cache(mut self, cache: SynthesisCache) -> Self {
        self.cache = cache;
        self
    }

    pub fn check_language_compatibility(&self, voice: &VoiceProfile, language: &str) -> bool {
        voice.compatible_with(language)
    }

    pub fn pending_count(&self) -> usize {
        0
    }
}

// ── TTS Bridge (Unified entry point) ──

#[derive(Debug)]
pub struct TtsBridge {
    pub engine: VoiceSynthesisEngine,
    pub designer: VoiceDesigner,
}

impl TtsBridge {
    pub fn new(endpoint: &str, api_key_var: &str) -> Self {
        Self {
            engine: VoiceSynthesisEngine::new(endpoint, api_key_var),
            designer: VoiceDesigner::new(),
        }
    }

    pub fn speak(
        &mut self,
        text: &str,
        voice: &VoiceProfile,
    ) -> Result<SynthesisRequest, VoiceSynthError> {
        if let Some(cached) = self.engine.cache.get(text, &voice.name) {
            return Ok(cached.clone());
        }
        self.engine.synthesize(text, voice)
    }

    pub fn speak_designed(
        &mut self,
        text: &str,
        design_name: &str,
    ) -> Result<SynthesisRequest, VoiceSynthError> {
        let description = self
            .designer
            .get_design(design_name)
            .ok_or(VoiceSynthError::UnsupportedVoice)?;

        let voice = VoiceProfile::new(
            design_name,
            Gender::Neutral,
            vec![description.to_string()],
            "en",
        );
        self.engine.add_voice(voice.clone());
        self.speak(text, &voice)
    }

    pub fn voices(&self) -> Vec<VoiceProfile> {
        self.engine.supported_voices()
    }

    pub fn cache_stats(&self) -> (u64, u64) {
        self.engine.cache.stats()
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_profile_creation() {
        let v = VoiceProfile::new(
            "test_voice",
            Gender::Female,
            vec!["warm".to_string(), "gentle".to_string()],
            "en",
        );
        assert_eq!(v.name, "test_voice");
        assert_eq!(v.gender, Gender::Female);
        assert!(v.has_style("warm"));
        assert!(!v.has_style("angry"));
        assert_eq!(v.language, "en");
    }

    #[test]
    fn test_gender_as_str() {
        assert_eq!(Gender::Male.as_str(), "male");
        assert_eq!(Gender::Female.as_str(), "female");
        assert_eq!(Gender::Neutral.as_str(), "neutral");
    }

    #[test]
    fn test_voice_designer_description() {
        let designer = VoiceDesigner::new();
        let traits = VoiceTraits::new(
            "young",
            Gender::Female,
            "sweet",
            vec!["gentle".to_string(), "friendly".to_string()],
            "moderate",
        );
        let desc = designer.generate_description(&traits);
        assert!(desc.contains("young"));
        assert!(desc.contains("female"));
        assert!(desc.contains("sweet"));
        assert!(desc.contains("gentle"));
        assert!(desc.contains("moderate"));
    }

    #[test]
    fn test_voice_designer_save_and_get() {
        let mut designer = VoiceDesigner::new();
        designer.save_design("sweet_girl", "young woman, gentle and sweet voice");
        assert_eq!(
            designer.get_design("sweet_girl"),
            Some("young woman, gentle and sweet voice")
        );
        assert_eq!(designer.get_design("nonexistent"), None);
    }

    #[test]
    fn test_synthesis_request_creation() {
        let mut engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let voice = default_voices()[0].clone();
        let req = engine.synthesize("Hello world", &voice).unwrap();
        assert_eq!(req.text, "Hello world");
        assert_eq!(req.voice_profile.name, "neutral_en_female");
        assert_eq!(req.api_endpoint, "https://tts.example.com/api");
        assert_eq!(req.api_key_placeholder, "${TTS_API_KEY}");
        assert!(req.parameters.contains_key("style_0"));
        assert!(req.estimated_duration_secs > 0.0);
    }

    #[test]
    fn test_estimate_duration() {
        let engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");

        let neutral_voice = VoiceProfile::new(
            "neutral",
            Gender::Neutral,
            vec!["neutral".to_string()],
            "en",
        );
        let warm_voice = VoiceProfile::new(
            "warm",
            Gender::Female,
            vec!["warm".to_string(), "soothing".to_string()],
            "en",
        );
        let cheerful_voice = VoiceProfile::new(
            "cheerful",
            Gender::Female,
            vec!["cheerful".to_string()],
            "en",
        );

        let short = engine.estimate_duration("Hi", &neutral_voice);
        let long =
            engine.estimate_duration("Hello, world! How are you? I am fine.", &neutral_voice);
        assert!(long > short);

        let same_text = "Hello world";
        let neutral_dur = engine.estimate_duration(same_text, &neutral_voice);
        let warm_dur = engine.estimate_duration(same_text, &warm_voice);
        let cheerful_dur = engine.estimate_duration(same_text, &cheerful_voice);

        assert!(warm_dur > neutral_dur);
        assert!(cheerful_dur > warm_dur || (cheerful_dur - warm_dur).abs() < 1.0);
    }

    #[test]
    fn test_cache_hit() {
        let mut engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let voice = default_voices()[0].clone();
        let _ = engine.synthesize("Cache me", &voice).unwrap();

        let mut bridge = TtsBridge::new("https://tts.example.com/api", "TTS_API_KEY");
        bridge.engine.voices = engine.voices.clone();
        bridge.engine.cache = engine.cache;

        let (hits_before, _) = bridge.cache_stats();
        let _ = bridge.speak("Cache me", &voice).unwrap();
        let (hits_after, _) = bridge.cache_stats();

        assert!(hits_after > hits_before);
    }

    #[test]
    fn test_cache_miss() {
        let mut bridge = TtsBridge::new("https://tts.example.com/api", "TTS_API_KEY");
        let voice = default_voices()[0].clone();
        let (_, misses_before) = bridge.cache_stats();
        let result = bridge.speak("Never seen before", &voice);
        let (_, misses_after) = bridge.cache_stats();
        assert!(result.is_ok());
        assert_eq!(misses_after, misses_before + 1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let voice = default_voices()[0].clone();
        engine.cache = SynthesisCache::new();

        for i in 0..CACHE_MAX_ENTRIES {
            let text = format!("Line number {}", i);
            let _ = engine.synthesize(&text, &voice).unwrap();
        }

        assert_eq!(engine.cache.len(), CACHE_MAX_ENTRIES);

        let _ = engine
            .synthesize("One more to trigger eviction", &voice)
            .unwrap();
        assert_eq!(engine.cache.len(), CACHE_MAX_ENTRIES);

        let first_text = "Line number 0";
        let still_there = engine.cache.contains(first_text, &voice.name);

        let second_text = "Line number 1";
        assert!(!still_there || !engine.cache.contains(second_text, &voice.name));
    }

    #[test]
    fn test_supported_voices_listing() {
        let engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let voices = engine.supported_voices();
        assert_eq!(voices.len(), 5);
        let names: Vec<&str> = voices.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"neutral_en_female"));
        assert!(names.contains(&"neutral_en_male"));
        assert!(names.contains(&"cheerful_en_female"));
        assert!(names.contains(&"professional_en_male"));
        assert!(names.contains(&"warm_en_female"));
    }

    #[test]
    fn test_error_empty_text() {
        let mut engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let voice = default_voices()[0].clone();
        let result = engine.synthesize("", &voice);
        assert!(result.is_err());
        match result.unwrap_err() {
            VoiceSynthError::SynthesisFailed(msg) => assert!(msg.contains("empty")),
            other => panic!("expected SynthesisFailed, got {:?}", other),
        }
    }

    #[test]
    fn test_language_compatibility() {
        let engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let en_voice = default_voices()[0].clone();

        assert!(engine.check_language_compatibility(&en_voice, "en"));
        assert!(!engine.check_language_compatibility(&en_voice, "zh"));

        let multilingual = VoiceProfile::new("multi", Gender::Neutral, vec![], "multilingual");
        assert!(engine.check_language_compatibility(&multilingual, "en"));
        assert!(engine.check_language_compatibility(&multilingual, "zh"));
        assert!(engine.check_language_compatibility(&multilingual, "fr"));
    }

    #[test]
    fn test_multi_style_voice_composition() {
        let voice = VoiceProfile::new(
            "expressive_ja_female",
            Gender::Female,
            vec![
                "cheerful".to_string(),
                "energetic".to_string(),
                "friendly".to_string(),
                "expressive".to_string(),
            ],
            "ja",
        );

        assert_eq!(voice.language, "ja");
        assert_eq!(voice.gender, Gender::Female);
        assert!(voice.has_style("cheerful"));
        assert!(voice.has_style("energetic"));
        assert!(voice.has_style("expressive"));
        assert!(!voice.has_style("monotone"));
        assert!(!voice.compatible_with("en"));
        assert!(voice.compatible_with("ja"));
    }

    #[test]
    fn test_tts_bridge_speak_designed() {
        let mut bridge = TtsBridge::new("https://tts.example.com/api", "TTS_API_KEY");
        bridge.designer.save_design(
            "mature_professional",
            "middle-aged man, authoritative and calm voice",
        );

        let result = bridge.speak_designed("Welcome to the presentation.", "mature_professional");
        assert!(result.is_ok());

        let req = result.unwrap();
        assert_eq!(req.text, "Welcome to the presentation.");
        assert_eq!(req.voice_profile.name, "mature_professional");
    }

    #[test]
    fn test_voice_synth_error_display() {
        let errs: Vec<VoiceSynthError> = vec![
            VoiceSynthError::NoApiKey,
            VoiceSynthError::EndpointUnreachable,
            VoiceSynthError::UnsupportedVoice,
            VoiceSynthError::SynthesisFailed("connection timeout".to_string()),
        ];
        let msgs: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
        assert!(msgs[0].contains("API key"));
        assert!(msgs[1].contains("unreachable"));
        assert!(msgs[2].contains("voice"));
        assert!(msgs[3].contains("timeout"));
    }

    #[test]
    fn test_unsupported_voice_error() {
        let mut engine = VoiceSynthesisEngine::new("https://tts.example.com/api", "TTS_API_KEY");
        let unknown = VoiceProfile::new("unknown_voice", Gender::Neutral, vec![], "xx");
        let result = engine.synthesize("hello", &unknown);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), VoiceSynthError::UnsupportedVoice);
    }
}
