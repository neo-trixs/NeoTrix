use std::collections::HashMap;

/// A concept-to-scene mapping — like xiaohu-ip-studio's visual metaphors
#[derive(Debug, Clone)]
pub struct VisualMetaphor {
    pub id: String,
    pub concept: String,
    pub scene_type: SceneType,
    pub scene_prompt: String,
    pub emotion_tone: String,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SceneType {
    Nature,
    Urban,
    Abstract,
    CharacterAction,
    Symbolic,
    Fantasy,
    Everyday,
}

/// Metaphor engine — maps abstract concepts to vivid visual scenes
#[derive(Debug, Clone)]
pub struct MetaphorEngine {
    metaphors: Vec<VisualMetaphor>,
    templates: HashMap<SceneType, Vec<String>>,
}

impl MetaphorEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert(
            SceneType::Nature,
            vec![
                "A {concept} scene set in a vast {setting}, with {detail}.".to_string(),
                "Nature illustrates {concept}: {detail} in a {setting}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::Urban,
            vec![
                "In the heart of the city, {concept} emerges through {detail}.".to_string(),
                "Urban {setting} showing {concept} through {detail}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::Abstract,
            vec![
                "Abstract visualization of {concept}: {detail} floating in {setting}.".to_string(),
                "Geometric forms express {concept}: {detail}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::CharacterAction,
            vec![
                "A character demonstrates {concept} by {detail} in a {setting}.".to_string(),
                "{concept} in action: {detail} amidst {setting}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::Symbolic,
            vec![
                "Symbolic representation of {concept}: {detail} against {setting}.".to_string(),
                "A {setting} symbolizing {concept}: {detail}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::Fantasy,
            vec![
                "In a fantasy realm, {concept} manifests as {detail} in {setting}.".to_string(),
                "Magical {setting} where {concept} takes the form of {detail}.".to_string(),
            ],
        );
        templates.insert(
            SceneType::Everyday,
            vec![
                "In everyday life, {concept} appears in {setting} through {detail}.".to_string(),
                "A simple {setting} moment reveals {concept}: {detail}.".to_string(),
            ],
        );

        Self {
            metaphors: Vec::new(),
            templates,
        }
    }

    /// Find or synthesize a visual metaphor for a given concept
    pub fn metaphor_for(&self, concept: &str) -> Result<VisualMetaphor, String> {
        let concept_lower = concept.to_lowercase();
        for m in &self.metaphors {
            if m.concept.to_lowercase() == concept_lower {
                return Ok(m.clone());
            }
        }
        Err(format!("No metaphor found for concept '{}'", concept))
    }

    /// Synthesize a new metaphor on the fly using templates
    pub fn synthesize(&self, concept: &str, scene: SceneType, tone: &str) -> VisualMetaphor {
        let id = format!(
            "synth_{}_{}",
            concept.replace(' ', "_"),
            scene.clone() as u8
        );
        let templates = self.templates.get(&scene).cloned().unwrap_or_default();

        let scene_prompt = if templates.is_empty() {
            format!("A scene about {} with {} tone.", concept, tone)
        } else {
            let template = &templates[0];
            template
                .replace("{concept}", concept)
                .replace("{setting}", "appropriate setting")
                .replace("{detail}", "carefully composed details")
        };

        VisualMetaphor {
            id,
            concept: concept.to_string(),
            scene_type: scene,
            scene_prompt,
            emotion_tone: tone.to_string(),
        }
    }

    /// Seed with 10+ default metaphors connecting abstract concepts to scenes
    pub fn with_defaults() -> Self {
        let mut engine = Self::new();

        engine.metaphors.push(VisualMetaphor {
            id: "resilience".to_string(),
            concept: "resilience".to_string(),
            scene_type: SceneType::Nature,
            scene_prompt: "A storm-battered tree standing tall at dawn, broken branches revealing new green shoots reaching for the light.".to_string(),
            emotion_tone: "hopeful, enduring".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "complexity".to_string(),
            concept: "complexity".to_string(),
            scene_type: SceneType::Urban,
            scene_prompt: "Interlocking gears of a clock tower, each gear different in size and speed, together creating a perfectly synchronized mechanism.".to_string(),
            emotion_tone: "intricate, ordered".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "growth".to_string(),
            concept: "growth".to_string(),
            scene_type: SceneType::Nature,
            scene_prompt: "A vibrant green plant determinedly breaking through a crack in grey concrete, sunlight streaming down on its first leaves.".to_string(),
            emotion_tone: "triumphant, organic".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "collaboration".to_string(),
            concept: "collaboration".to_string(),
            scene_type: SceneType::CharacterAction,
            scene_prompt: "Many hands of different skin tones reaching together to build an arching bridge over a flowing river.".to_string(),
            emotion_tone: "harmonious, constructive".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "innovation".to_string(),
            concept: "innovation".to_string(),
            scene_type: SceneType::Abstract,
            scene_prompt: "A single bright lightbulb connected by threads of light to surrounding smaller bulbs, each one illuminating a different fragment of a puzzle.".to_string(),
            emotion_tone: "bright, connected".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "freedom".to_string(),
            concept: "freedom".to_string(),
            scene_type: SceneType::Nature,
            scene_prompt: "A flock of birds lifting off from an open field at golden hour, wings catching the warm light as they rise toward an endless sky.".to_string(),
            emotion_tone: "liberating, expansive".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "trust".to_string(),
            concept: "trust".to_string(),
            scene_type: SceneType::Everyday,
            scene_prompt: "Two figures sitting back-to-back under a large ancient tree, each watching a different horizon, comfortable in shared silence.".to_string(),
            emotion_tone: "quiet, secure".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "transformation".to_string(),
            concept: "transformation".to_string(),
            scene_type: SceneType::Fantasy,
            scene_prompt: "A caterpillar dissolving into a cocoon of golden light, with the translucent silhouette of butterfly wings already forming within.".to_string(),
            emotion_tone: "magical, chrysalis".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "balance".to_string(),
            concept: "balance".to_string(),
            scene_type: SceneType::Symbolic,
            scene_prompt: "A scale with fire on one side and water on the other, perfectly balanced at the center under a full moon.".to_string(),
            emotion_tone: "serene, equilibrium".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "memory".to_string(),
            concept: "memory".to_string(),
            scene_type: SceneType::Abstract,
            scene_prompt: "Floating fragments of photographs fading from vivid color at the center to translucent sepia at the edges, drifting in a dark space.".to_string(),
            emotion_tone: "nostalgic, gentle".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "courage".to_string(),
            concept: "courage".to_string(),
            scene_type: SceneType::CharacterAction,
            scene_prompt: "A small figure standing at the edge of a vast dark forest, holding a single bright lantern forward, face lit with quiet determination.".to_string(),
            emotion_tone: "brave, resolute".to_string(),
        });

        engine.metaphors.push(VisualMetaphor {
            id: "connection".to_string(),
            concept: "connection".to_string(),
            scene_type: SceneType::Symbolic,
            scene_prompt: "Two separate islands connected by a luminous bridge of light spanning beneath a starry night sky.".to_string(),
            emotion_tone: "warm, bridging".to_string(),
        });

        engine
    }

    pub fn len(&self) -> usize {
        self.metaphors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metaphors.is_empty()
    }
}

impl Default for MetaphorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metaphors_count() {
        let engine = MetaphorEngine::with_defaults();
        assert_eq!(engine.len(), 12);
    }

    #[test]
    fn test_metaphor_for_found() {
        let engine = MetaphorEngine::with_defaults();
        let m = engine.metaphor_for("resilience").unwrap();
        assert_eq!(m.concept, "resilience");
        assert!(m.scene_prompt.contains("storm-battered"));
    }

    #[test]
    fn test_metaphor_for_missing() {
        let engine = MetaphorEngine::new();
        let result = engine.metaphor_for("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_synthesize() {
        let engine = MetaphorEngine::new();
        let m = engine.synthesize("curiosity", SceneType::Nature, "wonder");
        assert_eq!(m.concept, "curiosity");
        assert_eq!(m.emotion_tone, "wonder");
        assert_eq!(m.scene_type, SceneType::Nature);
    }

    #[test]
    fn test_metaphor_scene_types() {
        let engine = MetaphorEngine::with_defaults();
        let types: std::collections::HashSet<SceneType> = engine
            .metaphors
            .iter()
            .map(|m| m.scene_type.clone())
            .collect();
        assert!(types.contains(&SceneType::Nature));
        assert!(types.contains(&SceneType::Urban));
        assert!(types.contains(&SceneType::Abstract));
        assert!(types.contains(&SceneType::CharacterAction));
        assert!(types.contains(&SceneType::Symbolic));
        assert!(types.contains(&SceneType::Fantasy));
        assert!(types.contains(&SceneType::Everyday));
    }

    #[test]
    fn test_templates_all_scene_types() {
        let engine = MetaphorEngine::new();
        assert!(engine.templates.contains_key(&SceneType::Nature));
        assert!(engine.templates.contains_key(&SceneType::Urban));
        assert!(engine.templates.contains_key(&SceneType::Abstract));
        assert!(engine.templates.contains_key(&SceneType::CharacterAction));
        assert!(engine.templates.contains_key(&SceneType::Symbolic));
        assert!(engine.templates.contains_key(&SceneType::Fantasy));
        assert!(engine.templates.contains_key(&SceneType::Everyday));
    }

    #[test]
    fn test_synthesize_different_scenes() {
        let engine = MetaphorEngine::new();
        let m1 = engine.synthesize("hope", SceneType::Fantasy, "bright");
        let m2 = engine.synthesize("hope", SceneType::Urban, "bright");
        assert_ne!(m1.scene_type, m2.scene_type);
    }

    #[test]
    fn test_empty_engine() {
        let engine = MetaphorEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.len(), 0);
    }

    #[test]
    fn test_metaphor_case_insensitive() {
        let engine = MetaphorEngine::with_defaults();
        let m = engine.metaphor_for("RESILIENCE").unwrap();
        assert_eq!(m.concept, "resilience");
    }
}
