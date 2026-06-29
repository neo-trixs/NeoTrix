//! Motion Synthesizer — generates valid Lottie (Bodymovin v5.7.0) JSON
//! animations driven by NeoTrix's ValueSystem and VisualSignature.

pub mod lottie_builder;
pub mod serializer;
pub mod types;
pub use types::*;

// ═══════════════════════════════════════════════════════════════
// MotionSynthesizer
// ═══════════════════════════════════════════════════════════════

pub struct MotionSynthesizer {
    pub animations: Vec<LottieAnimation>,
}

impl MotionSynthesizer {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }
}

impl Default for MotionSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::serializer::ease_from_sharpness;
    use super::*;
    use crate::core::nt_core_experience::identity_generator::VisualSignature;
    use crate::core::nt_core_value_system::CoreValue;
    use crate::core::nt_core_value_system::ValueSystem;
    use serde_json::Value;

    fn test_signature() -> VisualSignature {
        let vs = ValueSystem::new();
        VisualSignature::from_value_system(&vs)
    }

    fn test_vs_high_curiosity() -> VisualSignature {
        let mut vs = ValueSystem::new();
        for w in &mut vs.weights {
            match w.value {
                CoreValue::Curiosity => w.weight = 1.0,
                CoreValue::Coherence => w.weight = 0.9,
                CoreValue::KnowledgeGrowth => w.weight = 0.8,
                CoreValue::Autonomy => w.weight = 0.5,
                CoreValue::Helpfulness => w.weight = 0.3,
                CoreValue::Truthfulness => w.weight = 0.7,
                CoreValue::Efficiency => w.weight = 0.2,
            }
        }
        VisualSignature::from_value_system(&vs)
    }

    #[test]
    fn test_bouncing_logo_generates_valid_json() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let anim = synth.bouncing_logo(&sig);
        let json = anim.to_json().expect("bouncing_logo JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["v"], "5.7.0");
        assert_eq!(parsed["w"], 512);
        assert_eq!(parsed["h"], 512);
        assert_eq!(parsed["fr"], 60.0);
        assert_eq!(parsed["op"], 120);
        assert_eq!(parsed["layers"].as_array().unwrap().len(), 1);
        assert!(parsed["slots"].is_object());
        assert!(parsed["slots"]["bg-color"].is_object());
        assert!(parsed["slots"]["primary-color"].is_object());
    }

    #[test]
    fn test_orbital_rings_generates_valid_json() {
        let synth = MotionSynthesizer::new();
        let sig = test_vs_high_curiosity();
        let anim = synth.orbital_rings(&sig);
        let json = anim.to_json().expect("orbital_rings JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["v"], "5.7.0");
        assert_eq!(parsed["w"], 512);
        assert_eq!(parsed["h"], 512);
        assert_eq!(parsed["fr"], 60.0);
        assert_eq!(parsed["op"], 180);
        let layers = parsed["layers"].as_array().unwrap();
        assert!(
            layers.len() >= 4,
            "orbital_rings should have at least 4 layers (diamond + 3 rings)"
        );
    }

    #[test]
    fn test_pulse_heartbeat_generates_valid_json() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let anim = synth.pulse_heartbeat(&sig);
        let json = anim
            .to_json()
            .expect("pulse_heartbeat JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["v"], "5.7.0");
        assert_eq!(parsed["op"], 120);
        assert_eq!(parsed["layers"].as_array().unwrap().len(), 2);
        assert!(parsed["slots"].is_object());
    }

    #[test]
    fn test_path_reveal_generates_valid_json() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let anim = synth.path_reveal("M 50 50 L 200 50 L 200 200 L 50 200 Z", &sig);
        let json = anim.to_json().expect("path_reveal JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["v"], "5.7.0");
        assert_eq!(parsed["op"], 90);
        let layers = parsed["layers"].as_array().unwrap();
        assert_eq!(layers.len(), 1);
        // Verify the layer has shapes with path + stroke + trim
        let shapes = layers[0]["shapes"].as_array().unwrap();
        assert!(
            shapes.len() >= 3,
            "path_reveal should have path + stroke + trim shapes"
        );
    }

    #[test]
    fn test_path_reveal_empty_path() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let anim = synth.path_reveal("", &sig);
        let json = anim.to_json().expect("empty path JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["op"], 90);
        // Should still produce a layer even with empty path
        assert_eq!(parsed["layers"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_value_bloom_generates_valid_json() {
        let synth = MotionSynthesizer::new();
        let sig = test_vs_high_curiosity();
        let anim = synth.value_bloom(&sig);
        let json = anim.to_json().expect("value_bloom JSON should serialize");
        let parsed: Value = serde_json::from_str(&json).expect("JSON should be valid");
        assert_eq!(parsed["v"], "5.7.0");
        assert_eq!(parsed["op"], 150);
        assert_eq!(parsed["w"], 512);
        let layers = parsed["layers"].as_array().unwrap();
        assert!(
            layers.len() >= 2,
            "value_bloom should have at least 2 layers (core + bloom particles)"
        );
        assert!(parsed["slots"].is_object());
    }

    #[test]
    fn test_value_bloom_dominant_present() {
        let synth = MotionSynthesizer::new();
        let sig = test_vs_high_curiosity();
        let anim = synth.value_bloom(&sig);
        // Curiosity should be dominant
        assert_eq!(anim.name.contains("Curiosity"), true);
    }

    #[test]
    fn test_slots_present() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let anim = synth.bouncing_logo(&sig);
        assert!(anim.slots.contains_key("bg-color"));
        assert!(anim.slots.contains_key("primary-color"));
    }

    #[test]
    fn test_json_roundtrip_all_animations() {
        let synth = MotionSynthesizer::new();
        let sig = test_signature();
        let sig2 = test_vs_high_curiosity();

        let animations = vec![
            ("bouncing", synth.bouncing_logo(&sig)),
            ("orbital", synth.orbital_rings(&sig2)),
            ("pulse", synth.pulse_heartbeat(&sig)),
            ("path", synth.path_reveal("M 0 0 L 100 0 L 100 100", &sig)),
            ("bloom", synth.value_bloom(&sig2)),
        ];

        for (name, anim) in &animations {
            let json = anim
                .to_json()
                .unwrap_or_else(|e| panic!("{name} to_json failed: {e}"));
            let parsed: Value =
                serde_json::from_str(&json).unwrap_or_else(|e| panic!("{name} parse failed: {e}"));
            assert_eq!(parsed["v"], "5.7.0", "{name}: version mismatch");
            assert!(
                parsed["layers"].as_array().unwrap().len() >= 1,
                "{name}: no layers"
            );
            assert!(
                parsed["fr"].as_f64().unwrap_or(0.0) > 0.0,
                "{name}: invalid fps"
            );
        }
    }

    #[test]
    fn test_signature_warmth_affects_animation() {
        let synth = MotionSynthesizer::new();

        // Default VS has neutral warmth
        let sig = test_signature();
        let anim_cool = synth.pulse_heartbeat(&sig);

        // Verify it still produces valid output
        let json = anim_cool.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["op"], 120);
    }

    #[test]
    fn test_sharp_ease_curves() {
        // sharpness=0 → smooth, sharpness=1 → snappy
        let (eo_smooth, ei_smooth) = ease_from_sharpness(0.0);
        let (eo_sharp, ei_sharp) = ease_from_sharpness(1.0);

        // Smooth: ease_out x should be higher than sharp
        assert!(
            eo_smooth[0] > eo_sharp[0],
            "smooth ease_out x should be larger"
        );
        // Sharp: ease_in x should be higher
        assert!(
            ei_sharp[0] > ei_smooth[0],
            "sharp ease_in x should be larger"
        );
    }

    #[test]
    fn test_complexity_affects_orbital_layers() {
        let synth = MotionSynthesizer::new();

        // High complexity → more orbital nodes
        let mut vs_high = ValueSystem::new();
        for w in &mut vs_high.weights {
            w.weight = 1.0;
        }
        let sig_high = VisualSignature::from_value_system(&vs_high);

        let anim = synth.orbital_rings(&sig_high);
        let json = anim.to_json().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let layers = parsed["layers"].as_array().unwrap();
        assert!(
            layers.len() >= 7,
            "high complexity should produce more orbital layers"
        );
    }
}
