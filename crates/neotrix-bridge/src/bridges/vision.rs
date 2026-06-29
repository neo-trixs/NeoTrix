use crate::types::{
    BridgeHealth, ConsciousnessAbility, CuriositySignal, Domain, GraceMode, IntentionVsa, VsaLight,
    VsaOrigin, VsaTagged, WorldEffect, Sensory,
};

pub struct VisionBridge {
    pub vsa: VsaLight,
    pub vision_available: bool,
    pub scenes_analyzed: u64,
    pub total_actuations: u64,
    pub last_vision_ms: i64,
    pub error_count: u64,
    pub supported_capabilities: Vec<String>,
}

impl Default for VisionBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl VisionBridge {
    pub fn new() -> Self {
        Self {
            vsa: VsaLight::new(4096),
            vision_available: false,
            scenes_analyzed: 0,
            total_actuations: 0,
            last_vision_ms: 0,
            error_count: 0,
            supported_capabilities: vec![
                "ocr".into(),
                "scene_description".into(),
                "object_detection".into(),
                "capture_screen".into(),
                "image_analysis".into(),
            ],
        }
    }

    fn seed_for_label(&self, label: &str, idx: usize) -> u64 {
        let mut s: u64 = 0xcafe_babe;
        for b in label.bytes() {
            s = s.wrapping_mul(31).wrapping_add(b as u64);
        }
        s.wrapping_add(idx as u64)
    }

    fn known_vectors(&self) -> Vec<Vec<u8>> {
        self.supported_capabilities
            .iter()
            .enumerate()
            .map(|(i, cap)| self.vsa.seeded_vector(self.seed_for_label(cap, i)))
            .collect()
    }
}

impl ConsciousnessAbility for VisionBridge {
    fn domain(&self) -> Domain {
        Domain::Vision
    }

    fn sense(&mut self) -> Vec<VsaTagged> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        self.last_vision_ms = now;

        if !self.vision_available {
            return Vec::new();
        }

        let known = self.known_vectors();
        let mut results = Vec::with_capacity(self.supported_capabilities.len());

        for (i, cap) in self.supported_capabilities.iter().enumerate() {
            let seed = self.seed_for_label(cap, i);
            let vector = self.vsa.seeded_vector(seed);

            let novelty = self.vsa.novelty(&known, &vector, 0.85);
            let info_density = match cap.as_str() {
                "scene_description" => 0.9,
                "object_detection" => 0.85,
                "ocr" => 0.75,
                "capture_screen" => 0.8,
                "image_analysis" => 0.7,
                _ => 0.6,
            };
            let negentropy_contribution = novelty * 0.4 + info_density * 0.6;

            results.push(VsaTagged {
                vector,
                origin: VsaOrigin::World(Sensory::VisionFrame),
                timestamp_ms: now,
                negentropy_contribution: (negentropy_contribution * 100.0).round() / 100.0,
            });
        }

        self.scenes_analyzed += 1;
        results
    }

    fn actuate(&mut self, intention: &IntentionVsa) -> Result<WorldEffect, String> {
        self.total_actuations += 1;

        if !self.vision_available {
            self.error_count += 1;
            return Err("vision module not available".into());
        }

        let start = std::time::Instant::now();

        match intention.action.as_str() {
            "capture_screen" => {
                let region = intention
                    .parameters
                    .get("region")
                    .and_then(|v| v.as_str())
                    .unwrap_or("full");
                Ok(WorldEffect {
                    domain: Domain::Vision,
                    description: format!("screen_captured region={}", region),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "analyze_image" => {
                let path = intention
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                self.scenes_analyzed += 1;
                Ok(WorldEffect {
                    domain: Domain::Vision,
                    description: format!("image_analyzed path={}", path),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "ocr" => {
                let path = intention
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                Ok(WorldEffect {
                    domain: Domain::Vision,
                    description: format!("ocr_extracted path={}", path),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            "describe_scene" => {
                let source = intention
                    .parameters
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("viewport");
                self.scenes_analyzed += 1;
                Ok(WorldEffect {
                    domain: Domain::Vision,
                    description: format!("scene_described source={}", source),
                    success: true,
                    latency_ms: start.elapsed().as_millis() as u64,
                })
            }
            other => {
                self.error_count += 1;
                Err(format!("unknown vision action: {}", other))
            }
        }
    }

    fn curiosity_signals(&self) -> Vec<CuriositySignal> {
        if !self.vision_available {
            return Vec::new();
        }

        let known = self.known_vectors();
        let mut signals = Vec::with_capacity(self.supported_capabilities.len());

        for (i, cap) in self.supported_capabilities.iter().enumerate() {
            let seed = self.seed_for_label(cap, i);
            let vector = self.vsa.seeded_vector(seed);
            let novelty = self.vsa.novelty(&known, &vector, 0.8);

            let query = match cap.as_str() {
                "scene_description" => "unanalyzed_scene",
                "object_detection" => "unrecognized_object",
                "ocr" => "unread_text",
                "capture_screen" => "fresh_viewport",
                "image_analysis" => "deep_scan",
                _ => "visual_pattern",
            };

            signals.push(CuriositySignal {
                domain: Domain::Vision,
                query: format!("{}:{}", query, cap),
                novelty_estimate: (novelty * 100.0).round() / 100.0,
                potential_negentropy: (novelty * 0.6 + 0.4)
                    .min(1.0)
                    .round(),
            });
        }

        signals
    }

    fn grace_mode(&self) -> GraceMode {
        GraceMode::SkipSilently
    }

    fn health(&self) -> BridgeHealth {
        BridgeHealth {
            domain: Domain::Vision,
            available: self.vision_available,
            last_seen_ms: self.last_vision_ms,
            error_count: self.error_count,
            total_actuations: self.total_actuations,
        }
    }

    fn probe_available(&self) -> bool {
        self.vision_available
    }

    fn negentropy_estimate(&self) -> f64 {
        if !self.vision_available {
            return 0.0;
        }
        let visual_input = 0.5;
        let scene_complexity = (self.scenes_analyzed as f64).clamp(0.0, 50.0) / 50.0 * 0.5;
        (visual_input + scene_complexity).min(1.0)
    }
}
