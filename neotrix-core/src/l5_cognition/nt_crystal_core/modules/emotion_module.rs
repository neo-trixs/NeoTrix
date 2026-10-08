use super::super::ctm::{Chunk, ChunkType, CTMModule, ModuleType};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct NtEmotionModule {
    valence: Mutex<f64>,
    arousal: Mutex<f64>,
    dominance: Mutex<f64>,
}

impl Default for NtEmotionModule {
    fn default() -> Self {
        Self::new()
    }
}

impl NtEmotionModule {
    pub fn new() -> Self {
        Self {
            valence: Mutex::new(0.0),
            arousal: Mutex::new(0.5),
            dominance: Mutex::new(0.5),
        }
    }
}

impl CTMModule for NtEmotionModule {
    fn name(&self) -> &str {
        "NT-EMOTION"
    }

    fn module_type(&self) -> ModuleType {
        ModuleType::Emotion
    }

    fn execute(&self, input: &Chunk) -> Chunk {
        let input_valence = input.metadata.get("valence").and_then(|v| v.parse::<f64>().ok());
        let input_arousal = input.metadata.get("arousal").and_then(|v| v.parse::<f64>().ok());

        let new_valence = input_valence.unwrap_or_else(|| match input.chunk_type {
            ChunkType::Emotion => input.score * 0.5,
            ChunkType::Safety => -0.2,
            ChunkType::Override => -0.5,
            ChunkType::Memory => 0.1,
            _ => 0.0,
        });

        let new_arousal = input_arousal.unwrap_or(match input.chunk_type {
            ChunkType::Override => 0.9,
            ChunkType::Safety => 0.7,
            ChunkType::Perception => 0.6,
            _ => 0.5,
        });

        let dominant = if input.score > 0.7 { 0.7 } else { 0.5 };

        let old_v = *self.valence.lock().unwrap();
        *self.valence.lock().unwrap() = (old_v * 0.7 + new_valence * 0.3).clamp(-1.0, 1.0);
        let old_a = *self.arousal.lock().unwrap();
        *self.arousal.lock().unwrap() = (old_a * 0.7 + new_arousal * 0.3).clamp(0.0, 1.0);
        let old_d = *self.dominance.lock().unwrap();
        *self.dominance.lock().unwrap() = (old_d * 0.8 + dominant * 0.2).clamp(0.0, 1.0);

        let v = *self.valence.lock().unwrap();
        let a = *self.arousal.lock().unwrap();
        let d = *self.dominance.lock().unwrap();

        let emotion_label = match (v > 0.2, a > 0.6, d > 0.6) {
            (true, true, true) => "confident",
            (true, true, false) => "excited",
            (true, false, _) => "calm",
            (false, true, true) => "angry",
            (false, true, false) => "anxious",
            (false, false, _) => "sad",
        };

        Chunk {
            content: format!(
                "Emotional state: {} (V={:.2}, A={:.2}, D={:.2})",
                emotion_label, v, a, d
            ),
            score: a * 0.6 + (1.0 - v.abs()) * 0.4,
            source_module: "NT-EMOTION".into(),
            chunk_type: ChunkType::Emotion,
            metadata: {
                let mut m = HashMap::new();
                m.insert("valence".into(), format!("{:.3}", v));
                m.insert("arousal".into(), format!("{:.3}", a));
                m.insert("dominance".into(), format!("{:.3}", d));
                m.insert("emotion_label".into(), emotion_label.into());
                m
            },
        }
    }

    fn write(&mut self, chunk: Chunk) {
        if let Some(v) = chunk.metadata.get("valence").and_then(|v| v.parse::<f64>().ok()) {
            *self.valence.lock().unwrap() = v;
        }
        if let Some(a) = chunk.metadata.get("arousal").and_then(|v| v.parse::<f64>().ok()) {
            *self.arousal.lock().unwrap() = a;
        }
        if let Some(d) = chunk.metadata.get("dominance").and_then(|v| v.parse::<f64>().ok()) {
            *self.dominance.lock().unwrap() = d;
        }
    }

    fn max_response_time_ms(&self) -> u64 {
        300
    }
}
