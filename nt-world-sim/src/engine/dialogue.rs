use std::collections::HashMap;

// ---------------------------------------------------------------------------
// DialogueCondition
// ---------------------------------------------------------------------------

/// A condition that gates a dialogue choice.
#[derive(Debug, Clone)]
pub enum DialogueCondition {
    HasItem { item_id: String, count: u32 },
    NotItem(String),
    QuestState { quest_id: String, state: QuestConditionState },
    Flag(String),
    NotFlag(String),
    Reputation { faction: String, min_value: i32 },
    Gold(u64),
    Level { min: u32, max: u32 },
    And(Vec<DialogueCondition>),
    Or(Vec<DialogueCondition>),
    Not(Box<DialogueCondition>),
}

/// Quest state filter for dialogue conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestConditionState {
    Unavailable,
    Available,
    Active,
    Complete,
    Failed,
}

// ---------------------------------------------------------------------------
// DialogueEffect
// ---------------------------------------------------------------------------

/// An effect applied when a dialogue choice is selected.
#[derive(Debug, Clone)]
pub enum DialogueEffect {
    GiveItem { item_id: String, count: u32 },
    RemoveItem { item_id: String, count: u32 },
    AddGold(i64),
    SetFlag(String),
    ClearFlag(String),
    StartQuest(String),
    CompleteQuest(String),
    ChangeReputation { faction: String, amount: i32 },
    HealPlayer(f32),
    DamagePlayer(f32),
    Teleport { x: f32, y: f32 },
    PlaySound(String),
    ShowNotification(String),
}

// ---------------------------------------------------------------------------
// DialoguePortrait
// ---------------------------------------------------------------------------

/// Portrait displayed alongside dialogue text.
#[derive(Debug, Clone)]
pub struct DialoguePortrait {
    pub texture: String,
    pub expression: String,
    pub position: PortraitPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortraitPosition {
    Left,
    Right,
    Center,
}

impl DialoguePortrait {
    pub fn new(texture: &str) -> Self {
        Self {
            texture: texture.to_string(),
            expression: "neutral".to_string(),
            position: PortraitPosition::Left,
        }
    }

    pub fn with_expression(mut self, expr: &str) -> Self {
        self.expression = expr.to_string();
        self
    }

    pub fn with_position(mut self, pos: PortraitPosition) -> Self {
        self.position = pos;
        self
    }
}

// ---------------------------------------------------------------------------
// DialogueNode
// ---------------------------------------------------------------------------

/// A single node in a dialogue tree.
#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub next_node: Option<String>,
    pub portrait: Option<DialoguePortrait>,
    pub on_enter: Vec<String>,
    pub on_exit: Vec<String>,
    pub effects: Vec<DialogueEffect>,
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
            portrait: None,
            on_enter: Vec::new(),
            on_exit: Vec::new(),
            effects: Vec::new(),
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

    pub fn with_portrait(mut self, portrait: DialoguePortrait) -> Self {
        self.portrait = Some(portrait);
        self
    }

    pub fn with_effect(mut self, effect: DialogueEffect) -> Self {
        self.effects.push(effect);
        self
    }
}

// ---------------------------------------------------------------------------
// DialogueChoice
// ---------------------------------------------------------------------------

/// A choice presented to the player.
#[derive(Debug, Clone)]
pub struct DialogueChoice {
    pub text: String,
    pub next_node: String,
    pub conditions: Vec<DialogueCondition>,
    pub effects: Vec<DialogueEffect>,
    pub ends_dialogue: bool,
    pub hide_if_unavailable: bool,
    pub on_select: Vec<String>,
}

impl DialogueChoice {
    pub fn new(text: &str, next_node: &str) -> Self {
        Self {
            text: text.to_string(),
            next_node: next_node.to_string(),
            conditions: Vec::new(),
            effects: Vec::new(),
            ends_dialogue: false,
            hide_if_unavailable: false,
            on_select: Vec::new(),
        }
    }

    pub fn with_condition(mut self, cond: DialogueCondition) -> Self {
        self.conditions.push(cond);
        self
    }

    pub fn with_effect(mut self, effect: DialogueEffect) -> Self {
        self.effects.push(effect);
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
// DialogueTree
// ---------------------------------------------------------------------------

/// A complete dialogue tree with nodes and a starting point.
#[derive(Debug, Clone)]
pub struct DialogueTree {
    pub nodes: HashMap<String, DialogueNode>,
    pub start_node: String,
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
// GameStateContext — external state for condition evaluation
// ---------------------------------------------------------------------------

/// Snapshot of game state used by DialogueRunner to evaluate conditions.
pub struct GameStateContext {
    pub flags: HashMap<String, bool>,
    pub items: HashMap<String, u32>,
    pub gold: i64,
    pub level: u32,
    pub reputation: HashMap<String, i32>,
    pub quest_states: HashMap<String, QuestConditionState>,
}

impl GameStateContext {
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
            items: HashMap::new(),
            gold: 0,
            level: 1,
            reputation: HashMap::new(),
            quest_states: HashMap::new(),
        }
    }

    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.flags.insert(name.to_string(), value);
    }

    pub fn add_item(&mut self, item_id: &str, count: u32) {
        *self.items.entry(item_id.to_string()).or_insert(0) += count;
    }

    pub fn remove_item(&mut self, item_id: &str, count: u32) -> bool {
        if let Some(c) = self.items.get_mut(item_id) {
            if *c >= count {
                *c -= count;
                if *c == 0 {
                    self.items.remove(item_id);
                }
                return true;
            }
        }
        false
    }

    pub fn has_item(&self, item_id: &str, count: u32) -> bool {
        self.items.get(item_id).map_or(false, |c| *c >= count)
    }

    pub fn set_gold(&mut self, amount: i64) {
        self.gold = amount.max(0);
    }

    pub fn modify_gold(&mut self, amount: i64) {
        self.gold = (self.gold + amount).max(0);
    }

    pub fn set_level(&mut self, level: u32) {
        self.level = level;
    }

    pub fn set_reputation(&mut self, faction: &str, amount: i32) {
        self.reputation.insert(faction.to_string(), amount);
    }

    pub fn set_quest_state(&mut self, quest_id: &str, state: QuestConditionState) {
        self.quest_states.insert(quest_id.to_string(), state);
    }
}

impl Default for GameStateContext {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// DialogueRunner
// ---------------------------------------------------------------------------

/// Runtime state for an active dialogue.
pub struct DialogueRunner {
    pub tree: DialogueTree,
    current_node: Option<String>,
    variables: HashMap<String, String>,
    history: Vec<String>,
    event_log: Vec<String>,
    effect_log: Vec<DialogueEffect>,
    game_state: GameStateContext,
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
            effect_log: Vec::new(),
            game_state: GameStateContext::new(),
        }
    }

    pub fn with_game_state(mut self, state: GameStateContext) -> Self {
        self.game_state = state;
        self
    }

    /// Start (or restart) the dialogue.
    pub fn start(&mut self) {
        self.current_node = Some(self.tree.start_node.clone());
        self.history.clear();
    }

    /// Start a specific tree by id (multi-tree support).
    pub fn start_tree(&mut self, tree_id: &str) -> bool {
        if let Some(node_id) = self.tree.metadata.get(tree_id).cloned() {
            self.current_node = Some(node_id);
            self.history.clear();
            true
        } else {
            false
        }
    }

    /// Get the current node (with variables substituted).
    pub fn get_current_node(&self) -> Option<DialogueNodeDisplay> {
        let node = self.tree.get_node(self.current_node.as_deref()?)?;
        let text = self.substitute(&node.text);
        let speaker = node.speaker.clone();
        let next_node = node.next_node.clone();
        let portrait = node.portrait.clone();

        let choices: Vec<ChoiceDisplay> = node.choices.iter().enumerate().map(|(i, c)| {
            let available = c.conditions.iter().all(|cond| self.evaluate_condition(cond));
            ChoiceDisplay {
                text: if available { self.substitute(&c.text) } else { c.text.clone() },
                index: i,
                available,
                ends_dialogue: c.ends_dialogue,
                hide: c.hide_if_unavailable && !available,
            }
        }).filter(|c| !c.hide).collect();

        Some(DialogueNodeDisplay { speaker, text, choices, next_node, portrait })
    }

    /// Get available (non-hidden) choices.
    pub fn get_available_choices(&self) -> Vec<&DialogueChoice> {
        let node_id = self.current_node.as_deref().unwrap_or("");
        let node = match self.tree.get_node(node_id) {
            Some(n) => n,
            None => return Vec::new(),
        };
        node.choices.iter().filter(|c| {
            !c.hide_if_unavailable || c.conditions.iter().all(|cond| self.evaluate_condition(cond))
        }).collect()
    }

    /// Select a choice by index. Returns the effects applied.
    pub fn select_choice(&mut self, index: usize) -> Option<Vec<DialogueEffect>> {
        let node_id = self.current_node.clone().unwrap_or_default();
        let node = match self.tree.get_node(&node_id) {
            Some(n) => n.clone(),
            None => return None,
        };

        let choice = node.choices.get(index)?;
        if !choice.conditions.iter().all(|c| self.evaluate_condition(c)) {
            return None;
        }

        let mut effects = Vec::new();

        // Apply choice effects
        for effect in &choice.effects {
            self.apply_effect(effect);
            effects.push(effect.clone());
        }

        // Apply node exit effects
        for effect in &node.effects {
            self.apply_effect(effect);
            effects.push(effect.clone());
        }

        // Fire events
        for event in &choice.on_select {
            self.event_log.push(event.clone());
        }

        self.history.push(node_id);

        if choice.ends_dialogue {
            self.current_node = None;
        } else {
            self.current_node = Some(choice.next_node.clone());
            // Fire enter events of next node
            if let Some(next_node) = self.tree.get_node(&choice.next_node) {
                for event in &next_node.on_enter {
                    self.event_log.push(event.clone());
                }
                for effect in &next_node.effects {
                    self.apply_effect(effect);
                    effects.push(effect.clone());
                }
            }
        }

        Some(effects)
    }

    /// Advance to the next node (for auto-advance nodes with no choices).
    pub fn advance(&mut self) -> bool {
        let node_id = self.current_node.clone().unwrap_or_default();
        let node = match self.tree.get_node(&node_id) {
            Some(n) => n.clone(),
            None => return false,
        };

        if !node.choices.is_empty() {
            return false;
        }

        for event in &node.on_exit {
            self.event_log.push(event.clone());
        }

        self.history.push(node_id);

        if let Some(next) = &node.next_node {
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

    pub fn is_active(&self) -> bool {
        self.current_node.is_some()
    }

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.variables.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|s| s.as_str())
    }

    pub fn set_game_state(&mut self, state: GameStateContext) {
        self.game_state = state;
    }

    pub fn game_state(&self) -> &GameStateContext {
        &self.game_state
    }

    pub fn game_state_mut(&mut self) -> &mut GameStateContext {
        &mut self.game_state
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.event_log)
    }

    pub fn drain_effects(&mut self) -> Vec<DialogueEffect> {
        std::mem::take(&mut self.effect_log)
    }

    fn substitute(&self, text: &str) -> String {
        let mut result = text.to_string();
        // Built-in substitutions
        let builtins: Vec<(&str, String)> = vec![
            ("player_level".into(), self.game_state.level.to_string()),
            ("player_gold".into(), self.game_state.gold.to_string()),
        ];
        for (key, value) in &builtins {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    fn apply_effect(&mut self, effect: &DialogueEffect) {
        match effect {
            DialogueEffect::GiveItem { item_id, count } => {
                self.game_state.add_item(item_id, *count);
            }
            DialogueEffect::RemoveItem { item_id, count } => {
                self.game_state.remove_item(item_id, *count);
            }
            DialogueEffect::AddGold(amount) => {
                self.game_state.modify_gold(*amount);
            }
            DialogueEffect::SetFlag(flag) => {
                self.game_state.set_flag(flag, true);
            }
            DialogueEffect::ClearFlag(flag) => {
                self.game_state.set_flag(flag, false);
            }
            DialogueEffect::StartQuest(quest_id) => {
                self.event_log.push(format!("quest_start:{}", quest_id));
            }
            DialogueEffect::CompleteQuest(quest_id) => {
                self.event_log.push(format!("quest_complete:{}", quest_id));
            }
            DialogueEffect::ChangeReputation { faction, amount } => {
                let current = self.game_state.reputation.get(faction).copied().unwrap_or(0);
                self.game_state.set_reputation(faction, current + amount);
            }
            DialogueEffect::HealPlayer(amount) => {
                self.event_log.push(format!("heal_player:{}", amount));
            }
            DialogueEffect::DamagePlayer(amount) => {
                self.event_log.push(format!("damage_player:{}", amount));
            }
            DialogueEffect::Teleport { x, y } => {
                self.event_log.push(format!("teleport:{},{}", x, y));
            }
            DialogueEffect::PlaySound(s) => {
                self.event_log.push(format!("play_sound:{}", s));
            }
            DialogueEffect::ShowNotification(s) => {
                self.event_log.push(format!("notify:{}", s));
            }
        }
        self.effect_log.push(effect.clone());
    }

    fn evaluate_condition(&self, condition: &DialogueCondition) -> bool {
        match condition {
            DialogueCondition::HasItem { item_id, count } => {
                self.game_state.has_item(item_id, *count)
            }
            DialogueCondition::NotItem(item_id) => {
                !self.game_state.has_item(item_id, 1)
            }
            DialogueCondition::QuestState { quest_id, state } => {
                self.game_state.quest_states.get(quest_id).map_or(false, |s| s == state)
            }
            DialogueCondition::Flag(flag) => {
                self.game_state.flags.get(flag).copied().unwrap_or(false)
                    || self.variables.get(flag).map_or(false, |v| v == "true")
            }
            DialogueCondition::NotFlag(flag) => {
                !self.game_state.flags.get(flag).copied().unwrap_or(false)
                    && !self.variables.get(flag).map_or(false, |v| v == "true")
            }
            DialogueCondition::Reputation { faction, min_value } => {
                self.game_state.reputation.get(faction).copied().unwrap_or(0) >= *min_value
            }
            DialogueCondition::Gold(amount) => {
                self.game_state.gold >= *amount as i64
            }
            DialogueCondition::Level { min, max } => {
                self.game_state.level >= *min && self.game_state.level <= *max
            }
            DialogueCondition::And(conds) => {
                conds.iter().all(|c| self.evaluate_condition(c))
            }
            DialogueCondition::Or(conds) => {
                conds.iter().any(|c| self.evaluate_condition(c))
            }
            DialogueCondition::Not(cond) => {
                !self.evaluate_condition(cond)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Display types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DialogueNodeDisplay {
    pub speaker: String,
    pub text: String,
    pub choices: Vec<ChoiceDisplay>,
    pub next_node: Option<String>,
    pub portrait: Option<DialoguePortrait>,
}

#[derive(Debug, Clone)]
pub struct ChoiceDisplay {
    pub text: String,
    pub index: usize,
    pub available: bool,
    pub ends_dialogue: bool,
    pub hide: bool,
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
            DialogueNode::new("farewell", "Guard", "Safe travels, {player_name}!")
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
    fn test_runner_advance() {
        let tree = build_test_tree();
        let mut runner = DialogueRunner::new(tree);
        assert!(!runner.advance());
        assert!(runner.select_choice(0).is_some());
        assert!(runner.is_active());
        let current = runner.get_current_node().unwrap();
        assert_eq!(current.speaker, "Guard");
        assert!(runner.advance());
        assert!(runner.advance());
        assert!(!runner.is_active());
    }

    #[test]
    fn test_variable_substitution() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(DialogueNode::new("talk", "NPC", "Hello, {player_name}!"));
        let mut runner = DialogueRunner::new(tree);
        runner.set_variable("player_name", "Hero");
        let display = runner.get_current_node().unwrap();
        assert_eq!(display.text, "Hello, Hero!");
    }

    #[test]
    fn test_game_state_conditions() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Choose:")
                .with_choice(
                    DialogueChoice::new("Free option", "free")
                )
                .with_choice(
                    DialogueChoice::new("Requires sword", "has_sword")
                        .with_condition(DialogueCondition::HasItem {
                            item_id: "sword".into(), count: 1,
                        })
                )
        );
        let mut state = GameStateContext::new();
        state.add_item("sword", 1);
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        let display = runner.get_current_node().unwrap();
        assert_eq!(display.choices.len(), 2);
        assert!(display.choices[0].available);
        assert!(display.choices[1].available);
    }

    #[test]
    fn test_nested_and_or_conditions() {
        let cond = DialogueCondition::And(vec![
            DialogueCondition::Gold(100),
            DialogueCondition::Level { min: 5, max: 99 },
        ]);
        let mut state = GameStateContext::new();
        state.gold = 200;
        state.level = 10;
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(runner.evaluate_condition(&cond));

        let cond_fail = DialogueCondition::Or(vec![
            DialogueCondition::Gold(999),
            DialogueCondition::Level { min: 99, max: 99 },
        ]);
        assert!(!runner.evaluate_condition(&cond_fail));
    }

    #[test]
    fn test_not_condition() {
        let cond = DialogueCondition::Not(Box::new(DialogueCondition::Flag("dead".into())));
        let mut state = GameStateContext::new();
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(runner.evaluate_condition(&cond));
    }

    #[test]
    fn test_choice_effects() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Here's a reward.")
                .with_choice(
                    DialogueChoice::new("Accept", "done")
                        .with_effect(DialogueEffect::GiveItem {
                            item_id: "potion".into(), count: 3,
                        })
                        .with_effect(DialogueEffect::AddGold(50))
                )
        );
        tree.add_node(DialogueNode::new("done", "NPC", "Done."));
        let mut runner = DialogueRunner::new(tree);
        let effects = runner.select_choice(0).unwrap();
        assert_eq!(effects.len(), 2);
        assert!(runner.game_state().has_item("potion", 3));
        assert_eq!(runner.game_state().gold, 50);
    }

    #[test]
    fn test_portrait() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(
            DialogueNode::new("talk", "NPC", "Hello!")
                .with_portrait(DialoguePortrait::new("npc_portrait.png")
                    .with_expression("happy")
                    .with_position(PortraitPosition::Right))
        );
        let runner = DialogueRunner::new(tree);
        let display = runner.get_current_node().unwrap();
        assert!(display.portrait.is_some());
        let p = display.portrait.unwrap();
        assert_eq!(p.expression, "happy");
        assert_eq!(p.position, PortraitPosition::Right);
    }

    #[test]
    fn test_reputation_condition() {
        let cond = DialogueCondition::Reputation { faction: "guards".into(), min_value: 50 };
        let mut state = GameStateContext::new();
        state.set_reputation("guards", 60);
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(runner.evaluate_condition(&cond));
    }

    #[test]
    fn test_quest_state_condition() {
        let cond = DialogueCondition::QuestState {
            quest_id: "q1".into(),
            state: QuestConditionState::Complete,
        };
        let mut state = GameStateContext::new();
        state.set_quest_state("q1", QuestConditionState::Complete);
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(runner.evaluate_condition(&cond));
    }

    #[test]
    fn test_gold_condition() {
        let cond = DialogueCondition::Gold(100);
        let mut state = GameStateContext::new();
        state.gold = 50;
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(!runner.evaluate_condition(&cond));
        runner.game_state_mut().gold = 150;
        assert!(runner.evaluate_condition(&cond));
    }

    #[test]
    fn test_level_condition() {
        let cond = DialogueCondition::Level { min: 5, max: 10 };
        let mut state = GameStateContext::new();
        state.level = 7;
        let mut tree = DialogueTree::new("n");
        tree.add_node(DialogueNode::new("n", "NPC", "Hi"));
        let mut runner = DialogueRunner::new(tree).with_game_state(state);
        assert!(runner.evaluate_condition(&cond));
        runner.game_state_mut().level = 3;
        assert!(!runner.evaluate_condition(&cond));
    }

    #[test]
    fn test_builtin_substitution() {
        let mut tree = DialogueTree::new("talk");
        tree.add_node(DialogueNode::new("talk", "NPC", "You have {player_gold} gold."));
        let mut state = GameStateContext::new();
        state.gold = 42;
        let runner = DialogueRunner::new(tree).with_game_state(state);
        let display = runner.get_current_node().unwrap();
        assert_eq!(display.text, "You have 42 gold.");
    }
}
