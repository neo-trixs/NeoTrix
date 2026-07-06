use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Modality {
    Text,
    Image,
    Video,
    Audio,
    Code,
    Mixed(Vec<Modality>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaOutput {
    pub modality: Modality,
    pub content_type: String,
    pub data: Vec<u8>,
    pub metadata: std::collections::HashMap<String, String>,
    pub embedding: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaScore {
    pub overall: f64,
    pub relevance: f64,
    pub quality: f64,
    pub coherence: f64,
}

pub trait MediaCapability: Send + Sync {
    fn modality(&self) -> Modality;
    fn produce(&self, prompt: &[f64], params: &std::collections::HashMap<String, String>) -> Result<MediaOutput, String>;
    fn score(&self, output: &MediaOutput) -> MediaScore;
    fn embed(&self, output: &MediaOutput) -> Result<Vec<f64>, String>;
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRegistry {
    capabilities: Vec<MediaCapabilityDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCapabilityDescriptor {
    pub name: String,
    pub modality: Modality,
    pub provider: String,
    pub is_available: bool,
    pub avg_latency_ms: f64,
    pub quality_score: f64,
}

impl MediaRegistry {
    pub fn new() -> Self {
        Self { capabilities: Vec::new() }
    }

    pub fn register(&mut self, desc: MediaCapabilityDescriptor) {
        self.capabilities.push(desc);
    }

    pub fn find_by_modality(&self, modality: &Modality) -> Vec<&MediaCapabilityDescriptor> {
        self.capabilities.iter()
            .filter(|c| c.modality == *modality && c.is_available)
            .collect()
    }

    pub fn best_for_modality(&self, modality: &Modality) -> Option<&MediaCapabilityDescriptor> {
        let mut candidates: Vec<&MediaCapabilityDescriptor> = self.capabilities.iter()
            .filter(|c| c.modality == *modality && c.is_available)
            .collect();
        candidates.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap_or(std::cmp::Ordering::Equal));
        candidates.into_iter().next()
    }

    pub fn all(&self) -> &[MediaCapabilityDescriptor] {
        &self.capabilities
    }

    pub fn count(&self) -> usize {
        self.capabilities.len()
    }
}

impl Default for MediaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_equality() {
        assert_eq!(Modality::Text, Modality::Text);
        assert_ne!(Modality::Text, Modality::Image);
    }

    #[test]
    fn test_media_score_defaults() {
        let s = MediaScore { overall: 0.85, relevance: 0.9, quality: 0.8, coherence: 0.85 };
        assert!((s.overall - 0.85).abs() < 1e-10);
    }

    #[test]
    fn test_media_registry_register_and_find() {
        let mut reg = MediaRegistry::new();
        reg.register(MediaCapabilityDescriptor {
            name: "sd-txt2img".into(), modality: Modality::Image,
            provider: "Stable Diffusion MCP".into(), is_available: true,
            avg_latency_ms: 2500.0, quality_score: 0.85,
        });
        reg.register(MediaCapabilityDescriptor {
            name: "voicebox-tts".into(), modality: Modality::Audio,
            provider: "Voicebox MCP".into(), is_available: true,
            avg_latency_ms: 800.0, quality_score: 0.92,
        });
        let images = reg.find_by_modality(&Modality::Image);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].name, "sd-txt2img");
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_best_for_modality_returns_highest_quality() {
        let mut reg = MediaRegistry::new();
        reg.register(MediaCapabilityDescriptor {
            name: "fast".into(), modality: Modality::Text,
            provider: "P1".into(), is_available: true,
            avg_latency_ms: 100.0, quality_score: 0.7,
        });
        reg.register(MediaCapabilityDescriptor {
            name: "quality".into(), modality: Modality::Text,
            provider: "P2".into(), is_available: true,
            avg_latency_ms: 500.0, quality_score: 0.95,
        });
        let best = reg.best_for_modality(&Modality::Text).unwrap();
        assert_eq!(best.name, "quality");
    }

    #[test]
    fn test_find_by_modality_ignores_unavailable() {
        let mut reg = MediaRegistry::new();
        reg.register(MediaCapabilityDescriptor {
            name: "offline".into(), modality: Modality::Video,
            provider: "Offline".into(), is_available: false,
            avg_latency_ms: 0.0, quality_score: 0.9,
        });
        let videos = reg.find_by_modality(&Modality::Video);
        assert!(videos.is_empty());
    }

    #[test]
    fn test_media_output_fields() {
        let mut meta = std::collections::HashMap::new();
        meta.insert("width".into(), "1024".into());
        meta.insert("height".into(), "768".into());
        let out = MediaOutput {
            modality: Modality::Image,
            content_type: "image/png".into(),
            data: vec![0u8; 100],
            metadata: meta,
            embedding: None,
        };
        assert_eq!(out.metadata.get("width").unwrap(), "1024");
        assert_eq!(out.data.len(), 100);
    }
}
