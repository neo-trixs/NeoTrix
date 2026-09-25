//! NT-IO multimodal_transform — 音频模态: CPU TTS 流水线 (`nt_audio_transform`).
//!
//! P19 吸收 (pocket-tts: 快速语音加载)。纯确定性管线, 无真实音频 IO / 无 tokio。

use std::collections::HashMap;
use std::fmt;

// ────────────────────────────────────────────────────────────────────────────
// P19 cpu_tts — CPU TTS 流水线 (pocket-tts 吸收: 快速语音加载)
// 纯确定性管线, 无真实音频 IO / 无 tokio。
// ────────────────────────────────────────────────────────────────────────────

/// 语音状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceState {
    pub name: String,
    pub model_path: String,
    pub sample_rate: u32,
    pub load_ms: u64,
}

/// TTS 错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsError {
    UnknownVoice(String),
    EmptyText,
}

impl fmt::Display for TtsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TtsError::UnknownVoice(v) => write!(f, "unknown voice: {v}"),
            TtsError::EmptyText => write!(f, "empty text"),
        }
    }
}

/// 语音加载器 — 预加载语音即时返回 (load_ms=0); 其余按 cost map 模拟
/// safetensors 秒级加载。已加载语音入缓存, 后续加载即时。
pub struct VoiceLoader {
    preloaded: Vec<VoiceState>,
    loading_cost_ms: HashMap<String, u64>,
}

impl VoiceLoader {
    pub fn new(preloaded: Vec<VoiceState>, loading_cost_ms: HashMap<String, u64>) -> Self {
        Self {
            preloaded,
            loading_cost_ms,
        }
    }

    pub fn empty() -> Self {
        Self {
            preloaded: Vec::new(),
            loading_cost_ms: HashMap::new(),
        }
    }

    pub fn cache_len(&self) -> usize {
        self.preloaded.len()
    }

    pub fn load(&mut self, name: &str) -> Result<VoiceState, TtsError> {
        if let Some(v) = self.preloaded.iter().find(|v| v.name == name) {
            let mut v = v.clone();
            v.load_ms = 0; // 已驻留内存 → 加载即时
            return Ok(v);
        }
        if let Some(cost) = self.loading_cost_ms.get(name) {
            let v = VoiceState {
                name: name.to_string(),
                model_path: format!("models/{name}.safetensors"),
                sample_rate: 24_000,
                load_ms: *cost,
            };
            self.preloaded.push(v.clone());
            return Ok(v);
        }
        Err(TtsError::UnknownVoice(name.to_string()))
    }
}

/// TTS 请求。
#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub text: String,
    pub voice: String,
    pub speed: f64,
}

/// TTS 合成结果 (纯确定性, 无真实音频 IO)。
#[derive(Debug, Clone)]
pub struct TtsResult {
    pub samples_len: usize,
    pub estimated_ms: u64,
    pub voice: VoiceState,
}

/// CPU TTS 引擎。
pub struct CpuTtsEngine {
    pub loader: VoiceLoader,
}

impl CpuTtsEngine {
    pub fn new(loader: VoiceLoader) -> Self {
        Self { loader }
    }

    pub fn synthesize(&mut self, req: TtsRequest) -> Result<TtsResult, TtsError> {
        if req.text.trim().is_empty() {
            return Err(TtsError::EmptyText);
        }
        let voice = self.loader.load(&req.voice)?;
        // speed-normalized factor (ms 尺度): speed=1.0 → 1000, speed=2.0 → 500。
        let factor = (1000.0 / req.speed.max(0.1)) as u64;
        let samples_len =
            ((req.text.chars().count() as u64) * (voice.sample_rate as u64) * factor / 10_000) as usize;
        let playback_ms = (samples_len as u64) * 1000 / (voice.sample_rate as u64);
        let estimated_ms = playback_ms + voice.load_ms;
        Ok(TtsResult {
            samples_len,
            estimated_ms,
            voice,
        })
    }

    /// 实时性: 合成耗时 ≤ 音频播放时长 (RTF ≤ 1.0×)。
    pub fn is_realtime(&self, result: &TtsResult) -> bool {
        let playback_ms = (result.samples_len as u64) * 1000 / (result.voice.sample_rate as u64);
        result.estimated_ms <= playback_ms
    }
}

/// SelfTest (T1): "nt_io_multimodal_cpu_tts" — 快速语音加载 + 实时性自检。
impl crate::l0_substrate::nt_core_self_test::SelfTest for CpuTtsEngine {
    fn name(&self) -> &str {
        "nt_io_multimodal_cpu_tts"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let voice = VoiceState {
            name: "selftest".into(),
            model_path: "models/selftest.safetensors".into(),
            sample_rate: 24_000,
            load_ms: 0,
        };
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![voice], HashMap::new()));
        let req = TtsRequest {
            text: "selftest".into(),
            voice: "selftest".into(),
            speed: 1.0,
        };
        match engine.synthesize(req.clone()) {
            Ok(r) => {
                if !engine.is_realtime(&r) {
                    failures.push("preloaded voice must synthesize in realtime".into());
                }
                if let Err(e) = engine.synthesize(req) {
                    failures.push(format!("synthesize failed: {e}"));
                }
            }
            Err(e) => failures.push(format!("synthesize failed: {e}")),
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod cpu_tts_tests {
    use super::*;

    fn make_voice(name: &str) -> VoiceState {
        VoiceState {
            name: name.to_string(),
            model_path: format!("models/{name}.safetensors"),
            sample_rate: 24_000,
            load_ms: 0,
        }
    }

    // ── P19 cpu_tts ───────────────────────────────────────────

    #[test]
    fn cpu_tts_preloaded_voice_loads_instantly() {
        let mut loader = VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new());
        let v = loader.load("en").expect("preloaded");
        assert_eq!(v.load_ms, 0);
        assert_eq!(loader.cache_len(), 1);
    }

    #[test]
    fn cpu_tts_cold_voice_load_cost_then_cached() {
        let mut cost = std::collections::HashMap::new();
        cost.insert("zh".to_string(), 2500);
        let mut loader = VoiceLoader::new(vec![], cost);
        let first = loader.load("zh").expect("cold load");
        assert_eq!(first.load_ms, 2500, "cold load pays safetensors cost");
        assert_eq!(loader.cache_len(), 1);
        let second = loader.load("zh").expect("cached load");
        assert_eq!(second.load_ms, 0, "cached voice loads instantly");
    }

    #[test]
    fn cpu_tts_unknown_voice_is_err() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::empty());
        let req = TtsRequest {
            text: "hi".into(),
            voice: "ghost".into(),
            speed: 1.0,
        };
        assert_eq!(
            engine.synthesize(req).expect_err("must err"),
            TtsError::UnknownVoice("ghost".into())
        );
    }

    #[test]
    fn cpu_tts_empty_text_is_err() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let req = TtsRequest {
            text: String::new(),
            voice: "en".into(),
            speed: 1.0,
        };
        assert_eq!(engine.synthesize(req).expect_err("must err"), TtsError::EmptyText);
    }

    #[test]
    fn cpu_tts_samples_determinism() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let req = TtsRequest {
            text: "hello world".into(),
            voice: "en".into(),
            speed: 1.0,
        };
        let a = engine.synthesize(req.clone()).expect("ok");
        let b = engine.synthesize(req).expect("ok");
        assert_eq!(a.samples_len, b.samples_len, "samples must be deterministic");
        // chars(11) × 24000 × 1000 / 10000 = 26400
        assert_eq!(a.samples_len, 11 * 24_000 * 1000 / 10_000);
    }

    #[test]
    fn cpu_tts_speed_scales_samples() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let normal = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 1.0 })
            .expect("ok");
        let fast = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 2.0 })
            .expect("ok");
        assert_eq!(fast.samples_len, normal.samples_len / 2);
    }

    #[test]
    fn cpu_tts_realtime_check() {
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![make_voice("en")], std::collections::HashMap::new()));
        let r = engine
            .synthesize(TtsRequest { text: "hello world".into(), voice: "en".into(), speed: 1.0 })
            .expect("ok");
        assert!(engine.is_realtime(&r), "preloaded voice must be realtime");
    }

    #[test]
    fn cpu_tts_cold_load_not_realtime_then_realtime() {
        let mut cost = std::collections::HashMap::new();
        cost.insert("slow".to_string(), 5000);
        let mut engine = CpuTtsEngine::new(VoiceLoader::new(vec![], cost));
        let req = TtsRequest {
            text: "hello".into(),
            voice: "slow".into(),
            speed: 1.0,
        };
        let first = engine.synthesize(req.clone()).expect("ok");
        assert!(!engine.is_realtime(&first), "cold-load cost must exceed playback");
        let second = engine.synthesize(req).expect("ok");
        assert!(engine.is_realtime(&second), "cached voice becomes realtime");
    }

    #[test]
    fn cpu_tts_selftest_name_matches() {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let e = CpuTtsEngine::new(VoiceLoader::empty());
        assert_eq!(e.name(), "nt_io_multimodal_cpu_tts");
        assert!(e.self_test().is_ok());
    }
}
