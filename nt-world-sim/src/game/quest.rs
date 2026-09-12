use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestStatus { Available, Active, Completed, Failed }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestType { Main, Side, Daily, Special, Achievement }

#[derive(Debug, Clone)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub quest_type: QuestType,
    pub giver: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<QuestReward>,
    pub status: QuestStatus,
    pub day_given: u32,
    pub day_deadline: Option<u32>,
    pub prerequisite_quests: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct QuestObjective {
    pub description: String,
    pub target_id: Option<u32>,
    pub target_count: u32,
    pub current_count: u32,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub enum QuestReward {
    Gold(u32),
    Item { item_id: u32, quantity: u32 },
    XP { skill: String, amount: u32 },
    Resonance { npc: String, amount: u32 },
    Unlock(String),
}

impl Quest {
    pub fn new(id: u32, name: &str, description: &str, quest_type: QuestType, giver: &str) -> Self {
        Self {
            id, name: name.to_string(), description: description.to_string(),
            quest_type, giver: giver.to_string(),
            objectives: Vec::new(), rewards: Vec::new(),
            status: QuestStatus::Available, day_given: 0, day_deadline: None,
            prerequisite_quests: Vec::new(),
        }
    }

    pub fn add_objective(mut self, desc: &str, target_id: Option<u32>, count: u32) -> Self {
        self.objectives.push(QuestObjective {
            description: desc.to_string(), target_id, target_count: count,
            current_count: 0, completed: false,
        });
        self
    }

    pub fn add_reward(mut self, reward: QuestReward) -> Self {
        self.rewards.push(reward);
        self
    }

    pub fn with_deadline(mut self, deadline: u32) -> Self {
        self.day_deadline = Some(deadline);
        self
    }

    pub fn with_prerequisites(mut self, prereqs: Vec<u32>) -> Self {
        self.prerequisite_quests = prereqs;
        self
    }

    pub fn update_progress(&mut self, item_id: u32, count: u32) {
        for obj in &mut self.objectives {
            if obj.target_id == Some(item_id) && !obj.completed {
                obj.current_count = (obj.current_count + count).min(obj.target_count);
                if obj.current_count >= obj.target_count {
                    obj.completed = true;
                }
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.objectives.iter().all(|o| o.completed)
    }

    pub fn accept(&mut self, day: u32) {
        self.status = QuestStatus::Active;
        self.day_given = day;
    }

    pub fn complete(&mut self) {
        if self.is_complete() {
            self.status = QuestStatus::Completed;
        }
    }

    pub fn fail(&mut self) {
        self.status = QuestStatus::Failed;
    }

    pub fn progress_summary(&self) -> String {
        let completed = self.objectives.iter().filter(|o| o.completed).count();
        let total = self.objectives.len();
        format!("{}/{} objectives completed", completed, total)
    }
}

pub struct QuestDatabase {
    pub quests: HashMap<u32, Quest>,
    pub active_quests: Vec<u32>,
    pub completed_quests: Vec<u32>,
}

impl QuestDatabase {
    pub fn new() -> Self {
        let mut db = Self { quests: HashMap::new(), active_quests: Vec::new(), completed_quests: Vec::new() };
        db.register_default_quests();
        db
    }

    fn register_default_quests(&mut self) {
        self.register(Quest::new(1, "First Steps", "Till your first plot of land", QuestType::Main, "Awareness")
            .add_objective("Till soil", Some(1), 1)
            .add_reward(QuestReward::Gold(100))
            .add_reward(QuestReward::XP { skill: "Awareness".to_string(), amount: 50 }));

        self.register(Quest::new(2, "Green Thumb", "Grow your first crop to harvest", QuestType::Main, "Awareness")
            .add_objective("Harvest crop", Some(1001), 1)
            .add_reward(QuestReward::Gold(200))
            .add_reward(QuestReward::Item { item_id: 1002, quantity: 5 })
            .with_prerequisites(vec![1]));

        self.register(Quest::new(3, "Deep Thoughts", "Reach mine floor 5", QuestType::Main, "Focus")
            .add_objective("Reach floor", Some(5000), 5)
            .add_reward(QuestReward::Gold(300))
            .add_reward(QuestReward::XP { skill: "Focus".to_string(), amount: 100 }));

        self.register(Quest::new(4, "Master of the Mine", "Reach mine floor 10", QuestType::Main, "Focus")
            .add_objective("Reach floor", Some(5000), 10)
            .add_reward(QuestReward::Gold(600))
            .add_reward(QuestReward::Item { item_id: 5001, quantity: 1 })
            .with_prerequisites(vec![3]));

        self.register(Quest::new(5, "Spirit Weaver", "Reach 1000 resonance with Empathy", QuestType::Main, "Empathy")
            .add_objective("Build resonance", None, 1000)
            .add_reward(QuestReward::Gold(800))
            .add_reward(QuestReward::Unlock("Empathy's Blessing".to_string()))
            .with_prerequisites(vec![11]));

        self.register(Quest::new(10, "Forager's Delight", "Collect 10 forest items", QuestType::Side, "Creativity")
            .add_objective("Collect items", Some(2001), 10)
            .add_reward(QuestReward::Gold(150))
            .add_reward(QuestReward::Resonance { npc: "Creativity".to_string(), amount: 20 }));

        self.register(Quest::new(11, "Community Builder", "Reach 500 resonance with any NPC", QuestType::Side, "Empathy")
            .add_objective("Build resonance", None, 500)
            .add_reward(QuestReward::Gold(500))
            .add_reward(QuestReward::Unlock("Special Dialogues".to_string()));

        self.register(Quest::new(12, "Master Crafter", "Craft 20 items", QuestType::Side, "Creativity")
            .add_objective("Craft items", Some(2001), 20)
            .add_reward(QuestReward::Gold(400))
            .add_reward(QuestReward::XP { skill: "Creativity".to_string(), amount: 150 }));

        self.register(Quest::new(13, "Monster Hunter", "Defeat 50 creatures in the Knowledge Mines", QuestType::Side, "Focus")
            .add_objective("Defeat creatures", Some(3001), 50)
            .add_reward(QuestReward::Gold(350))
            .add_reward(QuestReward::XP { skill: "Focus".to_string(), amount: 80 }));

        self.register(Quest::new(14, "Recipe Collector", "Discover 10 crafting recipes", QuestType::Side, "Creativity")
            .add_objective("Discover recipes", Some(2002), 10)
            .add_reward(QuestReward::Gold(300))
            .add_reward(QuestReward::Unlock("Recipe Book Upgrade".to_string()));

        self.register(Quest::new(15, "Nature's Friend", "Befriend all 4 NPCs to at least Friend level", QuestType::Side, "Empathy")
            .add_objective("Friend with Awareness", None, 750)
            .add_reward(QuestReward::Gold(600))
            .add_reward(QuestReward::Resonance { npc: "Empathy".to_string(), amount: 50 }));

        self.register(Quest::new(16, "Speed Farmer", "Harvest 50 crops in a single season", QuestType::Side, "Awareness")
            .add_objective("Harvest crops", Some(1001), 50)
            .add_reward(QuestReward::Gold(450))
            .add_reward(QuestReward::XP { skill: "Awareness".to_string(), amount: 120 }));

        self.register(Quest::new(17, "Gem Collector", "Find 5 rare gems in the mines", QuestType::Side, "Focus")
            .add_objective("Find gems", Some(5001), 5)
            .add_reward(QuestReward::Gold(500))
            .add_reward(QuestReward::Item { item_id: 5002, quantity: 1 }));

        self.register(Quest::new(20, "Daily Thought", "Ship 5 items today", QuestType::Daily, "Pierre")
            .add_objective("Ship items", None, 5)
            .add_reward(QuestReward::Gold(100)));

        self.register(Quest::new(21, "Mining Meditation", "Mine 15 minerals today", QuestType::Daily, "Focus")
            .add_objective("Mine minerals", None, 15)
            .add_reward(QuestReward::Gold(120))
            .add_reward(QuestReward::XP { skill: "Focus".to_string(), amount: 30 }));

        self.register(Quest::new(22, "Gardening Practice", "Water 10 crops today", QuestType::Daily, "Awareness")
            .add_objective("Water crops", None, 10)
            .add_reward(QuestReward::Gold(80))
            .add_reward(QuestReward::XP { skill: "Awareness".to_string(), amount: 20 }));

        self.register(Quest::new(23, "Social Butterfly", "Talk to 3 different NPCs today", QuestType::Daily, "Empathy")
            .add_objective("Talk to NPCs", None, 3)
            .add_reward(QuestReward::Gold(90))
            .add_reward(QuestReward::Resonance { npc: "Empathy".to_string(), amount: 10 }));

        self.register(Quest::new(24, "Crafter's Delight", "Craft 5 items today", QuestType::Daily, "Creativity")
            .add_objective("Craft items", None, 5)
            .add_reward(QuestReward::Gold(110))
            .add_reward(QuestReward::XP { skill: "Creativity".to_string(), amount: 25 }));

        self.register(Quest::new(30, "Achievement: Farmer", "Reach Farming level 10", QuestType::Achievement, "System")
            .add_objective("Reach level", Some(1001), 10)
            .add_reward(QuestReward::Gold(1000))
            .add_reward(QuestReward::Unlock("Golden Hoe".to_string())));

        self.register(Quest::new(31, "Achievement: Miner", "Reach Mining level 10", QuestType::Achievement, "System")
            .add_objective("Reach level", Some(1002), 10)
            .add_reward(QuestReward::Gold(1000))
            .add_reward(QuestReward::Unlock("Iridium Pickaxe".to_string())));

        self.register(Quest::new(32, "Achievement: Socialite", "Reach max resonance with all NPCs", QuestType::Achievement, "System")
            .add_objective("Max resonance", None, 2500)
            .add_reward(QuestReward::Gold(2000))
            .add_reward(QuestReward::Unlock("Golden Star Pendant".to_string())));

        self.register(Quest::new(33, "Achievement: Explorer", "Visit every location in the valley", QuestType::Achievement, "System")
            .add_objective("Visit locations", None, 8)
            .add_reward(QuestReward::Gold(750))
            .add_reward(QuestReward::Unlock("Explorer's Compass".to_string())));

        self.register(Quest::new(34, "Achievement: Collector", "Collect 100 unique items", QuestType::Achievement, "System")
            .add_objective("Unique items", None, 100)
            .add_reward(QuestReward::Gold(1500))
            .add_reward(QuestReward::Unlock("Collector's Bag".to_string())));

        self.register(Quest::new(40, "Season's Greeting", "Complete all seasonal events in one year", QuestType::Special, "System")
            .add_objective("Complete events", None, 8)
            .add_reward(QuestReward::Gold(3000))
            .add_reward(QuestReward::Unlock("Calendar of Seasons".to_string())));

        self.register(Quest::new(41, "The Lost Recipe", "Find the ancient recipe hidden in the mines", QuestType::Special, "Focus")
            .add_objective("Find recipe fragment", Some(5003), 3)
            .add_reward(QuestReward::Gold(1200))
            .add_reward(QuestReward::Item { item_id: 2003, quantity: 1 })
            .with_prerequisites(vec![4]));

        self.register(Quest::new(42, "Starlight Serenade", "Collect 10 stardust during Stargazing Night", QuestType::Special, "Creativity")
            .add_objective("Collect stardust", Some(6001), 10)
            .add_reward(QuestReward::Gold(800))
            .add_reward(QuestReward::XP { skill: "Creativity".to_string(), amount: 200 }));

        self.register(Quest::new(43, "Harvest King", "Win the Harvest Festival competition", QuestType::Special, "Empathy")
            .add_objective("Win competition", None, 1)
            .add_reward(QuestReward::Gold(1500))
            .add_reward(QuestReward::Unlock("Harvest Crown".to_string())));
    }

    pub fn register(&mut self, quest: Quest) {
        self.quests.insert(quest.id, quest);
    }

    pub fn get_available(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.status == QuestStatus::Available)
            .collect()
    }

    pub fn get_active(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.status == QuestStatus::Active)
            .collect()
    }

    pub fn get_completed(&self) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.status == QuestStatus::Completed)
            .collect()
    }

    pub fn get_by_type(&self, quest_type: QuestType) -> Vec<&Quest> {
        self.quests.values()
            .filter(|q| q.quest_type == quest_type)
            .collect()
    }

    pub fn accept_quest(&mut self, quest_id: u32, day: u32) -> bool {
        if let Some(quest) = self.quests.get_mut(&quest_id) {
            if quest.status == QuestStatus::Available {
                if quest.prerequisite_quests.iter().all(|&prereq| self.completed_quests.contains(&prereq)) {
                    quest.accept(day);
                    self.active_quests.push(quest_id);
                    return true;
                }
            }
        }
        false
    }

    pub fn update_progress(&mut self, item_id: u32, count: u32) -> Vec<u32> {
        let mut completed = Vec::new();
        for &quest_id in &self.active_quests {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                quest.update_progress(item_id, count);
                if quest.is_complete() {
                    quest.complete();
                    completed.push(quest_id);
                }
            }
        }
        completed
    }

    pub fn claim_reward(&mut self, quest_id: u32) -> Vec<QuestReward> {
        if let Some(quest) = self.quests.get_mut(&quest_id) {
            if quest.status == QuestStatus::Completed {
                let rewards = quest.rewards.clone();
                quest.status = QuestStatus::Failed;
                self.active_quests.retain(|&id| id != quest_id);
                self.completed_quests.push(quest_id);
                return rewards;
            }
        }
        Vec::new()
    }

    pub fn check_deadlines(&mut self, current_day: u32) -> Vec<u32> {
        let mut failed = Vec::new();
        for &quest_id in &self.active_quests {
            if let Some(quest) = self.quests.get_mut(&quest_id) {
                if let Some(deadline) = quest.day_deadline {
                    if current_day > deadline {
                        quest.fail();
                        failed.push(quest_id);
                    }
                }
            }
        }
        for &id in &failed {
            self.active_quests.retain(|&qid| qid != id);
        }
        failed
    }

    pub fn total_completed(&self) -> usize {
        self.completed_quests.len()
    }

    pub fn completion_percentage(&self) -> f32 {
        if self.quests.is_empty() { return 0.0; }
        self.completed_quests.len() as f32 / self.quests.len() as f32 * 100.0
    }
}

impl Default for QuestDatabase {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_creation() {
        let quest = Quest::new(1, "Test Quest", "Do something", QuestType::Side, "NPC")
            .add_objective("Collect items", Some(100), 5)
            .add_reward(QuestReward::Gold(100));
        assert_eq!(quest.name, "Test Quest");
        assert_eq!(quest.status, QuestStatus::Available);
        assert_eq!(quest.objectives.len(), 1);
        assert_eq!(quest.rewards.len(), 1);
    }

    #[test]
    fn test_quest_progress() {
        let mut quest = Quest::new(1, "Test", "Desc", QuestType::Side, "NPC")
            .add_objective("Collect", Some(100), 3);
        quest.update_progress(100, 2);
        assert_eq!(quest.objectives[0].current_count, 2);
        assert!(!quest.is_complete());
        quest.update_progress(100, 1);
        assert!(quest.is_complete());
    }

    #[test]
    fn test_quest_accept_and_complete() {
        let mut quest = Quest::new(1, "Test", "Desc", QuestType::Side, "NPC")
            .add_objective("Do", None, 1);
        quest.accept(1);
        assert_eq!(quest.status, QuestStatus::Active);
        quest.update_progress(0, 1);
        quest.complete();
        assert_eq!(quest.status, QuestStatus::Completed);
    }

    #[test]
    fn test_quest_database_default() {
        let db = QuestDatabase::new();
        assert!(db.quests.len() >= 10);
        let mains = db.get_by_type(QuestType::Main);
        assert!(mains.len() >= 3);
        let dailies = db.get_by_type(QuestType::Daily);
        assert!(dailies.len() >= 3);
    }

    #[test]
    fn test_quest_accept_and_update() {
        let mut db = QuestDatabase::new();
        assert!(db.accept_quest(1, 1));
        assert_eq!(db.active_quests.len(), 1);
        let completed = db.update_progress(1, 1);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0], 1);
    }

    #[test]
    fn test_quest_prerequisite() {
        let mut db = QuestDatabase::new();
        assert!(!db.accept_quest(2, 1));
        db.completed_quests.push(1);
        assert!(db.accept_quest(2, 1));
    }

    #[test]
    fn test_quest_claim_reward() {
        let mut db = QuestDatabase::new();
        db.accept_quest(1, 1);
        db.update_progress(1, 1);
        let rewards = db.claim_reward(1);
        assert!(!rewards.is_empty());
        assert_eq!(db.completed_quests.len(), 1);
    }

    #[test]
    fn test_quest_deadline() {
        let mut quest = Quest::new(1, "Timed", "Desc", QuestType::Daily, "NPC")
            .add_objective("Do", None, 1)
            .with_deadline(5);
        quest.accept(1);
        assert_eq!(quest.day_deadline, Some(5));
    }

    #[test]
    fn test_completion_percentage() {
        let mut db = QuestDatabase::new();
        let total = db.quests.len();
        assert_eq!(db.completion_percentage(), 0.0);
        db.completed_quests.push(1);
        let pct = db.completion_percentage();
        assert!(pct > 0.0 && pct <= 100.0);
    }
}
