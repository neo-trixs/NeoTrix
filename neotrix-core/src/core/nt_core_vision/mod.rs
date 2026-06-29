// Vision Module — ML-based image generation, IP characters, style DNA, visual metaphors
pub mod image_generator;
pub mod ip_character;
pub mod style_dna;
pub mod visual_metaphor;

pub use image_generator::{
    CharacterRef, ComfyUIGenerator, DallE3Generator, GenParams, GenQuality, GeneratedImage,
    ImageGenerator, MockImageGenerator,
};
pub use ip_character::{
    AgeGroup, Archetype, CharacterAppearance, CharacterRegistry, HairStyle, IpCharacter, OutfitVibe,
};
pub use style_dna::{
    ColorGradingProfile, LightingMood, LightingProfile, PostProcessProfile, RenderingProfile,
    ShadingStyle, StyleDna, StyleDnaRegistry,
};
pub use visual_metaphor::{MetaphorEngine, SceneType, VisualMetaphor};

// Re-export image understanding pipeline from nt_world_vision
pub use crate::neotrix::nt_world_vision::ImagePipeline;

#[cfg(test)]
mod tests {}
