use std::collections::HashMap;

// ---------------------------------------------------------------------------
// QuestState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestState {
    Unavailable,
    Available,
    Active,
    Complete,
    Failed,
}

// ---------------------------------------------------------------------------
// ObjectiveType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum ObjectiveType {
    Kill { enemy_id: String, count: u32 },
    Collect { item_id: String, count: u32 },
    Talk { npc_id: String },
    Goto { location_id: String, radius: f32 },
    Escort { npc_id: String, destination_id: String },
    TalkTo { npc_id: String, dialogue_id: Option<String> },
    Custom { description: String, target_count: u32 },
}

impl ObjectiveType {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Kill { .. } | Self::Collect { .. } | Self::Custom { .. })
    }

    /// Get the primary target id (if any).
    pub fn target_id(&self) -> Option<&str> {
        match self {
            ObjectiveType::Kill { enemy_id, .. } => Some(enemy_id),
            ObjectiveType::Collect { item_id, .. } => Some(item_id),
            ObjectiveType::Talk { npc_id } => Some(npc_id),
            ObjectiveType::Goto { location_id, .. } => Some(location_id),
            ObjectiveType::Escort { npc_id, .. } => Some(npc_id),
            ObjectiveType::TalkTo { npc_id, .. } => Some(npc_id),
            ObjectiveType::Custom { .. } => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Objective
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Objective {
    pub id: String,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub current_progress: u32,
    pub target_count: u32,
    pub completed: bool,
    pub optional: bool,
    pub unlocks: Option<String>,
}

impl Objective {
    pub fn new(id: &str, objective_type: ObjectiveType, description: &str) -> Self {
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
            id: id.to_string(),
            objective_type,
            description: description.to_string(),
            current_progress: 0,
            target_count,
            completed: false,
            optional: false,
            unlocks: None,
        }
    }

    pub fn with_progress(mut self, current: u32) -> Self {
        self.current_progress = current;
        self.completed = current >= self.target_count;
        self
    }

    pub fn with_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn with_unlocks(mut self, next_id: &str) -> Self {
        self.unlocks = Some(next_id.to_string());
        self
    }

    pub fn update_progress(&mut self, amount: u32) -> bool {
        if self.completed { return false; }
        self.current_progress = (self.current_progress + amount).min(self.target_count);
        if self.current_progress >= self.target_count && !self.completed {
            self.completed = true;
            true
        } else {
            false
        }
    }

    pub fn progress_pct(&self) -> f32 {
        if self.target_count == 0 { 1.0 } else { self.current_progress as f32 / self.target_count as f32 }
    }

    pub fn is_mandatory(&self) -> bool {
        !self.optional
    }
}

// ---------------------------------------------------------------------------
// Reward
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Reward {
    XP(u64),
    Gold(u64),
    Item { item_id: String, count: u32 },
    Reputation { faction: String, amount: i32 },
    Unlock(String),
    SkillPoint(u32),
}

impl Reward {
    pub fn description(&self) -> String {
        match self {
            Reward::XP(xp) => format!("{} XP", xp),
            Reward::Gold(g) => format!("{} Gold", g),
            Reward::Item { item_id, count } => format!("{} x{}", item_id, count),
            Reward::Reputation { faction, amount } => format!("{} reputation ({})", faction, amount),
            Reward::Unlock(id) => format!("Unlock: {}", id),
            Reward::SkillPoint(p) => format!("{} Skill Points", p),
        }
    }
}

// ---------------------------------------------------------------------------
// Quest
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: QuestState,
    pub objectives: Vec<Objective>,
    pub rewards: Vec<Reward>,
    pub prerequisites: Vec<String>,
    pub retryable: bool,
    pub is_main_quest: bool,
    pub time_limit: Option<f64>,
    pub elapsed_time: f64,
    pub required_level: u32,
    pub giver_npc: Option<String>,
    pub repeatable: bool,
    pub max_repeats: Option<u32>,
    pub repeat_count: u32,
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
            repeatable: false,
            max_repeats: None,
            repeat_count: 0,
        }
    }

    pub fn with_objective(mut self, objective: Objective) -> Self { self.objectives.push(objective); self }
    pub fn with_reward(mut self, reward: Reward) -> Self { self.rewards.push(reward); self }
    pub fn with_prerequisite(mut self, quest_id: &str) -> Self { self.prerequisites.push(quest_id.to_string()); self }
    pub fn with_retryable(mut self) -> Self { self.retryable = true; self }
    pub fn with_main_quest(mut self) -> Self { self.is_main_quest = true; self }
    pub fn with_time_limit(mut self, seconds: f64) -> Self { self.time_limit = Some(seconds); self }
    pub fn with_giver(mut self, npc_id: &str) -> Self { self.giver_npc = Some(npc_id.to_string()); self }
    pub fn with_level(mut self, level: u32) -> Self { self.required_level = level; self }
    pub fn with_repeatable(mut self, max: u32) -> Self { self.repeatable = true; self.max_repeats = Some(max); self }

    pub fn activate(&mut self) -> bool {
        if self.state == QuestState::Available
            || (self.state == QuestState::Failed && self.retryable)
            || (self.repeatable && self.state == QuestState::Complete
                && self.max_repeats.map_or(true, |max| self.repeat_count < max))
        {
            self.state = QuestState::Active;
            self.elapsed_time = 0.0;
            true
        } else {
            false
        }
    }

    pub fn all_objectives_complete(&self) -> bool {
        let mandatory: Vec<&Objective> = self.objectives.iter().filter(|o| o.is_mandatory()).collect();
        mandatory.iter().all(|o| o.completed)
    }

    pub fn complete(&mut self) -> bool {
        if self.state == QuestState::Active && self.all_objectives_complete() {
            self.state = QuestState::Complete;
            self.repeat_count += 1;
            true
        } else {
            false
        }
    }

    pub fn fail(&mut self) -> bool {
        if self.state == QuestState::Active {
            self.state = QuestState::Failed;
            true
        } else {
            false
        }
    }

    pub fn progress_pct(&self) -> f32 {
        if self.objectives.is_empty() { return 0.0; }
        let total: f32 = self.objectives.iter().map(|o| o.progress_pct()).sum();
        total / self.objectives.len() as f32
    }

    /// Update objective progress by objective id. Returns true if newly completed.
    pub fn complete_objective_by_id(&mut self, objective_id: &str, amount: u32) -> bool {
        if let Some(obj) = self.objectives.iter_mut().find(|o| o.id == objective_id) {
            obj.update_progress(amount)
        } else {
            false
        }
    }

    /// Update objective progress by type matching. Returns indices of newly completed objectives.
    pub fn update_objective_progress(&mut self, objective_type: &ObjectiveType, amount: u32) -> Vec<usize> {
        let mut completed_indices = Vec::new();
        for (i, obj) in self.objectives.iter_mut().enumerate() {
            if !obj.completed && std::mem::discriminant(&obj.objective_type) == std::mem::discriminant(objective_type) {
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

    /// Check if quest is timed and has expired.
    pub fn is_timed_out(&self) -> bool {
        self.time_limit.map_or(false, |limit| self.elapsed_time >= limit && self.state == QuestState::Active)
    }

    /// Update elapsed time. Returns true if timed out.
    pub fn update_time(&mut self, dt: f64) -> bool {
        if self.state == QuestState::Active {
            self.elapsed_time += dt;
            self.is_timed_out()
        } else {
            false
        }
    }

    /// Get a summary string.
    pub fn summary(&self) -> String {
        let obj_str: Vec<String> = self.objectives.iter().map(|o| {
            format!("{} [{}/{}]", o.description, o.current_progress, o.target_count)
        }).collect();
        format!("{}: {}", self.name, obj_str.join(", "))
    }
}

// ---------------------------------------------------------------------------
// QuestManager
// ---------------------------------------------------------------------------

pub struct QuestManager {
    pub quests: HashMap<String, Quest>,
    pub completed_quests: Vec<String>,
    pub journal: Vec<JournalEntry>,
    pub player_level: u32,
    pub player_flags: HashMap<String, bool>,
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
            player_level: 1,
            player_flags: HashMap::new(),
            event_log: Vec::new(),
        }
    }

    pub fn set_player_level(&mut self, level: u32) { self.player_level = level; }
    pub fn set_flag(&mut self, flag: &str, value: bool) { self.player_flags.insert(flag.to_string(), value); }

    pub fn add_quest(&mut self, quest: Quest) { self.quests.insert(quest.id.clone(), quest); }
    pub fn get_quest(&self, id: &str) -> Option<&Quest> { self.quests.get(id) }
    pub fn get_quest_mut(&mut self, id: &str) -> Option<&mut Quest> { self.quests.get_mut(id) }

    pub fn make_available(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.state == QuestState::Unavailable {
                quest.state = QuestState::Available;
                self.add_journal(quest_id, "Quest is now available.");
                self.event_log.push(format!("quest_available:{}", quest_id));
                true
            } else { false }
        } else { false }
    }

    pub fn check_prerequisites(&self, quest_id: &str) -> bool {
        let quest = match self.quests.get(quest_id) {
            Some(q) => q,
            None => return false,
        };
        if self.player_level < quest.required_level { return false; }
        quest.prerequisites.iter().all(|pid| self.completed_quests.contains(pid))
    }

    pub fn accept_quest(&mut self, quest_id: &str) -> bool {
        if !self.check_prerequisites(quest_id) { return false; }
        let result = self.quests.get_mut(quest_id).and_then(|q| {
            if q.activate() {
                let name = q.name.clone();
                Some(name)
            } else { None }
        });
        if let Some(name) = result {
            self.add_journal(quest_id, &format!("Quest accepted: {}", name));
            self.event_log.push(format!("quest_accepted:{}", quest_id));
            true
        } else { false }
    }

    pub fn complete_quest(&mut self, quest_id: &str) -> bool {
        let result = self.quests.get_mut(quest_id).and_then(|q| {
            if q.complete() {
                let name = q.name.clone();
                Some(name)
            } else { None }
        });
        if let Some(name) = result {
            self.completed_quests.push(quest_id.to_string());
            self.add_journal(quest_id, &format!("Quest completed: {}", name));
            self.event_log.push(format!("quest_completed:{}", quest_id));
            true
        } else { false }
    }

    pub fn fail_quest(&mut self, quest_id: &str) -> bool {
        let result = self.quests.get_mut(quest_id).and_then(|q| {
            if q.fail() {
                let name = q.name.clone();
                Some(name)
            } else { None }
        });
        if let Some(name) = result {
            self.add_journal(quest_id, &format!("Quest failed: {}", name));
            self.event_log.push(format!("quest_failed:{}", quest_id));
            true
        } else { false }
    }

    pub fn complete_objective(&mut self, quest_id: &str, objective_id: &str) -> bool {
        let result = self.quests.get_mut(quest_id).and_then(|q| {
            if q.state != QuestState::Active { return None; }
            let completed = q.complete_objective_by_id(objective_id, 1);
            if completed {
                // Auto-complete quest if all objectives done
                if q.all_objectives_complete() {
                    q.state = QuestState::Complete;
                    q.repeat_count += 1;
                }
                Some(())
            } else { None }
        });
        if result.is_some() {
            self.add_journal(quest_id, &format!("Objective completed: {}", objective_id));
            self.event_log.push(format!("objective_completed:{}:{}", quest_id, objective_id));
            // Track quest completion
            if let Some(q) = self.quests.get(quest_id) {
                if q.state == QuestState::Complete && !self.completed_quests.contains(&quest_id.to_string()) {
                    self.completed_quests.push(quest_id.to_string());
                    self.add_journal(quest_id, &format!("Quest completed: {}", q.name));
                    self.event_log.push(format!("quest_completed:{}", quest_id));
                }
            }
            true
        } else { false }
    }

    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Active).collect()
    }

    pub fn available_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Available).collect()
    }

    pub fn completed_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Complete).collect()
    }

    pub fn failed_quests(&self) -> Vec<&Quest> {
        self.quests.values().filter(|q| q.state == QuestState::Failed).collect()
    }

    pub fn is_completed(&self, quest_id: &str) -> bool {
        self.completed_quests.contains(&quest_id.to_string())
    }

    pub fn quest_journal(&self, quest_id: &str) -> Vec<&JournalEntry> {
        self.journal.iter().filter(|e| e.quest_id == quest_id).collect()
    }

    pub fn drain_events(&mut self) -> Vec<String> { std::mem::take(&mut self.event_log) }
    pub fn quest_count(&self) -> usize { self.quests.len() }

    pub fn auto_check_completions(&mut self) -> Vec<String> {
        let mut completed = Vec::new();
        let quest_ids: Vec<String> = self.quests.keys().cloned().collect();
        for id in quest_ids {
            let result = self.quests.get_mut(&id).and_then(|q| {
                if q.state == QuestState::Active && q.all_objectives_complete() {
                    q.state = QuestState::Complete;
                    q.repeat_count += 1;
                    let name = q.name.clone();
                    Some(name)
                } else { None }
            });
            if let Some(name) = result {
                self.completed_quests.push(id.clone());
                self.add_journal(&id, &format!("Quest completed: {}", name));
                self.event_log.push(format!("quest_completed:{}", id));
                completed.push(id);
            }
        }
        completed
    }

    /// Auto-fail timed out quests.
    pub fn auto_check_timeouts(&mut self) -> Vec<String> {
        let mut timed_out = Vec::new();
        let quest_ids: Vec<String> = self.quests.keys().cloned().collect();
        for id in quest_ids {
            let result = self.quests.get_mut(&id).and_then(|q| {
                if q.state == QuestState::Active && q.is_timed_out() {
                    q.state = QuestState::Failed;
                    let name = q.name.clone();
                    Some(name)
                } else { None }
            });
            if let Some(name) = result {
                self.add_journal(&id, &format!("Quest timed out: {}", name));
                self.event_log.push(format!("quest_timed_out:{}", id));
                timed_out.push(id);
            }
        }
        timed_out
    }

    /// Update all active quest timers. Returns timed out quest ids.
    pub fn tick(&mut self, dt: f64) -> Vec<String> {
        let mut timed_out = Vec::new();
        for quest in self.quests.values_mut() {
            if quest.state == QuestState::Active && quest.update_time(dt) {
                timed_out.push(quest.id.clone());
            }
        }
        timed_out
    }

    fn add_journal(&mut self, quest_id: &str, message: &str) {
        self.journal.push(JournalEntry {
            quest_id: quest_id.to_string(),
            message: message.to_string(),
            timestamp: 0.0,
        });
    }
}

impl Default for QuestManager {
    fn default() -> Self { Self::new() }
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
            .with_objective(Objective::new("obj1",
                ObjectiveType::Collect { item_id: "sword".into(), count: 1 },
                "Find the magic sword"))
            .with_reward(Reward::XP(100));

        assert_eq!(quest.state, QuestState::Unavailable);
        quest.state = QuestState::Available;
        assert!(quest.activate());
        assert_eq!(quest.state, QuestState::Active);
        assert!(!quest.all_objectives_complete());
        quest.complete_objective_by_id("obj1", 1);
        assert!(quest.all_objectives_complete());
        assert!(quest.complete());
        assert_eq!(quest.state, QuestState::Complete);
    }

    #[test]
    fn test_objective_progress() {
        let mut obj = Objective::new("obj1",
            ObjectiveType::Kill { enemy_id: "goblin".into(), count: 5 },
            "Kill goblins");
        assert!(!obj.update_progress(2));
        assert_eq!(obj.current_progress, 2);
        assert!(obj.update_progress(3));
        assert!(obj.completed);
    }

    #[test]
    fn test_optional_objective() {
        let mut quest = Quest::new("q1", "Test", "Test optional");
        quest.objectives.push(Objective::new("main", ObjectiveType::Custom { description: "Main".into(), target_count: 1 }, "Do main"));
        quest.objectives.push(Objective::new("opt", ObjectiveType::Custom { description: "Optional".into(), target_count: 1 }, "Optional").with_optional());
        quest.state = QuestState::Active;
        quest.complete_objective_by_id("main", 1);
        assert!(quest.all_objectives_complete()); // optional doesn't block
    }

    #[test]
    fn test_quest_manager_prerequisites() {
        let mut mgr = QuestManager::new();
        mgr.add_quest(Quest::new("q1", "First", "First quest"));
        mgr.add_quest(Quest::new("q2", "Second", "Second quest").with_prerequisite("q1"));
        mgr.make_available("q1");
        mgr.make_available("q2");
        assert!(!mgr.accept_quest("q2"));
        mgr.accept_quest("q1");
        mgr.complete_quest("q1");
        assert!(mgr.accept_quest("q2"));
    }

    #[test]
    fn test_complete_objective() {
        let mut mgr = QuestManager::new();
        let quest = Quest::new("q1", "Kill Goblins", "Kill 3")
            .with_objective(Objective::new("kill1",
                ObjectiveType::Kill { enemy_id: "goblin".into(), count: 3 },
                "Kill goblins"));
        mgr.add_quest(quest);
        mgr.make_available("q1");
        mgr.accept_quest("q1");
        mgr.complete_objective("q1", "kill1");
        mgr.complete_objective("q1", "kill1");
        assert!(mgr.complete_objective("q1", "kill1"));
        assert!(mgr.is_completed("q1"));
    }

    #[test]
    fn test_player_level_prerequisite() {
        let mut mgr = QuestManager::new();
        mgr.add_quest(Quest::new("q1", "Level 10", "High level").with_level(10));
        mgr.make_available("q1");
        assert!(!mgr.check_prerequisites("q1"));
        mgr.set_player_level(10);
        assert!(mgr.check_prerequisites("q1"));
    }

    #[test]
    fn test_quest_timer() {
        let mut quest = Quest::new("q1", "Timed", "Do it fast").with_time_limit(30.0);
        quest.state = QuestState::Active;
        assert!(!quest.update_time(20.0));
        assert!(quest.is_timed_out() == false);
        assert!(quest.update_time(15.0));
        assert!(quest.is_timed_out());
    }

    #[test]
    fn test_repeatable_quest() {
        let mut quest = Quest::new("q1", "Repeatable", "Do again")
            .with_repeatable(3)
            .with_objective(Objective::new("obj1",
                ObjectiveType::Custom { description: "X".into(), target_count: 1 },
                "X"));
        quest.state = QuestState::Available;
        quest.activate();
        quest.complete_objective_by_id("obj1", 1);
        quest.complete();
        assert_eq!(quest.repeat_count, 1);
        assert!(quest.activate()); // can repeat
        assert_eq!(quest.state, QuestState::Active);
    }

    #[test]
    fn test_quest_summary() {
        let quest = Quest::new("q1", "Find the Sword", "Desc")
            .with_objective(Objective::new("obj1",
                ObjectiveType::Collect { item_id: "sword".into(), count: 3 },
                "Find swords"));
        assert_eq!(quest.summary(), "Find the Sword: Find swords [0/3]");
    }
}
