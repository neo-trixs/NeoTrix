use crate::game::dialogue::DialogueTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcType { Awareness, Focus, Creativity, Empathy }

#[derive(Debug, Clone)]
pub struct NpcDefinition {
    pub npc_type: NpcType,
    pub name: String,
    pub color: String,
    pub position: (u32, u32),
    pub schedule: Vec<NpcScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct NpcScheduleEntry {
    pub hour: u32,
    pub location: String,
    pub activity: String,
}

impl NpcDefinition {
    pub fn all_npcs() -> Vec<Self> {
        vec![
            Self {
                npc_type: NpcType::Awareness, name: "Awareness".to_string(), color: "#4FC3F7".to_string(),
                position: (55, 32),
                schedule: vec![
                    NpcScheduleEntry { hour: 6, location: "Meadow".to_string(), activity: "Meditating".to_string() },
                    NpcScheduleEntry { hour: 10, location: "Hub".to_string(), activity: "Teaching".to_string() },
                    NpcScheduleEntry { hour: 18, location: "Lake".to_string(), activity: "Reflecting".to_string() },
                ],
            },
            Self {
                npc_type: NpcType::Focus, name: "Focus".to_string(), color: "#EF5350".to_string(),
                position: (58, 30),
                schedule: vec![
                    NpcScheduleEntry { hour: 6, location: "Mines".to_string(), activity: "Training".to_string() },
                    NpcScheduleEntry { hour: 12, location: "Hub".to_string(), activity: "Planning".to_string() },
                    NpcScheduleEntry { hour: 20, location: "Forest".to_string(), activity: "Guarding".to_string() },
                ],
            },
            Self {
                npc_type: NpcType::Creativity, name: "Creativity".to_string(), color: "#FFD54F".to_string(),
                position: (52, 35),
                schedule: vec![
                    NpcScheduleEntry { hour: 8, location: "Forest".to_string(), activity: "Gathering".to_string() },
                    NpcScheduleEntry { hour: 14, location: "Hub".to_string(), activity: "Crafting".to_string() },
                    NpcScheduleEntry { hour: 20, location: "Meadow".to_string(), activity: "Stargazing".to_string() },
                ],
            },
            Self {
                npc_type: NpcType::Empathy, name: "Empathy".to_string(), color: "#66BB6A".to_string(),
                position: (60, 33),
                schedule: vec![
                    NpcScheduleEntry { hour: 7, location: "Lake".to_string(), activity: "Fishing".to_string() },
                    NpcScheduleEntry { hour: 12, location: "Hub".to_string(), activity: "Healing".to_string() },
                    NpcScheduleEntry { hour: 18, location: "Meadow".to_string(), activity: "Singing".to_string() },
                ],
            },
        ]
    }
}

pub fn awareness_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Awareness", "The meadow remembers what the mind forgets. What brings you here?");
    tree.add_response(i0, "Teach me about awareness.", Some(1));
    tree.add_response(i0, "I need help with my farm.", Some(2));
    tree.add_line("Awareness", "Awareness is the seed of all growth. Tend to it as you tend your crops.");
    tree.add_line("Awareness", "Your farm reflects your inner state. Water your crops, water your mind.");
    tree
}

pub fn focus_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Focus", "Sharp mind cuts through confusion like an axe through wood.");
    tree.add_response(i0, "How do I sharpen my focus?", Some(1));
    tree.add_response(i0, "Tell me about the mines.", Some(2));
    tree.add_line("Focus", "One task at a time. One breath at a time. That is the way.");
    tree.add_line("Focus", "The Knowledge Mines hold fragments of forgotten truths. Be careful in there.");
    tree
}

pub fn creativity_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Creativity", "Every pattern was once chaos. Every masterpiece was once a mistake.");
    tree.add_response(i0, "How do I create?", Some(1));
    tree.add_response(i0, "Can you teach me crafting?", Some(2));
    tree.add_line("Creativity", "Play without purpose. The best ideas come when you stop trying.");
    tree.add_line("Creativity", "Combine fragments in unexpected ways. Thought + Curiosity = Inspiration!");
    tree
}

pub fn empathy_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Empathy", "To understand another, first understand yourself.");
    tree.add_response(i0, "What is empathy?", Some(1));
    tree.add_response(i0, "I feel lost sometimes.", Some(2));
    tree.add_line("Empathy", "It is feeling with, not feeling for. The lake reflects all that stands beside it.");
    tree.add_line("Empathy", "Even in stillness, you are not alone. The valley breathes with you.");
    tree
}
