use std::collections::HashMap;

// ---------------------------------------------------------------------------
// QuestState
// ---------------------------------------------------------------------------

/// Lifecycle state of a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestState {
    /// Not yet available to the player.
    Unavailable,
    /// Available but not started.
    Available,
    /// Currently in progress.
    Active,
    /// Successfully completed.
    Complete,
    /// Failed (may or may not be retryable).
    Failed,
}

// ---------------------------------------------------------------------------
// ObjectiveType
// ---------------------------------------------------------------------------

/// What type of objective the player must fulfill.
#[derive(Debug, Clone)]
pub enum ObjectiveType {
    /// Kill N of a specific enemy.
    Kill { enemy_id: String, count: u32 },
    /// Collect N of an item.
    Collect { item_id: String, count: u32 },
    /// Talk to a specific NPC.
    Talk { npc_id: String },
    /// Go to a specific location.
    Goto { location_id: String, radius: f32 },
    /// Escort an NPC to a destination.
    Escort { npc_id: String, destination_id: String },
    /// Talk to a specific NPC (alias for clarity).
    TalkTo { npc_id: String, dialogue_id: Option<String> },
    /// Custom objective with a description.
    Custom { description: String, target_count: u32 },
}

impl ObjectiveType {
    /// Whether this objective type can be tracked numerically.
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Kill { .. } | Self::Collect { .. } | Self::Custom { .. })
    }
}

// ---------------------------------------------------------------------------
// Objective
// ---------------------------------------------------------------------------

/// A single quest objective.
#[derive(Debug, Clone)]
pub struct Objective {
    pub objective_type: ObjectiveType,
    pub description: String,
    pub current_progress: u32,
    pub target_count: u32,
    pub completed: bool,
    /// Optional: ID of the next objective to unlock after this one.
    pub unlocks: Option<String>,
}

impl Objective {
    pub fn new(objective_type: ObjectiveType, description: &str) -> Self {
        let target_count = match &objective_type {
            ObjectiveType::Kill { count, .. } => *count,
            ObjectiveType::Collect { count, .. } => *count,
            ObjectiveType::Goto { .. } => 1,
            ObjectiveType::Talk { .. } => 1,
            ObjectiveType::Escort { .. } => 1,
            ObjectiveType::TalkTo { .. } => 1,
            ObjectiveType::Custom { target_count, .. } => *target_count,
        };
        Self {
            objective_type,
            description: description.to_string(),
            current_progress: 0,
            target_count,
            completed: false,
            unlocks: None,
        }
    }

    pub fn with_progress(mut self, current: u32) -> Self {
        self.current_progress = current;
        self.completed = current >= self.target_count;
        self
    }

    pub fn with_unlocks(mut self, next_objective_id: &str) -> Self {
        self.unlocks = Some(next_objective_id.to_string());
        self
    }

    /// Update progress. Returns true if newly completed.
    pub fn update_progress(&mut self, amount: u32) -> bool {
        if self.completed {
            return false;
        }
        self.current_progress = (self.current_progress + amount).min(self.target_count);
        if self.current_progress >= self.target_count && !self.completed {
            self.completed = true;
            true
        } else {
            false
        }
    }

    /// Completion percentage (0.0..1.0).
    pub fn progress_pct(&self) -> f32 {
        if self.target_count == 0 {
            1.0
        } else {
            self.current_progress as f32 / self.target_count as f32
        }
    }
}

// ---------------------------------------------------------------------------
// Reward
// ---------------------------------------------------------------------------

/// Quest reward.
#[derive(Debug, Clone)]
pub enum Reward {
    XP(u64),
    Gold(u64),
    Item { item_id: String, count: u32 },
    Reputation { faction: String, amount: i32 },
    Unlock(String),
}

// ---------------------------------------------------------------------------
// Quest
// ---------------------------------------------------------------------------

/// A complete quest definition.
#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: QuestState,
    pub objectives: Vec<Objective>,
    pub rewards: Vec<Reward>,
    /// Quest IDs that must be completed before this one becomes available.
    pub prerequisites: Vec<String>,
    /// Whether this quest can be retried after failure.
    pub retryable: bool,
    /// Whether this is a main story quest.
    pub is_main_quest: bool,
    /// Time limit in seconds (None = no limit).
    pub time_limit: Option<f64>,
    /// Elapsed time (updated while active).
    pub elapsed_time: f64,
    /// Level requirement.
    pub required_level: u32,
    /// Quest giver NPC.
    pub giver_npc: Option<String>,
}

impl Quest {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            state: QuestState::Unavailable,
            objectives: Vec::new(),
            rewards: Vec::new(),
            prerequisites: Vec::new(),
            retryable: false,
            is_main_quest: false,
            time_limit: None,
            elapsed_time: 0.0,
            required_level: 1,
            giver_npc: None,
        }
    }

    pub fn with_objective(mut self, objective: Objective) -> Self {
        self.objectives.push(objective);
        self
    }

    pub fn with_reward(mut self, reward: Reward) -> Self {
        self.rewards.push(reward);
        self
    }

    pub fn with_prerequisite(mut self, quest_id: &str) -> Self {
        self.prerequisites.push(quest_id.to_string());
        self
    }

    pub fn with_retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    pub fn with_main_quest(mut self) -> Self {
        self.is_main_quest = true;
        self
    }

    pub fn with_time_limit(mut self, seconds: f64) -> Self {
        self.time_limit = Some(seconds);
        self
    }

    pub fn with_giver(mut self, npc_id: &str) -> Self {
        self.giver_npc = Some(npc_id.to_string());
        self
    }

    pub fn with_level(mut self, level: u32) -> Self {
        self.required_level = level;
        self
    }

    /// Start the quest.
    pub fn activate(&mut self) -> bool {
        if self.state == QuestState::Available || self.state == QuestState::Failed && self.retryable {
            self.state = QuestState::Active;
            self.elapsed_time = 0.0;
            true
        } else {
            false
        }
    }

    /// Check if all objectives are complete.
    pub fn all_objectives_complete(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|o| o.completed)
    }

    /// Complete the quest.
    pub fn complete(&mut self) -> bool {
        if self.state == QuestState::Active && self.all_objectives_complete() {
            self.state = QuestState::Complete;
            true
        } else {
            false
        }
    }

    /// Fail the quest.
    pub fn fail(&mut self) -> bool {
        if self.state == QuestState::Active {
            self.state = QuestState::Failed;
            true
        } else {
            false
        }
    }

    /// Overall completion percentage.
    pub fn progress_pct(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        let total: f32 = self.objectives.iter().map(|o| o.progress_pct()).sum();
        total / self.objectives.len() as f32
    }

    /// Update objective progress by type matching. Returns indices of newly completed objectives.
    pub fn update_objective_progress(&mut self, objective_type: &ObjectiveType, amount: u32) -> Vec<usize> {
        let mut completed_indices = Vec::new();
        for (i, obj) in self.objectives.iter_mut().enumerate() {
            if !obj.completed && std::mem::discriminant(&obj.objective_type) == std::mem::discriminant(objective_type) {
                // Match specific sub-fields
                let matches = match (&obj.objective_type, objective_type) {
                    (ObjectiveType::Kill { enemy_id: a, .. }, ObjectiveType::Kill { enemy_id: b, .. }) => a == b,
                    (ObjectiveType::Collect { item_id: a, .. }, ObjectiveType::Collect { item_id: b, .. }) => a == b,
                    (ObjectiveType::Talk { npc_id: a }, ObjectiveType::Talk { npc_id: b }) => a == b,
                    (ObjectiveType::Goto { location_id: a, .. }, ObjectiveType::Goto { location_id: b, .. }) => a == b,
                    (ObjectiveType::Escort { npc_id: a, .. }, ObjectiveType::Escort { npc_id: b, .. }) => a == b,
                    (ObjectiveType::TalkTo { npc_id: a, .. }, ObjectiveType::TalkTo { npc_id: b, .. }) => a == b,
                    _ => false,
                };
                if matches && obj.update_progress(amount) {
                    completed_indices.push(i);
                }
            }
        }
        completed_indices
    }
}

// ---------------------------------------------------------------------------
// QuestManager
// ---------------------------------------------------------------------------

/// Manages all quests, their states, and provides journal functionality.
pub struct QuestManager {
    pub quests: HashMap<String, Quest>,
    pub completed_quests: Vec<String>,
    pub journal: Vec<JournalEntry>,
    /// Events to fire.
    event_log: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub quest_id: String,
    pub message: String,
    pub timestamp: f64,
}

impl QuestManager {
    pub fn new() -> Self {
        Self {
            quests: HashMap::new(),
            completed_quests: Vec::new(),
            journal: Vec::new(),
            event_log: Vec::new(),
        }
    }

    /// Register a quest.
    pub fn add_quest(&mut self, quest: Quest) {
        self.quests.insert(quest.id.clone(), quest);
    }

    /// Get a quest by ID.
    pub fn get_quest(&self, id: &str) -> Option<&Quest> {
        self.quests.get(id)
    }

    /// Get a mutable quest by ID.
    pub fn get_quest_mut(&mut self, id: &str) -> Option<&mut Quest> {
        self.quests.get_mut(id)
    }

    /// Make a quest available to the player.
    pub fn make_available(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.state == QuestState::Unavailable {
                quest.state = QuestState::Available;
                self.add_journal(quest_id, "Quest is now available.");
                self.event_log.push(format!("quest_available:{}", quest_id));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Accept (activate) a quest.
    pub fn accept_quest(&mut self, quest_id: &str) -> bool {
        // Check prerequisites
        let prereqs_met = {
            let quest = match self.quests.get(quest_id) {
                Some(q) => q,
                None => return false,
            };
            quest.prerequisites.iter().all(|pid| self.completed_quests.contains(pid))
        };

        if !prereqs_met {
            return false;
        }

        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.activate() {
                self.add_journal(quest_id, &format!("Quest accepted: {}", quest.name));
                self.event_log.push(format!("quest_accepted:{}", quest_id));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Complete a quest.
    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.complete() {
                self.completed_quests.push(quest_id.to_string());
                self.add_journal(quest_id, &format!("Quest completed: {}", quest.name));
                self.event_log.push(format!("quest_completed:{}", quest_id));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Fail a quest.
    pub fn fail_quest(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.fail() {
                self.add_journal(quest_id, &format!("Quest failed: {}", quest.name));
                self.event_log.push(format!("quest_failed:{}", quest_id));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get all active quests.
    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Active).collect()
    }

    /// Get all available (but not accepted) quests.
    pub fn available_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Available).collect()
    }

    /// Get completed quests.
    pub fn completed_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Complete).collect()
    }

    /// Check if a quest is completed.
    pub fn is_completed(&self, quest_id: &str) -> bool {
        self.completed_quests.contains(&quest_id)
    }

    /// Add a journal entry.
    fn add_journal(&mut self, quest_id: &str, message: &str) {
        self.journal.push(JournalEntry {
            quest_id: quest_id.to_string(),
            message: message.to_string(),
            timestamp: 0.0, // Set by caller if needed
        });
    }

    /// Get journal entries for a specific quest.
    pub fn quest_journal(&self, quest_id: &str) -> Vec<&JournalEntry> {
        self.journal.iter().filter(|e| e.quest_id == quest_id).collect()
    }

    /// Drain event log.
    pub fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.event_log)
    }

    /// Total quest count.
    pub fn quest_count(&self) -> usize {
        self.quests.len()
    }

    /// Auto-check all active quests for completion.
    pub fn auto_check_completions(&mut self) -> Vec<String> {
        let mut completed = Vec::new();
        let quest_ids: Vec<String> = self.quests.keys().cloned().collect();
        for id in quest_ids {
            if let Some(quest) = self.quests.get_mut(&id) {
                if quest.state == QuestState::Active && quest.all_objectives_complete() {
                    quest.state = QuestState::Complete;
                    self.completed_quests.push(id.clone());
                    self.add_journal(&id, &format!("Quest completed: {}", quest.name));
                    self.event_log.push(format!("quest_completed:{}", id));
                    completed.push(id);
                }
            }
        }
        completed
    }
}

impl Default for QuestManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_lifecycle() {
        let mut quest = Quest::new("q1", "Find the Sword", "Search the cave")
            .with_objective(Objective::new(
                ObjectiveType::Collect { item_id: "sword".to_string(), count: 1 },
                "Find the magic sword",
            ))
            .with_reward(Reward::XP(100));

        assert_eq!(quest.state, QuestState::Unavailable);
        quest.state = QuestState::Available;
        assert!(quest.activate());
        assert_eq!(quest.state, QuestState::Active);
        assert!(!quest.all_objectives_complete());
        quest.objectives[0].current_progress = 1;
        quest.objectives[0].completed = true;
        assert!(quest.complete());
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn test_objective_progress() {
        let mut obj = Objective::new(
            ObjectiveType::Kill { enemy_id: "goblin".to_string(), count: 5 },
            "Kill goblins",
        );
        assert!(!obj.update_progress(2));
        assert_eq!(obj.current_progress, 2);
        assert!(!obj.completed);
        assert!(!obj.update_progress(2));
        assert_eq!(obj.current_progress, 4);
        assert!(!obj.completed);
        assert!(obj.update_progress(1));
        assert!(obj.completed);
        assert_eq!(obj.progress_pct(), 1.0);
    }

    #[test]
    fn test_quest_manager() {
        let mut mgr = QuestManager::new();
        let quest = Quest::new("q1", "Test Quest", "Do stuff")
            .with_objective(Objective::new(
                ObjectiveType::Custom { description: "Complete".to_string(), target_count: 1 },
                "Do the thing",
            ));
        mgr.add_quest(quest);

        assert_eq!(mgr.quest_count(), 1);
        mgr.make_available("q1");
        assert_eq!(mgr.get_quest("q1").unwrap().state, QuestState::Available);

        assert!(mgr.accept_quest("q1"));
        assert_eq!(mgr.active_quests().len(), 1);

        // Complete the objective
        mgr.get_quest_mut("q1").unwrap().objectives[0].completed = true;
        mgr.complete_quest("q1");
        assert!(mgr.is_completed("q1"));
    }

    #[test]
    fn test_quest_prerequisites() {
        let mut mgr = QuestManager::new();
        mgr.add_quest(Quest::new("q1", "First", "First quest"));
        mgr.add_quest(Quest::new("q2", "Second", "Second quest")
            .with_prerequisite("q1"));

        mgr.make_available("q1");
        mgr.make_available("q2");

        // Cannot accept q2 without completing q1
        assert!(!mgr.accept_quest("q2"));

        // Complete q1
        mgr.accept_quest("q1");
        mgr.complete_quest("q1");

        // Now can accept q2
        assert!(mgr.accept_quest("q2"));
    }

    #[test]
    fn test_retryable_quest() {
        let mut quest = Quest::new("q1", "Test", "Fail then retry")
            .with_retryable()
            .with_objective(Objective::new(
                ObjectiveType::Custom { description: "Do".to_string(), target_count: 1 },
                "Do it",
            ));
        quest.state = QuestState::Available;
        quest.activate();
        quest.fail();
        assert!(quest.activate()); // retryable
    }

    #[test]
    fn test_journal() {
        let mut mgr = QuestManager::new();
        mgr.add_quest(Quest::new("q1", "Quest", "Desc"));
        mgr.make_available("q1");
        mgr.accept_quest("q1");
        let entries = mgr.quest_journal("q1");
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_auto_check_completions() {
        let mut mgr = QuestManager::new();
        mgr.add_quest(Quest::new("q1", "Q", "D")
            .with_objective(Objective::new(
                ObjectiveType::Custom { description: "X".to_string(), target_count: 1 },
                "X",
            ).with_progress(1)));
        mgr.make_available("q1");
        mgr.accept_quest("q1");
        let completed = mgr.auto_check_completions();
        assert_eq!(completed.len(), 1);
    }
}
