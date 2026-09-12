use super::dialogue::DialogueTree;
use crate::core::world::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcRole {
    Merchant,
    Guide,
    Companion,
    Mentor,
    Challenge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcType {
    Awareness,
    Focus,
    Creativity,
    Empathy,
    Memory,
    Logic,
    Wisdom,
    Dreams,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub name: String,
    pub role: NpcRole,
    pub resonance: u32,
    pub max_resonance: u32,
    pub gifts_this_week: u32,
    pub liked_items: Vec<u32>,
    pub disliked_items: Vec<u32>,
    pub loved_items: Vec<u32>,
    pub schedule: Vec<NpcScheduleEntry>,
}

#[derive(Debug, Clone)]
pub struct NpcScheduleEntry {
    pub hour: u32,
    pub location: String,
    pub activity: String,
    pub dialogue: Option<DialogueTree>,
}

#[derive(Debug, Clone)]
pub struct NpcDefinition {
    pub npc_type: NpcType,
    pub name: String,
    pub color: String,
    pub position: (u32, u32),
    pub schedule: Vec<NpcScheduleEntry>,
}

impl NpcDefinition {
    pub fn all_npcs() -> Vec<Self> {
        vec![
            Self {
                npc_type: NpcType::Awareness,
                name: "Awareness".to_string(),
                color: "#4FC3F7".to_string(),
                position: (55, 32),
                schedule: vec![
                    NpcScheduleEntry { hour: 6, location: "Meadow".to_string(), activity: "Meditating".to_string(), dialogue: Some(awareness_dialogue()) },
                    NpcScheduleEntry { hour: 10, location: "Hub".to_string(), activity: "Teaching".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 18, location: "Lake".to_string(), activity: "Reflecting".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Focus,
                name: "Focus".to_string(),
                color: "#EF5350".to_string(),
                position: (58, 30),
                schedule: vec![
                    NpcScheduleEntry { hour: 6, location: "Mines".to_string(), activity: "Training".to_string(), dialogue: Some(focus_dialogue()) },
                    NpcScheduleEntry { hour: 12, location: "Hub".to_string(), activity: "Planning".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 20, location: "Forest".to_string(), activity: "Guarding".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Creativity,
                name: "Creativity".to_string(),
                color: "#FFD54F".to_string(),
                position: (52, 35),
                schedule: vec![
                    NpcScheduleEntry { hour: 8, location: "Forest".to_string(), activity: "Gathering".to_string(), dialogue: Some(creativity_dialogue()) },
                    NpcScheduleEntry { hour: 14, location: "Hub".to_string(), activity: "Crafting".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 20, location: "Meadow".to_string(), activity: "Stargazing".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Empathy,
                name: "Empathy".to_string(),
                color: "#66BB6A".to_string(),
                position: (60, 33),
                schedule: vec![
                    NpcScheduleEntry { hour: 7, location: "Lake".to_string(), activity: "Fishing".to_string(), dialogue: Some(empathy_dialogue()) },
                    NpcScheduleEntry { hour: 12, location: "Hub".to_string(), activity: "Healing".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 18, location: "Meadow".to_string(), activity: "Singing".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Memory,
                name: "Memory".to_string(),
                color: "#AB47BC".to_string(),
                position: (48, 28),
                schedule: vec![
                    NpcScheduleEntry { hour: 7, location: "Forest".to_string(), activity: "Collecting".to_string(), dialogue: Some(memory_dialogue()) },
                    NpcScheduleEntry { hour: 13, location: "Hub".to_string(), activity: "Archiving".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 19, location: "Lake".to_string(), activity: "Reflecting".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Logic,
                name: "Logic".to_string(),
                color: "#5C6BC0".to_string(),
                position: (56, 28),
                schedule: vec![
                    NpcScheduleEntry { hour: 6, location: "Mines".to_string(), activity: "Analyzing".to_string(), dialogue: Some(logic_dialogue()) },
                    NpcScheduleEntry { hour: 14, location: "Hub".to_string(), activity: "Teaching".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 20, location: "Farm".to_string(), activity: "Planning".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Wisdom,
                name: "Wisdom".to_string(),
                color: "#FFA726".to_string(),
                position: (54, 26),
                schedule: vec![
                    NpcScheduleEntry { hour: 8, location: "Lake".to_string(), activity: "Meditating".to_string(), dialogue: Some(wisdom_dialogue()) },
                    NpcScheduleEntry { hour: 15, location: "Hub".to_string(), activity: "Counseling".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 21, location: "Forest".to_string(), activity: "Stargazing".to_string(), dialogue: None },
                ],
            },
            Self {
                npc_type: NpcType::Dreams,
                name: "Dreams".to_string(),
                color: "#EC407A".to_string(),
                position: (62, 36),
                schedule: vec![
                    NpcScheduleEntry { hour: 10, location: "Lake".to_string(), activity: "Dreaming".to_string(), dialogue: Some(dreams_dialogue()) },
                    NpcScheduleEntry { hour: 16, location: "Hub".to_string(), activity: "Sharing".to_string(), dialogue: None },
                    NpcScheduleEntry { hour: 22, location: "Farm".to_string(), activity: "Nightwatching".to_string(), dialogue: None },
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

pub fn memory_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Memory", "Every leaf in this forest holds a story. Shall I share one?");
    tree.add_response(i0, "Tell me about the forest's history.", Some(1));
    tree.add_response(i0, "How do I improve my memory?", Some(2));
    tree.add_line("Memory", "The oldest trees remember when this valley was young. Their roots are libraries of bark and moss.");
    tree.add_line("Memory", "Practice recall each evening. Write three things you learned before sleep. The mind is a garden — tend it daily.");
    tree
}

pub fn logic_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Logic", "Every problem is a puzzle. Every puzzle has pieces. Find the edges first.");
    tree.add_response(i0, "Teach me to think clearly.", Some(1));
    tree.add_response(i0, "What are you mining for?", Some(2));
    tree.add_line("Logic", "Start with what you know. Build from certainty to uncertainty. Never assume the shape before examining all pieces.");
    tree.add_line("Logic", "Ore of course! But the truest mine is the one between your ears. I extract reasoning from raw thought.");
    tree
}

pub fn wisdom_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Wisdom", "The lake at dusk teaches patience. What wisdom do you seek?");
    tree.add_response(i0, "How do I gain wisdom?", Some(1));
    tree.add_response(i0, "What is the meaning of this valley?", Some(2));
    tree.add_line("Wisdom", "Wisdom is not knowledge accumulated — it is knowledge applied. Fail, reflect, adapt. Repeat.");
    tree.add_line("Wisdom", "This valley exists to teach that growth and peace are not opposites. They are partners in the dance of being.");
    tree
}

pub fn dreams_dialogue() -> DialogueTree {
    let mut tree = DialogueTree::new();
    let i0 = tree.add_line("Dreams", "I wander between waking and sleep, collecting fragments of what might be.");
    tree.add_response(i0, "What do you dream about?", Some(1));
    tree.add_response(i0, "Can dreams help my farm?", Some(2));
    tree.add_line("Dreams", "I dream of futures not yet born. A crop that grows in starlight. A tool shaped by thought alone.");
    tree.add_line("Dreams", "Sleep well each night — a rested mind notices opportunities a tired mind overlooks. Dreams plant seeds of intuition.");
    tree
}

impl Npc {
    pub fn new(name: &str, role: NpcRole) -> Self {
        Self {
            name: name.to_string(),
            role,
            resonance: 0,
            max_resonance: 2500,
            gifts_this_week: 0,
            liked_items: vec![],
            disliked_items: vec![],
            loved_items: vec![],
            schedule: vec![],
        }
    }

    pub fn give_gift(&mut self, item_id: u32, _base_points: u32, quality_mult: f32, is_birthday: bool) -> i32 {
        if self.gifts_this_week >= 2 {
            return 0;
        }

        let preference = if self.loved_items.contains(&item_id) {
            80
        } else if self.liked_items.contains(&item_id) {
            45
        } else if self.disliked_items.contains(&item_id) {
            -20
        } else {
            20
        };

        let mut points = (preference as f32 * quality_mult) as i32;
        if is_birthday {
            points *= 8;
        }

        self.resonance = (self.resonance as i32 + points).max(0) as u32;
        self.gifts_this_week += 1;

        points
    }

    pub fn resonance_hearts(&self) -> u32 {
        self.resonance / 250
    }

    pub fn reset_weekly_gifts(&mut self) {
        self.gifts_this_week = 0;
    }

    pub fn relationship_level(&self) -> &str {
        let hearts = self.resonance_hearts();
        match hearts {
            0 => "Stranger",
            1..=3 => "Acquaintance",
            4..=6 => "Friend",
            7..=9 => "Close Friend",
            _ => "Soul Resonant",
        }
    }
}

impl Component for Npc {}
impl Component for Position {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npc_gift() {
        let mut npc = Npc::new("Sage", NpcRole::Mentor);
        npc.loved_items = vec![101];

        let points = npc.give_gift(101, 80, 1.0, false);
        assert!(points > 0);
        assert_eq!(npc.gifts_this_week, 1);
    }

    #[test]
    fn test_npc_resonance() {
        let mut npc = Npc::new("Guide", NpcRole::Guide);
        npc.resonance = 500;
        assert_eq!(npc.resonance_hearts(), 2);
    }

    #[test]
    fn test_npc_definition_all_npcs() {
        let npcs = NpcDefinition::all_npcs();
        assert_eq!(npcs.len(), 4);
        assert_eq!(npcs[0].name, "Awareness");
        assert_eq!(npcs[1].name, "Focus");
    }

    #[test]
    fn test_dialogue_fns() {
        let tree = awareness_dialogue();
        assert!(!tree.lines.is_empty());
        assert_eq!(tree.lines[0].speaker, "Awareness");
    }
}
