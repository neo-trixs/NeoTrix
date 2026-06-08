use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaOrigin {
    Self_(VsaSelfCategory),
    World(VsaWorldCategory),
}

impl VsaOrigin {
    pub fn is_self(&self) -> bool {
        matches!(self, VsaOrigin::Self_(_))
    }

    pub fn is_world(&self) -> bool {
        matches!(self, VsaOrigin::World(_))
    }

    pub fn category_name(&self) -> &'static str {
        match self {
            VsaOrigin::Self_(c) => c.name(),
            VsaOrigin::World(c) => c.name(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaSelfCategory {
    Thought,
    Memory,
    Plan,
    Intention,
    Emotion,
    Imagination,
    MetaCognition,
}

impl VsaSelfCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaSelfCategory::Thought => "thought",
            VsaSelfCategory::Memory => "memory",
            VsaSelfCategory::Plan => "plan",
            VsaSelfCategory::Intention => "intention",
            VsaSelfCategory::Emotion => "emotion",
            VsaSelfCategory::Imagination => "imagination",
            VsaSelfCategory::MetaCognition => "metacognition",
        }
    }

    pub fn all() -> &'static [VsaSelfCategory] {
        &[
            VsaSelfCategory::Thought,
            VsaSelfCategory::Memory,
            VsaSelfCategory::Plan,
            VsaSelfCategory::Intention,
            VsaSelfCategory::Emotion,
            VsaSelfCategory::Imagination,
            VsaSelfCategory::MetaCognition,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VsaWorldCategory {
    UserInput,
    Sensor,
    WebContent,
    ToolOutput,
    CodeAnalysis,
    FileContent,
    SystemEvent,
}

impl VsaWorldCategory {
    pub fn name(&self) -> &'static str {
        match self {
            VsaWorldCategory::UserInput => "user_input",
            VsaWorldCategory::Sensor => "sensor",
            VsaWorldCategory::WebContent => "web_content",
            VsaWorldCategory::ToolOutput => "tool_output",
            VsaWorldCategory::CodeAnalysis => "code_analysis",
            VsaWorldCategory::FileContent => "file_content",
            VsaWorldCategory::SystemEvent => "system_event",
        }
    }

    pub fn all() -> &'static [VsaWorldCategory] {
        &[
            VsaWorldCategory::UserInput,
            VsaWorldCategory::Sensor,
            VsaWorldCategory::WebContent,
            VsaWorldCategory::ToolOutput,
            VsaWorldCategory::CodeAnalysis,
            VsaWorldCategory::FileContent,
            VsaWorldCategory::SystemEvent,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VsaTagged {
    pub vector: Vec<u8>,
    pub tag: VsaOrigin,
    pub confidence: f64,
    pub timestamp: std::time::Instant,
}

impl VsaTagged {
    pub fn new(vector: Vec<u8>, tag: VsaOrigin) -> Self {
        Self {
            vector,
            tag,
            confidence: 1.0,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn is_self(&self) -> bool {
        self.tag.is_self()
    }

    pub fn is_world(&self) -> bool {
        self.tag.is_world()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsa_origin_self() {
        let tag = VsaOrigin::Self_(VsaSelfCategory::Thought);
        assert!(tag.is_self());
        assert!(!tag.is_world());
    }

    #[test]
    fn test_vsa_origin_world() {
        let tag = VsaOrigin::World(VsaWorldCategory::UserInput);
        assert!(!tag.is_self());
        assert!(tag.is_world());
    }

    #[test]
    fn test_vsa_tagged_roundtrip() {
        let vector = vec![1; 256];
        let tag = VsaOrigin::Self_(VsaSelfCategory::Memory);
        let tagged = VsaTagged::new(vector.clone(), tag);
        assert_eq!(tagged.vector, vector);
        assert_eq!(tagged.tag, tag);
        assert!(tagged.is_self());
    }

    #[test]
    fn test_self_categories_distinct() {
        let cats = VsaSelfCategory::all();
        let mut unique = cats.to_vec();
        unique.sort_by_key(|c| *c as u8);
        unique.dedup();
        assert_eq!(cats.len(), unique.len());
    }

    #[test]
    fn test_world_categories_distinct() {
        let cats = VsaWorldCategory::all();
        let mut unique = cats.to_vec();
        unique.sort_by_key(|c| *c as u8);
        unique.dedup();
        assert_eq!(cats.len(), unique.len());
    }

    #[test]
    fn test_confidence_clamping() {
        let vector = vec![0; 10];
        let tag = VsaOrigin::Self_(VsaSelfCategory::Emotion);
        let tagged = VsaTagged::new(vector, tag).with_confidence(1.5);
        assert!((tagged.confidence - 1.0).abs() < 1e-9);
    }
}
