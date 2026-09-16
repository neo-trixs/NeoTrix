use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
    pub conditions: HashMap<String, String>,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    pub next_node: Option<u32>,
    pub conditions: HashMap<String, String>,
    pub effects: Vec<DialogueEffect>,
}

#[derive(Debug, Clone)]
pub enum DialogueEffect {
    SetFlag(String, String),
    GiveItem(String),
    RemoveItem(String),
    ModifyRelationship(u64, f64),
    ModifyHealth(f64),
}

pub struct DialogueTree {
    nodes: HashMap<u32, DialogueNode>,
    current_node: Option<u32>,
    flags: HashMap<String, String>,
}

impl DialogueTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            current_node: None,
            flags: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.insert(node.id, node);
    }

    pub fn start(&mut self, start_id: u32) {
        self.current_node = Some(start_id);
    }

    pub fn current(&self) -> Option<&DialogueNode> {
        self.current_node.and_then(|id| self.nodes.get(&id))
    }

    pub fn respond(&mut self, response_index: usize) -> Option<&DialogueNode> {
        let (effects, next_node) = {
            let current = self.current()?;
            let response = current.responses.get(response_index)?;
            (response.effects.clone(), response.next_node)
        };

        for effect in &effects {
            self.apply_effect(effect);
        }

        if let Some(next_id) = next_node {
            self.current_node = Some(next_id);
            self.current()
        } else {
            self.current_node = None;
            None
        }
    }

    fn apply_effect(&mut self, effect: &DialogueEffect) {
        match effect {
            DialogueEffect::SetFlag(key, value) => {
                self.flags.insert(key.clone(), value.clone());
            }
            _ => {}
        }
    }

    pub fn get_flag(&self, key: &str) -> Option<&String> {
        self.flags.get(key)
    }

    pub fn set_flag(&mut self, key: &str, value: &str) {
        self.flags.insert(key.to_string(), value.to_string());
    }

    pub fn is_active(&self) -> bool {
        self.current_node.is_some()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for DialogueTree {
    fn default() -> Self {
        Self::new()
    }
}
