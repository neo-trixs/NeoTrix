use crate::core::nt_core_bank::ReasoningMemory;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct L1Memory {
    pub id: String,
    pub content: String,
    pub mem_type: String,
    pub priority: f64,
    pub source_memory_id: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneBlock {
    pub id: String,
    pub summary: String,
    pub content: Vec<String>,
    pub heat: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub memory_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Persona {
    pub base_anchors: Vec<String>,
    pub interest_map: Vec<String>,
    pub interaction_protocol: Vec<String>,
    pub cognitive_kernel: Vec<String>,
}

impl Default for Persona {
    fn default() -> Self {
        Self::new()
    }
}

impl Persona {
    pub fn new() -> Self {
        Self { base_anchors: Vec::new(), interest_map: Vec::new(), interaction_protocol: Vec::new(), cognitive_kernel: Vec::new() }
    }
}

pub struct ExtractionPrompt {
    pub layer: u8,
    pub input_memories: Vec<ReasoningMemory>,
    pub existing_persona: Option<String>,
}

impl ExtractionPrompt {
    pub fn l1_prompt(memories: &[ReasoningMemory]) -> String {
        let mem_str: Vec<String> = memories.iter().map(|m|
            format!("[{}] {} (reward={:.2}, type={:?})", m.id, m.task_description, m.reward, m.task_type)
        ).collect();
        format!(
            r#"Extract structured memories from the following reasoning traces.
For each trace, identify:
- persona: stable attributes about the user/system
- episodic: objective events and their outcomes
- instruction: global directives or preferences

Traces:
{}

Output JSON array: [{{"content":"...","mem_type":"persona|episodic|instruction","priority":0.0}}]"#,
            mem_str.join("\n---\n")
        )
    }

    pub fn l2_prompt(l1_memories: &[L1Memory]) -> String {
        let mem_str: Vec<String> = l1_memories.iter().map(|m|
            format!("[{}] ({}) {} (priority={:.2})", m.id, m.mem_type, m.content, m.priority)
        ).collect();
        format!(
            r#"Group the following atomic memories into coherent scenes.
Each scene represents a coherent topic or task session.

Memories:
{}

Output JSON array: [{{"summary":"...","memory_ids":["..."],"content":["..."],"heat":1}}]"#,
            mem_str.join("\n")
        )
    }

    pub fn l3_prompt(scenes: &[SceneBlock], existing_persona: &Option<String>) -> String {
        let scene_str: Vec<String> = scenes.iter().map(|s| format!("Scene {}: {}", s.id, s.summary)).collect();
        let existing = existing_persona.as_ref().map(|p| format!("\nExisting persona:\n{}", p)).unwrap_or_default();
        format!(
            r#"Based on the following scene summaries, generate a user persona.
The persona has 4 layers:
1. base_anchors: factual attributes
2. interest_map: topics of attention
3. interaction_protocol: communication patterns
4. cognitive_kernel: decision-making logic

Scenes:
{}{}

Output JSON: {{"base_anchors":[],"interest_map":[],"interaction_protocol":[],"cognitive_kernel":[]}}"#,
            scene_str.join("\n"), existing
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_new_is_empty() {
        let p = Persona::new();
        assert!(p.base_anchors.is_empty());
        assert!(p.interest_map.is_empty());
    }

    #[test]
    fn test_persona_default() {
        let p = Persona::default();
        assert!(p.cognitive_kernel.is_empty());
    }

    #[test]
    fn test_scene_block_creation() {
        let s = SceneBlock {
            id: "scene-1".into(),
            summary: "test scene".into(),
            content: vec!["line1".into()],
            heat: 5,
            created_at: 1000,
            updated_at: 1000,
            memory_ids: vec!["mem-1".into()],
        };
        assert_eq!(s.id, "scene-1");
        assert_eq!(s.heat, 5);
    }

    #[test]
    fn test_l1_prompt_contains_memories() {
        use crate::core::{RewardSource, TaskType};
        use crate::core::nt_core_bank::{MemoryTier, MemoryLifecycle, T3Views};
        let mem = ReasoningMemory {
            id: "mem-1".into(),
            task_description: "test task".into(),
            task_type: TaskType::General,
            micro_edits: vec![],
            reward: 0.8,
            reward_source: RewardSource::Internal,
            success: true,
            timestamp: 1000,
            embedding: None,
            tier: MemoryTier::Working,
            lifecycle: MemoryLifecycle::new(0.5),
            t3_views: T3Views::new(),
        };
        let prompt = ExtractionPrompt::l1_prompt(&[mem]);
        assert!(prompt.contains("test task"));
        assert!(prompt.contains("mem-1"));
    }

    #[test]
    fn test_l2_prompt_contains_memories() {
        let l1 = L1Memory {
            id: "l1-1".into(),
            content: "atomic memory".into(),
            mem_type: "episodic".into(),
            priority: 0.7,
            source_memory_id: "mem-1".into(),
            created_at: 1000,
        };
        let prompt = ExtractionPrompt::l2_prompt(&[l1]);
        assert!(prompt.contains("atomic memory"));
    }

    #[test]
    fn test_l3_prompt_contains_scenes() {
        let scene = SceneBlock {
            id: "scene-1".into(),
            summary: "user testing".into(),
            content: vec!["details".into()],
            heat: 3,
            created_at: 1000,
            updated_at: 1000,
            memory_ids: vec![],
        };
        let prompt = ExtractionPrompt::l3_prompt(&[scene], &None);
        assert!(prompt.contains("user testing"));
    }

    #[test]
    fn test_l3_prompt_with_existing_persona() {
        let prompt = ExtractionPrompt::l3_prompt(&[], &Some("existing data".into()));
        assert!(prompt.contains("existing data"));
    }

    #[test]
    fn test_l1_memory_creation() {
        let m = L1Memory {
            id: "test".into(),
            content: "test content".into(),
            mem_type: "persona".into(),
            priority: 0.5,
            source_memory_id: "src-1".into(),
            created_at: 1000,
        };
        assert_eq!(m.priority, 0.5);
    }
}
