use std::collections::HashMap;

// ---------------------------------------------------------------------------
// DialogueNode
// ---------------------------------------------------------------------------

/// A single node in a dialogue tree.
#[derive(Debug, Clone)]
pub struct DialogueNode {
    /// Unique node ID within this tree.
    pub id: String,
    /// Speaker name (displayed in UI).
    pub speaker: String,
    /// Dialogue text (may contain `{variable}` placeholders).
    pub text: String,
    /// Available choices from this node (empty = auto-advance).
    pub choices: Vec<DialogueChoice>,
    /// If no choices, auto-advance to this node after a delay.
    pub next_node: Option<String>,
    /// Optional events to fire when entering this node.
    pub on_enter: Vec<String>,
    /// Optional events to fire when leaving this node.
    pub on_exit: Vec<String>,
    /// Tags for filtering (e.g. "greeting", "shop", "quest").
    pub tags: Vec<String>,
}

impl DialogueNode {
    pub fn new(id: &str, speaker: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            speaker: speaker.to_string(),
            text: text.to_string(),
            choices: Vec::new(),
            next_node: None,
            on_enter: Vec::new(),
            on_exit: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_choice(mut self, choice: DialogueChoice) -> Self {
        self.choices.push(choice);
        self
    }

    pub fn with_next(mut self, next: &str) -> Self {
        self.next_node = Some(next.to_string());
        self
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    pub fn with_on_enter(mut self, event: &str) -> Self {
        self.on_enter.push(event.to_string());
        self
    }
}

// ---------------------------------------------------------------------------
// DialogueChoice
// ---------------------------------------------------------------------------

/// A choice presented to the player.
#[derive(Debug, Clone)]
pub struct DialogueChoice {
    /// Display text (may contain `{variable}` placeholders).
    pub text: String,
    /// Node to advance to when selected.
    pub next_node: String,
    /// Conditions that must be met for this choice to appear.
    pub conditions: Vec<DialogueCondition>,
    /// Whether this choice should end the dialogue.
    pub ends_dialogue: bool,
    /// Optional: hide this choice entirely if conditions fail (vs greyed out).
    pub hide_if_unavailable: bool,
    /// Events to fire when this choice is selected.
    pub on_select: Vec<String>,
}

impl DialogueChoice {
    pub fn new(text: &str, next_node: &str) -> Self {
        Self {
            text: text.to_string(),
            next_node: next_node.to_string(),
            conditions: Vec::new(),
            ends_dialogue: false,
            hide_if_unavailable: false,
            on_select: Vec::new(),
        }
    }

    pub fn with_condition(mut self, cond: DialogueCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn ends_dialogue(mut self) -> Self {
        self.ends_dialogue = true;
        self
    }

    pub fn hide_if_unavailable(mut self) -> Self {
        self.hide_if_unavailable = true;
        self
    }
}

// ---------------------------------------------------------------------------
// DialogueCondition
// ---------------------------------------------------------------------------

/// A condition that gates a dialogue choice.
#[derive(Debug, Clone)]
pub enum DialogueCondition {
    /// Player has a specific item.
    HasItem(String),
    /// Player does NOT have an item.
    NotItem(String),
    /// Quest is in a specific state.
    QuestState { quest_id: String, active: bool },
    /// A boolean flag is set.
    Flag(String),
    /// A flag is NOT set.
    NotFlag(String),
    /// Reputation check: faction >= value.
    Reputation { faction: String, min_value: i32 },
    /// Level check: player level >= value.
    MinLevel(u32),
    /// Gold check: player gold >= amount.
    MinGold(u64),
    /// AND of multiple conditions.
    All(Vec<DialogueCondition>),
    /// OR of multiple conditions.
    Any(Vec<DialogueCondition>),
}

// ---------------------------------------------------------------------------
// DialogueTree
// ---------------------------------------------------------------------------

/// A complete dialogue tree with nodes and a starting point.
#[derive(Debug, Clone)]
pub struct DialogueTree {
    /// All nodes indexed by ID.
    pub nodes: HashMap<String, DialogueNode>,
    /// The starting node ID.
    pub start_node: String,
    /// Tree-level metadata (quest_id, NPC id, etc).
    pub metadata: HashMap<String, String>,
}

impl DialogueTree {
    pub fn new(start_node: &str) -> Self {
        Self {
            nodes: HashMap::new(),
            start_node: start_node.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.nodes.get(id)
    }

    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut DialogueNode> {
        self.nodes.get_mut(id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

// ---------------------------------------------------------------------------
// DialogueRunner
// ---------------------------------------------------------------------------

/// Runtime state for an active dialogue.
pub struct DialogueRunner {
    /// The tree being executed.
    pub tree: DialogueTree,
    /// Current node ID (None = dialogue ended).
    current_node: Option<String>,
    /// Player-accessible variables for `{variable}` substitution.
    variables: HashMap<String, String>,
    /// History of visited node IDs.
    history: Vec<String>,
    /// Callbacks for events fired by nodes.
    event_log: Vec<String>,
}

impl DialogueRunner {
    pub fn new(tree: DialogueTree) -> Self {
        let start = tree.start_node.clone();
        Self {
            tree,
            current_node: Some(start),
            variables: HashMap::new(),
            history: Vec::new(),
            event_log: Vec::new(),
        }
    }

    /// Start (or restart) the dialogue.
    pub fn start(&mut self) {
        self.current_node = Some(self.tree.start_node.clone());
        self.history.clear();
    }

    /// Get the current node (with variables substituted).
    pub fn current(&self) -> Option<DialogueNodeDisplay> {
        let node = self.tree.get_node(self.current_node.as_deref()?)?;
        let text = self.substitute(&node.text);
        let speaker = node.speaker.clone();
        let next_node = node.next_node.clone();

        let choices: Vec<ChoiceDisplay> = node.choices.iter().map(|c| {
            let available = c.conditions.iter().all(|cond| self.evaluate_condition(cond));
            ChoiceDisplay {
                text: if available { self.substitute(&c.text) } else { c.text.clone() },
                index: node.choices.iter().position(|cc| cc.text == c.text).unwrap_or(0),
                available,
                ends_dialogue: c.ends_dialogue,
            }
        }).collect();

        Some(DialogueNodeDisplay { speaker, text, choices, next_node })
    }

    /// Select a choice by index.
    pub fn select_choice(&mut self, index: usize) -> bool {
        let node_id = self.current_node.clone().unwrap_or_default();
        let node = match self.tree.get_node(&node_id) {
            Some(n) => n.clone(),
            None => return false,
        };

        if let Some(choice) = node.choices.get(index) {
            if !choice.conditions.iter().all(|c| self.evaluate_condition(c)) {
                return false; // conditions not met
            }

            // Fire events
            for event in &choice.on_select {
                self.event_log.push(event.clone());
            }

            self.history.push(node_id);

            if choice.ends_dialogue {
                self.current_node = None;
                return true;
            }

            self.current_node = Some(choice.next_node.clone());
            true
        } else {
            false
        }
    }

    /// Advance to the next node (for auto-advance nodes with no choices).
    pub fn advance(&mut self) -> bool {
        let node_id = self.current_node.clone().unwrap_or_default();
        let node = match self.tree.get_node(&node_id) {
            Some(n) => n.clone(),
            None => return false,
        };

        if !node.choices.is_empty() {
            return false; // has choices, must use select_choice
        }

        // Fire exit events
        for event in &node.on_exit {
            self.event_log.push(event.clone());
        }

        self.history.push(node_id);

        if let Some(next) = &node.next_node {
            // Fire enter events of next node
            if let Some(next_node) = self.tree.get_node(next) {
                for event in &next_node.on_enter {
                    self.event_log.push(event.clone());
                }
            }
            self.current_node = Some(next.clone());
            true
        } else {
            self.current_node = None;
            false
        }
    }

    /// Is the dialogue still active?
    pub fn is_active(&self) -> bool {
        self.current_node.is_some()
    }

    /// Set a variable for substitution.
    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    /// Get a variable.
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    /// Visit history.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Drain event log.
    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.event_log)
    }

    /// Substitute `{variable}` placeholders in text.
    fn substitute(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Evaluate a condition against current game state.
    fn evaluate_condition(&self, condition: &DialogueCondition) -> bool {
        match condition {
            DialogueCondition::HasItem(_) => true, // Requires external state
            DialogueCondition::NotItem(_) => true,
            DialogueCondition::QuestState { .. } => true,
            DialogueCondition::Flag(flag) => self.variables.get(flag).map_or(false, |v| v == "true"),
            DialogueCondition::NotFlag(flag) => self.variables.get(flag).map_or(true, |v| v != "true"),
            DialogueCondition::Reputation { .. } => true,
            DialogueCondition::MinLevel(_) => true,
            DialogueCondition::MinGold(_) => true,
            DialogueCondition::All(conds) => conds.iter().all(|c| self.evaluate_condition(c)),
            DialogueCondition::Any(conds) => conds.iter().any(|c| self.evaluate_condition(c)),
        }
    }
}

// ---------------------------------------------------------------------------
// Display types (returned to UI)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DialogueNodeDisplay {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<ChoiceDisplay>,
    pub next_node: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChoiceDisplay {
    pub text: String,
    pub index: usize,
    pub available: bool,
    pub ends_dialogue: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_tree() -> DialogueTree {
        let mut tree = DialogueTree::new("start");
        tree.add_node(
            DialogueNode::new("start", "Guard", "Halt! Who goes there?")
                .with_choice(DialogueChoice::new("A traveler.", "greeting"))
                .with_choice(DialogueChoice::new("[Attack]", "combat").ends_dialogue())
        );
        tree.add_node(
            DialogueNode::new("greeting", "Guard", "Welcome, traveler. State your business.")
                .with_next("farewell")
        );
        tree.add_node(
            DialogueNode::new("farewell", "Guard", "Safe travels!")
        );
        tree.add_node(
            DialogueNode::new("combat", "Guard", "En garde!")
        );
        tree
    }

    #[test]
    fn test_dialogue_tree_basics() {
        let tree = build_test_tree();
        assert_eq!(tree.node_count(), 4);
        assert!(tree.get_node("start").is_some());
        assert!(tree.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_runner_advance_no_choices() {
        let tree = build_test_tree();
        let mut runner = DialogueRunner::new(tree);

        // Start node has choices, cannot auto-advance
        assert!(!runner.advance());

        // Select "A traveler." (index 0)
        assert!(runner.select_choice(0));
        assert!(runner.is_active());

        // greeting has no choices, auto-advance
        let current = runner.current().unwrap();
        assert_eq!(current.speaker, "Guard");
        assert!(runner.advance());

        // farewell — no next node, ends
        assert!(runner.advance());
        assert!(!runner.is_active());
    }

    #[test]
    fn test_variable_substitution() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(DialogueNode::new("talk", "NPC", "Hello, {player_name}!"));
        let mut runner = DialogueRunner::new(tree);
        runner.set_variable("player_name", "Hero");
        let display = runner.current().unwrap();
        assert_eq!(display.text, "Hello, Hero!");
    }

    #[test]
    fn test_choice_conditions() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Choose:")
                .with_choice(
                    DialogueChoice::new("Free option", "free")
                )
                .with_choice(
                    DialogueChoice::new("Locked option", "locked")
                        .with_condition(DialogueCondition::Flag("has_key".to_string()))
                )
        );
        let mut runner = DialogueRunner::new(tree);
        let display = runner.current().unwrap();
        assert_eq!(display.choices.len(), 2);
        assert!(display.choices[0].available);
        assert!(!display.choices[1].available); // no flag set
    }

    #[test]
    fn test_flag_condition() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Hello!")
                .with_choice(
                    DialogueChoice::new("Secret", "secret")
                        .with_condition(DialogueCondition::Flag("know_secret".to_string()))
                )
        );
        let mut runner = DialogueRunner::new(tree);

        // Without flag
        let display = runner.current().unwrap();
        assert!(!display.choices[0].available);

        // With flag
        runner.set_variable("know_secret", "true");
        let display = runner.current().unwrap();
        assert!(display.choices[0].available);
    }

    #[test]
    fn test_event_log() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Hello!")
                .with_on_enter("npc_greeted")
                .with_choice(
                    DialogueChoice::new("Bye", "end")
                        .with_on_select("dialogue_end")
                )
        );
        tree.add_node(DialogueNode::new("end", "NPC", "Bye!"));
        let mut runner = DialogueRunner::new(tree);
        runner.drain_events(); // clear on_enter
        runner.select_choice(0);
        let events = runner.drain_events();
        assert!(events.contains(&"dialogue_end".to_string()));
    }
}
