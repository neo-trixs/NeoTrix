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
